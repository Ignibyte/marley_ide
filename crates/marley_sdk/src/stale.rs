//! When Marley reads an agent as stale (Chad, 2026-09-30).
//!
//! Marley decides, from the agent's `last_seen_ms`, after three polls with no sign of it; a store
//! whose agents report less often widens that with its handshake's `stale_after_s`.

/// Polls with no sign of an agent before it reads stale.
pub const STALE_POLLS: u64 = 3;

/// How often Marley polls a store that names no `poll_s`, in seconds.
pub const DEFAULT_POLL_S: u64 = 5;

/// Whether an agent last seen at `last_seen_ms` reads stale at `now_ms`.
///
/// The store polls every `poll_s` seconds and may widen the window to `stale_after_s`. An agent
/// with no `last_seen_ms` never reads stale: the store keeps no such record.
#[must_use]
pub fn is_stale(
    last_seen_ms: Option<u64>,
    now_ms: u64,
    poll_s: Option<u64>,
    stale_after_s: Option<u64>,
) -> bool {
    let Some(last_seen_ms) = last_seen_ms else {
        return false;
    };
    let three_polls = poll_s
        .unwrap_or(DEFAULT_POLL_S)
        .max(1)
        .saturating_mul(STALE_POLLS);
    let window_s = stale_after_s.map_or(three_polls, |widened| widened.max(three_polls));
    now_ms.saturating_sub(last_seen_ms) > window_s.saturating_mul(1000)
}
