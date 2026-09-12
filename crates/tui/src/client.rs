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
            let mut buffer = String::new();
            futures_util::pin_mut!(stream);
            use futures_util::StreamExt;
            while let Some(chunk) = stream.next().await {
                buffer.push_str(&String::from_utf8_lossy(&chunk?));
                while let Some((end, delimiter_len)) = sse_frame_end(&buffer) {
                    let frame = buffer.drain(..end + delimiter_len).collect::<String>();
                    if let Some(data) = frame.lines().find_map(|line| {
                        line.strip_prefix("data:").map(str::trim_start)
                    }) {
                        let sequence = frame
                            .lines()
                            .find_map(|line| line.strip_prefix("id: "))
                            .and_then(|value| value.parse::<u64>().ok())
                            .ok_or_else(|| serde_json::Error::io(std::io::Error::other("SSE event is missing id")))?;
                        yield (EventSequence(sequence), serde_json::from_str::<Event>(data)?);
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

fn sse_frame_end(buffer: &str) -> Option<(usize, usize)> {
    match (buffer.find("\r\n\r\n"), buffer.find("\n\n")) {
        (Some(crlf), Some(lf)) if crlf < lf => Some((crlf, 4)),
        (Some(crlf), _) => Some((crlf, 4)),
        (_, Some(lf)) => Some((lf, 2)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::sse_frame_end;
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
    fn sse_frame_end_accepts_crlf_and_lf() {
        assert_eq!(sse_frame_end("data: x\r\n\r\n"), Some((7, 4)));
        assert_eq!(sse_frame_end("data: x\n\n"), Some((7, 2)));
    }
}
