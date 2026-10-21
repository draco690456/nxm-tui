# Research — Loop engineering per nxm-tui (plan→execute→reflect)

> **Stato:** studio preliminare · **Creato:** 2026-10-21
> Origine: richiesta utente — "il plan goal è il loop engineering, sarebbe un
> plus molto interessante, fai uno studio e crea una sessione futura".
> Ancorato al codice reale: `src/agent.rs` (loop), `src/todo.rs`,
> `src/plan_mode.rs`, `src/goal.rs`, `docs/wayfinder/tickets/P2-agentic-loop.md`.

## 1. Cos'è il "loop engineering" (definizione 2026)

> *Loop engineering is the practice of designing the execution environment
> around an AI agent: the triggers, stopping conditions, feedback mechanisms,
> and failure controls that determine how the agent runs.* — Data Science Dojo, 2026

È il livello **sopra** il prompt engineering. Non riguarda *cosa* chiedi al
modello, ma *come* lo fai girare: quando parte, quando si ferma, cosa gli dà
feedback, cosa succede quando fallisce. Boris Cherny (team Claude Code, Anthropic)
ha detto che il suo lavoro si è spostato dal prompting del modello allo scrivere
i **loop di esecuzione esterni** che coordinano le azioni del modello.

Il termine è esploso a giugno 2026 (un post ha superato 6.5M view in 24h), ma la
radice è il paper **ReAct (2022)**. Il `/goal` di Claude Code e Codex è la
versione "prodotto" del pattern **Ralph Loop**.

## 2. I tre pattern di base (e dove sta nxm-tui)

| Pattern | Idea | Fonte accademica |
|---|---|---|
| **ReAct** | reason → act → observe → repeat, finché "stop" | Yao et al. 2022 |
| **Plan-and-Execute** | pianifica tutti gli step in anticipo, poi esegui | Wang et al. 2023 (Plan-and-Solve) |
| **Reflection / Reflexion** | genera → un critic rivede → rigenera, fino OK | Shinn et al. 2023 / Self-Refine |

**Dove sta nxm-tui oggi:** l'`Agent` in `src/agent.rs` è un **ReAct loop puro**.
`run()` fa: `stream_turn` (il modello ragiona + emette `tool_calls`) →
`dispatch_tool_calls` (esegue i tool, appende i `role:"tool"`) → ri-interroga,
finché `finish_reason == "stop"`. Nessun planner esplicito, nessun critic.

Regola d'oro 2026: *parti da ReAct; passa a Plan-and-Execute quando noti che
l'agente ri-deriva lo stesso piano a ogni richiesta; aggiungi Reflection come
loop esterno quando la qualità della risposta finale conta più della latenza.*

## 3. I 10 pattern di loop engineering vs. lo stato di nxm-tui

| # | Pattern | A cosa serve | Stato in nxm-tui |
|---|---|---|---|
| 1 | ReAct loop | base reason/act/observe | ✅ `Agent::run` |
| 2 | Reflection | l'agente critica sé stesso | ❌ assente |
| 3 | Tool use | chiamate a tool esterni nel loop | ✅ `Registry` + dispatch |
| 4 | Prompt chaining | sequenza deterministica fissa | ⚠️ parziale (nessun orchestratore) |
| 5 | **Ralph loop** | itera finché un **validatore esterno** (test/lint) dà OK; reset del contesto a ogni giro | ❌ — ma `update_goal` + `todo` sono i mattoni |
| 6 | Evaluator-Optimizer | un **secondo** agente valuta, il primo revisiona | ❌ assente (serve l'epic sub-agenti) |
| 7 | Multi-Agent Supervisor | un supervisore coordina worker specializzati | ❌ → è l'epic sub-agenti |
| 8 | **Circuit Breaker** | se l'agente è bloccato (stessi errori/stati per N cicli) stacca e avvisa | ❌ assente — **rischio reale** |
| 9 | Heartbeat | l'agente si sveglia su schedule/evento | ❌ fuori scope TUI chat |
| 10 | **Bounded Execution + Context Engineering** | tetto duro su iterazioni/token/tempo + cosa entra nel contesto a ogni giro | ⚠️ `estimated_tokens`/`context_pct` mostrano, ma **nessun cap** sul loop |

**Lettura chiave:** nxm-tui ha già i pattern "foundational" (1, 3) e i *pezzi* di
planning (goal + todo + plan_mode). Mancano i tre pattern di **produzione**
(8, 10) che sono "non negoziabili quando un loop gira in autonomia" — e che oggi
mancano del tutto nel nostro `Agent::run` (un `loop {}` senza max_steps).

## 4. Il collegamento goal → loop engineering (la tua intuizione)

La tua idea — *"il plan goal è il loop engineering"* — è esatta e ha un nome
preciso: il **Ralph Loop**, di cui `/goal` (Claude Code) è la versione prodotto.
Lo schema:

1. L'utente fissa un **goal** (già abbiamo `update_goal` → `goal_handle` →
   mode_bar 🎯).
