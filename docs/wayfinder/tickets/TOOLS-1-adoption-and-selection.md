# TOOLS — Adozione tool di grok-build + selezione per-task

- **Label:** `wayfinder:task` (epic)
- **Type:** Adozione (estende il `Registry` di `nxm-tui-tools`)
- **Status:** 🚧 In design 2026-10-18
- **Related:** V3-render-adoption (stesso scopo: replicare+adattare grok in nxm-tui)

## Scopo (deciso dall'utente)

Due obiettivi legati:
1. **Un registry centrale** dei tool (ESISTE già: `nxm-tui-tools::Registry` —
   `register/get/call/manifest`) + **selezione del sottoinsieme rilevante** al task
   inviato al modello (oggi `agent.rs` manda TUTTI i tool via `manifest()`).
2. **Copiare i tool di grok-build** dentro nxm-tui, adattandoli (pure-Rust, dietro
   il nostro `trait Tool`), per priorità.

## Parte A — Meccanismo di selezione (design minimale, da grok ma adattato)

grok ha un sistema ricco (`tool_taxonomy.rs`, `tool_metadata.rs`,
`requirements.rs`, `registry/types.rs` 5580 righe) — **sovradimensionato**. Prendiamo
il CONCETTO (kind/categoria + subset + core-sempre-inclusi), non le 6000 righe.

- **MT-1** (no behavior change): `enum ToolKind {Read,Search,Edit,Core,Other}` +
  metodi default `kind()`/`keywords()` sul trait `Tool`; implementati sui 3 tool
  attuali. `manifest()` invariato.
- **MT-2**: `Registry::select(task)` (core sempre + keyword-match case-insensitive
  + **fail-open** se 0 match) + `manifest_for(task)`.
- **MT-3**: in `agent.rs::stream_turn`, `manifest()` → `manifest_for(&task)` (task =
  ultimo messaggio user). `call()`/`get()` restano sul registro completo.
- Strategia: keyword-match deterministico ora; evolvibile (capability/LLM-choice)
  dopo, senza riscritture.

## Parte B — Inventario tool grok-build (per decidere le priorità)

Fonte: `crates/codegen/xai-grok-tools/src/implementations/grok_build/`.
Dimensioni = righe upstream (incluse sandbox/permessi/test → la copia pure-Rust
adattata sarà MOLTO più piccola per molti). nxm-tui HA GIÀ: `read_file`,
`list_resources` (~`list_dir`), `web_search`.

### Core / editing file (priorità ALTA — un agente di coding li usa sempre)
| Tool | Righe grok | Cosa fa | Note adozione |
|---|---|---|---|
| `read_file` | 3151 | Legge file (range righe, citazioni) | **già abbiamo** una versione base; valutare upgrade |
| `list_dir` | 1640 | Elenca directory | **già** `list_resources`; allineare nome/schema |
| `grep` | 1621 | Ricerca regex nei file | ALTA — manca, fondamentale |
| `search_replace` | 2648 | Edit mirato (find→replace) nei file | ALTA — manca, è il write/edit |
| `bash` | 6124 | Esegue comandi shell (sandbox) | ALTA ma COMPLESSO (sandbox/permessi) → epic a sé; qui serve il VT engine (differito) |

### Pianificazione / task (priorità MEDIA — danno struttura all'agente)
| Tool | Righe | Cosa fa |
|---|---|---|
| `todo` | 1005 | Todo-list dell'agente |
| `update_goal` | 503 | Aggiorna il goal corrente |
| `enter_plan_mode` / `exit_plan_mode` | 711 / 409 | Modalità pianificazione |
| `task` | 3654 | Spawn di sub-task/sub-agenti |
| `task_output` / `kill_task` / `scheduler` / `monitor` | 2852 / 824 / 19 / 4 | Gestione task asincroni |
| `send_subagent_message` | 304 | Comunicazione tra sub-agenti |
| `workflow` | 754 | Orchestrazione workflow |

### Interazione utente (priorità MEDIA)
| Tool | Righe | Cosa fa |
|---|---|---|
| `ask_user_question` | 1187 | Chiede input all'utente |
| `send_feedback` | — | Invia feedback |

### Web (priorità BASSA — abbiamo già web_search)
| Tool | Righe | Cosa fa |
|---|---|---|
| `web_search` | 145 | Ricerca web (**già abbiamo**) |
| `web_fetch` | 229 | Scarica/estrae una URL | utile, piccolo → candidato facile |

### Media (priorità BASSA — fuori scope TUI leggera per ora)
| Tool | Righe | Cosa fa |
|---|---|---|
| `image_gen` / `image_edit` / `video_gen` | 727 / 717 / 2108 | Generazione media | servono protocolli immagine terminale → differiti |

### Altri set (lsp, memory, skills, codex/opencode/cursor adapters)
`implementations/{lsp,memory,skills,codex,opencode,...}` — integrazioni avanzate
(LSP, memoria, skill discovery, adattatori per altri agent). Da valutare dopo il core.

## Priorità proposta (da confermare con l'utente)
1. **P1 (meccanismo):** MT-1→MT-3 (selezione per-task sul Registry esistente).
2. **P2 (tool facili e utili):** `web_fetch` (229), `grep` (il più utile che manca).
3. **P3 (editing):** `search_replace` (il write), upgrade `read_file`/`list_dir`.
4. **P4 (planning):** `todo`, `update_goal`, plan_mode.
5. **Differiti/epic:** `bash` (sandbox+VT), `task`/sub-agenti, media, lsp/memory/skills.

## Done when (per l'epic; i singoli tool = ticket propri)
- Parte A implementata (select + manifest_for + attivazione).
- Lista priorità confermata; ogni tool copiato = ticket a sé (pure-Rust, trait Tool,
  kind/keywords, test), consolidato come V3.
