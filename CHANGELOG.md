# Changelog

# 2026-10-17

## feat(md): copy SourceMap (V3/T6a)
Derived from xai-grok-markdown/src/source_map.rs (Apache-2.0).
- New `src/md/source_map.rs` (105 lines ≤ cap): VERBATIM copy of upstream
  `SourceMap` — rendered→source byte mapping for copy/selection. All methods
  copied with upstream docs: `new`, `add`, `to_source` (overlap-clamped min/max
  resolution, `None` when not covered), `is_empty`, `len`, `extend_with_offsets`
  (rend./source offsets for merging frozen + tail maps), `segments`, `truncate`,
  `clear`. Zero dependencies (`std::ops::Range` only), pure logic, private
  `segments` field (public API only — matches upstream visibility).
- Provenance header replaces the upstream doc-module first line; the rest of
  upstream module doc is kept Doc-verbatim. The trailing design-note comment
  (post-impl fragment about ratatui line-level mapping) was OMITTED
  (per-ticket "copia o ometti": it is a leftover fragment, not API).
- `src/md/mod.rs`: `pub mod source_map;`.
- New `tests/md_source_map.rs` (5 tests): add+to_source 1:1 mapping; None for
  unmapped ranges/gaps; partial-overlap → source slice; `extend_with_offsets`
  merging with both offsets (segments + resolution through merged map);
  len/is_empty/truncate/clear coherence.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 264 passed,
  0 failed, 1 ignored (5 new); `cargo clippy --all-targets` = 0 unused, 3
  baseline, 0 new (one `length comparison to zero` in the new test fixed);
  `wc -l` src/md/source_map.rs = 105 ≤300.

# 2026-10-16

## feat(md): table rendering — format_table (V3/T5g)
Derived from xai-grok-markdown/src/parse.rs (Apache-2.0).
- New `src/md/parse_table.rs` (300 lines ≤ cap): `MarkdownParser::format_table`
  verbatim from upstream — column widths from `plain_text()` display widths,
  `max_table_width` proportional reflow (word minimums → grapheme floors →
  proportional distribution), alignments, top/header-sep/row/row-sep/bottom
  borders via `TableBorders::BOX` and `format_border_line`, hyperlink
  accumulation (`TableHyperlink` line_offset adjusted per row), `TableCellCopy`s
  and source line offsets. Returns the `FormattedTable` destructured in
  `parse_end.rs::on_end` → `TableReplace` push into `buffers.table_replaces`.
- New `src/md/parse_table_fmt.rs` (269 lines): `wrap_cell_text_joins`
  (textwrap FirstFit + custom word separator, grapheme hard-split fallback for
  unbreakable words), `format_styled_content_lines` (multi-visual-line rows,
  span slicing across wrap boundaries, bold header/body styles, code/ strike/
  link styling, `allow(clippy::too_many_arguments)` per upstream),
  `format_border_line`.
- New `src/md/parse_table_word.rs` (231 lines): free fns `cell_word_separator`
  (punct-break heuristic with URL protection via `url::Url::parse`) and
  `cell_wrap_joins` (Tight/Gap reconstruction of omitted text between wrapped
  lines).
- `src/md/parse_end.rs`: `format_table` stub REMOVED (real implementation in
  `parse_table.rs`); `FormattedTable` stays there (documented in its doc
  comment) — reachable via `use crate::md::parse_end::FormattedTable`; dead_code
  allow dropped (all fields now used).
- `src/md/mod.rs`: + `pub mod parse_table; parse_table_fmt; parse_table_word;`.
- Divergences (documented in each `Modified:` header): Rust 2021 — upstream
  let-chains rewritten as nested `if`s; `sort_by` → `sort_by_key(Reverse)`
  (clippy parity, same ordering); visibility-for-testability —
  `wrap_cell_text_joins`/`format_border_line`/`cell_wrap_joins` `pub` (tested
  directly in tests/); latex/mermaid remain stubs of their dedicated tickets.
- New `tests/md_parse_table.rs` (7 tests): GFM table → exactly one
  `TableReplace` (n_cols=2, borders + cell text in plain lines),
  `format_border_line` box/ASCII chars (TableBorders), wrap-joins quartet
  adapted from upstream (`zebraone_space_before_slash`, `foo_bar_punct_wrap`,
  `hard_split_url`, `likes_long_walks`), `max_table_width()` cap (all lines ≤
  30 display columns, unicode-width measured).
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 259 passed,
  0 failed, 1 ignored (7 new); `cargo clippy --all-targets` = 0 unused import,
  3 baseline, 0 new; `wc -l` parse_table.rs=300, parse_table_fmt.rs=269,
  parse_table_word.rs=231, parse_end.rs=199.

# 2026-10-16

## feat(md): copy MarkdownStyle + TableBorders (V3/T1)
Derived from xai-grok-markdown/src/style.rs (Apache-2.0).
- New `src/md/style.rs` (265 lines): verbatim copy of grok-build's markdown
  styling types — `TableBorders` (BOX/ASCII/DOUBLE, all short+long accessors)
  and `MarkdownStyle` (24 scalar `anstyle::Style` fields + 2 `[Style; 6]`
  heading arrays = 26 struct fields, 36 Style values) with its identical
  `adapt()` method, plus the `all_hidden` and `merge_styles` helpers. Inline
  `#[cfg(test)]` test module stripped (RULES.md: no inline tests). Origin
  header `//! Derived from ...` prepended. `anstyle` already present (T0).
  Visibility `pub(crate)` matches upstream verbatim (T1-review Resolution 1).
- New `src/md/style_adapt.rs` (20 lines): `adapt_style` identity stub
  (placeholder — real truecolor→256/16 downgrade lands with the future
  `colors.rs` ticket). `style.rs` imports `adapt_style` from here; swapping in
  the real implementation is a one-line import change. `pub(crate)` per
  upstream.
