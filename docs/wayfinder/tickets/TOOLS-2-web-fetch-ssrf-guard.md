# TOOLS-2 — `web_fetch` SSRF guard (fetch URL non fidate)

- **Label:** `wayfinder:task`
- **Type:** Security hardening (segue TOOLS-1 P2 `web_fetch`)
- **Status:** 📋 Aperto 2026-10-19
- **Priority:** ALTA prima di esporre `web_fetch` a un modello non fidato
- **Related:** `TOOLS-1-adoption-and-selection.md` (Parte B, P2). Commit feature: `ca4e298`+`9fdd47d` (main @ `9fdd47d`).

## Problema (P0 della code-review Standards su `web_fetch`)

`WebFetch::run` (`nxm-tui-tools/src/builtins.rs`) esegue una GET HTTP su una URL
**arbitraria** fornita dal modello, senza alcuna protezione SSRF. Un modello (o un
prompt-injection via contenuto non fidato) può far puntare `web_fetch` a:

- `http://127.0.0.1:*` / `localhost` — servizi locali (incl. l'eventuale server
  Nexum sulla stessa macchina).
- IP link-local / privati — `169.254.0.0/16` (metadata endpoint cloud
  `169.254.169.254`), `10/8`, `172.16/12`, `192.168/16`.
- Schemi non-HTTP o redirect verso i suddetti.

Era una **scelta di scope esplicita** del brief TOOLS-1 P2 ("nessuna policy di
sandbox, follow-up separato"): il tool è stato consegnato e mergiato come fetch
minimale. Questo ticket è quel follow-up.

## Done when
- `web_fetch` **rifiuta** (con `ToolResult::err`, non panico) URL che risolvono a:
  loopback, link-local, private-use, metadata endpoint, e schemi non `http(s)`.
  + **test**: per ciascuna classe (loopback, 169.254.169.254, 10.x, schema `file:`),
    un test che asserisce `res.ok == false` e un messaggio esplicativo.
- Gestione **redirect**: un redirect verso un indirizzo bloccato è anch'esso rifiutato
  (non seguito ciecamente). + **test** con server che risponde `302` verso `127.0.0.1`.
- Opzionale: allow-list via env (`NEXUM_WEBFETCH_ALLOW_LOCAL=1`) per sviluppo, default off.
- Gate verde (`run_gate`), nessun warning nuovo, `src/md/` non toccato.

## Note
- Delegabile come micro-task (skill `implement` o `tdd`) con lo stesso workflow:
  feature branch + work branch, review esterna, consolidamento su main privato.
- Superfici di fallimento: (1) risoluzione/validazione indirizzo, (2) policy redirect.
  Valutare se splittare in 2 micro-task.
