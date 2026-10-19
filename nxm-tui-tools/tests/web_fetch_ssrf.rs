//! SSRF guard tests for `web_fetch` (R1-R7).
//!
//! Tests the address classification and redirect protection.
//! Uses pure IpAddr checks where possible; integration tests use
//! TcpListener with allow-list for legitimate local servers.

use std::net::IpAddr;
use std::str::FromStr;

use nxm_tui_tools::{default_registry, is_blocked_addr, validate_url};
use serde_json::json;

#[test]
fn ssrf_blocks_loopback() {
    // Test is_blocked_addr directly for various loopback addresses
    assert!(is_blocked_addr(IpAddr::from_str("127.0.0.1").unwrap()));
    assert!(is_blocked_addr(
        IpAddr::from_str("127.255.255.255").unwrap()
    ));
    assert!(is_blocked_addr(IpAddr::from_str("::1").unwrap()));
}

#[test]
fn ssrf_blocks_metadata() {
    // Metadata endpoint 169.254.169.254
    assert!(is_blocked_addr(
        IpAddr::from_str("169.254.169.254").unwrap()
    ));
    // Also test link-local range 169.254.0.0/16
    assert!(is_blocked_addr(IpAddr::from_str("169.254.0.1").unwrap()));
    assert!(is_blocked_addr(
        IpAddr::from_str("169.254.255.255").unwrap()
    ));
}

#[test]
fn ssrf_blocks_private() {
    // 10.0.0.0/8
    assert!(is_blocked_addr(IpAddr::from_str("10.0.0.1").unwrap()));
    assert!(is_blocked_addr(IpAddr::from_str("10.255.255.255").unwrap()));
    // 172.16.0.0/12
    assert!(is_blocked_addr(IpAddr::from_str("172.16.0.1").unwrap()));
    assert!(is_blocked_addr(IpAddr::from_str("172.31.255.255").unwrap()));
    // 192.168.0.0/16
    assert!(is_blocked_addr(IpAddr::from_str("192.168.0.1").unwrap()));
    assert!(is_blocked_addr(
        IpAddr::from_str("192.168.255.255").unwrap()
    ));
    // IPv6 ULA fc00::/7
    assert!(is_blocked_addr(IpAddr::from_str("fc00::1").unwrap()));
    assert!(is_blocked_addr(IpAddr::from_str("fd00::1").unwrap()));
}

#[test]
fn ssrf_blocks_non_http_scheme() {
    let reg = default_registry();
    // ftp:// scheme - reqwest accepts this but our guard blocks it
    let res = reg
        .call("web_fetch", &json!({ "url": "ftp://example.com/file" }))
        .expect("run web_fetch");
    assert!(!res.ok);
    assert!(res.content.to_lowercase().contains("scheme"));

    // gopher:// scheme
    let res = reg
        .call("web_fetch", &json!({ "url": "gopher://example.com/file" }))
        .expect("run web_fetch");
    assert!(!res.ok);
    assert!(res.content.to_lowercase().contains("scheme"));
}

#[test]
fn ssrf_blocks_redirect_to_loopback() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    // Start a server on 127.0.0.1 that redirects to another 127.0.0.1 port
    let redirect_target_port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind redirect target");
        let port = listener.local_addr().expect("local addr").port();
        let body = "should not reach here".to_string();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut req = [0u8; 1024];
                let _ = stream.read(&mut req);
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        port
    };

    // Start the initial server that redirects to the loopback target
    let initial_port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind initial server");
        let port = listener.local_addr().expect("local addr").port();
        let redirect_url = format!("http://127.0.0.1:{redirect_target_port}/");
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut req = [0u8; 1024];
                let _ = stream.read(&mut req);
                let response = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {redirect_url}\r\nConnection: close\r\n\r\n"
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        port
    };

    // Use allow-local for the initial request, but redirect should still be blocked
    std::env::set_var("NEXUM_WEBFETCH_ALLOW_LOCAL", "1");
    let reg = default_registry();
    let args = json!({ "url": format!("http://127.0.0.1:{initial_port}/") });
    let res = reg.call("web_fetch", &args).expect("run web_fetch");
    std::env::remove_var("NEXUM_WEBFETCH_ALLOW_LOCAL");
    assert!(!res.ok);
    // Should contain SSRF block message
    assert!(
        res.content.to_lowercase().contains("ssrf")
            || res.content.to_lowercase().contains("block")
            || res.content.to_lowercase().contains("redirect")
    );
}

#[test]
fn ssrf_allow_local_env() {
    // With NEXUM_WEBFETCH_ALLOW_LOCAL=1, loopback should work
    std::env::set_var("NEXUM_WEBFETCH_ALLOW_LOCAL", "1");

    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let port = listener.local_addr().expect("local addr").port();
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut stream = stream;
                let mut req = [0u8; 1024];
                let _ = stream.read(&mut req);
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: close\r\n\r\nallowed-local";
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        port
    };

    let reg = default_registry();
    let args = json!({ "url": format!("http://127.0.0.1:{port}/") });
    let res = reg.call("web_fetch", &args).expect("run web_fetch");
    assert!(res.ok);
    assert_eq!(res.content, "allowed-local");

    std::env::remove_var("NEXUM_WEBFETCH_ALLOW_LOCAL");
}

#[test]
fn ssrf_allows_public_ip() {
    // example.org IP: 93.184.216.34
    assert!(!is_blocked_addr(IpAddr::from_str("93.184.216.34").unwrap()));
    // Google DNS: 8.8.8.8
    assert!(!is_blocked_addr(IpAddr::from_str("8.8.8.8").unwrap()));
    // Cloudflare DNS: 1.1.1.1
    assert!(!is_blocked_addr(IpAddr::from_str("1.1.1.1").unwrap()));
    // IPv6 public: 2606:4700:4700::1111 (Cloudflare)
    assert!(!is_blocked_addr(
        IpAddr::from_str("2606:4700:4700::1111").unwrap()
    ));
}

#[test]
fn ssrf_blocks_unspecified() {
    assert!(is_blocked_addr(IpAddr::from_str("0.0.0.0").unwrap()));
    assert!(is_blocked_addr(IpAddr::from_str("::").unwrap()));
}

#[test]
fn ssrf_blocks_ipv6_link_local() {
    // fe80::/10
    assert!(is_blocked_addr(IpAddr::from_str("fe80::1").unwrap()));
    assert!(is_blocked_addr(
        IpAddr::from_str("febf:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap()
    ));
}

#[test]
fn ssrf_validate_url_scheme() {
    // http allowed
    assert!(validate_url("http://example.com").is_ok());
    // https allowed
    assert!(validate_url("https://example.com").is_ok());
    // ftp blocked
    assert!(validate_url("ftp://example.com").is_err());
    // file blocked
    assert!(validate_url("file:///etc/passwd").is_err());
    // gopher blocked
    assert!(validate_url("gopher://example.com").is_err());
}
