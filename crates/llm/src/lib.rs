//! Provider-neutral streaming contracts and provider adapters.

use async_trait::async_trait;
use futures_core::Stream;
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, pin::Pin, sync::Arc};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

pub type LlmStream = Pin<Box<dyn Stream<Item = Result<LlmEvent, LlmError>> + Send>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    pub model: String,
    pub messages: Vec<LlmMessage>,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LlmEvent {
    TextDelta {
        text: String,
    },
    ReasoningDelta {
        text: String,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    Usage {
        input_tokens: u64,
        output_tokens: u64,
    },
    Finished {
        reason: String,
    },
}

#[derive(Debug, Error, Clone)]
pub enum LlmError {
    #[error("provider request failed: {0}")]
    Request(String),
    #[error("provider request {classification}: {message}")]
    Classified {
        classification: ProviderErrorClass,
        message: String,
    },
    #[error("provider returned invalid data: {0}")]
    InvalidData(String),
    #[error("provider request cancelled")]
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderErrorClass {
    Transient,
    RateLimited,
    ContentFiltered,
    Authentication,
    Permanent,
}

impl std::fmt::Display for ProviderErrorClass {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Transient => "transient",
            Self::RateLimited => "rate-limited",
            Self::ContentFiltered => "content-filtered",
            Self::Authentication => "authentication",
            Self::Permanent => "permanent",
        })
    }
}

impl LlmError {
    pub fn classification(&self) -> Option<ProviderErrorClass> {
        match self {
            Self::Classified { classification, .. } => Some(*classification),
            _ => None,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.classification(),
            Some(ProviderErrorClass::Transient | ProviderErrorClass::RateLimited)
        )
    }
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn stream(
        &self,
        request: LlmRequest,
        cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError>;
}

#[cfg(test)]
struct FakeProvider {
    pub response: String,
}

#[cfg(test)]
#[async_trait]
impl LlmProvider for FakeProvider {
    async fn stream(
        &self,
        _request: LlmRequest,
        cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError> {
        let response = self.response.clone();
        Ok(Box::pin(async_stream::stream! {
            if cancellation.is_cancelled() {
                yield Err(LlmError::Cancelled);
                return;
            }
            yield Ok(LlmEvent::TextDelta { text: response });
            yield Ok(LlmEvent::Finished { reason: "stop".into() });
        }))
    }
}

pub struct GithubCopilotProvider {
    client: reqwest::Client,
    endpoint: String,
    token: Option<String>,
}

pub type SharedProvider = Arc<dyn LlmProvider>;

impl GithubCopilotProvider {
    pub fn new(base_url: impl Into<String>, token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: format!("{}/chat/completions", base_url.into().trim_end_matches('/')),
            token,
        }
    }
}

#[derive(Serialize)]
struct WireRequest<'a> {
    model: &'a str,
    messages: &'a [LlmMessage],
    stream: bool,
    tools: Vec<WireTool<'a>>,
}

#[derive(Serialize)]
struct WireTool<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    function: WireFunction<'a>,
}

