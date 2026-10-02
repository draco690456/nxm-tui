# T2 — Provider/model commands

- **Label:** `wayfinder:task`
- **Type:** Task (HITL) — manual work the decisions wait on
- **Status:** ✅ Resolved 2026-10-02 (free agent + 2-round independent review)
- **Blocked by:** [D7-key-resolution-config.md](D7-key-resolution-config.md),
  [R2-models-api-shapes.md](R2-models-api-shapes.md)
- **Map:** [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Resolution (closed)

Implementato: `/models` (fetch async, parser due-envelope R2 in `src/models.rs`),
`/model use <id|numero>` (→ `cfg.model_name` + `Provider.default_model`),
`/provider remove-key <name>` (keychain `delete_credential`), modello attivo in
mode bar. Fetch non-bloccante (pending-state come `inference_task`), key
risolta per-provider via `resolve_key`, mai loggata. Prompt in
`T2-PROMPT-for-free-agent.md` + fix `T2-PROMPT-round2-fixes.md`. Verifica:
`tests/models.rs` 16/16, clippy 0 sui file toccati, 0 unwrap prod. Vedi CHANGELOG
2026-10-02.


## Question

Disegno dei comandi TUI (niente implementazione finale qui, solo il disegno
che D7+R2 sbloccano):

1. `/provider set-key <nome>` (prompt nascosto P3), `/provider remove-key
   <nome>`, messaggio quando la key manca e come si rimanda al set.
2. `/models` (lista dal provider attivo, stato pending/errore), `/model use
   <id|numero>` + default per provider; dove appare il modello attivo
   (mode bar? bottom bar?).
3. Testo help aggiornato + regola mai-stampare-key.
4. Casi di test per comando (mock `/v1/models`, keychain finto).

## Done when

Elenco comandi + interazioni + casi di test scritti e rivisti; l'implementazione
resta fuori dalla mappa (è la destination: spec, non codice).
