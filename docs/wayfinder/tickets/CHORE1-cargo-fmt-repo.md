# CHORE1 — `cargo fmt` di massa sul repo (debito rustfmt pre-esistente)

- **Label:** `wayfinder:chore`
- **Type:** Chore (formattazione)
- **Status:** 📋 Aperto 2026-10-19
- **Priority:** BASSA (non blocca le feature, ma sporca ogni `fmt --check`)

## Problema

`cargo fmt -- --check` è **rosso** su `main` (@ `9fdd47d`): ~85-87 file non
formattati secondo rustfmt. È **debito pre-esistente**, accumulato nel tempo (molto
del codice `src/md/` è copia verbatim upstream, più altri moduli `src/`).

Scoperto durante la review del consolidamento di `web_fetch` (TOOLS-1 P2): i file
della feature erano formattati correttamente, ma il gate `fmt` del repo era comunque
rosso per i file pre-esistenti — rende `fmt --check` inutile come gate finché non si
sana il debito.

## Attenzione (non è un semplice `cargo fmt` cieco)

- I file `//! Derived from <upstream>` (`src/md/**`) sono copie **verbatim** di
  grok-build per restare allineati all'upstream (vedi `RULES.md` → Verbatim-copy
  exemption). Riformattarli **devia dall'upstream** e complica i realign futuri.
  → **Decidere**: escludere `src/md/**` dal fmt di massa (es. `#[rustfmt::skip]` o
  path exclude), formattando solo il codice nostro; oppure accettare la deviazione e
  annotarla nei file header. Questa è una **decisione di design**, non un chore cieco.

## Done when
- Policy decisa (escludere o no i `Derived from`).
- `cargo fmt -- --check` verde sui path inclusi.
- Un solo commit `chore(fmt): …` isolato (nessuna feature mescolata).
- CI/gate: valutare se aggiungere `fmt --check` come gate hard dopo la sanatoria.
