//! Terminal client state and rendering.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use devfoundry_protocol::ModelOption;
use devfoundry_schema::{
    AgentName, ModelName, ModelRef, PermissionRequest, Project, ProviderName, Session,
    SessionStatus,
};
use ratatui::{
    Frame,
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use std::io;
use std::time::{Duration, Instant};

pub mod client;
pub mod terminal;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TranscriptRole {
    User,
    Assistant,
    Tool,
    System,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TranscriptEntry {
    pub role: TranscriptRole,
    pub text: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TuiState {
    pub project_name: String,
    pub session_title: String,
    pub agent: String,
    pub model: String,
    pub connected: bool,
    pub transcript: Vec<TranscriptEntry>,
    pub input: String,
    pub status: Option<SessionStatus>,
    pub active_tool: Option<String>,
    pub pending_permission: Option<PermissionRequest>,
    pub permission_modal: bool,
    pub scroll: usize,
    pub should_exit: bool,
    pub command_palette: bool,
    pub spinner: usize,
    pub scroll_mode: bool,
    pub agent_settings: AgentSettingsState,
    pub worker_dashboard: WorkerDashboardState,
    pub terminal: TerminalPaneState,
    pending_prompt: Option<String>,
    dismissed_permission: Option<devfoundry_schema::PermissionRequestId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentSettingsState {
    pub open: bool,
    pub profiles: Vec<String>,
    pub selected: usize,
    pub model: String,
    pub step_limit: u32,
    pub max_workers: u32,
    pub source: String,
}

impl Default for AgentSettingsState {
    fn default() -> Self {
        Self {
            open: false,
            profiles: ["boss", "build", "plan", "review", "test"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            selected: 1,
            model: "default".into(),
            step_limit: 12,
            max_workers: 1,
            source: "default".into(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkerDashboardState {
    pub open: bool,
    pub loading: bool,
    pub error: Option<String>,
    pub rows: Vec<WorkerRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerRow {
    pub task: String,
    pub profile: String,
    pub status: String,
    pub evidence: String,
    pub failure: Option<String>,
}

pub use terminal::TerminalPaneState;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PermissionApprovalAction {
    Allow,
    Deny,
    Cancel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    Quit,
    Interrupt,
    Refresh,
    AgentSettings,
    AgentSelected(String),
    WorkerDashboard,
}

impl TuiState {
    pub fn handle_paste(&mut self, text: String) {
        self.input.push_str(&text);
    }
    pub fn apply_event(&mut self, event: devfoundry_schema::Event) {
        use devfoundry_schema::Event;
        match event {
            Event::SessionStatus { status, .. } => self.status = Some(status),
            Event::MessageDelta { text, .. } => self.push_assistant_delta(&text),
            Event::ToolStarted { name, .. } => {
                self.active_tool = Some(name.clone());
                self.transcript.push(TranscriptEntry {
                    role: TranscriptRole::Tool,
                    text: format!("started {name}"),
                });
            }
            Event::ToolFinished { success, .. } => {
                self.active_tool = None;
                self.transcript.push(TranscriptEntry {
                    role: TranscriptRole::Tool,
                    text: if success {
                        "completed".into()
                    } else {
                        "failed".into()
                    },
                });
            }
            Event::PermissionRequested { .. } => {}
            Event::PermissionResolved { .. } => {
                self.pending_permission = None;
                self.permission_modal = false;
                self.dismissed_permission = None;
            }
            Event::Error { message, .. } => {
                self.status = Some(SessionStatus::Error);
                self.transcript.push(TranscriptEntry {
                    role: TranscriptRole::System,
                    text: message,
                });
            }
            Event::MessageCreated { .. } => {}
        }
    }

    pub fn push_user_prompt(&mut self) -> Option<String> {
        let prompt = self.begin_prompt_admission()?;
        self.confirm_prompt_admitted();
        Some(prompt)
    }

    pub fn begin_prompt_admission(&mut self) -> Option<String> {
        if self.pending_prompt.is_some() {
            return None;
        }
        let prompt = self.input.clone();
        if prompt.trim().is_empty() {
            return None;
        }
        self.pending_prompt = Some(prompt.clone());
        self.scroll = 0;
        self.transcript.push(TranscriptEntry {
            role: TranscriptRole::User,
            text: prompt.clone(),
        });
        self.transcript.push(TranscriptEntry {
            role: TranscriptRole::Assistant,
            text: String::new(),
        });
        Some(prompt)
    }

    /// Commits the optimistic prompt only after the server accepts admission.
    pub fn confirm_prompt_admitted(&mut self) {
        if self.pending_prompt.take().is_some() {
            self.input.clear();
        }
    }

    /// Drops optimistic transcript rows but intentionally leaves the draft editable.
    pub fn reject_prompt_admission(&mut self) {
        if self.pending_prompt.take().is_some() {
            self.transcript
                .truncate(self.transcript.len().saturating_sub(2));
        }
    }

    pub fn push_assistant_delta(&mut self, text: &str) {
        self.scroll = 0;
        if let Some(entry) = self
            .transcript
            .last_mut()
            .filter(|entry| entry.role == TranscriptRole::Assistant)
        {
            entry.text.push_str(text);
        } else {
            self.transcript.push(TranscriptEntry {
                role: TranscriptRole::Assistant,
                text: text.into(),
            });
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if self.agent_settings.open {
            match key.code {
                KeyCode::Up => {
                    self.agent_settings.selected = self.agent_settings.selected.saturating_sub(1)
                }
                KeyCode::Down => {
                    self.agent_settings.selected = (self.agent_settings.selected + 1)
                        .min(self.agent_settings.profiles.len().saturating_sub(1));
                }
                _ => {}
            }
            return None;
        }
        match key.code {
            KeyCode::Char(']')
                if key.modifiers.contains(event::KeyModifiers::CONTROL)
                    && self.terminal.focused =>
            {
                self.terminal.focused = false;
                None
            }
            KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.should_exit = true;
                None
            }
            KeyCode::Esc => {
                if self.agent_settings.open {
                    self.agent_settings.open = false;
                } else if self.worker_dashboard.open {
                    self.worker_dashboard.open = false;
                } else if self.command_palette {
                    self.command_palette = false;
                } else {
                    self.should_exit = true;
                }
                None
            }
            KeyCode::Char('p') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.command_palette = !self.command_palette;
                None
            }
            KeyCode::Enter => self.push_user_prompt(),
            KeyCode::Backspace => {
                self.input.pop();
                None
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_add(5);
                None
            }
            KeyCode::Up => {
                self.scroll = self.scroll.saturating_add(1);
                None
            }
            KeyCode::Char('k') if self.scroll_mode => {
                self.scroll = self.scroll.saturating_add(1);
                None
            }
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_sub(5);
                None
            }
            KeyCode::Down => {
                self.scroll = self.scroll.saturating_sub(1);
                None
            }
            KeyCode::Char('j') if self.scroll_mode => {
                self.scroll = self.scroll.saturating_sub(1);
                None
            }
            KeyCode::Home => {
                self.scroll = usize::MAX;
                None
            }
            KeyCode::End => {
                self.scroll = 0;
                None
            }
            KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_add(10);
                None
            }
            KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.scroll = self.scroll.saturating_sub(10);
                None
            }
            KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                self.scroll_mode = !self.scroll_mode;
                None
            }
            KeyCode::Char(character) => {
                if self.terminal.focused {
                    self.terminal.input.push(character as u8);
                    return None;
                }
                self.input.push(character);
                None
            }
            _ => None,
        }
    }

    pub fn reduce_permission_key(&mut self, key: KeyEvent) -> Option<PermissionApprovalAction> {
        self.pending_permission.as_ref()?;
        if key.kind == KeyEventKind::Release {
            return None;
        }
        if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
            self.should_exit = true;
            return None;
        }
        if !self.permission_modal {
            if key.code == KeyCode::Enter {
                self.permission_modal = true;
            }
            return None;
        }
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => Some(PermissionApprovalAction::Allow),
            KeyCode::Char('n') | KeyCode::Char('N') => Some(PermissionApprovalAction::Deny),
            KeyCode::Esc => {
                self.permission_modal = false;
                self.dismissed_permission = self.pending_permission.as_ref().map(|p| p.id);
                Some(PermissionApprovalAction::Cancel)
            }
            _ => None,
        }
    }

    pub fn handle_mouse(&mut self, mouse: event::MouseEvent) {
        match mouse.kind {
            event::MouseEventKind::ScrollUp => self.scroll = self.scroll.saturating_add(3),
            event::MouseEventKind::ScrollDown => self.scroll = self.scroll.saturating_sub(3),
            _ => {}
        }
    }

    pub fn command(&mut self, key: KeyEvent) -> Option<Command> {
        if self.agent_settings.open && key.code == KeyCode::Enter {
            let profile = self
                .agent_settings
                .profiles
                .get(self.agent_settings.selected)?
                .clone();
            self.agent_settings.open = false;
            return Some(Command::AgentSelected(profile));
        }
        if !self.command_palette {
            return None;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.command_palette = false;
                Some(Command::Quit)
            }
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.command_palette = false;
                Some(Command::Interrupt)
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.command_palette = false;
                Some(Command::Refresh)
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.command_palette = false;
                self.agent_settings.open = true;
                Some(Command::AgentSettings)
            }
            KeyCode::Char('w') | KeyCode::Char('W') => {
                self.command_palette = false;
                self.worker_dashboard.open = true;
                Some(Command::WorkerDashboard)
            }
            _ => None,
        }
    }
}

pub fn render(frame: &mut Frame<'_>, state: &TuiState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let spinner = ["|", "/", "-", "\\"][state.spinner % 4];
    let mut status = format!(
        " {}  {}  {}  {}  {}",
        match state.status.unwrap_or(SessionStatus::Idle) {
            SessionStatus::Idle => "idle",
            SessionStatus::Running => "running",
            SessionStatus::WaitingPermission => "waiting for approval",
            SessionStatus::WaitingQuestion => "waiting for input",
            SessionStatus::Cancelling => "cancelling",
            SessionStatus::Error => "error",
        },
        state
            .active_tool
            .as_deref()
            .map(|tool| format!("tool: {tool}"))
            .unwrap_or_else(|| "ready".into()),
        if state.pending_permission.is_some() {
            "approval required"
        } else {
            "local session"
        },
        if state.connected {
            "connected"
        } else {
            "disconnected"
        },
        if state.agent.is_empty() {
            "agent: unknown".into()
        } else {
            format!("agent: {} / {}", state.agent, state.model)
        },
    );
    if matches!(state.status, Some(SessionStatus::Running)) {
        status.push_str(&format!("  working {spinner}"));
    }
    frame.render_widget(
        Paragraph::new(status).style(Style::default().fg(Color::Black).bg(Color::Cyan)),
        layout[0],
    );

    let title = if state.pending_permission.is_some() {
        "DevFoundry | approval pending".to_owned()
    } else {
        format!(
            "DevFoundry | {} | {}",
            if state.project_name.is_empty() {
                "project"
            } else {
                &state.project_name
            },
            if state.session_title.is_empty() {
                "session"
            } else {
                &state.session_title
            }
        )
    };
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(30), Constraint::Length(28)])
        .split(layout[1]);
    let transcript_width = content_layout[0].width.saturating_sub(2) as usize;
    let transcript = render_transcript(state, spinner, transcript_width.max(1));
    let transcript_lines = transcript.lines.len();
    let viewport = content_layout[0].height.saturating_sub(2) as usize;
    let max_scroll = transcript_lines.saturating_sub(viewport);
    let top_offset = scroll_top_offset(transcript_lines, viewport, state.scroll);
    let body = Paragraph::new(transcript)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .scroll((top_offset.min(u16::MAX as usize) as u16, 0));
    frame.render_widget(body, content_layout[0]);

    let sidebar = vec![
        Line::from(Span::styled("SESSION", Style::default().fg(Color::Cyan))),
        Line::from(if state.session_title.is_empty() {
            "New session".into()
        } else {
            state.session_title.clone()
        }),
        Line::from(""),
        Line::from(Span::styled("AGENT", Style::default().fg(Color::Cyan))),
        Line::from(if state.agent.is_empty() {
            "unknown".into()
        } else {
            state.agent.clone()
        }),
        Line::from(""),
        Line::from(Span::styled("MODEL", Style::default().fg(Color::Cyan))),
        Line::from(if state.model.is_empty() {
            "unknown".into()
        } else {
            state.model.clone()
        }),
        Line::from(""),
        Line::from(Span::styled("SCROLL", Style::default().fg(Color::Cyan))),
        Line::from(if state.scroll == 0 {
            "bottom"
        } else {
            "history"
        }),
        Line::from(""),
        Line::from(Span::styled("KEYS", Style::default().fg(Color::Cyan))),
        Line::from(if state.scroll_mode {
            "SCROLL MODE: j/k active"
        } else {
            "Ctrl-S: scroll mode"
        }),
        Line::from("PgUp/PgDn page"),
        Line::from("Home/End jump"),
        Line::from(""),
        if state.pending_permission.is_some() {
            Line::from(Span::styled(
                "▶ Y allow   N deny",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ))
        } else {
            Line::from("No approval pending")
        },
    ];
    frame.render_widget(
        Paragraph::new(Text::from(sidebar))
            .block(Block::default().title("Details").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        content_layout[1],
    );
    let mut scrollbar = ScrollbarState::new(transcript_lines.max(1))
        .position(max_scroll.saturating_sub(state.scroll));
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        content_layout[0],
        &mut scrollbar,
    );

    let footer_title = if state.scroll_mode {
        "SCROLL MODE (Ctrl-S exit; j/k, arrows, PgUp/PgDn)"
    } else if state.scroll == 0 {
        "Prompt (at bottom)"
    } else {
        "Prompt (scrolling history)"
    };
    let cursor = "▌";
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::raw(state.input.as_str()),
        Span::styled(cursor, Style::default().fg(Color::Yellow)),
    ]))
    .block(
        Block::default()
            .title(if state.command_palette {
                "Commands: q quit | i interrupt | r refresh | Esc close"
            } else {
                footer_title
            })
            .borders(Borders::ALL),
    );
    frame.render_widget(footer, layout[2]);

    if state.agent_settings.open {
        render_agent_settings(frame, state);
    }
    if state.worker_dashboard.open {
        render_worker_dashboard(frame, state);
    }

    if state.permission_modal {
        render_permission_modal(frame, state);
    }
}

