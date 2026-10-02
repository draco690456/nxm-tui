# P3 — Hidden key-entry prompt prototype

- **Label:** `wayfinder:prototype`
- **Type:** Prototype (HITL) — cheap rough artifact to react to
- **Status:** ✅ Resolved (verificato nel codice 2026-10-02)
- **Blocked by:** [D7-key-resolution-config.md](D7-key-resolution-config.md) (risolto)
- **Map:** [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Resolution (closed)

Implementato e committato (arrivato con `cb05a5d` migrate; verificato pulito
nel git 2026-10-02):

1. **Stato segreto dedicato** (non `prompt_state`): buffer in `App`
   (`set_key_pending`), azzerato dopo save/Esc — la key non resta in memoria
   oltre il salvataggio.
2. **Mascheramento + conferma/annullo**: overlay dedicato modale
   (`src/ui.rs::render` set-key overlay, `ui.rs:476` "masked key entry
   overlay"); `Enter` → `keyring set_password` via `spawn_blocking`
   (`handler.rs:37`), `Esc` → scarta + azzera.
3. **Comando**: `ProviderCommand::SetKey(String)` in `app.rs` (`parse_command`
   `set-key`), risoluzione per-provider allo spawn Agent (`main.rs`
   `spawn_blocking(resolve_key)`), warning una tantum se keychain assente.
4. **Mai-log rigido** rispettato (D7-5): nessun valore key stampato.

> Nota: `remove-key` e i comandi `/models` restano in **T2** (ancora aperto).

