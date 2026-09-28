# Research — keyring crate evaluation (R1, 2026-09-09)

Subagent AFK su fonti primarie (docs.rs, repo ufficiale). Verdetto: **adatta**.

## Verdetto

Usare `keyring = "4"` con feature default (`v1`); una `Entry` per provider
(`service = "nexum-tui"`, `user = <provider name>`). Compatibile con lo stato
attuale: `config.rs` tiene già `api_key` fuori da `tui.toml` (`#[serde(skip)]`).

## API e indicizzazione

- `Entry::new(service, username) -> Result<Self>`; `set_password(&str)`,
  `get_password() -> Result<String>`, `set_secret(&[u8])` / `get_secret()`,
  `delete_credential()` (v4; v1–v3 la chiamava `delete_password`). `Entry` è
  `Send + Sync` ma le op sono **bloccanti** → `tokio::task::spawn_blocking`.
- Entry identificata da coppia `(service, user)` UTF-8 + opzionale `target`.
- `NoEntry` su `get_password` = "key non salvata"; `Ambiguous` = duplicati.

## Backend per OS e headless

- `v1` auto-seleziona uno store per piattaforma: macOS → Keychain Services,
  Windows → Credential Manager, Linux → Secret Service (login collection).
- NON abilitare feature `cli` (trascina tutti gli store).
- Senza keychain (headless/SSH, no D-Bus, collection lockata):
  `Entry::new` → `Err(Error::NoDefaultStore)`; a runtime `PlatformFailure(..)`
  o `NoStorageAccess(..)`. Probe senza creare entry:
  `Entry::store_status()`. Gestione TUI: fallback a env (`NEXUM_API_KEY`,
  `api_key_env`) + prompt.
- Workaround Linux headless documentato: `gnome-keyring-daemon --unlock`;
  WSL senza default collection non funziona senza `target`/collection.

## Versione, MSRV, peso, conflitti

- Pin **`keyring = "4"`** (stabile 4.2.0; `keyring-core 1.0.0`). `Error` è
  `#[non_exhaustive]` → sempre ramo wildcard.
- MSRV: toolchain ≥ 1.88 (edition 2024).
- Peso: diretto minimo; transitivi per piattaforma (Linux: `zbus` +
  `secret-service`; macOS: `security-framework`; Windows: `windows-sys`).
  Nessun overlap con tokio/crossterm/ratatui.
- Concorrenti sullo stesso entry inaffidabili su alcuni backend (Windows):
  serializzare gli accessi keychain.

## Test senza keychain reale

- `keyring_core::mock::Store` in-memory: `set_default_store(mock)` a inizio
  test **prima** di ogni `v1::Entry::new`; `Cred::set_error(...)` per
  iniettare fallimenti una tantum. Pattern consigliato: trait `KeyStore`
  (get/set/delete) + impl reale e mock.
- Alternativa test-only: feature `sample` (file-backed, esplicitamente non sicura).

## Alternativa (solo se keyring inadatta)

- `db-keystore` 0.5: SQLite cifrato sugli stessi trait `keyring_core`,
  headless-safe, ma richiede chiave di cifrazione fornita dall'utente + SQLite.

Fonti: docs.rs/keyring, docs.rs/keyring-core,
github.com/open-source-cooperative/keyring-rs (src/v1.rs, Cargo.toml, wiki),
docs.rs per apple/windows/zbus store, docs.rs/db-keystore. Report integrale
del subagent agli atti di sessione.
