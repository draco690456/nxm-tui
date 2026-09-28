# 🗺️ Wayfinder Map — Pure-Rust TUI harness for Nexum

> **⚠️ Allineamento post-riorganizzazione (2026-09-13).** Nomi/percorsi aggiornati
> dopo la riorganizzazione dei repo. Mapping storico → attuale (i ticket RESOLVED
> sotto conservano la dicitura originale come record):
> - Tool crate `nexum-tools` / `nxm-tools` (repo `nxm-ai/nxm-tools`) →
>   **`nxm-tui-tools`**, ora **workspace member dentro `nxm-tui`** (path locale),
>   importato nel codice come **`nxm_tools`** (package rename). Non è più una
>   git-dependency esterna.
> - Repo `nxm-ai/nxm-tools` → rinominato **`nxm-ai/nxm-mcp-servers`**; il tool
>   crate della TUI **non** vive più lì (è dentro `nxm-tui`). `nxm-mcp-servers`
>   contiene solo server MCP.
> - `nxm-tui-docs` (questo tracker) vive ora sotto `nxm-private/`.

# 🗺️ Wayfinder Map — Pure-Rust TUI harness for Nexum

> **Tracker:** local-markdown (`.wayfinder/`), default per the Wayfinder skill.
> `RULES.md` dice che gli artefatti di pianificazione vivono in
> `nxm-ai/nxm-projects`; `nxm-harness` invece tiene solo RULES/CONTEXT/STATUS/
> CHANGELOG. Perché questa sessione è sandboxata a `nxm-harness` come radice di
> lavoro, mappa + ticket sono qui in markdown (fallback del tracker). Migrare
> in `nxm-projects/plans/active/` (o in GitHub Issues) una volta che il sandbox
> può scrivere lì.
> **Scope lean (per `pi.dev`):** qui dentro `B` = *bene* (snello). Il TUI non
> contiene compressione/quant/vector: quelli sono del `nexum-agentai` backend.

## Destination

A **lightweight, `pi.dev`-style pure-Rust TUI** for the Nexum inference
engines — model‑picker, live token streaming, inline tool results, no bloat.

Concretamente: **streammi un modello OpenAPI, aggiungo tool, tutto qui.**
L'AgentAi (backend) si occupa della compressione, della quantizzazione
(TurboQuant) e del retrieval vettoriale; **questo crate (`nxm-tui`) resta il
frontend snello** (ratatui/crossterm/tokio + `nxm-tui-tools` come tool
predefiniti pluggabili). Nessun TS/WASM, nessun motore embedded.

> `pi.dev` reference: una finestra, un modello, streammizio, tool in linea,
> niente pannelli da 50 righe. Questo è lo spirito.

## Scope split (importante)

| Strato | Repo/crate | Cosa contiene |
|---|---|---|
| **Frontend** | `nxm-tui` (questo lavoro) | ratatui TUI: stream OpenAI, REPL agentico, tool round-trip (`role:"tool"`), tool pluggabili (`nxm-tui-tools`, workspace member, import `nxm_tools`). |
| **Backend** | `nexum-`* (agentai) | `engines.serve-*`, `kv-cache.turboquant`, RAM/GPU fit, `memexd`/`qdrant`, MCP vendor retrieval. |

## Notes

- **Domain context (MCP `list` + direct reads):** il progetto `nexum`
  (id `15c8ae702621`, indicizzato da workspace-qdrant/memexd) contiene
  `engines.*` (metal, ssd, tokenizer, serve-*, serve-turboquant…),
  `nxm-operations.*`, un componente **`tui`** (`basePath:"tui"`, `cargo`),
  `web/srv`, `xtask`.
