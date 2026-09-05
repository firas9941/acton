//! Startup readiness and graceful service shutdown.

use super::{Context, Runtime};
use crate::{Error, OperationProgress, Status, docker::DockerNetwork};
use std::time::{Duration, Instant};
use tokio::process::Child;

impl Context {
    pub(super) async fn start(&mut self, driver: &DockerNetwork) -> Result<(), Error> {
        self.entry.record.write().await.status = Status::Starting;
        self.phase("checkingImage").await?;
        let present = self
            .wait_child(
                driver,
                driver.spawn_image_inspect()?,
                Duration::from_secs(15),
            )
            .await?;
        if !present.success() {
            self.phase("pullingImage").await?;
            let isolated = driver.isolated_pull_target().await.ok().flatten();
            let child = match &isolated {
                Some(target) => driver.spawn_isolated_pull(target)?,
                None => driver.spawn_normal_pull()?,
            };

            let mut status = self
                .wait_child(driver, child, Duration::from_secs(1800))
                .await?;
            if !status.success() && isolated.is_some() {
                status = self
                    .wait_child(
                        driver,
                        driver.spawn_normal_pull()?,
                        Duration::from_secs(1800),
                    )
                    .await?;
            }

            if !status.success() {
                return Err(Error::Internal {
                    code: "image_pull_failed",
                    message: driver.startup_failure_message("pull image", status).await,
                });
            }
        }

        self.phase("startingContainers").await?;
        let result = self.start_containers(driver).await;
        if let Err(error) = result {
            return match driver.stop().await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(Error::Internal {
                    code: "startup_cleanup_failed",
                    message: format!("{error}; graceful cleanup also failed: {cleanup}"),
                }),
            };
        }

        let mut record = self.entry.record.write().await;
        record.status = Status::Running;
        record.error = None;
        drop(record);
        Ok(())
    }

    async fn start_containers(&mut self, driver: &DockerNetwork) -> Result<(), Error> {
        let status = self
            .wait_child(driver, driver.spawn_compose_up()?, Duration::from_secs(660))
            .await?;
        if !status.success() {
            return Err(Error::Internal {
                code: "network_start_failed",
                message: driver
                    .startup_failure_message("start network", status)
                    .await,
            });
        }

        self.phase("waitingForApis").await?;
        let endpoints = self.entry.record.read().await.endpoints.clone();
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| Error::invalid(e.to_string()))?;
        let urls = [
            ("API v2", format!("{}/getMasterchainInfo", endpoints.api_v2)),
            ("API v3", format!("{}/masterchainInfo", endpoints.api_v3)),
            (
                "Indexer",
                format!(
                    "{}/healthcheck",
                    endpoints.api_v3.trim_end_matches("/api/v3")
                ),
            ),
        ];
        let deadline = Instant::now() + Duration::from_secs(180);
        let mut closing = self.runtime.inner.closing.subscribe();
        let mut ready = [false; 3];

        loop {
            let checks = async {
                for (index, (_, url)) in urls.iter().enumerate() {
                    let response = client.get(url).send().await;
                    ready[index] =
                        matches!(response, Ok(response) if response.status().is_success());
                    let waiting = urls
                        .iter()
                        .zip(ready)
                        .filter_map(|((name, _), ready)| (!ready).then_some(*name))
                        .collect::<Vec<_>>();
                    self.progress(OperationProgress {
                        completed: ready.iter().filter(|ready| **ready).count() as u64,
                        total: Some(urls.len() as u64),
                        unit: "checks passed".to_owned(),
                        detail: if waiting.is_empty() {
                            "TON APIs and indexer ready".to_owned()
                        } else {
                            format!("Waiting for {}", waiting.join(", "))
                        },
                    })
                    .await?;
                }

                Ok::<_, Error>(ready.into_iter().all(|ready| ready))
            };
            let ready = tokio::select! {
                result = checks => result?,
                _ = async { if !*closing.borrow() { let _ = closing.changed().await; } } => {
                    return Err(Error::Conflict { code: "service_stopping", message: "Readiness checks interrupted by graceful service shutdown".to_owned() });
                }
            };

            if ready {
                return Ok(());
            }

            if Instant::now() >= deadline {
                return Err(Error::Internal {
                    code: "readiness_timeout",
                    message: "TON APIs or the indexer did not become ready within 180 seconds"
                        .to_owned(),
                });
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    async fn wait_child(
        &mut self,
        driver: &DockerNetwork,
        mut child: Child,
        duration: Duration,
    ) -> Result<std::process::ExitStatus, Error> {
        let mut closing = self.runtime.inner.closing.subscribe();
        let wait = async {
            tokio::select! {
                result = tokio::time::timeout(duration, child.wait()) => match result {
                    Ok(Ok(status)) => Ok(status),
                    Ok(Err(error)) => Err(Error::Internal { code: "process_wait_failed", message: error.to_string() }),
                    Err(_) => Err(Error::Internal { code: "process_timeout", message: format!("Docker operation exceeded {} seconds", duration.as_secs()) }),
                },
                _ = async { if !*closing.borrow() { let _ = closing.changed().await; } } => Err(Error::Conflict { code: "service_stopping", message: "Startup interrupted by graceful service shutdown".to_owned() }),
            }
        };
        let result = self.observe(driver, wait).await;

        if result.is_err() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        result
    }
}

impl Runtime {
    /// Refreshes cached status for idle networks. Operation progress remains owned
    /// by the active task; this method never overwrites an in-flight transition.
    pub async fn reconcile(&self) {
        let entry = &self.inner.entry;
        let Ok(_guard) = entry.mutation.try_lock() else {
            return;
        };

        if entry.record.read().await.status == Status::Deleted {
            return;
        }

        if !entry.data_dir.join("runtime.json").exists() {
            entry.record.write().await.status = Status::Stopped;
            return;
        }

        let result = match self.driver(entry).await {
            Ok(driver) => driver.status().await,
            Err(error) => Err(error),
        };
        {
            let mut record = entry.record.write().await;
            match result {
                Ok(status) => record.status = status,
                Err(error) => {
                    record.status = Status::Unknown;
                    record.error = Some(error.to_string());
                }
            }
        }

        if let Err(error) = Self::save(entry).await {
            log::error!("operation=reconcile outcome=failed error={error}");
        }
    }

    /// Stops this network while retaining its volumes. Startup processes are
    /// interrupted; snapshot writes reach a safe boundary before Docker stops.
    pub async fn shutdown(&self) -> Result<(), Error> {
        self.prepare_shutdown().await?;
        let entry = &self.inner.entry;
        let _guard = entry.mutation.lock().await;
        if entry.record.read().await.status == Status::Deleted {
            return Ok(());
        }
        if !entry.data_dir.join("runtime.json").exists() {
            entry.record.write().await.status = Status::Stopped;
            return Self::save(entry).await;
        }

        let started = Instant::now();
        let id = entry.record.read().await.id.clone();
        log::info!("operation=shutdown target={id} duration_ms=0 outcome=running");
        entry.record.write().await.status = Status::Stopping;
        let result = match self.driver(entry).await {
            Ok(driver) => driver.stop().await,
            Err(error) => Err(error),
        };
        {
            let mut record = entry.record.write().await;
            record.status = if result.is_ok() {
                Status::Stopped
            } else {
                Status::Failed
            };
            if let Err(error) = &result {
                record.error = Some(error.to_string());
            }
        }
        Self::save(entry).await?;
        log::info!(
            "operation=shutdown target={id} duration_ms={} outcome={}",
            started.elapsed().as_millis(),
            if result.is_ok() { "success" } else { "failed" }
        );
        result
    }
}
