# R1 — keyring crate evaluation

- **Label:** `wayfinder:research`
- **Type:** Research (AFK) — subagent, no human needed
- **Status:** ✅ Resolved 2026-09-09 (subagent AFK)
- **Blocked by:** — (none)
- **Map:** [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Question

La crate `keyring` (o alternativa equivalente) è adatta a salvare una API key
per provider in `nxm-tui` su macOS / Linux / Windows, restando pure-Rust e
compatibile con le dipendenze attuali (`ratatui 0.29`, `tokio 1`)? Da fonti
primarie (docs.rs, repo ufficiale, non write-up secondari):

1. API di set/get/delete + come si indicizza una entry (service/username).
2. Backend per OS (macOS Keychain, Windows Credential Manager, Linux
   Secret Service / eventuali fallback) e comportamento quando il keychain
   non è disponibile (headless/SSH): errore tipizzato? Come rilevarlo?
3. Versione stabile consigliata, MSRV / peso dipendenze, conflitti noti.
4. Pattern di test senza keychain reale (mock/in-memory?).
5. Alternative se `keyring` risulta inadatta (una sola, con perché).

## Resolution (pending)

findings → `../research/keyring-crate.md`; gist di una riga qui + in
`MAP-provider-keys-models.md` Decisions so far alla chiusura.

## Resolution (closed)

**`keyring = "4"` adatta** (feature `v1`; service `nexum-tui`, user
`<provider>`; op bloccanti → `spawn_blocking`; MSRV 1.88). Headless rilevato
via `NoDefaultStore`/`store_status()` → fallback env + prompt. Test con
`keyring_core::mock::Store` o trait `KeyStore`. Dettagli in
[research/keyring-crate.md](../research/keyring-crate.md).
