# nxm-tui

Terminal UI client for NXM inference servers. Built with ratatui + crossterm.

## Purpose
Connects to any OpenAI-compatible LLM server (including NXM engines) and provides
an interactive chat interface in the terminal.

## Features
- Streaming chat responses (SSE)
- Markdown rendering in terminal — cell-width aware (CJK/emoji count two columns,
  over-long words split, code fences truncated by width; V1, adapted from
  openviking `crates/ov_cli` wrap_display_text — Apache-2.0, see
  `THIRD-PARTY-NOTICES.md`)
- Session history and persistence
- Server connection management
- Autocomplete for commands
- Configurable via TOML
- Reasoning overlay (Ctrl+R) with live thinking stream
- Tool detail overlay (Ctrl+O) with last invocation args + output
- Tool approval gating (Y/N overlay; only list_resources auto-approved)
- Risk badges in approval (sensitive path / network egress / unknown tool)
- History auto-follow + scrollback (Up/Down/PgUp/PgDn)
- Overlay render tests on TestBackend (no TTY)
- Per-token metrics (tokens/sec now measured, Ctrl+M overlay)
- Draw-on-change + 30 Hz idle polling (no busy redraw at 60 Hz)
- Per-message history render cache (streaming re-renders one message, not O(N))
- MCP host subsystem (`src/mcp/`): client/host that spawns an MCP server via
  stdio, declares the `sampling` capability, and serves `sampling/createMessage`
  by bridging to the LLM backend (auto-approved). Unblocks server tools like
  `generate_handoff`. See `docs/mcp/ARCHITECTURE.md`. Wired into the `main.rs`
  runtime via `mcp::connect_from_config` (clean skip when disabled or
  the server binary is absent; chat never blocked; deferred to the loop when
  the encrypted keystore still needs its session passphrase).
- Secret storage behind the injectable `KeyStore` seam (`src/keystore/`):
  `store_for(mode, passphrase)` maps `NXM_KEY_SOURCE` to `EnvStore` /
  `OsKeychainStore` / `EncryptedFileStore` (AES-256-GCM + Argon2id, `keys.enc`
  0600); every read path resolves via `resolve_key_with`, every write path
  (`/provider set-key`, `/provider remove-key`) via `store.set`/`store.delete`.
  The encrypted file asks its session passphrase at most once per session
  (masked prompt, `Zeroizing` cache, never logged).

## Dependencies
- ratatui + crossterm for TUI rendering
- unicode-width for cell-width text layout (markdown wrap/truncation)
- reqwest for HTTP/SSE streaming
- tokio for async runtime
- Builds as bin + lib (`src/lib.rs`): the module graph is declared once and
  shared between the binary and the headless tests in `tests/`
- Domain types `Message`/`Role` live in `src/message.rs`, re-exported from
  `src/app.rs` so `app::Message`/`app::Role` paths stay stable (DEBT1).
- Code-fence syntax highlighting via `syntect` (V2): `src/highlight.rs`
  exposes `highlight_code(code, lang) -> Vec<Vec<(Style, String)>>`, used by
  `src/markdown.rs` in the `Segment::Code` arm; `SyntaxSet`/`ThemeSet` loaded
  once (`OnceLock`), plain-text fallback on unknown/empty lang.
- Rich markdown renderer (V3): `src/md/` module being populated piece-by-piece
  from grok-build's `xai-grok-markdown` (Apache-2.0). T0 scaffolded
  (`src/md/mod.rs` with a placeholder `render()`); T1 copies
  `MarkdownStyle` + `TableBorders` + helpers (`src/md/style.rs`,
  `src/md/style_adapt.rs`) with the upstream `test_style` const ported to
  `tests/md_style.rs` (8 tests via public API). `anstyle` bridges styles to ratatui.
- T2 adds `src/md/theme.rs`: `fixed_style() -> MarkdownStyle`, a single fixed
  tokyonight theme built from inlined RGB constants (verbatim bridge functions
  from grok's `md_style.rs`, `Reset→None` for NO_COLOR), no `Theme::current()`
  / `struct Theme` (multi-theme deferred). Tests in `tests/md_theme.rs` (3).
- T3a adds `src/md/core.rs`: `parser_options()` (GFM | STRIKETHROUGH | MATH |
  TASKLISTS | TABLES), `offset_events()` (events with byte ranges, single-tilde
  demoted to literal `~`), `DoubleTildeOnlyStrike` filter with
  `is_double_tilde_strike` / `strike_delim_text` helpers, full `Iterator::next()`
  impl. Keeps `#![deny(clippy::indexing_slicing)]`. Tests in `tests/md_core.rs` (4).
- T4 adds `src/md/buffers.rs` (core) + `src/md/buffers_table.rs` (table):
  `MarkdownBuffers`, `StyledCell`, `CellSpan`, `RenderEvent`/`RenderEventKind`,
  `Highlight`, `Replace`, `LinkTarget`, `CodeBlockMeta`, `Transform`,
  `CellJoin`, `TableState`, `TableCellCopy`, `TableCopyMeta`, `TableHyperlink`,
  `TableReplace`, `MermaidReplace`, `unicode_display_width`. Split per cap 300
  righe. Tests in `tests/md_buffers.rs` (5).
- T5a adds `src/md/checkpoint.rs`: `struct Checkpoint` (source_bytes,
  output_lines, kind) + `enum CheckpointKind` (8 variants: Heading, Paragraph,
  CodeBlock, BlockQuote, List, ThematicBreak, Table, HtmlBlock). Verbatim copy,
  no dependencies, no logic. Prerequisite for T5 parse. Tests in
  `tests/md_checkpoint.rs` (2).
- T5b adds `src/md/syntax.rs`: syntax highlighting infrastructure on syntect
  defaults (option B). `Syntect` struct (theme + syntax_set), `find_syntax_by_token`,
  `find_syntax_by_file_path`, `highlight_lines_for_token`,
  `highlight_lines_by_file_path`, `highlight_lines_for_fence_info`,
  `find_syntax_for_fence_info`, `parse_line_citation_fence_info`,
  `syntax_highlight_raw`. No two-face/build.rs/assets/Swift grammar.
  `SyntaxSet::load_defaults_newlines()` + `ThemeSet::load_defaults()`.
  Tests in `tests/md_syntax.rs` (5). Option A (two-face) deferred.
- T5c adds `src/md/url_scan.rs`: plain-URL detection over rendered ratatui
  Lines — `detect_plain_urls`, `detect_plain_urls_with_offset`,
  `for_each_plain_link` (verbatim from upstream). Anticipates `HyperlinkTarget`
  in new `src/md/output.rs` (full output module is T7). Tests in
  `tests/md_url_scan.rs` (10).

## NOT included
- Inference logic (connects to external server)
- Model loading (that's the engine's job)
