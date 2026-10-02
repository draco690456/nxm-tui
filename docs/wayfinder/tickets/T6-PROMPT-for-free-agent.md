# FREE-AGENT PROMPT — T6: split app.rs (RULES.md 300-line cap)

> Refactor behavior-preserving. NESSUN cambio di logica. NON commit git.
> Il revisore verificherà che il comportamento è identico (diff solo di
> spostamento + re-export) e che la suite resta verde.

## Repo & contesto

Repo `/Users/devdaniele/Projects/private/nxm-tui`. `src/app.rs` è ~973 righe,
oltre il cap RULES.md di 300. Ticket: `docs/wayfinder/tickets/T6-split-app-rs.md`.

## Obiettivo (subset SICURO)

Estrai due blocchi autocontenuti da `app.rs` in moduli nuovi, SENZA cambiare
logica, mantenendo gli import esistenti funzionanti via re-export.

### 1. `src/commands.rs`
Sposta da `app.rs`:
- `pub enum Command` (~riga 192)
- `pub enum ProviderCommand` (~214)
- `pub enum ServerCommand` (~230)
- `pub enum ConfigCommand` (~237)
- `pub fn parse_command(input: &str) -> Command` (~256–369)

### 2. `src/metrics.rs`
Sposta da `app.rs`:
- `pub struct Metrics` (~528) + `impl Metrics` (~549–617)

## REQUISITO DI COMPATIBILITÀ (critico)

I test e `handler.rs` importano `nxm_tui::app::{parse_command, Command,
ProviderCommand, Metrics}`. NON devono rompersi. Quindi in `app.rs` aggiungi
i re-export:
```rust
pub use crate::commands::{Command, ProviderCommand, ServerCommand, ConfigCommand, parse_command};
pub use crate::metrics::Metrics;
```
Così `app::parse_command`, `app::Command`, `app::Metrics` ecc. restano validi.
NON toccare i call site (handler.rs, tests/*). Verifica che `tests/provider.rs`,
`tests/models.rs`, `tests/thinking_metrics.rs` compilino senza modifiche.

## PASSI

1. Crea `src/commands.rs`: incolla i 4 enum + `parse_command`. Porta i `use`
   necessari (probabilmente nessun tipo esterno oltre a std; `parse_command`
   ritorna `Command` e usa solo `&str`/`split`). Aggiungi `//!` doc al modulo.
2. Crea `src/metrics.rs`: incolla `Metrics` + impl. Porta i `use` (es.
   `std::collections::VecDeque`, `std::time::Instant` — controlla cosa usa).
   Aggiungi `//!` doc.
3. In `src/lib.rs`: `pub mod commands;` e `pub mod metrics;` (ordine alfabetico).
4. In `src/app.rs`: rimuovi i blocchi spostati, aggiungi i `pub use` re-export
   in cima (dopo gli altri `use`). Rimuovi eventuali `use` ora inutilizzati in
   app.rs (clippy li segnala).
5. Se `Metrics`/`Command` usano tipi definiti in `app.rs` (es. `AgentMode`,
   `Message`), importali nel nuovo modulo con `use crate::app::...`. Verifica le
   dipendenze incrociate: `parse_command` NON deve dipendere da `App`; se c'è un
   ciclo, lascia il tipo problematico in app.rs e re-esporta solo il resto —
   documenta.

## VINCOLI RULES.md

- NESSUN cambio di comportamento: è puro spostamento + re-export. Il diff di
  `app.rs` deve essere quasi solo rimozioni; i nuovi file quasi solo il codice
  spostato verbatim.
- No `unwrap/expect` NUOVI (non ne introdurre; quelli esistenti spostati
  restano com'erano — ma `parse_command`/`Metrics` non dovrebbero averne).
- `commands.rs` e `metrics.rs` ≤300 righe (lo saranno). `app.rs` deve
  diminuire sensibilmente.
- Logging: non aggiungere nuovo logging (è un refactor).
- Inglese.

## VERIFICA (esegui e riporta output ESATTO)

- `cargo build` ok.
- `cargo test` IN PARALLELO: stesso risultato della baseline. L'UNICO fail
  ammesso è `keychain_unavailable_falls_back_to_env` (flaky preesistente dei
  test keys). Dimostra con `git stash` che la baseline ha lo stesso identico
  fail e che NON ne aggiungi.
- `cargo clippy --all-targets -- -D warnings` → 0 sui file toccati (correggi
  eventuali `unused_imports` lasciati in app.rs dallo spostamento).
- Conferma che `tests/provider.rs`, `tests/models.rs`, `tests/thinking_metrics.rs`
  compilano SENZA modifiche (i re-export funzionano).

## OUTPUT (per il revisore)

NON committare. Riporta: (1) righe di app.rs PRIMA e DOPO (quanto è sceso);
(2) elenco COMPLETO file creati/modificati; (3) i re-export aggiunti; (4) output
esatto di test (parallelo) + clippy; (5) conferma: nessun cambio di logica
(solo move+re-export), call site invariati, nessun unwrap nuovo.
