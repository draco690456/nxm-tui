# D1 — What does the TUI actually DO?

- **Label:** `wayfinder:grilling`
- **Type:** Grilling (HITL) — needs the human
- **Status:** ✅ Resolved · Decided in chart session (2026-09-05)
- **Blocked by:** none

## Question

The destination says "a TUI for the Nexum inference engines modeled on
grok-build / kimi-code / gemini-cli." Those vendors are **different beasts**,
and the TUI's purpose fixes its entire scope. We need one answer:

> **What is the TUI's job — agentic REPL, inference harness, or something else?**

## Options considered
1. Agentic coding-assistant REPL (kimi-code / gemini-cli / codex): chat + tool calls + file editing.
2. Nexum inference harness (nxm-tui + engines.serve-*): model selection, live streaming, memory/workspace.
3. Thin dashboard / launcher.
4. Other.

## Resolution

- **Chosen:** **An agentic REPL terminal harness for the Nexum engines.**
  The TUI drives `engines.serve-*` over the **OpenAI-compatible HTTP API**: a
  chat pane + a tool-call/results pane, with model selection, live token
  streaming, and a memory/workspace panel. The user selected "harness per
  motori Nexum" **and** "agentic REPL" — the two compose into one product: an
  agentic coding terminal whose inference backend is Nexum (not an external
  LLM provider).
- **Rationale:** keeps the binary light/fast (no Tauri/web), reuses the
  OpenAI-compatible path `nxm-tui` already proves, and mirrors the
  agentic-CLI behaviour of `kimi-code`/`gemini-cli`/`codex` — but in pure
  Rust.
- **Sources:** user grilling (D1 questions), 2026-09-05; vendor behaviour
  refs: kimi-code (TS pnpm monorepo), gemini-cli (ink), codex/codex-rs (Rust
  ratatui); nxm-tui (in-house, OpenAI-compatible).

## Why this unblocks the map
Scope is fixed → D3/D4/streaming/turboquant tickets can now be scoped against
a concrete behaviour set (agentic REPL + Nexum engines via HTTP).
