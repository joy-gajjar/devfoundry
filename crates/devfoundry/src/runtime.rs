use devfoundry_server::{ApiSecurity, ExecutionRegistry, ServerState, router_with_security};
use devfoundry_storage::SqliteStore;
use devfoundry_tools::ToolRegistry;
use futures_util::StreamExt;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;

const SMOKE_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_SMOKE_EVENTS: usize = 256;
const MAX_SMOKE_TEXT_CHARS: usize = 4_096;

fn provider_from_config(
    config: &crate::config::Config,
) -> Result<Arc<dyn devfoundry_llm::LlmProvider>, Box<dyn std::error::Error>> {
    match config.provider.kind.as_str() {
        "github-copilot" => {
            let token = config
                .provider
                .token_env
                .as_deref()
                .and_then(std::env::var_os)
                .map(|value| value.to_string_lossy().into_owned());
            if token.is_none() {
                return Err(format!(
                    "missing GitHub Copilot token environment variable: {}",
                    config.provider.token_env.as_deref().unwrap_or("<unset>")
                )
                .into());
            }
            Ok(Arc::new(devfoundry_llm::GithubCopilotProvider::new(
                config.provider.base_url.clone(),
                token,
            )))
        }
        kind => Err(format!("unsupported provider kind: {kind}").into()),
    }
}

pub async fn copilot_smoke(
    config: crate::config::Config,
    model: String,
    json: bool,
    mock: bool,
) -> Result<(), String> {
    validate_smoke_inputs(&config, &model)?;
    let provider: Arc<dyn devfoundry_llm::LlmProvider> = if mock {
        Arc::new(MockSmokeProvider)
    } else {
        let token_env = config.provider.token_env.as_deref().unwrap_or("<unset>");
        match std::env::var_os(token_env) {
            None => return Err(format!("missing token environment variable: {token_env}")),
            Some(value) if value.is_empty() || value.to_string_lossy().trim().is_empty() => {
                return Err(format!("token environment variable is empty: {token_env}"));
            }
            Some(_) => {}
        }
        provider_from_config(&config).map_err(|_| "provider initialization failed".to_owned())?
    };
    let request = devfoundry_llm::LlmRequest {
        model,
        messages: vec![devfoundry_llm::LlmMessage {
            role: "user".into(),
            content: "Reply with the single word: ok".into(),
        }],
        tools: Vec::new(),
    };
    tokio::time::timeout(SMOKE_TIMEOUT, async move {
        let mut stream = provider
            .stream(request, tokio_util::sync::CancellationToken::new())
            .await
            .map_err(|error| safe_provider_error(&error))?;
        let mut text_chars = 0usize;
        let mut event_count = 0usize;
        let mut finished = 0usize;
        while let Some(event) = stream.next().await {
            let event = event.map_err(|error| safe_provider_error(&error))?;
            event_count += 1;
            if event_count > MAX_SMOKE_EVENTS {
                return Err(format!(
                    "provider stream exceeded the {MAX_SMOKE_EVENTS}-event smoke limit"
                ));
            }
            match event {
                devfoundry_llm::LlmEvent::TextDelta { text } => {
                    text_chars += text.chars().count();
                    if text_chars > MAX_SMOKE_TEXT_CHARS {
                        return Err(format!(
                            "provider output exceeded the {MAX_SMOKE_TEXT_CHARS}-character smoke limit"
                        ));
                    }
                }
                devfoundry_llm::LlmEvent::Finished { .. } => finished += 1,
                devfoundry_llm::LlmEvent::ReasoningDelta { .. }
                | devfoundry_llm::LlmEvent::ToolCall { .. }
                | devfoundry_llm::LlmEvent::Usage { .. } => {}
            }
        }
        if finished == 0 {
            return Err("provider stream did not finish".into());
        }
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "event_count": event_count,
                    "finished_events": finished,
                    "text_chars": text_chars
                })
            );
        } else {
            println!("status=ok");
            println!("event_count={event_count}");
            println!("finished_events={finished}");
            println!("text_chars={text_chars}");
        }
        Ok::<(), String>(())
    })
    .await
    .map_err(|_| "provider smoke test timed out after 30 seconds".to_owned())?
}

struct MockSmokeProvider;

