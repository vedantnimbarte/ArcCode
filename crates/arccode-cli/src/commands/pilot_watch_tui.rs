//! Interactive ratatui UI for `arccode pilot watch`.
//!
//! Renders the same [`DashboardModel`] as `pilot status`, but as a live,
//! colour-coded, scrollable terminal UI laid out in a 2-column grid:
//!
//! ```text
//! ┌ Pilot: … ───────────────────────────────────┐
//! ┌ Tasks ───────────────┐┌ Agents ─────────────┐
//! │ …                    ││ …                   │
//! └──────────────────────┘└─────────────────────┘
//! ┌ Live log ───────────────────────────────────┐
//! │ …                                            │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! The polling model is identical to the plain `watch` loop — it watches
//! `<run-dir>/state.json`'s mtime and only reloads when it advances — but
//! input is drained every ~120 ms so scrolling and quitting stay snappy.
//! Terminal raw-mode / alternate-screen setup is torn down on every exit
//! path (including errors) so the shell is always left clean.

use std::io::{self, Stdout};
use std::path::Path;
use std::process::ExitCode;
use std::time::SystemTime;

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{self, Event as CtEvent, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

use arccode_autonomous::dashboard::{
    self, AgentRow, DashboardModel, HeaderInfo, LogSeverity, RunSummary, TaskRow,
};
use arccode_autonomous::{AgentStatus, RunStatus, TaskStatus};

type Term = Terminal<CrosstermBackend<Stdout>>;

/// Live-watch the pilot runs under `project_root` in a full-screen ratatui
/// UI, starting on `initial` (or the newest run). When more than one run is
/// active, a Runs sidebar appears and you can switch between them. Blocks
/// until the user quits (`q` / `Esc` / `Ctrl-C`).
pub fn run(project_root: &Path, initial: Option<String>, interval_ms: u64) -> Result<ExitCode> {
    let mut terminal = setup()?;
    // Whatever happens in the loop, always restore the terminal.
    let outcome = run_loop(&mut terminal, project_root, initial, interval_ms);
    teardown(&mut terminal)?;
    outcome
}

/// Which pane the arrow keys drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Focus {
    Runs,
    #[default]
    Tasks,
}

struct WatchUi {
    /// Runs offered in the sidebar — active (non-terminal) runs, plus the
    /// currently-watched run even if it has since finished.
    runs: Vec<RunSummary>,
    /// Index into `runs` of the run being watched.
    current: usize,
    focus: Focus,
    tasks_scroll: u16,
    last_mtime: Option<SystemTime>,
    model: Option<DashboardModel>,
    finished: bool,
    /// Animation frame, advanced off wall-clock time so the in-progress
    /// spinner rotates smoothly regardless of the state-poll cadence.
    frame: u64,
}

impl WatchUi {
    fn new(runs: Vec<RunSummary>, current: usize) -> Self {
        Self {
            runs,
            current,
            focus: Focus::default(),
            tasks_scroll: 0,
            last_mtime: None,
            model: None,
            finished: false,
            frame: 0,
        }
    }

    /// True once the sidebar is worth showing.
    fn show_runs(&self) -> bool {
        self.runs.len() > 1
    }

    fn current_dir(&self) -> Option<&Path> {
        self.runs.get(self.current).map(|r| r.dir.as_path())
    }

    /// Reload the current run's snapshot + a generous event tail for the log.
    fn reload(&mut self) {
        let Some(dir) = self.current_dir().map(Path::to_path_buf) else {
            return;
        };
        self.last_mtime = dashboard::state_mtime(&dir);
        if let (Ok(state), Ok(recent)) = (
            dashboard::load_state(&dir),
            dashboard::tail_events(&dir, 200),
        ) {
            self.finished = matches!(
                state.status,
                RunStatus::Done | RunStatus::Failed | RunStatus::Aborted
            );
            self.model = Some(dashboard::build_model(&state, &recent, Some(Utc::now())));
        }
    }

    /// Switch the watched run to `idx` and reload it.
    fn switch_to(&mut self, idx: usize) {
        if idx < self.runs.len() && idx != self.current {
            self.current = idx;
            self.tasks_scroll = 0;
            self.reload();
        }
    }

    fn select_prev(&mut self) {
        if self.current > 0 {
            self.switch_to(self.current - 1);
        }
    }

