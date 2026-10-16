## ⚠️ Dissenso dell'owner (2026-10-16) — verdetto DA RIVERIFICARE

L'owner **non è convinto** del verdetto di questa indagine ("strano che `vte`
abbia dato questi risultati; secondo me è sbagliato quanto prodotto"). Il
documento è consolidato come **record dell'analisi svolta**, NON come decisione
accettata. Il verdetto (specie "VT engine non serve oggi" e la scelta `vte`) va
**riverificato** prima di qualsiasi azione a valle. Possibili aree di revisione:
output dei tool futuri (subprocess/shell), scelta del parser, utilità reale della
dir `terminal/`. Finché il dissenso non è sciolto, i ticket a valle del VT engine
restano bloccati. Vedi follow-up aperto in `VD1-vte-investigation.md`.

---

# Research — dipendenza `vte` e VT engine di grok (VD1, 2026-10-16)

Indagine su fonti primarie: sorgente vendor in sola lettura
(`vendors/agents/grok-build`, allineato all'upstream HEAD `2bdd1d6`) +
docs.rs/GitHub di `vte`, `anstyle-parse`, `termwiz`. Ogni affermazione sul
vendor e su nxm-tui porta `file:riga` verificati. Branch `work/VD1-vte-research`.
Nessuna modifica a codice di produzione.

## Verdetto

| Oggetto | Decisione | Perché |
|---|---|---|
| `vte` 0.15.0 | **ADOTTARE DIRETTO** (quando il VT engine verrà attivato) | pure-Rust, `Apache-2.0 OR MIT`, footprint minimo (`arrayvec`+`memchr`), API identica a quella contro cui il vendor è scritto → riscrittura quasi letterale |
| `terminal_output.rs` di grok | **RISCRIVERE** (non copiare tale e quale) | dipende da `quantize` (tema interno del pager) e dal modello pager; per il resto è quasi standalone: 573 righe, un solo import esterno |
| dir `terminal/` di grok | **SCARTARE** (rinviabile) | non è il renderer VT: è detection di terminale/multiplexer/editor (env sniffing) e trascina le `xai-*`; a nxm-tui non serve |
| VT engine **oggi** | **NON SERVE** | l'output dei tool di nxm-tui è testo semplice: nessun subprocess, zero ANSI nel path (§3) |
| VT engine **domani** | **SERVE se** arrivano tool che catturano subprocess (es. bash/shell) | è la ragion d'essere del modulo vendor: progress bar (`\r`), SGR, cursor (docstring `terminal_output.rs:1-7`) |

Sintesi: **adottare `vte` diretto + riscrivere `terminal_output` in forma minima
indipendente** quando il VT engine verrà attivato (differito per V3, dopo
T0–T16). Oggi non serve né un VT engine né strip-ANSI. Forma minima in §4.

## 1. Cosa fa `vte`

**Modello.** `vte` è solo un **parser a state-machine** per sequenze ANSI/VT in
**output** (implementa il modello di Paul Williams, vt100.net). Il `Parser`
fa il bookkeeping e **non assegna significato** ai dati: "The state machine
doesn't assign meaning to the parsed data and is thus not itself sufficient
for writing a terminal emulator" (docs.rs/crate/vte/0.15.0). Il significato lo
fornisce il consumer implementando il trait **`Perform`** (9 metodi, tutti con
default no-op, trait object-safe):

- `print(char)` — carattere stampabile; `execute(u8)` — controlli C0/C1
  (`\r`, `\n`, `\t`…);
- `csi_dispatch(&Params, &[u8], bool, char)` — fine sequenza CSI (SGR, cursor);
- `osc_dispatch(&[&[u8]], bool)` — OSC (parametri grezzi; le **semantics** OSC 8
  sono a carico del consumer);
- `esc_dispatch(&[u8], bool, u8)`; DCS via `hook`/`put`/`unhook`;
- `terminated()` — per `advance_until_terminated`.
- API: `Parser::new()`, `parser.advance(&mut Perform, &[u8])`
  (docs.rs/vte/0.15.0/vte/struct.Parser.html).

**Cosa copre** (stati in `src/lib.rs`: Ground, Escape, Csi*, Dcs*, OscString):
CSI (incl. SGR e cursor), OSC (incl. BEL-terminated), ESC, DCS/APC/PM/SOS,
print/execute. **Cosa NON copre** (confermato docs + sorgente): nessun modello
schermo/griglia/scrollback, nessun rendering, nessuna conversione colori,
nessun wrapping, nessun parsing input (tastiera).

**Fatti di implementazione** (Cargo.toml + src/lib.rs del repo alacritty/vte):
pure Rust (nessun FFI), `no_std` opzionale (`std` feature default), 5 blocchi
`unsafe` (lib.rs:579,586,629,683,695 — `MaybeUninit`, `from_utf8_unchecked`),
state-machine scritta a mano (match, non generata da spec; `memchr` SIMD solo
in ground state). Dipendenze: `arrayvec 0.7.2` + `memchr 2.7.4` (+ 4 opzionali).
MSRV 1.62.1. **Licenza `Apache-2.0 OR MIT`**. v0.15.0 (tag 2025-02-02), repo
attivo (push 2026-02). Usato da **alacritty** (`alacritty_terminal` 0.15.0);
wezterm **non** lo usa (ha termwiz/vtparse).

## 2. Come lo usa grok in `terminal_output.rs`

File: `xai-grok-pager-render/src/render/terminal_output.rs` (573 righe), pin
`vte = "0.15.0"` nel `[workspace.dependencies]` del vendor
(`grok-build/Cargo.toml:317`), consumato come `vte = { workspace = true }`
(`xai-grok-pager-render/Cargo.toml:44`).

**Import** (righe 9-13): ratatui `Style/Color/Modifier/Line/Span`,
`vte::{Params, Parser, Perform}` (:11), e **un solo** import interno:
`crate::theme::color_support::quantize` (:13).

**Callback `Perform` implementate** (`impl Perform for TermSink`, :209): solo
**tre**; OSC/ESC/DCS restano i no-op di default (test ~:445-450: OSC title e
DEC private modes ignorati).

- `print` (:210) → `put` (:81): scrive il char nella cella corrente.
- `execute` (:214): `\n`/VT/FF → `newline()`; `\r` → `col=0`; `\t` → tab 8-stop;
  BS → `col-1`; altri ignorati.
- `csi_dispatch` (:224): `'m'` → `apply_sgr` (:230); `'K'` erase-line e
  `'J'` erase-display (:231-232); `'A'-'D'` cursor up/down/fwd/back e
  `'G'` colonna assoluta (:234-244); **tutto il resto `_ => {}`** (:246) —
  niente save/restore, niente `H`, niente scroll region, niente alt-screen.

**SGR → ratatui** (`apply_sgr`, :154): reset a `base` (:157,162); `1/2/3/4/7`
→ `Modifier` BOLD/DIM/ITALIC/UNDERLINED/REVERSED (:163-167); `22/23/24/27`
rimuovono (:168-171); `30-37`/`40-47` fg/bg → `quantize(ansi16(n))`
(named Color, :172-174); `39/49` → base (:173,175); `90-97`/`100-107` bright
(:176-177); `38/48` → `ext_color` (:178-183) con `38;5;n` → `Color::Indexed`
e `38;2;r;g;b` → `Color::Rgb` (`ext_color` :290, `parse_ext` :315, forma `:`
supportata). **Ogni colore passa da `quantize`** (capability-downgrade del
pager) — l'unica dipendenza non std/vte/ratatui del file.

**Stato** (`TermSink`, :53-59): `rows: Vec<Vec<Cell>>` + cursore `(row, col)`,
`Cell { ch, style }` (:48-51). **È un transcript a righe illimitato, non una
griglia**: nessun wrapping, nessun viewport, nessuna scroll region; guardie
`MAX_ROWS=50_000`/`MAX_COLS=8_192` (:17-18); `ensure_row` (:72) e `put`
(pad con spazi `base`, :81-96); `erase_line` (:111), `erase_display` (:133);
`finish` (:196) → `Vec<RenderedLine { line: Line<'static>, plain: String }>`
(:21). `row_to_line` (:331) comprime le celle a style uguale in
`Span::styled` e taglia i blank finali.

**Nota importante vs ticket V3**: la preoccupazione "12 crate xai-*" viene
dalla dir `terminal/` (detection: `terminal/mod.rs` importa
`xai_ratatui_inline::WidthShrink` :12, `crate::host::HostOs` :13, ecc.) e dal
contesto pager — **non** da `terminal_output.rs`, che è quasi standalone.

## 3. Ci è utile per nxm-tui? — Verdetto motivato

**Percorso dell'output tool oggi** (tutto verificato):

- Tool = 3, tutti bloccanti, **nessun subprocess/PTY**: `read_file`
  (`std::fs::read_to_string`, `nxm-tui-tools/src/lib.rs:99-106`),
  `list_resources` (read_dir → JSON, :116-130), `web_search` (reqwest, :144-161).
- `ToolResult { ok, content: String }` (lib.rs:20-29) → `registry.call` →
  `r.content` in `ToolPart::ToolResult` (`src/agent.rs:242-251`) →
  `Message::tool` (`src/main.rs:396-399`, `src/message.rs:25,59`).
- Rendering: `Role::Tool` → `Span` semplice (`src/history.rs:157-164`); overlay
  Ctrl+O → `fit()` span semplici (`src/tool_overlay.rs:118-142`); preview
  troncata a 100 char (`src/tool_types.rs:189-194`).
- **Zero handling ANSI**: grep `ansi|escape|\x1b|\u{1b}|SGR|OSC|strip` in
  `src/` e `nxm-tui-tools/src/` → nessun codice di parsing/stripping (solo
  `Span::styled`); nessuna dep `strip-ansi`/`ansi_term` in Cargo.toml/lock;
  gli `anstyle*` (Cargo.toml:73-75) servono solo a markdown/syntect.
- Unici subprocess fuori dai tool: `git` per la status bar
  (`src/app.rs:242-247`) e il server MCP (`src/mcp/transport.rs:58-76`) —
  nessuno dei due passa dal path tool-output.

**Verdetto**: l'output dei tool di nxm-tui **non contiene ANSI grezzo** — è
testo semplice catturato in toto, mai in streaming, e renderizzato com'è.
Quindi: **oggi non serve un VT engine**, e nemmeno strip-ANSI (al più difensivo
per `read_file` su file che contengono escape — problema cosmetico, non blocca
nulla). **Il VT engine diventa necessario solo quando arriveranno tool che
catturano output di subprocess** (bash/shell): in quel caso, senza VT engine,
ratatui dipinge gli escape letteralmente (docstring vendor `terminal_output.rs:1-7`:
"the scrollback shows literal escape codes like `[1m[36m`"). La decisione V3 di
differire il VT engine a dopo il markdown (T0–T16) resta corretta; questa
indagine approva la sua prima dipendenza (`vte`) per quel momento.

## 4. Adottare diretto vs riscrivere vs scartare + forma minima

- **`vte`: adottare diretto.** Crate pubblico, pure-Rust, permissivo, minuscolo,
  manutenuto, ed è esattamente l'API contro cui il codice vendor è scritto.
- **`terminal_output.rs`: riscrivere** (il comportamento, non il file):
  rimuovere `quantize` (nxm-tui ha tema fisso truecolor — la capability
  downgrade è già "rinviata" in V3), il `RenderedLine.plain` (clipboard/search
  del pager — si riprende se serve), e il `base: Style` diventa parametro del
  chiamante. La logica VT→ratatui (sink, SGR, span) si tiene com'è.
- **dir `terminal/`: scartare** (rinviabile): detection di
  terminale/multiplexer/editor embedded (`mod.rs` = env-map "Detects terminal
  emulator, multiplexer, and Byobu from environment variables", `term_version.rs`,
  `hyperlinks.rs` = capability OSC 8 per brand, `embedded_editor.rs` = Neovim/
  Emacs `:terminal`). Serve a un CLI che deve adattarsi al terminale ospite;
  nxm-tui delega già capability a crossterm. Nessun impatto sul rendering
  dell'output tool.

**Forma minima proposta** (solo `vte` pubblico + ratatui, zero `xai-*`), per
quando il VT engine verrà attivato:

1. `src/vt/mod.rs` (+ `src/vt/sgr.rs` se serve stare <300 righe per file,
   RULES.md): port di `TermSink` — `rows: Vec<Vec<Cell>>`, cursore `(row,col)`,
   guardie `MAX_ROWS`/`MAX_COLS`.
2. `impl Perform`: `print`, `execute` (`\n \r \t BS`), `csi_dispatch`
   (`m/K/J/A-D/G`); OSC/ESC/DCS no-op come nel vendor.
3. SGR→Style: ANSI16/bright → `Color` named; `38/48;5;n` → `Color::Indexed`;
   `38/48;2;r;g;b` → `Color::Rgb`. **Niente quantize.**
4. API: `render_terminal_lines(raw: &str, base: Style) -> Vec<Line<'static>>`
   (+ plain-text solo se/ quando servirà search/clipboard).
5. Test: port dei test del vendor (SGR, 256/truecolor, `\r` progress bar,
   erase, OSC ignorato) in `tests/`.
6. Licenza: header `Derived from xai-grok-pager-render (Apache-2.0)` +
   voce in `THIRD-PARTY-NOTICES.md` (convenzione V3).

Stima: ~300 righe di produzione + test — coerente col fatto che il vendor è
573 righe con docstring, RenderedLine e test inline.

## 5. Alternative pure-Rust (confronto)

| Crate | Versione | Licenza (SPDX) | Modello | Footprint |
|---|---|---|---|---|
| **vte** | 0.15.0 (2025-02) | `Apache-2.0 OR MIT` | `Parser` + trait `Perform`, `advance(&[u8])`, action `char` | `arrayvec`+`memchr` |
| anstyle-parse | 1.x (fork dichiarato di vte) | `Apache-2.0 OR MIT` | `Parser`+`Perform` con `advance(u8)` per singolo byte, action `u8`, niente `terminated`, `CharAccumulator` | `arrayvec`+`utf8parse` |
| termwiz (escape) | 0.24.0 | `MIT` | escape parsing delegato a `wezterm-escape-parser 0.1.0` (su `vtparse 0.7.0`, entrambi `MIT`): callback + enum `Action` | **enorme**: ~20+ crate del workspace wezterm |

- **anstyle-parse** è un fork di vte (README: "forked from" alacritty/vte) —
  stessa state-machine, stessa licenza. L'allineamento all'ecosistema anstyle
  non è un vantaggio qui: `anstyle-parse` **non** è già nel dependency tree di
  nxm-tui (Cargo.lock: solo `anstyle`, `anstyle-lossy`, `anstyle-syntect`) e
  il VT engine è un problema di emulazione terminale, non di styling. Con
  anstyle-parse ogni callback del vendor andrebbe riscritta di firma
  (action `u8` vs `char`, byte-per-byte vs buffer).
- **termwiz::escape** sproporzionato per un TUI leggero (trascina
  wezterm-cell, wezterm-bidi, terminfo, image…), modello completamente diverso
  dal `Perform` del vendor.
- Altri (una riga, non approfonditi): `avt` 0.18.0 (`Apache-2.0`, parser+screen
  model), `ansi-parser` 0.9.1 (licenza poco chiara su docs.rs).

**Scelta: `vte`** — criterio primario V3 (fedeltà al vendor = port quasi
letterale), a parità di licenza e footprint.

## Decisioni categorizzate (nxm-decision)

- `noul` "l'output dei tool attuali può contenere ANSI grezzo?" → **p=0.50**
  (nessun segnale dal backend calibrato). Verdetto fondato sull'evidenza
  codice: **no** (grep zero hit, nessun subprocess — §3).
- `noul` "con tool subprocess servirà un VT engine?" → **p=0.49** (idem,
  nessun segnale). Verdetto fondato sull'evidenza: **sì** (docstring vendor
  + comportamento ratatui verbatim).
