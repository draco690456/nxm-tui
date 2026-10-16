//! Tests for `src/md/style.rs` and `src/md/style_adapt.rs`.
//!
//! The `STYLE` constant is the faithful port of the upstream
//! `test_style` module (behind `#[cfg(any(test, fuzzing))]` in
//! `xai-grok-markdown/src/style.rs`, Apache-2.0), kept here as an external
//! integration test per RULES.md (no inline `#[cfg(test)]` modules).

use anstyle::{Color, Effects, RgbColor, Style};

use nxm_tui::md::style::{MarkdownStyle, TableBorders, all_hidden, merge_styles};
use nxm_tui::md::style_adapt::adapt_style;

/// A minimal style for testing with no colors (ex-upstream `test_style::STYLE`).
pub const STYLE: MarkdownStyle = MarkdownStyle {
    heading_inner: [Style::new().bold(); 6],
    heading_outer: [Style::new().dimmed().hidden(); 6],
    strong_inner: Style::new().bold(),
    strong_outer: Style::new().dimmed().hidden(),
    emphasis_inner: Style::new().italic(),
    emphasis_outer: Style::new().dimmed().hidden(),
    strikethrough_inner: Style::new().strikethrough(),
    strikethrough_outer: Style::new().dimmed().hidden(),
    inline_code_inner: Style::new().bold(),
    inline_code_outer: Style::new().dimmed().hidden(),
    blockquote_outer: Style::new().dimmed(),
    task_checked: Style::new(),
    task_unchecked: Style::new().dimmed(),
    list_item: Style::new().dimmed(),
    rule: Style::new(),
    link_outer: Style::new(),
    link_text: Style::new().bold(),
    link_url: Style::new().dimmed(),
    link_title: Style::new(),
    code_outer: Style::new().dimmed().hidden(),
    code_language: Style::new().hidden(),
    code_untagged: Style::new(),
    code_background: Style::new(),
    table_outer: Style::new().bold(),
    text: Style::new(),
    math: Style::new().italic(),
};

// ---------------------------------------------------------------------------
// MarkdownStyle field contract
// ---------------------------------------------------------------------------

#[test]
fn markdown_style_has_all_upstream_fields() {
    // Constructing via the const proves every field exists with the expected type.
    let _ = STYLE;

    // Spot-check the effects the ticket requires.
    assert!(STYLE.strong_inner.get_effects().contains(Effects::BOLD));
    assert!(STYLE.emphasis_inner.get_effects().contains(Effects::ITALIC));
    assert!(STYLE
        .strikethrough_inner
        .get_effects()
        .contains(Effects::STRIKETHROUGH));
}

#[test]
fn markdown_style_default_is_plain() {
    let def = MarkdownStyle::default();
    assert!(def.text.get_effects().is_plain());
    assert!(def.strong_inner.get_effects().is_plain());
}

// ---------------------------------------------------------------------------
// TableBorders
// ---------------------------------------------------------------------------

#[test]
fn table_borders_box_chars() {
    assert_eq!(TableBorders::BOX.horizontal(), '─');
    assert_eq!(TableBorders::BOX.vertical(), '│');
    assert_eq!(TableBorders::BOX.cross(), '┼');
}

#[test]
fn table_borders_ascii_chars() {
    assert_eq!(TableBorders::ASCII.horizontal(), '-');
    assert_eq!(TableBorders::ASCII.vertical(), '|');
    assert_eq!(TableBorders::ASCII.cross(), '+');
}

#[test]
fn table_borders_double_chars() {
    assert_eq!(TableBorders::DOUBLE.horizontal(), '═');
    assert_eq!(TableBorders::DOUBLE.vertical(), '║');
    assert_eq!(TableBorders::DOUBLE.cross(), '╬');
}

#[test]
fn table_borders_default_is_box() {
    assert_eq!(TableBorders::default(), TableBorders::BOX);
}

#[test]
fn table_borders_new_preserves_chars() {
    let t = TableBorders::new(['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k']);
    assert_eq!(t.horizontal(), 'a');
    assert_eq!(t.vertical(), 'b');
}

// ---------------------------------------------------------------------------
// merge_styles
// ---------------------------------------------------------------------------

#[test]
fn merge_styles_strips_hidden() {
    let bold = Style::new().bold();
    let hidden = Style::new().hidden();
    let merged = merge_styles([Some(bold), Some(hidden)]);

    // HIDDEN must be stripped from the final result; BOLD preserved.
    assert!(!merged.get_effects().contains(Effects::HIDDEN));
    assert!(merged.get_effects().contains(Effects::BOLD));
}

#[test]
fn merge_styles_preserves_fg_color() {
    let fg = Style::new().fg_color(Some(Color::Rgb(RgbColor(255, 100, 50))));
    let merged = merge_styles([Some(fg)]);

    assert_eq!(
        merged.get_fg_color(),
        Some(Color::Rgb(RgbColor(255, 100, 50)))
    );
}

#[test]
fn merge_styles_none_is_passthrough() {
    // Only `None` items → plain default.
    let merged = merge_styles([None, None]);
    assert!(merged.get_effects().is_plain());
    assert!(merged.get_fg_color().is_none());
}

// ---------------------------------------------------------------------------
// all_hidden
// ---------------------------------------------------------------------------

#[test]
fn all_hidden_true_when_every_style_is_hidden() {
    let h = Style::new().hidden();
    assert!(all_hidden([Some(h)]));
    assert!(all_hidden([Some(h), Some(Style::new().hidden())]));
}

#[test]
fn all_hidden_false_when_some_style_is_visible() {
    let hidden = Style::new().hidden();
    let bold = Style::new().bold();
    assert!(!all_hidden([Some(hidden), Some(bold)]));
    assert!(!all_hidden([Some(bold)]));
}

#[test]
fn all_hidden_false_when_empty() {
    let empty: Vec<Option<Style>> = Vec::new();
    assert!(!all_hidden(empty));
}

#[test]
fn all_hidden_none_items_not_hidden() {
    // None counts as "not hidden", so the result is false.
    assert!(!all_hidden([None]));
    assert!(!all_hidden([None, None]));
}

// ---------------------------------------------------------------------------
// adapt_style (identity placeholder)
// ---------------------------------------------------------------------------

#[test]
fn adapt_style_is_identity() {
    let s = Style::new().fg_color(Some(Color::Rgb(RgbColor(255, 100, 50))));
    assert_eq!(adapt_style(s), s);
}

#[test]
fn adapt_style_identity_through_adapt() {
    // MarkdownStyle::adapt() delegates to adapt_style on every field.
    // Since adapt_style is the identity, adapt() must return an identical style.
    let md = MarkdownStyle {
        text: Style::new().fg_color(Some(Color::Rgb(RgbColor(255, 100, 50)))),
        ..MarkdownStyle::default()
    };
    let adapted = md.adapt();
    assert_eq!(adapted.text, md.text);
    assert_eq!(adapted.strong_inner, md.strong_inner);
}
