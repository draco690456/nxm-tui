# T3 — efficienza: draw-on-change + cache rendering

- **Label:** `wayfinder:task`
- **Type:** Task (perf)
- **Status:** ✅ Resolved 2026-09-20
- **Blocked by:** — (none; sbloccato da R4)
- **Map:** [MAP.md](../MAP.md)

## Scope (da R4, research/efficiency-audit.md)

1. **Draw-on-change + poll idle 33 ms** (`event.rs:13`, `main.rs:129`):
   niente `terminal.draw` incondizionato a ~60 Hz; disegnare solo su evento /
   token drainato / resize.
2. **Cache rendering history/markdown** (`history.rs`, `markdown.rs`): per-message
   cache delle `Vec<Line>`; durante lo streaming si re-renderizza solo l'ultimo
   messaggio, non l'intera O(N).

Out of scope (ticket futuri): memoizzazione `estimated_tokens`, cache
`current_dir_name`, split file >300 righe, rimozione `unwrap/expect`.

## Vincoli RULES.md

- Test in `tests/` (mai `#[cfg(test)]` inline), unit test prima del merge.
- Nessun `unwrap`/`expect` in prod. Conventional commit `perf:`.

## Resolution (closed)

**Fatto in un commit unico** (il tree conteneva già il lavoro command-menu
non committato della sessione precedente, intrecciato con la conversione lib):

1. **Draw-on-change + poll idle 33 ms** — `event.rs` 16→33 ms; `main.rs` dirty
   flag: `terminal.draw` solo su key/resize/token-drain/status/health change;
   lo spinner continua ad animare mentre working (stream in stall incluso).
2. **Cache rendering history** — `history.rs` `HistoryCache` per-message,
   key `(content.len(), tool_parts.len(), width)`: durante lo streaming si
   re-renderizza solo l'ultimo messaggio, non l'intera O(N); resize invalida
   tutto. 3 test di regressione in `tests/history_cache.rs` (frame ripetuti,
   invalidazione streaming, re-wrap resize — output differential, non internals).

Infra forzata inclusa: **target lib** (`src/lib.rs`) — un solo grafo moduli,
test su `nxm_tui::`, specchi `#[path]` per-test eliminati (erano già stantii:
i test crate non risolvevano `crate::autocomplete` → 9 file di test ora
compilano davvero: **64/64 verdi, clippy 0 error**).

Skipped (ticket futuri, vedi research/efficiency-audit.md): memoizzazione
`estimated_tokens` (O(N) 3×/frame), cache `current_dir_name`, split file
>300 righe, rimozione `unwrap/expect`. Nessun test di timing del cache
(flaaky); l'efficacia è data dal meccanismo + test di correttezza.