#[async_trait::async_trait]
impl devfoundry_llm::LlmProvider for MockSmokeProvider {
    async fn stream(
        &self,
        _request: devfoundry_llm::LlmRequest,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> Result<devfoundry_llm::LlmStream, devfoundry_llm::LlmError> {
        if cancellation.is_cancelled() {
            return Err(devfoundry_llm::LlmError::Cancelled);
        }
        Ok(Box::pin(futures_util::stream::iter([
            Ok(devfoundry_llm::LlmEvent::TextDelta { text: "ok".into() }),
            Ok(devfoundry_llm::LlmEvent::Finished {
                reason: "stop".into(),
            }),
        ])))
    }
}

fn validate_smoke_inputs(config: &crate::config::Config, model: &str) -> Result<(), String> {
    if config.provider.kind != "github-copilot" {
        return Err(format!(
            "unsupported provider kind: {}; expected github-copilot",
            config.provider.kind
        ));
    }
    let base_url = reqwest::Url::parse(&config.provider.base_url)
        .map_err(|_| "invalid provider base URL; use an http or https URL".to_owned())?;
    if !matches!(base_url.scheme(), "http" | "https") || base_url.host_str().is_none() {
        return Err("invalid provider base URL; use an http or https URL with a host".into());
    }
    if !base_url.username().is_empty() || base_url.password().is_some() {
        return Err("invalid provider base URL; userinfo is not allowed".into());
    }
    if model.trim().is_empty() {
        return Err("model must not be empty".into());
    }
    if model.chars().any(char::is_control) {
        return Err("model must not contain control characters".into());
    }
    if model.chars().count() > 128 {
        return Err("model must be at most 128 characters".into());
    }
    let token_env = config
        .provider
        .token_env
        .as_deref()
        .ok_or_else(|| "provider token_env must be configured".to_owned())?;
    if token_env.is_empty()
        || token_env.chars().any(char::is_control)
        || !token_env
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
        || token_env
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
    {
        return Err("provider token_env must be a valid environment variable name".into());
    }
    Ok(())
}

fn safe_provider_error(error: &devfoundry_llm::LlmError) -> String {
    match error {
        devfoundry_llm::LlmError::Classified { classification, .. } => match classification {
            devfoundry_llm::ProviderErrorClass::Authentication => {
                "authentication failed; check the GitHub Copilot token and credential handoff"
                    .to_owned()
            }
            devfoundry_llm::ProviderErrorClass::Transient => {
                "network request failed; check connectivity and the provider base URL".into()
            }
            devfoundry_llm::ProviderErrorClass::RateLimited => {
                "provider rate limit reached; retry later".into()
            }
            devfoundry_llm::ProviderErrorClass::ContentFiltered => {
                "provider rejected the smoke prompt as content-filtered".into()
            }
            devfoundry_llm::ProviderErrorClass::Permanent => {
                "provider rejected the smoke request".into()
            }
        },
        devfoundry_llm::LlmError::Request(_) => {
            "network request failed; check connectivity and the provider base URL".into()
        }
        devfoundry_llm::LlmError::InvalidData(_) => "provider returned invalid data".into(),
        devfoundry_llm::LlmError::Cancelled => "provider request cancelled".into(),
    }
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod tests {
    use super::*;

    fn config() -> crate::config::Config {
        crate::config::Config::default()
    }

    #[test]
    fn smoke_validation_rejects_unsafe_inputs() {
        let mut config = config();
        config.provider.base_url = "file:///tmp/token".into();
        assert_eq!(
            validate_smoke_inputs(&config, "gpt-4o-mini").unwrap_err(),
            "invalid provider base URL; use an http or https URL with a host"
        );

        config.provider.base_url = "https://copilot.example.test".into();
        assert_eq!(
            validate_smoke_inputs(&config, " ").unwrap_err(),
            "model must not be empty"
        );
        config.provider.token_env = Some("TOKEN-NAME".into());
        assert_eq!(
            validate_smoke_inputs(&config, "gpt-4o-mini").unwrap_err(),
            "provider token_env must be a valid environment variable name"
        );
    }

    #[test]
    fn provider_errors_are_redacted_and_classified() {
        let error = devfoundry_llm::LlmError::Classified {
            classification: devfoundry_llm::ProviderErrorClass::Authentication,
            message: "Bearer secret-value response body".into(),
        };
        let safe = safe_provider_error(&error);
        assert!(safe.contains("authentication"));
        assert!(!safe.contains("secret-value"));
        assert!(!safe.contains("response body"));
    }

    #[tokio::test]
    async fn mock_smoke_does_not_require_or_read_a_token() {
        let mut config = config();
        config.provider.token_env = Some("COPILOT_SMOKE_TOKEN_MUST_NOT_BE_READ".into());
        copilot_smoke(config, "gpt-4o-mini".into(), true, true)
            .await
            .unwrap();
    }
}

async fn build_app(
    database_url: &str,
    security: ApiSecurity,
) -> Result<(axum::Router, Arc<ExecutionRegistry>), Box<dyn std::error::Error>> {
    let database_url = if database_url.starts_with("sqlite:") {
        database_url.to_owned()
    } else {
        let path = std::path::PathBuf::from(database_url);
        let path = if path.is_absolute() {
            path
        } else {
            std::env::current_dir()?.join(path)
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let store = Arc::new(SqliteStore::connect_path(&path).await?);
        return build_app_with_store(store, security).await;
    };
    let store = Arc::new(SqliteStore::connect(&database_url).await?);
    build_app_with_store(store, security).await
}

async fn build_app_with_store(
    store: Arc<SqliteStore>,
    security: ApiSecurity,
) -> Result<(axum::Router, Arc<ExecutionRegistry>), Box<dyn std::error::Error>> {
    store.recover_interrupted_work().await?;
    let config = crate::config::load_from_root(&std::env::current_dir()?)?;
    let provider = provider_from_config(&config)?;
    let registry = ToolRegistry::standard();
    let tools = Arc::new(registry);
    let runner = Arc::new(devfoundry_core::SessionRunner::new(
        store.clone(),
        provider,
        tools,
    ));
    let executions = Arc::new(ExecutionRegistry::default());
    let app = router_with_security(
        ServerState {
            store,
            runner,
            executions: executions.clone(),
        },
        security,
    );
    Ok((app, executions))
}

pub async fn start_server(
    database_url: &str,
    bind: &str,
) -> Result<(String, JoinHandle<()>), Box<dyn std::error::Error>> {
    let bind_address: SocketAddr = bind.parse().map_err(
        |_| "bind must be an IP address and port; use 127.0.0.1:4096 for the local default",
    )?;
    let security = api_security(bind_address)?;
    let (app, executions) = build_app(database_url, security).await?;
    let listener = tokio::net::TcpListener::bind(bind).await?;
    let address = listener.local_addr()?;
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown_signal().await;
                executions.shutdown();
            })
            .await;
    });
    Ok((format!("http://{address}"), task))
}