- `src/md/mod.rs`: declares `pub mod style;` and `mod style_adapt;`.
- New `tests/md_style.rs` (8 tests, all via public API): moved the upstream
  `test_style::STYLE` const here as an external integration test + contract
  tests — MarkdownStyle field effects (BOLD/ITALIC/STRIKETHROUGH),
  `TableBorders::BOX/ASCII/DOUBLE` characters (+ default/new), and
  `adapt_style` identity exercised indirectly through `MarkdownStyle::adapt()`
  (which delegates to `adapt_style`; `all_hidden`/`merge_styles` are
  `pub(crate)` and tested in a future ticket when the renderer wires them).
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 201 passed,
  0 failed, 1 ignored (8 new); `cargo clippy --all-targets` = 3 baseline
  (type_complexity + Default×2), 0 new; `wc -l` style.rs=265 ≤300, style_adapt.rs=20 ≤300.

## feat(md): fixed tokyonight MarkdownStyle theme (V3/T2)
Derived from xai-grok-markdown .../theme/md_style.rs + theme/tokyonight.rs (Apache-2.0).
- New `src/md/theme.rs` (159 lines): `fixed_style() -> MarkdownStyle` returning
  a single fixed tokyonight theme, built from inlined RGB constants (no
  `Theme::current()`, no `struct Theme` — multi-theme deferred). Verbatim bridge
  functions copied from grok's `md_style.rs` (`to_anstyle` with `Reset→None`
  for NO_COLOR, `fg`, `bg`, `modifier_to_anstyle`, `heading_inner_styles`,
  `heading_outer_styles`). Color values from grok's `theme/tokyonight.rs`
  (md_heading_h1..h6 = TEAL/BLUE/ORANGE/RED/GREEN/MAGENTA all +BOLD,
  md_code=GREEN1, md_task_checked=CYAN, md_task_unchecked=BLUE,
  md_muted=COMMENT, md_code_bg=BG_HIGHLIGHT, md_text=FG, link_fg=BLUE).
- `src/md/mod.rs`: declares `pub mod theme;`.
- New `tests/md_theme.rs` (3 tests): `reset_maps_to_no_color` (NO_COLOR),
  `named_colors_map_concretely` (DarkGray→BrightBlack, Gray→White, Red→Red),
  `fixed_style_has_tokyonight_fields` (text fg truecolor, strong_inner BOLD,
  emphasis_inner ITALIC, link_text UNDERLINE, heading_inner[0] fg+BOLD).
- No `#[cfg(test)]` inline; no tracing/doc-esempio sulle funzioni-ponte
  copiate (esenzione copia-fedele, RULES.md).
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 204 passed,
  0 failed, 1 ignored (3 new); `cargo clippy --all-targets` = 3 baseline,
  0 new; `wc -l src/md/theme.rs` = 159 ≤300.

## feat(md): copy parsing core — parser_options + offset_events (V3/T3a)
Derived from xai-grok-markdown-core/src/lib.rs (Apache-2.0).
- New `src/md/core.rs` (108 lines): verbatim copy of grok-build's parsing core —
  `parser_options()` (GFM | STRIKETHROUGH | MATH | TASKLISTS | TABLES),
  `offset_events()` (events with source byte ranges, single-tilde demoted to
  literal `~`), `DoubleTildeOnlyStrike` filter with `is_double_tilde_strike` and
  `strike_delim_text` helpers, and the full `Iterator::next()` impl. Keeps
  `#![deny(clippy::indexing_slicing)]`. No `MarkdownStats`/`analyze` (deferred
  to T3b). No inline tests (RULES.md).
