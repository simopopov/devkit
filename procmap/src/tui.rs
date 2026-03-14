use crate::network::NetConn;
use crate::tree::{flatten_tree, FlatRow, TreeNode};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};
use std::collections::HashSet;
use std::io::stdout;

pub fn run_tui(forest: Vec<TreeNode>) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = event_loop(&mut terminal, forest);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

struct AppState {
    rows: Vec<FlatRow>,
    /// Set of PIDs whose children are collapsed.
    collapsed: HashSet<u32>,
    /// Index into the visible rows.
    selected: usize,
    scroll_offset: usize,
}

impl AppState {
    fn new(forest: Vec<TreeNode>) -> Self {
        let rows = flatten_tree(&forest);
        Self {
            rows,
            collapsed: HashSet::new(),
            selected: 0,
            scroll_offset: 0,
        }
    }

    fn visible_rows(&self) -> Vec<&FlatRow> {
        // Walk the flat rows, skipping children of collapsed nodes.
        let mut visible = Vec::new();
        let mut skip_depth: Option<usize> = None;

        for row in &self.rows {
            if let Some(d) = skip_depth {
                if row.depth > d {
                    continue;
                } else {
                    skip_depth = None;
                }
            }
            visible.push(row);
            if self.collapsed.contains(&row.pid) && row.has_children {
                skip_depth = Some(row.depth);
            }
        }
        visible
    }

    fn toggle_collapse(&mut self) {
        let visible = self.visible_rows();
        if let Some(row) = visible.get(self.selected) {
            let pid = row.pid;
            if row.has_children {
                if self.collapsed.contains(&pid) {
                    self.collapsed.remove(&pid);
                } else {
                    self.collapsed.insert(pid);
                }
            }
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        let max = self.visible_rows().len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
    }

    fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.selected - viewport_height + 1;
        }
    }
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, forest: Vec<TreeNode>) -> Result<()> {
    let mut state = AppState::new(forest);

    loop {
        terminal.draw(|frame| draw(frame, &mut state))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up | KeyCode::Char('k') => state.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => state.move_down(),
                    KeyCode::Enter | KeyCode::Char(' ') => state.toggle_collapse(),
                    KeyCode::Home => state.selected = 0,
                    KeyCode::End => {
                        state.selected = state.visible_rows().len().saturating_sub(1);
                    }
                    KeyCode::PageUp => {
                        state.selected = state.selected.saturating_sub(20);
                    }
                    KeyCode::PageDown => {
                        let max = state.visible_rows().len().saturating_sub(1);
                        state.selected = (state.selected + 20).min(max);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, state: &mut AppState) {
    let area = frame.area();

    let block = Block::default()
        .title(" procmap - Process & Network Topology (q: quit, Enter: expand/collapse, arrows: navigate) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let viewport_height = inner.height as usize;
    state.ensure_visible(viewport_height);

    let visible = state.visible_rows();
    let total_rows = visible.len();

    let mut lines: Vec<Line> = Vec::new();

    for (vi, row) in visible
        .iter()
        .enumerate()
        .skip(state.scroll_offset)
        .take(viewport_height)
    {
        let is_selected = vi == state.selected;
        let line = render_row(row, is_selected, state.collapsed.contains(&row.pid));
        lines.push(line);
    }

    let paragraph = Paragraph::new(Text::from(lines));
    frame.render_widget(paragraph, inner);

    // Scrollbar
    if total_rows > viewport_height {
        let mut scrollbar_state = ScrollbarState::new(total_rows.saturating_sub(viewport_height))
            .position(state.scroll_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn render_row<'a>(row: &FlatRow, selected: bool, collapsed: bool) -> Line<'a> {
    let mut spans: Vec<Span<'a>> = Vec::new();

    // Indentation
    let indent = "  ".repeat(row.depth);
    spans.push(Span::raw(indent));

    // Collapse indicator
    if row.has_children {
        let marker = if collapsed { "[+] " } else { "[-] " };
        spans.push(Span::styled(
            marker.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        spans.push(Span::raw("    "));
    }

    // Process name in cyan
    spans.push(Span::styled(
        row.name.clone(),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));

    // PID
    spans.push(Span::styled(
        format!(" ({})", row.pid),
        Style::default().fg(Color::DarkGray),
    ));

    // User
    spans.push(Span::styled(
        format!(" {}", row.user),
        Style::default().fg(Color::Gray),
    ));

    // CPU% - red if high
    let cpu_color = if row.cpu > 50.0 {
        Color::Red
    } else if row.cpu > 10.0 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled(
        format!(" cpu:{:.1}%", row.cpu),
        Style::default().fg(cpu_color),
    ));

    // MEM%
    let mem_color = if row.mem > 10.0 {
        Color::Red
    } else if row.mem > 3.0 {
        Color::Yellow
    } else {
        Color::Green
    };
    spans.push(Span::styled(
        format!(" mem:{:.1}%", row.mem),
        Style::default().fg(mem_color),
    ));

    // Ports in yellow
    if !row.ports.is_empty() {
        let port_str = format_ports_short(&row.ports);
        spans.push(Span::styled(
            format!(" [{}]", port_str),
            Style::default().fg(Color::Yellow),
        ));
    }

    let style = if selected {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    Line::from(spans).style(style)
}

fn format_ports_short(conns: &[NetConn]) -> String {
    let parts: Vec<String> = conns
        .iter()
        .map(|c| {
            if c.state == "LISTEN" {
                format!("{}:{} LISTEN", c.proto, c.local_port)
            } else if !c.remote_addr.is_empty() && c.remote_port > 0 {
                format!(":{}->{}", c.local_port, c.remote_port)
            } else {
                format!(":{}", c.local_port)
            }
        })
        .collect();
    parts.join(", ")
}
