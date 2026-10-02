# nxm-tui

Terminal UI client for NXM inference servers. Built with ratatui + crossterm.

## Purpose
Connects to any OpenAI-compatible LLM server (including NXM engines) and provides
an interactive chat interface in the terminal.

## Features
- Streaming chat responses (SSE)
- Markdown rendering in terminal
- Session history and persistence
- Server connection management
- Autocomplete for commands
- Configurable via TOML
- Reasoning overlay (Ctrl+R) with live thinking stream
- Tool detail overlay (Ctrl+O) with last invocation args + output
- Tool approval gating (Y/N overlay; only list_resources auto-approved)
- Risk badges in approval (sensitive path / network egress / unknown tool)
- History auto-follow + scrollback (Up/Down/PgUp/PgDn)
- Overlay render tests on TestBackend (no TTY)
- Per-token metrics (tokens/sec now measured, Ctrl+M overlay)
- Draw-on-change + 30 Hz idle polling (no busy redraw at 60 Hz)
- Per-message history render cache (streaming re-renders one message, not O(N))
- MCP host subsystem (`src/mcp/`): client/host that spawns an MCP server via
  stdio, declares the `sampling` capability, and serves `sampling/createMessage`
  by bridging to the LLM backend (auto-approved). Unblocks server tools like
  `generate_handoff`. See `docs/mcp/ARCHITECTURE.md`. Wired into the `main.rs`
  runtime at startup via `mcp::connect_from_config` (clean skip when disabled or
  the server binary is absent; chat never blocked).

## Dependencies
- ratatui + crossterm for TUI rendering
- reqwest for HTTP/SSE streaming
- tokio for async runtime
- Builds as bin + lib (`src/lib.rs`): the module graph is declared once and
  shared between the binary and the headless tests in `tests/`

## NOT included
- Inference logic (connects to external server)
- Model loading (that's the engine's job)
