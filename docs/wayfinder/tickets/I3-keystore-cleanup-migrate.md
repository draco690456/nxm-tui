# I3 — Keystore cleanup + `/provider migrate-keys`

- **Label:** `wayfinder:task`
- **Type:** Follow-up (deriva da I2)
- **Status:** ✅ Resolved 2026-10-15
- **Related:** [I1-keystore-seam.md](I1-keystore-seam.md),
  [I2-wire-keystore-runtime.md](I2-wire-keystore-runtime.md)

## Motivazione

Dopo I2 tutti i call site di produzione passano dal seam `KeyStore`
(`store_for(mode, passphrase)` + `resolve_key_with`). Restano due code
successori:

1. **`resolve_key` legacy** (`src/keys/resolve.rs:28`) non è più chiamato in
   produzione, **ma NON è morto**: lo usano ancora i test
   `tests/keys.rs` (righe ~84/107/128/151) per il path env-first/keychain-first
   process-global. Non va rimosso ciecamente.
2. **`/provider migrate-keys`** — import one-shot delle chiavi dal keychain
   nativo (`OsKeychainStore`) al file cifrato (`EncryptedFileStore`), tolto da
   I2 per rispettare il budget di 1 ticket/sessione.

## Scope

- **Decidere** il destino di `resolve_key`:
  - opzione A — riscrivere i test di `tests/keys.rs` sul seam
    (`resolve_key_with` + `MockKeyStore`/`EnvStore`) e **rimuovere** il wrapper
    legacy; oppure
  - opzione B — mantenere il wrapper come API retro-compatibile documentata e
    marcarlo esplicitamente come "test-only / legacy".
  - La scelta è una micro-decisione da grill veloce prima di implementare.
- **`/provider migrate-keys`**: per ogni provider con chiave in
  `OsKeychainStore.get`, scrivila in `EncryptedFileStore.set` (richiede la
  passphrase di sessione se `EncryptedFile`). Status: "N chiavi migrate".
  Nessun valore loggato (mai-log D7). Idempotente (ri-eseguire non duplica).

## Done when

- Destino di `resolve_key` risolto (rimosso con test migrati, **oppure**
  documentato come legacy/test-only) — nessun ambiguo "codice morto".
- `/provider migrate-keys` implementato + testato con fake store (source→dest),
  idempotente, mai-log rispettato.
- Build + clippy puliti (0 lint nuovi vs baseline); suite verde single-thread;
  file ≤300 righe; zero `unwrap`/`expect` in prod.

## As-built (2026-10-15)

**Scelta: opzione A** — `resolve_key` legacy **rimosso** e test migrati sul seam.

- `src/keys/resolve.rs`: eliminato il wrapper `resolve_key` (e l'uso diretto di
  `keyring::Entry`); restano `resolve_key_with(provider, &dyn KeyStore)` e
  `resolve_key_via`. `src/keys.rs` aggiorna doc + re-export (`resolve_key` tolto).
- `tests/keys.rs`: rimossi i test keychain-backed basati sul mock process-global
  `keyring_core` (origine della flakiness storica); i case env-first riscritti su
  `resolve_key_with(&p, &EnvStore::new())`. La logica "store beats env" vive
  deterministica in `tests/keystore.rs` (`store_beats_env_when_present`).
- `src/keystore/migrate.rs` (107 righe): `migrate_keys(source, dest, &[names])`
  backend-agnostico (`&dyn KeyStore`), idempotente (set overwrite), fail-soft
  per-provider, **mai-log D7** (solo nomi + conteggi, mai il valore).
  `MigrateReport { migrated, absent, errors }` + `status_line()` value-free.
- `src/commands.rs`: parsing `/provider migrate-keys` (alias `migrate`) →
  `ProviderCommand::MigrateKeys`.
- `src/handler.rs`: handler `MigrateKeys` gated sulla passphrase di sessione
  (prompt, non spawn cieco), filtra i provider `requires_api_key`, esegue in
  `block_in_place` (`OsKeychainStore` → `EncryptedFileStore`), status pulito.
- `tests/keystore_migrate.rs` (7 test puri): happy path, absent accounting,
  idempotenza, fail-soft read sorgente, fail-soft write destinazione, lista
  vuota no-op, `status_line`.

**Verifica:** `cargo build` ok; **178 test passati / 0 falliti / 1 ignored**
(doctest MCP preesistente) single-thread; clippy = 3 lint baseline (App/Metrics
Default, type_complexity), **0 nuovi**; file ≤300 righe.
