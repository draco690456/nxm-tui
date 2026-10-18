//! Agent request-body tests (T1 DoD): the `tools` payload sent to the model
//! is `registry.manifest_selected()` — a disabled tool must NOT appear in the
//! request body.

use nxm_tui::agent::Agent;
use nxm_tui_tools::{default_registry, Registry};
use reqwest::Client;

/// Build an agent over a caller-supplied registry (the T1 seam: `App` owns
/// the selection and hands a snapshot to the agent at spawn).
fn agent_over(reg: Registry, client: &Client) -> Agent<'_> {
    Agent::new(
        client,
        "http://localhost:1",
        "test-model",
        None,
        Vec::new(),
        None,
        reg,
    )
}

fn tool_names(body: &serde_json::Value) -> Vec<String> {
    body["tools"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|t| t["function"]["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn default_registry_body_lists_every_tool() {
    let client = Client::new();
    let agent = agent_over(default_registry(), &client);
    let names = tool_names(&agent.request_body());
    assert_eq!(names.len(), 3);
    assert!(names.iter().any(|n| n == "read_file"));
    assert!(names.iter().any(|n| n == "list_resources"));
    assert!(names.iter().any(|n| n == "web_search"));
}

#[test]
fn disabled_tool_is_absent_from_the_request_body() {
    let mut reg = default_registry();
    reg.set_enabled("web_search", false);
    let client = Client::new();
    let agent = agent_over(reg, &client);

    let body = agent.request_body();
    let names = tool_names(&body);
    assert!(
        !names.iter().any(|n| n == "web_search"),
        "disabled tool must not appear in body.tools, got {names:?}"
    );
    assert!(names.iter().any(|n| n == "read_file"));
    assert!(names.iter().any(|n| n == "list_resources"));
}

#[test]
fn all_tools_disabled_sends_an_empty_tools_array() {
    let mut reg = default_registry();
    reg.disable_all();
    let client = Client::new();
    let agent = agent_over(reg, &client);
    let names = tool_names(&agent.request_body());
    assert!(names.is_empty(), "empty selection, got {names:?}");
}
