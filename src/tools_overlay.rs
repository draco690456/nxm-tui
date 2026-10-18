//! Tools selection overlay for nexum-terminal TUI (T1).
//!
//! Flat checklist of the registry tools with `[x]`/`[ ]` markers (active/
//! inactive) and a cursor on the selected row. No grouping by source yet —
//! that arrives with MCP in T2. Kept in its own module so `overlays.rs`
//! stays under the 300-line RULES cap (same reason as `tool_overlay.rs`).

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render the Tools selection overlay dialog (T1).
pub fn render_tools(f: &mut Frame, app: &App) {
    let area = f.area();
    let w = 56.min(area.width);

    let names = app.tool_registry.tool_names();
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            " Tools",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for (i, name) in names.iter().enumerate() {
        let enabled = app.tool_registry.is_enabled(name);
        let cursor = if i == app.tools_cursor { "> " } else { "  " };
        let mark = if enabled { "[x]" } else { "[ ]" };
        lines.push(Line::from(vec![
            Span::styled(cursor, Style::default().fg(Color::Yellow)),
            Span::styled(
                mark,
                Style::default().fg(if enabled {
                    Color::Green
                } else {
                    Color::DarkGray
                }),
            ),
            Span::raw(" "),
            Span::raw(*name),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Space", Style::default().fg(Color::Magenta)),
        Span::raw(" toggle  "),
        Span::styled("a", Style::default().fg(Color::Magenta)),
        Span::raw(" all  "),
        Span::styled("n", Style::default().fg(Color::Magenta)),
        Span::raw(" none  "),
        Span::styled("Esc", Style::default().fg(Color::DarkGray)),
        Span::raw(" close"),
    ]));

    // Size to content (+2 borders) like render_help.
    let h = ((lines.len() + 2) as u16).min(area.height).max(8);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 3;
    let overlay = Rect::new(x, y, w, h);
    f.render_widget(Clear, overlay);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 100, 100)))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));
    f.render_widget(block, overlay);

    let inner = Rect::new(x + 1, y + 1, w.saturating_sub(2), h.saturating_sub(2));
    f.render_widget(Paragraph::new(lines), inner);
}
