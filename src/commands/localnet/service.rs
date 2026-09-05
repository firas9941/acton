//! Foreground ownership and startup discovery for the control service.

use acton_localnet::{catalog::NetworkDirectory, client::Client};
use anyhow::Context;
use std::{
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::process::{Child, Command};

use super::{output, progress::label};

pub(super) async fn serve(root: &Path, port: u16, json: bool) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .context("Failed to bind the localnet control API")?;
    if !json {
        eprintln!(
            "{} the localnet control API on http://{}",
            label("Starting", false),
            listener.local_addr()?
        );
    }

    let (send, receive) = tokio::sync::oneshot::channel();
    let serving = acton_localnet::http::serve(root, listener, async {
        let _ = receive.await;
    });
    tokio::pin!(serving);

    tokio::select! {
        result = &mut serving => {
            result?;
            if !json {
                eprintln!("{} Acton localnet gracefully", label("Stopped", false));
            }
        }
        _ = shutdown_signal() => {
            let _ = send.send(());
            output::shutdown(json, true, async { serving.await.map_err(Into::into) }).await?;
        }
    }

    Ok(())
}

/// Starts only the service belonging to this state directory. The child is kept
/// by the foreground command so Ctrl-C can request graceful HTTP shutdown.
pub(super) async fn connect_or_start(
    catalog_root: &Path,
    location: NetworkDirectory,
) -> anyhow::Result<(Client, Option<Child>)> {
    if let Ok(client) = Client::connect(&location.path).await {
        return Ok((client, None));
    }

    let location = location.prepare(catalog_root).await?;
    let root = &location.path;
    let log_path = root.join("service.log");
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("--project-root")
        .arg(acton_config::config::project_root())
        .args(["localnet", "--state-dir"])
        .arg(catalog_root)
        .arg("serve")
        .arg(&location.network.id)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log));

    // An independent process group prevents the terminal from interrupting the
    // child before the owning client sends the graceful shutdown request.
    #[cfg(unix)]
    command.process_group(0);

    let mut child = command
        .spawn()
        .context("Failed to start the localnet service")?;
    let deadline = Instant::now() + Duration::from_secs(20);

    loop {
        if let Ok(client) = Client::connect(root).await {
            if child.id() == Some(client.service_pid()) {
                return Ok((client, Some(child)));
            }

            // This child lost the lock. Reap it before adopting the winner, and
            // never make this command the owner of somebody else's service.
            let _ = child.wait().await;
            return Ok((client, None));
        }

        if let Some(status) = child.try_wait()? {
            // Concurrent callers may lose the filesystem lock to another service.
            // Only a verified descriptor permits adopting the winner.
            if let Ok(client) = Client::connect(root).await {
                return Ok((client, None));
            }
            anyhow::bail!(
                "Localnet service exited with {status}; full log: {}",
                log_path.display()
            );
        }

        if Instant::now() >= deadline {
            terminate_owned(&mut child).await?;
            anyhow::bail!(
                "Localnet service did not become ready; full log: {}",
                log_path.display()
            );
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub(super) async fn stop_owned(client: &Client, child: &mut Child) -> anyhow::Result<()> {
    let result = client.shutdown().await;
    if result.is_err() && child.try_wait()?.is_none() {
        terminate_owned(child).await?;
    }

    let status = child.wait().await?;
    result?;
    anyhow::ensure!(
        status.success(),
        "Localnet service could not stop cleanly ({status}); inspect service.log"
    );
    Ok(())
}

/// The PID comes directly from our Child handle. SIGTERM runs the same graceful
/// service shutdown as HTTP when discovery or the listener has failed.
async fn terminate_owned(child: &mut Child) -> anyhow::Result<()> {
    if let Some(pid) = child.id() {
        #[cfg(unix)]
        {
            let status = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status()
                .await?;
            anyhow::ensure!(
                status.success(),
                "Failed to signal the owned localnet service"
            );
        }

        #[cfg(not(unix))]
        child.start_kill()?;
    }

    child.wait().await?;
    Ok(())
}

pub(super) async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate.recv() => {},
            }
            return;
        }
    }

    let _ = ctrl_c.await;
}
