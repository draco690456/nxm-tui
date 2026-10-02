# FREE-AGENT PROMPT — T2: provider/model commands (nxm-tui)

> Consegna questo prompt a un agente esecutore. Il revisore (altra sessione)
> verificherà contro il codice reale. Lavora con precisione e verifica tutto;
> NON fare commit git (li fa il revisore).

## Repo & contesto

- Repo: `/Users/devdaniele/Projects/private/nxm-tui` (Rust, bin+lib via `src/lib.rs`).
- È una TUI ratatui 0.29 + crossterm + tokio + reqwest, client OpenAI-compatibile.
- Ticket: `docs/wayfinder/tickets/T2-provider-model-commands.md`.
- Research da rispettare: `docs/wayfinder/research/models-api-shapes.md` (R2).

## Obiettivo (T2)

Implementare i comandi provider/model mancanti. Lo stato ATTUALE verificato:
- `Provider.default_model: Option<String>` ESISTE già (`src/provider.rs`).
- `GET /v1/models` esiste SOLO come health-probe in `detect_endpoint` (`src/main.rs`),
  NON lista i modelli.
- Comandi `/provider list|add|use|remove|set-key` esistono (`ProviderCommand` in
  `src/app.rs:212`, dispatch `handle_provider_command` in `src/handler.rs:537`).

Da AGGIUNGERE:
1. **`/models`** — lista i modelli dal provider attivo (`GET {base}/v1/models`),
   mostrati con un indice numerico; stato pending/errore nello status.
2. **`/model use <id|numero>`** — seleziona il modello: imposta
   `cfg.model_name` e `Provider.default_model` del provider attivo, salva config.
3. **`/provider remove-key <name>`** — elimina la key dal keychain
   (`keyring::Entry::new("nexum-tui", &name).delete_credential()`), con guided
   error se il keychain non è disponibile. (NB: in dev env-first il keychain può
   non essere usato — vedi `keys.rs`; `remove-key` opera comunque sul keychain.)
4. **Modello attivo visibile**: mostra `cfg.model_name` nella bottom bar
   (`src/bottom.rs`) o mode bar (`src/mode_bar.rs`).
5. **Help** aggiornato (`Command::Help`) con i nuovi comandi + regola mai-stampare-key.

## VINCOLO ARCHITETTURALE CRITICO (async vs sync)

`handle_provider_command` è SINCRONO, ma `GET /v1/models` è ASYNC. NON bloccare
l'UI e NON usare `block_on` dentro l'handler. Segui lo STESSO pattern di
`ProviderCommand::SetKey`:
- Aggiungi uno **stato pending** in `App` (es. `models_fetch_pending: Option<String>`
  con il base_url, e `models_list: Vec<ModelEntry>` per il risultato).
- `handle_provider_command`/`parse_command` imposta solo lo stato pending.
- Il **loop async in `src/main.rs`** (dove già si fa `spawn_blocking(resolve_key)`
  e si drena `part_rx`) esegue il fetch async, poi riempie `app.models_list` e
  aggiorna lo status. Guarda come `SetKey` → `set_key_pending` è consumato e
  replica quel flusso per i modelli.
- In alternativa, un canale mpsc dedicato (come `part_tx`) se più pulito.
Documenta nel codice la scelta.

## PARSER MODELLI — regole R2 (rispettare alla lettera)

Metti il parsing in un **NUOVO modulo** `src/models.rs` (NON ingrossare `app.rs`
933 righe o `handler.rs` 612 — sono già oltre il cap RULES.md di 300). Esporta
`pub mod models;` in `src/lib.rs` (ordine alfabetico).

1. **Due envelope**: accetta sia OpenAI `{data:[...]}` sia Ollama nativo
   `{models:[...]}`. MAI richiedere `data`.
2. **ID**: preferisci `id`, fallback `name`, poi `model`; salta entry vuote.
3. Scalari opzionali con default (`owned_by="unknown"`, `created=0`); `created`
   può essere int O stringa ISO (gestisci entrambi senza fallire).
