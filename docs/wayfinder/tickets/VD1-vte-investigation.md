# VD1 — Indagine dipendenze del VT engine (mappatura, non "serve adesso?")

- **Label:** `wayfinder:research`
- **Type:** Verifica dipendenze (prerequisito del VT engine, differito)
- **Status:** 🔴 DA RIFARE 2026-10-16 — il primo giro ha risposto alla domanda
  sbagliata ("il VT engine serve oggi?") invece del mandato ("quali sono le
  dipendenze del VT engine e quali ci servono?"). Report del primo giro
  consolidato come record in `../research/VD1-vte.md` (verdetto contestato
  dall'owner), MA l'indagine va **rifatta** sul mandato corretto qui sotto.
- **Related:** V3-render-adoption (nota VT), epic VT engine

## Mandato CORRETTO (cosa l'indagine deve davvero produrre)

Il task è **mappare le dipendenze** del VT engine di grok e decidere, **per
ciascuna**, se ci serve. NON è valutare se/quando la feature VT serve a nxm-tui.

Il VT engine (`render/terminal_output.rs` + `terminal/`) trascina ~12 crate
interne `xai-*` (viste nel Cargo.toml del pager-render:
`xai-tty-utils`, `xai-ratatui-textarea`, `xai-ratatui-inline`, `xai-grok-config`,
`xai-grok-shared`, `xai-grok-workspace`, `xai-grok-telemetry`, `xai-grok-tools`,
`xai-grok-paths`, `xai-grok-gboom`, `xai-dirs`, …) + crate pubbliche (`vte`).
Per l'adozione "100% Rust indipendente" serve sapere, DIPENDENZA PER DIPENDENZA,
cosa fa e se ci è utile.

## Done when (nuovo giro)

Una **tabella-mappa** con una riga per OGNI dipendenza del VT engine, colonne:
`dipendenza | cosa fa (1 riga) | usata da quale file del VT | ci serve? (SI/NO/FORSE) | verdetto (adottare pubblico / riscrivere in Rust / scartare) | note licenza`.
Coprire: tutte le `xai-*` elencate + `vte` + eventuali altre crate pubbliche non
già in nxm-tui che il VT engine richiede. Più una **sintesi**: quale sottoinsieme
minimo di dipendenze va effettivamente portato/riscritto per avere il VT engine
indipendente, e quali si scartano.

## Cosa NON rifare / errore del primo giro

- Il primo brief (`prompts/04-vd1...`) conteneva una domanda sviante ("Ci è utile
  un VT engine o basta strip-ANSI? serve adesso?") — ERRORE DI BRIEF (mio). Ha
  portato l'indagine a decidere la *tempistica della feature* invece di
  *mappare le dipendenze*. Il nuovo brief deve chiedere SOLO la mappa dipendenze.
- Materiale riusabile dal primo giro (già verificato, citazioni corrette):
  `terminal_output.rs` ha **un solo** import interno (`quantize`); la dir
  `terminal/` è detection e importa `xai_ratatui_inline`/`crate::host`. Questo è
  un buon punto di partenza per la mappa, ma va ESTESO a tutte le dipendenze del
  VT engine, non fermato a `vte`.

## Materiale primo giro (record, non accettato)

Vedi `../research/VD1-vte.md` (verdetto contestato). Da trattare come dati grezzi
di partenza, non come conclusione.
