//! Per-provider API key resolution via keychain + env fallback.
//!
//! Resolution order (D7 hard):
//! 1. Keychain entry `nexum-tui` / `<provider name>` (via `spawn_blocking`)
//! 2. Provider's `api_key_env` (e.g. `NVIDIA_API_KEY`)
//! 3. Global `NEXUM_API_KEY`
//! 4. Global `OPENAI_API_KEY`
//! 5. Missing
//!
//! Keychain errors `NoDefaultStore`/`PlatformFailure`/`NoStorageAccess` =
//! keychain unavailable → env fallback. `NoEntry` = no entry → env fallback.
//! `Error` is `#[non_exhaustive]` → always wildcard branch.
//!
//! mai-log RIGID: key value NEVER logged/printed (overlay, debug log, test output).
//! Logs emit only: source, presence, env var name.

use std::env;

use keyring::Entry;
use tracing::{debug, info, warn};

use crate::provider::Provider;

/// Result of key resolution for a provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyResolution {
    /// Key found in keychain.
    Keychain(String),
    /// Key found in environment variable. `keychain_available` = whether the
    /// keychain was reachable at all (false on NoDefaultStore / PlatformFailure /
    /// NoStorageAccess).
    Env { key: String, keychain_available: bool },
    /// No key found anywhere.
    Missing,
}

impl KeyResolution {
    /// Consume and return the key string if present.
    pub fn into_key(self) -> Option<String> {
        match self {
            KeyResolution::Keychain(key) | KeyResolution::Env { key, .. } => Some(key),
            KeyResolution::Missing => None,
        }
    }

    /// Whether a key was found (any source).
    pub fn is_some(&self) -> bool {
        !matches!(self, KeyResolution::Missing)
    }

    /// Source label for status/logging (never the value).
    pub fn source(&self) -> &'static str {
        match self {
            KeyResolution::Keychain(_) => "keychain",
            KeyResolution::Env { .. } => "env",
            KeyResolution::Missing => "missing",
        }
    }

    /// Env var name if source is Env, for one-time warning.
    pub fn env_var(&self) -> Option<&str> {
        match self {
            KeyResolution::Env { key: _, keychain_available: _ } => None, // would need to track which env
            _ => None,
        }
    }
}

/// Which source to try first. In `EnvFirst` the OS keychain is never touched,
/// so no password prompt appears — the dev-friendly path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySourceMode {
    /// Read keys from the environment only; never touch the keychain.
    EnvFirst,
    /// Try the keychain first, then env (production default, D7).
    KeychainFirst,
}

/// Decide the key-source mode from an explicit override and whether a local
/// `.env` is present. Pure (no I/O) so it is unit-testable.
///
/// - `NXM_KEY_SOURCE=env`      → always [`KeySourceMode::EnvFirst`]
/// - `NXM_KEY_SOURCE=keychain` → always [`KeySourceMode::KeychainFirst`]
/// - `NXM_KEY_SOURCE=auto` or unset → `EnvFirst` iff a `.env` is present,
///   otherwise `KeychainFirst`.
///
/// # Examples
///
/// ```rust
/// use nxm_tui::keys::{key_source_mode, KeySourceMode};
///
/// // A local .env (dev) skips the keychain — no password prompt.
/// assert_eq!(key_source_mode(true, None), KeySourceMode::EnvFirst);
/// // No .env, no override: production default.
/// assert_eq!(key_source_mode(false, None), KeySourceMode::KeychainFirst);
/// // Explicit override always wins.
/// assert_eq!(key_source_mode(false, Some("env")), KeySourceMode::EnvFirst);
/// assert_eq!(key_source_mode(true, Some("keychain")), KeySourceMode::KeychainFirst);
/// ```
pub fn key_source_mode(env_present: bool, override_var: Option<&str>) -> KeySourceMode {
    match override_var.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("env") => KeySourceMode::EnvFirst,
        Some("keychain") => KeySourceMode::KeychainFirst,
        // "auto", unset, or anything else: infer from the .env presence.
        _ => {
            if env_present {
                KeySourceMode::EnvFirst
            } else {
                KeySourceMode::KeychainFirst
            }
        }
    }
}

/// Resolve the API key for a provider.
///
/// Called ONCE at Agent spawn (never in the render loop). Keychain access runs
/// on `spawn_blocking` from the caller (`main.rs`).
///
/// In dev mode (a local `.env` is present or `NXM_KEY_SOURCE=env`) the keychain
/// is skipped entirely, so macOS never shows a password prompt.
pub fn resolve_key(provider: &Provider) -> KeyResolution {
    let env_present = std::path::Path::new(".env").exists();
    let override_var = env::var("NXM_KEY_SOURCE").ok();
    let mode = key_source_mode(env_present, override_var.as_deref());

    if mode == KeySourceMode::EnvFirst {
        debug!(target: "nexum::keys", provider = %provider.name, "env-first mode: skipping keychain (no prompt)");
        // keychain_available is reported true here: it was not *unavailable*,
        // we deliberately chose not to consult it. No one-time warning needed.
        return resolve_env_fallback(provider, true);
    }

    // 1. Keychain (production default)
    let entry = match Entry::new("nexum-tui", &provider.name) {
        Ok(e) => e,
        Err(e) => {
            warn!(target: "nexum::keys", provider = %provider.name, error = %e, "keychain entry creation failed");
            return resolve_env_fallback(provider, false);
        }
    };
    match entry.get_password() {
        Ok(key) => {
            info!(target: "nexum::keys", provider = %provider.name, source = "keychain", "key resolved");
            return KeyResolution::Keychain(key);
        }
        Err(keyring::Error::NoEntry) => {
            debug!(target: "nexum::keys", provider = %provider.name, "no keychain entry");
        }
        Err(keyring::Error::NoDefaultStore) |
        Err(keyring::Error::PlatformFailure(_)) |
        Err(keyring::Error::NoStorageAccess(_)) => {
            warn!(target: "nexum::keys", provider = %provider.name, "keychain unavailable");
            return resolve_env_fallback(provider, false);
        }
        Err(e) => {
            // keyring::Error is #[non_exhaustive] — wildcard catch-all.
            warn!(target: "nexum::keys", provider = %provider.name, error = %e, "keychain error");
            return resolve_env_fallback(provider, false);
        }
    }

    // 2-4. Env fallback (keychain was available but no entry)
    resolve_env_fallback(provider, true)
}

fn resolve_env_fallback(provider: &Provider, keychain_available: bool) -> KeyResolution {
    // Provider-specific env var
    if let Some(ref env_var) = provider.api_key_env {
        if let Ok(key) = env::var(env_var) {
            if !key.is_empty() {
                info!(target: "nexum::keys", provider = %provider.name, source = "env", env_var = %env_var, "key resolved");
                return KeyResolution::Env { key, keychain_available };
            }
        }
    }

    // Global NEXUM_API_KEY
    if let Ok(key) = env::var("NEXUM_API_KEY") {
        if !key.is_empty() {
            info!(target: "nexum::keys", provider = %provider.name, source = "env", env_var = "NEXUM_API_KEY", "key resolved");
            return KeyResolution::Env { key, keychain_available };
        }
    }

    // Global OPENAI_API_KEY
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            info!(target: "nexum::keys", provider = %provider.name, source = "env", env_var = "OPENAI_API_KEY", "key resolved");
            return KeyResolution::Env { key, keychain_available };
        }
    }

    info!(target: "nexum::keys", provider = %provider.name, source = "missing", "no key found");
    KeyResolution::Missing
}
