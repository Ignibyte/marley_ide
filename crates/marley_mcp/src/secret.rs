//! PURE — minting a 128-bit secret token (#375, cluster A) from OS entropy. The bearer AND every
//! `Mcp-Session-Id` are formatted here so they share one code path (32 lowercase hex chars). No clock,
//! no pid, no counter — the ONLY input is 16 bytes of OS CSPRNG (`/dev/urandom`), read by the masked
//! transport shim and passed in, so the format + the refuse-on-failure decision stay pure + tested.

use std::fmt;

/// Format 16 bytes of entropy as a 128-bit token: exactly 32 lowercase hex chars, no separators. The
/// single formatting path for the bearer and session ids (D8).
#[must_use]
pub fn hex128(bytes: [u8; 16]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(32);
    for byte in bytes {
        // Two lowercase hex digits per byte, high nibble first, so a leading zero byte keeps width.
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

/// The server could not obtain OS entropy — it MUST refuse to start rather than fall back to a weaker
/// source (D2). A silently-weak bearer is worse than no server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntropyError;

impl fmt::Display for EntropyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP server: OS entropy unavailable — refusing to start")
    }
}

impl std::error::Error for EntropyError {}

/// Mint a token from an entropy read: `Some(bytes)` (a successful `/dev/urandom` read) → the hex
/// token.
///
/// # Errors
///
/// `None` (the read failed) → `Err(EntropyError)`, so the caller refuses to start (D2).
///
/// The masked shim maps the `io::Result<[u8; 16]>` to `Option` (via `.ok()`) before calling this —
/// keeping the decision pure and testable with an injected failure.
pub fn mint_secret(entropy: Option<[u8; 16]>) -> Result<String, EntropyError> {
    entropy.map(hex128).ok_or(EntropyError)
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-001 — the hex-128 formatter: exactly 32 lowercase hex chars, deterministic, distinct inputs
    // produce distinct outputs, a zero draw keeps its width.
    #[test]
    fn hex128_is_32_lowercase_hex_chars() {
        let token = hex128([
            0x00, 0x0f, 0xa1, 0xff, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xab,
            0xcd, 0xef,
        ]);
        assert_eq!(token.len(), 32);
        assert!(
            token
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
        assert_eq!(&token[..4], "000f"); // leading zero byte keeps its two-digit width
        assert!(token.ends_with("abcdef"));
        // all-zeros → 32 zeros; distinct inputs → distinct outputs.
        assert_eq!(hex128([0u8; 16]), "0".repeat(32));
        assert_ne!(hex128([1u8; 16]), hex128([2u8; 16]));
    }

    // REQ-002 — mint refuses (typed) on an entropy failure, never a fallback.
    #[test]
    fn mint_secret_ok_on_entropy_err_on_failure() {
        assert_eq!(mint_secret(Some([0xab; 16])), Ok(hex128([0xab; 16])));
        assert_eq!(mint_secret(None), Err(EntropyError));
        assert!(EntropyError.to_string().contains("refusing to start"));
    }
}
