# FREE-AGENT PROMPT — T2 ROUND 2 (fix dalla review)

> La review ha approvato la struttura (parser, pattern async, test 12/12,
> clippy pulito, 0 unwrap in prod) MA ha trovato 4 problemi da correggere.
> Lavora SOLO su questi punti. NON rifare ciò che funziona. NON commit git.

## Contesto

Repo `/Users/devdaniele/Projects/private/nxm-tui`. Il lavoro T2 è già in working
tree (non committato): `src/models.rs`, `tests/models.rs`, e modifiche a
`app.rs`, `handler.rs`, `main.rs`, `mode_bar.rs`, `overlays.rs`,
`autocomplete.rs`, `lib.rs`. Build e test passano. Correggi i difetti sotto.

## 🔴 FIX 1 — `app.model_name` non sincronizzato all'avvio (bug di coerenza)

**Problema:** hai introdotto `app.model_name` accanto a `cfg.model_name`
(esistente). All'avvio `app.model_name` resta `None`, quindi la mode bar mostra
`model: —` anche quando la config ha già un modello. Due fonti di verità non
sincronizzate.

**Fix:** in `src/main.rs`, quando l'endpoint diventa `Running` (sia nel ramo
`Some((url, name))` iniziale, sia nel ramo del menu NoServer
`Connecting→Running`), popola `app.model_name = cfg.model_name.clone();`.
Così la mode bar riflette subito il modello di config. Verifica che dopo
`/model use` entrambi restino allineati (già fai `cfg.save()` + `app.model_name =`).

> In alternativa più pulita (se preferisci UNA fonte di verità): elimina
> `app.model_name` e fa' leggere alla mode bar `cfg.model_name`. MA la mode bar
> non ha `cfg` in scope → serve passare il valore. Se è troppo invasivo, resta
> sulla sincronizzazione sopra e documentala con un commento.

## 🔴 FIX 2 — fetch con doppio `/v1`

**Problema:** `fetch_models` prova `{base}/v1/models` per primo. Se `base_url`
finisce già in `/v1` (Ollama `http://127.0.0.1:11434/v1`, LM Studio, NVIDIA),
il primo tentativo diventa `.../v1/v1/models` → 404; funziona solo per il
fallback successivo, con un warning spurio.

**Fix:** in `src/models.rs::fetch_models`, normalizza prima di costruire i path.
Togli un eventuale suffisso `/v1` dal base, poi costruisci i candidati in modo
che non si duplichi mai:
```rust
let base = base_url.trim_end_matches('/');
let root = base.strip_suffix("/v1").unwrap_or(base); // bare host
let paths = [
    format!("{root}/v1/models"),   // OpenAI dialect
    format!("{root}/models"),       // bare
    format!("{root}/api/tags"),     // Ollama native
];
```
Aggiungi un test in `tests/models.rs` che verifica la costruzione dei path
(estrai la logica in una `pub fn models_candidate_paths(base_url: &str) -> Vec<String>`
pura e testala: input con e senza `/v1`, con trailing slash). Mantieni
`fetch_models` che usa quella funzione.

## 🟡 FIX 3 — `api_key: None` nel fetch (NVIDIA e cloud danno 401)

**Problema:** in `src/main.rs` il fetch passa sempre `None` come api_key →
per provider che richiedono key (NVIDIA) `/models` fallisce sempre con 401.

**Fix:** risolvi la key per-provider PRIMA di spawnare il task, come già fai per
l'Agent. Nel ramo che consuma `models_fetch_pending`:
- trova il provider attivo (match su `base_url` vs `app.endpoint`, come fa il
  blocco inference con `provider::all_providers` + `find`),
- se `requires_api_key`, risolvi con
  `tokio::task::spawn_blocking(move || resolve_key(&provider)).await` →
  `.into_key()` (riusa `nxm_tui::keys::resolve_key`, già importato),
- passa la key risolta a `fetch_models(&client, &base, api_key.as_deref())`.
MAI loggare il valore della key (regola mai-log).

## 🟡 FIX 4 — report incompleto (processo)

Nel prossimo report elenca TUTTI i file modificati (nel round 1 avevi omesso
`autocomplete.rs` e `overlays.rs`). Le modifiche erano corrette, ma il report
deve essere completo e verificabile.

## VINCOLI (invariati)

- No `unwrap()`/`expect()` in prod. `models.rs` ≤300 righe (ora 181; resta sotto).
- Test in `tests/` (no inline). Logging `tracing nexum::models`/`nexum::keys`,
  mai la key. Doc `///` con esempio sulle nuove funzioni pubbliche.
- Codice/commenti in inglese.

## VERIFICA (esegui e riporta output ESATTO)

- `cargo test --test models` (nuovo test path building incluso).
- `cargo test` intero IN PARALLELO (riporta totale; i soli fail ammessi sono i
  `keys` preesistenti già noti/ignored — dimostra che non ne aggiungi).
- `cargo clippy --all-targets -- -D warnings` → 0 sui file toccati.
- `cargo build` ok.

## OUTPUT FINALE (per il revisore)

NON committare. Riporta: (1) elenco COMPLETO dei file toccati in questo round;
(2) come hai risolto FIX 1/2/3 con lo snippet chiave; (3) output esatto di test
(parallelo) + clippy; (4) conferma: 0 unwrap/expect prod nei file toccati,
nessuna key loggata, nessun doppio `/v1`, mode bar mostra il modello di config
all'avvio.
