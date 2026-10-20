//! Selection tests for `Registry` (T1): in-memory `enabled` state + the
//! filtered `manifest_selected()` the agent sends to the model.
//!
//! Contract: every registered tool is enabled by default (behaviour unchanged
//! until the user disables one); `manifest()` stays the full compat manifest;
//! `manifest_selected()` is the model-visible subset.

use nxm_tui_tools::{default_registry, Registry};

#[test]
fn registry_defaults_all_tools_enabled() {
    let reg = default_registry();
    let names = reg.tool_names();
    assert!(!names.is_empty(), "default registry has tools");
    for name in names {
        assert!(reg.is_enabled(name), "`{name}` enabled by default");
    }
    // Default selection == full manifest (no behaviour change until toggled).
    assert_eq!(reg.manifest_selected(), reg.manifest());
}

#[test]
fn set_enabled_false_excludes_tool_from_selected_manifest() {
    let mut reg = default_registry();
    reg.set_enabled("web_search", false);

    assert!(!reg.is_enabled("web_search"));
    let selected: Vec<String> = reg
        .manifest_selected()
        .iter()
        .map(|v| {
            v["function"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        })
        .collect();
    assert!(!selected.contains(&"web_search".to_string()));
    assert!(selected.contains(&"read_file".to_string()));
    assert!(selected.contains(&"list_resources".to_string()));
    // Compat: the unfiltered manifest still lists everything.
    assert_eq!(reg.manifest().len(), 6);
}

#[test]
fn set_enabled_true_re_enables_a_disabled_tool() {
    let mut reg = default_registry();
    reg.set_enabled("read_file", false);
    assert!(!reg.is_enabled("read_file"));
    reg.set_enabled("read_file", true);
    assert!(reg.is_enabled("read_file"));
    assert_eq!(reg.manifest_selected(), reg.manifest());
}

#[test]
fn disable_all_yields_empty_selected_manifest() {
    let mut reg = default_registry();
    reg.disable_all();
    assert!(reg.manifest_selected().is_empty());
    // Compat manifest unaffected by the selection.
    assert_eq!(reg.manifest().len(), 6);
}

#[test]
fn enable_all_restores_full_selected_manifest() {
    let mut reg = default_registry();
    reg.disable_all();
    reg.enable_all();
    assert_eq!(reg.manifest_selected(), reg.manifest());
    for name in reg.tool_names() {
        assert!(reg.is_enabled(name));
    }
}

#[test]
fn tool_names_lists_registered_tools_in_registration_order() {
    let mut reg = Registry::new();
    reg.register(nxm_tui_tools::ReadFile);
    reg.register(nxm_tui_tools::ListResources);
    assert_eq!(reg.tool_names(), vec!["read_file", "list_resources"]);
}

#[test]
fn registering_a_new_tool_defaults_it_to_enabled() {
    let mut reg = Registry::new();
    reg.register(nxm_tui_tools::ReadFile);
    reg.set_enabled("read_file", false);
    // A tool registered after a disable_all (or individually disabled set)
    // still comes up enabled: registration always inserts the name.
    reg.register(nxm_tui_tools::ListResources);
    assert!(reg.is_enabled("list_resources"));
}