fn render_agent_settings(frame: &mut Frame<'_>, state: &TuiState) {
    let area = centered_rect(70, 70, frame.area());
    let lines = state
        .agent_settings
        .profiles
        .iter()
        .enumerate()
        .map(|(index, profile)| {
            let marker = if index == state.agent_settings.selected {
                ">"
            } else {
                " "
            };
            Line::from(format!("{marker} {profile}"))
        })
        .chain([
            Line::from(""),
            Line::from(format!("Model: {}", state.agent_settings.model)),
            Line::from(format!("Step limit: {}", state.agent_settings.step_limit)),
            Line::from(format!("Workers: {}", state.agent_settings.max_workers)),
            Line::from(format!("Source: {}", state.agent_settings.source)),
            Line::from(""),
            Line::from("Up/Down select | Esc close | custom profiles unavailable"),
        ])
        .collect::<Vec<_>>();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines)).block(
            Block::default()
                .title("AGENT SETTINGS")
                .borders(Borders::ALL),
        ),
        area,
    );
}

fn render_worker_dashboard(frame: &mut Frame<'_>, state: &TuiState) {
    let area = centered_rect(80, 70, frame.area());
    let mut lines = vec![
        Line::from("Workers are evidence, not acceptance."),
        Line::from(""),
    ];
    if state.worker_dashboard.loading {
        lines.push(Line::from("Loading worker state..."));
    }
    if let Some(error) = &state.worker_dashboard.error {
        lines.push(Line::from(format!("Error: {error}")));
    }
    if state.worker_dashboard.rows.is_empty() && !state.worker_dashboard.loading {
        lines.push(Line::from("No assigned workers."));
    }
    for row in &state.worker_dashboard.rows {
        lines.push(Line::from(format!(
            "{} | {} | {} | evidence: {}",
            row.task, row.profile, row.status, row.evidence
        )));
        if let Some(failure) = &row.failure {
            lines.push(Line::from(format!("  failure: {failure}")));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Esc close"));
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("WORKER DASHBOARD")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_permission_modal(frame: &mut Frame<'_>, state: &TuiState) {
    let Some(permission) = state.pending_permission.as_ref() else {
        return;
    };
    let area = centered_rect(70, 60, frame.area());
    let risk = permission_risk(&permission.operation);
    let lines = vec![
        Line::from(Span::styled(
            "A tool is waiting for your decision",
            Style::default().add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Operation: {}", permission.operation)),
        Line::from(format!("Target:    {}", permission.target)),
        Line::from(format!("Risk:      {risk}")),
        Line::from("Scope:     once (this exact request)"),
        Line::from(""),
        Line::from(Span::styled(
            "[Y] Allow once   [N] Deny   [Esc] Cancel",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from("Only these keys act. Other keys are ignored."),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .title("PERMISSION APPROVAL")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn permission_risk(operation: &str) -> &'static str {
    if matches!(
        operation,
        "read" | "list" | "search" | "status" | "diff" | "log"
    ) {
        "read-only"
    } else if operation.contains("network") || operation.contains("http") {
        "network / external"
    } else {
        "mutating or external"
    }
}

fn render_transcript(state: &TuiState, spinner: &str, width: usize) -> Text<'static> {
    let mut lines = Vec::new();
    for entry in &state.transcript {
        let (label, color) = match entry.role {
            TranscriptRole::User => ("you", Color::Cyan),
            TranscriptRole::Assistant => ("agent", Color::Green),
            TranscriptRole::Tool => ("tool", Color::Yellow),
            TranscriptRole::System => ("system", Color::Magenta),
        };
        let text = if entry.role == TranscriptRole::Assistant
            && entry.text.is_empty()
            && state.status == Some(SessionStatus::Running)
        {
            format!("thinking {spinner}")
        } else {
            entry.text.clone()
        };
        lines.push(Line::from(vec![
            Span::styled("┌ ", Style::default().fg(color)),
            Span::styled(
                format!("{label} "),
                Style::default()
                    .fg(color)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ),
            Span::styled(
                "─".repeat(width.saturating_sub(label.len() + 3)),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        render_entry_body(&mut lines, &text, entry.role == TranscriptRole::Tool, width);
        lines.push(Line::from(Span::styled("└", Style::default().fg(color))));
        lines.push(Line::from(""));
    }
    Text::from(lines)
}

fn render_entry_body(lines: &mut Vec<Line<'static>>, text: &str, is_tool: bool, width: usize) {
    let mut in_code = false;
    let raw_lines: Vec<&str> = text.lines().collect();
    let mut index = 0;
    while index < raw_lines.len() {
        let raw = raw_lines[index];
        if raw.trim_start().starts_with("```") {
            in_code = !in_code;
            let language = raw.trim().trim_start_matches("```").trim();
            let marker = if in_code && !language.is_empty() {
                format!("  code: {language}")
            } else {
                "  code".into()
            };
            lines.push(Line::from(Span::styled(
                marker,
                Style::default().fg(Color::Gray),
            )));
        } else if in_code {
            for wrapped in wrap_preserving_indent(raw, width.saturating_sub(4).max(1)) {
                lines.push(Line::from(Span::styled(
                    format!("  │ {wrapped}"),
                    Style::default()
                        .fg(Color::LightYellow)
                        .bg(Color::Rgb(35, 35, 35)),
                )));
            }
        } else if is_table_row(raw_lines.get(index).copied()) {
            let start = index;
            while index < raw_lines.len() && is_table_row(raw_lines.get(index).copied()) {
                index += 1;
            }
            render_table(lines, &raw_lines[start..index], width);
            continue;
        } else if let Some((level, heading)) = heading(raw) {
            let prefix = format!("{} ", "#".repeat(level.min(6)));
            render_wrapped_line(
                lines,
                &format!("  {prefix}{heading}"),
                width,
                Style::default()
                    .fg(Color::LightCyan)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            );
        } else if let Some(item) = bullet(raw) {
            render_wrapped_line(
                lines,
                &format!("  • {item}"),
                width,
                Style::default().fg(Color::Reset),
            );
        } else {
            let prefix = if is_tool { "  │ " } else { "  " };
            for wrapped in wrap_text(raw, width.saturating_sub(prefix.len()).max(1)) {
                lines.push(Line::from(vec![
                    Span::styled(
                        prefix,
                        if is_tool {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default()
                        },
                    ),
                    Span::raw(wrapped),
                ]));
            }
        }
        index += 1;
    }
}

fn render_wrapped_line(lines: &mut Vec<Line<'static>>, text: &str, width: usize, style: Style) {
    for wrapped in wrap_text(text, width.max(1)) {
        lines.push(Line::from(Span::styled(wrapped, style)));
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    text.split_whitespace().fold(Vec::new(), |mut lines, word| {
        if lines
            .last()
            .is_some_and(|line: &String| line.chars().count() + 1 + word.chars().count() <= width)
        {
            lines.last_mut().unwrap().push(' ');
            lines.last_mut().unwrap().push_str(word);
        } else if word.chars().count() <= width {
            lines.push(word.to_owned());
        } else {
            let mut current = String::new();
            for character in word.chars() {
                if current.chars().count() == width {
                    lines.push(std::mem::take(&mut current));
                }
                current.push(character);
            }
            if !current.is_empty() {
                lines.push(current);
            }
        }
        lines
    })
}

fn wrap_preserving_indent(text: &str, width: usize) -> Vec<String> {
    let indent = text
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    let prefix = " ".repeat(indent);
    let content = text.trim_start();
    wrap_text(content, width.saturating_sub(indent).max(1))
        .into_iter()
        .map(|line| format!("{prefix}{line}"))
        .collect()
}

fn heading(text: &str) -> Option<(usize, &str)> {
    let trimmed = text.trim_start();
    let level = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if level == 0 || trimmed.as_bytes().get(level) != Some(&b' ') {
        return None;
    }
    Some((level, trimmed.get(level + 1..)?.trim()))
}

fn bullet(text: &str) -> Option<&str> {
    let trimmed = text.trim_start();
    ["- ", "* ", "+ "]
        .iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix))
}

fn is_table_row(text: Option<&str>) -> bool {
    text.is_some_and(|line| line.trim().starts_with('|') && line.trim().ends_with('|'))
}

fn render_table(lines: &mut Vec<Line<'static>>, rows: &[&str], width: usize) {
    for (row_index, row) in rows.iter().enumerate() {
        let cells: Vec<&str> = row
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let separator = cells.iter().all(|cell| {
            !cell.is_empty()
                && cell
                    .chars()
                    .all(|character| character == '-' || character == ':' || character == ' ')
        });
        if separator {
            lines.push(Line::from(Span::styled(
                format!("  ├{}", "─".repeat(width.saturating_sub(6).max(1))),
                Style::default().fg(Color::DarkGray),
            )));
            continue;
        }
        let rendered = format!("  │ {}", cells.join(" │ "));
        let style = if row_index == 0 {
            Style::default()
                .fg(Color::LightCyan)
                .add_modifier(ratatui::style::Modifier::BOLD)
        } else {
            Style::default()
        };
        render_wrapped_line(lines, &rendered, width, style);
    }
}

fn scroll_top_offset(
    total_lines: usize,
    viewport_lines: usize,
    distance_from_bottom: usize,
) -> usize {
    total_lines
        .saturating_sub(viewport_lines)
        .saturating_sub(distance_from_bottom)
}

pub fn run() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let _cleanup = TerminalCleanup::new(ratatui::restore);
    let mut terminal = ratatui::init();
    run_loop(&mut terminal)
}

pub async fn run_session(
    client: client::ApiClient,
    session: devfoundry_schema::Session,
) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let _cleanup = TerminalCleanup::new(ratatui::restore);
    let mut terminal = ratatui::init();
    run_connected_loop(&mut terminal, client, session, None).await
}

pub async fn run_project_session(client: client::ApiClient, root: String) -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let _cleanup = TerminalCleanup::new(ratatui::restore);
    let mut terminal = ratatui::init();
    let session = select_session(&mut terminal, &client, &root).await?;
    run_connected_loop(&mut terminal, client, session.0, Some(session.1)).await
}

struct TerminalCleanup {
    restore: fn(),
    restored: bool,
}

impl TerminalCleanup {
    fn new(restore: fn()) -> Self {
        Self {
            restore,
            restored: false,
        }
    }
}

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        if !self.restored {
            (self.restore)();
            self.restored = true;
        }
    }
}

async fn select_session(
    terminal: &mut ratatui::DefaultTerminal,
    client: &client::ApiClient,
    root: &str,
) -> io::Result<(Session, String)> {
    let projects = client.list_projects().await.map_err(client_io)?;
    if projects.is_empty() {
        return Err(io::Error::other("no projects are available"));
    }
    let mut project_index = projects
        .iter()
        .position(|project| project.root == root)
        .unwrap_or(0);
    loop {
        terminal.draw(|frame| render_project_picker(frame, &projects, project_index))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Up => project_index = project_index.saturating_sub(1),
                KeyCode::Down => project_index = (project_index + 1).min(projects.len() - 1),
                KeyCode::Enter => break,
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Err(io::Error::other("selection cancelled"));
                }
                _ => {}
            }
        }
    }
    let project = &projects[project_index];
    let mut sessions = client.list_sessions(project.id).await.map_err(client_io)?;
    let models = client.list_models().await.map_err(client_io)?;
    let mut selected = 0usize;
    let mut filter = String::new();
    let agents = ["build", "plan", "review"];
    loop {
        let visible = filtered_session_indices(&sessions, &filter);
        if !visible.is_empty() {
            selected = selected.min(visible.len() - 1);
        } else {
            selected = 0;
        }
        terminal
            .draw(|frame| render_picker(frame, project, &sessions, &visible, selected, &filter))?;
        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            if key.code == KeyCode::Char('/')
                && !key.modifiers.contains(event::KeyModifiers::CONTROL)
            {
                filter.clear();
                edit_filter(terminal, &mut filter)?;
                continue;
            }
            match key.code {
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down if !visible.is_empty() => {
                    selected = (selected + 1).min(visible.len() - 1)
                }
                KeyCode::Char('n') => {
                    let agent = select_agent(terminal, &agents).await?;
                    let model = select_model(terminal, &models).await?;
                    let session = client
                        .create_session(
                            project.id,
                            &devfoundry_protocol::CreateSessionRequest {
                                project_id: project.id,
                                title: None,
                                agent: Some(AgentName(agent)),
                                model: Some(ModelRef {
                                    provider: ProviderName(model.provider),
                                    model: ModelName(model.id),
                                }),
                            },
                        )
                        .await
                        .map_err(client_io)?;
                    return Ok((session, project.name.clone()));
                }
                KeyCode::Enter if !visible.is_empty() => {
                    return Ok((sessions[visible[selected]].clone(), project.name.clone()));
                }
                KeyCode::Char('r') if !visible.is_empty() => {
                    let index = visible[selected];
                    let session_id = sessions[index].id;
                    if let Some(title) =
                        edit_text(terminal, "Rename session", &sessions[index].title)?
                        && !title.trim().is_empty()
                    {
                        sessions[index] = client
                            .update_session(
                                session_id,
                                &devfoundry_protocol::UpdateSessionRequest {
                                    title: Some(title.trim().to_owned()),
                                    agent: None,
                                    model: None,
                                },
                            )
                            .await
                            .map_err(client_io)?;
                    }
                }
                KeyCode::Char('a') if !visible.is_empty() => {
                    let index = visible[selected];
                    let agent = select_agent(terminal, &agents).await?;
                    let session_id = sessions[index].id;
                    sessions[index] = client
                        .update_session(
                            session_id,
                            &devfoundry_protocol::UpdateSessionRequest {
                                title: None,
                                agent: Some(AgentName(agent)),
                                model: None,
                            },
                        )
                        .await
                        .map_err(client_io)?;
                }
                KeyCode::Char('m') if !visible.is_empty() => {
                    let index = visible[selected];
                    let model = select_model(terminal, &models).await?;
                    let session_id = sessions[index].id;
                    sessions[index] = client
                        .update_session(
                            session_id,
                            &devfoundry_protocol::UpdateSessionRequest {
                                title: None,
                                agent: None,
                                model: Some(ModelRef {
                                    provider: ProviderName(model.provider),
                                    model: ModelName(model.id),
                                }),
                            },
                        )
                        .await
                        .map_err(client_io)?;
                }
                KeyCode::Backspace if !filter.is_empty() => {
                    filter.pop();
                }
                KeyCode::Char('q') if filter.is_empty() => {
                    return Err(io::Error::other("selection cancelled"));
                }
                KeyCode::Char(character)
                    if !key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    filter.push(character);
                }
                KeyCode::Esc if !filter.is_empty() => filter.clear(),
                KeyCode::Esc => {
                    return Err(io::Error::other("selection cancelled"));
                }
                _ => {}
            }
        }
    }
}

