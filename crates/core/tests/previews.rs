use devfoundry_core::{PreviewId, PreviewService, PreviewState};

#[test]
fn preview_lifecycle_is_fail_closed_and_terminal() {
    let id = PreviewId::new();
    let mut service = PreviewService::new(id, "worktree-1", "revision-1");

    assert_eq!(service.state(), PreviewState::Requested);
    service.starting().unwrap();
    service.ready("http://127.0.0.1:4173").unwrap();
    assert_eq!(service.state(), PreviewState::Ready);
    service.stopped().unwrap();
    assert_eq!(service.state(), PreviewState::Stopped);
    assert!(service.ready("http://127.0.0.1:4173").is_err());
}

#[test]
fn preview_annotations_require_current_revision() {
    let id = PreviewId::new();
    let mut service = PreviewService::new(id, "worktree-1", "revision-1");
    service.starting().unwrap();
    service.ready("http://127.0.0.1:4173").unwrap();

    assert!(service.accept_annotation("revision-2").is_err());
    assert!(service.accept_annotation("revision-1").is_ok());
}
