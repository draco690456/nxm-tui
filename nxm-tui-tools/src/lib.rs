//! `nxm-tui-tools` — pure-Rust actuator set for the Nexum agentic TUI.
//!
//! A single crate holding the tools the OpenAI agent calls (`web_search`,
//! `read_file`, `list_resources`). `nxm-tui`'s `Agent` (src/agent.rs) depends on
//! this crate's [`Tool`] trait + [`Registry`] (P2-proper wiring). v0.1 scaffold:
//! blocking I/O is intentional — P2-proper makes `Tool::run` async.
//!
//! Run: `cargo test --workspace` (lib + integration tests under `tests/`).

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use serde_json::{json, Value};
use tracing::info;

mod builtins;
mod grep;
mod search_replace;
mod ssrf;
pub use builtins::{ListResources, ReadFile, WebFetch, WebSearch};
pub use grep::Grep;
pub use search_replace::SearchReplace;
pub use ssrf::{
    fetch_with_ssrf_guard, is_allowed_scheme, is_blocked_addr, validate_host, validate_url,
};

/// Tracing target per `RULES.md` (`nexum::<module>::<feature>`).
pub const TARGET: &str = "nexum::tools::runtime";

/// Outcome of a tool run, returned to the model as a `role: "tool"` result.
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Whether the tool succeeded.
    pub ok: bool,
    /// Text content handed back to the model.
    pub content: String,
}

impl ToolResult {
    pub fn ok(content: String) -> Self {
        Self { ok: true, content }
    }
    pub fn err(msg: String) -> Self {
        Self {
            ok: false,
            content: msg,
        }
    }
}

/// A tool callable by the OpenAI agent. Implementations are stateless and
/// `Send + Sync` so the [`Registry`] can hold `Arc<dyn Tool>`.
pub trait Tool: Send + Sync {
    /// Canonical, kebab-case name the model calls (e.g. `read_file`).
    fn name(&self) -> &str;
    /// Human-readable description given to the model.
    fn description(&self) -> &str;
    /// JSON schema (`function.parameters`) for the tool's arguments.
    fn schema(&self) -> Value;
    /// Execute the tool. Synchronous in v0.1 (P2-proper: make `async`).
    fn run(&self, args: &Value) -> Result<ToolResult>;
}

/// Registry of available tools + OpenAI manifest builder.
///
/// Holds the T1 in-memory selection: every registered tool is **enabled** by
/// default, and [`manifest_selected`](Registry::manifest_selected) is the
/// model-visible subset. Selection is orthogonal to approval gating — a
/// selected tool can still be gated in `tool_types::needs_approval`.
#[derive(Clone)]
pub struct Registry {
    tools: Vec<Arc<dyn Tool>>,
    /// Names of the tools currently sent to the model (T1 selection).
    enabled: HashSet<String>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            enabled: HashSet::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        info!(target: TARGET, "register tool `{name}`", name = tool.name());
        // Invariant: registering a tool always (re-)enables it.
        self.enabled.insert(tool.name().to_string());
        self.tools.push(Arc::new(tool));
    }

    /// Enable or disable a tool for the model-visible manifest (T1).
    ///
    /// ```
    /// use nxm_tui_tools::default_registry;
    ///
    /// let mut reg = default_registry();
    /// reg.set_enabled("web_search", false);
    /// assert!(!reg.is_enabled("web_search"));
    /// ```
    pub fn set_enabled(&mut self, name: &str, on: bool) {
        tracing::debug!(target: TARGET, "set tool `{name}` enabled={on}");
        if on {
            self.enabled.insert(name.to_string());
        } else {
            self.enabled.remove(name);
        }
    }

    /// Whether a tool is part of the model-visible selection.
    ///
    /// ```
    /// use nxm_tui_tools::default_registry;
    ///
    /// assert!(default_registry().is_enabled("read_file"));
    /// ```
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled.contains(name)
    }

    /// Enable every registered tool (restore the full selection).
    ///
    /// ```
    /// use nxm_tui_tools::default_registry;
    ///
    /// let mut reg = default_registry();
    /// reg.disable_all();
    /// reg.enable_all();
    /// assert_eq!(reg.manifest_selected(), reg.manifest());
    /// ```
    pub fn enable_all(&mut self) {
        tracing::debug!(target: TARGET, "enable all tools");
        self.enabled = self.tools.iter().map(|t| t.name().to_string()).collect();
    }

    /// Disable every registered tool (the model sees an empty `tools` list).
    ///
    /// ```
    /// use nxm_tui_tools::default_registry;
    ///
    /// let mut reg = default_registry();
    /// reg.disable_all();
    /// assert!(reg.manifest_selected().is_empty());
    /// ```
    pub fn disable_all(&mut self) {
        tracing::debug!(target: TARGET, "disable all tools");
        self.enabled.clear();
    }

    /// Registered tool names, in registration order (for the Tools overlay UI).
    ///
    /// ```
    /// use nxm_tui_tools::Registry;
    ///
    /// let mut reg = Registry::new();
    /// reg.register(nxm_tui_tools::ReadFile);
    /// assert_eq!(reg.tool_names(), vec!["read_file"]);
    /// ```
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.iter().map(|t| t.name()).collect()
    }

    /// Look up a tool by name (clones the `Arc`, no cloning of the tool itself).
    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.iter().find(|t| t.name() == name).cloned()
    }

    /// Execute a named tool. Returns an `Err` only if the tool is unknown or
    /// the binary throws; tool-level failures come back as `ToolResult::err`.
    pub fn call(&self, name: &str, args: &Value) -> Result<ToolResult> {
        let tool = self
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown tool `{name}`"))?;
        tool.run(args)
    }

    /// Build the OpenAI `tools` manifest sent in the chat request.
    pub fn manifest(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|t| manifest_entry(t.as_ref()))
            .collect()
    }

    /// Like [`manifest`](Registry::manifest) but filtered to the enabled
    /// subset — what the agent puts in the request's `tools` body (T1).
    ///
    /// ```
    /// use nxm_tui_tools::default_registry;
    ///
    /// let mut reg = default_registry();
    /// reg.set_enabled("web_search", false);
    /// let selected = reg.manifest_selected();
    /// let names: Vec<&str> = selected
    ///     .iter()
    ///     .filter_map(|v| v["function"]["name"].as_str())
    ///     .collect();
    /// assert!(!names.contains(&"web_search"));
    /// ```
    pub fn manifest_selected(&self) -> Vec<Value> {
        self.tools
            .iter()
            .filter(|t| self.is_enabled(t.name()))
            .map(|t| manifest_entry(t.as_ref()))
            .collect()
    }
}

/// One OpenAI `tools` entry for `t` — shared by [`Registry::manifest`] and
/// [`Registry::manifest_selected`].
fn manifest_entry(t: &dyn Tool) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": t.name(),
            "description": t.description(),
            "parameters": t.schema(),
        }
    })
}

/// Build the default MVP registry (read_file, list_resources, web_search).
pub fn default_registry() -> Registry {
    let mut reg = Registry::new();
    reg.register(ReadFile);
    reg.register(ListResources);
    reg.register(WebSearch::default());
    reg.register(WebFetch);
    reg.register(Grep);
    reg.register(SearchReplace);
    reg
}
