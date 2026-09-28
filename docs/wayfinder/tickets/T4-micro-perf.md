# T4 — micro-perf: memoizzazione estimated_tokens + cache cwd

- **Label:** `wayfinder:task`
- **Type:** Task (perf)
- **Status:** ✅ Resolved 2026-09-20
- **Blocked by:** — (none; resto di R4, risolto dopo T3 nello stesso giorno —
  eccezione alla regola max-1-ticket autorizzata esplicitamente dall'utente)
- **Map:** [MAP.md](../MAP.md)

## Scope (da R4, research/efficiency-audit.md)

1. **`estimated_tokens` memoizzato** (`app.rs`): O(N) su messages, chiamato
   3-4× per frame (mode_bar, bottom, context overlay, context_pct).
   Fingerprint O(1) = `(messages.len(), last message)` → ricalcola solo
   quando messages cambia (streaming appende all'ultimo, nessuna mutazione
   dei messaggi intermedi).
2. **`current_dir_name` cache** (`app.rs`, chiamato per frame da `bottom.rs`):
   `std::env::current_dir()` è una syscall per frame; il processo non fa mai
   `chdir` → cache una tantum (OnceLock), commento con upgrade path (TTL se
   una feature futura farà chdir).

## Vincoli RULES.md

- Test in `tests/` (nuovo `tests/render_caches.rs`), nessun `unwrap`/`expect`
  in prod, Conventional commit `perf:`. app.rs resta >300 righe (split = T5).

## Resolution (closed)

**Fatto** (1 commit `perf:`):

1. **`estimated_tokens` memoizzato** — `Cell<Option<(fingerprint, value)>>`
   in `App`; fingerprint O(1) = `(messages.len(), last message)`; ricalcola
   solo quando messages cambia. 3 test in `tests/render_caches.rs`
   (streaming growth — un cache con key solo-su-len servirebbe stantio 8,
   nuovo messaggio, oracle differenziale vs App fresca).
2. **`current_dir_name` cache una tantum** — `OnceLock<String>`: il processo
   non fa mai chdir; commento ponytail con upgrade path (TTL come
   `git_branch` se una feature futura farà chdir). Test di coerenza su 100
   chiamate.

**68/68 test verdi, clippy 0 error.**
