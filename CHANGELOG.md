# Changelog

# 2026-10-02

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
