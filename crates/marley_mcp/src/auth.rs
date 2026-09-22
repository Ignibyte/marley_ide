//! Pre-dispatch security guards (D1/D9, the MCP Streamable-HTTP Security Warning): bind loopback only,
//! validate `Origin`, require the bearer. All PURE — the transport shim runs these BEFORE any dispatch.

/// Whether an authority (`host` or `host:port`, IPv6 bracketed) names a loopback host — the ONLY address
/// the server binds and the only `Origin` it accepts. The server mirror of the retired forge sidecar
/// client's `is_loopback_authority` (#411): `localhost`, `127.0.0.0/8`, `::1` qualify.
pub fn is_loopback(authority: &str) -> bool {
    let host = match authority.strip_prefix('[') {
        Some(rest) => rest.split(']').next().unwrap_or(rest), // [::1]:9 → ::1
        None => authority
            .rsplit_once(':')
            .map_or(authority, |(host, _)| host), // 127.0.0.1:9 → 127.0.0.1
    };
    host == "localhost"
        || host
            .parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false)
}

/// The authority (`host[:port]`) of an `Origin` header value like `http://127.0.0.1:9`. `None` if it has
/// no `scheme://`. A bare `null` origin (the literal the spec allows for opaque origins) has no authority.
fn origin_authority(origin: &str) -> Option<&str> {
    let rest = origin.split_once("://").map(|(_, rest)| rest)?;
    Some(rest.split(['/', '?', '#']).next().unwrap_or(rest))
}

/// Whether a request's `Origin` is allowed. A NON-browser MCP client (e.g. the manager's Claude Code
/// harness) sends no `Origin` — allowed. A browser Origin is allowed ONLY if it names a loopback host;
/// a cross-site / non-loopback Origin is REFUSED (the DNS-rebinding protection the spec mandates).
pub fn origin_allowed(origin: Option<&str>) -> bool {
    match origin {
        None => true,
        Some(origin) => origin_authority(origin).map(is_loopback).unwrap_or(false),
    }
}

/// A length-independent-of-content byte comparison (#375, D5): length-check, then XOR-accumulate over
/// EVERY byte with no early content-dependent exit, so the compare time doesn't leak how many leading
/// bytes matched. Used for the bearer AND session ids. On a loopback-only server this is defense-in-depth
/// (a local timing side-channel is a stretch), not a hot vulnerability — hence a 10-line pure fn, not a
/// `subtle`-style dependency. A length mismatch is a fast `false` (the length is not the secret).
pub fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Whether the presented bearer token matches the expected one. The expected token is never empty (an
/// empty expected would make an absent/empty presented spuriously "match" — refuse that outright). The
/// token compared here is already stripped of the `Bearer ` prefix by the transport shim. The match is
/// [`ct_eq`] (constant-time-ish — D5), not `==`.
pub fn bearer_ok(presented: Option<&str>, expected: &str) -> bool {
    !expected.is_empty() && presented.is_some_and(|p| ct_eq(p, expected))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_loopback_accepts_loopback_rejects_public() {
        assert!(is_loopback("127.0.0.1:9"));
        assert!(is_loopback("[::1]:9"));
        assert!(is_loopback("localhost"));
        assert!(is_loopback("127.0.0.1"));
        assert!(!is_loopback("evil.com:9"));
        assert!(!is_loopback("10.0.0.5")); // private, but NOT loopback
        assert!(!is_loopback("0.0.0.0"));
    }

    #[test]
    fn origin_allowed_missing_ok_loopback_ok_cross_site_refused() {
        assert!(origin_allowed(None)); // non-browser client
        assert!(origin_allowed(Some("http://127.0.0.1:8080")));
        assert!(origin_allowed(Some("http://localhost:9")));
        assert!(!origin_allowed(Some("http://evil.com")));
        assert!(!origin_allowed(Some("http://evil.com/127.0.0.1"))); // path can't smuggle the host
        assert!(!origin_allowed(Some("garbage-no-scheme"))); // malformed → refuse
    }

    #[test]
    fn bearer_ok_requires_nonempty_expected_and_exact_match() {
        assert!(bearer_ok(Some("tok"), "tok"));
        assert!(!bearer_ok(Some("wrong"), "tok"));
        assert!(!bearer_ok(None, "tok"));
        assert!(!bearer_ok(Some("anything"), "")); // empty expected never matches
        assert!(!bearer_ok(None, "")); // both empty → still no match
        assert!(!bearer_ok(Some(""), "")); // an EMPTY presented against an empty expected must NOT pass
        // (kills the `&&`→`||` bypass: the non-empty guard is a hard AND)
    }

    // #375 REQ-003 — the constant-time compare: equal → true; a diff at ANY position (first / middle /
    // last) → false with no early exit; a length mismatch → false; empty vs empty → true.
    #[test]
    fn ct_eq_matches_only_identical_bytes() {
        assert!(ct_eq("abcdef", "abcdef"));
        assert!(!ct_eq("Xbcdef", "abcdef")); // first byte differs
        assert!(!ct_eq("abcXef", "abcdef")); // middle byte differs
        assert!(!ct_eq("abcdeX", "abcdef")); // last byte differs (no short-circuit before here)
        assert!(!ct_eq("abcde", "abcdef")); // shorter
        assert!(!ct_eq("abcdefg", "abcdef")); // longer
        assert!(ct_eq("", "")); // both empty → equal (bearer_ok's non-empty guard handles the secret case)
        assert!(!ct_eq("a", ""));
    }
}
