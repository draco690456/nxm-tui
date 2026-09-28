# P3 — Hidden key-entry prompt prototype

- **Label:** `wayfinder:prototype`
- **Type:** Prototype (HITL) — cheap rough artifact to react to
- **Status:** Open · blocked (needs D7 shape)
- **Blocked by:** [D7-key-resolution-config.md](D7-key-resolution-config.md)
- **Map:** [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Question

Come si presenta l'inserimento key mascherato (`/provider set-key <nome>`)
nella TUI ratatui esistente (stato `prompt_state`, multiline)? Il prototipo
(money throwaway, non il codice finale) deve mostrare:

1. Dove vive lo stato del prompt segreto (riuso `prompt_state` o stato
   dedicato che non lascia la key in memoria oltre il salvataggio?).
2. Mascheramento a schermo (echi `•`, niente preview) + conferma/annullo
   (`Enter` salva nel keychain, `Esc` scarta e azzera il buffer).
3. Reazione visiva minima (overlay dedicato vs riuso overlay esistenti).

Riferimento pattern: overlay modali esistenti (`render_approval` in
`src/tool_overlay.rs`, Y/N modale in `handler.rs`).
