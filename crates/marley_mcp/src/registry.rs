//! The tool-family REGISTRY (D4) — first-class `(family, verb)` typed data, ONE source of truth. Three
//! families: `fleet` (read) and `session` (write) from L1, and `terminal` (read, #491), the one the server
//! lists today. Adding `editor`/`browser` later is a new `Family` variant + a [`REGISTRY`] row + its
//! `tool_schemas`/`dispatch` arm — additive, no rework of permissions (REQ-011). PURE.

use crate::permission::Tier;
use serde_json::{Value, json};

/// A tool FAMILY — a CLOSED enum (D4). The public wire name keeps family + verb as separate typed fields;
/// the join is the single [`ToolSpec::name`] fn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// The fleet read family.
    Fleet,
    /// The session write family (`surface_to_human`, and Layer-2 verbs later).
    Session,
    /// Marley's terminals and their blocks, read by the app (#491).
    Terminal,
}

impl Family {
    /// The wire prefix for this family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fleet => "fleet",
            Self::Session => "session",
            Self::Terminal => "terminal",
        }
    }

    /// Whether `tools/list` lists this family's tools. The fleet and session families wait for prong
    /// 2's C1 to feed them; a client that names one of their tools still reaches it.
    #[must_use]
    pub const fn is_served(self) -> bool {
        matches!(self, Self::Terminal)
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
    /// The wire tool name `family_verb` (D4 — the one place family+verb are joined).
    #[must_use]
    pub fn name(&self) -> String {
        tool_name(self.family, self.verb)
    }
}

/// Compose a wire tool name from a family + verb (`fleet` + `snapshot` → `fleet_snapshot`). Claude
/// Code and the Anthropic API take tool names without dots (#491 D4).
#[must_use]
pub fn tool_name(family: Family, verb: &str) -> String {
    format!("{}_{}", family.as_str(), verb)
}

/// The L1 tool table (D4) as a CONST slice — the SINGLE source of truth. `tools_list` derives its names +
/// descriptions from this, so the wire list can never drift from what `lookup`/`dispatch` know (the D4
/// charter; the inspect-caught duplication is gone). Adding `editor`/`browser` = a new row here + its
/// `tool_schemas` arm + its `dispatch` arm.
const REGISTRY: &[ToolSpec] = &[
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
    ToolSpec {
        family: Family::Terminal,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List Marley's terminals: each one's id, title, project, working directory, the \
                      command running in it, and how many blocks it holds.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "blocks",
        tier: Tier::Read,
        grant_class: "",
        description: "List a terminal's blocks, the commands run in it, oldest first: each command, \
                      whether the shell's own hook reported it, its exit code, working directory, \
                      start time and duration, whether it still runs, and whether its output is \
                      still in the scrollback.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "Read one block's output as text: at most 2,000 lines, the end kept when there \
                      are more.",
    },
];

/// The L1 tool table (the const `REGISTRY`).
#[must_use]
pub const fn registry() -> &'static [ToolSpec] {
    REGISTRY
}

/// Look up a tool by its wire name (`None` = unknown tool).
#[must_use]
pub fn lookup(wire_name: &str) -> Option<ToolSpec> {
    REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.name() == wire_name)
}

/// The `tools/list` result (REQ-001) — DERIVED from `REGISTRY`: name + description from the spec,
/// the per-tool input/output schemas from `tool_schemas`, for the families the server serves.
///
/// One source of truth, no hand-repeated parallel table → no drift (a row added to `REGISTRY` is
/// automatically listed once its family is served).
#[must_use]
pub fn tools_list() -> Value {
    let tools: Vec<Value> = REGISTRY
        .iter()
        .filter(|spec| spec.family.is_served())
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
/// its schemas are wired: additivity is compiler-enforced, REQ-011). The terminal family's three tools
/// have an inner `verb` match.
fn tool_schemas(spec: &ToolSpec) -> (Value, Value) {
    match spec.family {
        Family::Terminal => terminal_schemas(spec.verb),
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

/// The input and output schemas of the terminal family's tools (#491).
fn terminal_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "blocks" => terminal_blocks_schemas(),
        "read" => terminal_read_schemas(),
        _ => terminal_list_schemas(),
    }
}

/// The schema of the `terminal` argument, a terminal's id.
fn terminal_argument_schema() -> Value {
    json!({ "type": "integer", "description": "The terminal's id, from terminal_list." })
}

/// `terminal_list`: no arguments; each terminal.
fn terminal_list_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "terminals": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "title": { "type": "string" },
                            "project": { "type": ["string", "null"] },
                            "cwd": { "type": ["string", "null"] },
                            "running": {
                                "type": ["string", "null"],
                                "description": "The command running in the terminal, if any."
                            },
                            "blocks": { "type": "integer" }
                        },
                        "required": ["id", "title", "blocks"]
                    }
                }
            },
            "required": ["terminals"]
        }),
    )
}

