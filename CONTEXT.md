# nxm-tui

Terminal UI client for NXM inference servers. Built with ratatui + crossterm.

## In progress (micro-task 40 — not yet merged)

`feat/tools-update-goal` / work branch `work/40-update-goal`: `update_goal` tool
setting the agent's current goal in shared `App.goal_handle` (`Arc<Mutex<Option<String>>>`),
rendered inline in the mode bar (` 🎯 <goal>`, truncated ~40 col, only when
Some/non-empty). No overlay/key. `src/goal.rs`, `src/update_goal.rs` new;
`src/app.rs`, `src/mode_bar.rs`, `src/lib.rs` touched; `tests/update_goal.rs`
(7 tests, R1..R5). Gate green (`cargo test --test update_goal` 7/7,
`cargo test --workspace`, clippy 0 new warnings, fmt ok). Pending external
review before `--ff-only` into `feat/tools-update-goal`. No commit on
`main`/`develop`.

## Purpose
Connects to any OpenAI-compatible LLM server (including NXM engines) and provides
an interactive chat interface in the terminal.

## Features
- Streaming chat responses (SSE)
- Markdown rendering in terminal — cell-width aware (CJK/emoji count two columns,
  over-long words split, code fences truncated by width; V1, adapted from
  openviking `crates/ov_cli` wrap_display_text — Apache-2.0, see
  `THIRD-PARTY-NOTICES.md`). Since T8, `render_markdown` can (env
  `NXM_USE_NEW_MD=1`, default OFF) render via the V3 rich pipeline
  (`src/md/`, grok-build `xai-grok-markdown` port: headings, lists, bold,
  inline/fenced code, GFM tables, links); legacy stays as fallback.
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
- T5d adds `src/md/open_code_highlighter.rs`: incremental syntax-highlighting
  cache for streamed code fences (verbatim from upstream). `OpenCodeHighlighter`
  (`new`, `highlight_block`, `highlight_closed`, `highlight`,
  `committed_prefix_matches`), `HlLine`, `CLOSED_MEMO_CAP_BYTES`. No two-face,
  no build.rs, no assets, no Swift grammar; `Syntect::new()` (option B defaults).
  Tests in `tests/md_open_code.rs` (8).
- T5e adds `src/md/parse.rs`: scaffold of `MarkdownParser` — `new()`,
  builders `collapse_soft_breaks()` / `max_table_width()`, `parse()` entry
  point (iterates `offset_events` → stub `on_event` → `ParsedMarkdown`),
  `ParsedMarkdown` struct + `new()`. Pure helpers: `StyleInto` trait + impl,
  `anstyle_to_ratatui_color`, `find_substring`, `decode_html_entity`,
  `has_blank_line_after`, `is_br_tag`, `PendingCodeBlock`. `on_event`/`on_start`
  /`on_end` are empty stubs (T5f); `format_table` omitted (T5g); `latex`
  deferred. Tests in `tests/md_parse.rs` (9).
- T5f adds `src/md/parse_events.rs` (`on_event`/`push_highlight`),
  `src/md/parse_blocks.rs` (`on_start`/`on_end`/`format_table` stub),
  `src/md/parse_inline.rs` (inline helpers: `style_inline_code_span`,
  `scan_inline_html_entities`, `push_table_text_linking_urls`,
  `try_push_mermaid` stub, `push_display_math_block` stub),
  `src/md/latex.rs` (stub: `latex_to_unicode_inline`/`latex_to_unicode_display`
  return `None`). Parser now produces real output for headings, bold, inline
  code, strikethrough; math degrades gracefully via stub. Tests in
  `tests/md_parse_events.rs` (4).
- T5g adds `src/md/parse_table.rs` (+ `parse_table_fmt.rs`,
  `parse_table_word.rs`): real `format_table` (verbatim from upstream,
  laid out per the 300-line cap) — replaces the T5f stub; `on_end` now pushes
  real `TableReplace`s into `buffers.table_replaces`, so tables render as
  box-drawing output with hyperlinks, cell-copies and source line offsets.
  Rust-2021 rewrites documented in the file headers. Tests in
  `tests/md_parse_table.rs` (7).
- T6a adds `src/md/source_map.rs`: verbatim copy of upstream `SourceMap`
  (byte-level rendered→source mapping for copy/selection; `new`/`add`/
  `to_source`/`is_empty`/`len`/`extend_with_offsets`/`segments`/`truncate`/
  `clear`). Pure logic, no deps. Prerequisite of the T6 render pipeline.
  Tests in `tests/md_source_map.rs` (5).
