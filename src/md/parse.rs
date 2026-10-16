//! Derived from xai-grok-markdown/src/parse.rs (xai-org/grok-build, Apache-2.0). Modified: scaffold only (struct/new/builder/parse/ParsedMarkdown + pure helpers); on_event/on_start/on_end/format_table are stubs for T5f/T5g; latex deferred; offset_events from crate::md::core. Visibility-for-testability: ParsedMarkdown fields are `pub` for tests/.

use std::ops::Range;

use anstyle::Style;
use pulldown_cmark::{CowStr, Event, Tag, TagEnd, TextMergeWithOffset};

use crate::md::buffers::MarkdownBuffers;
use crate::md::buffers_table::TableState;
use crate::md::checkpoint::CheckpointKind;
use crate::md::open_code_highlighter::OpenCodeHighlighter;
use crate::md::style::MarkdownStyle;
use crate::md::syntax::Syntect;
use crate::md::core::offset_events;

/// Trait for converting anstyle to ratatui style.
#[allow(dead_code)]
trait StyleInto<T> {
    fn style_into(self) -> T;
}

#[allow(dead_code)]
impl StyleInto<ratatui::style::Style> for Style {
    fn style_into(self) -> ratatui::style::Style {
        use ratatui::style::{Modifier, Style as RStyle};

        let mut style = RStyle::default();

        if let Some(fg) = self.get_fg_color() {
            style = style.fg(anstyle_to_ratatui_color(fg));
        }
        if let Some(bg) = self.get_bg_color() {
            style = style.bg(anstyle_to_ratatui_color(bg));
        }

        let effects = self.get_effects();
        let mut modifiers = Modifier::empty();
        if effects.contains(anstyle::Effects::BOLD) {
            modifiers |= Modifier::BOLD;
        }
        if effects.contains(anstyle::Effects::DIMMED) {
            modifiers |= Modifier::DIM;
        }
        if effects.contains(anstyle::Effects::ITALIC) {
            modifiers |= Modifier::ITALIC;
        }
        if effects.contains(anstyle::Effects::UNDERLINE) {
            modifiers |= Modifier::UNDERLINED;
        }
        if effects.contains(anstyle::Effects::STRIKETHROUGH) {
            modifiers |= Modifier::CROSSED_OUT;
        }
        if effects.contains(anstyle::Effects::HIDDEN) {
            modifiers |= Modifier::HIDDEN;
        }

        style.add_modifier(modifiers)
    }
}

#[allow(dead_code)]
fn anstyle_to_ratatui_color(color: anstyle::Color) -> ratatui::style::Color {
    use ratatui::style::Color;
    match color {
        anstyle::Color::Ansi(ansi) => match ansi {
            anstyle::AnsiColor::Black => Color::Black,
            anstyle::AnsiColor::Red => Color::Red,
            anstyle::AnsiColor::Green => Color::Green,
            anstyle::AnsiColor::Yellow => Color::Yellow,
            anstyle::AnsiColor::Blue => Color::Blue,
            anstyle::AnsiColor::Magenta => Color::Magenta,
            anstyle::AnsiColor::Cyan => Color::Cyan,
            anstyle::AnsiColor::White => Color::Gray,
            anstyle::AnsiColor::BrightBlack => Color::DarkGray,
            anstyle::AnsiColor::BrightRed => Color::LightRed,
            anstyle::AnsiColor::BrightGreen => Color::LightGreen,
            anstyle::AnsiColor::BrightYellow => Color::LightYellow,
            anstyle::AnsiColor::BrightBlue => Color::LightBlue,
            anstyle::AnsiColor::BrightMagenta => Color::LightMagenta,
            anstyle::AnsiColor::BrightCyan => Color::LightCyan,
            anstyle::AnsiColor::BrightWhite => Color::White,
        },
        anstyle::Color::Ansi256(idx) => Color::Indexed(idx.index()),
        anstyle::Color::Rgb(rgb) => Color::Rgb(rgb.0, rgb.1, rgb.2),
    }
}

#[allow(dead_code)]
fn find_substring(
    haystack: &str,
    needle: &CowStr,
    allow_outside: bool,
    rfind: bool,
) -> Option<Range<usize>> {
    if needle.is_empty() || haystack.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    if !allow_outside {
        if let CowStr::Borrowed(needle) = needle {
            let (hp, np) = (haystack.as_ptr(), needle.as_ptr());
            unsafe {
                let (he, ne) = (hp.add(haystack.len()), np.add(needle.len()));
                if np >= hp && ne <= he {
                    let offset = np.offset_from(hp) as usize;
                    let range = offset..(offset + needle.len());
                    if cfg!(debug_assertions) {
                        assert_eq!(
                            haystack.as_bytes().get(range.clone()),
                            Some(needle.as_bytes())
                        );
                    }
                    return Some(range);
                }
            }
        }
        None
    } else {
        if rfind {
            haystack.rfind(needle.as_ref())
        } else {
            haystack.find(needle.as_ref())
        }
        .map(|pos| pos..(pos + needle.len()))
    }
}

