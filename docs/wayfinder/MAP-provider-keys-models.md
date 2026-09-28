# Map — Provider keys in OS keychain + model management

- **Label:** `wayfinder:map`
- **Status:** Open · charted 2026-09-09

## Destination

Uno **spec implementabile** per la gestione provider in `nxm-tui`: API key
per-provider salvate nel keychain di sistema, risoluzione
keychain → `api_key_env` → env globale, inserimento via prompt nascosto nella
TUI, lista/selezione modelli da `GET /v1/models` con default per provider.
La mappa è finita quando non resta nulla da decidere prima di implementare.

## Notes

- Dominio: `nxm-tui` (ratatui + crossterm, Rust puro). Skills per sessione:
  `grilling`, `domain-modeling`, `research`, `prototype`.
- Stato attuale (verificato nel codice): `Provider { name, base_url,
  requires_api_key, api_key_env, is_cloud }` + preset/custom in
  `~/.nexum/tui.toml`; `TuiConfig.api_key` è `serde(skip)` (mai su disco),
  riempita da `NEXUM_API_KEY`/`OPENAI_API_KEY`; `model_name` è stringa libera,
  nessun elenco modelli (`/v1/models` usato solo come health probe).
- Sicurezza non negoziabile: key mai in argv, mai in shell history, mai su
  disco in chiaro, mai nei log. Vale anche per i test (mock only).
- Tracker: local-markdown (`tickets/`, `research/`). Claim: registrarsi in
  `tickets/index.md` Frontier prima di lavorare. Max 1 ticket non-research
  per sessione.

## Decisions so far

- [Destination: spec da implementare](MAP-provider-keys-models.md) — la mappa
  produce decisioni + ticket fino allo spec, non esegue.
- [Storage: OS keychain](MAP-provider-keys-models.md) — una entry per
  provider via crate `keyring`; niente secret su disco (R1 valuta la crate).
- [Precedenza: keychain poi env](MAP-provider-keys-models.md) — keychain
  per-provider → `api_key_env` → env globale; retrocompatibile e scriptabile.
- [Scope modelli: lista e selezione](MAP-provider-keys-models.md) — elenco da
  `GET /v1/models`, selezione + default per provider; niente pull/delete.
- [Inserimento: prompt nascosto](MAP-provider-keys-models.md) — input
  mascherato nella TUI; la key non tocca mai argv/history/disco in chiaro.
- [R1 keyring adatta: `keyring = "4"`](research/keyring-crate.md) — service
  `nexum-tui` / user `<provider>`; op bloccanti via `spawn_blocking`;
  headless via `store_status()` → fallback env; test con mock store.
- [R2 parser a due envelope](research/models-api-shapes.md) — `data[]` +
  `models[]`; id→name→model; auth solo NVIDIA (e LM Studio se abilitata).
- **D7 (grilling HITL, 2026-09-20 — dettagli nel ticket):**
  - **Risoluzione**: `keys.rs::resolve_key(provider) -> KeyResolution`, ordine
    keychain → `api_key_env` → `NEXUM_API_KEY` → `OPENAI_API_KEY` → Missing;
    una volta allo spawn dell'Agent, keychain dietro `spawn_blocking`.
  - **`default_model` dentro `Provider`** (`#[serde(default)]`) — migrazione
    zero-rottura; preset senza default (lo setta la TUI, T2); `model_name`
    resta il modello corrente.
  - **`api_key` globale rimossa** — una sola via di risoluzione; env-fill in
    `load()` eliminato; `handler.rs:518` consulta `resolve_key`.
  - **Headless a+b+c**: resolve silenzioso + warning una tantum se keychain
    assente + guided error → `/provider set-key` quando Missing (scelta
    utente: gli utenti vanno guidati a ogni livello).
  - **Mai-log rigido**: valore key MAI stampato (nemmeno mascherato, anche
    nei log/test); passano solo source/presenza/nome env var; test solo mock.

## Not yet specified

- Cache lista modelli / comportamento offline (ultima lista nota?).
- Validazione modello alla selezione vs all'invio (fail fast o lazy?).

*(Risolti da D7: fallback keychain non disponibile → a+b+c; migrazione
`tui.toml` → serde default zero-rottura; chi visualizza cosa → mai-log rigido
nel ticket D7.)*

## Out of scope

- Pull/delete modelli (es. Ollama) — deciso: solo lista e selezione.
- File cifrato con passphrase — alternativa scartata a favore del keychain.
- Key in chiaro su disco (`tui.toml` o altrove) — esplicitamente vietato.
- Solo-env senza salvataggio — superato dalla decisione keychain.
- Login OAuth/browser flow per provider — nessun preset lo richiede oggi.
