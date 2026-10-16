# V3 — Adozione del render di grok-build (markdown ricco, tema fisso)

- **Label:** `wayfinder:task` (epic multi-ticket)
- **Type:** Vendor adoption (sostituisce il markdown handmade)
- **Status:** 🚧 In design 2026-10-16
- **Related:** V1 (cell-width wrap), V2 (syntect highlight), DEBT1

## Obiettivo

Portare in nxm-tui un rendering dell'output **ricco e corretto come grok-build**
(heading, liste, tabelle, blockquote, task-list, link, strikethrough, streaming
incrementale), con **UN tema fisso**. Il multi-tema è esplicitamente **rimandato**
(prima la qualità del rendering, poi la configurabilità).

Strategia utente: **copiare il più fedelmente possibile** dall'upstream, tutto ciò
che serve; nessun adattamento ridotto; restare allineati a
`github.com/xai-org/grok-build` per riallineamenti futuri.

## Fonte

`vendors/agents/grok-build` (Apache-2.0), vendor **già allineato all'upstream**
(HEAD `2bdd1d6`, 0 commit dietro, verificato 2026-10-16). Stack **identico** a
nxm-tui: ratatui + crossterm + tokio (la nota R1 "non ratatui" era imprecisa — il
pager È costruito su ratatui; `vte` serve solo all'output comandi, fuori scope).

Crate sorgente: `crates/codegen/xai-grok-markdown` (19.406 righe, pulldown-cmark)
+ `xai-grok-markdown-core` (1.016) + parti di `xai-grok-pager-render/src/render/`.

## Grafo dipendenze

Percorso critico: **C (stile/anstyle) → A (parsing) → B (render elementi)**.
Da B diramano: code-fence (syntect), E (link), F (wrapping), D (streaming).
G (bidi) / H (chrome) isolati, fuori percorso iniziale.
Catena xai-* minima: **solo `xai-grok-markdown-core`** → `src/md/core.rs`.
Tutto il resto = moduli interni copiati e **splittati** per il cap 300 righe,
sotto `src/md/`.

## Principio guida (deciso 2026-10-16)

**Copia praticamente identica del codice e delle funzionalità — NESSUNA
riduzione — ma 100% Rust e indipendenti dal workspace xai-*.** Dove grok usa
librerie o codice non-Rust (es. `syntect` con `default-onig` = binding C a
Oniguruma), lo **riscriviamo in Rust puro per noi** (qui: `syntect` con
`default-fancy`/fancy-regex, come già in V2). Dove grok si appoggia a crate
`xai-*` del suo workspace, valutiamo caso per caso: il markdown renderer dipende
solo da `xai-grok-markdown-core` (pura, copiabile); il resto NON va trascinato
ciecamente.

Deviazioni tecniche obbligate (documentate, NON riduzioni di funzionalità):
- `syntect` → `default-fancy` (pure-Rust), non `default-onig`. Se
  `open_code_highlighter` usa API onig-specifiche (resume per-linea), **riscrivere
  in Rust** l'equivalente con fallback.
- File > 300 righe → **split in più moduli** (RULES.md); il codice resta identico,
  cambia solo la distribuzione su file.
- **Esenzione copia-fedele (RULES.md §Logging/§Documentation):** i file
  `//! Derived from ...` sono esenti da `tracing` per-funzione e dai `///` con
  esempio, per preservare la fedeltà all'upstream (decisione 2026-10-16). Il
  codice NOSTRO e la logica non banale che aggiungiamo a un file portato seguono
  le RULES piene. Visibilità: mantenere quella dell'upstream (`pub(crate)` resta
  `pub(crate)`); i test esercitano gli interni **solo via API pubblica**.

## Nota — VT engine e dipendenze (differito, verifica per-dipendenza)

**Il VT engine NON serve per il markdown renderer** (il markdown è autoportante:
dipende solo da `xai-grok-markdown-core` + crate pubbliche). Il VT
(`render/terminal_output.rs` + `terminal/`) serve a renderizzare l'**output dei
comandi/subprocess** che emettono ANSI (SGR, cursor, `\r`) — ortogonale al
markdown. Quindi è **differito a dopo** il markdown (T0–T16).

`vte` (crate di Alacritty, `Apache-2.0 OR MIT`, v0.15.0) è **pure-Rust** → usabile.
MA il modulo VT di grok trascina **12 crate interne** `xai-*` (`xai-tty-utils`,
`xai-ratatui-textarea`, `xai-ratatui-inline`, `xai-grok-config`,
`xai-grok-shared`, `xai-grok-workspace`, `xai-grok-telemetry`, `xai-grok-tools`,
`xai-grok-paths`, `xai-grok-gboom`, `xai-dirs`, ...). La copia letterale non è
praticabile.

