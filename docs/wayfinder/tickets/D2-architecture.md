# D2 — Architecture: reuse harness-core/engines or standalone?

- **Label:** `wayfinder:grilling`
- **Type:** Grilling (HITL)
- **Status:** ✅ Resolved · Decided in chart session (2026-09-05)
- **Blocked by:** D1 (purpose) — resolved.

## Question

How does the pure-Rust TUI meet the Nexum engines and the existing codebase?
Two reference poles exist and they are very different:

- **`nxm-tui` model (standalone native):** a self-contained Rust crate
  (`ratatui`+`crossterm`+`tokio`+`reqwest`) that talks to engines via the
  **OpenAI-compatible HTTP API** (like `nxm-tui` does, like `grok-build`
  does via `async-openai`). Zero dependency on the Tauri `harness-core`.
- **`harness-core`/`engines` model (in-process reuse):** depend on
  `harness-core`, `nxm-modelplan`, `nxm-memory`, and/or link
  `engines.serve-*` directly. Tighter integration, but heavier.

## Resolution

- **Chosen: A) Standalone, pure-Rust binary, OpenAI-compatible HTTP to engines.**
- **Crate location & name:** a new standalone Rust crate. The P1 prototype
  lives as a throwaway under `.wayfinder/prototypes/p1-skeleton/`; final crate
  placement/identity is decided in **D3** (relation to nxm-tui).
- **Key deps:** `ratatui` + `crossterm` + `tokio` + `clap` + `reqwest`
  (+`anyhow`/`thiserror`); logging via `tracing` with target
  `nexum::tui::runtime` (use `nxm-shared` logging utilities per RULES.md).
  No link to `harness-core` / `engines.serve-*` in-process — the engines are
  reached over HTTP.
- **Rationale:** matches "semplice / leggero / veloce"; mirrors the proven
  stack in `nxm-tui` and the Rust vendor TUIs (codex, openviking, llmfit-tui);
  avoids coupling a terminal binary to the heavy Tauri `harness-core` tree.
- **Sources:** user grilling (D2), 2026-09-05; references: nxm-tui
  (ratatui 0.29/crossterm 0.28/reqwest), grok-build (async-openai),
  codex/llmfit-tui/openviking (ratatui/crossterm).

## Constraint honoured
Pure Rust only; `RULES.md` compliance (≤300-line files, tests in `tests/`,
no `unwrap`/`expect` in prod, `anyhow`/`thiserror`, `nxm-shared` logging,
bilingual docs) — carried into P1.