async fn select_model(
    terminal: &mut ratatui::DefaultTerminal,
    models: &[ModelOption],
) -> io::Result<ModelOption> {
    if models.is_empty() {
        return Err(io::Error::other("no models are available"));
    }
    let mut selected = 0usize;
    loop {
        terminal.draw(|frame| {
            let items = models
                .iter()
                .enumerate()
                .map(|(index, model)| {
                    let prefix = if index == selected { "> " } else { "  " };
                    Line::from(format!("{prefix}{} ({})", model.display_name, model.id))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(Text::from(items)).block(
                    Block::default()
                        .title("Models (Up/Down, Enter select, Esc cancel)")
                        .borders(Borders::ALL),
                ),
                frame.area(),
            );
        })?;
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down => selected = (selected + 1).min(models.len() - 1),
                KeyCode::Enter => return Ok(models[selected].clone()),
                KeyCode::Esc => return Err(io::Error::other("model selection cancelled")),
                _ => {}
            }
        }
    }
}

async fn select_agent(
    terminal: &mut ratatui::DefaultTerminal,
    agents: &[&str],
) -> io::Result<String> {
    if agents.is_empty() {
        return Err(io::Error::other("no agents are available"));
    }
    let mut selected = 0usize;
    loop {
        terminal.draw(|frame| {
            let items = agents
                .iter()
                .enumerate()
                .map(|(index, agent)| {
                    Line::from(format!(
                        "{} {}",
                        if index == selected { ">" } else { " " },
                        agent
                    ))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(Text::from(items)).block(
                    Block::default()
                        .title("Agents (Up/Down, Enter select, Esc cancel)")
                        .borders(Borders::ALL),
                ),
                frame.area(),
            );
        })?;
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Up => selected = selected.saturating_sub(1),
                KeyCode::Down => selected = (selected + 1).min(agents.len() - 1),
                KeyCode::Enter => return Ok(agents[selected].to_owned()),
                KeyCode::Esc => return Err(io::Error::other("agent selection cancelled")),
                _ => {}
            }
        }
    }
}