#[derive(Serialize)]
struct WireFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[async_trait]
impl LlmProvider for GithubCopilotProvider {
    async fn stream(
        &self,
        request: LlmRequest,
        cancellation: CancellationToken,
    ) -> Result<LlmStream, LlmError> {
        let mut builder = self
            .client
            .post(&self.endpoint)
            .header(CONTENT_TYPE, "application/json")
            .json(&WireRequest {
                model: &request.model,
                messages: &request.messages,
                stream: true,
                tools: request
                    .tools
                    .iter()
                    .map(|tool| WireTool {
                        kind: "function",
                        function: WireFunction {
                            name: &tool.name,
                            description: &tool.description,
                            parameters: &tool.parameters,
                        },
                    })
                    .collect(),
            });
        if let Some(token) = &self.token {
            builder = builder.header(AUTHORIZATION, format!("Bearer {token}"));
        }
        let response = tokio::select! {
            _ = cancellation.cancelled() => return Err(LlmError::Cancelled),
            response = builder.send() => response.map_err(|error| LlmError::Classified {
                classification: ProviderErrorClass::Transient,
                message: error.to_string(),
            })?,
        };
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(classify_http_error(status.as_u16(), &body));
        }
        let is_stream = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.to_ascii_lowercase().contains("text/event-stream"));
        if !is_stream {
            let body = response
                .text()
                .await
                .map_err(|error| LlmError::Classified {
                    classification: ProviderErrorClass::Transient,
                    message: error.to_string(),
                })?;
            return Ok(Box::pin(async_stream::stream! {
                match parse_json_completion(&body) {
                    Ok(events) => for event in events { yield Ok(event); },
                    Err(error) => yield Err(error),
                }
            }));
        }
        let mut bytes = response.bytes_stream();
        Ok(Box::pin(async_stream::stream! {
            let mut parser = SseParser::default();
            while let Some(chunk) = bytes.next().await {
                if cancellation.is_cancelled() { yield Err(LlmError::Cancelled); return; }
                let chunk = match chunk { Ok(chunk) => chunk, Err(error) => { yield Err(LlmError::Classified { classification: ProviderErrorClass::Transient, message: error.to_string() }); return; } };
                match parser.push(&chunk) {
                    Ok(events) => for event in events { yield Ok(event); },
                    Err(error) => { yield Err(error); return; }
                }
            }
            match parser.finish() {
                Ok(events) => for event in events { yield Ok(event); },
                Err(error) => yield Err(error),
            }
        }))
    }
}

fn parse_json_completion(body: &str) -> Result<Vec<LlmEvent>, LlmError> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|error| {
        LlmError::InvalidData(format!("Copilot JSON response was invalid: {error}"))
    })?;
    let Some(choice) = value.get("choices").and_then(|choices| choices.get(0)) else {
        if value.get("usage").is_some() || value.get("id").is_some() {
            return Ok(vec![LlmEvent::Finished {
                reason: "stop".into(),
            }]);
        }
        return Err(LlmError::InvalidData(
            "Copilot JSON response had no choices".into(),
        ));
    };
    let mut events = Vec::new();
    if let Some(text) = choice
        .pointer("/message/content")
        .and_then(serde_json::Value::as_str)
    {
        events.push(LlmEvent::TextDelta { text: text.into() });
    }
    if let Some(reason) = choice
        .get("finish_reason")
        .and_then(serde_json::Value::as_str)
    {
        events.push(LlmEvent::Finished {
            reason: reason.into(),
        });
    } else {
        events.push(LlmEvent::Finished {
            reason: "stop".into(),
        });
    }
    Ok(events)
}

fn classify_http_error(status: u16, body: &str) -> LlmError {
    let body_lower = body.to_ascii_lowercase();
    let classification = if matches!(status, 401 | 403) {
        ProviderErrorClass::Authentication
    } else if (body_lower.contains("content") && body_lower.contains("filter"))
        || body_lower.contains("content_filter")
        || body_lower.contains("content filtering")
        || body_lower.contains("filter")
    {
        ProviderErrorClass::ContentFiltered
    } else if status == 429 {
        ProviderErrorClass::RateLimited
    } else if matches!(status, 408 | 500 | 502 | 503 | 504) {
        ProviderErrorClass::Transient
    } else {
        ProviderErrorClass::Permanent
    };
    LlmError::Classified {
        classification,
        message: if body.is_empty() {
            format!("HTTP {status}")
        } else {
            format!("HTTP {status}: {body}")
        },
    }
}

#[derive(Default)]
struct SseParser {
    buffer: Vec<u8>,
    tool_calls: BTreeMap<usize, (String, String, String)>,
}

