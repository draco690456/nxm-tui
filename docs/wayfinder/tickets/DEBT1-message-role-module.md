# DEBT1 — Estrarre `Message`/`Role` in un modulo di dominio

- **Label:** `wayfinder:task`
- **Type:** RULES.md debt (follow-up T6)
- **Status:** ✅ Resolved 2026-10-15
- **Related:** T6 (app.rs 973→729, split `commands.rs`+`metrics.rs`)

## Motivazione

`Message` (struct) e `Role` (enum) sono tipi di **dominio core** (rappresentano
la conversazione), ma vivono dentro `src/app.rs` (790 righe) mescolati con lo
state/UI dell'app. Sono referenziati da ~60 call site su 19 file. Isolarli in un
modulo dedicato chiarisce il dominio e sblocca il lavoro successivo (V2 syntect,
V3 streaming, T2 provider/model) che vi si appoggia.

## Design (deciso — A1 + B1)

- **A1:** nuovo modulo `src/message.rs` che ospita `Message`, `Role`, i loro
  `impl` (`Message::new/with_tool/tool`) e gli helper strettamente legati ai
  tipi conversazione se e solo se dipendono solo da `Message`/`Role`
  (`push_thinking`, `absorb_token` — da valutare: spostare solo se non trascinano
  altro state di `app.rs`).
- **B1 (scope mirato):** SOLO estrazione + re-export. **NON** fare lo split
  ampio di `app.rs` sotto il cap 300 — è un ticket debt separato.

## Pattern obbligato (precedente T6)

Replica esattamente quanto fatto per `commands.rs`/`metrics.rs`: dopo
l'estrazione, `app.rs` **re-exporta** i tipi
(`pub use crate::message::{Message, Role};`) così i ~60 call site che usano
`app::Message`/`app::Role` continuano a risolvere **senza modifiche**. Registra
`pub mod message;` in `lib.rs`.

## Done when

- `src/message.rs` contiene `Message` + `Role` + loro `impl`; `app.rs` li
  re-exporta; `lib.rs` dichiara `pub mod message;`.
- I call site esistenti NON vengono riscritti (risolvono via il re-export) —
  oppure, se si preferisce il path diretto `crate::message::`, l'aggiornamento è
  puramente meccanico e coperto dalla compilazione.
- `app.rs` scende di righe (meno `Message`/`Role`), resta comunque > cap: OK,
  lo split pieno è altro ticket.
- Build + clippy puliti (0 lint nuovi vs baseline: App/Metrics `Default`,
  `type_complexity`); suite verde single-thread; zero `unwrap`/`expect` prod;
  file nuovo ≤300 righe.
- CHANGELOG aggiornato se il repo lo richiede.

## As-built (2026-10-15)

Eseguito da coding agent esterno (brief `prompts/01-debt1-message-role-module_{ITA,ENG}.md`),
review indipendente del revisore prima del merge. PR #2 → `main` (rebase).

- `src/message.rs` (100 righe): `Message` + `Role` + `impl` (`new`/`with_tool`/
  `tool`) spostati verbatim da `app.rs`, con doc `///` ed esempi in blocchi `text`
  (non eseguiti → conteggio doctest invariato). Import `ToolPart` localizzato.
- `src/app.rs`: definizioni rimosse, aggiunto `pub use crate::message::{Message, Role};`
  (pattern T6). `push_thinking`/`apply_token` lasciati in `app.rs` (dipendono dallo
  state thinking di `App`, A1+B1).
- `src/lib.rs`: `pub mod message;` (ordine alfabetico, nit di review).
- CHANGELOG.md + CONTEXT.md aggiornati.

**Verifica (revisione indipendente):** `cargo build` ok; `cargo test
--test-threads=1` **185 passed / 0 failed / 1 ignored** (= baseline); doctest 16/1
invariato; clippy = 3 lint baseline, 0 nuovi; `message.rs` 100 ≤ 300; nessun
`unwrap`/`expect` prod; 0 dei 19 file consumer modificati (re-export stabile).
Commit `cfb2178` + `ec45ca1`.