- T6b completes `src/md/output.rs` (verbatim production slice of upstream
  `output.rs`): `CodeBlockSpan`, `MarkdownRenderOutput` (+ `new`/`clear`/
  `as_view`), `MarkdownRenderView<'a>` (+ `line_count`), pure
  `build_code_block_spans` mapping (`CodeBlockMeta` → `CodeBlockSpan` over
  the `line_source_map`). `HyperlinkTarget` unchanged (no duplication).
  Render types ready for the T6d render pipeline. Tests in `tests/md_output.rs`
  (7).
- T6c adds `src/md/hyperlinks.rs` (verbatim production copy of upstream
  `hyperlinks.rs`, 153 lines ≤ cap): link projection onto display cells
  across the three coordinate systems (source bytes → transformed bytes →
  display cells) — `ChunkLinkRange`, `source_to_chunk_offset`,
  `chunk_link_offsets`, `emit_segment_hyperlinks` (per-segment
  `HyperlinkTarget`s sharing one `id` for wrapped links, CJK display-width
  aware). Upstream inline tests (renderer-dependent) NOT copied (T16);
  adapted renderer-free tests in `tests/md_hyperlinks.rs` (5). Last render
  prerequisite before T6d.
- T6d adds the render core (verbatim production copy of upstream
  `render.rs`, ratatui path, split per the 300-line cap across
  `render.rs` (279) + `render_events.rs` (223) + `render_segments.rs`
  (247) + `render_blocks.rs` (240), all `impl ParsedMarkdown` with a
  `RenderLoopState` for the loop locals): `render_ratatui` returns
  `(MarkdownRenderOutput, Option<Checkpoint>)`; the ANSI path is NOT
  copied (0 calls). Reuses `StyleInto`/`anstyle_to_ratatui_color` from
  parse.rs (latter widened pub(crate)), the T1 `adapt_style` stub, and the
  T6c link projection in both text paths. Renderer-free tests in
  `tests/md_render.rs` (4: heading/bold/code, code fence, GFM table,
  link hyperlinks). Parse + render now complete → the swap is next
  (T8/T10).
- T8 (SWAP) wires the batch render entry and keeps `history.rs` frozen:
  `src/md/mod.rs::render` now assembles the real pipeline
  (`MarkdownParser::new(content, fixed_style(), &mut buffers, syntect.as_ref())`
  → `.max_table_width(Some(max_width))` → `.parse()` → `.render_ratatui(true)`
  → `output.lines`; fresh `MarkdownBuffers` per call, `Syntect::new()`
  propagated as `Option` — no `unwrap`/`expect`). `max_width` currently
  bounds GFM table columns only; full prose wrapping is T9/T10 (documented
  in the doc comment). `src/markdown.rs::render_markdown` becomes a
  dispatcher on the `NXM_USE_NEW_MD` env flag (default **OFF** → legacy
  fallback): ON runs the V3 pipeline and re-applies `prefix` to every line
  (`with_prefix`). The V1 renderer moved verbatim to `src/markdown_legacy.rs`
  and its layout helpers to `src/markdown_layout.rs` (RULES 300-line cap);
  `history.rs` (and `highlight.rs`) untouched. Tests in `tests/md_swap.rs`
   (6). NOTE: both `markdown.rs` and `markdown_legacy.rs` carry
  `tracing::debug!` (target `nexum::markdown::swap`) as they are our code.
- T9a adds `src/md/line_utils.rs` (196 lines ≤ cap): verbatim copy of
  pager-render (NOT xai-grok-markdown) `render/line_utils.rs` prod 1-209 —
  `line_to_static`, `push_owned_lines`, `truncate_str`, `truncate_line`,
  grapheme-aware `fit_line_to_width` (clip/pad, no ellipsis — kills ghost
  glyphs beside tables), `cascade_truncate`, private `take_width`. Zero
  xai-* refs: `floor_char_boundary` reused from `crate::md::buffers`
  (dedup); `truncate_str`/`take_width` backed by
  `crate::markdown_layout::truncate_to_width` (ellipsis-free, matching the
  delegation contract), upstream `pub use` of un-ported xai-* helpers
  dropped — all documented in the header `Modified:` clause. Registered as
  `pub mod line_utils;`. Prerequisite of T9b (wrap). Tests in
  `tests/md_line_utils.rs` (28, adapted from the upstream inline tests).
