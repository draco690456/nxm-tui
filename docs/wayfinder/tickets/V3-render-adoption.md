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
- **T0** ✅ (main `5b28c02`) scaffold `src/md/mod.rs` + `THIRD-PARTY-NOTICES` +
  `LICENSE-APACHE`; dep V3 pin esatti (10 crate pure-Rust).
- **T1** ✅ (main `312ab46`, PR-less FF) `MarkdownStyle` + `TableBorders` +
  `all_hidden`/`merge_styles` ← grok `style.rs`, split `style.rs` (265) +
  `style_adapt.rs` (20, `adapt_style` placeholder-identità); visibilità upstream
  (`pub(crate)`), test via API pubblica; esenzione copia-fedele applicata.
- **T2** ✅ (main `e2df8df`) `src/md/theme.rs` (159) — `fixed_style() ->
  MarkdownStyle` tokyonight (12 costanti RGB inline, verificate vs sorgente),
  funzioni-ponte ratatui→anstyle copiate verbatim (`to_anstyle` con `Reset→None`),
  `Theme::current()` non letto (multi-tema differito). 204 test.

### Fase 1 — Primo valore (rendering base migliore)
- **T3a** ✅ (main `3df44dd`) `src/md/core.rs` (88) — `parser_options` +
  `offset_events` + `DoubleTildeOnlyStrike` ← `markdown-core`, copia verbatim.
  Review: rimossa `MarkdownStats` fuori scope + header provenienza corretto. 208 test.
- **T3b** ⬜ (posticipabile) `src/md/analysis.rs` — `MarkdownStats`/`analyze`
  (+`strum`). Brief pronto `prompts/08-...`. Non blocca T4–T8.
- **T4** ✅ (main `cac6e5b`) `src/md/buffers.rs` (235) + `buffers_table.rs` (125)
  ← `buffers.rs` (348, split per cap 300). Tutti i tipi di output copiati verbatim
  (`MarkdownBuffers`, `StyledCell`, `CellSpan`, `RenderEvent`, `Table*`,
  `MermaidReplace`, `unicode_display_width`, polyfill char-boundary). 213 test.
  Review: APPROVED senza fix.
- **T5** `parse.rs` ← CORE di `parse.rs` (2248 righe). **Segmentato** (deciso
  2026-10-16): copia prima le dipendenze piccole/autoportanti, poi il parse
  splittato per il cap 300. `latex` (1968 righe) **stubbato** ora → ticket
  dedicato dopo (feature math di nicchia, ortogonale al rendering base; coerente
  con "nessuna riduzione" perché posticipata, non tolta).
  - **T5a** ✅ (main `982fa18`) `src/md/checkpoint.rs` (58) ← `checkpoint.rs` verbatim (Checkpoint + CheckpointKind, 8 varianti). 215 test.
  - **T5b** ✅ (main `ee7a9b7`) `src/md/syntax.rs` (150). **OPZIONE B** (decisa
    2026-10-16): adatta a `SyntaxSet::load_defaults_newlines()` (come V2
    `highlight.rs`), **senza** two-face/`build.rs`/assets/Swift; `expect` prod →
    gestione errore. **DEBITO VOLUTO** → in futuro OPZIONE A completa (two-face +
    build.rs `syntaxes.bin` + `tokyo-night.tmTheme` + `Swift.sublime-syntax` +
    suite test Swift, 250+ lingue) con ticket dedicato.
  - **T5c** ✅ (main `eeaf024`) `src/md/url_scan.rs` (96) + `src/md/output.rs` (17,
    solo `HyperlinkTarget` anticipato → resto in T7). `detect_plain_urls`/
    `for_each_plain_link` verbatim; test non-streaming. 231 test. Visibilità `pub`
    per testabilità (policy RULES).
  - **T5d** ✅ (main `121636e`) `src/md/open_code_highlighter.rs` (243) — OpenCodeHighlighter streaming cache (resume per-linea syntect, memo closed-fence), test su Syntect::new(). 239 test.
  - **T5e** ✅ (main `841bacb`) `src/md/parse.rs` (298) — scaffold `MarkdownParser`
    + `new`/builder + `parse()` + `ParsedMarkdown` + helper puri; `on_event`/
    `on_start`/`on_end` stub vuoti (no `todo!()`), `format_table` omesso. 248 test.
  - **T5f** ✅ (main `881bf0f`) dispatch eventi, split in 4 file (tutti ≤300):
    `parse_events.rs` (290, `on_event`+`push_highlight`), `parse_start.rs` (265,
    `on_start`), `parse_end.rs` (204, `on_end`+stub `format_table`/mermaid/math),
    `parse_inline.rs` (295, metodi inline). `latex.rs` (17, stub `None`). Ora
    `parse()` PRODUCE highlights. 252 test. Review: ~26 warning clippy nuovi
    (unused imports + 2 lint) trovati e corretti — il report li aveva dichiarati 0.
  - **T5g** ✅ (main `70af2ee`) rendering tabelle, split in 3 file (≤300):
    `parse_table.rs` (300, `format_table`), `parse_table_fmt.rs` (269, row/border),
    `parse_table_word.rs` (231, `cell_wrap_joins`). Stub `format_table` rimosso da
    `parse_end.rs`. 259 test. Deviazioni: let-chains→if annidati (Rust 2021),
    sort_by_key(Reverse). **→ PARSE COMPLETO (T5a-g tutti fatti).**
