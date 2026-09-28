# D7 — Key resolution order + config model

- **Label:** `wayfinder:grilling`
- **Type:** Grilling (HITL) — needs the human
- **Status:** ✅ Resolved 2026-09-20 (grilling HITL)
- **Blocked by:** [R1-keyring-crate.md](R1-keyring-crate.md) (chiarita)
- **Map:** [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Question

Forma esatta del modello dati dopo la decisione "keychain poi env":

1. Dove vive la risoluzione — funzione pura `resolve_key(provider) -> Option<String>`
   con ordine keychain → `api_key_env` → `NEXUM_API_KEY`/`OPENAI_API_KEY`?
2. `TuiConfig`: nuovo campo `default_model` per provider (dentro `Provider`?
   mappa separata nome→modello?) + migrazione `tui.toml` esistenti.
3. Che fine fa il singolo `api_key` globale (resta come override esplicito?
   deprecato?).
4. Fallback quando il keychain non è disponibile (fog della mappa): solo env
   con warning, o errore guidato verso `/provider set-key`?
5. Mai-log/mai-stampa: regola su cosa può apparire in overlay, log, test.

## Resolution (closed) — 5 decisioni (grilling HITL 2026-09-20)

1. **Risoluzione**: nuovo modulo `keys.rs`, funzione pura
   `resolve_key(provider) -> KeyResolution`; ordine **keychain**
   (`nexum-tui`/`<provider>`) → **`provider.api_key_env`** →
   **`NEXUM_API_KEY`** → **`OPENAI_API_KEY`** → Missing. Chiamata una volta
   allo spawn dell'Agent (mai nel render loop), lettura keychain dietro
   `spawn_blocking`. Tensione accettata dall'utente: il keychain vince anche
   su un env esportato deliberatamente.
2. **`default_model` dentro `Provider`** (opzione a): `#[serde(default)]
   pub default_model: Option<String>` nell'array `providers` di `tui.toml`.
   Migrazione zero-rottura (serde default → `None` per le entry esistenti);
   preset senza default (lo setta la TUI alla scelta — comportamento T2);
   `TuiConfig.model_name` resta il modello corrente (`/provider use X` →
   `model_name = X.default_model` se presente, altrimenti invariato).
3. **`api_key` globale rimossa** (opzione a): `resolve_key` assorbe il
   fallback env globale; env-fill in `TuiConfig::load()` eliminato;
   `handler.rs:518` consulta `resolve_key` invece di `cfg.api_key`;
   `Agent::new(..., api_key: Option<String>)` invariato (cambia solo il
   valore passato — risolto per-provider allo spawn).
4. **Fallback headless: a+b+c** (scelta utente: "gli utenti sono poco consci
   di cosa fare"): `KeyResolution::{Keychain(String), Env { key,
   keychain_available }, Missing}` — resolve silenzioso (fallback interno,
   trasparente); **warning una tantum** nello status se keychain assente e
   key da env; **guided error** → `/provider set-key` quando Missing.
5. **Mai-log/mai-stampa — rigido**: il valore della key non viene MAI
   stampato da nessuna parte (overlay, log anche debug, output dei test —
   nemmeno mascherato: i log si copiano/incolano nei documenti wayfinder).
   Passano solo i metadati: source (`"keychain"`, `"env:NVIDIA_API_KEY"`),
   presenza/assenza, nome della env var nel warning. Test: solo key mock
   (es. `"test-key-123"`); key reali mai nei test. `keys.rs` logga
   source/presenza con `tracing` (regola RULES.md sulle funzioni pubbliche).

Dettagli crate in [research/keyring-crate.md](../research/keyring-crate.md);
spec implementabile aggiornato in [MAP-provider-keys-models.md](../MAP-provider-keys-models.md).
