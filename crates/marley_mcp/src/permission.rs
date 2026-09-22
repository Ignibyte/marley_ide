//! The permission model (D5, §10 binding) — PURE, deny-by-default. marley_mcp OWNS `GrantTable` (the S1
//! resolution of the #370↔#371 build-order trap): the server is the source of truth for what a grant IS;
//! #371's `[[mcp.*]]` settings round-trip merely DESERIALIZES into this type, so #370 ships + tests on
//! fixture grants with no dependency on #371 landing first.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The set of granted WRITE tool-classes (deny-by-default: a class not listed is denied). Read tools need
/// no grant. `#[serde(default)]` so a hand-edited settings file missing the key deserializes to "no
/// grants" (the safe default) rather than failing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrantTable {
    /// Explicitly-granted write tool-classes (e.g. `"session.write"`).
    #[serde(default)]
    pub write_classes: BTreeSet<String>,
}

impl GrantTable {
    /// Build a grant table from a set of class names (test + settings convenience).
    pub fn from_classes<I: IntoIterator<Item = S>, S: Into<String>>(classes: I) -> Self {
        GrantTable {
            write_classes: classes.into_iter().map(Into::into).collect(),
        }
    }
}

/// A tool's permission tier (D5): read = loose (zero config), write = explicit grant required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// A read tool — allowed with no grant.
    Read,
    /// A write tool — allowed only on an explicit per-class grant.
    Write,
}

/// The outcome of a permission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The call may proceed.
    Allow,
    /// The call is refused; the string names why (surfaced in the typed refusal).
    Deny(String),
}

/// Decide one call (D5). Read tier → always `Allow` (loose, zero configuration). Write tier → `Allow`
/// ONLY when its non-empty `grant_class` is explicitly present in `grants`; everything else → `Deny`.
/// Deny-by-default by construction: the only `Allow` arm for a write requires the class to be listed.
pub fn decide(tier: Tier, grant_class: &str, grants: &GrantTable) -> Decision {
    match tier {
        Tier::Read => Decision::Allow,
        Tier::Write if !grant_class.is_empty() && grants.write_classes.contains(grant_class) => {
            Decision::Allow
        }
        Tier::Write => Decision::Deny(format!(
            "write tool requires an explicit grant for class '{grant_class}' (deny-by-default)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_tier_always_allows_even_with_no_grants() {
        assert_eq!(
            decide(Tier::Read, "", &GrantTable::default()),
            Decision::Allow
        );
        assert_eq!(
            decide(Tier::Read, "", &GrantTable::from_classes(["x"])),
            Decision::Allow
        );
    }

    #[test]
    fn write_tier_denies_by_default() {
        // no grants at all → Deny
        assert!(matches!(
            decide(Tier::Write, "session.write", &GrantTable::default()),
            Decision::Deny(_)
        ));
        // an UNRELATED grant → Deny
        assert!(matches!(
            decide(
                Tier::Write,
                "session.write",
                &GrantTable::from_classes(["editor.write"])
            ),
            Decision::Deny(_)
        ));
        // an EMPTY grant_class is never spuriously allowed, even if "" is in the table
        assert!(matches!(
            decide(Tier::Write, "", &GrantTable::from_classes([""])),
            Decision::Deny(_)
        ));
    }

    #[test]
    fn write_tier_allows_only_on_the_exact_grant() {
        assert_eq!(
            decide(
                Tier::Write,
                "session.write",
                &GrantTable::from_classes(["session.write"])
            ),
            Decision::Allow
        );
    }

    #[test]
    fn grant_table_missing_key_deserializes_to_empty_and_round_trips() {
        let empty: GrantTable = serde_json::from_str("{}").expect("default");
        assert!(empty.write_classes.is_empty());
        let full = GrantTable::from_classes(["a", "b"]);
        let json = serde_json::to_string(&full).expect("ser");
        assert_eq!(serde_json::from_str::<GrantTable>(&json).expect("de"), full);
    }
}