- **Ancro in-house:** `nxm-tui` (repo sibling) è già un client pure-Rust
  (`ratatui 0.29`+`crossterm 0.28`+`tokio`+`reqwest`, "connects to any
  OpenAI-compatible LLM server") — punto di riuso confermato da D3.
- **RULES.md (vincola ogni ticket):** ≤300 righe/file, nessun `#[cfg(test)]`
  inline, nessun `unwrap`/`expect` in prod, `anyhow`/`thiserror`, logging su
  `tracing` (`tracing targets`), doc bilingue (`_ENG`/`_ITA`), codice in
  inglese.
- **MCP in-sandbox:** il server `workspace-qdrant-mcp` è presente e `list`
  funziona, ma il backend vettoriale (qdrant :6333/:6334) **non è servito** in
  questa sessione → `grep`/`search`/`retrieve` dei vendor tornano 0.
  Interrogazione vendor fatta per **direct read** dei cloni locali (affidabile).
  Vedasi `research/vendor-interrogation.md` (+ `T1`, ora deferito ad AgentAi).
- **Efficienza (R4, 2026-09-20):** audit completo in `research/efficiency-audit.md`
  — causa principale: draw incondizionato a ~60 Hz (`event.rs:13`) + ricostruzione
  history/markdown O(N) ogni frame; fix top: draw-on-change + poll 33 ms idle,
  cache rendering, memoizzare `estimated_tokens`. Bloat: 5 file > 300 righe
  (RULES.md), 3 `unwrap/expect` in prod. Alimenta la prossima frontiera UX/perf.
- **Engines to drive:** `engines.serve-cpu`/`serve-metal`/`serve-mlx-c`/
  `serve-ssd` — il TUI ne streamma uno (via OpenAI-HTTP), non li implementa.

## Decisions so far

- **Stack:** ratatui + crossterm (+ tokio + clap + reqwest) — confermato da 4
  vendor Rust + l'in-house `nxm-tui` (R2). Nessun crate TUI Rust alternativo.
- **grok-build = Rust** (Zed-lineage `xai-grok-*`, `xai-grok-pager-*` render) —
  referenza Rust-nativa, non TS (R1).
- **kimi-code/gemini-cli = TS** (ink/React) — fuori come implementazione
  (vincolo: Rust puro), comportamento di riferimento (R3).
- **D1 (purpose):** *lean agentic REPL* per motori Nexum — stream + tool +
  workspace, stile `pi.dev`.
- **D2 (architecture):** A) binario Rust standalone (non `harness-core` in-process).
- **D3:** evolvi `nxm-tui` in place (multi-pane esistente + `tool_types.rs`).
- **D6:** tool in un crate solo → **`nxm-tui-tools`** (read_file/list_resources/
  web_search); **workspace member dentro `nxm-tui`** (import `nxm_tools`), già
  creato, 4 test verdi. È il set di default pluggabile. *(Storicamente deciso
  come `nxm-ai/nxm-tools`; spostato in-tree nella riorganizzazione 2026-09-13.)*

## Not ancora specificato (frontend)

- Layout multi-pannello / dashboard (grok-build `pager-render`) vs. singolo
  buffer agentico — `pi.dev` preferisce uno stile (da fissare in D1 follow-up).
- Paginazione / rendering virtualizzato per catalogo modelli e file memoria.
- Piani di compliance bilingue + RULES.md per il nuovo crate.

## Out of scope (di nxm-tui — vanno nell'AgentAi backend)

- **TurboQuant / RAM-GPU fit (→ D5, deferito ad `nexum-agentai`):** il `fit`
  e la compressione sono backend, non frontend. Si espone via API, non nel TUI.
- **Motore vettoriale embeddato / MCP vendor (→ T1, deferito ad AgentAi):**
  qdrant/memexd retrieval resta backend; il TUI lo consuma via HTTP.
- GPUI / GPU rendering (ruled out → Tauri; qui TUI leggera).
- TS / React / ink / WASM come implementazione (vincolo Rust puro); i CLI TS
  (kimi-code, gemini-cli, opencode, kiro-cli, cursor-agent) sono solo referenza.
