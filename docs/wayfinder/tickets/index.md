# Wayfinder tickets — index

Local-markdown tracker per la mappa `MAP.md` (`Pure-Rust TUI harness for Nexum`).
Stato: `resolved` · `open` · `deferred`. Una sessione rivendica una frontiera
registrandola in `MAP.md` "Frontier" prima di lavorarla. Regola: max **1 ticket
non-research risolto per sessione** (la ricerca è esclusa). I ticket `D5`/`T1`
sono **deferred** all'AgentAi backend (out-of-scope di nxm-tui, vedi MAP scope-split).

## Frontier (open · unblocked)

| Ticket | Labels | Nota |
|---|---|---|
| [R5-os-independent-keystore.md](R5-os-independent-keystore.md) | `wayfinder:research` ✅ | **Resolved**: studio completo (confronto backend, seam, spec). Raccomandazione: seam `KeyStore` + file cifrato come default portabile, OS-keychain/env dietro il trait. |
| *(vendor TUI adoption residua)* | | wrap unicode (openviking) ✅ **V1**; syntect highlight ✅ **V2**; residuo: streaming incrementale (**V3**), rifinitura resa visiva (lavoro dedicato). `search` CABLATO (T5 ✅). |
| *(RULES.md debt residuo)* | | T6 ✅ (app.rs 973→729). DEBT1 ✅ (`Message`/`Role` → `message.rs`); residuo: split `main.rs::run` (382 righe), split pieno `app.rs`. `VENDOR-TUI-ANALYSIS.md` ha la nota di licenza openviking **corretta** (root AGPL-3.0; copiabile solo `crates/ov_cli`). |

## Frontier — mappa provider keys + models (`MAP-provider-keys-models.md`)

| Ticket | Labels | Nota |
|---|---|---|
| [R1-keyring-crate.md](R1-keyring-crate.md) | `wayfinder:research` ✅ | Resolved: `keyring = "4"` adatta (dettagli in research). |
| [R2-models-api-shapes.md](R2-models-api-shapes.md) | `wayfinder:research` ✅ | Resolved: parser a due envelope (dettagli in research). |
| [D7-key-resolution-config.md](D7-key-resolution-config.md) | `wayfinder:grilling` | ✅ Resolved 2026-09-20: `keys.rs::resolve_key` (keychain→api_key_env→env), `default_model` in Provider, `api_key` globale rimossa, headless a+b+c, mai-log rigido. |
| [P3-key-entry-prompt.md](P3-key-entry-prompt.md) | `wayfinder:prototype` | ✅ **Resolved** (verificato nel codice 2026-10-02): `keys.rs::resolve_key`, `ProviderCommand::SetKey`, overlay mascherato (`ui.rs::render` set-key), `set_password` via keyring (`handler.rs:37`), buffer segreto dedicato azzerato dopo save/Esc, risoluzione per-provider allo spawn Agent (`main.rs` `spawn_blocking`). `keyring = "4.2.0"` pinnato. |
| [T2-provider-model-commands.md](T2-provider-model-commands.md) | `wayfinder:task` | **Open · parziale** (verificato 2026-10-02): `default_model` in `Provider` ✔; `GET /v1/models` presente **solo come health-probe** in `detect_endpoint` (NON lista modelli). Mancano: comando `/models` (lista+selezione, parser due-envelope R2), `/model use <id>`, `/provider remove-key`, modello attivo in mode/bottom bar. **Frontier residua.** |

## Deferred (→ nexum-agentai backend, fuori da nxm-tui)

| Ticket | Labels | Perché deferito |
|---|---|---|
| [T1-mcp-vector-search.md](T1-mcp-vector-search.md) | `wayfinder:task` | qdrant/memexd retrieval è backend; il TUI lo consuma via HTTP (solo se serve retrieval in-sandbox). |
| [D5-hardware-fit.md](D5-hardware-fit.md) | `wayfinder:task` | RAM/GPU budget + TurboQuant `fit` = AgentAi; TUI espone solo il fit-score via API. |

## In progress (claimed this session)

| Ticket | Labels | Stato |
|---|---|---|
| *(nessuno)* | | V2 risolto in questa sessione (vedi Resolved). DEBT1 + V1 + I1/I2/I3 keystore chiusi (sessioni 2026-10-02 → 2026-10-16). |