- `choice` parser (vte vs anstyle-parse vs termwiz) → argmax
  **anstyle-parse p=0.77, vte p=0.23, confidence 0.54** — segnale debole a
  favore dell'"ecosistema anstyle". **Override motivato**: (1) il criterio
  primario V3 è la fedeltà al vendor, che dà `vte`; (2) anstyle-parse non è
  già nel tree (verificato), quindi il vantaggio ecosistema è nullo; (3) è un
  fork della stessa state-machine: nessun guadagno di correttezza a fronte di
  firme da riscrivere. → **`vte`**.

## Fonti

- Vendor (read-only): `vendors/agents/grok-build/crates/codegen/xai-grok-pager-render/src/render/terminal_output.rs` (righe 1-18, 21, 48-62, 81, 111, 133, 154-183, 196, 209-246, 253, 261, 275, 290, 315, 331, ~445-450); `src/terminal/{mod.rs,term_version.rs,hyperlinks.rs,embedded_editor.rs}` (header); `grok-build/Cargo.toml:317`; `xai-grok-pager-render/Cargo.toml:44`.
- nxm-tui: `nxm-tui-tools/src/lib.rs:20-29,99-161`; `src/agent.rs:242-251`; `src/main.rs:132,396-399`; `src/message.rs:25,59`; `src/history.rs:157-164`; `src/tool_overlay.rs:118-142`; `src/tool_types.rs:128-140,189-194`; `src/app.rs:242-247`; `src/mcp/transport.rs:58-76`; `Cargo.toml:8,73-75`; `Cargo.lock` (anstyle 1.0.14, anstyle-lossy 1.1.5, anstyle-syntect 1.0.5; nessun vte/anstyle-parse).
- Crate: docs.rs/vte/0.15.0 (`Parser`, `Perform`, crate page); github.com/alacritty/vte (README, Cargo.toml, src/lib.rs); alacritty `alacritty_terminal/Cargo.toml`; docs.rs/crate/anstyle-parse + github.com/rust-cli/anstyle (README fork, Cargo.toml, lib.rs); github.com/wezterm/wezterm (termwiz, wezterm-escape-parser, vtparse Cargo.toml + parser/mod.rs); docs.rs/avt, docs.rs/ansi-parser.
