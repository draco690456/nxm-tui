//! Tests for `keys`: key-source mode selection and `resolve_key` resolution.
//!
//! Relocated out of `src/keys.rs` (RULES.md: no inline `#[cfg(test)]`). The
//! keychain-backed cases force `NXM_KEY_SOURCE=keychain` so they exercise the
//! keychain path deterministically regardless of whether a `.env` exists in
//! the working dir; the env-first cases force `NXM_KEY_SOURCE=env` so they
//! never touch the mock store.

use keyring_core::mock::{Cred, Store};
use keyring_core::{set_default_store, Error};
use nxm_tui::keys::{key_source_mode, resolve_key, KeyResolution, KeySourceMode};
use nxm_tui::provider::Provider;
use std::sync::Mutex;

/// `resolve_key` reads process-global state (`NXM_KEY_SOURCE` env + the shared
/// mock keychain store). Tests that mutate it must not run concurrently, or
/// they clobber each other. This mutex serializes them; `key_source_mode`
/// (pure) tests don't need it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Install a fresh mock keychain store for the process. Idempotent-ish: later
/// calls are no-ops on the already-set default store.
fn setup_mock() {
    if let Ok(store) = Store::new() {
        set_default_store(store);
    }
}

fn provider(name: &str, env_var: &str) -> Provider {
    Provider {
        name: name.into(),
        base_url: "http://localhost".into(),
        requires_api_key: true,
        api_key_env: Some(env_var.into()),
        default_model: None,
        is_cloud: true,
    }
}

// ---- key_source_mode (pure, no I/O) ----

#[test]
fn mode_env_present_defaults_to_env_first() {
    assert_eq!(key_source_mode(true, None), KeySourceMode::EnvFirst);
}

#[test]
fn mode_no_env_defaults_to_keychain_first() {
    assert_eq!(key_source_mode(false, None), KeySourceMode::KeychainFirst);
}

#[test]
fn mode_explicit_env_override_wins() {
    assert_eq!(key_source_mode(false, Some("env")), KeySourceMode::EnvFirst);
    assert_eq!(key_source_mode(false, Some("ENV")), KeySourceMode::EnvFirst);
}

#[test]
fn mode_explicit_keychain_override_wins() {
    assert_eq!(
        key_source_mode(true, Some("keychain")),
        KeySourceMode::KeychainFirst
    );
}

#[test]
fn mode_auto_or_unknown_falls_back_to_env_presence() {
    assert_eq!(key_source_mode(true, Some("auto")), KeySourceMode::EnvFirst);
    assert_eq!(
        key_source_mode(false, Some("garbage")),
        KeySourceMode::KeychainFirst
    );
}

// ---- resolve_key: env-first (dev) path — keychain never touched ----

#[test]
fn env_first_resolves_from_provider_env_without_keychain() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("NXM_KEY_SOURCE", "env");
    std::env::set_var("TEST_ENVFIRST_1", "env-key");
    let p = provider("EnvFirstProvider1", "TEST_ENVFIRST_1");

    let res = resolve_key(&p);
    match res {
        KeyResolution::Env { key, keychain_available } => {
            assert_eq!(key, "env-key");
            // env-first means we did not find the keychain *unavailable*; we
            // deliberately skipped it, so this is reported true.
            assert!(keychain_available);
        }
        other => panic!("expected Env, got {other:?}"),
    }
    std::env::remove_var("NXM_KEY_SOURCE");
    std::env::remove_var("TEST_ENVFIRST_1");
}

#[test]
fn env_first_missing_when_no_env_set() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("NXM_KEY_SOURCE", "env");
    std::env::remove_var("TEST_ENVFIRST_2");
    std::env::remove_var("NEXUM_API_KEY");
    std::env::remove_var("OPENAI_API_KEY");
    let p = provider("EnvFirstProvider2", "TEST_ENVFIRST_2");

    assert_eq!(resolve_key(&p), KeyResolution::Missing);
    std::env::remove_var("NXM_KEY_SOURCE");
}

// ---- resolve_key: keychain-first (production) path ----

// NOTE: the former `#[ignore]`d `keychain_beats_env` test lived here. It could
// not run because production `resolve_key` used `keyring::Entry` (native store),
// not the `keyring_core` mock these tests install — separate namespaces. The
// I1 `KeyStore` seam fixed this: `tests/keystore.rs::store_beats_env_when_present`
// is its deterministic, un-ignored successor, asserting "store beats env" via an
// injectable `MockKeyStore`.

#[test]
fn keychain_first_falls_back_to_env_on_no_entry() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    setup_mock();
    std::env::set_var("NXM_KEY_SOURCE", "keychain");
    std::env::set_var("TEST_KC_2", "env-key");
    let p = provider("KcProvider2_NoEntry", "TEST_KC_2");

    match resolve_key(&p) {
        KeyResolution::Env { key, keychain_available } => {
            assert_eq!(key, "env-key");
            assert!(keychain_available);
        }
        other => panic!("expected Env, got {other:?}"),
    }
    std::env::remove_var("NXM_KEY_SOURCE");
    std::env::remove_var("TEST_KC_2");
}

#[test]
fn keychain_unavailable_falls_back_to_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    setup_mock();
    std::env::set_var("NXM_KEY_SOURCE", "keychain");
    let entry = keyring_core::Entry::new("nexum-tui", "KcProvider3_Unavailable").unwrap();
    if let Some(cred) = entry.as_any().downcast_ref::<Cred>() {
        cred.set_error(Error::PlatformFailure("mock unavailable".into()));
    }
    std::env::set_var("TEST_KC_3", "env-key");
    let p = provider("KcProvider3_Unavailable", "TEST_KC_3");

    match resolve_key(&p) {
        KeyResolution::Env { key, keychain_available } => {
            assert_eq!(key, "env-key");
            assert!(!keychain_available);
        }
        other => panic!("expected Env with keychain_available=false, got {other:?}"),
    }
    std::env::remove_var("NXM_KEY_SOURCE");
    std::env::remove_var("TEST_KC_3");
}
