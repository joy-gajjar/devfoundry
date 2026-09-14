use serde::Serialize;
use std::{net::TcpListener, process::Stdio, sync::Arc, time::Duration};
use tauri::{Manager, State};
use thiserror::Error;
use tokio::{
    process::{Child, Command},
    time::{sleep, timeout},
};

const HEALTH_TIMEOUT: Duration = Duration::from_secs(10);
const HEALTH_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Error)]
enum BackendError {
    #[error("could not reserve a local port")]
    Port,
    #[error("backend failed to start")]
    Spawn(#[source] std::io::Error),
    #[error("backend health check timed out")]
    Timeout,
}

#[derive(Debug, Serialize)]
struct BackendStatus {
    base_url: String,
}

struct BackendProcess {
    child: Child,
    status: BackendStatus,
}

impl BackendProcess {
    async fn start() -> Result<Self, BackendError> {
        let port = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|_| BackendError::Port)?
            .local_addr()
            .map_err(|_| BackendError::Port)?
            .port();
        let base_url = format!("http://127.0.0.1:{port}");
        let child = Command::new("devfoundry")
            .args([
                "serve",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--database",
                ".devfoundry.db",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(BackendError::Spawn)?;
        let status = BackendStatus { base_url };
        let health_url = format!("{}/health", status.base_url);
        let ready = timeout(HEALTH_TIMEOUT, async {
            loop {
                if reqwest::get(&health_url).await.is_ok() {
                    break true;
                }
                sleep(HEALTH_INTERVAL).await;
            }
        })
        .await
        .map_err(|_| BackendError::Timeout)?;
        if !ready {
            return Err(BackendError::Timeout);
        }
        Ok(Self { child, status })
    }

    async fn stop(&mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
    }
}

#[tauri::command]
fn backend_status(
    state: State<'_, Arc<tauri::async_runtime::Mutex<Option<BackendProcess>>>>,
) -> Option<BackendStatus> {
    state.blocking_lock().as_ref().map(|backend| BackendStatus {
        base_url: backend.status.base_url.clone(),
    })
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    let parsed = url::Url::parse(&url).map_err(|_| "invalid URL")?;
    if !matches!(parsed.scheme(), "https" | "http") {
        return Err("unsupported URL scheme".into());
    }
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .map_err(|_| "could not open URL".into())
        .map(|_| ())
}

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(tauri::async_runtime::Mutex::new(
            None::<BackendProcess>,
        )))
        .setup(|app| {
            let state = app
                .state::<Arc<tauri::async_runtime::Mutex<Option<BackendProcess>>>>()
                .inner()
                .clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(backend) = BackendProcess::start().await {
                    *state.lock().await = Some(backend);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![backend_status, open_external])
        .build(tauri::generate_context!())
        .expect("error while building DevFoundry desktop")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                let state = app
                    .state::<Arc<tauri::async_runtime::Mutex<Option<BackendProcess>>>>()
                    .inner()
                    .clone();
                tauri::async_runtime::block_on(async move {
                    if let Some(mut backend) = state.lock().await.take() {
                        backend.stop().await;
                    }
                });
            }
        });
}