## Resolved (this chart session)
| Ticket | Labels | Esito |
|---|---|---|
| [V2-syntect-highlight.md](V2-syntect-highlight.md) | `wayfinder:task` ✅ | **Resolved 2026-10-16** (vendor, segue V1): syntax highlighting code-fence via `syntect 5.3.0` `default-fancy` (pure-Rust, no onig). Nuovo `src/highlight.rs` (`highlight_code` → span per-token, `OnceLock`, tema `base16-eighties.dark`, fallback plain-text); `markdown.rs` consuma per-span preservando box + cell-width V1. Eseguito da agent esterno + review indipendente + simulazione visiva (fixati: newline negli span, tema, fallback trailing). Verifica: 193 test verdi, clippy baseline (0 nuovi), file ≤300, no onig. PR #3. |
| [DEBT1-message-role-module.md](DEBT1-message-role-module.md) | `wayfinder:task` ✅ | **Resolved 2026-10-15** (debt RULES.md, post-T6): `Message`/`Role` estratti da `app.rs` in `src/message.rs` (100 righe) + re-export `pub use crate::message::{Message, Role};` (pattern T6), ~60 call site stabili senza modifiche; `push_thinking`/`apply_token` restano in `app.rs` (A1+B1). Eseguito da agent esterno (branch dedicato), review indipendente. Verifica: 185 test verdi, doctest 16/1 invariato, clippy = baseline (0 nuovi), ≤300 righe. PR #2. |
| [V1-unicode-wrap.md](V1-unicode-wrap.md) | `wayfinder:task` ✅ | **Resolved 2026-10-15**: `markdown.rs` misura a **larghezza di display** (`unicode-width`), adattando `wrap_display_text` di `openviking crates/ov_cli` (Apache-2.0; la root AGPL-3.0 non è stata copiata): append per parola + split char-level delle parole più lunghe, fence troncati per larghezza di cella, `prefix.width()`. `unicode-width = "0.2.0"` pinnato, `THIRD-PARTY-NOTICES.md` + nota di licenza in `VENDOR-TUI-ANALYSIS.md` corretti. 6 test in `tests/markdown.rs` (4/6 rossi sul renderer vecchio). Verifica: 180 passed / 0 failed / 1 ignored noto, clippy = baseline (0 nuovi), `markdown.rs` 268 righe. |
| [I1-keystore-seam.md](I1-keystore-seam.md) | `wayfinder:task` ✅ | **Resolved 2026-10-02**: seam `KeyStore` iniettabile + `EncryptedFileStore` (AES-256-GCM+Argon2id+zeroize, 0600) + `OsKeychainStore` + `EnvStore` dietro il trait; `resolve_key_with` ordine D7 generalizzato; `keychain_beats_env` riscritto su `MockKeyStore` (commit f2addf1). Wiring runtime → I2. |
| [I2-wire-keystore-runtime.md](I2-wire-keystore-runtime.md) | `wayfinder:task` ✅ | **Resolved 2026-10-02**: wiring in produzione — factory `store_for(mode, passphrase)` (errore pulito senza passphrase), passphrase 1x/sessione (`Zeroizing` + prompt mascherato), 4 read-path su `resolve_key_via` (factory + `resolve_key_with`) con gate "prompt, non spawnare", write-path `set-key`/`remove-key` sul trait. Verifica: build ok, 173 test verdi single-thread, clippy = baseline (0 nuovi). |
| [I3-keystore-cleanup-migrate.md](I3-keystore-cleanup-migrate.md) | `wayfinder:task` ✅ | **Resolved 2026-10-15** (follow-up I2): opzione A — `resolve_key` legacy **rimosso**, test migrati sul seam (`resolve_key_with`+`EnvStore`/`MockKeyStore`, niente mock keychain process-global); `/provider migrate-keys` (`OsKeychainStore`→`EncryptedFileStore`, idempotente, fail-soft, mai-log) in `keystore/migrate.rs` + 7 test. Verifica: 178 test verdi single-thread, clippy = baseline (0 nuovi), file ≤300 righe. |
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