    fn select_next(&mut self) {
        if self.current + 1 < self.runs.len() {
            self.switch_to(self.current + 1);
        }
    }

    /// Re-list runs (active + the watched one), preserving the current
    /// selection by id and refreshing each run's progress/status.
    fn refresh_runs(&mut self, project_root: &Path) {
        let all = dashboard::list_runs(project_root).unwrap_or_default();
        let current_id = self.runs.get(self.current).map(|r| r.run_id.clone());
        let list = active_plus(all, current_id.as_deref());

        self.current = current_id
            .and_then(|cid| list.iter().position(|r| r.run_id == cid))
            .unwrap_or(0)
            .min(list.len().saturating_sub(1));
        self.runs = list;
        // Focus can't sit on a hidden sidebar.
        if !self.show_runs() {
            self.focus = Focus::Tasks;
        }
    }
}

/// Active (non-terminal) runs, plus `keep` (the watched run) even if it has
/// finished — so a run doesn't vanish from the sidebar mid-watch, and
/// `watch <finished-id>` still shows something. Falls back to *all* runs when
/// nothing is active, so the UI is never blank.
fn active_plus(all: Vec<RunSummary>, keep: Option<&str>) -> Vec<RunSummary> {
    let mut list: Vec<RunSummary> = all.iter().filter(|r| !r.is_terminal()).cloned().collect();
    if let Some(id) = keep {
        if !list.iter().any(|r| r.run_id == id) {
            if let Some(r) = all.iter().find(|r| r.run_id == id) {
                list.push(r.clone());
            }
        }
    }
    if list.is_empty() {
        list = all;
    }
    list
}

/// Initial run list + the index to start on.
fn initial_runs(project_root: &Path, initial: &Option<String>) -> (Vec<RunSummary>, usize) {
    let all = dashboard::list_runs(project_root).unwrap_or_default();
    let list = active_plus(all, initial.as_deref());
    let current = initial
        .as_ref()
        .and_then(|id| list.iter().position(|r| &r.run_id == id))
        .unwrap_or(0);
    (list, current)
}

