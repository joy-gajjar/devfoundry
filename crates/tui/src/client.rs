use devfoundry_protocol::{
    CreateProjectRequest, CreateSessionRequest, ModelOption, PromptRequest, UpdateSessionRequest,
};
use devfoundry_schema::{
    Event, EventSequence, PermissionRequest, Project, ProjectId, Session, SessionId,
};
use futures_util::Stream;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("server returned {status}: {message}")]
    Server { status: StatusCode, message: String },
    #[error("invalid server response: {0}")]
    Decode(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').into(),
            http: reqwest::Client::new(),
        }
    }

    pub async fn create_project(
        &self,
        request: &CreateProjectRequest,
    ) -> Result<Project, ClientError> {
        self.send(self.http.post(self.url("/api/v1/projects")).json(request))
            .await
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>, ClientError> {
        self.send(self.http.get(self.url("/api/v1/projects"))).await
    }

    pub async fn create_session(
        &self,
        project_id: ProjectId,
        request: &CreateSessionRequest,
    ) -> Result<Session, ClientError> {
        self.send(
            self.http
                .post(self.url(&format!("/api/v1/projects/{project_id}/sessions")))
                .json(request),
        )
        .await
    }

    pub async fn get_session(&self, session_id: SessionId) -> Result<Session, ClientError> {
        self.send(
            self.http
                .get(self.url(&format!("/api/v1/sessions/{session_id}"))),
        )
        .await
    }

    pub async fn list_sessions(&self, project_id: ProjectId) -> Result<Vec<Session>, ClientError> {
        self.send(
            self.http
                .get(self.url(&format!("/api/v1/projects/{project_id}/sessions"))),
        )
        .await
    }

    pub async fn list_models(&self) -> Result<Vec<ModelOption>, ClientError> {
        self.send(self.http.get(self.url("/api/v1/models"))).await
    }

    pub async fn update_session(
        &self,
        session_id: SessionId,
        request: &UpdateSessionRequest,
    ) -> Result<Session, ClientError> {
        self.send(
            self.http
                .patch(self.url(&format!("/api/v1/sessions/{session_id}")))
                .json(request),
        )
        .await
    }

    pub async fn prompt(
        &self,
        session_id: SessionId,
        prompt: &str,
    ) -> Result<serde_json::Value, ClientError> {
        self.send(
            self.http
                .post(self.url(&format!("/api/v1/sessions/{session_id}/prompt")))
                .json(&PromptRequest {
                    session_id,
                    prompt: prompt.into(),
                }),
        )
        .await
    }

    pub async fn permissions(
        &self,
        session_id: SessionId,
    ) -> Result<Vec<PermissionRequest>, ClientError> {
        self.send(
            self.http
                .get(self.url(&format!("/api/v1/sessions/{session_id}/permissions"))),
        )
        .await
    }

    pub async fn resolve_permission(
        &self,
        request_id: devfoundry_schema::PermissionRequestId,
        allowed: bool,
    ) -> Result<(), ClientError> {
        let response = self
            .http
            .post(self.url(&format!("/api/v1/permissions/{request_id}/resolve")))
            .json(&serde_json::json!({ "allowed": allowed }))
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(self.server_error(response).await)
        }
    }

    pub async fn interrupt(&self, session_id: SessionId) -> Result<(), ClientError> {
        let response = self
            .http
            .post(self.url(&format!("/api/v1/sessions/{session_id}/interrupt")))
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(self.server_error(response).await)
        }
    }

    pub async fn events(
        &self,
        session_id: SessionId,
        after: Option<u64>,
    ) -> Result<
        Pin<Box<dyn Stream<Item = Result<(EventSequence, Event), ClientError>> + Send>>,
        ClientError,
    > {
        let url = self.url(&format!("/api/v1/sessions/{session_id}/events"));
        let response = self.http.get(url).query(&[("after", after)]).send().await?;
        if !response.status().is_success() {
            return Err(self.server_error(response).await);
        }
        let stream = response.bytes_stream();
        Ok(Box::pin(async_stream::try_stream! {
            let mut decoder = SseDecoder::default();
            futures_util::pin_mut!(stream);
            use futures_util::StreamExt;
            while let Some(chunk) = stream.next().await {
                decoder.push(&chunk?)?;
                while let Some(frame) = decoder.next()? {
                    let sequence = frame.id.ok_or_else(|| serde_json::Error::io(std::io::Error::other("SSE event is missing id")))?;
                    if !frame.data.is_empty() {
                        yield (EventSequence(sequence), serde_json::from_str::<Event>(&frame.data)?);
                    }
                }
            }
        }))
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn send<T: DeserializeOwned>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T, ClientError> {
        let response = request.send().await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(self.server_error(response).await)
        }
    }

    async fn server_error(&self, response: reqwest::Response) -> ClientError {
        let status = response.status();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "request failed".into());
        ClientError::Server { status, message }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct SseFrame {
    id: Option<u64>,
    event: Option<String>,
    data: String,
}

