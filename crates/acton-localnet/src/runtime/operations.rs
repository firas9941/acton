//! Durable asynchronous operations shared by HTTP and command-line clients.

use super::{Entry, Runtime};
use crate::{Error, Operation, OperationStatus, OperationStep, Status, storage};
use serde_json::Value;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub(crate) enum Action {
    Start,
    Stop,
    Delete,
    AddNode { name: String, validator: bool },
    RemoveNode { id: String, force: bool },
    Validation { id: String, enabled: bool },
    CreateSnapshot { name: Option<String> },
    RestoreSnapshot { id: String },
    DeleteSnapshot { id: String },
}

impl Action {
    const fn kind(&self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Stop => "stop",
            Self::Delete => "delete",
            Self::AddNode { .. } => "addNode",
            Self::RemoveNode { .. } => "removeNode",
            Self::Validation { enabled: true, .. } => "enterValidation",
            Self::Validation { enabled: false, .. } => "leaveValidation",
            Self::CreateSnapshot { .. } => "createSnapshot",
            Self::RestoreSnapshot { .. } => "restoreSnapshot",
            Self::DeleteSnapshot { .. } => "deleteSnapshot",
        }
    }
}

pub(super) struct Context {
    pub runtime: Runtime,
    pub entry: Arc<Entry>,
    pub operation: Operation,
    pub started: Instant,
    phase_started: Instant,
}

impl Runtime {
    pub(crate) async fn submit(&self, action: Action) -> Result<Operation, Error> {
        let admission = self.inner.admission.lock().await;
        if !*admission {
            return Err(Error::Conflict {
                code: "service_stopping",
                message: "The localnet service is stopping".to_owned(),
            });
        }

        let entry = self.entry().await?;
        let guard = Arc::clone(&entry.mutation)
            .try_lock_owned()
            .map_err(|_| Error::busy())?;
        let operation = Operation {
            id: uuid::Uuid::new_v4().to_string(),
            kind: action.kind().to_owned(),
            phase: "preparing".to_owned(),
            status: OperationStatus::Running,
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_ms: 0,
            progress: None,
            completed_steps: Vec::new(),
            error: None,
            result: None,
            log_path: entry.data_dir.join("startup.log").display().to_string(),
        };

        let mut context = Context {
            runtime: self.clone(),
            entry,
            operation: operation.clone(),
            started: Instant::now(),
            phase_started: Instant::now(),
        };
        context.publish().await?;
        tokio::spawn(async move {
            let _guard = guard;
            let result = context.execute(action).await;
            context.operation.duration_ms = context.started.elapsed().as_millis() as u64;
            match result {
                Ok(value) => {
                    context.finish_phase();
                    context.operation.status = OperationStatus::Completed;
                    "completed".clone_into(&mut context.operation.phase);
                    context.operation.result = Some(value);
                }
                Err(error) => {
                    let message = format!("{error}\nFull log: {}", context.operation.log_path);
                    context.operation.status = OperationStatus::Failed;
                    "failed".clone_into(&mut context.operation.phase);
                    context.operation.error = Some(message.clone());
                    let mut record = context.entry.record.write().await;
                    record.error = Some(message);
                    if matches!(record.status, Status::Starting | Status::Stopping) {
                        record.status = Status::Failed;
                    }
                }
            }

            if let Err(error) = context.publish().await {
                log::error!(
                    "operation={} target={} outcome=persist_failed error={error}",
                    context.operation.kind,
                    context.operation.id
                );
            }
        });

        // Transfer the deployment lock to the task before admitting shutdown.
        // Otherwise shutdown could miss an accepted operation that has not started.
        drop(admission);

        Ok(operation)
    }
}

impl Context {
    pub(super) async fn phase(&mut self, phase: &str) -> Result<(), Error> {
        if *self.runtime.inner.closing.borrow() {
            return Err(Error::Conflict {
                code: "service_stopping",
                message: "The service is stopping; the network will be shut down gracefully"
                    .to_owned(),
            });
        }

        if self.operation.phase != phase {
            self.finish_phase();
            self.operation.phase = phase.to_owned();
            self.operation.progress = None;
        }

        self.publish().await
    }

    fn finish_phase(&mut self) {
        self.operation.completed_steps.push(OperationStep {
            phase: self.operation.phase.clone(),
            duration_ms: self.phase_started.elapsed().as_millis() as u64,
        });
        self.phase_started = Instant::now();
    }