fn api_security(bind: SocketAddr) -> Result<ApiSecurity, Box<dyn std::error::Error>> {
    if bind.ip().is_loopback() {
        return Ok(ApiSecurity::default());
    }

    let token_env = std::env::var("DEVFOUNDRY_API_TOKEN_ENV")
        .unwrap_or_else(|_| "DEVFOUNDRY_API_TOKEN".to_owned());
    let token = std::env::var(&token_env).map_err(|_| {
        format!("non-loopback binding requires API token environment variable: {token_env}")
    })?;
    if token.trim().is_empty() {
        return Err(format!("API token environment variable is empty: {token_env}").into());
    }
    let origins = std::env::var("DEVFOUNDRY_ALLOWED_ORIGINS").map_err(|_| {
        "non-loopback binding requires DEVFOUNDRY_ALLOWED_ORIGINS with one or more exact origins"
    })?;
    let allowed_origins = origins
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(|origin| origin.parse::<axum::http::HeaderValue>())
        .collect::<Result<Vec<_>, _>>()?;
    if allowed_origins.is_empty() {
        return Err("DEVFOUNDRY_ALLOWED_ORIGINS must contain at least one exact origin".into());
    }
    Ok(ApiSecurity {
        bearer_token: Some(token),
        allowed_origins,
    })
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut terminate = signal(SignalKind::terminate()).ok();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = async {
                terminate
                    .as_mut()
                    .expect("SIGTERM handler must be available")
                    .recv()
                    .await
            } => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

pub async fn serve(database_url: &str, bind: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (url, task) = start_server(database_url, bind).await?;
    println!("server_listening={url}");
    task.await?;
    Ok(())
}

pub async fn interactive(root: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let database = root.join(".devfoundry.db");
    let (url, server) = start_server(&database.to_string_lossy(), "127.0.0.1:0").await?;
    let client = devfoundry_tui::client::ApiClient::new(url);
    let _project = if let Some(project) = client
        .list_projects()
        .await?
        .into_iter()
        .find(|project| project.root == root.to_string_lossy())
    {
        project
    } else {
        client
            .create_project(&devfoundry_protocol::CreateProjectRequest {
                root: root.to_string_lossy().into_owned(),
                name: None,
            })
            .await?
    };
    let result =
        devfoundry_tui::run_project_session(client, root.to_string_lossy().into_owned()).await;
    server.abort();
    result?;
    Ok(())
}
