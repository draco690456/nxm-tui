//! Rich markdown renderer (V3) — adopted from grok-build's `xai-grok-markdown`.
//!
//! Derived from xai-grok-markdown (xai-org/grok-build, Apache-2.0). This is a
//! copy-identical-in-behaviour port, kept 100% pure-Rust and independent of the
//! upstream `xai-*` workspace: the pulldown-cmark pipeline, element rendering,
//! styling and streaming are reproduced faithfully; `syntect` stays on
//! `default-fancy` (no onig/C) and styles are bridged through `anstyle`.
//!
//! See `docs/wayfinder/tickets/V3-render-adoption.md` for the full copy plan
//! (tickets T0–T16). This module is populated piece by piece:
//!
//! - `core`     — pulldown-cmark options + offset events (← xai-grok-markdown-core)
//! - `style`    — per-element styles (← style.rs)
//! - `style_adapt` — adapt_style identity stub (← style_adapt.rs)
//! - `theme`    — the single fixed theme (anstyle)
//! - `parse`    — markdown → intermediate events (← parse.rs)
//! - `buffers`  — output accumulation (← buffers.rs)
//! - `render`   — events → styled lines (← render.rs)
//! - `output`   — line assembly (← output.rs)
//! - `wrap`     — cell-width wrapping (← wrapping.rs)
//! - `links`    — URL scan + OSC 8 hyperlinks (← url_scan.rs/hyperlinks.rs)
//! - `streaming`— incremental render for the token stream (← streaming.rs)
//!
//! Attribution: `THIRD-PARTY-NOTICES.md` + per-file `//! Derived from ...` headers.

pub mod style;
pub mod core;
pub mod buffers;
pub mod buffers_table;
pub mod checkpoint;
pub mod hyperlinks;
pub mod latex;
pub mod line_utils;
pub mod open_code_highlighter;
pub mod output;
pub mod parse;
pub mod parse_end;
pub mod parse_events;
pub mod parse_inline;
pub mod parse_start;
pub mod parse_table;
pub mod parse_table_fmt;
pub mod parse_table_word;
pub mod render;
pub mod render_blocks;
pub mod render_events;
pub mod render_segments;
pub mod source_map;
pub mod syntax;
pub mod theme;
pub mod url_scan;
pub mod wrap;
pub mod wrap_input;
pub mod wrap_line;
mod style_adapt;

use ratatui::text::Line;

use crate::md::buffers::MarkdownBuffers;
use crate::md::parse::MarkdownParser;
use crate::md::syntax::Syntect;
use crate::md::theme::fixed_style;

/// Render markdown `content` into styled terminal lines using the real V3
/// pipeline (`MarkdownParser::parse` → `render_ratatui`), with the single
/// fixed theme and best-effort syntect highlighting.
///
/// `max_width` bounds the output: GFM table columns are shrunk
/// proportionally (`max_table_width`) and every rendered line is word-wrapped
/// to `max_width` (T10, closes Fase 2). Table lines are never wrapped (the
/// wrap engine skips them, so column alignment survives); prose and code
/// content wrap with their styles preserved.
///
/// ```text
/// use nxm_tui::md::render;
/// let lines = render("# Titolo\n\ntesto **bold**", 80);
/// assert!(!lines.is_empty());
/// ```
pub fn render(content: &str, max_width: usize) -> Vec<Line<'static>> {
    // Fresh per-call buffers: the legacy sync call sites reuse nothing here.
    let mut buffers = MarkdownBuffers::new();
    // Syntect::new() is Option: code highlighting is best-effort, so a failed
    // syntax/theme load still renders (plain code) instead of unwrapping.
    let syntect = Syntect::new();
    let ms = fixed_style();
    let mut parsed = MarkdownParser::new(content, ms, &mut buffers, syntect.as_ref())
        .max_table_width(Some(max_width))
        .parse();
    let (output, _checkpoint) = parsed.render_ratatui(true); // pretty = true
    // Wrap every line to the pane width (T10). Table lines are skipped by the
    // wrap engine (`is_table_line`), so pre-formatted tables stay whole; prose
    // and code content wrap instead of overflowing (was 182 cols at width 40).
    crate::md::wrap_line::word_wrap_lines(output.lines, max_width)
}
