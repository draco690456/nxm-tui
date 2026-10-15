//! Contract tests for `src/highlight.rs` (V2 syntect highlighting).
//!
//! These tests assert the structural contract: multiple styled spans per line
//! for tokenized code, plain-text fallback for unknown/empty language, no
//! panics, and assets loaded once. Visual TTY rendering is the user's
//! responsibility and is not asserted here.

use nxm_tui::highlight::highlight_code;

/// A Rust snippet with distinct tokens (keyword `fn`, ident `main`) should
/// yield at least one line with >= 2 spans of different `Style`.
#[test]
fn rust_snippet_yields_multiple_styles_per_line() {
    let lines = highlight_code("fn main() {}", "rust");
    assert!(!lines.is_empty(), "rust snippet must produce at least one line");

    let has_multi = lines.iter().any(|line| {
        line.len() >= 2
            && line.windows(2).any(|window| window[0].0 != window[1].0)
    });
    assert!(
        has_multi,
        "expected at least one line with >=2 distinct styles (keyword vs ident), got: {lines:?}"
    );
}

/// Unknown language must not panic and must preserve the text (one line,
/// concatenated content == input).
#[test]
fn unknown_lang_plain_text_fallback_preserves_text() {
    let lines = highlight_code("x", "zzz-nope");
    assert_eq!(lines.len(), 1, "unknown lang must yield exactly one line");
    let joined: String = lines
        .iter()
        .flat_map(|line| line.iter().map(|(_, t)| t.as_str()))
        .collect();
    assert_eq!(joined, "x");
}

/// Empty language must not panic and must preserve the text.
#[test]
fn empty_lang_plain_text_fallback_preserves_text() {
    let lines = highlight_code("x", "");
    assert_eq!(lines.len(), 1, "empty lang must yield exactly one line");
    let joined: String = lines
        .iter()
        .flat_map(|line| line.iter().map(|(_, t)| t.as_str()))
        .collect();
    assert_eq!(joined, "x");
}

/// Two consecutive calls must return consistent results (assets loaded once
/// via `OnceLock`, not re-initialized per call).
#[test]
fn assets_loaded_once_consistent_results() {
    let first = highlight_code("fn x() {}", "rust");
    let second = highlight_code("fn x() {}", "rust");
    assert_eq!(first, second);
}
