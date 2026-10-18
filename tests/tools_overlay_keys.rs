//! Tools overlay keyboard tests (T1): cursor movement, Space toggle,
//! a/n bulk enable-disable, and Esc close — all routed through `handle_key`.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nxm_tui::app::App;
use nxm_tui::handler::handle_key;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn ctrl_e_opens_overlay_and_esc_closes_it() {
    let mut app = App::new();
    handle_key(&mut app, ctrl('e'));
    assert!(app.show_tools, "Ctrl+E opens the Tools overlay");
    handle_key(&mut app, key(KeyCode::Esc));
    assert!(!app.show_tools, "Esc closes the Tools overlay");
}

#[test]
fn arrow_keys_move_cursor_within_bounds() {
    let mut app = App::new();
    let len = app.tool_registry.tool_names().len();
    assert!(len > 0);
    handle_key(&mut app, ctrl('e'));

    // Up at the top wraps to the last row; Down advances and wraps to 0.
    handle_key(&mut app, key(KeyCode::Up));
    assert_eq!(app.tools_cursor, len - 1, "Up at top wraps to last");
    handle_key(&mut app, key(KeyCode::Down));
    assert_eq!(app.tools_cursor, 0, "Down from last wraps to first");
    handle_key(&mut app, key(KeyCode::Down));
    assert_eq!(app.tools_cursor, 1, "Down advances by one");
}

#[test]
fn space_toggles_the_tool_under_cursor() {
    let mut app = App::new();
    handle_key(&mut app, ctrl('e'));
    let name = app.tool_registry.tool_names()[app.tools_cursor].to_string();
    assert!(app.tool_registry.is_enabled(&name), "enabled by default");

    handle_key(&mut app, key(KeyCode::Char(' ')));
    assert!(!app.tool_registry.is_enabled(&name), "Space disables it");
    handle_key(&mut app, key(KeyCode::Char(' ')));
    assert!(app.tool_registry.is_enabled(&name), "Space re-enables it");
}

#[test]
fn a_and_n_bulk_enable_and_disable() {
    let mut app = App::new();
    handle_key(&mut app, ctrl('e'));

    handle_key(&mut app, key(KeyCode::Char('n')));
    let any_enabled = app
        .tool_registry
        .tool_names()
        .iter()
        .any(|n| app.tool_registry.is_enabled(n));
    assert!(!any_enabled, "n disables all tools");

    handle_key(&mut app, key(KeyCode::Char('a')));
    let all_enabled = app
        .tool_registry
        .tool_names()
        .iter()
        .all(|n| app.tool_registry.is_enabled(n));
    assert!(all_enabled, "a enables all tools");
}

#[test]
fn toggling_selection_filters_the_agent_request_body() {
    // End-to-end seam: overlay toggle -> App registry -> agent snapshot body.
    use nxm_tui::agent::Agent;
    use reqwest::Client;

    let mut app = App::new();
    handle_key(&mut app, ctrl('e'));
    handle_key(&mut app, key(KeyCode::Char('n'))); // disable all

    let client = Client::new();
    let agent = Agent::new(
        &client,
        "http://localhost:1",
        "test-model",
        None,
        Vec::new(),
        None,
        app.tool_registry.clone(),
    );
    let body = agent.request_body();
    assert!(
        body["tools"].as_array().map(|a| a.len()) == Some(0),
        "disabled tools absent from body, got {}",
        body["tools"]
    );
}
