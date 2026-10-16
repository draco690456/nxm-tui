# V2 — Syntax highlighting dei code-fence via `syntect`

- **Label:** `wayfinder:task`
- **Type:** Vendor TUI adoption (segue V1)
- **Status:** ✅ Resolved 2026-10-16
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

## As-built (2026-10-16)

Eseguito da coding agent esterno (brief `prompts/02-v2-syntect-highlight_{ITA,ENG}.md`),
review indipendente del revisore + **simulazione visiva**. PR #3 → `main` (rebase).

- `src/highlight.rs` (155 righe): `highlight_code(code, lang) -> Vec<Vec<(Style,
  String)>>` per-token via syntect; `SyntaxSet`/`ThemeSet` via `OnceLock`;
  tema `base16-eighties.dark` (fg chiaro); mapping `syntect Color → ratatui Rgb`;
  fallback plain-text (lang vuoto/ignoto/errore) senza panic. `join_spans`/
  `truncate_spans` ospitati qui (cap `markdown.rs`).
- `src/markdown.rs` (283 righe): ramo `Segment::Code` consuma `highlight_code`
  per-span, box/border + troncatura cell-width V1 preservati; fallback a
  `highlight_style` (colore unico) su lang ignoto.
- `Cargo.toml`: `syntect 5.3.0` `default-fancy` (pure-Rust, no onig, no
  syntect-tui); `THIRD-PARTY-NOTICES.md` (MIT, URL `trishume/syntect` corretto).

**Difetti trovati dal test visivo (fix di review):** span con trailing `\n`
(`LinesWithEndings`) → riga vuota spuria per riga di codice → strip `\n`; tema
non deterministico/grigio-scuro → `base16-eighties.dark` esplicito; riga vuota
residua nel fallback su trailing newline → `plain_text` corretto. Regressioni
coperte da `spans_never_contain_newline`, `highlighted_row_count_matches_fallback`,
`fallback_drops_trailing_newline_keeps_interior_blanks`.

**Verifica (revisione indipendente):** `cargo build` ok; `cargo test
--test-threads=1` **193 passed / 0 failed / 1 ignored**; clippy = baseline, 0
nuovi; `cargo tree -i syntect` = 5.3.0 senza onig; file ≤300; no `unwrap`/`expect`
prod. Commit `3d7be26` + `c10ad60` + `6dd53cd`.

**Residuo (deferito a lavoro dedicato sul rendering):** rifinitura resa visiva
(scelta tema definitiva, eventuale sfondo del box, leggibilità) — accettata così
per ora su decisione utente.
