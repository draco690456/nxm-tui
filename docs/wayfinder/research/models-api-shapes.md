# Research — /v1/models shapes per preset (R2, 2026-09-09)

Subagent AFK su docs ufficiali vendor. Sintesi operativa per il parser TUI.

## Per preset

- **Nexum Inferentia** (bare host, no `/v1`): nessuna doc primaria pubblica;
  trattare come dialetto Ollama locale. Chiamata effettiva
  `GET {base}/v1/models`. Attenzione collisione porta `11434` con Ollama.
  Fallback: `{base}/models`, poi Ollama nativo `{base}/api/tags`
  (`{models:[{name,…}]}`).
- **Ollama** (`/v1`): nessuna auth in locale (solo cloud `ollama.com`).
  Quirk SDK: `api_key='ollama'` ignorata; curl senza header.
  Envelope OpenAI `{object:"list",data:[{id,created,owned_by}]}`;
  `created` = last-modified, `owned_by` default `"library"`. Errori
  `200/400/404/429/500/502` body `{"error":"…"}`. **Listed vs loaded**:
  `/v1/models` e `/api/tags` = tutti i disponibili; `/api/ps` = solo i
  caricati (con `expires_at`, `size_vram`, `context_length`).
- **LM Studio** (`/v1`): nessuna auth di default; se abilitata (0.4.0+),
  `Authorization: Bearer $LM_API_TOKEN`. `api_key="lm-studio"` è placeholder
  ignorato. `/v1/models` senza header negli esempi. Envelope OpenAI; con
  Just-In-Time loading la lista ⊇ caricati. Dettaglio nativo:
  `GET /api/v0/models` (type/publisher/arch/quantization).
- **NVIDIA** (cloud, key **obbligatoria** `nvapi-…`): envelope OpenAI + extra
  NIM (`root/parent/max_model_len/permission[]`). Self-hosted
  (`localhost:8000/v1/models`, senza key) = solo modello caricato.
  Errori: 401 auth, 403 regione, 429 quota (onorare `Retry-After`), 500/503.

## Parser tollerante (regole)

1. Due envelope: OpenAI `data[]` **e** Ollama nativo `models[]`; mai
   richiedere `data`.
2. ID: preferire `id`, fallback `name`, poi `model`; saltare entry vuote.
3. Scalari opzionali con default (`owned_by="unknown"`, `created=0`);
   `created` int **o** stringa ISO.
4. Ignorare chiavi ignote, mai fallire su di esse.
5. Status: refused/timeout → "server down"; 401 → azionabile solo su NVIDIA
   (o LM Studio con auth on → prompt key); 404 → riprovare path alterno
   prima di riportare; 429/503 → backoff + `Retry-After`. Nessuna
   paginazione documentata (cap display comunque).

Fonti: docs.ollama.com (api/tags, ps, auth, errors, openai-compatibility),
lmstudio.ai/docs/developer (auth, models, rest/endpoints, server),
docs.nvidia.com (nim api-reference, nemo api-keys),
platform.openai.com (models/list, error-codes). Report integrale del
subagent agli atti di sessione.
