use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::io::stdout;
use std::time::{Duration, Instant};

use crate::scanner::{self, PortEntry};

pub fn run_tui() -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run_loop(&mut terminal);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

struct AppState {
    entries: Vec<PortEntry>,
    table_state: TableState,
    last_refresh: Instant,
    status_msg: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            table_state: TableState::default(),
            last_refresh: Instant::now() - Duration::from_secs(10), // force immediate refresh
            status_msg: String::new(),
        }
    }

    fn refresh(&mut self) {
        match scanner::scan_ports() {
            Ok(entries) => {
                self.entries = entries;
                self.status_msg = format!("{} ports found", self.entries.len());
            }
            Err(e) => {
                self.status_msg = format!("Error: {e}");
            }
        }
        self.last_refresh = Instant::now();

        // Ensure selection stays in bounds
        if self.entries.is_empty() {
            self.table_state.select(None);
        } else if let Some(sel) = self.table_state.selected() {
            if sel >= self.entries.len() {
                self.table_state.select(Some(self.entries.len() - 1));
            }
        }
    }

    fn selected_entry(&self) -> Option<&PortEntry> {
        self.table_state
            .selected()
            .and_then(|i| self.entries.get(i))
    }

    fn move_up(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn move_down(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let mut state = AppState::new();
    state.refresh();
    if !state.entries.is_empty() {
        state.table_state.select(Some(0));
    }

    loop {
        // Auto-refresh every 2 seconds
        if state.last_refresh.elapsed() > Duration::from_secs(2) {
            state.refresh();
        }

        terminal.draw(|f| render(f, &mut state))?;

        // Poll for events with a short timeout so we can auto-refresh
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up | KeyCode::Char('k') => state.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => state.move_down(),
                    KeyCode::Char('x') => {
                        // Kill selected process
                        if let Some(entry) = state.selected_entry().cloned() {
                            state.status_msg =
                                format!("Killing PID {} ({})...", entry.pid, entry.process);
                            terminal.draw(|f| render(f, &mut state))?;

                            // Temporarily exit TUI to kill
                            disable_raw_mode()?;
                            stdout().execute(LeaveAlternateScreen)?;

                            match crate::killer::kill_pid(entry.pid) {
                                Ok(()) => {
                                    state.status_msg =
                                        format!("Killed PID {} ({})", entry.pid, entry.process);
                                }
                                Err(e) => {
                                    state.status_msg = format!("Kill failed: {e}");
                                }
                            }

                            // Re-enter TUI
                            enable_raw_mode()?;
                            stdout().execute(EnterAlternateScreen)?;
                            state.refresh();
                        }
                    }
                    KeyCode::Char('r') => {
                        state.refresh();
                        state.status_msg = "Refreshed.".to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn render(f: &mut Frame, state: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(5),   // table
            Constraint::Length(3), // status bar
        ])
        .split(f.area());

    // Title
    let title = Paragraph::new("  portwatch — port monitor & manager")
        .style(Style::default().fg(Color::Cyan).bold())
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // Table
    let header = Row::new(vec!["PORT", "PID", "PROCESS", "USER", "CPU%", "MEM(MB)", "PROTO"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .bold()
                .add_modifier(Modifier::UNDERLINED),
        )
        .height(1);

    let rows: Vec<Row> = state
        .entries
        .iter()
        .map(|e| {
            Row::new(vec![
                Cell::from(e.port.to_string()).style(Style::default().fg(Color::Green)),
                Cell::from(e.pid.to_string()).style(Style::default().fg(Color::Cyan)),
                Cell::from(e.process.clone()),
                Cell::from(e.user.clone()),
                Cell::from(format!("{:.1}", e.cpu)),
                Cell::from(format!("{:.1}", e.mem)),
                Cell::from(e.proto.clone()).style(Style::default().fg(Color::Magenta)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),  // PORT
            Constraint::Length(8),  // PID
            Constraint::Length(18), // PROCESS
            Constraint::Length(12), // USER
            Constraint::Length(7),  // CPU%
            Constraint::Length(9),  // MEM(MB)
            Constraint::Length(6),  // PROTO
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(" Listening Ports "))
    .row_highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
    .highlight_symbol(">> ");

    f.render_stateful_widget(table, chunks[1], &mut state.table_state);

    // Status bar
    let status = Paragraph::new(format!(
        " {} | q: quit  ↑↓/jk: navigate  x: kill  r: refresh",
        state.status_msg
    ))
    .style(Style::default().fg(Color::White))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, chunks[2]);
}
