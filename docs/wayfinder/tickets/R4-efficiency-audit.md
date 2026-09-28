# R4 — audit di efficienza nxm-tui

- **Label:** `wayfinder:research`
- **Type:** Research (AFK) — subagent, no human needed
- **Status:** ✅ Resolved 2026-09-20 (subagent AFK)
- **Blocked by:** — (none)
- **Map:** [MAP.md](../MAP.md)

## Question

nxm-tui è percepito lento/inefficiente. Dove sta il costo reale (fonte primaria:
sorgenti in `src/`, RULES.md) e quali correzioni hanno il maggiore impatto?

1. Performance: strategia di redraw (full redraw ogni tick vs diff), wake-up
   cadence, allocazioni in hot-path, parsing SSE, syscall per frame, overlays.
2. Bloat/architettura: file oltre il limite di 300 righe di RULES.md,
   duplicazione, dead code, `unwrap`/`expect` in produzione.
3. Classifica dei fix per impatto percepito.

## Resolution (closed)

**Causa principale: redraw completo a ~60 Hz + ricostruzione di history/
markdown/linee ogni frame (O(N)).** Fix top: draw-on-change + poll 33 ms idle;
cache rendering history/markdown; memoizzare `estimated_tokens` (calcolato 3×
per frame); cache `current_dir_name`. Bloat: 5 file oltre 300 righe
(`app.rs` 863, `handler.rs` 540, `ui.rs` 470, `agent.rs` 384, `main.rs` 319),
3 `unwrap/expect` in prod, `/v1/status` duplicata. Dettagli e classifica in
[research/efficiency-audit.md](../research/efficiency-audit.md).
