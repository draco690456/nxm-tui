# I2 — Wire KeyStore in produzione (runtime)

- **Label:** `wayfinder:task`
- **Type:** Implementation (deriva da I1)
- **Status:** ✅ Resolved 2026-10-02 (verificato + pushato; wiring runtime completo)
- **Related:** [I1-keystore-seam.md](I1-keystore-seam.md),
  [R5-os-independent-keystore.md](R5-os-independent-keystore.md),
  [D7-key-resolution-config.md](D7-key-resolution-config.md)

## Motivazione

I1 ha costruito il seam (`trait KeyStore` + `EnvStore`/`OsKeychainStore`/
`EncryptedFileStore`) e `resolve_key_with(provider, &dyn KeyStore)`, tutti
testati — ma la produzione usa ancora `resolve_key(provider)` legacy che
costruisce `keyring::Entry` direttamente. Il seam è quindi **cablato solo nei
test**: il backend cifrato portabile non è raggiungibile dal TUI.

## Scope (questo ticket)

- **Factory (TDD)**: `keystore::store_for(mode, passphrase) ->
  Result<Box<dyn KeyStore>, KeyStoreError>`. Mapping:
  `EnvFirst→EnvStore`, `KeychainFirst`/`OsKeychain→OsKeychainStore`,
  `EncryptedFile→EncryptedFileStore`. EncryptedFile senza passphrase =
  **errore pulito** (`KeyStoreError::Unavailable`), mai panic.
- **Passphrase 1x/sessione**: `app.passphrase: Option<Zeroizing<String>>` +
  input mascherato in `handler.rs` modellato su `set_key_pending`
  (`PassphraseEntry.buffer: Zeroizing<String>` — azzerato da tipo su Enter
  **ed** Esc, mai loggato — D7 mai-log). Esc = rifiuto dell'azione: nulla viene
  cachato, quindi una successiva azione gated può riproporre il prompt (di
  proposito — l'utente ha solo annullato quell'azione).
- **Read-path**: i 4 call site passano a `store_for` + `resolve_key_with`
  (via `keys::resolve_key_via`): `main.rs` models fetch, `main.rs` agent
  spawn, `mcp/mod.rs::connect_from_config`, `handler.rs` `/provider use`.
  `spawn_blocking`/`block_in_place` mantenuti. Se serve la passphrase e
  manca → **attiva il prompt, non spawnare**. Le azioni con stato pending
  (models fetch, connect MCP, send accodata) **riprendono alla Enter** e si
  **annullano alla Esc**; `/provider use` è un check diagnostico senza stato
  pending — dopo la Enter basta ripetere il comando. Il connect MCP è
  differito alla prima iterazione del loop per questo motivo.
- **Write-path**: `handler.rs` set-key (Enter) e `RemoveKey` usano
  `store.set`/`store.delete` (sempre via `store_for`) invece di
  `keyring::Entry` diretto.
- `keys.rs` tiene `resolve_key` legacy come wrapper finché non serve più
  (test `tests/keys.rs` lo coprono); **non rimosso** in questo ticket.

## Vincoli (regole repo)

- Zero `unwrap`/`expect` in prod; mai-log rigido (target `nexum::keys` /
  `nexum::keystore`): valore chiave **e** passphrase MAI loggati.
- Cap `RULES.md` 300 righe/file: vale per i file **nuovi/split** (come in I1:
  `keystore/*` ≤300). I file pre-esistenti già oltre il cap (`app.rs`,
  `handler.rs`, `main.rs`, `ui.rs`) restano il refactor RULES.md debt già
  mappato in index.md — non crescono per questo ticket più del necessario.
- Non toccare le modifiche non-staged in `docs/patterns/`.
- Push solo su `origin` (draco), mai su `public`.

## Done when

- 4 read-path su `resolve_key_via` (factory + `resolve_key_with`); write-path
  sul trait (`store.set`/`delete`); passphrase max 1x/sessione e mai loggata;
- `cargo build` pulito; `cargo test -- --test-threads=1` verde;
  `cargo clippy` = baseline 3 lint noti, **0 nuovi** (verifica con
  `git stash`); file nuovi ≤300 righe; I2 → Resolved in index.md; pushato.