fn run_loop(
    terminal: &mut Term,
    project_root: &Path,
    initial: Option<String>,
    interval_ms: u64,
) -> Result<ExitCode> {
    // Cap the wait so we repaint at least every ~120 ms — enough to animate
    // the spinner smoothly even when the user picked a slow --interval-ms.
    let poll = std::time::Duration::from_millis(interval_ms.clamp(50, 120));
    let started = std::time::Instant::now();

    let (runs, current) = initial_runs(project_root, &initial);
    if runs.is_empty() {
        return Ok(ExitCode::from(1));
    }
    let mut ui = WatchUi::new(runs, current);
    ui.reload();

    // Throttle the (relatively expensive) full run-list rescan.
    let mut last_list = std::time::Instant::now();
    let list_every = std::time::Duration::from_millis(1000);

    loop {
        // ~8 frames/sec: one spinner step per redraw at the 120 ms cap.
        ui.frame = (started.elapsed().as_millis() / 120) as u64;
        terminal.draw(|f| draw(f, &mut ui))?;

        // Drain input first so keys feel responsive.
        if event::poll(poll)? {
            if let CtEvent::Key(k) = event::read()? {
                if k.kind == KeyEventKind::Release {
                    continue;
                }
                let on_runs = ui.focus == Focus::Runs;
                match (k.code, k.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => return Ok(ExitCode::SUCCESS),
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(ExitCode::SUCCESS),
                    (KeyCode::Tab, _) if ui.show_runs() => {
                        ui.focus = if on_runs { Focus::Tasks } else { Focus::Runs };
                    }
                    // Arrows drive the focused pane: run selection or task scroll.
                    (KeyCode::Up | KeyCode::Char('k'), _) if on_runs => ui.select_prev(),
                    (KeyCode::Down | KeyCode::Char('j'), _) if on_runs => ui.select_next(),
                    (KeyCode::Up | KeyCode::Char('k'), _) => {
                        ui.tasks_scroll = ui.tasks_scroll.saturating_sub(1)
                    }
                    (KeyCode::Down | KeyCode::Char('j'), _) => {
                        ui.tasks_scroll = ui.tasks_scroll.saturating_add(1)
                    }
                    (KeyCode::PageUp, _) if !on_runs => {
                        ui.tasks_scroll = ui.tasks_scroll.saturating_sub(10)
                    }
                    (KeyCode::PageDown, _) if !on_runs => {
                        ui.tasks_scroll = ui.tasks_scroll.saturating_add(10)
                    }
                    (KeyCode::Home | KeyCode::Char('g'), _) if on_runs => ui.switch_to(0),
                    (KeyCode::Home | KeyCode::Char('g'), _) => ui.tasks_scroll = 0,
                    // Number keys jump straight to a run, regardless of focus.
                    (KeyCode::Char(c), _) if c.is_ascii_digit() && c != '0' => {
                        ui.switch_to(c as usize - '1' as usize);
                    }
                    _ => {}
                }
            }
        }

        // Refresh the run list periodically (new runs, status changes).
        if last_list.elapsed() >= list_every {
            ui.refresh_runs(project_root);
            last_list = std::time::Instant::now();
        }

        // Reload the current run's state when its state.json advances.
        if let Some(dir) = ui.current_dir().map(Path::to_path_buf) {
            if dashboard::state_mtime(&dir) != ui.last_mtime {
                ui.reload();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn draw(f: &mut Frame, ui: &mut WatchUi) {
    let area = f.area();
    let Some(model) = ui.model.clone() else {
        let p = Paragraph::new("loading run…").block(bordered("Pilot"));
        f.render_widget(p, area);
        return;
    };

    // Rows: header (4) · grid (rest) · footer (1).
    let rows = Layout::vertical([
        Constraint::Length(4),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    render_header(f, rows[0], &model.header);

    // Grid: top row then Live log full width. When >1 run is active, the
    // top row gains a Runs sidebar on the left: Runs | Tasks | Agents.
    let grid = Layout::vertical([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(rows[1]);

    let work = if ui.show_runs() {
        let cols =
            Layout::horizontal([Constraint::Length(22), Constraint::Min(0)]).split(grid[0]);
        render_runs(f, cols[0], &ui.runs, ui.current, ui.focus, ui.frame);
        cols[1]
    } else {
        grid[0]
    };
    let top = Layout::horizontal([Constraint::Percentage(56), Constraint::Percentage(44)]).split(work);

    let tasks_focused = ui.show_runs() && ui.focus == Focus::Tasks;
    render_tasks(
        f,
        top[0],
        &model.tasks,
        &mut ui.tasks_scroll,
        ui.frame,
        tasks_focused,
    );
    render_agents(f, top[1], &model.agents, ui.frame);
    render_log(f, grid[1], &model.log);
    render_footer(f, rows[2], ui.finished, ui.show_runs());
}

/// The Runs sidebar: one row per active run, the watched one marked and the
/// focused selection highlighted. Running runs animate the spinner.
fn render_runs(
    f: &mut Frame,
    area: Rect,
    runs: &[RunSummary],
    current: usize,
    focus: Focus,
    frame: u64,
) {
    let lines: Vec<Line> = runs
        .iter()
        .enumerate()
        .map(|(i, r)| run_line(r, i == current, frame))
        .collect();
    let title = format!("Runs ({})", runs.len());
    f.render_widget(
        Paragraph::new(lines).block(bordered_focused(&title, focus == Focus::Runs)),
        area,
    );
}

fn run_line(r: &RunSummary, is_current: bool, frame: u64) -> Line<'static> {
    let (glyph, color) = run_status_glyph(r.status, frame);
    let marker = if is_current { "▸" } else { " " };
    let label = short_run_label(&r.run_id);
    let mut style = Style::default();
    if is_current {
        style = style.add_modifier(Modifier::BOLD);
    }
    Line::from(vec![
        Span::styled(format!("{marker}{glyph} "), Style::default().fg(color)),
        Span::styled(label, style),
        Span::styled(format!("  {}/{}", r.done, r.total), dim()),
    ])
}

/// Short, stable label for a run id — the trailing random suffix, which is
/// what disambiguates same-minute runs (`2026-07-01-0707-hq27zr` → `hq27zr`).
fn short_run_label(run_id: &str) -> String {
    run_id.rsplit('-').next().unwrap_or(run_id).to_string()
}

fn render_header(f: &mut Frame, area: Rect, h: &HeaderInfo) {
    let (status_label, status_color) = run_status_style(h.status);
    let mut counts = vec![
        Span::styled(format!("{}", h.done), Style::default().fg(Color::Green)),
        Span::raw(format!("/{} done", h.total)),
    ];
    if h.running > 0 {
        counts.push(Span::raw(" · "));
        counts.push(Span::styled(
            format!("{}▶ running", h.running),
            Style::default().fg(Color::Cyan),
        ));
    }
    if h.failed > 0 {
        counts.push(Span::raw(" · "));
        counts.push(Span::styled(
            format!("{}✗ failed", h.failed),
            Style::default().fg(Color::Red),
        ));
    }
    if h.blocked > 0 {
        counts.push(Span::raw(" · "));
        counts.push(Span::styled(
            format!("{}‼ blocked", h.blocked),
            Style::default().fg(Color::Yellow),
        ));
    }

    let line1 = Line::from(
        [
            vec![
                Span::styled(
                    h.run_id.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(status_label, Style::default().fg(status_color)),
                Span::raw("   "),
            ],
            counts,
        ]
        .concat(),
    );

    let mut meta = vec![
        Span::styled("elapsed ", dim()),
        Span::raw(h.elapsed_secs.map(fmt_dur).unwrap_or_else(|| "—".into())),
        Span::styled("  ·  spend ", dim()),
        Span::styled(format!("${:.2}", h.usd), Style::default().fg(Color::Green)),
        Span::styled("  ·  branch ", dim()),
        Span::raw(h.branch.clone()),
    ];
    if !h.base_short.is_empty() {
        meta.push(Span::styled("  ·  base ", dim()));
        meta.push(Span::raw(h.base_short.clone()));
    }

    let p = Paragraph::new(vec![line1, Line::from(meta)]).block(bordered("Pilot"));
    f.render_widget(p, area);
}

fn render_tasks(
    f: &mut Frame,
    area: Rect,
    tasks: &[TaskRow],
    scroll: &mut u16,
    frame: u64,
    focused: bool,
) {
    let lines: Vec<Line> = tasks.iter().map(|t| task_line(t, frame)).collect();
    // Clamp scroll so we never page past the end.
    let inner_h = area.height.saturating_sub(2);
    let max = (lines.len() as u16).saturating_sub(inner_h);
    if *scroll > max {
        *scroll = max;
    }
    let title = format!("Tasks ({})", tasks.len());
    let p = Paragraph::new(lines)
        .block(bordered_focused(&title, focused))
        .scroll((*scroll, 0));
    f.render_widget(p, area);
}

fn render_agents(f: &mut Frame, area: Rect, agents: &[AgentRow], frame: u64) {
    let lines: Vec<Line> = if agents.is_empty() {
        vec![Line::from(Span::styled("(no agents yet)", dim()))]
    } else {
        agents.iter().map(|a| agent_line(a, frame)).collect()
    };
    let title = format!("Agents ({})", agents.len());
    f.render_widget(Paragraph::new(lines).block(bordered(&title)), area);
}

fn render_log(f: &mut Frame, area: Rect, log: &[LogSeverityLine]) {
    let lines: Vec<Line> = log.iter().map(log_line).collect();
    // Stick to the bottom so the newest events are always visible.
    let inner_h = area.height.saturating_sub(2);
    let scroll = (lines.len() as u16).saturating_sub(inner_h);
    f.render_widget(
        Paragraph::new(lines)
            .block(bordered("Live log"))
            .scroll((scroll, 0)),
        area,
    );
}

fn render_footer(f: &mut Frame, area: Rect, finished: bool, show_runs: bool) {
    let cyan = Style::default().fg(Color::Cyan);
    let mut spans = Vec::new();
    if show_runs {
        spans.push(Span::styled(" Tab", cyan));
        spans.push(Span::styled(" focus · ", dim()));
        spans.push(Span::styled("↑/↓", cyan));
        spans.push(Span::styled(" select/scroll · ", dim()));
        spans.push(Span::styled("1-9", cyan));
        spans.push(Span::styled(" run · ", dim()));
    } else {
        spans.push(Span::styled(" ↑/↓", cyan));
        spans.push(Span::styled(" scroll tasks · ", dim()));
    }
    spans.push(Span::styled("q", cyan));
    spans.push(Span::styled(" quit ", dim()));

    // With a sidebar the switch hints stay useful even after the watched
    // run finishes; only the single-run view collapses to the exit note.
    let hint = if finished && !show_runs {
        Line::from(Span::styled(
            " run finished — press q to exit ",
            Style::default().fg(Color::Green),
        ))
    } else {
        Line::from(spans)
    };
    f.render_widget(Paragraph::new(hint), area);
}

// ---------------------------------------------------------------------------
// Row → styled Line
// ---------------------------------------------------------------------------

fn task_line(t: &TaskRow, frame: u64) -> Line<'static> {
    // In-progress rows get an animated circular spinner so the currently
    // worked task is obvious at a glance; everything else keeps its glyph.
    let (glyph, color) = match t.status {
        TaskStatus::InProgress => (spinner(frame), task_status_style(t.status).1),
        other => task_status_style(other),
    };
    let mut spans = vec![
        Span::styled(format!(" {glyph} "), Style::default().fg(color)),
        Span::styled(
            format!("{:<4}", t.id),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("[{}] ", t.role), dim()),
        Span::raw(t.title.clone()),
    ];

    let mut meta: Vec<String> = Vec::new();
    if let Some(a) = &t.agent {
        meta.push(a.clone());
    }
    if !t.deps.is_empty() {
        meta.push(format!("deps: {}", t.deps.join(",")));
    }
    if t.writes > 0 {
        meta.push(format!("✎{}", t.writes));
    }
    if t.usd > 0.0 {
        meta.push(format!("${:.2}", t.usd));
    }
    if let Some(secs) = t.elapsed_secs {
        meta.push(fmt_dur(secs));
    }
    if t.attempts > 1 {
        meta.push(format!("try{}", t.attempts));
    }
    if !meta.is_empty() {
        spans.push(Span::styled(format!("  · {}", meta.join(" · ")), dim()));
    }
    Line::from(spans)
}

fn agent_line(a: &AgentRow, frame: u64) -> Line<'static> {
    // Working agents animate the same spinner as their in-progress task.
    let (glyph, color) = match a.status {
        AgentStatus::InProgress => (spinner(frame), agent_status_style(a.status).1),
        other => agent_status_style(other),
    };
    // The friendly name gets a stable per-agent colour (hashed from the
    // name) so the same worker is easy to track at a glance.
    let name_color = agent_color(&a.name);
    let mut spans = vec![
        Span::styled(format!(" {glyph} "), Style::default().fg(color)),
        Span::styled(
            format!("{} ", a.name),
            Style::default().fg(name_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("[{}] ", a.role), dim()),
    ];
    let task = a
        .task
        .as_deref()
        .map(|t| format!("task={t}"))
        .unwrap_or_else(|| "idle".into());
    spans.push(Span::raw(task));

    let mut meta: Vec<String> = Vec::new();
    if let Some(tool) = &a.tool {
        meta.push(format!("▸{tool}"));
    }
    if let Some(p) = a.pid {
        meta.push(format!("pid={p}"));
    }
    if let Some(secs) = a.uptime_secs {
        meta.push(fmt_dur(secs));
    }
    if a.usd > 0.0 {
        meta.push(format!("${:.2}", a.usd));
    }
    if !meta.is_empty() {
        spans.push(Span::styled(format!("  · {}", meta.join(" · ")), dim()));
    }
    Line::from(spans)
}

/// The dashboard's `LogRow` (aliased here for readability).
type LogSeverityLine = arccode_autonomous::dashboard::LogRow;

fn log_line(r: &LogSeverityLine) -> Line<'static> {
    let color = match r.severity {
        LogSeverity::Ok => Color::Green,
        LogSeverity::Warn => Color::Yellow,
        LogSeverity::Error => Color::Red,
        LogSeverity::Info => Color::Gray,
    };
    Line::from(Span::styled(r.text.clone(), Style::default().fg(color)))
}

// ---------------------------------------------------------------------------
// Style helpers
// ---------------------------------------------------------------------------

fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

/// Circular progress spinner frame for the given animation tick. Rotates
/// through the four quarter-filled circles to read as a spinning disc.
fn spinner(frame: u64) -> char {
    const FRAMES: [char; 4] = ['◐', '◓', '◑', '◒'];
    FRAMES[(frame as usize) % FRAMES.len()]
}

/// Stable colour for an agent, hashed from its name. Red/yellow/green are
/// left out of the palette so agent colours don't read as status signals.
fn agent_color(name: &str) -> Color {
    const PALETTE: [Color; 6] = [
        Color::Cyan,
        Color::Magenta,
        Color::Blue,
        Color::LightCyan,
        Color::LightMagenta,
        Color::LightBlue,
    ];
    // Tiny FNV-1a so the mapping is stable across frames and processes.
    let mut h: u32 = 0x811c_9dc5;
    for b in name.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    PALETTE[(h as usize) % PALETTE.len()]
}

fn bordered(title: &str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
}

/// Like [`bordered`], but a focused pane gets a brighter cyan border so it's
/// clear which pane the arrow keys are driving.
fn bordered_focused(title: &str, focused: bool) -> Block<'static> {
    let block = bordered(title);
    if focused {
        block.border_style(Style::default().fg(Color::Cyan))
    } else {
        block
    }
}

/// Status glyph + colour for a run in the sidebar. Non-terminal runs animate
/// the spinner; terminal runs get a static mark.
fn run_status_glyph(s: RunStatus, frame: u64) -> (char, Color) {
    match s {
        RunStatus::Planning => (spinner(frame), Color::Blue),
        RunStatus::AwaitingApproval => (spinner(frame), Color::Yellow),
        RunStatus::Running => (spinner(frame), Color::Cyan),
        RunStatus::Merging => (spinner(frame), Color::Magenta),
        RunStatus::Done => ('✓', Color::Green),
        RunStatus::Failed => ('✗', Color::Red),
        RunStatus::Aborted => ('⊘', Color::Yellow),
    }
}

fn task_status_style(s: TaskStatus) -> (char, Color) {
    match s {
        TaskStatus::Pending => ('·', Color::DarkGray),
        TaskStatus::Todo => ('○', Color::Blue),
        TaskStatus::InProgress => ('↻', Color::Cyan),
        TaskStatus::Review => ('◇', Color::Magenta),
        TaskStatus::Done => ('✓', Color::Green),
        TaskStatus::Failed => ('✗', Color::Red),
        TaskStatus::Blocked => ('‼', Color::Yellow),
    }
}

fn agent_status_style(s: AgentStatus) -> (char, Color) {
    match s {
        AgentStatus::Idle => ('·', Color::DarkGray),
        AgentStatus::InProgress => ('↻', Color::Cyan),
        AgentStatus::Done => ('✓', Color::Green),
        AgentStatus::Failed => ('✗', Color::Red),
        AgentStatus::Aborted => ('⊘', Color::Yellow),
    }
}

fn run_status_style(s: RunStatus) -> (String, Color) {
    let (label, color) = match s {
        RunStatus::Planning => ("planning", Color::Blue),
        RunStatus::AwaitingApproval => ("awaiting-approval", Color::Yellow),
        RunStatus::Running => ("running", Color::Cyan),
        RunStatus::Merging => ("merging", Color::Magenta),
        RunStatus::Done => ("done", Color::Green),
        RunStatus::Failed => ("failed", Color::Red),
        RunStatus::Aborted => ("aborted", Color::Yellow),
    };
    (label.to_string(), color)
}

/// Compact duration: `45s`, `1m20s`, `2h03m`.
fn fmt_dur(secs: i64) -> String {
    let s = secs.max(0);
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m{:02}s", s / 60, s % 60)
    } else {
        format!("{}h{:02}m", s / 3600, (s % 3600) / 60)
    }
}

// ---------------------------------------------------------------------------
// Terminal setup / teardown
// ---------------------------------------------------------------------------

fn setup() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}

fn teardown(terminal: &mut Term) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_cycles_through_four_circular_frames() {
        let frames: Vec<char> = (0..4).map(spinner).collect();
        assert_eq!(frames, vec!['◐', '◓', '◑', '◒']);
        // Wraps around and stays in-bounds for large ticks.
        assert_eq!(spinner(4), spinner(0));
        assert_eq!(spinner(4_000_001), spinner(1));
    }

    fn summary(id: &str, status: RunStatus) -> RunSummary {
        RunSummary {
            run_id: id.into(),
            dir: std::path::PathBuf::from(id),
            status,
            goal: String::new(),
            done: 0,
            total: 0,
        }
    }

    fn sample_model() -> DashboardModel {
        DashboardModel {
            header: HeaderInfo {
                run_id: "2026-07-01-0707-hq27zr".into(),
                status: RunStatus::Running,
                done: 0,
                running: 1,
                failed: 0,
                blocked: 0,
                total: 1,
                usd: 0.0,
                elapsed_secs: Some(10),
                branch: "b".into(),
                base_short: "abc".into(),
            },
            tasks: vec![],
            agents: vec![],
            log: vec![],
        }
    }

    fn render_to_string(ui: &mut WatchUi, w: u16, h: u16) -> String {
        use ratatui::backend::TestBackend;
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| draw(f, ui)).unwrap();
        let buf = term.backend().buffer().clone();
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    #[test]
    fn sidebar_shows_and_lists_runs_when_multiple_active() {
        let runs = vec![
            summary("2026-07-01-0707-hq27zr", RunStatus::Running),
            summary("2026-07-01-0709-a1b2c3", RunStatus::Running),
        ];
        let mut ui = WatchUi::new(runs, 0);
        ui.model = Some(sample_model());
        let s = render_to_string(&mut ui, 120, 30);
        assert!(s.contains("Runs (2)"), "sidebar title missing:\n{s}");
        assert!(s.contains("hq27zr"), "run 1 label missing");
        assert!(s.contains("a1b2c3"), "run 2 label missing");
    }

    #[test]
    fn sidebar_hidden_for_a_single_run() {
        let mut ui = WatchUi::new(vec![summary("only", RunStatus::Running)], 0);
        ui.model = Some(sample_model());
        let s = render_to_string(&mut ui, 120, 30);
        assert!(!s.contains("Runs ("), "sidebar should be hidden:\n{s}");
    }

    #[test]
    fn short_run_label_is_the_trailing_suffix() {
        assert_eq!(short_run_label("2026-07-01-0707-hq27zr"), "hq27zr");
        assert_eq!(short_run_label("nodashes"), "nodashes");
    }

    #[test]
    fn active_plus_filters_to_non_terminal_runs() {
        let all = vec![
            summary("a", RunStatus::Running),
            summary("b", RunStatus::Done),
            summary("c", RunStatus::Planning),
        ];
        let list = active_plus(all, None);
        let ids: Vec<&str> = list.iter().map(|r| r.run_id.as_str()).collect();
        assert_eq!(ids, vec!["a", "c"], "finished run 'b' filtered out");
    }

    #[test]
    fn active_plus_keeps_the_watched_run_even_if_finished() {
        let all = vec![
            summary("a", RunStatus::Running),
            summary("b", RunStatus::Done),
        ];
        let list = active_plus(all, Some("b"));
        assert!(
            list.iter().any(|r| r.run_id == "b"),
            "watched-but-finished run stays visible"
        );
    }

    #[test]
    fn active_plus_falls_back_to_all_when_none_active() {
        let all = vec![
            summary("a", RunStatus::Done),
            summary("b", RunStatus::Failed),
        ];
        let list = active_plus(all, None);
        assert_eq!(list.len(), 2, "shows everything rather than a blank UI");
    }

    #[test]
    fn run_status_glyph_spins_only_for_active_runs() {
        assert_eq!(run_status_glyph(RunStatus::Running, 0).0, spinner(0));
        assert_eq!(run_status_glyph(RunStatus::Done, 0).0, '✓');
        assert_eq!(run_status_glyph(RunStatus::Failed, 0).0, '✗');
    }

    #[test]
    fn agent_colour_is_stable_and_excludes_status_hues() {
        // Deterministic across calls…
        assert_eq!(agent_color("brave_otter"), agent_color("brave_otter"));
        // …and never red/green/yellow, which are reserved for status.
        for name in ["brave_otter", "lucid_lynx", "swift_heron", "calm_panda"] {
            let c = agent_color(name);
            assert!(
                !matches!(c, Color::Red | Color::Green | Color::Yellow),
                "{name} got a status-reserved colour: {c:?}"
            );
        }
    }
}
