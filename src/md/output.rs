//! Derived from xai-grok-markdown/src/output.rs (xai-org/grok-build, Apache-2.0). Modified: only HyperlinkTarget copied; full output module is ticket T7.

use std::ops::Range;

/// A hyperlink target extracted from rendered markdown.
/// The shared `id` enables OSC 8 hover-grouping across wrapped lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperlinkTarget {
    /// Index of the rendered line this target appears on.
    pub line_index: usize,
    /// Column range (in display cells) of the link text on that line.
    pub column_range: Range<usize>,
    /// The destination URL.
    pub url: String,
    /// Stable identifier for grouping link fragments that belong to the same logical link (e.g., a link whose text wraps across lines).
    pub id: u32,
}
