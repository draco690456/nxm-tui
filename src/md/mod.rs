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
pub mod syntax;
pub mod theme;
mod style_adapt;

use ratatui::text::Line;

/// Render markdown `content` into styled terminal lines at `max_width` display
/// columns, using the single fixed theme.
///
/// Placeholder (T0 scaffold): the real pipeline is filled in by T3–T8. Until
/// then this returns an empty render so the module compiles and can be wired
/// behind the `USE_NEW_MD` rollout flag without affecting `render_markdown`.
///
/// ```text
/// use nxm_tui::md::render;
/// let _lines = render("hello", 40);
/// ```
pub fn render(_content: &str, _max_width: usize) -> Vec<Line<'static>> {
    // Pipeline assembled incrementally across T3–T8 (parse → render → output).
    Vec::new()
}
