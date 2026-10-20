# CHORE2 — Isolamento env var nei test SSRF (test flaky + 30s timeout)

- **Label:** `wayfinder:chore`
- **Type:** Chore (correttezza test)
- **Status:** ✅ Risolto 2026-10-20 — `#[serial]` (serial_test) sui test che toccano la env var (commit `3039700`); suite `web_fetch` ~30s → ~1.9s, deterministica (3 run verdi). Zero tocchi al codice di produzione.
- **Priority:** MEDIA (non è un bug di produzione, ma rende la suite flaky e lenta)
- **Related:** `TOOLS-2-web-fetch-ssrf-guard.md` (feature mergiata con `984b1aa`).

## Problema (trovato in review indipendente di TOOLS-2)

I test in `nxm-tui-tools/tests/web_fetch_ssrf.rs` usano
`std::env::set_var/remove_var("NEXUM_WEBFETCH_ALLOW_LOCAL")` tramite l'helper
`with_allow_local`. La env var è **globale di processo**, i test girano in **parallelo**
(default `cargo test`), e l'helper **non serializza** l'accesso.

Conseguenza osservata: mentre un test `with_allow_local` ha la var impostata, il test
`ssrf_web_fetch_private_ip_blocked` (che fa fetch a `http://127.0.0.1:9999/` aspettandosi
il blocco SSRF) può vedere `ALLOW_LOCAL=1`, **saltare la guardia** e tentare la fetch
reale → **timeout TCP ~30s**. La suite `web_fetch*` passa da 0.03s (thread singolo) a
**30s** (parallelo).

Due rischi:
1. **Lentezza**: +30s a ogni `cargo test` che include questi test.
2. **Flakiness / green per il motivo sbagliato**: `ssrf_web_fetch_private_ip_blocked`
   può passare per *timeout* invece che per *blocco SSRF* — o fallire in modo
   intermittente a seconda dell'interleaving.

**Il codice di produzione `ssrf.rs` è corretto** (bypass mapped-IPv6 chiuso, verificato).
Questo ticket riguarda SOLO la correttezza/determinismo dei test.

## Done when
- I test che toccano `NEXUM_WEBFETCH_ALLOW_LOCAL` sono **serializzati** (es. `serial_test`
  crate, o un `static Mutex` condiviso) così non c'è interleaving con i test di blocco.
- Nessun test di *blocco* SSRF effettua una fetch reale che dipende da un timeout:
  i casi di blocco devono asserire il rifiuto **prima** di qualsiasi I/O di rete
  (già vero per la validazione; assicurarsi che la env var non sia contaminata).
  + **test/verifica**: `cargo test -p nxm-tui-tools web_fetch` completa in **< 2s** e
    i risultati sono stabili su esecuzioni ripetute (no flakiness).
- Niente nuove dipendenze pesanti senza chiedere (`serial_test` è leggero e standard;
  valutare un `static Mutex` a zero deps).

## Note
- Delegabile come micro-task (skill `tdd`/`implement`), stesso workflow:
  feature branch + work branch, review esterna, consolidamento su main privato.
- Superficie unica: isolamento dei test. Piccolo.