- T9b adds the word-wrap, verbatim copy of pager-render
  `render/wrapping.rs` prod 1-690 (upstream inline tests NOT ported: they
  need `crate::theme`/`safe_buf`), split for the 300-line cap into
  `src/md/wrap.rs` (byte-range helpers + `RtOptions` + relocated
  `is_table_line`/`blockquote_prefix_len`, `pub(crate)`),
  `src/md/wrap_line.rs` (`word_wrap_line`, `word_wrap_line_with_joiners`,
  `word_wrap_lines(_with_joiners)`, span slicing) and `src/md/wrap_input.rs`
  (`LineInput`/`IntoLineInput` made `pub` for visibility-for-testability,
  `wrap_header_hanging`/`wrap_header_flush`). line_utils resolved via
  `crate::md::line_utils`. Copy-only: NOT wired into the render pipeline
  yet (T10). Tests in `tests/md_wrap.rs` (5, behavioural: width ≤ width,
  short line unchanged, multi-line wrap, style preservation, table lines
  never wrapped).
- T10 wires the wrap into the render: `md::render` applies
  `crate::md::wrap_line::word_wrap_lines(output.lines, max_width)` right
  after `render_ratatui` — prose now fits the pane (was 1 row of 182 cols
  at width 40; closes Fase 2). Our own integration (RULES piene), no
  `unwrap`/`expect`. Table lines are skipped by the wrap engine
  (`is_table_line`) so pre-formatted tables stay whole (rows are padded to
  `max_width` by `fit_line_to_width`, invisible trailing spaces); code-fence
  marker rows stay whole, long code content wraps like prose. Tests in
  `tests/md_wrap_integration.rs` (5): the Fase-2 key test (every row ≤ 40
  for ~180-col prose), short prose untouched, table rows never split, code
  fences whole, styles preserved across wrapped rows.
- T14a (prerequisiti streaming T14b): `src/md/latex.rs` `LatexDelimiterNormalizer` stub passthrough; `src/md/mod.rs` `render_markdown_ratatui_with_link_id` (gestisce il link_id counter) + builder `open_code` su `MarkdownParser`; `src/md/parse.rs` builder `link_id_start`. Test in `tests/md_stream_prereq.rs`. Lo streaming vero (`streaming.rs`) resta T14b, non implementato qui. Branch `work/T14a-md-stream-prereq` su `origin/main` (ecf798b).

## T1 — tool selection (in-memory)

- `nxm-tui-tools` `Registry` tiene `enabled: HashSet<String>` accanto a `tools: Vec<Arc<dyn Tool>>` (ora `Clone`). `register()` inserisce sempre il nome in `enabled` (invariante: default attivo). `manifest()` resta il manifest completo (compat); `manifest_selected()` filtra sui tool abilitati ed è ciò che l'agente manda al modello. `set_enabled`/`is_enabled`/`enable_all`/`disable_all`/`tool_names` esposti per l'UI. La selezione è ORTOGONALE al gating di approvazione (`needs_approval`): un tool può essere selezionato e comunque gated. I tool nativi (`ReadFile`/`ListResources`/`WebSearch`/`WebFetch`/`Grep`/`SearchReplace`) vivono in `nxm-tui-tools/` (`builtins.rs` + `grep.rs` + `ssrf.rs` + `search_replace.rs`), re-exportati alla radice. `WebFetch` ha guardia SSRF: blocca loopback/link-local/private/metadata/unspecified, solo http/https, segue redirect (max 10) validando ogni hop; `NEXUM_WEBFETCH_ALLOW_LOCAL=1` per dev. I test SSRF che toccano la env var sono serializzati con `#[serial]` (`serial_test`, dev-dep) per evitare race in esecuzione parallela (prima la suite poteva colpire un timeout ~30s per contaminazione). `Grep` (regex su `path`, output `file:line:testo`) è in `default_registry()`; esclusioni `.git/`/`target/`, file binari saltati, cap 1000 match; nessun glob/context-lines/ignore-file (follow-up separato). `SearchReplace` (exact-string find→replace, `old` deve essere univoco, write atomico tmp+rename) è in `default_registry()`; nessun regex/replace-all/multi-file (follow-up separato).
- `Agent::new` prende la `Registry` come argomento. `App` la possiede (`app.tool_registry`, costruita da `default_registry()`) ed è la fonte di verità della selezione; `main.rs` clona uno snapshot allo spawn. `Agent::request_body()` (funzione pura estratta da `stream_turn`) costruisce il body con `self.registry.manifest_selected()` — un tool disattivato dall'utente non compare in `body.tools`. I toggle dell'overlay valgono dal prossimo spawn.
- Overlay "Tools": `src/tools_overlay.rs` `render_tools` (checkbox `[x]`/`[ ]`, cursore `>`), stato in `App` (`show_tools`, `tools_cursor`). Tasti in `tools_overlay::handle_tools_key` (chiamato da `handler.rs`): Ctrl+E apre/chiude (Ctrl+T era già sessions), ↑/↓ muovono il cursore con wrap, Space toggola il tool sotto il cursore, `a` = enable_all, `n` = disable_all, Esc chiude. L'overlay è MODALE: mentre è aperto nessun tasto raggiunge il prompt (stesso contratto della ricerca). Lista piatta: il raggruppamento per sorgente serve con MCP → T2.
- Fuori scope T1: tool MCP / `McpTool` proxy / async (T2), persistenza TOML (fase 2), profili/preset.