4. Ignora chiavi ignote, mai fallire su di esse.
5. Nessuna paginazione; applica comunque un cap di display (es. 100).
6. Errori: refused/timeout → "server down"; 401 → azionabile (NVIDIA / LM Studio
   con auth) suggerendo `/provider set-key`; 404 → prova path alterno
   (`{base}/models`, poi Ollama `{base}/api/tags`) prima di riportare errore.

Definisci un tipo tipo `pub struct ModelEntry { pub id: String, pub owned_by: String, pub created: i64 }`
e una `pub fn parse_models(json: &serde_json::Value) -> Vec<ModelEntry>` PURA e testabile.

## VINCOLI RULES.md (il revisore li verifica)

- **MAI `unwrap()`/`expect()` in produzione.** Usa `anyhow::Result` / match.
- **Test SOLO in `tests/`** (no `#[cfg(test)]` inline). Crea `tests/models.rs`
  che importa `use nxm_tui::models::...`. Copri: parser due-envelope (OpenAI +
  Ollama), fallback id→name→model, created int vs stringa ISO, entry vuote
  saltate, chiavi ignote ignorate, lista vuota. Per i comandi: parsing di
  `/models`, `/model use <n>` e `<id>`, `/provider remove-key <name>` nel
  `parse_command` (test sincroni, nessuna rete).
- **Versioni pinnate** se aggiungi dep (preferisci nessuna nuova dep: `reqwest`
  e `serde_json` ci sono già).
- **Logging** `tracing` con target `nexum::models` sulle funzioni pubbliche;
  MAI loggare la key.
- **Doc `///` con esempio** sulle API pubbliche di `models.rs`.
- **File ≤300 righe**: `models.rs` nuovo deve stare sotto; non peggiorare
  app.rs/handler.rs oltre lo stretto necessario (poche righe di wiring).
- Codice/commenti in INGLESE.

## PASSI SUGGERITI

1. `src/models.rs`: `ModelEntry` + `parse_models` (puro) + eventuale
   `async fn fetch_models(client, base_url, api_key) -> anyhow::Result<Vec<ModelEntry>>`
   con fallback path (R2 regola 6). `pub mod models;` in `lib.rs`.
2. `src/app.rs`: aggiungi varianti `Command::Models` e `Command::ModelUse(String)`
   + `ProviderCommand::RemoveKey(String)`; estendi `parse_command` (`/models`,
   `/model use <x>`, `/provider remove-key <name>`). Aggiungi lo stato pending
   per il fetch (vedi vincolo async). Aggiorna il testo di `Command::Help`.
3. `src/handler.rs`: dispatch dei nuovi comandi (imposta pending / applica
   selezione / remove-key via keyring `delete_credential`).
4. `src/main.rs`: nel loop async, consuma il pending fetch → `fetch_models` →
   popola `app.models_list` + status; renderizza la lista (riusa l'history o uno
   status multilinea — scelta tua, documentala).
5. `src/bottom.rs` o `src/mode_bar.rs`: mostra `cfg.model_name` attivo.
6. `/model use <numero>` mappa l'indice sulla lista corrente; `<id>` match
   diretto; errore se fuori range / id assente.

## VERIFICA (esegui e riporta l'output ESATTO)

- `cargo test --test models` (riporta conteggio).
- `cargo test` intero (riporta totale; se ci sono fail PREESISTENTI non tuoi,
  dimostralo con `git stash` sulla baseline). Esegui anche in PARALLELO.
- `cargo clippy --all-targets -- -D warnings` → 0 sui file che tocchi
  (i warning preesistenti in app.rs/keys.rs non contano, ma dimostralo).
- `cargo build` ok.
- Smoke manuale descritto a parole (non serve server live): cosa fa `/models`
  con server down, con envelope OpenAI, con envelope Ollama.

## OUTPUT FINALE (per il revisore)

NON committare. Riporta:
1. File creati/modificati con righe aggiunte.
2. Output esatto di cargo test (parallelo e, se serve, `--test-threads=1`) + clippy.
3. Come hai risolto il vincolo async/sync (pending state o canale) e perché è
   corretto (UI mai bloccata).
4. Come il parser copre i due envelope + i fallback path.
5. Eventuali deviazioni dal ticket/R2 e perché.
6. Conferma esplicita: nessun `unwrap/expect` in prod nei file nuovi; nessuna
   key loggata; file nuovi ≤300 righe.
