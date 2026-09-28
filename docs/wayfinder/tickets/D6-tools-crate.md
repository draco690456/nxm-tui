# D6 — Tool crate: separate `nexum-tools` (MVP set)

> **Nota post-riorganizzazione (2026-09-13):** il testo sotto è il record storico
> della decisione. Assetto attuale: il crate è **`nxm-tui-tools`**, non più un
> repo separato `nxm-ai/nxm-tools` ma un **workspace member dentro `nxm-tui`**,
> importato come **`nxm_tools`** (package rename). L'esito della decisione (un
> crate unico di tool pluggabili) resta valido; è cambiata solo la collocazione.

- **Label:** `wayfinder:grilling`
- **Type:** Grilling (HITL) — quick decision (this is a design scoping ticket,
  sub-decision of P2)
- **Status:** ✅ Resolved · Decided this session
- **Blocked by:** D3 (crate location) — resolved.

## Question

Which tools live in the shared `nexum-tools` crate, and where does that crate
live? (The user wants "un crate singolo da integrare" for actuators like
`web_search`, rather than tool logic inlined in `nxm-tui`.)

## Resolution

- **Chosen:** a **separate** pure-Rust crate **`nexum-tools`** (repo
  `nxm-ai/nxm-tools`), depended-on by `nxm-tui`. `nxm-tui` = TUI shell;
  `nexm-tools` = agent actuators.
- **MVP tool set (v0.1):** `read_file`, `list_resources`, `web_search`.
  - `read_file`/`list_resources`: local filesystem (real, tested).
  - `web_search`: HTTP call to a configurable endpoint
    (`NEXUM_SEARCH_ENDPOINT`, default `http://localhost:8080/search`) — the
    real engine behind it is the server's responsibility, not the TUI's.
- **Interface:** `Tool` trait (`name`/`description`/`schema`/`run`) +
  `Registry` (`register`/`get`/`call`/`manifest`). `Agent` (nxm-tui) calls
  `registry.call(name, args)` instead of the P2-spike stub.
- **v0.1 note:** `Tool::run` is synchronous (blocking I/O) — a scaffold; P2-proper
  makes it `async` + integrates the `Registry` into `agent.rs`.
- **Source:** user grilling (D6), confirmed: "ok sui tool / nome nxm-tools".

## Acceptance (P2-proper)
- `Agent::run` dispatches tool calls through `nexum-tools::Registry`.
- `web_search` is wired to a real engine endpoint (not a stub).
- Adding a tool = implementing `Tool` + `registry.register(...)`; nxm-tui core
  is untouched.
