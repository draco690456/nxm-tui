# R2 — /v1/models shapes across presets

- **Label:** `wayfinder:research`
- **Type:** Research (AFK) — subagent, no human needed
- **Status:** ✅ Resolved 2026-09-09 (subagent AFK)
- **Blocked by:** — (none)
- **Map:** [MAP-provider-keys-models.md](../MAP-provider-keys-models.md)

## Question

Forma reale di `GET /v1/models` per i preset di `nxm-tui/src/provider.rs`
(Nexum Inferentia, Ollama, LM Studio, NVIDIA) — da docs ufficiali vendor
(non write-up secondari). Per ciascuno:

1. Richiede auth? Quale (Bearer key, `lm-studio`, nessuna)?
2. Forma JSON di risposta (campo `data[]`, nome campo id: `id`? altro?) con
   un esempio minimo reale dalla doc.
3. Paginazione, errori tipici (401/404/modello-scaricato-ma-non-caricato),
   note Ollama (modelli listati vs caricati) e LM Studio.
4. Cosa deve tollerare il parser TUI (campi mancanti, envelope diverse).

## Resolution (pending)

findings → `../research/models-api-shapes.md`; gist di una riga qui + in
`MAP-provider-keys-models.md` Decisions so far alla chiusura.

## Resolution (closed)

**Parser tollerante a due envelope** (`data[]` OpenAI + `models[]` Ollama;
id→name→model; chiavi ignote ignorate). Auth: NVIDIA sempre, LM Studio solo
se abilitata, locali mai. 404 → path alterno; 429/503 → backoff. Dettagli in
[research/models-api-shapes.md](../research/models-api-shapes.md).