/// `terminal_blocks`: a terminal and how many of its newest blocks; each block.
fn terminal_blocks_schemas() -> (Value, Value) {
    let terminal = terminal_argument_schema();
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal,
                "last": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 500,
                    "description": "How many of the newest blocks to list; 50 when left out."
                }
            },
            "required": ["terminal"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "total": { "type": "integer", "description": "How many blocks the terminal holds." },
                "blocks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "index": { "type": "integer" },
                            "command": { "type": "string" },
                            "verified": {
                                "type": "boolean",
                                "description": "Whether the shell's own hook reported the command, not output that imitates one."
                            },
                            "running": { "type": "boolean" },
                            "exit_code": { "type": ["integer", "null"] },
                            "cwd": { "type": ["string", "null"] },
                            "started_at_ms": {
                                "type": ["integer", "null"],
                                "description": "When the command started, in Unix milliseconds."
                            },
                            "duration_ms": { "type": ["integer", "null"] },
                            "output_kept": {
                                "type": "boolean",
                                "description": "Whether the output is still in the terminal's scrollback."
                            }
                        },
                        "required": ["index", "command", "verified", "running", "output_kept"]
                    }
                }
            },
            "required": ["terminal", "total", "blocks"]
        }),
    )
}

/// `terminal_read`: a terminal and a block; the block's output.
fn terminal_read_schemas() -> (Value, Value) {
    let terminal = terminal_argument_schema();
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal,
                "block": { "type": "integer", "description": "The block's index, from terminal_blocks." }
            },
            "required": ["terminal", "block"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "block": { "type": "integer" },
                "command": { "type": "string" },
                "running": { "type": "boolean" },
                "output": { "type": "string" },
                "truncated": {
                    "type": "boolean",
                    "description": "Whether the start of the output was left out."
                }
            },
            "required": ["terminal", "block", "command", "running", "output", "truncated"]
        }),
    )
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
    fn tool_name_composes_family_underscore_verb() {
        assert_eq!(tool_name(Family::Fleet, "snapshot"), "fleet_snapshot");
        assert_eq!(
            tool_name(Family::Session, "surface_to_human"),
            "session_surface_to_human"
        );
    }

    #[test]
    fn registry_is_exactly_the_l1_set_with_correct_tiers() {
        let names: Vec<String> = registry().iter().map(ToolSpec::name).collect();
        assert_eq!(
            names,
            [
                "fleet_snapshot",
                "session_surface_to_human",
                "terminal_list",
                "terminal_blocks",
                "terminal_read"
            ]
        );
        assert_eq!(registry().len(), 5);
        assert_eq!(
            lookup("fleet_snapshot").expect("read tool").tier,
            Tier::Read
        );
        let write = lookup("session_surface_to_human").expect("write tool");
        assert_eq!(write.tier, Tier::Write);
        assert_eq!(write.grant_class, "session.write");
        assert!(lookup("nope.nope").is_none());
    }

    #[test]
    fn tools_list_derives_from_registry_with_input_and_output_schemas() {
        let list = tools_list();
        let tools = list["tools"].as_array().expect("tools array");
        // REQ-001 / F2: the wire list is EXACTLY the served registry names, in order (no drift).
        let listed: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        let registered: Vec<String> = registry()
            .iter()
            .filter(|spec| spec.family.is_served())
            .map(ToolSpec::name)
            .collect();
        assert_eq!(listed, registered);
        // REQ-001: each advertises an inputSchema AND an outputSchema.
        for tool in tools {
            assert!(tool["inputSchema"].is_object());
            assert!(tool["outputSchema"].is_object());
        }
        // the blocks tool's input requires a `terminal`.
        let blocks = tools
            .iter()
            .find(|t| t["name"] == "terminal_blocks")
            .expect("blocks tool");
        assert_eq!(blocks["inputSchema"]["required"][0], "terminal");
    }
}
