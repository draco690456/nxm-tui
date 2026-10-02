# FREE-AGENT PROMPT — T5: wire scrollback search into the UI (nxm-tui)

> Esegui questo prompt. Il revisore verificherà contro il codice reale.
> Lavora con precisione, verifica tutto, NON fare commit git.

## Repo & contesto

- Repo: `/Users/devdaniele/Projects/private/nxm-tui` (Rust, bin+lib via `src/lib.rs`).
- TUI ratatui 0.29 + crossterm + tokio. Ticket: `docs/wayfinder/tickets/T5-search-ui-wiring.md`.
- Il modulo `src/search.rs` È GIÀ PORTATO E TESTATO (commit 31b6f72). NON riscriverlo.
  API disponibili:
  - `search::QueryKind {Substring, Regex}`
  - `search::TextMatcher::new(query, kind)`, `.query()`, `.is_error()`,
    `.compiled_regex() -> Option<&regex::Regex>`, `.is_match(haystack) -> bool`
  - `search::next_index_after(sorted: &[usize], current: usize) -> Option<usize>`
  - `search::prev_index_before(sorted: &[usize], current: usize) -> Option<usize>`

## Obiettivo

Ricerca interattiva nello scrollback della chat:
- `/` (quando il prompt è vuoto) apre la modalità ricerca con una query bar.
- Digitando si aggiorna la query live; i match vengono evidenziati.
- `n` = match successivo, `N` = precedente (con wrap); il viewport salta al match.
- `Esc` chiude la ricerca e pulisce lo stato.
- `Enter` conferma/mantiene (vedi sotto) — scelta tua, documentala.

## Mappa del codice reale (verificata)

- **Render history**: `src/history.rs::render_history(f, app, area)` costruisce
  `let mut lines: Vec<Line<'static>>` (dopo wrap/markdown, con `HistoryCache`),
  poi `let offset = App::viewport_offset(lines.len(), area.height as usize, app.scrollback);`
  e `Paragraph::new(lines).scroll((offset,0))`.
- **Scroll model**: `app.scrollback: u16` = righe nascoste SOTTO il viewport
  (0 = tail). `App::viewport_offset(total, height, scrollback)` (app.rs:823)
  mappa a offset ratatui. Metodi `scroll_up/scroll_down/...` esistono.
- **Input modale**: pattern `set_key_pending` — stato dedicato in `App`,
  intercettato IN CIMA a `src/handler.rs::handle_key` prima del resto.
  Vedi i blocchi `if app.pending_approval.is_some()` (handler.rs:13) e
  `if let Some(entry) = app.set_key_pending.as_mut()` (handler.rs:25).
- **Layout**: `src/ui.rs` ~riga 74 — `Layout` verticale con mode bar / history /
  working / separator / input / separator / bottom-3. La query bar della
  ricerca può riusare la riga input o un overlay sopra la history (scelta tua).
- **Esc handling**: handler.rs:107 chiude help/sessions/metrics — aggiungi il
  ramo per chiudere la ricerca con la STESSA priorità.

## PUNTO DI DESIGN CRITICO (leggi con attenzione)

La ricerca deve operare sulle **righe FINALI già renderizzate** (`Vec<Line>`
prodotto da `render_history`), NON sul testo grezzo dei messaggi, altrimenti gli
indici non si allineano col wrapping/markdown. Approccio consigliato:

1. Estrai in `render_history` (o in una funzione helper riusabile) la
   costruzione di `lines: Vec<Line>` separata dal rendering, così la ricerca può
   calcolare i match sulle STESSE righe. Per cercare, estrai il testo di ogni
   `Line` (concatena gli `Span.content`).
2. Calcola `matches: Vec<usize>` = indici delle righe che il `TextMatcher`
   matcha (`is_match` sul testo-riga). Mantieni l'indice corrente in `App`.
3. `n`/`N` → `next_index_after`/`prev_index_before` sull'elenco ordinato →
   nuovo indice-riga `line_idx`. Converti in scrollback così la riga entra in
   viewport: `app.scrollback = (lines_total.saturating_sub(height)).saturating_sub(line_idx)`
   clampato ≥0 (verifica il senso con `viewport_offset`; aggiungi un test della
   conversione se la estrai in funzione pura).
4. Evidenziazione: quando la ricerca è attiva, in `render_history` ri-stila gli
   `Span` che contengono match usando `matcher.compiled_regex()` (match ranges)
   con uno stile di background/foreground evidente. Mantieni la perf (non
   ricompilare il regex per riga: compila una volta).

## Stato da aggiungere in `App` (src/app.rs)

Qualcosa come:
```rust
pub struct SearchState {
    pub query: String,
    pub matcher: Option<crate::search::TextMatcher>, // ricompila su cambio query
    pub matches: Vec<usize>,   // line indices
    pub current: usize,        // index into matches (or into line space)
}
pub search: Option<SearchState>,  // Some(..) quando la ricerca è attiva
```
Documenta le scelte. Reset su `/clear`, nuova sessione, invio messaggio (come fa
`scrollback = 0`).

## VINCOLI RULES.md (il revisore verifica)

- No `unwrap()`/`expect()` in prod. Test SOLO in `tests/` (no inline).
- `app.rs` è già ~960 righe (oltre il cap 300): metti la logica di ricerca-UI in
  un **nuovo modulo** `src/search_ui.rs` (o estendi `search.rs` con funzioni
  pure tipo `line_matches(lines_text: &[String], matcher) -> Vec<usize>` e
  `scrollback_for_line(line_idx, total, height) -> u16`), tenendolo testabile.
  `pub mod ...;` in lib.rs (ordine alfabetico). Non gonfiare app.rs oltre lo
  stretto necessario (lo stato + il wiring).
- Logging `tracing` target `nexum::search` sulle funzioni pubbliche nuove.
- Doc `///` con esempio. Codice/commenti in inglese.

## TEST (headless, in tests/)

- `tests/search_ui.rs` (o estendi `tests/search.rs`): funzioni pure
  `line_matches` (match su righe, smart-case) e `scrollback_for_line`
  (conversione indice→scrollback: primo match, ultimo, wrap, buffer più corto
  del viewport). Se possibile, un test su `TestBackend` (pattern
  `tests/overlay_render.rs`) che apre la ricerca, digita e verifica che una riga
  matchante sia resa evidenziata senza panic.

## VERIFICA (esegui e riporta output ESATTO)

- `cargo test` intero IN PARALLELO (riporta totale; l'unico fail ammesso è il
  noto `keychain_unavailable_falls_back_to_env` flaky dei test keys — dimostra
  che non ne aggiungi di nuovi).
- `cargo test --test search_ui` (o il file che crei).
- `cargo clippy --all-targets -- -D warnings` → 0 sui file toccati.
- `cargo build` ok.
- Descrivi a parole il flusso UX: `/` apri, digita "foo", `n`/`N`, `Esc`.

## OUTPUT FINALE (per il revisore)

NON committare. Riporta: (1) elenco COMPLETO file toccati/creati con righe;
(2) la scelta di design (dove vive la query bar, come estrai le righe, come
converti indice→scrollback) con lo snippet chiave; (3) output esatto test
(parallelo) + clippy; (4) conferma: 0 unwrap/expect prod nei file toccati,
nuovi file ≤300 righe, regex compilato una sola volta per query.
