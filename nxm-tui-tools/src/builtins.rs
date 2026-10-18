//! Built-in tool implementations: `read_file`, `list_resources`, `web_search`.
//!
//! Split out of `lib.rs` to respect the 300-line `RULES.md` cap (the registry
//! and selection logic live there). Re-exported from the crate root, so the
//! public API (`nxm_tui_tools::ReadFile`, etc.) is unchanged.

use std::path::PathBuf;

use anyhow::Result;
use serde_json::{json, Value};

use crate::{Tool, ToolResult};

/// Read the full contents of a file (`args.path`).
pub struct ReadFile;

impl Tool for ReadFile {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read the full contents of a file on disk."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Absolute or relative file path." }
            },
            "required": ["path"]
        })
    }
    fn run(&self, args: &Value) -> Result<ToolResult> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("`path` argument is required"))?;
        let content = std::fs::read_to_string(PathBuf::from(path))?;
        Ok(ToolResult::ok(content))
    }
}

/// List entries in a directory (`args.path`, default cwd).
pub struct ListResources;

impl Tool for ListResources {
    fn name(&self) -> &str {
        "list_resources"
    }
    fn description(&self) -> &str {
        "List files and directories in a path."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Directory to list; defaults to cwd." }
            },
            "required": []
        })
    }
    fn run(&self, args: &Value) -> Result<ToolResult> {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let mut names: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(PathBuf::from(path))? {
            let entry = entry?;
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        names.sort();
        Ok(ToolResult::ok(json!({ "entries": names }).to_string()))
    }
}

/// Web search via a configurable HTTP endpoint (Nexum engine / provider).
/// Endpoint from `NEXUM_SEARCH_ENDPOINT` (default `http://localhost:8080`).
pub struct WebSearch {
    endpoint: String,
}

impl WebSearch {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl Default for WebSearch {
    fn default() -> Self {
        let ep = std::env::var("NEXUM_SEARCH_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::new(ep)
    }
}

impl Tool for WebSearch {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "Search the web via a configured HTTP search endpoint."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query." }
            },
            "required": ["query"]
        })
    }
    fn run(&self, args: &Value) -> Result<ToolResult> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("`query` argument is required"))?;
        let mut url =
            reqwest::Url::parse(&format!("{}/search", self.endpoint.trim_end_matches('/')))?;
        url.query_pairs_mut().append_pair("q", query);
        let resp = reqwest::blocking::get(url.as_str())?;
        if !resp.status().is_success() {
            return Ok(ToolResult::err(format!(
                "search endpoint returned {}",
                resp.status()
            )));
        }
        let text = resp.text()?;
        Ok(ToolResult::ok(text))
    }
}
