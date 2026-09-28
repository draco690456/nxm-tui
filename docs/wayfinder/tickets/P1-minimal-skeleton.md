# P1 — Minimal pure-Rust TUI skeleton

- **Label:** `wayfinder:prototype`
- **Type:** Prototype (HITL)
- **Status:** 🔄 Claimed · In progress (P1 prototype, chart session)

## Question

Produce the cheapest, roughest concrete artifact to lock the next decisions:
a bare `ratatui` + `crossterm` "hello world" that opens a terminal, renders a
title + a status line, and exits on `q`. Modeled on the `hello_world`
example in `vendor/gpui/examples/hello_world.rs` (same shape: create
`Application` → open window → root view → render a `div`) and on
`llmfit-tui`'s crate set.

## Acceptance criteria
- `cargo run` prints a single terminal pane with a title and a status line.
- Honours `RULES.md`: `clap` for args, `nxm-shared`/`tracing` logging target
  `nexum::tui::runtime` (ENTER/EXIT), no `unwrap`/`expect`, doc-comment on
  public fn, ≤300-line file, tests in `tests/`.
- Pin ratatui/crossterm versions (decide 0.29/0.28 vs latest 0.30/0.29 — carry
  this decision from P1 into D2's crate list).

## Assets
- `/wayfinder/prototypes/p1-skeleton/` (to be created).
- `examples/hello_world.rs` in `vendor/gpui/` — the "create App → render div"
  shape to mirror in ratatui.

## Decision record (resolved)

- **Scaffold location:** `.wayfinder/prototypes/p1-skeleton/` — throwaway standalone
  crate, isolated from the `nxm-harness` workspace via an empty `[workspace]` table.
- **Versions (picked):** `ratatui = "0.29"`, `crossterm = "0.28"`, `clap = "4"`,
  `tokio = "1"`, `anyhow`, `tracing`, `tracing-subscriber` (fmt+env-filter).
  Resolved pins: ratatui 0.29.0, crossterm 0.28.1, clap 4.6.6, tokio 1.53.1 —
  matches the in-house `nxm-tui` anchor.
- **Traps hit on the way in (so the real crate skips these):**
  - ratatui 0.29 `DefaultTerminal::new()` is 1-arg (needs a backend) → manual
    raw-mode + alt-screen via `crossterm`, with explicit restore on exit.
  - `Constraint::Len` → it is `Constraint::Length` in 0.29.
  - `Block`/`Paragraph` come from `ratatui::widgets`, **not** `ratatui::prelude`.
- **Outcome:** `cargo check` ✅ · `cargo test` ✅ (2 integration + 1 doctest) ·
  `cargo clippy --all-targets` ✅ clean. Opens a terminal, renders a title pane
  + centered status line, exits on `q`/`Q`/`Esc`. Files: lib.rs 36 + main.rs 104
  + tests/tui.rs 21 = 161 lines (each ≤300). The GPUI "create App → window →
  root view → div" shape is mirrored in ratatui terms
  (setup terminal → `Terminal::draw` → `render_widget`).
