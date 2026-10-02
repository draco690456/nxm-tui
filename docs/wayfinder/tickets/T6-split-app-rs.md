# T6 — Split app.rs for the 300-line RULES.md cap

- **Label:** `wayfinder:task`
- **Type:** Task (refactor, behavior-preserving)
- **Status:** ✅ Resolved 2026-10-02 (free agent exec + independent review) — app.rs 973→729, commands.rs 165 + metrics.rs 100, clippy = baseline (3, no new)
- **Map:** RULES.md debt (max 300 lines/file)

## Problem

`src/app.rs` is ~973 lines — well over the RULES.md 300-line cap. It mixes
several separable concerns.

## Scope (this ticket — SAFE subset)

Behavior-preserving extraction, NO logic changes:
1. **`src/commands.rs`** ← `Command`, `ProviderCommand`, `ServerCommand`,
   `ConfigCommand`, `parse_command` (app.rs ~192–369).
2. **`src/metrics.rs`** ← `Metrics` + its impl (app.rs ~528–617).

**Compatibility requirement:** existing imports `nxm_tui::app::{parse_command,
Command, ProviderCommand, Metrics, ...}` MUST keep working (tests + handler use
them). Achieve this with `pub use crate::commands::*;` / `pub use
crate::metrics::Metrics;` re-exports in `app.rs`. Zero changes to call sites.

## Out of scope (later tickets)

- Moving `Message`/`Role` (60+ call sites — higher churn).
- Splitting `main.rs::run` (340-line fn) into sub-functions.

## Done when

`app.rs` meaningfully smaller; `commands.rs` + `metrics.rs` created & re-exported;
`cargo test` fully green IN PARALLEL (same baseline, incl. known flaky keys),
clippy 0 on touched files, no behavior change. Prompt:
`T6-PROMPT-for-free-agent.md`.
