//! An agent stopped at its account's usage limit (#715, #765).
//!
//! A seat is limited from the turn its usage limit failed until its next turn: Codex's on a
//! `turn/completed` that failed with `usageLimitExceeded` or `rateLimitExceeded`
//! (`codex_events`), Claude Code's on a `StopFailure` whose error is `rate_limit`
//! (`claude_events`). The limit's reset is the latest reset after the failure among the seat's
//! quota windows at 100 % or more, the `quota.KIND.` labels Codex's rate-limit updates and Claude
//! Code's shared plugin put on the seat; Claude Code's windows may land after its failure, so the
//! reset is read from the labels each time unless the fold fixed it.

use std::collections::BTreeMap;

use marley_fleet::{Session, State};

/// The label for a limited seat: the identity of the failure, Codex's turn id or the moment
/// Marley folded Claude Code's `StopFailure`.
pub const LIMITED_LABEL: &str = "limited";
/// The label for when a limited seat's limit resets, in milliseconds since the epoch, when the
/// fold fixed it.
pub const LIMIT_RESETS_AT_LABEL: &str = "limit_resets_at_ms";
/// The label for when the limit stopped the turn, in milliseconds since the epoch.
pub const LIMITED_AT_LABEL: &str = "limited_at_ms";

/// The prefix of the labels that carry an account's quota windows, as `report.rs` writes them.
pub(crate) const QUOTA_PREFIX: &str = "quota.";

/// A usage limit an agent stopped at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limit {
    /// The failure the limit stopped: Codex's turn id, or Claude Code's moment.
    pub turn: String,
    /// When the limit resets, in milliseconds since the epoch; `None` when the agent did not say.
    pub resets_at_ms: Option<u64>,
}

/// The limit a limited seat stopped at; `None` when the seat is not limited.
#[must_use]
pub fn limit(seat: &Session) -> Option<Limit> {
    if seat.state != State::Error {
        return None;
    }
    let turn = seat.labels.get(LIMITED_LABEL)?.clone();
    let fixed = seat
        .labels
        .get(LIMIT_RESETS_AT_LABEL)
        .and_then(|reset| reset.parse().ok());
    let resets_at_ms = fixed.or_else(|| {
        let at = seat.labels.get(LIMITED_AT_LABEL)?.parse().ok()?;
        limit_reset(&seat.labels, at)
    });
    Some(Limit { turn, resets_at_ms })
}

/// When a limit that stopped a turn at `after_ms` resets: the latest reset after it among the
/// windows at 100 % or more, since every full window must reopen before a turn can run. `None`
/// when a full window gives no reset, or none is full.
pub(crate) fn limit_reset(labels: &BTreeMap<String, String>, after_ms: u64) -> Option<u64> {
    let mut latest = None;
    for (key, value) in labels {
        let Some(kind) = key
            .strip_prefix(QUOTA_PREFIX)
            .and_then(|key| key.strip_suffix(".percent_used"))
        else {
            continue;
        };
        if !value
            .trim()
            .parse::<f64>()
            .is_ok_and(|percent| percent >= 100.0)
        {
            continue;
        }
        let reset: u64 = labels
            .get(&format!("{QUOTA_PREFIX}{kind}.resets_at_ms"))?
            .parse()
            .ok()?;
        if reset > after_ms {
            latest = latest.max(Some(reset));
        }
    }
    latest
}