fn filtered_session_indices(sessions: &[Session], filter: &str) -> Vec<usize> {
    let filter = filter.trim().to_lowercase();
    sessions
        .iter()
        .enumerate()
        .filter_map(|(index, session)| {
            let matches = filter.is_empty()
                || session.title.to_lowercase().contains(&filter)
                || session.agent.0.to_lowercase().contains(&filter)
                || session.model.provider.0.to_lowercase().contains(&filter)
                || session.model.model.0.to_lowercase().contains(&filter);
            matches.then_some(index)
        })
        .collect()
}

fn edit_filter(terminal: &mut ratatui::DefaultTerminal, filter: &mut String) -> io::Result<()> {
    loop {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new(format!("/{}", filter)).block(
                    Block::default()
                        .title("Filter sessions (Enter apply, Esc clear)")
                        .borders(Borders::ALL),
                ),
                frame.area(),
            );
        })?;
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => return Ok(()),
                KeyCode::Backspace => {
                    filter.pop();
                }
                KeyCode::Char(character) => filter.push(character),
                _ => {}
            }
        }
    }
}

fn edit_text(
    terminal: &mut ratatui::DefaultTerminal,
    title: &str,
    initial: &str,
) -> io::Result<Option<String>> {
    let mut value = initial.to_owned();
    loop {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new(value.as_str()).block(
                    Block::default()
                        .title(format!("{title} (Enter save, Esc cancel)"))
                        .borders(Borders::ALL),
                ),
                frame.area(),
            );
        })?;
        if event::poll(Duration::from_millis(10))?
            && let Event::Key(key) = event::read()?
        {
            match key.code {
                KeyCode::Enter => return Ok(Some(value)),
                KeyCode::Esc => return Ok(None),
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(character) => value.push(character),
                _ => {}
            }
        }
    }
}

