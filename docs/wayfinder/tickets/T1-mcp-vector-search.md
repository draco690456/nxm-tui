# T1 — Restore MCP vector search (qdrant + memexd)

- **Label:** `wayfinder:task`
- **Type:** Task (HITL, partly AFK)
- **Blocks:** R1b/R2b/R3b re-runs of semantic vendor interrogation
- **Status:** Open · Unclaimed

## Question / What must happen

`workspace-qdrant-mcp`'s `list`/`retrieve` work, but its **semantic
`grep`/`search` need the qdrant+memexd vector stack**, which is not serving in
this session:

- `qdrant` HTTP :6333 and gRPC :6334 are **closed** in this sandbox.
- `memexd` (launch-agent, holds control port :7799) is alive but its gRPC
  :50051 is **unmapped** — it can't reach qdrant, so it hasn't started.
- `wqm project status nxm` → **Inactive** (3,746 docs indexed but not served).
- Sandboxed shells can't see qdrant's real storage
  (`/Users/devdanelle/.local/share/workspace-qdrant`), so a sandboxed
  `qdrant`/`memexd` can't load the indexed payloads.

## Done so far (diagnosis)
1. Built a stdio MCP client; verified `initialize` + `tools/list` + `list`
   (returned nexum components incl. `tui`).
2. `wqm service install` installed the memexd launch agent
   (`com.workspace-qdrant.memexd.plist`).
3. Started `qdrant` (HTTP :6333 / gRPC :6334 came up with collections
   `projects/libraries/rules/images/scratchpad`, but **0 points** from a
   sandboxed instance — real storage is outside the sandbox).
4. memexd is alive but Inactive/not serving gRPC.

## Acceptance criteria
- `grep "ratatui"` and `search "grok-build"` via MCP return real matches
  (not "daemon not reachable" / 0 results).
- The `nxm` project (and ideally the `vendors`) are Active & indexed.

## Proposed next steps
- `wqm service restart` (or `launchctl unload/load` the memexd plist) so the
  **launch-agent** memexd (full FS access) starts qdrant with the real
  `~/.local/share/workspace-qdrant` storage and binds :50051.
- Then `wqm project activate nxm` and re-run the vendor interrogtion from R1
  / R2 / R3 as reproducible MCP queries.
- If the vendors aren't a registered project/library, register
  `venum/vendors` (the `vendors/` tree) so `grok-build`, `kimi-code`,
  `gemini-cli`, `codex`, `llmfit-tui`, `openviking` are searchable by name.

## Resulting facts (to re-derive via MCP once healthy)
- ratatui/crossterm usage across Rust vendors (codex, llmfit-tui, openviking,
  pmetal, uzu, hindsight, mistral-rs).
- grok-build pager-render crate dependencies (unconfirmed under current
  sandbox).
- gemini-cli / kimi-code rendering approach (ink reference).
