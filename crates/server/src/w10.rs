use axum::http::StatusCode;
use devfoundry_protocol::UpdateTaskRequest;

#[derive(Debug)]
pub struct TaskUpdateApiError {
    pub status: StatusCode,
}

pub fn parse_task_update(
    request: UpdateTaskRequest,
) -> Result<UpdateTaskRequest, TaskUpdateApiError> {
    if request.expected_revision.0 == 0 {
        return Ok(request);
    }
    Err(TaskUpdateApiError {
        status: StatusCode::CONFLICT,
    })
}