- `src/md/mod.rs`: declares `pub mod core;`.
- New `tests/md_core.rs` (4 tests): `parser_options_contains_all_flags` (5
  flags), `offset_events_double_tilde_produces_strikethrough`,
  `offset_events_single_tilde_not_strikethrough`, `offset_events_byte_ranges_consistent`.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 205 passed,
  0 failed, 1 ignored (4 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `wc -l src/md/core.rs` = 108 ≤300.

## feat(md): copy MarkdownBuffers + output types (V3/T4)
Derived from xai-grok-markdown/src/buffers.rs (Apache-2.0).
- New `src/md/buffers.rs` (235 lines): core output types — `Highlight`,
  `Replace`, `LinkTarget`, `CodeBlockMeta`, `Transform`, `CellSpan`,
  `StyledCell`, `CellJoin`, `RenderEventKind`, `RenderEvent`,
  `MarkdownBuffers`, `unicode_display_width`, `floor_char_boundary`,
  `ceil_char_boundary`. Keeps `#![deny(clippy::indexing_slicing)]`.
- New `src/md/buffers_table.rs` (125 lines): table types — `TableState`,
  `TableCellCopy`, `TableCopyMeta`, `TableHyperlink`, `TableReplace`,
  `MermaidReplace`. Split per cap 300 righe (file sorgente upstream: 348).
- `src/md/mod.rs`: declares `pub mod buffers;` and `pub mod buffers_table;`;
  `buffers.rs` re-exports `MermaidReplace`/`TableReplace` publicly.
- New `tests/md_buffers.rs` (5 tests): `markdown_buffers_new_and_clear`,
  `styled_cell_new_plain_text_clear`, `unicode_display_width_ascii_and_cjk`,
  `table_state_new_and_push_text`, `cell_join_variants`.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 213 passed,
  0 failed, 1 ignored (5 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `wc -l src/md/buffers.rs` = 235 ≤300, `buffers_table.rs` = 125 ≤300.

## feat(md): copy Checkpoint + CheckpointKind (V3/T5a)
Derived from xai-grok-markdown/src/checkpoint.rs (Apache-2.0).
- New `src/md/checkpoint.rs` (58 lines): verbatim copy of grok-build's checkpoint types —
  `struct Checkpoint` (3 fields: `source_bytes`, `output_lines`, `kind`) and
  `enum CheckpointKind` (8 variants: Heading, Paragraph, CodeBlock, BlockQuote,
  List, ThematicBreak, Table, HtmlBlock). All `Debug, Clone, Copy, PartialEq, Eq`.
  No dependencies; no logic (prerequisite for T5 parse). No inline tests (RULES.md).
- `src/md/mod.rs`: declares `pub mod checkpoint;`.
- New `tests/md_checkpoint.rs` (2 tests): `checkpoint_kind_has_all_variants`
  (8 variants + PartialEq), `checkpoint_fields_and_copy` (3 fields + Copy semantics).
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 215 passed,
  0 failed, 1 ignored (2 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `wc -l src/md/checkpoint.rs` = 58 ≤300.

## feat(md): syntax highlighting module on syntect defaults (V3/T5b, option B)
Derived from xai-grok-markdown/src/syntax.rs (Apache-2.0).
- New `src/md/syntax.rs` (149 lines): verbatim structure/API from grok-build's
  `syntax.rs`, adapted to option B — `SyntaxSet::load_defaults_newlines()`
  instead of the two-face `syntaxes.bin` build dump; `ThemeSet::load_defaults()`
  for the theme; no `.tmTheme` assets, no `build.rs`, no Swift grammar.
  `Syntect::new()` returns `Option<Self>` (no `expect`/`unwrap` in production).
  `parse_line_citation_fence_info` and `syntax_highlight_raw` copied verbatim.
  `find_syntax_by_token`, `find_syntax_by_file_path`,
  `highlight_lines_for_token`, `highlight_lines_by_file_path`,
  `highlight_lines_for_fence_info`, `find_syntax_for_fence_info` copied with
  Swift/patching logic removed.
- `src/md/mod.rs`: declares `pub mod syntax;`.
- New `tests/md_syntax.rs` (5 tests): `line_citation_fence_parses_start_end_path`,
  `line_citation_rejects_non_numeric_line`,
  `line_citation_rejects_plain_lang_token`, `syntect_default_builds_and_finds_rust`,
  `syntax_highlight_raw_produces_spans`, `syntax_highlight_raw_none_and_unknown_lang`.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 221 passed,
  0 failed, 1 ignored (6 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `cargo tree | grep -iE "two-face|onig"` → vuoto;
  `wc -l src/md/syntax.rs` = 149 ≤300.
- Nota: opzione A (two-face + build.rs + assets + Swift) rimandata a ticket dedicato.

## feat(md): copy url_scan + anticipate HyperlinkTarget (V3/T5c)
Derived from xai-grok-markdown/src/{url_scan,output}.rs (Apache-2.0).
- New `src/md/url_scan.rs` (96 lines): verbatim copy of grok-build's plain-URL
  detection — `detect_plain_urls`, `detect_plain_urls_with_offset`,
  `for_each_plain_link`. Keeps `is_char_boundary` guards, scp-skip,
  email→`mailto:`, overlap dedup. No `unwrap`/`expect` in production.
- New `src/md/output.rs` (17 lines): only `HyperlinkTarget` anticipated from
  upstream `output.rs` (ticket T7 will complete the module). Fields:
  `line_index`, `column_range`, `url`, `id`; derives `Debug, Clone,
  PartialEq, Eq`.
- `src/md/mod.rs`: declares `pub mod output;` and `pub mod url_scan;`.
- New `tests/md_url_scan.rs` (10 tests): URL without trailing dot, email→mailto,
  scp not mailto, split-across-spans is one target, empty document, offset,
  dedup, incremental ids, HyperlinkTarget fields/eq.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 231 passed,
  0 failed, 1 ignored (10 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `wc -l src/md/url_scan.rs` = 96 ≤300, `output.rs` = 17 ≤300;
  `cargo tree | grep -iE "two-face|onig"` → vuoto.

## feat(md): copy OpenCodeHighlighter streaming cache (V3/T5d)
Derived from xai-grok-markdown/src/open_code_highlighter.rs (Apache-2.0).
- New `src/md/open_code_highlighter.rs` (243 lines): verbatim copy of grok-build's
  incremental syntax-highlighting cache for streamed code fences —
  `OpenCodeHighlighter` (`new`, `highlight_block`, `highlight_closed`, `highlight`,
  `committed_prefix_matches`), `HlLine` type alias, `CLOSED_MEMO_CAP_BYTES` const.
  Keeps `is_char_boundary` guards, `match ... Err => return None` (no panic in prod).
  `Syntect` sourced via `Syntect::new()` (option B defaults, T5b) — no two-face,
  no `build.rs`, no assets, no Swift grammar.
- `src/md/mod.rs`: declares `pub mod open_code_highlighter;`.
- Visibility-for-testability: `new`, `highlight_block`, `highlight_closed`,
  `highlight`, `committed_prefix_matches`, `closed_memo_bytes`, `CLOSED_MEMO_CAP_BYTES`,
  `HlLine` widened to `pub` for `tests/` (documented in header).
- New `tests/md_open_code.rs` (8 tests): `append_only_growth_matches_fresh_full_highlight`,
  `fence_change_invalidates`, `start_offset_change_invalidates`,
  `unknown_fence_returns_none`, `closed_memo_matches_batch_and_is_idempotent`,
  `closed_memo_distinguishes_fence_info_and_body`,
  `closed_memo_does_not_disturb_open_block_state`,
  `closed_memo_cap_overflow_keeps_output_correct`.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 239 passed,
  0 failed, 1 ignored (8 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `cargo tree | grep -iE "two-face|onig"` → vuoto;
  `wc -l src/md/open_code_highlighter.rs` = 243 ≤300.

## feat(md): scaffold MarkdownParser (V3/T5e)
Derived from xai-grok-markdown/src/parse.rs (Apache-2.0).
- New `src/md/parse.rs` (298 lines): scaffold of grok-build's markdown parser —
  `MarkdownParser` struct (text, ms, buffers, syntect, open_code, tag_stack,
  table_state, depth, bq_depth, last_checkpoint, max_table_width,
  link_id_counter, collapse_soft_breaks, pending_code_block), `new()`,
  builders `collapse_soft_breaks()` / `max_table_width()`, `parse()` entry
  point (iterates `offset_events` → stub `on_event` → `ParsedMarkdown`),
  `ParsedMarkdown` struct + `new()`. Pure helpers: `StyleInto` trait + impl,
  `anstyle_to_ratatui_color`, `find_substring`, `decode_html_entity`,
  `has_blank_line_after`, `is_br_tag`, `PendingCodeBlock`.
- `on_event`/`on_start`/`on_end` are empty stubs (T5f); `format_table` omitted
  (T5g). `latex` deferred (dedicated ticket). `offset_events` imported from
  `crate::md::core` (T3a). No `unwrap`/`expect` in production; stubs use empty
  bodies, not `todo!()`.
- `src/md/mod.rs`: declares `pub mod parse;`.
- Visibility-for-testability: `ParsedMarkdown` fields widened to `pub` for
  `tests/` (documented in header).
- New `tests/md_parse.rs` (9 tests): `parser_new_and_builders`,
  `parse_simple_input_no_panic`, `parse_empty_input_no_panic`,
  `parse_no_syntect_no_panic`, `collapse_soft_breaks_false`,
  `max_table_width_some`, `max_table_width_none`, `parsed_markdown_fields`,
  `checkpoint_kind_variants`.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 248 passed,
  0 failed, 1 ignored (9 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `wc -l src/md/parse.rs` = 298 ≤300.

## feat(md): parser event dispatch — on_event/on_start/on_end (V3/T5f)
Derived from xai-grok-markdown/src/parse.rs (Apache-2.0).
- New `src/md/parse_events.rs` (~200 lines): `on_event` dispatch (events:
  Start/End/Text/Code/InlineMath/SoftBreak/HardBreak/Html/InlineHtml/
  DisplayMath/FootnoteReference/Rule/TaskListMarker), `push_highlight`.
- New `src/md/parse_blocks.rs` (~250 lines): `on_start`/`on_end` with heading,
  blockquote, code block, HTML block, list, item, table, emphasis, strong,
  strikethrough, link/image handling; `format_table` stub (T5g);
  `try_push_mermaid` stub (T5g).
- New `src/md/parse_inline.rs` (~150 lines): `style_inline_code_span`,
  `scan_inline_html_entities`, `push_table_text_linking_urls`,
  `try_push_mermaid` stub, `push_display_math_block` stub (T-latex).
- New `src/md/latex.rs` (stub): `latex_to_unicode_inline`/`latex_to_unicode_display`
  return `None` (T-latex deferred).
- `src/md/mod.rs`: declares `pub mod parse;`, `pub mod parse_blocks;`,
  `pub mod parse_events;`, `pub mod parse_inline;`, `pub mod latex;`.
- `src/md/parse.rs`: stubs `on_event`/`on_start`/`on_end` removed; kept struct,
  new/builder/parse/ParsedMarkdown/pure helpers.
- Visibility-for-testability: `push_highlight`, `on_start`, `on_end`,
  `highlight_block`, `highlight_closed`, `highlight`,
  `committed_prefix_matches`, `closed_memo_bytes`, `CLOSED_MEMO_CAP_BYTES`,
  `HlLine` widened to `pub` for `tests/` (documented in headers).
- New `tests/md_parse_events.rs` (4 tests): `parse_produces_highlights_for_heading_and_bold`,
  `inline_code_produces_span_with_style`, `strikethrough_is_handled`,
  `math_inline_does_not_panic`.
- Verifica: `cargo build` ok; `cargo test -- --test-threads=1` = 252 passed,
  0 failed, 1 ignored (4 new); `cargo clippy --all-targets` = 3 baseline, 0
  new; `cargo tree | grep -iE "two-face|onig"` → vuoto;
  `wc -l src/md/parse*.rs` → tutti ≤300.

# 2026-10-15

## fix: wrap markdown a larghezza di display (V1 — adozione openviking `crates/ov_cli`)
- `src/markdown.rs` misurava in **byte** (`cur.len() + word.len() + 1 > max`) e in
  **char** (troncamento fence): con CJK/emoji (2 celle, 3–4 byte) il wrap sbagliava
  e i fence sfondavano il pane. Ora `wrap()` usa `unicode-width`, adattata da
  `openviking crates/ov_cli/src/commands/render_utils.rs::wrap_display_text`
  (Apache-2.0; la root del vendor è AGPL-3.0 — nessun codice preso da lì):
  append per parola finché entra, **split char-level** per parole più lunghe della
  riga, `UnicodeWidthStr::width` anche per `prefix`. Deviazioni documentate: niente
  `max_lines`/ellissi (la history mostra tutte le righe), `…` conservato sui fence.
- Nuova dep `unicode-width = "0.2.0"` (pinnata; era già nel lock via
  `unicode-truncate`, nessuna copia nuova). `THIRD-PARTY-NOTICES.md` attribuisce
  openviking (`crates/ov_cli`, snapshot `f6010a5`); corretta la nota di licenza
  errata in `docs/patterns/VENDOR-TUI-ANALYSIS.md` (root AGPL-3.0, non Apache-2.0).
- Doc comment + esempio su `render_markdown` (RULES: doc pubblica con esempio).
- Test: `tests/markdown.rs` (6, headless su `render_markdown`): nessuna riga
  renderizzata supera `max_width` per prose CJK, parole non spezzabili, fence
  multibyte; rosso sul renderer vecchio (4/6 falliscono), verde sul nuovo.
  Verifica: build ok, suite **180/180** verde + 15 doctest (1 ignored noto),
  `--test-threads=1`; clippy = 3 lint baseline identici, 0 nuovi; `markdown.rs`
  268 righe (≤300). V1 → Resolved.

## refactor: estrai Message/Role in src/message.rs (DEBT1)
- `Message` (struct) e `Role` (enum) con i loro `impl` (`new`, `with_tool`,
  `tool`) spostati da `src/app.rs` al nuovo modulo `src/message.rs` (≤300 righe),
  re-exportati da `app.rs` tramite `pub use crate::message::{Message, Role};`
  così tutti i call site esistenti (`app::Message`, `app::Role`) restano stabili
  senza modifiche. `lib.rs` dichiara `pub mod message;`.
- Nessun cambio di comportamento: `push_thinking`/`apply_token` restano in
  `app.rs` (dipendono dallo stato di thinking di `App`, per decisione A1+B1).
- Verifica: `cargo build`, `cargo test` (185/0/1, `--test-threads=1`),
  `cargo clippy --all-targets` (3 lint baseline, 0 nuovi),
  `grep "struct Message\|enum Role" src/` → solo in `message.rs`.

## feat: syntect syntax highlighting for code fences (V2)
- `src/highlight.rs` (nuovo, ≤300 righe): `highlight_code(code, lang) ->
  Vec<Vec<(Style, String)>>` per-token highlighting via `syntect` 5.3.0
  (`default-features = false`, `features = ["default-fancy"]`, pure-Rust
  `fancy-regex`, no `onig`/C, no `syntect-tui`). `SyntaxSet`/`ThemeSet`
  caricati una sola volta (`OnceLock`); selezione sintassi per token di lingua
  con fallback plain-text su lang vuoto/sconosciuto; mapping
  `syntect::highlighting::Color` → `ratatui::style::Color::Rgb`. Nessun
  `unwrap`/`expect` in produzione.
- `src/markdown.rs`: nel ramo `Segment::Code` la riga è costruita dagli span
  di `highlight_code` (più span colorati per riga), mantenendo box/border e
  applicando la troncatura cell-width V1 **per-span**; su lang sconosciuto o
  errore si fall back al vecchio `highlight_style` (colore unico) senza
  rompere il render. `highlight_style` resta per il fallback.
- Test: `tests/highlight.rs` (multi-span su Rust, fallback lang sconosciuto,
  fallback lang vuoto, coerenza OnceLock) + caso di troncatura cell-width in
  `tests/markdown.rs`.
- `THIRD-PARTY-NOTICES.md`: attribuzione syntect (MIT).
- Verifica: `cargo build`, `cargo test -- --test-threads=1` verde (≥185 +
  nuovi, 0 failed, 1 ignored noto), `cargo clippy --all-targets` = 3 lint
  baseline, `wc -l` entrambi i file ≤300, `cargo tree -i syntect` =
  default-fancy senza onig.

# 2026-10-02

# Changelog

# 2026-10-02

## feat: KeyStore cablato in produzione (I2 — wiring del seam I1)
- **Factory (TDD)**: `keystore::store_for(mode, passphrase) -> Result<Box<dyn KeyStore>, KeyStoreError>`
  con mapping `EnvFirst→EnvStore`, `KeychainFirst`/`OsKeychain→OsKeychainStore`,
  `EncryptedFile→EncryptedFileStore`; EncryptedFile senza (o con) passphrase
  vuota = `KeyStoreError::Unavailable` **pulito**, mai panic.
- **Passphrase 1x/sessione**: `app.passphrase: Option<Zeroizing<String>>` +
  prompt mascherato in `handler.rs` modellato su `set_key_pending` (buffer
  azzerato su Enter/Esc, mai loggato — D7) + overlay
  `ui.rs::render_passphrase_overlay` (bullet per char, mai il valore). Esc
  annulla tutte le azioni in attesa (models fetch, connect MCP, send accodata)
  così niente loop di re-prompt.
- **Read-path (4 call site) su `resolve_key_via` = factory + `resolve_key_with`**,
  con `spawn_blocking`/`block_in_place` mantenuti: `main.rs` models-fetch,
  `main.rs` agent spawn, `mcp/mod.rs::connect_from_config` (passphrase ora in
  firma; il connect MCP è **differito al loop** per il gate), `handler.rs`
  `/provider use`. Gate `needs_passphrase`: se serve la passphrase e manca →
  apre il prompt, **non spawna**; Enter riprende, Esc annulla.
- **Write-path sul trait**: `/provider set-key` (Enter) e `/provider
  remove-key` usano `store.set`/`store.delete` via factory — niente più
  `keyring::Entry` diretto in `handler.rs`. In modalità cifrata il set-key
  prompta la passphrase retenendo il buffer e salva alla seconda Enter.
- `keys.rs::resolve_key` legacy **non rimosso** (ancora coperto da
  `tests/keys.rs`); `current_mode()` estratto e condiviso.
- Tests: `tests/keystore_factory.rs` (6) + `tests/passphrase.rs` (6, incl.
  overlay masked); `tests/mcp_sampling.rs` adattato alla nuova firma. Split
  300-cap: le funzioni di risoluzione sono passate a `src/keys/resolve.rs`
  (re-export da `keys.rs`, percorsi stabili). Verifica:
  build ok, suite **173/173** verde con `--test-threads=1` + 15 doctest,
  clippy = 3 lint **identici** alla baseline pre-cambiamento (verificata prima
  di toccare il codice, senza `git stash` per non toccare le modifiche
  untracked in `docs/patterns/`), 0 nuovi, 0 unwrap/expect nuovi in prod.
  I2 → Resolved.

## refactor: split app.rs — commands.rs + metrics.rs (RULES.md debt, T6)
- Estrazione behavior-preserving: `src/commands.rs` (165 righe: `Command`,
  `ProviderCommand`, `ServerCommand`, `ConfigCommand`, `parse_command`) e
  `src/metrics.rs` (100 righe: `Metrics` + impl) spostati **verbatim** fuori da
  `app.rs`. `app.rs` 973 → 729 righe (diff +7/−251 = solo re-export + rimozioni).
- Compat: `app.rs` ri-esporta i tipi (`pub use crate::commands::{...}` /
  `crate::metrics::Metrics`), così gli import esistenti
  (`nxm_tui::app::{parse_command, Command, Metrics, ...}` in handler + test)
  restano validi **senza toccare un solo call site**.
- Dipendenze pulite: `commands.rs` usa solo `crate::app::AgentMode`,
  `metrics.rs` solo std (nessun ciclo).
- Fuori scope (ticket futuri): spostare `Message`/`Role` (60+ call site), split
  di `main.rs::run` (fn da 340 righe).
- Eseguito da free agent + review indipendente. Verifica: build ok, suite verde
  in parallelo, clippy = 3 lint IDENTICI alla baseline (type_complexity +
  Default×2, preesistenti, provati con git stash), 0 nuovi, 0 unwrap nuovi.
  T6 → Resolved.

## feat: interactive scrollback search in the TUI (T5 — wires src/search.rs)
- Nuovo `src/search_ui.rs` (181 righe, funzioni pure testabili): `line_matches`
  (indici righe che matchano), `scrollback_for_line` (indice-riga → `scrollback`
  così il match entra in viewport), `highlight_line` (restyle degli span
  matchanti via `compiled_regex`, slice byte-safe allineati al regex).
- Wiring: `/` (prompt vuoto) apre la ricerca; query live con highlight; `n`/`N`
  navigano i match con wrap; `Esc`/Backspace-su-vuoto chiudono. Stato
  `App::search: Option<SearchState>` (pattern modale come `set_key_pending`),
  query bar overlay in `ui.rs`. La ricerca opera sulle RIGHE FINALI renderizzate
  (`render_history`), così gli indici sono allineati a wrap/markdown; lo
  scrollback-per-match è calcolato in `render_history` dove `lines.len()` e
  `area.height` sono reali (il round-1 usava valori approssimati → non saltava).
- Collega il modulo `search` portato da grok (commit `31b6f72`) → chiude il
  cerchio sulla vendor-adoption prio #1.
- Eseguito da free agent, **2 round di review indipendente** (round 1: `n`/`N`
  non saltava al match per `scrollback_for_line` con `messages.len()`/height
  hardcoded; round 2: calcolo spostato in render con valori reali + semantica
  `current` chiarita). Verifica: `tests/search_ui.rs` 21/21 (incl. navigazione
  wrap + jump), clippy 0 sui file toccati, 0 unwrap prod. T5 → Resolved.

## feat: provider/model commands — /models, /model use, /provider remove-key (T2)
- Nuovo `src/models.rs` (212 righe): `ModelEntry` + `parse_models` (parser
  tollerante R2: due envelope OpenAI `data[]` / Ollama `models[]`, id→name→model,
  `created` int o ISO, chiavi ignote ignorate, cap 100) + `models_candidate_paths`
  (normalizza `/v1` → mai `.../v1/v1/models`) + `fetch_models` async (fallback
  path `/v1/models` → `/models` → `/api/tags`, 401 azionabile, server-down).
- `/models` — lista dal provider attivo (fetch async nel loop di `main.rs`,
  stato pending come `inference_task`, UI mai bloccata; key risolta per-provider
  via `resolve_key`+`spawn_blocking`, mai loggata).
- `/model use <id|numero>` — seleziona il modello (indice sulla lista o id),
  imposta `cfg.model_name` + `Provider.default_model`, salva config.
- `/provider remove-key <name>` (alias `rm-key`) — elimina la key dal keychain
  (`delete_credential`) con guided error se keychain non disponibile.
- Modello attivo mostrato nella mode bar (`src/mode_bar.rs`); `app.model_name`
  sincronizzato da `cfg.model_name` all'avvio (entrambi i rami → Running).
- Help (`overlays.rs`) + autocomplete aggiornati; sezione Security "API keys are
  NEVER printed or logged".
- Eseguito da free agent esterno, **2 round di review indipendente** (round 1:
  4 difetti trovati — mode bar `—`, doppio `/v1`, api_key None, report
  incompleto; round 2: tutti corretti). Verifica: `tests/models.rs` 16/16,
  clippy 0 sui file toccati, 0 unwrap/expect prod. T2 → Resolved.
- Debito confermato (non regressione): flakiness `keys` in parallelo (mock
  keychain) — la risolverà il seam `KeyStore` di R5.

## feat: dev-friendly key loading — .env fallback, no keychain prompt
- **Problema risolto**: `resolve_key` interrogava SEMPRE il keychain OS per
  primo → su macOS un dialog password a ogni spawn Agent (ogni messaggio),
  anche quando la key era in una env var. Fastidioso in sviluppo.
- **Nuovo**: `keys::KeySourceMode {EnvFirst, KeychainFirst}` + funzione pura
  `key_source_mode(env_present, override)`. `resolve_key` sceglie il mode:
  - `NXM_KEY_SOURCE=env` → **env-first**: keychain MAI toccato (zero prompt).
  - `NXM_KEY_SOURCE=keychain` → keychain-first (default produzione, D7).
  - `auto`/unset → env-first SE esiste un `.env` locale, altrimenti keychain-first.
- `main.rs`: `dotenvy::dotenv()` all'avvio carica un `.env` locale (gitignored)
  se presente → modalità dev automatica. Crea `.env` (template `.env.example`),
  metti le chiavi, nessun prompt. Prod senza `.env` resta keychain-first.
- Cargo: `dotenvy = "0.15.7"` (pin esatto). `.gitignore`: `/.env`. Nuovo
  `.env.example`.
- **Debito RULES.md risolto contestualmente**: i test inline di `keys.rs`
  spostati in `tests/keys.rs` (keys.rs 356→196 righe, sotto il cap 300). La
  FLAKINESS parallela del mock keychain è risolta: `ENV_LOCK` serializza i test
  che toccano stato globale; la suite completa è ora verde IN PARALLELO (prima
  richiedeva `--test-threads=1`).
- **Scoperta onesta**: il test `keychain_beats_env` è `#[ignore]` — la prod usa
  `keyring::Entry` (store nativo Apple), i test montano un mock su
  `keyring_core`: namespace separati, il mock non è visibile a `keyring`.
  Serve un seam `KeyStore` iniettabile per testarlo davvero (debito tracciato).
  Il path env-first — quello usato in dev — è coperto e verde.

## feat: wire MCP host into the runtime (startup session)
- `src/mcp/mod.rs`: new `connect_from_config(cfg, endpoint) -> Result<Option<McpHandle>>`
  helper — resolves the active provider's API key (keychain→env, off-runtime
  via `spawn_blocking`, value never logged) and delegates to `connect_if_present`.
  Clean no-op (`Ok(None)`) when `cfg.mcp.enabled=false` or the server binary is
  absent.
- `src/main.rs`: at startup, when the endpoint is `Running`, open an MCP session
  and keep the `McpHandle` alive for the whole loop (`_mcp` binding; drop stops
  the run loop). Connect failure logs a warning and continues — chat is never
  blocked. (Closes roadmap item #6 in `docs/mcp/ARCHITECTURE.md`.)
- Known limitation (noted, not addressed): a session connected via the NoServer
  menu *after* startup is not yet given an MCP session (handle is computed once
  pre-loop). Startup auto-detect path — the common case — is covered.
- Tests: `tests/mcp_sampling.rs` +2 — `connect_from_config` clean-skip when
  disabled and when the binary is absent (deterministic, headless). mcp_sampling
  6/6, mcp_protocol 5/5. clippy: 0 on touched files.

## feat: scrollback search primitives — ported from grok-build (search module)
- New `src/search.rs` (vendor adoption, prio #1 di VENDOR-TUI-ANALYSIS):
  `QueryKind {Substring, Regex}` + `TextMatcher` (smart-case alla Vim/ripgrep:
  case-insensitive salvo maiuscole nella query; substring via `regex::escape`)
  e `next_index_after`/`prev_index_before` (navigazione `n`/`N` con wrap su
  slice ordinata). Adattato da grok-build `xai-grok-pager-render/src/search`
  (xai-org/grok-build, Apache-2.0).
- **Conformità RULES.md** (il port corregge la fonte): niente `unwrap`/`expect`
  in prod — il campo `regex` è `Option<regex::Regex>` (regex invalida → `None`
  + `is_error`, `is_match` → false, nessun panic path); test spostati in
  `tests/search.rs` (no `#[cfg(test)]` inline); logging `tracing` target
  `nexum::search`; doc `///` con doctest; header licenza + `THIRD-PARTY-NOTICES.md`.
- Cargo: `regex = "1.13.1"` (pin esatto, versione realmente risolta).
- Verifica (revisore indipendente): `cargo test --test search` 9/9,
  `cargo test --doc` 3/3, clippy 0 menzioni di `search.rs`. NON ancora cablato
  nella UI (campo query in bottom bar + highlight match + tasti `n`/`N`) — step
  successivo.
- Debito preesistente confermato (non introdotto qui): i test inline di
  `keys.rs` falliscono in esecuzione parallela per mock-keychain globale
  condiviso (verdi con `--test-threads=1`); si lega alla violazione RULES.md
  "test inline" di `keys.rs`.

## docs: riconciliazione ticket wayfinder col codice (P3 ✅, T2 parziale)
- Audit tracker vs codice: l'handoff 2026-09-20 indicava "prossima sessione:
  implementa P3", ma P3 risulta **già implementato e committato** (arrivato con
  `cb05a5d` migrate, git pulito). Allineati i doc che erano rimasti indietro.
- **P3 → ✅ Resolved**: `keys.rs::resolve_key` (keychain→api_key_env→
  NEXUM_API_KEY→OPENAI_API_KEY→Missing), `ProviderCommand::SetKey`, overlay
  mascherato (`ui.rs`), `set_password` via keyring (`handler.rs:37`), buffer
  segreto dedicato azzerato dopo save/Esc, risoluzione per-provider allo spawn
  Agent (`main.rs` `spawn_blocking`). `keyring = "4.2.0"` pinnato.
- **T2 → resta Open · parziale**: `default_model` in `Provider` ✔ ma
  `GET /v1/models` è presente **solo come health-probe** in `detect_endpoint`;
  mancano comando `/models` (lista+selezione), `/model use <id>`,
  `/provider remove-key`, modello attivo in mode/bottom bar.
- Aggiornati: `tickets/index.md`, `tickets/P3-key-entry-prompt.md`.
- Debito annotato (non in scope): modulo test inline in `keys.rs` viola la
  regola RULES.md "test in tests/, no #[cfg(test)] inline".

# 2026-09-30

## feat: MCP host subsystem — sampling capability (minimal working cut)
- New `src/mcp/` subsystem: nxm-tui is now an MCP client/host. It spawns an
  MCP server (`nxm-session-mcp --transport stdio`) as its own child, completes
  the `initialize` handshake declaring `capabilities.sampling = {}`, and serves
  `sampling/createMessage` by forwarding to the OpenAI-compatible LLM backend.
  This unblocks server tools (e.g. `generate_handoff`) that need LLM sampling
  to populate their output sections.
  - `protocol.rs` (244): JSON-RPC 2.0 + sampling wire types (serde, spec
    2025-06-18). `transport.rs` (146): stdio spawn + newline framing +
    `binary_present`. `sampling.rs` (156): `SamplingHandler` seam +
    `LlmSamplingHandler` (auto-approved, no `async-trait` — uses `BoxFuture`).
    `client.rs` (273): lifecycle + id-correlated calls + inbound dispatch.
    `mod.rs` (72): `connect_if_present` (clean skip when binary absent).
  - Config: `[mcp]` table (`enabled`, `command`, `args`) in `TuiConfig`.
  - Cargo: added `anyhow` (workspace) to the bin/lib; enabled tokio `process`.
    No new external crates beyond features already in the tree.
  - Tests: `tests/mcp_protocol.rs` (5, serde shapes vs spec) +
    `tests/mcp_sampling.rs` (4, handler contract + live `initialize` against
    the real binary, which passed end-to-end).
  - Docs: `docs/mcp/ARCHITECTURE.md` (as-built + refinement roadmap: approval
    overlay, `tools/call`, streaming, socket transport, main.rs wiring).
  - NOT wired into `main.rs` runtime yet by design — see roadmap.

# 2026-09-27

## Sotto-tappa 2 — Migrazione a repo autonomo + fix remotes (Tappa 2 MIGRATION-PLAN)
- Remotes updated per ADR-002/003; public mirror managed by `nxm-sync`.

# 2026-09-20

## perf: memoized token estimate + cached cwd (T4)
- `app.rs`: `estimated_tokens` memoized in a `Cell` fingerprinted by
  (messages.len(), last message) — O(N) walk ran 3-4× per frame in the
  render path; recomputes only when messages change.
- `app.rs`: `current_dir_name` cached once (`OnceLock`) — was a syscall
  per frame; the process never chdirs (upgrade path: TTL if that changes).
- New `tests/render_caches.rs` (4 tests): streaming growth (a len-only
  cache key would serve stale 8), new-message recompute, differential
  oracle vs a fresh App, cwd consistency. 68/68 green, clippy 0 error.

## perf: draw-on-change + per-message history render cache (T3)
- `event.rs`: idle poll 16 ms → 33 ms (~30 Hz wake-up; halves idle CPU).
- `main.rs`: dirty flag — `terminal.draw` only on key/resize/token drain/
  status/health change; the Thinking spinner keeps animating while working.
- `history.rs`: per-message render cache (`HistoryCache`, key = content_len
  + tool_parts + width): streaming re-renders only the last message instead
  of reparsing the whole O(N) history each frame; resize invalidates all.
- New `tests/history_cache.rs` (3 tests): repeated frames identical, streaming
  token invalidation, resize re-wrap (output differential, not internals).
- Infra forced by the perf change: lib target (`src/lib.rs`) — one module
  graph; tests consume `nxm_tui::`, per-test `#[path]` mirrors deleted (they
  had gone stale: test crates couldn't resolve `crate::autocomplete`). 9 test
  files now compile: 64/64 green, clippy 0 error.
- Includes the uncommitted command-menu work from the previous session
  (autocomplete/handler/sidebar + tests/command_menu.rs) — entangled with the
  lib conversion, so it rides in the same commit.

# 2026-09-08

## feat: overlay render regression net + content-sized help
- New `tests/overlay_render.rs` (7 tests) on ratatui `TestBackend` (80x24,
  no TTY — pi-tui VirtualTerminal pattern): every overlay renders without
  panicking and shows its content (help, thinking full/empty, tool detail
  full/empty, approval with risk badge, approval empty no-op).
- The first run caught a real bug: help content had outgrown its fixed h=24
  box, cutting the newest hints. Help height is now content-sized
  (`items.len()+2`, clamped) and section blanks condensed.
- Note: `cargo fmt --check` is dirty repo-wide (pre-existing style, incl.
  untouched files) — intentionally not reformatted to avoid churn.

## feat: risk badge in approval overlay
- Approval overlay now shows a red advisory badge on top of the Y/N decision:
  `⚠ sensitive path` for secret-bearing paths (`.env`, `.ssh`/`.aws`/`.gnupg`
  segments, keys/certs/credentials/tokens, `..` traversal), `⇄ network egress`
  for `web_search`, `⚠ unrecognized tool` for unknown tools.
- Matcher is deliberately conservative (case-insensitive, segment-aware):
  false positives only add a warning line, a miss could leak a secret into
  the transcript. Advisory only — the user still decides.
- Tests: 4 new in `tests/tool_approval.rs` (network, sensitive matrix incl.
  Windows paths, quiet negatives, unknown tools).

## feat: history viewport (follow-tail + scrollback model)
- Fixed inverted/missing follow: new tokens now stay visible (pinned tail);
  previously offset 0 showed the oldest lines and fresh output scrolled away
  unseen. `App::scrollback` counts lines hidden below the viewport;
  `App::viewport_offset()` clamps short buffers to top, deep scrollbacks to
  the oldest line.
- Conventional scroll keys: Up = older, Down = newer, plus PgUp/PgDn (±10).
  Send/clear/load re-pin to the tail. Scrolling up never loses the live tail
  position — Down returns to it.
- Fix markdown code-fence truncation panic (`&line[..w]` byte slice) with a
  char-safe `…` truncation, same family as the earlier `ToolResult` fix.
- Tests: `tests/history_viewport.rs` (8 tests) — offset math, key direction,
  follow invariant, emoji fence no-panic, plain-text regression.

## feat: tool approval gating (Y/N overlay)
- Agent now asks before running gated tools: `read_file` (arbitrary file
  contents), `web_search` (network egress) and unknown tools. Only
  `list_resources` is auto-approved (`needs_approval()` in `tool_types.rs`).
- Modal approval overlay (`y` approve / `n` or `Esc` deny) in
  `tool_overlay.rs::render_approval`, rendered above all others; other keys
  are swallowed while a decision is pending so a tool can never run or skip
  by an unrelated key. The agent task parks on a oneshot reply; quit/drop =
  fail-safe deny.
- Denials are recorded as the tool result (`denied by user …`) so the model
  reacts on the next turn instead of stalling.
- Plumbing: `ApprovalRequest` channel (`tool_types.rs`), `Agent::approve()`,
  `App::{pending_approval, resolve_approval}`, drain in `main.rs` Tick loop.
  Headless (`None` channel, tests) auto-approves so non-interactive runs
  never hang.
- Tests: `tests/tool_approval.rs` (4 new: policy, resolve, deny e2e with
  no-leak assert, approve e2e) + updated `roundtrip.rs` for the new
  `Agent::new` signature.

## feat: tool detail overlay + char-safe truncation
- New tool detail overlay (`Ctrl+O`, `Esc` closes) in `src/tool_overlay.rs`:
  `last_tool_exchange()` finds the most recent invocation + its `role:"tool"`
  result; overlay shows name, call id, full args and output (6-line cap).
  Read-only kimi `approval-preview` pattern — gating the agent is a later step.
- Fix `ToolResult` truncation panic: byte slice `&s[..100]` panicked on
  multibyte boundaries; new `ToolPart::truncate_chars()` is char-safe + `…`.
- Tool history rows now render an 80-char args preview (`▶ read_file {…}`)
  instead of the bare tool name.
- Tests: `tests/tool_overlay.rs` (5 tests) — multibyte truncation, args
  preview, pending/resolved/most-recent exchange lookup.

## feat: reasoning overlay + per-token metrics + thinking-tag parser fix
- New reasoning overlay (`Ctrl+R`, `Esc` closes): renders live `thinking_content`
  in `src/overlays.rs::render_thinking`, wired in `src/ui.rs`, `src/handler.rs`.
  Mirrors codex transcript overlay / kimi `streaming-ui` thinking separation.
- Per-token metrics wired: `main.rs` Tick loop now calls
  `metrics.record_chars()` on every `ToolPart::Text` (was dead code, tokens/sec
  always 0). Metrics overlay (`Ctrl+M`) now shows real throughput.
- Fix `apply_token` thinking parser: standalone `>` closing bracket previously
  trapped all later tokens in thinking mode; now exits thinking and preserves
  text after the bracket. Handles split (`<thinking>`/`>`), combined
  (`>visible`), and single-token (`a<thinking>b</thinking>c`) shapes.
- Tests: `tests/thinking_metrics.rs` (6 tests) — streaming, metrics, overlay
  flag, three thinking-tag shapes.

# 2026-09-07

## feat: configurable LLM providers via /provider commands
- New `src/provider.rs`: `Provider` struct + presets (Nexum Inferentia, Ollama,
  LM Studio, NVIDIA) and custom-provider validation.
- New slash commands: `/provider list|add <name> <url>|use <name>|remove <name>`
  (aliases `/prov`, `/p`).
- Custom providers persisted in `~/.nexum/tui.toml`; API keys stay env-only
  (never on disk).
- NoServer menu now generated dynamically from the provider registry.
- Fix: Ollama preset uses its official OpenAI-compatible port `11434` (was `11435`).
- NVIDIA cloud endpoint (`https://integrate.api.nvidia.com/v1`) with Bearer auth.
- Tests: `tests/provider.rs` (8 tests) — presets, validation, merge/lookup, parsing.

# 2026-08-22

## feat: migrate nxm-tui to GitHub
- Migrated from monorepo (Projects_Tmp/nxm/tui/)
- Converted workspace deps to standalone versions
- 20 source files (app, connection, ui, markdown, history, etc.)
- Added RULES.md, CONTEXT.md, CHANGELOG.md