## MT-1 — taxonomia per-task (ToolKind + kind()/keywords(), groundwork)

- `nxm-tui-tools/src/taxonomy.rs`: `enum ToolKind { Read, Search, Edit, Core, Web, Other }`
  (derive `Debug, Clone, Copy, PartialEq, Eq`), puro groundwork della selezione
  per-task (obiettivo: MT-2 `select(task)`/`manifest_for(task)` + fail-safe Core,
  MT-3 aggancio `agent.rs`). `Core` = `read_file` + `list_resources` (sempre
  inclusi dall'automatica E set fail-safe); `grep`=`Search`, `search_replace`=`Edit`,
  `web_fetch`/`web_search`=`Web`; `Read` riservato (nessun tool built-in lo usa);
  `Other` è il default del trait.
- Trait `Tool` (in `lib.rs`): metodi default `kind() -> ToolKind` e
  `keywords() -> &[&str]`, entrambi con doc `///` + doctest; ovveride nei blocchi
  `impl Tool` di ciascun tool. Keywords EN+IT lowercase per i non-Core
  (match case-insensitivo deciso, a MT-2); i Core non hanno keyword (non
  dipendono dal match). Set finale mandato al modello = unione `automatica ∪
  manuale` (unione con l'overlay T1, non intersezione).
- **Nessun comportamento nuovo**: `manifest()`/`manifest_selected()` e l'overlay
  T1 IDENTICI (nessun bump conteggi); suites `tools.rs`/`selection.rs`/
  `agent_request.rs` verdi senza modifiche. MT-2/MT-3 NON implementati qui.
- Test: `nxm-tui-tools/tests/taxonomy.rs` (5). Branch `work/33-select-mt1` su
  `feat/tools-select-mechanism` (da main); da consolidare dopo review esterna.

## NOT included
- Inference logic (connects to external server)
- Model loading (that's the engine's job)

## MT-2 — selezione automatica per-task (`select`/`manifest_for`, logica pura)

- `nxm-tui-tools/src/selection.rs` (nuovo): `select(registry, task)` — Core sempre
  (fail-safe) + keyword match case-insensitive substring, fail-safe = solo Core,
  ordine registrazione; `manifest_for(registry, task)` — unione `select ∪ enabled`
  per nome, no duplicati, ordine registrazione. Logging `tracing::debug!`
  (target `nexum::tools::runtime`) su select. `Registry::select` /
  `Registry::manifest_for` deleganti in `lib.rs` con doc `///` + doctest.
- Nessun cambio a runtime: `agent.rs`, overlay, default `enabled`/`register()`,
  `manifest()`/`manifest_selected()` invariati (MT-3). Su registry default
  `manifest_for` restituisce ancora tutti i tool (tutti abilitati).
- Test: `nxm-tui-tools/tests/selection.rs` — 7 nuovi test (R1..R6), isolano
  l'automatica con `disable_all()` + `set_enabled(...)` mirati.
- Branch `work/34-select-mt2` su `feat/tools-select-mechanism`; da consolidare
  con `--ff-only` dopo review esterna. Nessun commit su `main`/`develop`.

## MT-3 — switch atomico: default manuale vuoto + `manifest_for` in `agent.rs`

- `nxm-tui-tools/src/lib.rs`: `register()` non abilita più automaticamente; set manuale `enabled` parte vuoto. Doc-comment aggiornati.
- `src/agent.rs`: `request_body()` usa `registry.manifest_for(&task)` dove `task` = ultimo messaggio `Role::User` del transcript (stringa vuota → fail-safe soli Core). Logging `tracing::info!` (target `nexum::agent::request`). Doc/doctest aggiornati (2 tool = Core con transcript vuoto).
- `src/tools_overlay.rs` / `src/handler.rs`: meccanismo invariato, default tutti `[ ]` (utente *aggiunge* via overlay). Nessun cambio di codice UI necessario.
- Test aggiornati: `selection.rs` (default vuoto, registration non abilita), `grep`/`web_fetch`/`search_replace` `*_in_default_registry` (via `tool_names()`), `agent_request.rs` (R3: failsafe core-only + per-task selection; R4: manual addition), `tools_overlay_keys.rs` + `tools_overlay.rs` (default `[ ]`).
- `manifest_selected()` mantenuto (usato dai test); `agent.rs` non lo chiama più.
- Branch `work/35-select-mt3` su `feat/tools-select-mechanism`; da consolidare con `--ff-only` dopo review esterna. Nessun commit su `main`/`develop`.

## MT-1 — TodoList types + shared state (todo tool, pure logic, no tool/UI)

- `src/todo.rs` (new): `TodoItem { text, done }`, `TodoList` (wrapper over
  `Vec<TodoItem>`) with `add`, `complete_by_index`, `complete_by_text`,
  `list()`, `render()`; typed `TodoError`; `tracing::debug!` logging
  (target `nexum::todo::state`) on `add`/`complete_*`. `TodoListHandle =
  Arc<Mutex<TodoList>>` + `todo_list_handle()` (shared App↔Agent, same
  pattern as `tool_registry`).
- `src/lib.rs`: `pub mod todo;`.
- Test: `tests/todo.rs` (7 tests, R1..R5).
- Does not touch `agent.rs`, overlay, `nxm-tui-tools`, `src/md/`; no new tool
  in the registry; no runtime change.
- Branch `work/37-todo-mt1` on `feat/tools-todo` (from `main`); to consolidate
  with `--ff-only` after external review. No commit on `main`/`develop`.

## MT-2 — todo tool mutating shared TodoList state

- `src/todo_tool.rs` (new): `struct Todo { pub todos: TodoListHandle }`
  implementing `nxm_tui_tools::Tool`. Schema: `action` (required) + optional
  `text`/`index`. Actions: `add`, `complete` (by index or text), `list`.
  Missing/unknown action → `ToolResult::err`. Mutex lock poisoning handled
  without `unwrap`/`panic`. `tracing::info!` (target `nexum::todo::tool`) on
  `run`. `kind() == Other` with keywords `todo, task, plan, checklist, lista,
  piano` so per-task `select` includes it on planning tasks.
- `src/lib.rs`: `pub mod todo_tool;`.
- `src/app.rs`: `App` creates one `TodoListHandle` (via `todo_list_handle()`),
  clones it into the `todo` tool registered in `tool_registry`, and keeps the
  handle in `App.todo_handle` for the MT-3 UI. Same handle shared App↔tool.
- Test: `tests/todo_tool.rs` (7 tests, R1..R6).
- NOT touched: `agent.rs`, overlay, `nxm-tui-tools`, `src/md/`; no new
  dependency; `Todo` is not in `default_registry()` so existing selection
  tests are unaffected.
- Branch `work/38-todo-mt2` on `feat/tools-todo`; to consolidate with
  `--ff-only` after external review. No commit on `main`/`develop`.

## MT-3 — todo overlay (TUI, read-only)

- `src/todo_overlay.rs` (new): `render_todo(f, app)` renders `App.todo_handle`
  as `[x]`/`[ ]` checklist with "Todo" title; empty list shows "(no todos)";
  poisoned mutex shows error placeholder instead of panicking.
- `src/lib.rs`: `pub mod todo_overlay;`.
- `src/app.rs`: added `pub show_todo: bool` (default `false`).
- `src/ui.rs`: `render()` draws `render_todo` when `app.show_todo` is true.
- `src/handler.rs`: Ctrl+D toggles the todo overlay; Esc closes it (Ctrl+D
  is free — not used by any other overlay/action).
- Test: `tests/todo_overlay.rs` (4 tests, R1..R3 + bonus), `tests/todo_overlay_keys.rs`
  (3 tests, R4 + R5).
- NOT touched: `agent.rs`, `nxm-tui-tools`, `src/md/`, `TodoList`/`Todo`
  (MT-1/MT-2); no new dependency; no UI mutation in this MT.
- Branch `work/39-todo-mt3` on `feat/tools-todo`; to consolidate with
  `--ff-only` after external review. No commit on `main`/`develop`.