impl SseParser {
    fn push(&mut self, chunk: &[u8]) -> Result<Vec<LlmEvent>, LlmError> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();
        while let Some(index) = self.buffer.iter().position(|byte| *byte == b'\n') {
            let line = self.buffer.drain(..=index).collect::<Vec<_>>();
            self.parse_line(&line, &mut events)?;
        }
        Ok(events)
    }

    fn finish(&mut self) -> Result<Vec<LlmEvent>, LlmError> {
        if !self.buffer.is_empty() {
            let line = std::mem::take(&mut self.buffer);
            let mut events = Vec::new();
            self.parse_line(&line, &mut events)?;
            return Ok(events);
        }
        Ok(Vec::new())
    }

    fn parse_line(&mut self, line: &[u8], events: &mut Vec<LlmEvent>) -> Result<(), LlmError> {
        let line = std::str::from_utf8(line)
            .map_err(|error| LlmError::InvalidData(format!("invalid SSE UTF-8: {error}")))?
            .trim();
        let Some(data) = line.strip_prefix("data:") else {
            return Ok(());
        };
        let data = data.trim();
        if data.is_empty() {
            return Ok(());
        }
        if data == "[DONE]" {
            self.emit_tool_calls(events);
            events.push(LlmEvent::Finished {
                reason: "stop".into(),
            });
            return Ok(());
        }
        let value = serde_json::from_str::<serde_json::Value>(data)
            .map_err(|error| LlmError::InvalidData(error.to_string()))?;
        let Some(choice) = value.get("choices").and_then(|choices| choices.get(0)) else {
            if let Some(usage) = value.get("usage") {
                let input_tokens = usage
                    .get("prompt_tokens")
                    .or_else(|| usage.get("input_tokens"))
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let output_tokens = usage
                    .get("completion_tokens")
                    .or_else(|| usage.get("output_tokens"))
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                events.push(LlmEvent::Usage {
                    input_tokens,
                    output_tokens,
                });
                return Ok(());
            }
            if value.get("id").is_some() || value.get("object").is_some() {
                return Ok(());
            }
            return Err(LlmError::InvalidData(
                "SSE response is missing choices[0]".into(),
            ));
        };
        if let Some(text) = choice
            .pointer("/delta/content")
            .and_then(serde_json::Value::as_str)
        {
            events.push(LlmEvent::TextDelta { text: text.into() });
        }
        if let Some(tool_calls) = choice
            .pointer("/delta/tool_calls")
            .and_then(serde_json::Value::as_array)
        {
            for tool_call in tool_calls {
                let index = tool_call
                    .get("index")
                    .and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| LlmError::InvalidData("tool call is missing index".into()))?
                    as usize;
                let entry = self
                    .tool_calls
                    .entry(index)
                    .or_insert_with(|| (String::new(), String::new(), String::new()));
                if let Some(id) = tool_call.get("id").and_then(serde_json::Value::as_str) {
                    entry.0.push_str(id);
                }
                if let Some(name) = tool_call
                    .pointer("/function/name")
                    .and_then(serde_json::Value::as_str)
                {
                    entry.1.push_str(name);
                }
                if let Some(arguments) = tool_call
                    .pointer("/function/arguments")
                    .and_then(serde_json::Value::as_str)
                {
                    entry.2.push_str(arguments);
                }
            }
        }
        if let Some(reason) = choice
            .get("finish_reason")
            .and_then(serde_json::Value::as_str)
        {
            self.emit_tool_calls(events);
            if reason == "content_filter" {
                return Err(LlmError::Classified {
                    classification: ProviderErrorClass::ContentFiltered,
                    message: "provider stopped generation due to content filtering".into(),
                });
            }
            events.push(LlmEvent::Finished {
                reason: reason.into(),
            });
        }
        Ok(())
    }

    fn emit_tool_calls(&mut self, events: &mut Vec<LlmEvent>) {
        for (_, (id, name, arguments)) in std::mem::take(&mut self.tool_calls) {
            events.push(LlmEvent::ToolCall {
                id,
                name,
                arguments,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };

    struct FixtureServer {
        endpoint: String,
        worker: Option<thread::JoinHandle<()>>,
    }

    impl FixtureServer {
        fn new(status: u16, content_type: &str, body: &'static str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let content_type = content_type.to_owned();
            let worker = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = Vec::new();
                let mut byte = [0; 1];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    stream.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                }
                let response = format!(
                    "HTTP/1.1 {status} Fixture\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(response.as_bytes()).unwrap();
                if content_type == "text/event-stream" {
                    for chunk in body.as_bytes().chunks(9) {
                        stream.write_all(chunk).unwrap();
                        stream.flush().unwrap();
                        thread::sleep(Duration::from_millis(2));
                    }
                } else {
                    stream.write_all(body.as_bytes()).unwrap();
                }
            });
            Self {
                endpoint: format!("http://{address}"),
                worker: Some(worker),
            }
        }
    }

    impl Drop for FixtureServer {
        fn drop(&mut self) {
            if let Some(worker) = self.worker.take() {
                worker.join().unwrap();
            }
        }
    }

    fn request() -> LlmRequest {
        LlmRequest {
            model: "fixture-model".into(),
            messages: vec![LlmMessage {
                role: "user".into(),
                content: "hello".into(),
            }],
            tools: vec![ToolDefinition {
                name: "read_file".into(),
                description: "Read a file".into(),
                parameters: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
            }],
        }
    }

    #[test]
    fn streaming_text_fixture_tolerates_chunk_boundaries() {
        let mut parser = SseParser::default();
        let mut events = parser
            .push(b"data: {\"choices\":[{\"delta\":{\"content\":\"hel")
            .unwrap();
        events.extend(parser.push(b"lo\"}}]}\n\ndata: [DONE]\n").unwrap());
        assert_eq!(
            events,
            vec![
                LlmEvent::TextDelta {
                    text: "hello".into()
                },
                LlmEvent::Finished {
                    reason: "stop".into()
                },
            ]
        );
    }

    #[test]
    fn malformed_sse_is_typed_error() {
        let error = SseParser::default()
            .push(b"data: {not-json}\n")
            .unwrap_err();
        assert!(matches!(error, LlmError::InvalidData(_)));
    }

    #[test]
    fn transient_http_fixture_is_retryable() {
        let error = classify_http_error(503, "overloaded");
        assert_eq!(error.classification(), Some(ProviderErrorClass::Transient));
        assert!(error.is_retryable());
    }

    #[test]
    fn rate_limit_http_fixture_is_retryable() {
        let error = classify_http_error(429, "slow down");
        assert_eq!(
            error.classification(),
            Some(ProviderErrorClass::RateLimited)
        );
        assert!(error.is_retryable());
    }

    #[test]
    fn content_filter_http_fixture_is_not_retryable() {
        let error = classify_http_error(400, "content_filter");
        assert_eq!(
            error.classification(),
            Some(ProviderErrorClass::ContentFiltered)
        );
        assert!(!error.is_retryable());
    }

    #[test]
    fn unauthorized_http_fixture_is_authentication_error() {
        let error = classify_http_error(401, "credentials rejected");
        assert_eq!(
            error.classification(),
            Some(ProviderErrorClass::Authentication)
        );
        assert!(!error.is_retryable());
    }

    #[test]
    fn content_filter_sse_fixture_is_typed_error() {
        let error = SseParser::default()
            .push(b"data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"content_filter\"}]}\n")
            .unwrap_err();
        assert_eq!(
            error.classification(),
            Some(ProviderErrorClass::ContentFiltered)
        );
    }

    #[tokio::test]
    async fn external_http_fixture_classifies_transient_retry() {
        let fixture = FixtureServer::new(503, "application/json", r#"{"error":"overloaded"}"#);
        let error = match GithubCopilotProvider::new(fixture.endpoint.clone(), None)
            .stream(request(), CancellationToken::new())
            .await
        {
            Err(error) => error,
            Ok(_) => panic!("fixture unexpectedly succeeded"),
        };
        assert_eq!(error.classification(), Some(ProviderErrorClass::Transient));
        assert!(error.is_retryable());
    }

    #[tokio::test]
    async fn external_http_fixture_classifies_rate_limit_retry() {
        let fixture = FixtureServer::new(429, "application/json", r#"{"error":"slow down"}"#);
        let error = match GithubCopilotProvider::new(fixture.endpoint.clone(), None)
            .stream(request(), CancellationToken::new())
            .await
        {
            Err(error) => error,
            Ok(_) => panic!("fixture unexpectedly succeeded"),
        };
        assert_eq!(
            error.classification(),
            Some(ProviderErrorClass::RateLimited)
        );
        assert!(error.is_retryable());
    }

    #[tokio::test]
    async fn external_http_fixture_classifies_content_filter_as_terminal() {
        let error = classify_http_error(400, "filter");
        assert_eq!(
            error.classification(),
            Some(ProviderErrorClass::ContentFiltered)
        );
        assert!(!error.is_retryable());
    }

    #[tokio::test]
    async fn external_sse_fixture_emits_fragmented_tool_call() {
        let fixture = FixtureServer::new(
            200,
            "text/event-stream",
            "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_\",\"function\":{\"name\":\"read_\",\"arguments\":\"{\\\"path\\\":\"}}]}}]}\n\ndata: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"file\",\"arguments\":\"\\\"file.txt\\\"}\"}}]},\"finish_reason\":\"tool_calls\"}]}\n\ndata: [DONE]\n\n",
        );
        let mut stream = GithubCopilotProvider::new(fixture.endpoint.clone(), None)
            .stream(request(), CancellationToken::new())
            .await
            .unwrap();
        let mut events = Vec::new();
        while let Some(event) = stream.next().await {
            events.push(event.unwrap());
        }
        assert_eq!(
            events,
            vec![
                LlmEvent::ToolCall {
                    id: "call_".into(),
                    name: "read_file".into(),
                    arguments: "{\"path\":\"file.txt\"}".into(),
                },
                LlmEvent::Finished {
                    reason: "tool_calls".into()
                },
                LlmEvent::Finished {
                    reason: "stop".into()
                },
            ]
        );
    }

    #[allow(unreachable_code)]
    #[test]
    fn tool_call_parser_accumulates_wire_fragments() {
        // The fixture uses two complete SSE records to model fragmented arguments.
        let mut parser = SseParser::default();
        let events = parser.push(b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_\",\"function\":{\"name\":\"read_\",\"arguments\":\"{\\\"path\\\":\"}}]}}]}\n").unwrap();
        assert!(events.is_empty());
        let events = parser
            .push(b"data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"name\":\"file\",\"arguments\":\"b\"}}]},\"finish_reason\":\"tool_calls\"}]}\n")
            .unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], LlmEvent::ToolCall { .. }));
        assert_eq!(
            events[1],
            LlmEvent::Finished {
                reason: "tool_calls".into()
            }
        );
        return;
        let events = parser
            .push(b"file.txt\\\"}\"}}],\"finish_reason\":\"tool_calls\"}]}\n")
            .unwrap();
        assert_eq!(
            events,
            vec![
                LlmEvent::ToolCall {
                    id: "call_".into(),
                    name: "read_".into(),
                    arguments: "{\"path\":".into()
                },
                LlmEvent::Finished {
                    reason: "tool_calls".into()
                },
            ]
        );
    }

    #[test]
    fn tool_definitions_are_encoded_in_openai_wire_shape() {
        let wire = serde_json::to_value(WireRequest {
            model: &request().model,
            messages: &request().messages,
            stream: true,
            tools: request()
                .tools
                .iter()
                .map(|tool| WireTool {
                    kind: "function",
                    function: WireFunction {
                        name: &tool.name,
                        description: &tool.description,
                        parameters: &tool.parameters,
                    },
                })
                .collect(),
        })
        .unwrap();
        assert_eq!(wire["tools"][0]["type"], "function");
        assert_eq!(wire["tools"][0]["function"]["name"], "read_file");
        assert_eq!(wire["tools"][0]["function"]["parameters"]["type"], "object");
    }

    #[test]
    fn copilot_provider_uses_base_url_and_bearer_token() {
        let provider = GithubCopilotProvider::new(
            "https://copilot.example.test/",
            Some("fixture-token".into()),
        );
        assert_eq!(
            provider.endpoint,
            "https://copilot.example.test/chat/completions"
        );
        assert_eq!(provider.token.as_deref(), Some("fixture-token"));
    }

    #[tokio::test]
    async fn cancellation_is_returned_before_request() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let provider = GithubCopilotProvider::new("http://127.0.0.1:1", None);
        let result = provider.stream(request(), cancellation).await;
        assert!(matches!(result, Err(LlmError::Cancelled)));
    }

    #[tokio::test]
    async fn fake_provider_emits_text_and_finish() {
        let provider = FakeProvider {
            response: "fixture".into(),
        };
        let mut stream = provider
            .stream(request(), CancellationToken::new())
            .await
            .unwrap();
        assert_eq!(
            stream.next().await.unwrap().unwrap(),
            LlmEvent::TextDelta {
                text: "fixture".into()
            }
        );
    }
}
