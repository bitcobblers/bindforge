//! The `--upstream` endpoint gate (design 026, Chosen design: §`apps/registry`'s
//! copied list, §Caching mode).
//!
//! This is the copied `parse_endpoint` from the shipped client
//! (`apps/discovery-cli/src/catalog.rs`) with `026-L1`'s one amendment: the
//! loopback host allowlist is gone, so any parseable host+port behind a literal
//! `http://` is admitted. The literal-scheme requirement, the credentials refusal,
//! the empty-authority refusal, and the digits-only port rule are kept verbatim,
//! and `https` stays out of v1 (`026-C1`: std-only, no TLS; the transport hop's
//! assurance is the deferred `026-R1`).
//!
//! Admission is not a connection: nothing here resolves a name, and a freshly
//! started caching instance dials nothing (Test strategy #14).

use bindforge_core::escape_and_truncate;

/// The only scheme this registry will speak: plain HTTP, no TLS (A11).
const SCHEME: &str = "http://";

/// One refused endpoint spelling, with the reason an operator reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointError {
    /// The spelling that was refused.
    pub url: String,
    /// The named condition it refuses.
    pub reason: String,
}

impl EndpointError {
    /// The line the refusal prints.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "--upstream `{}` is not an endpoint this registry runs: {}",
            escape_and_truncate(&self.url),
            escape_and_truncate(&self.reason)
        )
    }
}

/// Splits an authority into its host and explicit port, keeping bracketed IPv6
/// literals intact. `None` port means "use the scheme default".
fn split_authority(authority: &str) -> Option<(&str, Option<&str>)> {
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']')?;
        // `rest` drops the `[`, so the `]` sits at `end + 1` in `authority`.
        let host = &authority[..=end + 1];
        let tail = &rest[end + 1..];
        if tail.is_empty() {
            return Some((host, None));
        }
        return Some((host, tail.strip_prefix(':')));
    }
    match authority.split_once(':') {
        Some((host, port)) => Some((host, Some(port))),
        None => Some((authority, None)),
    }
}

/// The endpoint policy: a literal `http://` scheme, a non-empty authority with no
/// credentials, a parseable host, and a digits-only port. Returns the normalized
/// `http://<host>:<port>` base without any path.
pub fn parse_endpoint(url: &str) -> Result<String, EndpointError> {
    let failed = |reason: String| EndpointError {
        url: url.to_string(),
        reason,
    };
    let rest = url.strip_prefix(SCHEME).ok_or_else(|| {
        failed(format!(
            "`{}` does not start with the literal `http://` scheme",
            escape_and_truncate(url)
        ))
    })?;
    let authority = match rest.find(['/', '?', '#']) {
        Some(index) => &rest[..index],
        None => rest,
    };
    if authority.is_empty() {
        return Err(failed(
            "the authority is empty — a host is required".to_string(),
        ));
    }
    if authority.contains('@') {
        return Err(failed(format!(
            "the authority `{}` carries credentials",
            escape_and_truncate(authority)
        )));
    }
    let (host, port) = split_authority(authority).ok_or_else(|| {
        failed(format!(
            "the authority `{}` is not a parseable host",
            escape_and_truncate(authority)
        ))
    })?;
    if host.is_empty() {
        return Err(failed(
            "the authority is empty — a host is required".to_string(),
        ));
    }
    let port = match port {
        None => 80,
        Some(digits) => {
            if digits.is_empty()
                || digits.len() > 5
                || !digits.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(failed(format!(
                    "the port `{}` is not a decimal number",
                    escape_and_truncate(digits)
                )));
            }
            digits
                .parse::<u16>()
                .map_err(|_| failed(format!("the port `{digits}` is outside 0–65535")))?
        }
    };
    Ok(format!("{SCHEME}{host}:{port}"))
}

/// The `<host>:<port>` authority of a normalized base, for connecting.
#[must_use]
pub fn authority_of(base: &str) -> (String, u16) {
    let authority = base.trim_start_matches(SCHEME);
    let (host, port) = split_authority(authority).unwrap_or((authority, Some("80")));
    // `to_socket_addrs` wants the bare address; brackets belong to the URL.
    let host = host.trim_start_matches('[').trim_end_matches(']');
    (
        host.to_string(),
        port.unwrap_or("80").parse::<u16>().unwrap_or(80),
    )
}

#[cfg(test)]
mod tests {
    //! The gate as Test strategy #14 states it, plus the `026-L1` amendment that
    //! remote admission is a `parse_endpoint` fact: any parseable host is admitted,
    //! so no name here is ever resolved.
    use super::{authority_of, parse_endpoint};

    #[test]
    fn the_gate_admits_any_parseable_host_behind_a_literal_http_origin() {
        for (spelling, normalized) in [
            ("http://127.0.0.1:8080", "http://127.0.0.1:8080"),
            ("http://127.0.0.5:9", "http://127.0.0.5:9"),
            ("http://localhost:9/", "http://localhost:9"),
            ("http://[::1]:8080/v1", "http://[::1]:8080"),
            ("http://127.0.0.1", "http://127.0.0.1:80"),
            // `026-L1`: the host allowlist is gone, so a remote spelling parses.
            ("http://catalog.example:8080", "http://catalog.example:8080"),
            ("http://0177.0.0.1/", "http://0177.0.0.1:80"),
        ] {
            assert_eq!(
                parse_endpoint(spelling).expect("admitted"),
                normalized,
                "`{spelling}` is admitted and normalized"
            );
        }
        assert_eq!(
            authority_of("http://127.0.0.1:8080"),
            ("127.0.0.1".to_string(), 8080)
        );
        assert_eq!(
            authority_of("http://[::1]:9"),
            ("::1".to_string(), 9),
            "brackets belong to the URL, not the address"
        );
    }

    #[test]
    fn the_gate_refuses_every_spelling_beyond_a_literal_http_origin() {
        for spelling in [
            "https://127.0.0.1:8080",
            "https://catalog.example",
            "ftp://127.0.0.1:8080",
            "http://operator@127.0.0.1:8080",
            "",
            "http://127.0.0.1:not-a-port",
            "127.0.0.1:8080",
            "http:/127.0.0.1:8080",
            "http://:8080",
            "http://127.0.0.1:70000",
        ] {
            let error = parse_endpoint(spelling).expect_err("`{spelling}` is refused");
            let line = error.line();
            assert!(
                line.contains("--upstream"),
                "the refusal says which argument it is about: {line}"
            );
            assert!(!line.contains('\n'), "one refusal is one line: {line:?}");
        }
    }
}
