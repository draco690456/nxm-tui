# T5 — Wire scrollback search into the UI

- **Label:** `wayfinder:task`
- **Type:** Task
- **Status:** In progress (claimed 2026-10-02) — free agent exec + independent review
- **Depends on:** `src/search.rs` (ported from grok, commit `31b6f72`) — DONE
- **Map:** vendor adoption (`docs/patterns/VENDOR-TUI-ANALYSIS.md` prio #1)

## Goal

Collegare il modulo `src/search.rs` (già portato e testato) alla TUI: ricerca
interattiva nello scrollback della chat con `/` per aprire, query live,
evidenziazione dei match, navigazione `n`/`N` con wrap, `Esc` per chiudere.

## Vincoli di design (dal codice reale)

- La history si renderizza in `history.rs::render_history` come `Vec<Line>`,
  poi `Paragraph::new(lines).scroll((offset,0))` con
  `offset = App::viewport_offset(lines.len(), height, app.scrollback)`.
- La ricerca deve operare sulle **righe finali già costruite** (dopo
  wrap/markdown) per allineare indici logici e renderizzati.
- Input modale come `set_key_pending`: stato dedicato in `App` intercettato in
  cima a `handle_key` (NON `prompt_state`).
- Navigazione: `search::next_index_after`/`prev_index_before` su indici-riga
  ordinati; converti l'indice del match in `app.scrollback` così il match entra
  nel viewport.
- Evidenziazione: `TextMatcher::compiled_regex()` sulle righe visibili.

## Done when

Ricerca funzionante (`/` apri, digita, `n`/`N` naviga, `Esc` chiudi, match
evidenziati), test headless su `TestBackend` dove possibile, clippy 0, no
unwrap prod, file ≤300 righe (nuova logica in modulo/funzioni dedicate, non
gonfiare app.rs ~960). Prompt: `T5-PROMPT-for-free-agent.md`.