fn client_io(error: client::ClientError) -> io::Error {
    io::Error::other(error.to_string())
}

fn render_picker(
    frame: &mut Frame<'_>,
    project: &Project,
    sessions: &[Session],
    visible: &[usize],
    selected: usize,
    filter: &str,
) {
    let items = if visible.is_empty() {
        vec![Line::from(if sessions.is_empty() {
            "No sessions yet. Press n to start one."
        } else {
            "No sessions match the filter. Press / to search again."
        })]
    } else {
        visible
            .iter()
            .enumerate()
            .map(|(index, session_index)| {
                let session = &sessions[*session_index];
                Line::from(Span::styled(
                    format!(
                        "{} {} | {} / {} | {:?}",
                        if index == selected { ">" } else { " " },
                        session.title,
                        session.agent.0,
                        session.model.model.0,
                        session.status
                    ),
                    if index == selected {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    },
                ))
            })
            .collect()
    };
    let detail = visible.get(selected).map(|index| &sessions[*index]);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(30)])
        .split(frame.area());
    frame.render_widget(
        Paragraph::new(Text::from(items)).block(
            Block::default()
                .title(format!(
                    "{} | Sessions (Enter resume, n new, / filter, r rename, a agent, m model, q quit){}",
                    project.name,
                    if filter.is_empty() { String::new() } else { format!(" | filter: {filter}") }
                ))
                .borders(Borders::ALL),
        ),
        body[0],
    );
    let details = detail.map_or_else(
        || vec![Line::from("No session selected")],
        |session| {
            vec![
                Line::from(Span::styled("SESSION", Style::default().fg(Color::Cyan))),
                Line::from(session.title.clone()),
                Line::from(""),
                Line::from(Span::styled("AGENT", Style::default().fg(Color::Cyan))),
                Line::from(session.agent.0.clone()),
                Line::from(""),
                Line::from(Span::styled("MODEL", Style::default().fg(Color::Cyan))),
                Line::from(format!(
                    "{}:{}",
                    session.model.provider.0, session.model.model.0
                )),
                Line::from(""),
                Line::from(Span::styled("STATUS", Style::default().fg(Color::Cyan))),
                Line::from(format!("{:?}", session.status)),
            ]
        },
    );
    frame.render_widget(
        Paragraph::new(Text::from(details)).block(
            Block::default()
                .title("Selected session")
                .borders(Borders::ALL),
        ),
        body[1],
    );
}

