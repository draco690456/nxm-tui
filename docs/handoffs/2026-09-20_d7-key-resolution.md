# Handoff: efficienza chiusa (R4→T3→T4) + D7 deciso → prossima sessione: P3

> **Prossima sessione: implementare P3** (key entry + risoluzione per-provider
> via keychain) seguendo lo spec qui sotto — già cotto con le 5 decisioni D7.
> Poi T2 (provider/model commands). Tutto il contesto è nel wayfinder.

**Created**: 2026-09-20
**Source session**: pi (R4 audit + T3 + T4 + D7 grilling HITL)
**Target repo**: nxm-tui (private) — `main` @ `0fae988`, pushato su `draco`
**Priority**: high

---

## 1. Fatto questa sessione (riferimenti, non rifare)

- **R4** — `wayfinder/research/efficiency-audit.md`: audit efficienza. Causa
  lentezza: redraw ~60 Hz + ricostruzione history/markdown O(N) per frame.
- **T3 ✅** (`f158325`) — draw-on-change + poll idle 33 ms + `HistoryCache`
  per-message. Inoltre **target lib** (`src/lib.rs`): un grafo moduli, test
  su `nxm_tui::`, specchi `#[path]` eliminati (erano stantii).
- **T4 ✅** (`0fae988`) — `estimated_tokens` memoizzato (Cell + fingerprint
  O(1)), `current_dir_name` cached (OnceLock). **68/68 test verdi, clippy 0.**
- **D7 ✅** — 5 decisioni nel ticket
  [`D7-key-resolution-config.md`](../wayfinder/tickets/D7-key-resolution-config.md),
  memorizzate anche in nxm-memory (recall: "D7" / "key resolution").

## 2. PROSSIMA SESSIONE — P3 (claim nel tracker PRIMA di lavorare)

