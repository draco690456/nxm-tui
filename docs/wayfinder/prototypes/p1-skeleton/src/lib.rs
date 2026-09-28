//! P1 prototype — pure-Rust Nexum TUI skeleton.
//!
//! A *throwaway* `ratatui` + `crossterm` scaffold (the "create Application →
//! open window → root view → render a `div`" shape mirrored from
//! `vendor/gpui/examples/hello_world.rs`, in ratatui terms). It opens a
//! terminal, paints a title bar + a status line, and quits on `q`.
//!
//! Its only job is to give something concrete to react to before the real
//! crate (placement decided in D3) is built. It deliberately obeys
//! `RULES.md` so the real crate can inherit the shape.
//!
//! Library half (pure, testable fns). The binary lives in `src/main.rs`.
//! Run: `cargo run -p p1-tui-skeleton -- --greeting "Ciao Nexum" --endpoint http://127.0.0.1:8080`

/// Tracing target — mirrors the per-functionality convention from `RULES.md`
/// (`nexum::<module>::<feature>`). The real crate should obtain the subscriber
/// via `nxm-shared`; this throwaway uses a minimal `tracing-subscriber` init.
pub const TARGET: &str = "nexum::tui::runtime";

/// Compose the one-line status bar.
///
/// Kept pure (no terminal state) so it is unit-testable. `RULES.md` mandates
/// tests under `tests/`, never inline — see `tests/tui.rs`.
///
/// # Example
/// ```
/// let s = p1_tui_skeleton::status_line("Ciao Nexum", None, 0);
/// assert!(s.contains("Ciao Nexum") && s.contains("files: 0"));
/// ```
pub fn status_line(greeting: &str, endpoint: Option<&str>, files: usize) -> String {
    let ep = match endpoint {
        Some(e) => format!("endpoint={e}"),
        None => "endpoint=\u{2205}".to_string(),
    };
    format!("{greeting} \u{00b7} {ep} \u{00b7} files: {files} \u{00b7} press q to quit")
}
