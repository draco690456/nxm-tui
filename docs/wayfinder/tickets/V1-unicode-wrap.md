# V1 — Unicode-aware wrap in the markdown renderer

- **Label:** `wayfinder:task`
- **Type:** Task (bugfix + vendor adoption)
- **Status:** 🚧 Claimed 2026-10-15 (`wayfinder/v1-unicode-wrap`)
- **Map:** vendor adoption — `docs/patterns/VENDORS-TUI.md` prio **#0**
  (COPIA `openviking wrap_display_text`), analisi in `VENDOR-TUI-ANALYSIS.md`
- **Depends on:** nothing. `search` (T5) già cablato; V2 (syntect) e V3
  (streaming incrementale) restano open e **non** toccati qui.

## Problem

`src/markdown.rs` misura il testo in **byte**, non in colonne di display:

- `wrap()` usa `cur.len() + word.len() + 1 > max`: con CJK/emoji (2 colonne per
  char, 3–4 byte per char) il wrap sbaglia di molto — righe che escono dal pane
  o si spezzano prima del necessario.
- il troncamento dei code fence usa `line.chars().count() > inner_w`: un char
  largo conta 1 invece di 2 (e il `+ "…"` non è contato). Il panic UTF-8 è già
  stato tamponato (troncamento char-safe), ma la larghezza resta sbagliata.
- una "parola" più lunga della larghezza (URL, hash, path) non viene spezzata:
  sfonda il pane e disallinea la history.

Nota di licenza (verificata **nel sorgente**, non nei doc): la root di
`OpenViking` è **AGPL-3.0** (Main Project), ma `crates/ov_cli` è
**Apache-2.0** esplicito (`crates/ov_cli/Cargo.toml: license = "Apache-2.0"`,
`crates/LICENSE`, README §License). Il file da adattare
(`crates/ov_cli/src/commands/render_utils.rs`) sta **dentro `crates/ov_cli`**
→ copia/derivazione consentita con attribuzione (§4). La nota in
`VENDOR-TUI-ANALYSIS.md` che dava per Apache-2.0 anche la root è **errata** e va
corretta in questo ticket.

## Scope

1. `src/markdown.rs`: `wrap()` a larghezza di display (`unicode-width`),
   adattato da `wrap_display_text`/`append_word_to_line` di openviking:
   normalizzazione spazi, word-append, **split char-level** per le parole più
   lunghe della riga.
2. Troncamento dei code fence per larghezza di display, `…` finale incluso nel
   budget (comportamento attuale preservato: `…`, non `...`).
3. `Cargo.toml`: `unicode-width` pinnato (`0.2.0` — già in `Cargo.lock`, nessuna
   nuova copia di crate).
4. `THIRD-PARTY-NOTICES.md`: attribuzione openviking (`crates/ov_cli`,
   Apache-2.0).
5. `docs/patterns/VENDOR-TUI-ANALYSIS.md`: correggere il claim di licenza sulla
   root (AGPL-3.0) e restringere l'ok-a-copiare a `crates/ov_cli`.

## Deviations dal sorgente openviking (documentate)

- Nessun `max_lines`/troncamento con ellissi ASCII: la history deve mostrare
  **tutte** le righe (il viewport scrolla già, D4/T3). L'`…` resta solo sul
  troncamento orizzontale dei fence.
- Nessuna normalizzazione distruttiva dei newline hard: si mantiene la
  tokenizzazione dei paragrafi come oggi (il reflow dei blocchi è V2/V3).

## Out of scope (frontiera vendor residua)

- **V2** markdown + syntect (`xai-grok-markdown`): highlight reale per token,
  sostituisce `highlight_style()`.
- **V3** streaming render incrementale + cache highlight resumable.

## Done when

Wrap corretto a larghezza di display per prose e fence (nessuna riga renderizzata
supera `max_width`), test headless in `tests/markdown.rs` (RULES: niente
`#[cfg(test)]` inline), `cargo test` verde, `clippy` = baseline (0 nuovi),
`markdown.rs` ≤300 righe, attribuzione di licenza presente. Nessun cambio di
API pubblica (`render_markdown` invariata).
