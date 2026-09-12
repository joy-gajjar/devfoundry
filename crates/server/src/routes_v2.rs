use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use devfoundry_schema::{
    Message, MessageId, MessagePart, MessageRole, ProjectId, Session, SessionId, SessionStatus,
};
use devfoundry_storage::{EventRepository, SessionRepository};
use serde::{Deserialize, Serialize};

use crate::{ApiResult, ServerState, error, storage_error};
use axum::http::StatusCode;

const MAX_HISTORY_LIMIT: u32 = 100;
const MAX_HISTORY_BYTES: usize = 1024 * 1024;

#[derive(Debug, Deserialize)]
pub(crate) struct HistoryQuery {
    pub after: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub(crate) struct MessageWire {
    pub id: MessageId,
    pub session_id: SessionId,
    pub role: MessageRole,
    pub parts: Vec<MessagePart>,
    pub created_at: DateTime<Utc>,
}

impl From<Message> for MessageWire {
    fn from(message: Message) -> Self {
        Self {
            id: message.id,
            session_id: message.session_id,
            role: message.role,
            parts: message.parts,
            created_at: message.created_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct HistoryResponse {
    pub version: u16,
    pub messages: Vec<MessageWire>,
    pub next: Option<MessageId>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SessionWire {
    pub id: SessionId,
    pub project_id: ProjectId,
    pub title: String,
    pub agent: devfoundry_schema::AgentName,
    pub model: devfoundry_schema::ModelRef,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Session> for SessionWire {
    fn from(session: Session) -> Self {
        Self {
            id: session.id,
            project_id: session.project_id,
            title: session.title,
            agent: session.agent,
            model: session.model,
            status: session.status,
            created_at: session.created_at,
            updated_at: session.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SnapshotResponse {
    pub version: u16,
    pub session: SessionWire,
    pub messages: Vec<MessageWire>,
    pub replay: ReplayDescriptor,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReplayDescriptor {
    pub after: u64,
    pub events_url: String,
}

pub(crate) async fn history(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> ApiResult<Json<HistoryResponse>> {
    let session_id = parse_session_id(&session_id)?;
    ensure_session(&state, session_id).await?;
    let limit = query.limit.unwrap_or(50);
    if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
        return Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_limit",
            "limit must be between 1 and 100",
        ));
    }
    let after = query
        .after
        .map(|value| {
            value.parse::<MessageId>().map_err(|_| {
                error(
                    StatusCode::BAD_REQUEST,
                    "invalid_cursor",
                    "after cursor is invalid",
                )
            })
        })
        .transpose()?;
    let messages = state
        .store
        .list_messages_after(session_id, after, limit, MAX_HISTORY_BYTES)
        .await
        .map_err(storage_error)?;
    let next = (messages.len() == limit as usize)
        .then(|| messages.last().map(|(id, _)| *id))
        .flatten();
    Ok(Json(HistoryResponse {
        version: 2,
        messages: messages
            .into_iter()
            .map(|(_, message)| message.into())
            .collect(),
        next,
    }))
}

pub(crate) async fn snapshot(
    State(state): State<ServerState>,
    Path(session_id): Path<String>,
) -> ApiResult<Json<SnapshotResponse>> {
    let session_id = parse_session_id(&session_id)?;
    let session = ensure_session(&state, session_id).await?;
    let messages = state
        .store
        .list_messages_after(session_id, None, MAX_HISTORY_LIMIT, MAX_HISTORY_BYTES)
        .await
        .map_err(storage_error)?;
    let events = state
        .store
        .list_events_after(session_id, None)
        .await
        .map_err(storage_error)?;
    let after = events.last().map_or(0, |(sequence, _)| sequence.0);
    Ok(Json(SnapshotResponse {
        version: 2,
        session: session.into(),
        messages: messages
            .into_iter()
            .map(|(_, message)| message.into())
            .collect(),
        replay: ReplayDescriptor {
            after,
            events_url: format!("/api/v2/sessions/{session_id}/events"),
        },
    }))
}

pub(crate) async fn idempotent_prompt() -> ApiResult<Json<serde_json::Value>> {
    Err(error(
        StatusCode::NOT_IMPLEMENTED,
        "idempotency_unavailable",
        "v2 prompt admission is unavailable until the host admission bridge is integrated",
    ))
}

fn parse_session_id(value: &str) -> ApiResult<SessionId> {
    value.parse().map_err(|_| {
        error(
            StatusCode::BAD_REQUEST,
            "invalid_session_id",
            "session_id is invalid",
        )
    })
}

async fn ensure_session(state: &ServerState, id: SessionId) -> ApiResult<Session> {
    state
        .store
        .get_session(id)
        .await
        .map_err(storage_error)?
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "session_not_found",
                "session was not found",
            )
        })
}
