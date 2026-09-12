use axum::http::StatusCode;
use devfoundry_protocol::UpdateTaskRequest;
use devfoundry_schema::Revision;
use devfoundry_server::w10::parse_task_update;

#[test]
fn task_update_parser_maps_revision_conflicts_to_http_conflict() {
    let request = UpdateTaskRequest {
        expected_revision: Revision(1),
        title: None,
        status: None,
        dependencies: None,
    };
    let response = parse_task_update(request).unwrap_err();
    assert_eq!(response.status, StatusCode::CONFLICT);
}
