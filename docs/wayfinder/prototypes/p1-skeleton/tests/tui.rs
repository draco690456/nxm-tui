//! Integration tests for the P1 skeleton (RULES: tests under `tests/`).
//! Exercises the pure logic in `p1_tui_skeleton::status_line` — the
//! ratatui event loop is left to manual inspection (terminal-bound).

use p1_tui_skeleton::status_line;

#[test]
fn status_line_contains_all_parts_with_endpoint() {
    let s = status_line("Ciao Nexum", Some("http://127.0.0.1:8080"), 3);
    assert!(s.contains("Ciao Nexum"));
    assert!(s.contains("endpoint=http://127.0.0.1:8080"));
    assert!(s.contains("files: 3"));
    assert!(s.contains("press q to quit"));
}

#[test]
fn status_line_no_endpoint_shows_empty_set() {
    let s = status_line("Hi", None, 0);
    assert!(s.contains("files: 0"));
    assert!(s.contains("endpoint=\u{2205}"));
}
