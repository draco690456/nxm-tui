# VD1 — Indagine dipendenza: `vte` (VT engine)

- **Label:** `wayfinder:research`
- **Type:** Verifica dipendenza (prerequisito del VT engine, differito)
- **Status:** ⚠️ Report consolidato 2026-10-16, **verdetto CONTESTATO dall'owner — da riverificare** (doc: `../research/VD1-vte.md`). Il documento registra l'analisi svolta; la decisione NON è accettata. I ticket a valle del VT engine restano bloccati finché il dissenso non è sciolto.
- **Related:** V3-render-adoption (nota VT), epic VT engine

## Perché

Il VT engine di grok-build (`render/terminal_output.rs` + `terminal/`) serve a
renderizzare l'output di comandi/subprocess che emettono ANSI (SGR, cursor,
`\r`). È **differito** a dopo il markdown (T0–T16), ma la sua adozione richiede —
per decisione utente — un **task di verifica per ogni dipendenza** prima di
copiare: capire cosa fa, se ci è utile, cosa riscrivere in Rust indipendente vs
scartare. Questo è il **primo** di quei task: `vte`.

## Fatti noti (da verificare/approfondire)

- `vte = "0.15.0"` (crate di Alacritty, `Apache-2.0 OR MIT`, **pure-Rust**, nessun
  binding C).
- In grok è usato in `xai-grok-pager-render/src/render/terminal_output.rs` (573
  righe) via `vte::{Params, Parser, Perform}` — pattern `impl Perform`.
- nxm-tui **non** usa `vte` e **non** ha oggi rendering di output-comandi.

## Domande dell'indagine (Done when risposte)

1. **Cosa fa `vte`**: modello (parser VT state-machine + trait `Perform`
   callback), cosa copre (SGR, cursor, CSI/OSC, `\r`, scroll), cosa NON copre.
2. **Come lo usa grok** in `terminal_output.rs`: quali callback `Perform`
   implementa, come traduce in celle/`Line`/`Span` ratatui, come gestisce lo
   stato (griglia? linee?), dipendenze (`quantize`, altre xai-*).
3. **Ci è utile per nxm-tui?** nxm-tui esegue tool via `nxm-tools`: l'output è
   testo semplice o può contenere ANSI grezzo (progress bar, colori)? Serve un VT
   engine o basta strip-ANSI? Verdetto motivato.
4. **Cosa riscrivere in Rust indipendente vs scartare**: `vte` è adottabile
   diretto (pubblico, pure-Rust → sì); il `terminal_output.rs` di grok va
   riscritto (dipende da `quantize`/tema e dal loro contesto). Proponi la forma
   minima di un VT→ratatui per nxm-tui.
5. **Alternative pure-Rust** a `vte` (es. `vte` vs `anstyle-parse` vs
   `termwiz::escape`): confronto breve, con licenza.

## Done when

- Documento di findings in `docs/wayfinder/research/VD1-vte.md` che risponde a
  tutte le 5 domande, con verdetto: **adottare `vte` diretto / riscrivere
  terminal_output / scartare** + forma minima proposta.
- Nessuna modifica a codice di produzione (è solo indagine).

## Esito (2026-10-16)

**Verdetto: adottare `vte` 0.15.0 diretto** (pure-Rust, `Apache-2.0 OR MIT`,
footprint `arrayvec`+`memchr`, API identica al vendor) **+ riscrivere
`terminal_output`** (~300 righe, senza `quantize`/pager, forma minima nel doc
§4) quando il VT engine verrà attivato; **dir `terminal/` scartata** (detection,
non rendering, trascina `xai-*`). Oggi il VT engine **non serve**: l'output dei
3 tool (`read_file`/`list_resources`/`web_search`) è testo semplice, zero ANSI
nel path (nessun subprocess); servirà solo con tool che catturano subprocess
(bash/shell). Differito per V3 come da piano.
