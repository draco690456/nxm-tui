# I1 — KeyStore seam + EncryptedFileStore

- **Label:** `wayfinder:task`
- **Type:** Implementation (deriva da R5)
- **Status:** ✅ Resolved 2026-10-02 (commit f2addf1; wiring runtime → I2)
- **Related:** [R5-os-independent-keystore.md](R5-os-independent-keystore.md),
  [D7-key-resolution-config.md](D7-key-resolution-config.md),
  spec: `nxm-docs/research/2026-10-02_os-independent-keystore-R5.md`

## Motivazione

R5 ha concluso: introdurre un **seam `KeyStore` iniettabile** (trait proprio)
risolve due problemi in un colpo —

1. disaccoppia `resolve_key` dal backend concreto (OS keychain) → **portabilità**;
2. rende la risoluzione **testabile** con un fake in-memory → rimuove lo
   `#[ignore]` su `keychain_beats_env` (debito di testabilità noto).

Default portabile = **file cifrato locale** AES-256-GCM + Argon2id + zeroize.

## Scope (questo ticket)

- `src/keystore/mod.rs`: `trait KeyStore` (`get`/`set`/`delete`/`is_available`)
  + `enum KeyStoreError` (thiserror). Tracing target `nexum::keystore`, mai-log
  del valore (regola D7).
- `src/keystore/env.rs`: `EnvStore` (livello 0 — legge `api_key_env`,
  `NEXUM_API_KEY`, `OPENAI_API_KEY`).
- `src/keystore/os_keychain.rs`: `OsKeychainStore` (wrapper su `keyring::Entry`,
  mappa gli errori keyring su `KeyStoreError`).
- `src/keystore/encrypted_file.rs`: `EncryptedFileStore` (AES-256-GCM +
  Argon2id, header versionato `salt + nonce + ciphertext`, file `0600`,
  path `dirs::config_dir()/nxm-tui/keys.enc`, buffer `zeroize`).
- `keys.rs`: `resolve_key_with(provider, store: &dyn KeyStore)` — ordine D7
  generalizzato (`store.get` → `api_key_env` → `NEXUM_API_KEY` →
  `OPENAI_API_KEY` → Missing). `resolve_key(provider)` resta come wrapper
  backward-compatible (costruisce lo store dal `KeySourceMode`).
- `KeySourceMode` esteso → `{EnvFirst, EncryptedFile, OsKeychain}` via
  `NXM_KEY_SOURCE`.
- `tests/keystore.rs`: `MockKeyStore` in-memory; roundtrip set/get/delete sul
  fake e su `EncryptedFileStore` (tempdir); ordine di risoluzione con
  `MockKeyStore` che **sostituisce** il `#[ignore]`.

## Dipendenze (pin esatti, RustCrypto, MIT/Apache-2.0)

- `aes-gcm = "0.11.1"`
- `argon2 = "0.6.0"`
- `zeroize = "1.9.0"`
- `rand = "0.10.3"`

## Done when

Build + clippy puliti sui file toccati; `tests/keystore.rs` verde con il
roundtrip e l'ordine di risoluzione; `keychain_beats_env` non più `#[ignore]`
(riscritto contro il seam con `MockKeyStore`); zero `unwrap`/`expect` in prod;
THIRD-PARTY-NOTICES aggiornato se necessario; file ≤300 righe (split per impl).
