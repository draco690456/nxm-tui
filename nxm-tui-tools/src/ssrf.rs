//! SSRF protection for `web_fetch` — address classification and host validation.
//!
//! Blocks requests to non-routable/sensitive addresses (loopback, link-local,
//! private-use, metadata endpoint, unspecified) and non-HTTP(S) schemes.
//! Also validates redirect targets to prevent DNS rebinding.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

use anyhow::{Context, Result};
use reqwest::Url;
use tracing::{info, warn};

use crate::TARGET;

/// Check if an IP address is blocked by SSRF policy.
///
/// Returns `true` for:
/// - Loopback (127.0.0.0/8, ::1)
/// - Link-local (169.254.0.0/16, fe80::/10)
/// - Private-use (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, fc00::/7)
/// - Metadata endpoint (169.254.169.254)
/// - Unspecified (0.0.0.0, ::)
///
/// ```
/// use std::net::IpAddr;
/// use std::str::FromStr;
/// use nxm_tui_tools::is_blocked_addr;
///
/// assert!(is_blocked_addr(IpAddr::from_str("127.0.0.1").unwrap()));
/// assert!(is_blocked_addr(IpAddr::from_str("10.0.0.1").unwrap()));
/// assert!(is_blocked_addr(IpAddr::from_str("192.168.1.1").unwrap()));
/// assert!(is_blocked_addr(IpAddr::from_str("169.254.169.254").unwrap()));
/// assert!(!is_blocked_addr(IpAddr::from_str("93.184.216.34").unwrap())); // example.org
/// ```
pub fn is_blocked_addr(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => is_blocked_ipv4(ipv4),
        IpAddr::V6(ipv6) => is_blocked_ipv6(ipv6),
    }
}

fn is_blocked_ipv4(ip: Ipv4Addr) -> bool {
    // Loopback: 127.0.0.0/8
    if ip.is_loopback() {
        return true;
    }
    // Link-local: 169.254.0.0/16
    if ip.is_link_local() {
        return true;
    }
    // Private-use: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
    if ip.is_private() {
        return true;
    }
    // Unspecified: 0.0.0.0
    if ip.is_unspecified() {
        return true;
    }
    // Metadata endpoint: 169.254.169.254
    if ip == Ipv4Addr::new(169, 254, 169, 254) {
        return true;
    }
    false
}

fn is_blocked_ipv6(ip: Ipv6Addr) -> bool {
    // Loopback: ::1
    if ip.is_loopback() {
        return true;
    }
    // Link-local: fe80::/10
    if ip.is_unicast_link_local() {
        return true;
    }
    // ULA (private): fc00::/7
    if ip.segments()[0] & 0xfe00 == 0xfc00 {
        return true;
    }
    // Unspecified: ::
    if ip.is_unspecified() {
        return true;
    }
    false
}

/// Check if a URL scheme is allowed (only http/https).
pub fn is_allowed_scheme(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

/// Resolve a hostname to IP addresses and check if any is blocked.
///
/// Returns `Ok(())` if all resolved addresses are allowed.
/// Returns `Err` if resolution fails or any address is blocked.
pub fn validate_host(host: &str) -> Result<()> {
    let addrs: Vec<SocketAddr> = (host, 0)
        .to_socket_addrs()
        .with_context(|| format!("resolve host `{host}`"))?
        .collect();
    if addrs.is_empty() {
        anyhow::bail!("host `{host}` resolved to no addresses");
    }
    for addr in &addrs {
        if is_blocked_addr(addr.ip()) {
            warn!(
                target: TARGET,
                "SSRF block: host `{host}` resolves to blocked address `{}`",
                addr.ip()
            );
            anyhow::bail!("host `{host}` resolves to blocked address `{}`", addr.ip());
        }
    }
    Ok(())
}

/// Validate a URL for SSRF safety: scheme + host resolution.
///
/// Returns the validated URL if allowed, or an error describing the block.
pub fn validate_url(url_str: &str) -> Result<Url> {
    let url = Url::parse(url_str).context("parse URL")?;
    if !is_allowed_scheme(&url) {
        warn!(
            target: TARGET,
            "SSRF block: disallowed scheme `{}` for URL `{}`",
            url.scheme(),
            url_str
        );
        anyhow::bail!("scheme `{}` not allowed (only http/https)", url.scheme());
    }
    let host = url.host_str().context("URL has no host")?;
    validate_host(host).with_context(|| format!("validate host `{host}`"))?;
    Ok(url)
}

/// Maximum number of redirects to follow.
const MAX_REDIRECTS: usize = 10;

/// Perform a GET request with SSRF protection on redirects.
///
/// Uses a custom redirect policy that validates each hop's host.
/// The initial URL is also validated before the first request.
/// If `NEXUM_WEBFETCH_ALLOW_LOCAL=1`, the initial URL validation is skipped
/// but redirect targets are still validated.
pub fn fetch_with_ssrf_guard(url_str: &str) -> Result<reqwest::blocking::Response> {
    let allow_local = std::env::var("NEXUM_WEBFETCH_ALLOW_LOCAL").is_ok();
    if allow_local {
        info!(target: TARGET, "SSRF guard: initial URL validation skipped via NEXUM_WEBFETCH_ALLOW_LOCAL");
    }

    // Validate initial URL (unless allow_local)
    let mut url = if allow_local {
        Url::parse(url_str).context("parse URL")?
    } else {
        validate_url(url_str).context("initial URL validation")?
    };

    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("build HTTP client")?;

    for hop in 0..MAX_REDIRECTS {
        info!(target: TARGET, "web_fetch GET hop={} url={}", hop, url);
        let resp = client.get(url.as_str()).send()?;

        // If not a redirect, return the response
        if !resp.status().is_redirection() {
            return Ok(resp);
        }

        // Follow redirect: validate the Location header
        let location = resp
            .headers()
            .get(reqwest::header::LOCATION)
            .context("redirect missing Location header")?
            .to_str()
            .context("Location header not valid UTF-8")?;

        // Resolve relative redirect
        url = url.join(location).context("resolve redirect URL")?;

        // Validate redirect target (always, even with allow_local)
        validate_url(url.as_str())
            .with_context(|| format!("redirect to `{}` blocked by SSRF guard", url))?;
    }

    anyhow::bail!("too many redirects (>{MAX_REDIRECTS})")
}
