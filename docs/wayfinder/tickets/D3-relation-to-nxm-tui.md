# D3 — Relationship to the existing `nxm-tui`

> **Nota post-riorganizzazione (2026-09-13):** record storico. La decisione
> ("evolvi `nxm-tui` in place") resta valida. Aggiornamenti di collocazione:
> `nxm-tui` è pubblicato come **`dangranaz/nxm-tui`** (upstream `nxm-ai/nxm-tui`)
> e ora **ospita anche il tool crate `nxm-tui-tools`** come workspace member.
> I percorsi locali citati sotto sono quelli storici.

- **Label:** `wayfinder:grilling`
- **Type:** Grilling (HITL) — needs the human
- **Status:** ✅ Resolved · Evolve `nxm-tui` in place (grilled this session)
- **Blocked by:** none (D1 / D2 resolved) — now unblocked

## Question

There is already an in-house pure-Rust terminal client, **`nxm-tui`**
(`ratatui 0.29` + `crossterm 0.28` + `tokio` + `reqwest`; "connects to any
OpenAI-compatible LLM server"), living in the `nexum` monorepo. The Wayfinder
`list` tool also indexes a `tui` cargo component of that same project.

> Once D1 (purpose = agentic REPL over OpenAI HTTP) and D2 (architecture =
> standalone pure-Rust) are decided: do we **coexist** (keep `nxm-tui`, build
> the agentic REPL as a new crate), **replace** `nxm-tui` (fold the REPL into
> it), or **evolve` `nxm-tui` in place (move the prototype into its tree)?

## Framing questions (for the grilling turn)

1. Is `nxm-tui` in active maintenance / is its repo the home the team expects,
   or is it a frozen stub? (Decides evolve-vs-coexist, and whether we'd be
   forking or extending.)
2. Does the agentic REPL belong *inside* `nxm-tui` (it already speaks the
   OpenAI HTTP transport D2 chose), or as a separate `nexum-tui` crate that
   *consumes* `nxm-tui`'s pieces?
3. If coexist: crate name/location — `crates/nxm-tui/` extend, or a repo-level
   `tui/` crate (align to the `tui` component the index reports)?

## Acceptance (when resolved)

- One of {coexist / replace / evolve} + a named crate path.
- If **evolve/replace**: P1's prototype is migrated into `nxm-tui`'s tree and the
  standalone `.wayfinder/prototypes/p1-skeleton` is retired.
- If **coexist**: a new crate name/location is reserved (this becomes P2).
- Unlocks: P2 (hardware-fit) scoping depends on crate ownership.

## Decision record (resolved)

- **Chosen:** **evolve `nxm-tui` in place** — fold the D1 agentic REPL into the
  existing pure-Rust `nxm-tui` terminal client. No new crate.
- **Crate name/path:** `nxm-tui` (repo `https://github.com/nxm-ai/nxm-tui`,
  branch `main`; sibling dir `/Users/devdanelle/Projects/nexum/nxm-tui`). The
  throwaway `.wayfinder/prototypes/p1-skeleton` retires once its validated loop
  lands in `nxm-tui`.
- **Confirmed anchor (why not a new crate):** `connection.rs` already exposes
  `chat_stream` (OpenAI SSE `bytes_stream` → tokio channel `tx`); `ui.rs` is
  multi-pane (`sidebar`/`history`/`prompt`/`mode_bar`/`overlays`/`bottom`);
  `tool_types.rs` + `app.rs` thinking-stream deltas exist. nxm-tui already
  matches D1/D2 (`ratatui 0.29` + `crossterm 0.28` + `tokio 1` + `reqwest 0.12`
  + `tracing`, OpenAI-HTTP) — folding the REPL in avoids a second pure-Rust CLI.
- **Next:** port P1's loop into `nxm-tui` (P2), then D4 streaming rendering +
  D5 `/fit` + TurboQuant.
- **Source:** D3 grilling (coexistence / REPL-placement / crate-location),
  answered "extend in place / inside nxm-tui".
