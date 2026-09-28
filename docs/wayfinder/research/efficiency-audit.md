# Audit di efficienza — nxm-tui

Data: 2026-09-20  
Repo: /Users/devdaniele/Projects/nxm-private/nxm-tui  
Fonti primarie: sorgenti Rust in `src/`, RULES.md.  
MCP nxm-memory non disponibile in questa sessione; analisi basata su lettura diretta del codice.

## Sintesi

La TUI è percepita lenta perché il ciclo principale forza un redraw completo a ~60 Hz e ricostruisce ogni frame intere strutture di rendering (history, markdown, metriche). A ciò si aggiungono file che violano il limite di 300 righe di RULES.md e allocazioni ripetute su hot-path. Le correzioni con impatto maggiore sono: ridurre la frequenza di wake-up, evitare la ricostruzione completa della history, memoizzare i valori O(N) e spezzare i file oversize.

---

## Performance

### 1. Wake-up a 16 ms e draw incondizionato [Impatto alto]
`src/event.rs:13`  
```rust
if event::poll(Duration::from_millis(16))? { ... }
```
Il loop in `src/main.rs:129-131` fa:
```rust
terminal.draw(|f| render(f, &app))?;
match event::poll_event()? { ... }
```
`terminal.draw` viene eseguito ad ogni iterazione indipendentemente da cambiamenti, e il poll sveglia il thread ogni 16 ms (~60 Hz). Ratatui fa diff interno, ma la costruzione dell’albero widget e la serializzazione del buffer avvengono sempre.
*Fix*: disegnare solo su evento o su cambiamento di stato; aumentare il timeout di poll quando idle e usare `event::poll` con `Duration::from_millis(33)` o più.

### 2. Ricostruzione completa della history ogni frame [Impatto alto]
`src/history.rs:15-55`  
`render_history` alloca un `Vec<Line>` e itera su `app.messages` per ogni tick:
- `wrap(&msg.content, w)` per ogni messaggio user
- `markdown::render_markdown(&msg.content, w, "   ")` per ogni messaggio assistant

Entrambe le funzioni allocano stringhe e span ad ogni frame. Con conversazioni lunghe il costo cresce linearmente con N messaggi.
*Cita*: `history.rs:22-35`, `markdown.rs:7-30`.
*Fix*: render incrementale / caching delle linee già formate; invalidare solo quando `messages` cambia o `scrollback` cambia. Pre-wrap il testo all’arrivo del token.

### 3. Parsing Markdown ripetuto [Impatto medio-alto]
`src/markdown.rs:7-40`  
`render_markdown` fa `split_blocks`, `parse_inline` con iteratori su char_indices per ogni messaggio ad ogni frame. Il documento non cambia tra due tick se non arriva un nuovo token, ma viene riparseggiato.
*Fix*: cache della rappresentazione `Vec<Line>` per messaggio, invalidata su `push_token`.

### 4. `estimated_tokens` calcolato più volte per frame [Impatto medio]
`app.rs:679`
```rust
pub fn estimated_tokens(&self) -> usize {
    self.messages.iter().map(|m| m.content.len() / 4 + 4).sum::<usize>().max(8)
}
```
Chiamato in:
- `src/mode_bar.rs:23`
- `src/bottom.rs:36`
- `src/ui.rs:376` (context overlay)

O(N) tre volte per frame. Con 100+ messaggi è spreco.
*Fix*: memoizzare in `App` e aggiornare solo su `push_token`/nuovo messaggio; esporre `tokens_cached`.

### 5. Syscall per frame in bottom bar [Impatto basso-medio]
`src/bottom.rs:24-25`
```rust
let cwd = crate::app::current_dir_name();
let branch = crate::app::git_branch().unwrap_or_default();
```
`current_dir_name` chiama `std::env::current_dir()` ogni frame `src/app.rs:347`. `git_branch` è cache TTL 5 s, ok, ma `current_dir_name` no.
*Fix*: cache TTL anche per cwd.

### 6. Accumulo del buffer SSE e parsing JSON per riga [Impatto medio]
`src/agent.rs:150-200`  
`stream_turn` usa `String buf` che cresce con tutti i chunk e per ogni newline fa `serde_json::from_str`. Crea molte allocazioni intermedie.
*Fix*: parsing streaming con `bytes_stream` + decoder line-oriented; evitare `String::from_utf8_lossy` su ogni chunk.

### 7. Overlays che ricostruiscono vettori grandi ogni frame [Impatto medio]
`src/ui.rs:340-420` `render_metrics_overlay` e `src/ui.rs:470-560` `render_context_overlay` costruiscono `Vec<Line>` completo anche se l’overlay è chiuso. Il codice è eseguito solo se il flag è attivo, ma ancora alloca molto.
Migliorare con rendering differito o lazy.

---

## Code bloat / Architettura

### Conformità RULES.md
RULES.md: `Max 300 lines per file`. Conteggio attuale:
- `src/app.rs` 863 righe
- `src/handler.rs` 540 righe
- `src/ui.rs` 470 righe
- `src/agent.rs` 384 righe
- `src/main.rs` 319 righe

Tutti violano il limite.

*Fix*: spezzare per responsabilità:
- `app.rs` → `app/state.rs`, `app/metrics.rs`, `app/commands.rs`, `app/git_cache.rs`
- `handler.rs` → `handler/input.rs`, `handler/commands.rs`, `handler/provider.rs`
- `ui.rs` → `ui/chat.rs`, `ui/overlays/metrics.rs`, `ui/overlays/context.rs`, `ui/command_menu.rs`
- `agent.rs` → `agent/stream.rs`, `agent/tools.rs`

### Unwrap/Expect in produzione
RULES.md vieta `unwrap()/expect()`.
- `src/main.rs:53` `tokio::runtime::Runtime::new().expect("tokio runtime")`
- `src/main.rs:207` `app.pending_message.take().unwrap();`
- `src/handler.rs:320` `app.server_process.as_ref().unwrap().port`

*Fix*: gestire errori con `?` o valori di default loggati.

### Duplicazione codice
Query `/v1/status` duplicata in `src/main.rs:85-115` e `src/main.rs:170-205`. Stessa logica di parsing ruoli.
*Fix*: estrarre `async fn fetch_roles(client, endpoint) -> ...`.

### Dead / over-abstraction
`src/app.rs:apply_token` viene usata sia da `App::push_token` sia da `Agent`, ma la firma accetta riferimenti mutabili a `messages`, `in_thinking`, `thinking`. Il doppio mantenimento di stato di thinking tra App e Agent è confuso.
`src/tool_types.rs` contiene sia logica di policy che rendering: separare.

---

## Priorità raccomandata

1. Disegnare solo su cambiamento + ridurre poll a 33 ms idle
2. Cache del rendering history / markdown, invalidare su nuovi token
3. Memoizzare `estimated_tokens` e `context_pct`
4. Cache `current_dir_name`
5. Spezzare file >300 righe secondo RULES.md
6. Rimuovere `unwrap/expect` in produzione
7. Estrarre funzione condivisa per fetch ruoli server

Queste modifiche riducono allocazioni per frame, CPU idle e migliorano la manutenibilità senza cambiare funzionalità.