    pub(super) async fn publish(&mut self) -> Result<(), Error> {
        self.operation.duration_ms = self.started.elapsed().as_millis() as u64;
        storage::write_json(
            &self
                .runtime
                .inner
                .root
                .join("operations")
                .join(format!("{}.json", self.operation.id)),
            &self.operation,
        )
        .await?;
        self.entry.record.write().await.operation = Some(self.operation.clone());
        Runtime::save(&self.entry).await?;
        log::info!(
            "operation={} target={} phase={} duration_ms={} outcome={:?} progress={:?}",
            self.operation.kind,
            self.entry.record.read().await.id,
            self.operation.phase,
            self.operation.duration_ms,
            self.operation.status,
            self.operation.progress
        );
        Ok(())
    }

    async fn execute(&mut self, action: Action) -> Result<Value, Error> {
        self.phase("preparing").await?;

        // Stopped definitions have no Docker resources until their first start.
        // Basic lifecycle commands must remain usable without materializing them.
        {
            let mut record = self.entry.record.write().await;
            match &action {
                Action::Start if record.status == Status::Running => {
                    drop(record);
                    return Ok(Value::Null);
                }
                Action::Stop if record.status == Status::Stopped => {
                    drop(record);
                    return Ok(Value::Null);
                }
                Action::Delete if !self.entry.data_dir.join("runtime.json").exists() => {
                    record.status = Status::Deleted;
                    drop(record);
                    return Ok(Value::Null);
                }
                _ => {}
            }
        }

        let driver = self.runtime.driver(&self.entry).await?;
        match action {
            Action::Start => {
                self.start(&driver).await?;
            }
            Action::Stop | Action::Delete => {
                self.phase("stopping").await?;
                self.entry.record.write().await.status = Status::Stopping;
                if matches!(action, Action::Delete) {
                    self.observe(&driver, driver.delete()).await?;
                    self.entry.record.write().await.status = Status::Deleted;
                } else {
                    self.observe(&driver, driver.stop()).await?;
                    self.entry.record.write().await.status = Status::Stopped;
                }
            }
            Action::AddNode { name, validator } => {
                return self.add_node(&driver, name, validator).await;
            }
            Action::RemoveNode { id, force } => self.remove_node(&driver, &id, force).await?,
            Action::Validation { id, enabled } => self.validation(&driver, &id, enabled).await?,
            Action::CreateSnapshot { name } => {
                let restart = self.entry.record.read().await.status == Status::Running;
                self.phase("stopping").await?;
                self.observe(&driver, driver.stop()).await?;
                self.entry.record.write().await.status = Status::Stopped;
                self.phase("creatingArchive").await?;
                let result = driver.create_snapshot(name.as_deref()).await;
                let restarted = if restart && !*self.runtime.inner.closing.borrow() {
                    self.start(&driver).await
                } else {
                    Ok(())
                };

                return snapshot_result(result, restarted);
            }
            Action::RestoreSnapshot { id } => {
                storage::validate_id(&id)?;
                self.phase("stopping").await?;
                self.observe(&driver, driver.stop()).await?;
                self.entry.record.write().await.status = Status::Stopped;
                self.phase("restoringState").await?;
                let snapshot = driver.restore_snapshot(&id).await?;
                self.phase("resettingIndexer").await?;
                driver.reset_indexer().await?;
                self.start(&driver).await?;
                return serde_json::to_value(snapshot).map_err(|e| Error::invalid(e.to_string()));
            }
            Action::DeleteSnapshot { id } => {
                storage::validate_id(&id)?;
                self.phase("deletingArchive").await?;
                driver.delete_snapshot(&id).await?;
            }
        }

        Ok(Value::Null)
    }
}

fn snapshot_result(
    result: Result<crate::Snapshot, Error>,
    restart: Result<(), Error>,
) -> Result<Value, Error> {
    match (result, restart) {
        (Ok(snapshot), Ok(())) => {
            serde_json::to_value(snapshot).map_err(|e| Error::invalid(e.to_string()))
        }
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(restart)) => Err(Error::Internal {
            code: "snapshot_failed",
            message: format!("{error}; restart also failed: {restart}"),
        }),
    }
}