**Ticket**: `wayfinder/tickets/P3-key-entry-prompt.md` — sbloccato da D7.
**Regola wayfinder**: registrare il claim in `tickets/index.md` "In progress"
+ `MAP.md` Frontier; max 1 non-research ticket per sessione (P3 è l'unico).

### Vincoli D7 (hard, non negoziabili)

1. **Risoluzione**: nuovo `keys.rs` — `resolve_key(provider) -> KeyResolution`,
   ordine **keychain** (`nexum-tui`/`<provider>`) → **`provider.api_key_env`**
   → **`NEXUM_API_KEY`** → **`OPENAI_API_KEY`** → Missing. Chiamata UNA volta
   allo spawn dell'Agent (mai nel render loop), keychain dietro
   `spawn_blocking`. Il keychain vince anche su un env esportato
   deliberatamente (tensione accettata).
2. **`default_model` dentro `Provider`**: `#[serde(default)] pub
   default_model: Option<String>` — zero migrazione (serde default → None).
   Preset senza default. `TuiConfig.model_name` resta il modello corrente.
3. **`TuiConfig.api_key` globale RIMOSSA** — env-fill in `load()` eliminato;
   `handler.rs:518` consulta `resolve_key`; `Agent::new(..., api_key:
   Option<String>)` invariato (cambia solo il valore: risolto per-provider).
4. **Headless a+b+c** (scelta utente): resolve silenzioso + **warning una
   tantum** nello status se keychain assente e key da env + **guided error**
   → `/provider set-key` quando Missing.
5. **Mai-log RIGIDO**: il valore della key non viene MAI stampato (overlay,
   log anche debug, output test — nemmeno mascherato; i log si copiano nei
   wayfinder docs). Passano solo: source, presenza, nome env var. Test: solo
   key mock; key reali mai nei test.

### Passi (in ordine)

1. **Toolchain check**: `rustc --version` — keyring 4 richiede ≥ 1.88. Se
   inferiore, fermarsi e segnalarlo (non downgradare la crate in silenzio).
2. **Cargo.toml**: `keyring = "4.2.0"` (pin esatto, RULES.md). NON abilitare
   feature `cli`. Aggiungere `pub mod keys;` a `src/lib.rs`.
3. **`src/keys.rs` (nuovo)**:
   - `pub enum KeyResolution { Keychain(String), Env { key: String,
     keychain_available: bool }, Missing }` + `pub fn into_key(self) ->
     Option<String>` (consumato dall'Agent).
   - `pub fn resolve_key(provider: &Provider) -> KeyResolution`: keyring
     entry (`Entry::new("nexum-tui", &provider.name)`) → errori
     `NoDefaultStore`/`PlatformFailure`/`NoStorageAccess` = keychain non
     disponibile → env fallback (`provider.api_key_env`, poi `NEXUM_API_KEY`,
     poi `OPENAI_API_KEY`); `NoEntry` = nessuna entry → env fallback.
     `Error` è `#[non_exhaustive]` → SEMPRE ramo wildcard.
   - Logging `tracing` (target `nexum::keys`): source + presenza — MAI il valore.
4. **`src/provider.rs`**: campo `default_model` (punto 2 dei vincoli).
5. **`src/config.rs`**: rimuovere `api_key` + env-fill (punto 3).
6. **`src/main.rs`**: allo spawn dell'Agent — trovare il provider attivo
   (match per endpoint, come fa `server_name_from_url`),
   `tokio::task::spawn_blocking(|| resolve_key(&p))` → await → passare
   `.into_key()` all'Agent. Warning (b) una tantum: se `Env { keychain_
   available: false }` → `app.set_status("keychain non disponibile — key da
   <env var> (non persistente)")`.
7. **`src/app.rs` + `src/handler.rs` — `/provider set-key <nome>` (cuore P3)**:
   - Nuovo variant `ProviderCommand::SetKey(String)` in `parse_command`.
   - **Stato dedicato** in `App` (NON `prompt_state`): buffer segreto che
     viene **azzerato** dopo save/Esc — la key non resta in memoria oltre il
     salvataggio. (Es. `set_key_pending: Option<SetKeyEntry>` con provider +
     buffer.)
   - Render mascherato: un `•` per carattere, niente preview (overlay
     dedicato o riuso — è la domanda del prototipo: referenza
     `render_approval` in `src/tool_overlay.rs`, Y/N modale in `handler.rs`).
   - `Enter` → keyring `set_password` via `spawn_blocking` → status "key
     salvata per `<provider>`" (mai il valore); `Esc` → scarta + azzera.
   - Salvataggio fallito con `NoDefaultStore` → guided error: "il keychain
     non è disponibile su questo sistema — usa `NVIDIA_API_KEY` o
     `api_key_env`" (D7-4c).
   - `handler.rs:518`: check "key mancante" via `resolve_key` → Missing =
     suggerire `/provider set-key`.
8. **Test (mock only, in `tests/`)**:
   - `keyring_core::mock::Store` con `set_default_store(mock)` PRIMA di ogni
     `Entry::new` (pattern R1); `Cred::set_error(...)` per iniettare
     fallimenti una tantum. In alternativa trait `KeyStore` + impl
     real/mock.
   - Copertura minima: ordine risoluzione (keychain batte env — la tensione
     D7), fallback env su NoDefaultStore, Missing → guided, set-key salva
     nel mock + buffer azzerato dopo (nessuna key in memoria), roundtrip
     bearer con mock key (estende `tests/roundtrip.rs`).
   - Niente `#[cfg(test)]` inline (RULES.md); test in `tests/<modulo>.rs`.
9. **Verifica**: `cargo test` (tutti verdi) + `cargo clippy --all-targets`
   (0 error). Smoke opzionale: Ollama senza key continua a girare; NVIDIA
   con entry keychain.
10. **Chiusura**: commit Conventional (`feat: per-provider key resolution
    via keychain (P3)` — un cambio logico), **push su `draco`** (NON origin:
    403, credenziali sono `draco690456`), P3 ticket → resolved (o note di
    reazione se il prototipo va ritoccato), index + MAP + CHANGELOG +
    CONTEXT aggiornati, handoff per la sessione dopo (T2).

## 3. Dopo P3 — T2 (provider/model commands, già sbloccato da D7+R2)

- `GET /v1/models` con **parser a due envelope** (`data[]` + `models[]`,
  R2: `research/models-api-shapes.md` — id→name→model).
- Comando `/models`: elenco + selezione; alla scelta setta
  `Provider.default_model`; `/provider use X` ripristina
  `model_name = X.default_model` se presente.
- Aperti dalla mappa (decidere in sessione): cache lista modelli/offline,
  validazione modello alla selezione vs all'invio.

## 4. Note d'ambiente

- **Git**: credenziali locali = `draco690456` → push su `draco` funziona,
  `origin` (dangranaz, pubblico) → 403. Sync pubblico serve l'account
  dangranaz. `upstream` (nxm-ai) è 5 commit dietro (era pre-reorg).
- **Wayfinder sempre** (regola utente, in nxm-memory): ogni ricerca/decisione
  nel tracker (`nxm-tui-docs/wayfinder/`), mai file isolati.
- **RULES.md hard**: mai `unwrap`/`expect` in prod, ≤300 righe/file
  (`app.rs` ~900 — lo split è ticket futuro, non di P3), test in `tests/`,
  Conventional Commits, logging `tracing` con target per-modulo.
- **nxm-memory**: decisione D7 + regole (wayfinder, credenziali) già
  memorizzate — `memory_recall` con "D7", "wayfinder", "credenziali".