2. L'agente **pianifica** la lista di task (già abbiamo `plan_mode` che si
   astiene dai write + `todo` che ospita la lista).
3. L'agente **esegue** un task alla volta.
4. Un **validatore esterno** (il nostro `run_gate` di nxm-exec: check/clippy/fmt/test)
   decide se il giro è OK.
5. Se non OK → **ri-pianifica/riprova**; se bloccato N volte → **circuit breaker**.
6. Loop finché il goal è raggiunto **o** il tetto (Bounded Execution) scatta.

Differenza cruciale Ralph vs Reflection (pattern 2): in Ralph la condizione di
uscita viene da **software deterministico** (test verdi, 0 errori di tipo), non
dall'autogiudizio del modello — che tende al "rubber stamp" (bias: i modelli
approvano il proprio output, Panickssery et al. 2024). Per nxm-tui questo calza
perfettamente: **abbiamo già `run_gate`** come validatore esterno.

## 5. Gap analysis — cosa manca per un vero loop engineering

Ordinati per rapporto valore/rischio:

**P0 — Safety del loop (fondamentali, mancano del tutto):**
- **Bounded Execution** (pattern 10): oggi `Agent::run` è `loop {}` senza
  `max_steps`/`max_tool_calls`/budget token. È lo stesso errore che è costato
  $47K a 4 agenti LangChain girati 11 giorni. **Da aggiungere a prescindere dal
  resto**: è la riga che separa un agente funzionante da un conto imprevisto.
- **Circuit Breaker** (pattern 8): rilevare stagnazione (stessi tool/argomenti
  ripetuti, nessun progresso per N cicli) e staccare con un messaggio in UI.
  Failure mode noto: "verifier stall" (il modello richiama `verify` all'infinito).

**P1 — Il Ralph/goal loop (la feature che vuoi):**
- Stato "goal attivo" che guida il loop (riusa `goal_handle`).
- Integrazione con `run_gate` come validatore esterno (già esposto da nxm-exec,
  ma **bloccato sulla deny-root** in questo workspace → serve un checkout fuori
  dagli indici, o invocare il gate via un canale consentito).
- Reset/compaction del contesto a ogni iterazione (pattern 10, Context
  Engineering) per non degradare su run lunghi.

**P2 — Reflection / Evaluator-Optimizer (qualità):**
- Reflection single-model (pattern 2): outer loop che ri-lancia i test falliti
  (stile AlphaCodium: 19%→44% su CodeContests).
- Evaluator-Optimizer (pattern 6) e Supervisor (pattern 7): richiedono un
  **secondo agente** → dipendono dall'**epic sub-agenti** (sessione dedicata
  `ses_20261021_...epic_sub_agenti`).

## 6. Rischio architetturale specifico di nxm-tui

`Agent::run` vive in un `tokio::spawn`, NON tocca `&mut App`, comunica via
`mpsc<ToolPart>`. Un loop engineering completo deve:
- rispettare questo confine (il cap e il circuit breaker vivono **dentro**
  l'Agent, emettono un `ToolPart::Error`/nuovo variant verso la UI);
- esporre lo stato del loop (iterazione corrente, ultimo verdetto gate) nella
  mode_bar/overlay come già fanno goal/todo/plan;
- NON diventare un orchestratore di auto-esecuzione *prima* di avere Bounded
  Execution + Circuit Breaker, altrimenti è esattamente il $47K bug.

## 7. Raccomandazione (ordine di implementazione)

1. **Bounded Execution** in `Agent::run` — `max_steps` + budget, `ToolPart`
   dedicato quando scatta. Micro-task singolo, basso rischio, alto valore.
2. **Circuit Breaker** — rilevamento stagnazione (hash di nome+args per ciclo).
3. **Goal-driven Ralph loop** — cablare goal + todo + plan_mode + `run_gate`
   come validatore; questa è la feature richiesta. Design da sciogliere:
   dove gira il gate (deny-root!), quante iterazioni, UI del loop.
4. **Reflection** (single-model, outer loop sui test) — opzionale.
5. **Evaluator-Optimizer / Supervisor** — rimandati all'epic sub-agenti.

## Fonti (primarie e secondarie)

- Data Science Dojo — *10 loop engineering design patterns* (2026): definizione + i 10 pattern.
- dev.to / Gabriel Anhaia — *ReAct, Plan-and-Execute, or Reflection* (2026): decision matrix, failure modes, skeleton.
- ReAct: Yao et al., arXiv:2210.03629 (2022).
- Plan-and-Solve: Wang et al., arXiv:2305.04091 (2023).
- Reflexion: Shinn et al., arXiv:2303.11366 (2023); Self-Refine: Madaan et al., arXiv:2303.17651.
- AlphaCodium: arXiv:2401.08500 (2024) — reflection + test esterni, 19%→44%.
- Ralph Loop: github.com/snarktank/ralph (Geoffrey Huntley, lug 2025).
- "The $47K agent loop" — perché Bounded Execution non è opzionale.