#[allow(dead_code)]
fn decode_html_entity(entity: &str) -> Option<String> {
    let decoded = html_escape::decode_html_entities(entity);
    if decoded.as_ref() == entity {
        return None;
    }
    if decoded.chars().any(char::is_control) {
        return None;
    }
    Some(decoded.into_owned())
}

#[allow(dead_code)]
fn has_blank_line_after(text: &str, pos: usize) -> bool {
    text.as_bytes()
        .get(pos..)
        .and_then(|s| s.iter().copied().find(|&c| c != b' ' && c != b'\t'))
        == Some(b'\n')
}

#[allow(dead_code)]
fn is_br_tag(html: &str) -> bool {
    let Some(inner) = html
        .trim()
        .strip_prefix('<')
        .and_then(|s| s.strip_suffix('>'))
    else {
        return false;
    };
    let tag = inner.trim();
    let tag = tag.strip_suffix('/').map_or(tag, str::trim);
    tag.eq_ignore_ascii_case("br")
}

/// Transient state for the fenced code block currently being parsed.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PendingCodeBlock {
    info: String,
    body_range: Range<usize>,
    body_text: String,
    body_seen: bool,
}

/// Markdown parser that processes events and populates buffers.
///
/// After calling `parse()`, the transient state (tag_stack, table_state, depth) is dropped and a `ParsedMarkdown` is returned for rendering.
#[allow(dead_code)]
pub struct MarkdownParser<'a, 'b, 'syn, 'oc> {
    text: &'a str,
    ms: MarkdownStyle,
    buffers: &'b mut MarkdownBuffers,
    syntect: Option<&'syn Syntect>,
    /// Incremental highlighter for the trailing still-open fenced code block.
    open_code: Option<&'oc mut OpenCodeHighlighter>,
    // Transient state (dropped after parse)
    tag_stack: Vec<Tag<'a>>,
    table_state: Option<TableState>,
    depth: usize,
    bq_depth: usize,
    last_checkpoint: Option<(CheckpointKind, usize)>,
    max_table_width: Option<usize>,
    link_id_counter: u32,
    collapse_soft_breaks: bool,
    pending_code_block: Option<PendingCodeBlock>,
}

impl<'a, 'b, 'syn, 'oc> MarkdownParser<'a, 'b, 'syn, 'oc> {
    pub fn new(
        text: &'a str,
        ms: MarkdownStyle,
        buffers: &'b mut MarkdownBuffers,
        syntect: Option<&'syn Syntect>,
    ) -> Self {
        Self {
            text,
            ms,
            buffers,
            syntect,
            open_code: None,
            tag_stack: Vec::new(),
            table_state: None,
            depth: 0,
            bq_depth: 0,
            last_checkpoint: None,
            max_table_width: None,
            link_id_counter: 0,
            collapse_soft_breaks: true,
            pending_code_block: None,
        }
    }

    /// Set whether CommonMark soft breaks collapse to a space.
    pub fn collapse_soft_breaks(mut self, collapse: bool) -> Self {
        self.collapse_soft_breaks = collapse;
        self
    }

    /// Set the maximum width for rendered tables.
    pub fn max_table_width(mut self, width: Option<usize>) -> Self {
        self.max_table_width = width;
        self
    }

    /// Parse markdown and return a ParsedMarkdown ready for rendering.
    pub fn parse(mut self) -> ParsedMarkdown<'a, 'b> {
        self.tag_stack.clear();
        self.buffers.clear();
        self.table_state = None;
        self.depth = 0;
        self.last_checkpoint = None;
        self.pending_code_block = None;

        for (event, range) in TextMergeWithOffset::new(offset_events(self.text)) {
            self.on_event(event, range);
        }

        ParsedMarkdown::new(
            self.text,
            self.ms,
            self.buffers,
            self.last_checkpoint,
            self.link_id_counter,
        )
    }

    /// Stub: dispatch a parsed markdown event (T5f).
    #[allow(dead_code)]
    fn on_event(&mut self, _event: Event<'a>, _range: Range<usize>) {
        // T5f: event dispatch not yet implemented.
    }

    /// Stub: handle a tag start event (T5f).
    #[allow(dead_code)]
    fn on_start(&mut self, _tag: Tag<'a>) {
        // T5f: tag start handling not yet implemented.
    }

    /// Stub: handle a tag end event (T5f).
    #[allow(dead_code)]
    fn on_end(&mut self, _tag_end: TagEnd) {
        // T5f: tag end handling not yet implemented.
    }
}

/// Parsed markdown ready for rendering.
#[allow(dead_code)]
pub struct ParsedMarkdown<'a, 'b> {
    pub text: &'a str,
    pub ms: MarkdownStyle,
    pub buffers: &'b mut MarkdownBuffers,
    pub last_checkpoint: Option<(CheckpointKind, usize)>,
    pub next_link_id: u32,
}

impl<'a, 'b> ParsedMarkdown<'a, 'b> {
    pub fn new(
        text: &'a str,
        ms: MarkdownStyle,
        buffers: &'b mut MarkdownBuffers,
        last_checkpoint: Option<(CheckpointKind, usize)>,
        next_link_id: u32,
    ) -> Self {
        Self {
            text,
            ms,
            buffers,
            last_checkpoint,
            next_link_id,
        }
    }
}
