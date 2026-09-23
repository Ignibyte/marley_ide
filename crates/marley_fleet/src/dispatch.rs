//! The dispatch delivery-state machine — the mailbox contract that "never failed"
//! (fleet-control-plane §2/§5), as monotone-join types.
//!
//! `Deposited < Claimed < Started`; an observation only ever advances, so a duplicate and a missed
//! intermediate are both safe (Layer-2-ready).

use serde::{Deserialize, Serialize};

/// The delivery lifecycle of a dispatched brief. Ordering is declaration order: a later variant is
/// "further along" (the derived `Ord` follows declaration order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryState {
    /// Written to the mailbox, not yet picked up.
    Deposited,
    /// A seat has claimed the brief.
    Claimed,
    /// The seat has started acting on it.
    Started,
}

/// The outcome of observing a delivery-state: the resulting state and whether it advanced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeliveryAdvance {
    /// The state after the observation.
    pub state: DeliveryState,
    /// Whether the observation moved the machine forward.
    pub advanced: bool,
}

impl DeliveryState {
    /// Observe a (possibly duplicate or out-of-order) delivery-state. Advances to `observed` iff it is
    /// strictly further along than `self` — a skipped intermediate is legal (`Deposited → Started`). An
    /// observation at or behind `self` is an idempotent no-op reported as not advanced.
    #[must_use]
    pub fn observe(self, observed: Self) -> DeliveryAdvance {
        if observed > self {
            DeliveryAdvance {
                state: observed,
                advanced: true,
            }
        } else {
            DeliveryAdvance {
                state: self,
                advanced: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-012 + B2: the full 3×3 observe table. A strictly-greater observation advances (incl. the
    // Deposited→Started skip); an observation at-or-behind is a no-op. B2: assert `.advanced`
    // explicitly on the diagonal (equal) pairs — that flag, not `.state`, is what the >→>= mutant
    // flips.
    #[test]
    fn t367_req012_delivery_monotone_join_all_pairs() {
        use DeliveryState::*;
        let all = [Deposited, Claimed, Started];
        for &from in &all {
            for &observed in &all {
                let adv = from.observe(observed);
                if observed > from {
                    assert_eq!(adv.state, observed);
                    assert!(adv.advanced);
                } else {
                    assert_eq!(adv.state, from);
                    assert!(!adv.advanced);
                }
            }
        }
        // The skip is explicitly legal.
        assert_eq!(
            Deposited.observe(Started),
            DeliveryAdvance {
                state: Started,
                advanced: true,
            }
        );
    }

    // REQ-013: DeliveryState round-trips and serializes lowercase.
    #[test]
    fn t367_req013_delivery_state_round_trip() {
        for st in [
            DeliveryState::Deposited,
            DeliveryState::Claimed,
            DeliveryState::Started,
        ] {
            let json = serde_json::to_string(&st).unwrap();
            let back: DeliveryState = serde_json::from_str(&json).unwrap();
            assert_eq!(st, back);
        }
        assert_eq!(
            serde_json::to_string(&DeliveryState::Deposited).unwrap(),
            "\"deposited\""
        );
    }
}