- **T6** render — `render_ratatui` (path ratatui, NON render_ansi). **Segmentato**
  (deciso 2026-10-17, opzione A fedele): copia prima i prerequisiti di
  `render_ratatui`, poi il render. Dipendenze non-copiate che render tira dentro:
  `source_map` (16 usi), `output::MarkdownRenderOutput`/`MarkdownRenderView` (era
  T7, anticipato), `hyperlinks` (OSC8+chunk link), `adapt_style` (già stub T1).
  - **T6a** ✅ (main `6fa7c0a`) `src/md/source_map.rs` (105) ← verbatim (mapping rendered→source). 264 test. ← FATTO
  - **T6b** ✅ (main `cfa5647`) `src/md/output.rs` (180) completato: CodeBlockSpan, MarkdownRenderOutput, MarkdownRenderView, build_code_block_spans. 271 test. ← FATTO
  - **T6c** ✅ (main `ee8944a`) `src/md/hyperlinks.rs` (153, prod ~150 → no split) — proiezione link→celle: ChunkLinkRange, source_to_chunk_offset, chunk_link_offsets, emit_segment_hyperlinks. 276 test. ← FATTO. (prerequisiti render COMPLETI)
  - **T6d** ✅ (main `fbb1ee1`) render ratatui, split in 4 file (≤300):
    `render.rs` (279), `render_events.rs` (223), `render_segments.rs` (247),
    `render_blocks.rs` (240). `render_ratatui` + `apply_transforms` +
    `build_render_events`. Escluso path ANSI (0 chiamate). Helper riusati da
    parse.rs. **280 test. → PARSE + RENDER COMPLETI.** render_ratatui produce
    MarkdownRenderOutput con Line ratatui (heading/bold/code/tabella/link verificati).
- **T8 (SWAP)** ✅ (main `7e23dd0`) pipeline cablata + shim. `md::render` ora
  esegue `MarkdownParser::new(content, fixed_style(), &mut buffers, Syntect::new()
  .as_ref()).max_table_width(Some(w)).parse().render_ratatui(true).lines`.
  `render_markdown` = shim dietro flag `NXM_USE_NEW_MD` (default OFF): ON → V3 +
  prefix per riga; OFF → legacy (estratto verbatim in `markdown_legacy.rs`,
  fallback intatto). `history.rs`/`highlight.rs` INVARIATI. 286 test; 6 test swap.
  **→ IL RENDERER V3 È CABLATO E ATTIVABILE. Test visivo utente: `NXM_USE_NEW_MD=1`.**
  **→ dopo T8: PRIMO OUTPUT MARKDOWN RICCO A SCHERMO.**

### Fase 2 — Wrapping (BISOGNO VERIFICATO 2026-10-17: il render V3 NON wrappa la
prosa → una riga lunga sfora il pane. Serve prima del test visivo.)
- **T9a** `src/md/line_utils.rs` ← `pager-render/src/render/line_utils.rs` (470,
  prerequisito di wrapping: `fit_line_to_width`, `push_owned_lines`, + helper).
  Split per cap 300.
- **T9b** ✅ (main `940b0e1`) word-wrap, split 3 file: `wrap.rs` (280), `wrap_line.rs` (291, word_wrap_line/lines), `wrap_input.rs` (132). dep line_utils T9a. 319 test. Test comportamentali (width≤N). ← FATTO
- **T10** integra il wrap nel path: `md::render` (o lo shim) applica
  `word_wrap_lines(output.lines, max_width)` così la prosa sta nel pane.
  Verifica: la prosa lunga di 182 col a width 40 → più righe ≤40.

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
