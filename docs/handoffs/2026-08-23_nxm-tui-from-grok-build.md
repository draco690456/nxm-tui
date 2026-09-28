# Handoff: nxm-tui rebuild from grok-build

> Decisione presa: ricostruire nxm-tui partendo da grok-build come base, non come cherry-pick incrementale.

**Created**: 2026-08-23 23:11
**Source session**: kiro-cli session (migration plan + tauri setup)
**Target repo**: nxm-ai/nxm-tui
**Priority**: high

---

## Context

L'utente vuole un TUI professionale per NXM. L'attuale nxm-tui (20 file, basico) funziona ma è minimale.
Grok-build (SpaceXAI, Apache 2.0, 25.9k stars) è un coding agent TUI in Rust — architettura identica a quello che ci serve.

La decisione è: **copiare i crates da grok-build in nxm-tui, rinominare, rimuovere ciò che non serve, collegare al nostro backend**. Licenza Apache 2.0 lo permette (basta mantenere NOTICE).

## Current state

- `nxm-tui/` attuale: 20 file Rust, ratatui + crossterm, connessione OpenAI-compat, compila
- `vendors/grok-build/`: 82MB, ~50 crates, TUI + agent runtime + tools + workspace

## Exact resume point

Bisogna:
1. Analizzare in profondità i crates di grok-build (quali servono, quali no)
2. Creare un piano di integrazione (non cherry-pick, ma fork-and-adapt)
3. Eseguire il piano step by step

## Next actions (in order)

1. **Creare un piano** in `nxm-projects/plans/active/2026-08-24_nxm-tui-from-grok-build.md`
2. Il piano deve mappare: crates grok-build → cosa diventa in nxm-tui
3. Identificare cosa buttiamo (xAI auth, protobuf, grok-specific)
4. Identificare cosa teniamo (TUI pager, tools, workspace, MCP)
5. Identificare cosa adattiamo (backend → OpenAI-compat via nxm-shared)

## Files to read first

- `vendors/grok-build/Cargo.toml` — workspace structure
- `vendors/grok-build/crates/codegen/xai-grok-pager/` — il TUI core
- `vendors/grok-build/crates/codegen/xai-grok-shell/` — agent runtime
- `vendors/grok-build/crates/codegen/xai-grok-tools/` — tool implementations
- `vendors/grok-build/crates/codegen/xai-grok-workspace/` — filesystem/VCS

## Commands to verify state

```bash
cd ~/Projects/nexum/vendors/grok-build
find crates -name "Cargo.toml" | wc -l  # numero di crates
cat Cargo.toml | head -50               # workspace overview
```

---

## When consumed

**Picked up by**: (next session)
**Date**: 
**Outcome**: 