- GUI Tauri (`src-tauri`+`frontend/`) — sforzo separato.
- Motore di ragionamento agentico (piano/act) — è strato inference, non TUI.

## Frontier (open · unblocked)

> **MVP frontend BLOCCATO.** Dopo P2+D4, il frontend lean (`pi.dev` style) è
> completo: stream + tool round-trip + cancellazione. La prossima frontiera è
> UX polish (layout) — i big-ticket hardware/vector sono spostati nell'AgentAi.

| Ticket | Type | Note |
|---|---|---|
| [T4-micro-perf.md](tickets/T4-micro-perf.md) | task | ✅ DONE: estimated_tokens memoizzato + cache cwd (resto di R4). Prossima frontiera: D7 (key resolution) o UX polish. |

## Done this session — P2 (proprio) · D4

P2-proprio su `nxm-ai/nxm-tui` `wayfinder/p2-proprio` (tip `94b15b0`, PR #1):
l'agentic REPL gira nel loop ratatui:

- `main.rs`: channel `ToolPart`; submit prompt spawna `Agent::run(&tx)`
  (rimpiazza `chat_stream`); la tick-loop svuota `ToolPart::{Text,
  Reasoning,ToolInvocation,ToolResult,Error}` → `app.messages`.
- `agent.rs`: owns transcript, dispatch tool via `nexum_tools::Registry::call`,
  serializza `role:"tool"` + `tool_calls`; `no &mut App` (safe to `tokio::spawn`).
  3 delta-parser test verdi.
- `app.rs`: `Role::Tool` + `Message::tool(call_id,output)` + `tool_call_id`;
  `apply_token` estratto e condiviso.
- D4: strategy **Option A** (mpsc → drain 16 ms/tick → ~60 Hz frame), UI mai
  bloccata; `inference_task.abort()` su Quit (`main.rs`) = cancel-stream.
- Dipende dal tool crate (storicamente `nxm-ai/nxm-tools` v0.1; ora `nxm-tui-tools`, workspace member di `nxm-tui`).
- Verifica: `cargo check` + `cargo test` 6/6 verdi; `clippy --all-targets` 0 error.

End-to-end (≥1 round-trip tool) serve un motore live (l'AgentAi) — loop,
dispatch, serializzazione sono compilati e unit-testati.

### P3 (connectable) · wayfinder/p2-proprio · PR #1 (@ `e9413e9`)

- `api_key` da env (`NEXUM_API_KEY`, fallback `OPENAI_API_KEY`) in `config.rs`,
  `#[serde(skip)]` → non viene mai scritto su disco (`~/.nexum/tui.toml` ha solo
  `endpoint` + `model_name`).
- `agent.rs` aggiunge `Authorization: Bearer <key>` alla POST `/v1/chat/completions`
  + parsing `reasoning_content` (NVIDIA-Thinking/QwQ/o1) → `ToolPart::Reasoning`.
- `main.rs` passa `cfg.api_key` all'Agent.
- `tests/roundtrip.rs` (nuovo): include via `#[path]` i moduli reali (`agent`,
  `app`, `tool_types`, `session`, `server_proc`, `prompt`, `prompt_lines`) e
  gira l'Agent contro un mock OpenAI-SSE; verifica **reale** round-trip con
  `read_file` → `role:"tool"` + header `authorization: bearer <key>`, headlessly
  (nessuna key/TTY). 6 (bin) + 4 (roundtrip) = 10 test verdi, 0 clippy error.
- Usa con un provider reale:
  - **Ollama** — nessuna key (`endpoint=http://localhost:11434/v1`, model=`qwen2.5`);
    bearer è omesso quando `api_key` è `None`.
  - **NVIDIA** — `tui.toml` con `endpoint="https://integrate.api.nvidia.com/v1"`
    + `model_name="nvidia/nemotron-3.5-lightning-30b-a3b"` e env `NEXUM_API_KEY=<key>`.