fn render_project_picker(frame: &mut Frame<'_>, projects: &[Project], selected: usize) {
    let items = projects
        .iter()
        .enumerate()
        .map(|(index, project)| {
            Line::from(Span::styled(
                format!(
                    "{} {} | {}",
                    if index == selected { ">" } else { " " },
                    project.name,
                    project.root
                ),
                if index == selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                },
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(Text::from(items)).block(
            Block::default()
                .title("Projects (Up/Down, Enter select, q quit)")
                .borders(Borders::ALL),
        ),
        frame.area(),
    );
}

async fn run_connected_loop(
    terminal: &mut ratatui::DefaultTerminal,
    client: client::ApiClient,
    session: devfoundry_schema::Session,
    project_name: Option<String>,
) -> io::Result<()> {
    let (events_tx, mut events_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_client = client.clone();
    let session_id = session.id;
    tokio::spawn(async move {
        use futures_util::StreamExt;
        let mut after = None;
        let mut retry_delay = Duration::from_millis(100);
        loop {
            if let Ok(mut events) = event_client.events(session_id, after).await {
                retry_delay = Duration::from_millis(100);
                while let Some(result) = events.next().await {
                    match result {
                        Ok((sequence, event)) => {
                            after = Some(sequence.0);
                            if events_tx.send(event).is_err() {
                                return;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            tokio::time::sleep(retry_delay).await;
            retry_delay = (retry_delay * 2).min(Duration::from_secs(5));
        }
    });
    let mut state = TuiState {
        project_name: project_name.unwrap_or_default(),
        session_title: session.title.clone(),
        agent: session.agent.0.clone(),
        model: format!("{}:{}", session.model.provider.0, session.model.model.0),
        connected: true,
        status: Some(session.status),
        ..Default::default()
    };
    let mut next_refresh = Instant::now();
    loop {
        if Instant::now() >= next_refresh {
            next_refresh = Instant::now() + Duration::from_millis(500);
            if let Ok(mut permissions) = client.permissions(session_id).await {
                state.pending_permission = permissions.drain(..).next();
                if state.pending_permission.as_ref().map(|p| p.id) != state.dismissed_permission
                    && state.pending_permission.is_some()
                {
                    state.permission_modal = true;
                    state.status = Some(SessionStatus::WaitingPermission);
                }
            }
            match client.get_session(session_id).await {
                Ok(current) => {
                    state.connected = true;
                    state.status = Some(current.status);
                    if state.pending_permission.is_some() {
                        state.status = Some(SessionStatus::WaitingPermission);
                    }
                }
                Err(_) => state.connected = false,
            }
        }
        while let Ok(event) = events_rx.try_recv() {
            state.apply_event(event);
        }
        state.spinner = state.spinner.wrapping_add(1);
        terminal.draw(|frame| render(frame, &state))?;
        if state.should_exit {
            return Ok(());
        }
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                Event::Paste(text) => {
                    state.handle_paste(text);
                }
                Event::Mouse(mouse) => state.handle_mouse(mouse),
                Event::Key(key) => {
                    if state.pending_permission.is_some() {
                        let permission_decision = state.reduce_permission_key(key);
                        if let Some(action) = permission_decision
                            && let Some(request_id) =
                                state.pending_permission.as_ref().map(|r| r.id)
                        {
                            match action {
                                PermissionApprovalAction::Allow
                                | PermissionApprovalAction::Deny => {
                                    let allowed = action == PermissionApprovalAction::Allow;
                                    if let Err(error) =
                                        client.resolve_permission(request_id, allowed).await
                                    {
                                        state.transcript.push(TranscriptEntry {
                                            role: TranscriptRole::System,
                                            text: error.to_string(),
                                        });
                                    } else {
                                        state.pending_permission = None;
                                        state.permission_modal = false;
                                        state.status = Some(SessionStatus::Running);
                                    }
                                }
                                PermissionApprovalAction::Cancel => {}
                            }
                        }
                        continue;
                    }
                    if let Some(command) = state.command(key) {
                        match command {
                            Command::Quit => state.should_exit = true,
                            Command::Interrupt => {
                                if let Err(error) = client.interrupt(session_id).await {
                                    state.transcript.push(TranscriptEntry {
                                        role: TranscriptRole::System,
                                        text: error.to_string(),
                                    });
                                }
                            }
                            Command::Refresh => {
                                state.connected = client.get_session(session_id).await.is_ok();
                            }
                            Command::AgentSettings | Command::WorkerDashboard => {}
                            Command::AgentSelected(agent) => {
                                match client
                                    .update_session(
                                        session_id,
                                        &devfoundry_protocol::UpdateSessionRequest {
                                            title: None,
                                            agent: Some(devfoundry_schema::AgentName(
                                                agent.clone(),
                                            )),
                                            model: None,
                                        },
                                    )
                                    .await
                                {
                                    Ok(session) => state.agent = session.agent.0,
                                    Err(error) => state.transcript.push(TranscriptEntry {
                                        role: TranscriptRole::System,
                                        text: error.to_string(),
                                    }),
                                }
                            }
                        }
                        continue;
                    }
                    let prompt = if key.code == KeyCode::Enter {
                        state.begin_prompt_admission()
                    } else {
                        state.handle_key(key)
                    };
                    if let Some(prompt) = prompt {
                        match client.prompt(session_id, &prompt).await {
                            Ok(_) => state.confirm_prompt_admitted(),
                            Err(error) => {
                                state.reject_prompt_admission();
                                state.transcript.push(TranscriptEntry {
                                    role: TranscriptRole::System,
                                    text: error.to_string(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn run_loop(terminal: &mut ratatui::DefaultTerminal) -> io::Result<()> {
    run_loop_with_events(terminal, || {
        if event::poll(std::time::Duration::from_millis(100))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    })
}

fn run_loop_with_events<B, F>(
    terminal: &mut ratatui::Terminal<B>,
    mut next_event: F,
) -> io::Result<()>
where
    B: Backend,
    F: FnMut() -> io::Result<Option<Event>>,
{
    let mut state = TuiState::default();
    loop {
        terminal
            .draw(|frame| render(frame, &state))
            .map_err(|error| io::Error::other(error.to_string()))?;
        if state.should_exit {
            return Ok(());
        }
        if let Some(event) = next_event()? {
            handle_local_event(terminal, &mut state, event)?;
        }
    }
}

fn handle_local_event<B: Backend>(
    terminal: &mut ratatui::Terminal<B>,
    state: &mut TuiState,
    event: Event,
) -> io::Result<()> {
    match event {
        Event::Key(key) => {
            state.handle_key(key);
        }
        Event::Paste(text) => state.handle_paste(text),
        Event::Mouse(mouse) => state.handle_mouse(mouse),
        Event::Resize(width, height) => terminal
            .resize(Rect::new(0, 0, width, height))
            .map_err(|error| io::Error::other(error.to_string()))?,
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_submits_and_clears_prompt() {
        let mut state = TuiState {
            input: "hello".into(),
            ..Default::default()
        };
        assert_eq!(state.push_user_prompt().as_deref(), Some("hello"));
        assert!(state.input.is_empty());
        assert_eq!(state.transcript.len(), 2);
        assert_eq!(state.transcript[1].role, TranscriptRole::Assistant);
    }

    #[test]
    fn prompt_draft_is_preserved_until_admission_succeeds() {
        let mut state = TuiState {
            input: "retry me".into(),
            ..Default::default()
        };

        let prompt = state.begin_prompt_admission();

        assert_eq!(prompt.as_deref(), Some("retry me"));
        assert_eq!(state.input, "retry me");
        state.confirm_prompt_admitted();
        assert!(state.input.is_empty());
    }

    #[test]
    fn rejected_prompt_admission_restores_editable_draft() {
        let mut state = TuiState {
            input: "retry me".into(),
            ..Default::default()
        };
        assert_eq!(state.begin_prompt_admission().as_deref(), Some("retry me"));
        state.reject_prompt_admission();
        assert_eq!(state.input, "retry me");
        assert!(state.transcript.is_empty());
    }

    #[test]
    fn paste_appends_text_without_key_event_round_trip() {
        let mut state = TuiState::default();
        state.handle_paste("Explain the project\nstructure".into());
        assert_eq!(state.input, "Explain the project\nstructure");
    }

    #[test]
    fn separate_prompts_keep_assistant_responses_separate() {
        let mut state = TuiState {
            input: "first".into(),
            ..Default::default()
        };
        assert_eq!(state.push_user_prompt().as_deref(), Some("first"));
        state.push_assistant_delta("answer one");
        state.input = "second".into();
        assert_eq!(state.push_user_prompt().as_deref(), Some("second"));
        state.push_assistant_delta("answer two");
        assert_eq!(state.transcript[1].text, "answer one");
        assert_eq!(state.transcript[3].text, "answer two");
    }

    #[test]
    fn arrow_keys_scroll_transcript() {
        let mut state = TuiState::default();
        state.handle_key(KeyEvent::new(KeyCode::Up, event::KeyModifiers::NONE));
        assert_eq!(state.scroll, 1);
        state.handle_key(KeyEvent::new(KeyCode::Down, event::KeyModifiers::NONE));
        assert_eq!(state.scroll, 0);
    }

    #[test]
    fn empty_running_assistant_is_progress_state() {
        let mut state = TuiState {
            input: "test".into(),
            status: Some(SessionStatus::Running),
            ..Default::default()
        };
        state.push_user_prompt();
        assert!(state.transcript.last().unwrap().text.is_empty());
        assert_eq!(state.status, Some(SessionStatus::Running));
    }

    #[test]
    fn assistant_deltas_are_combined() {
        let mut state = TuiState::default();
        state.push_assistant_delta("a");
        state.push_assistant_delta("b");
        assert_eq!(state.transcript[0].text, "ab");
    }

    #[test]
    fn structured_transcript_renders_markdown_blocks_and_wraps() {
        let state = TuiState {
            transcript: vec![TranscriptEntry {
                role: TranscriptRole::Assistant,
                text: "# Heading\n\n- one\n- two\n\n```rust\n    let value = a_very_long_identifier;\n```\n\n| Name | State |\n| --- | --- |\n| job | done |".into(),
            }],
            ..Default::default()
        };
        let rendered = render_transcript(&state, "|", 32);
        let lines: Vec<String> = rendered
            .lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect()
            })
            .collect();
        assert!(lines.iter().any(|line| line.contains("# Heading")));
        assert!(lines.iter().any(|line| line.contains("• one")));
        assert!(lines.iter().any(|line| line.contains("code: rust")));
        assert!(lines.iter().any(|line| line.contains("let value")));
        assert!(lines.iter().any(|line| line.contains("Name")));
        assert!(lines.len() > 10);
    }

    #[test]
    fn tool_transcript_uses_card_body_marker() {
        let state = TuiState {
            transcript: vec![TranscriptEntry {
                role: TranscriptRole::Tool,
                text: "started read\noutput is deliberately long enough to wrap".into(),
            }],
            ..Default::default()
        };
        let rendered = render_transcript(&state, "|", 24);
        assert!(rendered.lines.iter().any(|line| {
            let content: String = line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect();
            content.contains("│ output")
        }));
    }

    #[test]
    fn session_filter_matches_title_agent_and_model() {
        let project_id = devfoundry_schema::ProjectId::new();
        let sessions = vec![Session {
            id: devfoundry_schema::SessionId::new(),
            project_id,
            title: "API cleanup".into(),
            agent: AgentName("build".into()),
            model: ModelRef {
                provider: ProviderName("github-copilot".into()),
                model: ModelName("gpt-4.1".into()),
            },
            status: SessionStatus::Idle,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];
        assert_eq!(filtered_session_indices(&sessions, "cleanup"), vec![0]);
        assert_eq!(filtered_session_indices(&sessions, "BUILD"), vec![0]);
        assert_eq!(filtered_session_indices(&sessions, "4.1"), vec![0]);
        assert!(filtered_session_indices(&sessions, "missing").is_empty());
    }

    #[test]
    fn tool_events_update_activity() {
        let session_id = devfoundry_schema::SessionId::new();
        let mut state = TuiState::default();
        state.apply_event(devfoundry_schema::Event::ToolStarted {
            session_id,
            call_id: devfoundry_schema::ToolCallId::new(),
            name: "read".into(),
        });
        assert_eq!(state.active_tool.as_deref(), Some("read"));
    }

    #[test]
    fn command_palette_dispatches_safe_commands() {
        let mut state = TuiState {
            command_palette: true,
            ..Default::default()
        };
        assert_eq!(
            state.command(KeyEvent::new(KeyCode::Char('i'), event::KeyModifiers::NONE)),
            Some(Command::Interrupt)
        );
        assert!(!state.command_palette);
    }

    fn permission() -> PermissionRequest {
        PermissionRequest {
            id: devfoundry_schema::PermissionRequestId::new(),
            session_id: devfoundry_schema::SessionId::new(),
            operation: "write".into(),
            target: "src/main.rs".into(),
            status: devfoundry_schema::PermissionStatus::Pending,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn permission_reducer_requires_explicit_modal_and_decision_key() {
        let mut state = TuiState {
            pending_permission: Some(permission()),
            ..Default::default()
        };
        assert_eq!(
            state.reduce_permission_key(KeyEvent::new(
                KeyCode::Char('x'),
                event::KeyModifiers::NONE
            )),
            None
        );
        assert!(!state.permission_modal);
        assert_eq!(
            state.reduce_permission_key(KeyEvent::new(KeyCode::Enter, event::KeyModifiers::NONE)),
            None
        );
        assert!(state.permission_modal);
        assert_eq!(
            state.reduce_permission_key(KeyEvent::new(
                KeyCode::Char('x'),
                event::KeyModifiers::NONE
            )),
            None
        );
        assert_eq!(
            state.reduce_permission_key(KeyEvent::new(
                KeyCode::Char('y'),
                event::KeyModifiers::NONE
            )),
            Some(PermissionApprovalAction::Allow)
        );
    }

    #[test]
    fn escape_cancels_modal_without_resolving_request() {
        let mut state = TuiState {
            pending_permission: Some(permission()),
            permission_modal: true,
            ..Default::default()
        };
        assert_eq!(
            state.reduce_permission_key(KeyEvent::new(KeyCode::Esc, event::KeyModifiers::NONE)),
            Some(PermissionApprovalAction::Cancel)
        );
        assert!(!state.permission_modal);
        assert!(state.pending_permission.is_some());
    }

    #[test]
    fn dismissed_permission_is_not_reopened_by_authoritative_poll() {
        let mut state = TuiState {
            pending_permission: Some(permission()),
            permission_modal: true,
            ..Default::default()
        };
        let request_id = state.pending_permission.as_ref().unwrap().id;
        assert_eq!(
            state.reduce_permission_key(KeyEvent::new(KeyCode::Esc, event::KeyModifiers::NONE)),
            Some(PermissionApprovalAction::Cancel)
        );
        assert_eq!(state.dismissed_permission, Some(request_id));
    }

    #[test]
    fn ctrl_c_remains_global_while_permission_has_focus() {
        let mut state = TuiState {
            pending_permission: Some(permission()),
            permission_modal: true,
            ..Default::default()
        };

        state.reduce_permission_key(KeyEvent::new(
            KeyCode::Char('c'),
            event::KeyModifiers::CONTROL,
        ));

        assert!(state.should_exit);
    }

    #[test]
    fn scroll_distance_is_converted_to_top_anchored_offset() {
        assert_eq!(scroll_top_offset(100, 20, 0), 80);
        assert_eq!(scroll_top_offset(100, 20, 3), 77);
        assert_eq!(scroll_top_offset(100, 20, usize::MAX), 0);
    }

    #[test]
    fn escape_closes_palette_before_exiting() {
        let mut state = TuiState {
            command_palette: true,
            ..Default::default()
        };
        state.handle_key(KeyEvent::new(KeyCode::Esc, event::KeyModifiers::NONE));
        assert!(!state.command_palette);
        assert!(!state.should_exit);
        state.handle_key(KeyEvent::new(KeyCode::Esc, event::KeyModifiers::NONE));
        assert!(state.should_exit);
    }

    #[test]
    fn resize_renders_without_panicking() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| render(frame, &TuiState::default()))
            .unwrap();
        terminal.backend_mut().resize(120, 40);
        terminal
            .resize(ratatui::layout::Rect::new(0, 0, 120, 40))
            .unwrap();
        terminal
            .draw(|frame| {
                assert_eq!(frame.area().width, 120);
                assert_eq!(frame.area().height, 40);
                render(frame, &TuiState::default());
            })
            .unwrap();
    }

    #[test]
    fn ctrl_c_interrupt_exits_interactive_loop() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut events = Some(Event::Key(KeyEvent::new(
            KeyCode::Char('c'),
            event::KeyModifiers::CONTROL,
        )));
        run_loop_with_events(&mut terminal, || Ok(events.take())).unwrap();
    }

    #[test]
    fn paste_and_resize_events_reach_local_loop() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut state = TuiState::default();
        handle_local_event(
            &mut terminal,
            &mut state,
            Event::Paste("pasted\ntext".into()),
        )
        .unwrap();
        assert_eq!(state.input, "pasted\ntext");
        handle_local_event(&mut terminal, &mut state, Event::Resize(120, 40)).unwrap();

        let mut events = vec![
            Event::Key(KeyEvent::new(KeyCode::Enter, event::KeyModifiers::NONE)),
            Event::Key(KeyEvent::new(KeyCode::Esc, event::KeyModifiers::NONE)),
        ]
        .into_iter();
        run_loop_with_events(&mut terminal, || Ok(events.next())).unwrap();
    }

    #[test]
    fn local_mouse_scroll_is_safe() {
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        let mut events = vec![
            Event::Mouse(event::MouseEvent {
                kind: event::MouseEventKind::ScrollUp,
                column: 0,
                row: 0,
                modifiers: event::KeyModifiers::NONE,
            }),
            Event::Key(KeyEvent::new(KeyCode::Esc, event::KeyModifiers::NONE)),
        ]
        .into_iter();
        run_loop_with_events(&mut terminal, || Ok(events.next())).unwrap();
    }

    #[test]
    fn terminal_cleanup_runs_during_panic_unwind() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        use std::sync::atomic::{AtomicBool, Ordering};

        static RESTORED: AtomicBool = AtomicBool::new(false);
        RESTORED.store(false, Ordering::Relaxed);
        let result = catch_unwind(AssertUnwindSafe(|| {
            let _cleanup = TerminalCleanup::new(|| {
                RESTORED.store(true, Ordering::Relaxed);
            });
            panic!("exercise terminal cleanup");
        }));
        assert!(result.is_err());
        assert!(RESTORED.load(Ordering::Relaxed));
    }

    #[test]
    fn terminal_cleanup_runs_once_on_normal_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static RESTORE_COUNT: AtomicUsize = AtomicUsize::new(0);
        RESTORE_COUNT.store(0, Ordering::Relaxed);
        {
            let _cleanup = TerminalCleanup::new(|| {
                RESTORE_COUNT.fetch_add(1, Ordering::Relaxed);
            });
        }
        assert_eq!(RESTORE_COUNT.load(Ordering::Relaxed), 1);
    }
}
