//! Binary half of the P1 skeleton (throwaway prototype — see `src/lib.rs`).
//!
//! ratatui 0.29 + crossterm 0.28 terminal loop: title pane + status line,
//! quits on `q`/`Q`/`Esc`. Mirrors the `hello_world` GPUI example shape
//! (Application → window → root view → `div`) translated to ratatui
//! (setup terminal → `Terminal::draw` → `render_widget`).

use std::io;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::{Alignment, Constraint, Direction, Frame, Layout},
    widgets::{Block, Paragraph},
    Terminal,
};
use tracing::{info, info_span};

use p1_tui_skeleton::{status_line, TARGET};

/// CLI args (the `hello_world` example's constants, promoted to clap).
#[derive(Parser, Debug)]
#[command(name = "p1-tui-skeleton", about = "Pure-Rust Nexum TUI prototype")]
struct Args {
    /// Greeting rendered in the title bar.
    #[arg(short, long, default_value = "Nexum TUI — pure Rust prototype")]
    greeting: String,
    /// OpenAI-compatible endpoint the (stub) TUI would call.
    #[arg(short, long)]
    endpoint: Option<String>,
}

fn main() -> Result<()> {
    let _span = info_span!(target: TARGET, "run").entered();
    tracing_subscriber::fmt()
        .with_env_filter("nexum::tui::runtime=debug")
        .init();
    let args = Args::parse();
    let ep = args.endpoint.as_deref().unwrap_or("<none>");
    info!(target: TARGET, "startup greeting={} endpoint={}", args.greeting, ep);
    run(args)
}

fn run(args: Args) -> Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode().context("enable_raw_mode")?;
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    let result = {
        let backend = CrosstermBackend::new(&mut stdout);
        let mut terminal = Terminal::new(backend).context("Terminal::new")?;
        event_loop(&mut terminal, &args)
    };
    // Restore terminal state no matter what (RULES: no unwrap — ignore restore).
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = disable_raw_mode();
    result
}

fn event_loop<B: Backend>(terminal: &mut Terminal<B>, args: &Args) -> Result<()> {
    let _loop_span = info_span!(target: TARGET, "event_loop").entered();
    let mut should_quit = false;
    while !should_quit {
        terminal
            .draw(|f| render(f, args))
            .context("terminal draw")?;
        if let Event::Key(key) = event::read().context("read key")? {
            if key.kind == KeyEventKind::Press && is_quit(&key.code) {
                should_quit = true;
            }
        }
    }
    info!(target: TARGET, "shutdown");
    Ok(())
}

/// Render pass: a titled pane + a centered status line.
fn render(frame: &mut Frame, args: &Args) {
    let status = status_line(&args.greeting, args.endpoint.as_deref(), 0);
    let constraints = [Constraint::Length(3), Constraint::Min(1)];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(constraints)
        .split(frame.area());
    let title = Paragraph::new("nexum tui")
        .alignment(Alignment::Center)
        .block(Block::bordered().title(" NEXUM TUI "));
    let body = Paragraph::new(status.as_str())
        .alignment(Alignment::Center)
        .block(Block::bordered().title(" status "));
    frame.render_widget(title, chunks[0]);
    frame.render_widget(body, chunks[1]);
}

/// `q`, `Q`, or Esc quits — matches the GPUI `hello_world` (exit on close).
fn is_quit(code: &KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}
