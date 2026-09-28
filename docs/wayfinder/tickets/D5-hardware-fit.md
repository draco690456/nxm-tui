# D5 — Hardware-aware fit (RAM/GPU budget + TurboQuant) in the terminal

- **Label:** `wayfinder:task`
- **Type:** Task (design + small prototype)
- **Status:** Open · Unclaimed · Unblocked
- **Blocked by:** P1 (loop validated) + D3 (crate location) — soft.

## Question

Port `harness-core`'s hardware-aware surface into the pure-Rust TUI, that area
today lives only in the Tauri GUI:

- per-model **RAM / GPU-RAM fit** ("does it fit my machine?"),
- **TurboQuant** selection (`kv-cache.turboquant`) from the terminal,
- the `fit_score` / budget summary panels.

## Background (from MAP "Not yet specified")

- Engines to front: `engines.serve-{cpu,metal,mlx-c,ssd,turboquant}`.
- The budget truth lives in `kv-cache.turboquant`, `kv-cache.disk`, and the
  `shared` / `model-plan` crates — the TUI must surface it, not guess it.
- Terminal vendor refs are chat-only (kimi/codex/gemini); the closest Rust
  GPU-fit UI refs are `pmetal`, `uzu`, `mistral-rs` (R2).

## Acceptance

- `fit <model>` prints RAM requirement vs. available RAM/GPU-RAM + a fit verdict.
- A TurboQuant toggle is reachable from the TUI (sets `kv-cache.turboquant`).
- Decision resolved: **reimplement** fit logic in the TUI crate vs. **delegate**
  to an engine-side `/fit` OpenAI tool.

## Decision record (fill when resolved)

- Chosen: reimplement / delegate
- Budget source: `nxm-shared` / `kv-cache.turboquant` / engine RPC
- Engines consulted: …
