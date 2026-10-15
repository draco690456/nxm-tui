# V2 — Syntax highlighting dei code-fence via `syntect`

- **Label:** `wayfinder:task`
- **Type:** Vendor TUI adoption (segue V1)
- **Status:** 🚧 Claimed 2026-10-15
- **Related:** V1 (cell-width markdown wrap), T6 (pattern estrazione modulo)

## Motivazione

Oggi i code-fence in `src/markdown.rs` hanno un **colore unico per linguaggio**
(`highlight_style(lang, _content)` → un solo `Style` per l'intera riga; `_content`
è già nella firma ma inutilizzato, predisposto). V2 introduce **vero syntax
highlighting per-token** via `syntect`, continuando la linea vendor sbloccata da V1.

## Design (deciso)

1. **Dipendenza:** `syntect = { version = "5.3.0", default-features = false,
   features = ["default-fancy"] }` — MIT; **pure-Rust** via `fancy-regex` (NON
   `onig`/C, coerente col vincolo "pure-Rust TUI" del repo). Pin esatto (RULES.md).
   **NO `syntect-tui`:** il mapping colore→ratatui è poche righe, non vale una
   seconda dep.
2. **Nuovo modulo `src/highlight.rs`:** espone
   `highlight_code(code: &str, lang: &str) -> Vec<Vec<(Style, String)>>`
   (righe → span tipizzati `ratatui::style::Style`). `SyntaxSet`/`ThemeSet` di
   default caricati **una sola volta** (`OnceLock` o equivalente, mai per-chiamata).
   Selezione syntax per token di lingua con **fallback plain-text**. Mapping
   `syntect::highlighting::Color` → `ratatui::style::Color::Rgb`.
3. **`src/markdown.rs`:** nel ramo `Segment::Code`, sostituire il singolo
   `highlight_style` con le righe tokenizzate di `highlight_code`, mantenendo il
   box/border e la **troncatura cell-width di V1** applicata **per-span**. Se il
   lang è sconosciuto o l'highlight fallisce → **fallback** al comportamento
   attuale (colore unico `highlight_style`). `markdown.rs` deve restare ≤300: se
   sfora, l'estrazione in `highlight.rs` lo riporta sotto.
4. **Superficie unica dominante:** integrazione syntect + mapping colori. La
   logica di larghezza (wrap/troncatura) è già V1 e **non va ritoccata**.

## Done when

- `src/highlight.rs` espone `highlight_code` con caricamento one-shot delle asset
  syntect e fallback plain-text; mapping colori→ratatui.
- `markdown.rs` produce più span colorati per riga di codice (non uno solo),
  preservando box, border e troncatura cell-width V1; fallback a colore-unico su
  lang ignoto / errore.
- `syntect` pinnato con `default-fancy` (no onig). `THIRD-PARTY-NOTICES.md`
  aggiornato (MIT).
- Build + clippy puliti (0 lint nuovi vs baseline); suite verde single-thread;
  zero `unwrap`/`expect` prod; `markdown.rs` e `highlight.rs` ≤300 righe.
- CHANGELOG + CONTEXT aggiornati.
- **Nota:** la verifica visiva su TTY reale è a carico dell'utente (test utente);
  i test automatici asseriscono il contratto strutturale, non la resa visiva.
