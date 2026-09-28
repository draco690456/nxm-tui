# Wayfinder tickets — index

Local-markdown tracker per la mappa `MAP.md` (`Pure-Rust TUI harness for Nexum`).
Stato: `resolved` · `open` · `deferred`. Una sessione rivendica una frontiera
registrandola in `MAP.md` "Frontier" prima di lavorarla. Regola: max **1 ticket
non-research risolto per sessione** (la ricerca è esclusa). I ticket `D5`/`T1`
sono **deferred** all'AgentAi backend (out-of-scope di nxm-tui, vedi MAP scope-split).

## Frontier (open · unblocked)

| Ticket | Labels | Nota |
|---|---|---|
| *(nessuna — frontend MVP `pi.dev`-style chiuso a P2+D4)* | | Prossima: UX polish/layout; big-ticket → AgentAi. |

## Frontier — mappa provider keys + models (`MAP-provider-keys-models.md`)

| Ticket | Labels | Nota |
|---|---|---|
| [R1-keyring-crate.md](R1-keyring-crate.md) | `wayfinder:research` ✅ | Resolved: `keyring = "4"` adatta (dettagli in research). |
| [R2-models-api-shapes.md](R2-models-api-shapes.md) | `wayfinder:research` ✅ | Resolved: parser a due envelope (dettagli in research). |
| [D7-key-resolution-config.md](D7-key-resolution-config.md) | `wayfinder:grilling` | ✅ Resolved 2026-09-20: `keys.rs::resolve_key` (keychain→api_key_env→env), `default_model` in Provider, `api_key` globale rimossa, headless a+b+c, mai-log rigido. |
| [P3-key-entry-prompt.md](P3-key-entry-prompt.md) | `wayfinder:prototype` | **Frontier**: sbloccato da D7 — prossimo step. |
| [T2-provider-model-commands.md](T2-provider-model-commands.md) | `wayfinder:task` | **Frontier**: sbloccato da D7 + R2 — dopo P3. |

## Deferred (→ nexum-agentai backend, fuori da nxm-tui)

| Ticket | Labels | Perché deferito |
|---|---|---|
| [T1-mcp-vector-search.md](T1-mcp-vector-search.md) | `wayfinder:task` | qdrant/memexd retrieval è backend; il TUI lo consuma via HTTP (solo se serve retrieval in-sandbox). |
| [D5-hardware-fit.md](D5-hardware-fit.md) | `wayfinder:task` | RAM/GPU budget + TurboQuant `fit` = AgentAi; TUI espone solo il fit-score via API. |

## In progress (claimed this session)

| Ticket | Labels | Stato |
|---|---|---|
| *(nessuno — sessione conclusa a T3 ✔ + T4 ✔ + D7 ✔)* | | |

## Resolved (this chart session)
| Ticket | Labels | Esito |
|---|---|---|
| [research/vendor-interrogation.md](../research/vendor-interrogation.md) | `wayfinder:research` ✅ | R1/R2/R3 locked. |
| [D1-tui-purpose.md](D1-tui-purpose.md) | `wayfinder:grilling` ✅ | lean agentic REPL, stile `pi.dev` (stream + tool + workspace). |
| [D2-architecture.md](D2-architecture.md) | `wayfinder:grilling` ✅ | A) standalone pure-Rust binary (ratatui/crossterm/tokio/clap/reqwest). |
| [D3-relation-to-nxm-tui.md](D3-relation-to-nxm-tui.md) | `wayfinder:grilling` ✅ | Evolvi `nxm-tui` in place. |
| [D6-tools-crate.md](D6-tools-crate.md) | `wayfinder:grilling` ✅ | Tool crate creato (4 test verdi). Post-reorg 2026-09-13: è `nxm-tui-tools`, workspace member di `nxm-tui` (import `nxm_tools`). |
| [D4-streaming-model.md](D4-streaming-model.md) | `wayfinder:grilling` ✅ | Option A (mpsc → drain 16ms/tick ~60Hz); cancel = abort inference_task on Quit (main.rs). Impl. in P2-proprio. |
| [P1-minimal-skeleton.md](P1-minimal-skeleton.md) | `wayfinder:prototype` ✅ | Skeleton; check/test/clippy green; ritirato a favore di D3 (evolvi nxm-tui). |
| [P2-agentic-loop.md](P2-agentic-loop.md) | `wayfinder:prototype` ✅ | DONE: `Agent::run` nel loop ratatui, `Role::Tool`, dispatch tool crate reale (ora `nxm-tui-tools`/`nxm_tools`), `check`+6 test verdi, PR #1. |
| [R4-efficiency-audit.md](R4-efficiency-audit.md) | `wayfinder:research` ✅ | Resolved 2026-09-20: causa = redraw completo ~60 Hz + ricostruzione history/markdown O(N) per frame; fix top in research. |
| [T3-efficiency-fixes.md](T3-efficiency-fixes.md) | `wayfinder:task` ✅ | Resolved 2026-09-20: draw-on-change + poll 33 ms + cache rendering history (64/64 test verdi, commit f158325). |
| [T4-micro-perf.md](T4-micro-perf.md) | `wayfinder:task` ✅ | Resolved 2026-09-20: estimated_tokens memoizzato + cache cwd (68/68 verdi). |
| P3-provider-connect ✅ (no ticket file) | `wayfinder:prototype` | Connectable a OpenAI/NVIDIA/Ollama: `api_key` (env `NEXUM_API_KEY`/`OPENAI_API_KEY`) → `Authorization: Bearer` sulla /chat/completions + parsing `reasoning_content` → `ToolPart::Reasoning`. `main.rs` passa la key all'Agent. `tests/roundtrip.rs` verifica headless un tool round-trip reale (`read_file` → role:"tool") + bearer; 10/10 test verdi, clippy 0 error. (@ e9413e9) |

## Out of scope (di nxm-tui)

- TurboQuant / RAM-GPU fit, motore vettoriale embeddato / MCP vendor — questi
  sono del `nexum-agentai` backend, non del frontend TUI.
- GPUI / TS / React / ink / WASM / GUI Tauri — vincolo è Rust puro, TUI leggera.
