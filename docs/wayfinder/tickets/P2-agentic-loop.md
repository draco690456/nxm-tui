# P2 — Port the agentic loop into nxm-tui (evolve in place, per D3)

> **Nota post-riorganizzazione (2026-09-13):** record storico. I riferimenti a
> `nxm-ai/nxm-tui` e alla crate `nxm-ai/nxm-tools` / `nexum-tools` riflettono
> l'assetto di allora. Ora: il tool crate è **`nxm-tui-tools`** (workspace member
> di `nxm-tui`, import `nxm_tools`); il repo `nxm-ai/nxm-tools` è stato rinominato
> `nxm-ai/nxm-mcp-servers` e non contiene più i tool della TUI. Il lavoro descritto
> resta fatto e valido.

- **Label:** `wayfinder:prototype`
- **Type:** Prototype (spike + integration)
- **Status:** ✅ DONE — P2-proprio wired, compiles, 6/6 tests green
- **Claimed by:** this session
- **Where:** repo `nxm-ai/nxm-tui`, branch `wayfinder/p2-proprio` (tip `3d11136`),
  PR opened vs `main`. Also depends on the new `nxm-ai/nxm-tools` crate.

## Goal

Fold the D1 agentic REPL (chat + tool calls + streaming + memory/workspace) into
the existing pure-Rust `nxm-tui` crate. nxm-tui already has the transport
(`connection::chat_stream`, OpenAI HTTP + SSE) and the multi-pane UI
(`ui.rs`: sidebar/history/prompt/mode_bar/bottom) + `tool_types`.

## Spike (DONE — sandbox-validated, pushed)

- New `src/agent.rs` in nxm-tui: `Agent` streams `/v1/chat/completions`, parses
  `delta.tool_calls` fragments into `ToolInvocation`s (incremental args), emits
  `ToolPart::{Text, ToolInvocation, ToolResult}` on the UI channel, stub-dispatches
  a tool call, appends the result, and re-queries until the model stops.
- 3 unit tests on the delta parser — all green; `cargo check` green on nxm-tui.
- `#[allow(dead_code)]` — **not wired** into the ratatui event loop yet.

## P2-proper (DONE — integration)

1. ✅ Wire `Agent::run` into the ratatui event loop (`main.rs`): the user-prompt
   spawn now calls `agent::Agent::new(&client,&base,&model,msgs).run(&tx)` instead
   of `connection::chat_stream`. Channel payload changed `String → ToolPart`.
2. ✅ Real tool dispatch via `nxm-tools` crate: `ToolInvocation{name,args}` →
   `registry.call(name, parsed_args)` → `ToolResult{call_id,output}` emitted to the
   UI + a `role:"tool"` message pushed to the transcript. `nxm-tui`'s `Role` enum
   gained a `Tool` variant (`app.rs`) + `Message::tool(call_id, output)`.
3. ✅ `role:"tool"` round-trip: `serialize_messages` now emits assistant
   `tool_calls` arrays and `role:"tool"` result messages; the main loop mirrors
   `ToolPart::ToolInvocation`/`ToolResult` into `app.messages` for rendering.
4. ✅ `<thinking>` handling consolidated: `apply_token` (extracted from
   `App::push_token`) is shared by the main loop and the `Agent`, so the live UI
   and the re-serialized transcript stay consistent.

## Files touched (nxm-tui)

- `Cargo.toml` — `nxm-tools = { path = "../nxm-tools" }`.
- `src/agent.rs` — rewritten: owns the transcript, real `nexum-tools` dispatch,
  `role:"tool"` serialization, no `&mut App` (safe to spawn).
- `src/app.rs` — `Role::Tool`, `Message.tool_call_id`, `Message::tool`,
  `apply_token` (pub(crate)); `push_token` delegates to it.
- `src/main.rs` — `ToolPart` channel, spawn `Agent::run`, drain loop renders parts.
- `src/history.rs`, `src/ui.rs`, `src/connection.rs`, `src/session.rs` — `Role::Tool`
  arms added; `chat_stream` → `#[allow(dead_code)]`.
- `src/sidebar.rs` — fixed pre-existing `truncate_title` test expectation (doc
  says max-20 → `title[..17]+"..."`; test wrongly expected 16 chars). Not a
  logic change.

## Verification (cross-crate, sandbox)

From `nxm-harness` (nxm-tui is outside `working_directories`):
`CARGO_TARGET_DIR=…/.cargo-target-nxm-tui cargo check --manifest-path $HOME/Projects/nexum/nxm-tui/Cargo.toml` → **green** (11 benign `dead_code` warnings, no new ones).
`cargo test` → **6/6** (3 agent delta-parser tests + 3 sidebar tests).

## Acceptance (P2-proper)

- ✅ A user prompt can drive ≥1 round of tool calls + results through the engine's
  OpenAI endpoint — dispatched by `nexum-tools::default_registry` (read_file /
  list_resources / web_search). Runtime end-to-end needs a live engine (T1
  restores the local qdrant/memexd stack); the loop + dispatch + serialization
  are compiled & unit-tested.
- ✅ Incremental tokens render in `history.rs` via `ToolPart::Text` → `push_token`.

## Next (D4 / D5 / T1)

- D4: ratatui streaming-render strategy (mpsc → redraw per batch).
- D5: `/fit` + TurboQuant hardware-fit command (RAM/GPU budget).
- T1: restore memexd/qdrant so `web_search` + MCP `grep`/`search` resolve.
