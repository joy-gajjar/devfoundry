use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use devfoundry_tui::{TerminalPaneState, TuiState};

#[test]
fn terminal_focus_escapes_without_closing_the_session() {
    let mut state = TuiState::default();
    state.terminal.focused = true;

    state.handle_key(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::CONTROL));

    assert!(!state.terminal.focused);
    assert!(!state.should_exit);
}

#[test]
fn terminal_output_gap_is_visible_in_pure_state() {
    let mut pane = TerminalPaneState::default();
    pane.apply_output_gap(12);

    assert!(pane.gap);
    assert_eq!(pane.output_offset, 12);
}
