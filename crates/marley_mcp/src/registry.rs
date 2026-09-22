//! The tool-family REGISTRY (D4) — first-class `(family, verb)` typed data, ONE source of truth. L1 ships
//! two families: `fleet` (read) and `session` (write). Adding `editor`/`browser` later is a new `Family`
//! variant + a [`REGISTRY`] row + its `tool_schemas`/`dispatch` arm — additive, no rework of permissions
//! (REQ-011). PURE.

use crate::permission::Tier;
use serde_json::{Value, json};

/// A tool FAMILY — a CLOSED enum (D4). The public wire name keeps family + verb as separate typed fields;
/// the join is the single [`ToolSpec::name`] fn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// The fleet read family.
    Fleet,
    /// The session write family (surface_to_human, and Layer-2 verbs later).
    Session,
}

impl Family {
    /// The wire prefix for this family.
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Fleet => "fleet",
            Family::Session => "session",
        }
    }
}

/// One registry entry: a family + verb, its permission tier, the write grant-class it needs (empty for a
/// read tool), and a description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSpec {
    /// The tool's family.
    pub family: Family,
    /// The verb within the family.
    pub verb: &'static str,
    /// Read (loose) or Write (grant-gated).
    pub tier: Tier,
    /// The permission grant-class a WRITE tool requires (empty for read tools).
    pub grant_class: &'static str,
    /// One-line description (surfaced in `tools/list`).
    pub description: &'static str,
}

impl ToolSpec {
    /// The wire tool name `family.verb` (D4 — the one place family+verb are joined).
    pub fn name(&self) -> String {
        tool_name(self.family, self.verb)
    }
}

/// Compose a wire tool name from a family + verb (`fleet` + `snapshot` → `fleet.snapshot`).
pub fn tool_name(family: Family, verb: &str) -> String {
    format!("{}.{}", family.as_str(), verb)
}

/// The L1 tool table (D4) as a CONST slice — the SINGLE source of truth. `tools_list` derives its names +
/// descriptions from this, so the wire list can never drift from what `lookup`/`dispatch` know (the D4
/// charter; the inspect-caught duplication is gone). Adding `editor`/`browser` = a new row here + its
/// `tool_schemas` arm + its `dispatch` arm.
pub const REGISTRY: &[ToolSpec] = &[
    ToolSpec {
        family: Family::Fleet,
        verb: "snapshot",
        tier: Tier::Read,
        grant_class: "",
        description: "Return the current fleet snapshot (all seats + their state).",
    },
    ToolSpec {
        family: Family::Session,
        verb: "surface_to_human",
        tier: Tier::Write,
        grant_class: "session.write",
        description: "Bring a session's surface into the human's view (focus it).",
    },
];

/// The L1 tool table (the const `REGISTRY`).
pub fn registry() -> &'static [ToolSpec] {
    REGISTRY
}

/// Look up a tool by its wire name (`None` = unknown tool).
pub fn lookup(wire_name: &str) -> Option<ToolSpec> {
    REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.name() == wire_name)
}

/// The `tools/list` result (REQ-001) — DERIVED from `REGISTRY`: name + description from the spec, the
/// per-tool input/output schemas from `tool_schemas`. One source of truth, no hand-repeated parallel
/// table → no drift (a row added to `REGISTRY` is automatically listed).
pub fn tools_list() -> Value {
    let tools: Vec<Value> = REGISTRY
        .iter()
        .map(|spec| {
            let (input, output) = tool_schemas(spec);
            json!({
                "name": spec.name(),
                "description": spec.description,
                "inputSchema": input,
                "outputSchema": output,
            })
        })
        .collect();
    json!({ "tools": tools })
}

/// The input + output JSON Schemas for a tool (the one thing that is NOT a `&'static str` on `ToolSpec`).
/// Matches on the family (EXHAUSTIVE — no catch-all, so adding a `Family` variant is a compile error until
/// its schemas are wired: additivity is compiler-enforced, REQ-011). L1 has one tool per family; a family
/// with multiple tools gains an inner `verb` match.
fn tool_schemas(spec: &ToolSpec) -> (Value, Value) {
    match spec.family {
        Family::Fleet => (
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
            fleet_snapshot_schema(),
        ),
        Family::Session => (
            json!({
                "type": "object",
                "properties": { "id": { "type": "string", "description": "The session id to surface." } },
                "required": ["id"],
                "additionalProperties": false
            }),
            receipt_schema(),
        ),
    }
}

/// The JSON Schema for a `FleetSnapshot` (the #367 shape — one seam, three consumers, D2). Kept in step
/// with `marley_fleet::FleetSnapshot`/`Session`; a drift is the defect class D2 kills.
fn fleet_snapshot_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "seats": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "state": { "type": "string", "enum": ["starting", "working", "idle", "waiting", "error", "done"] },
                        "question": { "type": ["object", "null"] },
                        "labels": { "type": "object" },
                        "last_event_ms": { "type": "integer" },
                        "transport": { "type": ["string", "null"], "enum": ["tmux", "bridge", "local", null] }
                    },
                    "required": ["id", "title", "state"]
                }
            }
        },
        "required": ["seats"]
    })
}

/// The JSON Schema for a `Receipt<T>` (the #367 tagged `Accepted`/`Refused`).
fn receipt_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "result": { "type": "string", "enum": ["accepted", "refused"] },
            "value": {},
            "reason": { "type": "string" }
        },
        "required": ["result"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_name_composes_family_dot_verb() {
        assert_eq!(tool_name(Family::Fleet, "snapshot"), "fleet.snapshot");
        assert_eq!(
            tool_name(Family::Session, "surface_to_human"),
            "session.surface_to_human"
        );
    }

    #[test]
    fn registry_is_exactly_the_l1_set_with_correct_tiers() {
        let names: Vec<String> = registry().iter().map(ToolSpec::name).collect();
        assert_eq!(names, ["fleet.snapshot", "session.surface_to_human"]);
        assert_eq!(registry().len(), 2);
        assert_eq!(
            lookup("fleet.snapshot").expect("read tool").tier,
            Tier::Read
        );
        let write = lookup("session.surface_to_human").expect("write tool");
        assert_eq!(write.tier, Tier::Write);
        assert_eq!(write.grant_class, "session.write");
        assert!(lookup("nope.nope").is_none());
    }

    #[test]
    fn tools_list_derives_from_registry_with_input_and_output_schemas() {
        let list = tools_list();
        let tools = list["tools"].as_array().expect("tools array");
        // REQ-001 / F2: the wire list is EXACTLY the registry names, in order (no drift).
        let listed: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        let registered: Vec<String> = registry().iter().map(ToolSpec::name).collect();
        assert_eq!(listed, registered);
        // REQ-001: each advertises an inputSchema AND an outputSchema.
        for tool in tools {
            assert!(tool["inputSchema"].is_object());
            assert!(tool["outputSchema"].is_object());
        }
        // the surface tool's input requires an `id`.
        let surface = tools
            .iter()
            .find(|t| t["name"] == "session.surface_to_human")
            .expect("surface tool");
        assert_eq!(surface["inputSchema"]["required"][0], "id");
        // the fleet tool's output schema carries the snapshot shape (the REAL fleet schema, not a stub).
        let fleet = tools
            .iter()
            .find(|t| t["name"] == "fleet.snapshot")
            .expect("fleet tool");
        assert!(fleet["outputSchema"]["properties"]["seats"].is_object());
    }
}