#[derive(Debug, Default)]
struct SseDecoder {
    buffer: Vec<u8>,
}

impl SseDecoder {
    const MAX_FRAME_BYTES: usize = 1024 * 1024;

    fn push(&mut self, bytes: &[u8]) -> Result<(), serde_json::Error> {
        self.buffer.extend_from_slice(bytes);
        if self.buffer.len() > Self::MAX_FRAME_BYTES {
            return Err(serde_json::Error::io(std::io::Error::other(
                "SSE frame is too large",
            )));
        }
        Ok(())
    }

    fn next(&mut self) -> Result<Option<SseFrame>, serde_json::Error> {
        let Some((end, delimiter_len)) = find_sse_frame_end(&self.buffer) else {
            return Ok(None);
        };
        let frame = self.buffer.drain(..end + delimiter_len).collect::<Vec<_>>();
        let text = std::str::from_utf8(&frame[..end])
            .map_err(|error| serde_json::Error::io(std::io::Error::other(error)))?;
        let mut result = SseFrame::default();
        let mut data_lines = Vec::new();
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            let (field, value) = line.split_once(':').map_or((line, ""), |(field, value)| {
                (field, value.strip_prefix(' ').unwrap_or(value))
            });
            match field {
                "id" => result.id = value.parse().ok(),
                "event" => result.event = Some(value.to_owned()),
                "data" => data_lines.push(value),
                _ => {}
            }
        }
        result.data = data_lines.join("\n");
        Ok(Some(result))
    }
}

fn find_sse_frame_end(buffer: &[u8]) -> Option<(usize, usize)> {
    for index in 0..buffer.len().saturating_sub(1) {
        if buffer[index..].starts_with(b"\n\n") {
            return Some((index, 2));
        }
        if buffer[index..].starts_with(b"\r\n\r\n") {
            return Some((index, 4));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::SseDecoder;
    #[test]
    fn sse_data_prefix_accepts_optional_spacing() {
        let line = "data:{\"type\":\"message_delta\"}";
        assert_eq!(
            line.strip_prefix("data:").map(str::trim_start),
            Some("{\"type\":\"message_delta\"}")
        );
        let line = "data: {\"type\":\"message_delta\"}";
        assert_eq!(
            line.strip_prefix("data:").map(str::trim_start),
            Some("{\"type\":\"message_delta\"}")
        );
    }

    #[test]
    fn sse_decoder_handles_split_utf8_and_multiple_data_lines() {
        let mut decoder = SseDecoder::default();
        let payload = b"id: 7\r\nevent: message\r\ndata: {\"text\":\"caf\xc3\xa9\"}\r\n\r\n";

        decoder.push(&payload[..38]).unwrap();
        assert!(decoder.next().unwrap().is_none());
        decoder.push(&payload[38..]).unwrap();

        let frame = decoder.next().unwrap().expect("complete SSE frame");
        assert_eq!(frame.id, Some(7));
        assert_eq!(frame.event.as_deref(), Some("message"));
        assert_eq!(frame.data, "{\"text\":\"café\"}");
    }

    #[test]
    fn sse_decoder_accepts_id_without_space_and_ignores_comments() {
        let mut decoder = SseDecoder::default();
        decoder
            .push(b": heartbeat\n id: ignored\nid:9\ndata: ok\n\n")
            .unwrap();

        let frame = decoder.next().unwrap().expect("complete SSE frame");
        assert_eq!(frame.id, Some(9));
        assert_eq!(frame.data, "ok");
    }
}
