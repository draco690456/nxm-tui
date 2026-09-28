# Vendor interrogation — findings (resolves R1, R2, R3)

> **Method.** `workspace-qdrant-mcp` (stdio, read-only, workspace-scoped) is
> reachable: `initialize`, `tools/list`, and `list` succeed and return the
> indexed `nexum` project (`projectId: 15c8ae702621`, `projectPath:
> /Users/devdanelle/Projects/nexum`, **3,746 docs**). However the **vector
> backend (qdrant :6333/:6334 + memexd gRPC :50051) is not serving in this
> session** — `grep`/`search`/`retrieve` return 0 (qdrant ports closed; nxm
> project "Inactive"). Full vector interrogtion is therefore blocked; see
> `tickets/T1-mcp-vector-search.md`.
>
> **Fallback used (reliable):** direct reads of the local vendor clones under
> `vendors/` (sandbox-allowed) + the working `list` tool. Reads were captured
> before the sandbox path-allowlist tightened mid-session; later direct reads
> of identical paths returned `No such file or directory` (see §0).

## 0. Environment caveat
- `/Users/devdanelle/.local/bin/` (PATH) holds the installed agent CLIs:
  `grok`, `codex`, `kiro-cli`, `kiro-cli-chat`, `kiro-cli-term`, `kimchi`,
  `cursor-agent`, `qodercli`, `claude`, `kuro`, `muse`, plus the toolchain
  `qdrant`, `memexd`, `workspace-qdrant-mcp`, `nxm-mcp-server`, `wqm`, `uv`.
- Absolute paths under `/Users/devdanelle/...` are intermittently
  **sandboxed-as-ENOENT**; only the working root (`nxm-harness/.`) is
  stably readable via relative paths in later shells.

## R1 — grok-build (the Rust-native harness reference)
- **Language:** pure **Rust** (NOT TypeScript) — a Zed/xAI lineage workspace
  (`xai-grok-*` crates).
- **Structure:** workspace root `Cargo.toml` (auto-generated) with
  `[patch.crates-io] async-openai = { git = "https://github.com/our-forks/async-openai.git", rev = "95b52eb..." }`
  — i.e. an agentic CLI talking to an **OpenAI-compatible API** (exactly what
  the target TUI needs). Members under `crates/`: `build/`, `codegen/`,
  `common/`.
- **Rendering:** `codegen/` ships the `xai-grok-pager-*` family
  (`xai-grok-pager`, `xai-grok-pager-bin`, `xai-grok-pager-minimal`,
  `xai-grok-pager-pty-harness`, `xai-grok-pager-render`) — a native Rust
  terminal pager/renderer (custom VT-style, **not** ratatui; unconfirmed
  whether it depends on `linecf`/`term`/`viper`/`sgr`). The `*-pty-harness`
  crate signals PTY spawning for subprocess I/O — a key behavioural reference
  for an agentic terminal harness.
- **Verdict:** the strongest "harness like ours" reference (Rust + agentic +
  OpenAI-compatible + paging). Adopt its **architecture** (multi-crate,
  agent-lifecycle → render → pager) and **behaviour** (streaming, pty), but
  **not** its custom pager (we standardise on ratatui — see R2).

## R2 — ratatui-based Rust TUIs (the stack decision)
`grep -rl --include=Cargo.toml "ratatui|crossterm|..." vendors/` matched:
`grok-build` (custom pager, not ratatui — see R1), `llmfit-tui`, `openviking`
(`ov_cli`), `codex` (`codex-rs`), `hindsight` (`hindsight-cli`),
`mistral-rs` (`mistralrs-cli`), `pmetal`, `uzu`.

Confirmed deps (direct `Cargo.toml` reads):
| Vendor | ratatui | crossterm | tokio | clap | Notes |
|---|---|---|---|---|---|
| `llmfit-tui` | 0.30 | 0.29 | 1.52 (rt-multi-thread,signal,net,io-std) | 4.6 derive,env | "Interactive TUI to match models against RAM/CPU/GPU" — hardware-aware, like harness-core scoring |
| `openviking` (`ov_cli`) | 0.29 | 0.28 | 1.38 (full) | 4.5 derive,env | |
| `nxm-tui` (in-house) | 0.29 | 0.28 | 1 (+macros,rt-multi-thread) | — | pure Rust; **ratatui+crossterm+tokio+reqwest**; "connects to any OpenAI-compatible LLM server" — **anchor reference** |
| `codex` (`codex-cli`) | (matched) | (matched) | | | exact version unconfirmed (path shifted mid-session) |

**Decision locked:** `ratatui` + `crossterm` + `tokio` + `clap`. ratatui
0.29–0.30 / crossterm 0.28–0.29 — pick the latest stable pair (0.30 / 0.29)
unless `nxm-tui` version-pinning (0.29/0.28) wins for consistency. No
competing pure-Rust TUI crate surfaced. Behavioural patterns to borrow:
async token streaming (tokio channels → `Frame` redraw), virtualised lists
(`llmfit-tui` models list), status bar / input box layout.

## R3 — agentic-CLI TUI behaviour (TS references, behavioural only)
- **`kimi-code`**: TypeScript / Node, pnpm monorepo, `apps/kimi-code`. Agentic
  coding CLI. (TS → not adopted as implementation.)
- **`gemini-cli`**: TypeScript / Node; renders via **`ink`**
  (`npm:@jrichman/ink@6.6.9`) — React-for-the-terminal. (TS+ink → not adopted;
  behavioural reference for structured streaming output.)
- **System CLIs present (PATH):** `grok`, `codex`, `kiro-cli`, `cursor-agent`,
  `qodercli`, `claude`, `kimchi` (kimi-code), `kuro`, `muse` — most are
  TS/Go; the Rust ones (`grok`, `codex`) are already covered in R1/R2.
- **Behavioural patterns (language-agnostic) to mirror in the pure-Rust TUI:**
  interleaved assistant/user/tool message streaming; multi-pane chat | tools |
  results | status; command palette / `/`-commands; markdown rendering; pty
  spawning for interactive subprocesses; clipboard + keyboard shortcuts;
  scrollable virtualised buffers.

## Resolution
- **R1 locked:** grok-build is the Rust-native harness reference; adopt its
  architecture/behaviour, not its custom pager.
- **R2 locked:** ratatui + crossterm (+ tokio + clap); version choice deferred
  to P1 (pin to nxm-tui's 0.29/0.28 for consistency, or go latest 0.30/0.29).
- **R3 locked (behaviour only):** mirror the agentic-CLI TUI behaviour
  (streaming, multi-pane, pty, command palette); do NOT adopt TS/ink.
