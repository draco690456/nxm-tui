# D4 — Streaming token rendering in the TUI

- **Label:** `wayfinder:grilling`
- **Type:** Grilling (HITL) — needs the human
- **Status:** ✅ Resolved · Chosen this session (P2-proprio)
- **Claimed by:** this session
- **Blocked by:** P1 (prototype validates the event loop) — resolved.

## Question

How does the agentic REPL render streamed OpenAI tokens inside a ratatui frame
without blocking the input loop / key handling? Vendor refs (R2): `codex` and
`llmfit-tui` pull tokens off the async HTTP (SSE `data:`) stream and push them
through a `tokio` channel into the `Frame::render` loop.

## Options

- **(A) tokio mpsc channel + ratatui drain (preferred):** an async task reads
  the SSE stream and pushes `String`/`ToolPart` batches → the ratatui
  `event::poll` loop drains pending batches each tick and redraws. Matches
  `codex`/`llmfit-tui`.
- **(B) Double-buffered `Paragraph` + manual scrollback.**
- **(C) Lock-free ring buffer of tokens, redraw on a fixed tick timer.**

## Decision record (filled)

- **Chosen: A.** `Agent::run` runs in a `tokio::spawn` task that **owns** the
  transcript (never borrows `&mut App`); SSE `data:` deltas become `ToolPart`s
  pushed over an `mpsc::unbounded_channel<ToolPart>`. The ratatui loop drains
  pending parts each frame via `try_recv` → `App::push_token` /
  `Message::tool` → `history.rs` re-render. Because the agent is decoupled from
  the crossterm loop, `event::poll(Duration::from_millis(16))` (≈60 Hz, see
  `event.rs`) never stalls — the UI stays keyboard-responsive.
- **Batch size / tick interval:** no fixed batch — drain *all* pending parts
  per frame (`try_recv` loop); cadence set by the 16 ms poll (~60 Hz).
- **Stream-cancel contract:** on `RunState::Quit` (Ctrl-C / Ctrl-Q while
  streaming), `main.rs` calls `inference_task.abort()` before breaking — the
  in-flight SSE read is cancelled and the channel drops with the task. The
  partial assistant message is already in `app.messages` (no flicker/lose).
  `Esc`-cancel-turn-keep-partial is a small P3 polish; `Ctrl-C` cancel-exit is
  covered now.
- **Status:** implemented + verified in P2-proprio — `cargo check` + 6/6 tests
  green.

## Acceptance (verified)

- ✅ Tokens arrive incrementally (SSE → `ToolPart::Text` → `push_token` →
  `history.rs`).
- ✅ UI stays responsive (Agent in a spawned tokio task; `poll(16ms)` drain).
- ✅ Cancel in-flight stream on quit (`inference_task.abort()`).
