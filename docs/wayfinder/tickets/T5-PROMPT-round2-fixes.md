# FREE-AGENT PROMPT — T5 ROUND 2 (fix dalla review)

> Review round 1: struttura OK (modulo puro `search_ui.rs` 17 test, highlight
> funziona, query live, 0 unwrap, clippy pulito). MA la navigazione n/N NON
> salta al match: lo scrollback è calcolato con valori sbagliati. Fix sotto.
> NON rifare ciò che funziona. NON commit git.

## 🔴 FIX 1 — n/N non porta il viewport al match (bug funzionale)

**Problema:** in `src/handler.rs`, nel ramo `KeyCode::Char('n') | 'N'`, chiami:
```rust
let total_lines = app.messages.len(); // SBAGLIATO: conta i MESSAGGI, non le righe
let height = 20;                        // SBAGLIATO: hardcoded
app.scrollback = scrollback_for_line(line_idx, total_lines, height);
```
`line_idx` è un indice nello spazio delle RIGHE RENDERIZZATE (centinaia),
mentre `app.messages.len()` è il numero di messaggi (es. 10) e `height` è finto.
Risultato: `max_scrollback = messages.len().saturating_sub(20) = 0` → scrollback
sempre 0 → il viewport non salta MAI. L'highlight si vede ma la navigazione è rotta.

**Causa radice:** il totale righe (`lines.len()`) e l'altezza reale del viewport
(`area.height`) sono noti SOLO in `render_history`, non nell'handler.

**Fix (approccio consigliato):** sposta il calcolo dello scrollback-per-match
in `render_history`, dove i valori sono reali. Nell'handler, `n`/`N` aggiorna
solo `search.current` (la posizione nella lista match) e setta un flag/segnale
"jump pending". In `render_history`, dopo aver calcolato `search.matches` sulle
righe reali:
```rust
if let Some(ref mut search) = app.search {
    if let Some(ref matcher) = search.matcher {
        // ... calcolo lines_text + search.matches come già fai ...
        // Se c'è una richiesta di jump (o sempre, se matches non vuoto):
        if !search.matches.is_empty() {
            search.current = search.current.min(search.matches.len() - 1);
            let line_idx = search.matches[search.current];
            app.scrollback = crate::search_ui::scrollback_for_line(
                line_idx, lines.len(), area.height as usize);
        }
    }
}
```
Nota: `render_history` prende `area: Rect`, quindi `area.height` è disponibile;
`lines.len()` è il totale reale. Attenzione al borrow: potresti dover leggere
`lines.len()` e `area.height` in variabili PRIMA del blocco `if let Some(ref mut
search)` per evitare conflitti di borrow con `app`.

**Chiarisci anche la semantica di `search.current`:** deve essere la POSIZIONE
nella lista `search.matches` (0-based), NON un indice-riga. In handler n/N:
- forward: `search.current = (search.current + 1) % search.matches.len()`
- backward: `search.current = (search.current + len - 1) % len`
Questo è più chiaro di passare indici-riga a `next_index_after`. (Se preferisci
mantenere `next_index_after`, usalo su una sequenza `0..matches.len()` con wrap,
ma il modulo wrap classico è più leggibile per la posizione.) Documenta.

## 🟡 FIX 2 (perf, opzionale ma consigliato) — ricalcolo ad ogni frame

`line_matches` scorre tutte le righe e `highlight_line` rigenera le righe
matchanti ad OGNI frame mentre la ricerca è attiva, vanificando parte della
cache T3. Mitigazione minima: ricalcola `search.matches` solo quando la query
cambia (non ad ogni render). Puoi tenere un hash/len della query o un flag
`dirty` settato dall'handler su modifica query, e in render ricalcolare solo se
dirty. Se è troppo invasivo, LASCIA com'è ma AGGIUNGI un commento che documenta
il trade-off (ricalcolo per-frame solo quando search attiva). Scelta tua, motivala.

## VERIFICA (esegui e riporta output ESATTO)

- Aggiungi/aggiorna un test puro in `tests/search_ui.rs` per la navigazione:
  dato `matches` e `current`, forward/backward con wrap producono il `current`
  atteso; e `scrollback_for_line(matches[current], total, height)` porta la riga
  in viewport (già testato scrollback_for_line — aggiungi il caso navigazione).
- `cargo test` IN PARALLELO (unico fail ammesso: `keychain_unavailable_falls_back_to_env`
  flaky preesistente — dimostra che non ne aggiungi).
- `cargo test --test search_ui`.
- `cargo clippy --all-targets -- -D warnings` → 0 sui file toccati.
- `cargo build` ok.
- Descrivi a parole: con 200 righe renderizzate e viewport 20, cercando una
  stringa che matcha a riga 150, `n` porta la riga 150 in viewport? (deve!).

## VINCOLI (invariati)

No unwrap/expect prod. Test in tests/. File ≤300 righe. Logging `nexum::search`.
Doc `///`. Inglese. Attenzione ai borrow in render_history.

## OUTPUT (per il revisore)

NON committare. Riporta: (1) file toccati; (2) come hai spostato il calcolo
scrollback in render_history e chiarito search.current, con snippet;
(3) output test (parallelo) + clippy; (4) conferma che n/N ora salta davvero al
match (descrizione del flusso con numeri reali).