**Processo deciso per VT + dipendenze:** per **ogni dipendenza** (vte e ogni
`xai-*` coinvolta) si crea un **task di verifica dedicato** che: (1) studia cosa
fa quella dipendenza, (2) decide se ci è utile, (3) determina cosa riscrivere in
Rust indipendente vs cosa scartare. Poi si **adatta il nostro VT** a ciò che
effettivamente portiamo (riscrittura Rust indipendente del comportamento, basata
su `vte` pubblico + ratatui, senza le `xai-*`). Questi task saranno aggiunti al
piano DOPO la chiusura del markdown.

## Piano a fasi (ticket T0–T16)

Tutto il nuovo renderer in `src/md/`. `src/markdown.rs` resta uno **shim** fino
allo swap; `history.rs` non cambia fino alla Fase 4.

### Fase 0 — Fondamenta
- **T0** scaffold `src/md/mod.rs` + `THIRD-PARTY-NOTICES` + `LICENSE-APACHE`; dep
  `anstyle`, `pulldown-cmark`, `anstyle-syntect` (pin esatti).
- **T1** `style.rs` ← grok `style.rs` (split `style.rs` + `style_adapt.rs`).
- **T2** `theme.rs`: UN tema fisso concreto (tokyo-night) in anstyle.

### Fase 1 — Primo valore (rendering base migliore)
- **T3** `core.rs` ← `xai-grok-markdown-core` (parser_options, offset_events,
  strikethrough). Omesso MarkdownStats/analyze (no strum).
- **T4** `buffers.rs` + `buffers_table.rs` ← `buffers.rs`.
- **T5** `parse.rs` + `parse_inline.rs` + `parse_block.rs` ← CORE di `parse.rs`
  (2100 righe splittate). Stub no-op: table/math/mermaid/html/checkpoint.
- **T6** `render_style.rs` + `render_events.rs` + `render.rs` ← CORE di
  `render.rs`. Escluso render_ansi + test.
- **T7** `output.rs` ← CORE di `output.rs`.
- **T8** code-fence: NON il backend grok; estendere il `highlight.rs` esistente
  con path syntect→anstyle. **→ dopo T8: swap possibile, interfaccia invariata.**

### Fase 2 — Wrapping
- **T9** `wrap.rs` + `wrap_spans.rs` ← `wrapping.rs` CORE (flatten, slice_line_spans,
  RtOptions, joiners, blockquote). dep `textwrap` + `unicode-segmentation`.
- **T10** integra wrap nel render path.

### Fase 3 — Link
- **T11** `url_scan.rs` ← `url_scan.rs` (linkify).
- **T12** `hyperlinks.rs` ← `hyperlinks.rs`.
- **T13** `osc8.rs` ← 3 fn escape; integrazione schermo DA DECIDERE (ratatui std
  non espone OSC8 per-cella).

### Fase 4 — Streaming (tocca history.rs)
- **T14** `checkpoint.rs` + emissione incrementale.
- **T15** `open_code.rs` ← `open_code_highlighter.rs` (verificare resume per-linea
  con fancy-regex vs onig; fallback re-highlight tail).
- **T16** `streaming.rs` + `streaming_tail.rs`; `StreamingMarkdownRenderer`
  per-messaggio live, one-shot per lo storico.

## NON copiare ora (motivazione)
Mermaid (grafico), LaTeX/Math (nicchia, eventi stub no-op), immagini/video/gboom
(protocollo terminale + fork), terminal_output VT (enorme, off-scope), bidi RTL
(off-default anche in grok, opt-in), glyphs.rs (ConHost Windows legacy; target
macOS/Linux), render_ansi + SourceMap byte (nxm usa solo path ratatui),
colors.rs capability-downgrade (tema fisso truecolor → rinviato), osc8.rs pager
2000 righe (deps pager), draw loop / TermWriter / fork ratatui-inline (rewrite
event loop, zero valore md), scrollbar/preview chrome, two-face/onig (viola
pure-Rust).

## Swap con minimo blast radius
`src/markdown.rs` resta **shim**: `render_markdown` chiama
`crate::md::render(content, width, &theme::fixed())` e riapplica `prefix` a ogni
`Line`. `history.rs` non cambia (Fase 1–3). Rollout dietro flag `USE_NEW_MD` per
confronto. Solo Fase 4 tocca `history.rs`.

## Attribuzione Apache-2.0
`THIRD-PARTY-NOTICES.md` (upstream + URL + Apache-2.0 + copyright + mapping file +
note di modifica); header per-file `//! Derived from xai-grok-markdown
(Apache-2.0). Modified: ...`; aggiungere `LICENSE-APACHE` accanto a `LICENSE` MIT;
trailer di provenienza nei Conventional Commit.
