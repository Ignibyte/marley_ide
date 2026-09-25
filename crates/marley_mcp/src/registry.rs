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
    /// The page in Marley's Browser tab, seen and driven by the app (#492).
    Browser,
}

impl Family {
    /// The wire prefix for this family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fleet => "fleet",
            Self::Session => "session",
            Self::Terminal => "terminal",
            Self::Browser => "browser",
        }
    }

    /// Whether `tools/list` lists this family's tools. The fleet and session families wait for prong
    /// 2's C1 to feed them; a client that names one of their tools still reaches it.
    #[must_use]
    pub const fn is_served(self) -> bool {
        matches!(self, Self::Terminal | Self::Browser)
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
    browser_read(
        "tabs",
        "List Marley's Browser tabs, one per page of its browser: each tab's id, which the other \
         browser tools take as `tab`, its title and URL, whether it loads, and which one the tools \
         act on when a call names no tab, the one the user focused last.",
    ),
    browser_read(
        "look",
        "See a page in Marley's Browser tabs as the user sees it: its URL, title, viewport, \
         scroll, whether it loads, the focused element and the selection, and the frame on the \
         screen as an image.",
    ),
    browser_read(
        "snapshot",
        "A page's accessibility tree as text: its interactive elements (every node with \
         `full`), each with a ref for the write tools, cross-site iframes included.",
    ),
    browser_read(
        "console",
        "A page's latest console messages and uncaught errors, oldest first, at most 200.",
    ),
    browser_read(
        "network",
        "A page's latest requests, oldest first, at most 200: method, URL with secret-looking \
         values hidden, type, status, duration and failure; no headers or bodies.",
    ),
    browser_read(
        "picks",
        "List the elements the user picked in the Browser tabs this session, oldest first: each \
         pick's id, its tab, the page's URL and title, what the element is, the user's caption, \
         and whether the user sent it to you.",
    ),
    browser_read(
        "pick",
        "Read an element the user picked in a Browser tab, by its id from the user's line or \
         browser_picks, as it was at the pick: its locators, the most durable first, with \
         whether each finds it alone; its role and name; the listeners on it and its ancestors \
         with their scripts, lines and columns; what would block a click on it; its box in the \
         page; and the page around it as an image.",
    ),
    browser_write(
        "navigate",
        "Load an http or https URL in a Browser tab, or in a new tab with `new_tab`, opening one \
         when none is open; answers once the page has loaded, with the tab's id.",
    ),
    browser_write(
        "back",
        "Go back in a Browser tab's history; answers once the page has loaded.",
    ),
    browser_write(
        "click",
        "Click an element by its ref from browser_snapshot, or a point of the viewport, as the \
         user's mouse does.",
    ),
    browser_write(
        "type",
        "Type text as key presses into an element by its ref, or where the focus is; `submit` \
         presses Enter after.",
    ),
    browser_write(
        "press",
        "Press a key or a chord: Enter, Tab, Escape, ArrowDown, Ctrl+A, Shift+Tab.",
    ),
    browser_write(
        "scroll",
        "Scroll the page by pixels (dy down, dx right), or an element by its ref into view.",
    ),
];

/// A browser tool that reads the page (#492).
const fn browser_read(verb: &'static str, description: &'static str) -> ToolSpec {
    ToolSpec {
        family: Family::Browser,
        verb,
        tier: Tier::Read,
        grant_class: "",
        description,
    }
}

/// A browser tool that acts in the page the user sees (#492): Marley grants `browser.write` when
/// it starts the server, and a setting can take it away.
const fn browser_write(verb: &'static str, description: &'static str) -> ToolSpec {
    ToolSpec {
        family: Family::Browser,
        verb,
        tier: Tier::Write,
        grant_class: "browser.write",
        description,
    }
}

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
        Family::Browser => browser_schemas(spec.verb),
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

/// The input and output schemas of the browser family's tools (#492, #493).
fn browser_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "tabs" => tabs_schemas(),
        "look" => look_schemas(),
        "snapshot" => snapshot_schemas(),
        "console" | "network" => entries_schemas(verb),
        "picks" => picks_schemas(),
        "pick" => pick_schemas(),
        _ => browser_write_schemas(verb),
    }
}

/// The schema of the `tab` argument every browser tool but `browser_tabs` takes (#493).
fn tab_argument_schema() -> Value {
    json!({
        "type": "string",
        "description": "A tab's id from browser_tabs; left out, the tab the user focused last."
    })
}

/// A browser tool's arguments: `properties`, with `tab`, and the `required` ones.
fn browser_arguments(mut properties: Value, required: &[&str]) -> Value {
    if let Some(properties) = properties.as_object_mut() {
        properties.extend([("tab".to_string(), tab_argument_schema())]);
    }
    let mut schema = json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false
    });
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    schema
}

/// `browser_tabs`: no arguments; each tab.
fn tabs_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "tabs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "title": { "type": "string" },
                            "url": { "type": "string" },
                            "loading": { "type": "boolean" },
                            "focused": {
                                "type": "boolean",
                                "description": "The tab the tools act on when a call names none."
                            }
                        },
                        "required": ["id", "title", "url", "loading", "focused"]
                    }
                }
            },
            "required": ["tabs"]
        }),
    )
}

fn look_schemas() -> (Value, Value) {
    (
        browser_arguments(json!({}), &[]),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "url": { "type": "string" },
                "title": { "type": "string" },
                "loading": { "type": "boolean" },
                "viewport": { "type": "object" },
                "focused": { "type": ["object", "null"] },
                "selection": { "type": "string" }
            },
            "required": ["tab", "url", "title", "loading", "viewport"]
        }),
    )
}

fn snapshot_schemas() -> (Value, Value) {
    (
        browser_arguments(
            json!({
                "full": { "type": "boolean", "description": "Every node, not only the interactive ones." }
            }),
            &[],
        ),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "snapshot": { "type": "string" },
                "refs": { "type": "integer" },
                "cut": { "type": "boolean" }
            },
            "required": ["tab", "snapshot", "refs", "cut"]
        }),
    )
}

/// `browser_console` and `browser_network`: the entries of the tab's ring.
fn entries_schemas(verb: &str) -> (Value, Value) {
    let item = if verb == "console" {
        json!({
            "type": "object",
            "properties": {
                "level": { "type": "string" },
                "text": { "type": "string" },
                "source": { "type": ["string", "null"] },
                "line": { "type": ["integer", "null"] },
                "time_ms": { "type": ["number", "null"] }
            },
            "required": ["level", "text"]
        })
    } else {
        json!({
            "type": "object",
            "properties": {
                "method": { "type": "string" },
                "url": { "type": "string" },
                "kind": { "type": ["string", "null"] },
                "status": { "type": ["integer", "null"] },
                "duration_ms": { "type": ["integer", "null"] },
                "failure": { "type": ["string", "null"] }
            },
            "required": ["method", "url"]
        })
    };
    (
        browser_arguments(json!({}), &[]),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "entries": { "type": "array", "items": item }
            },
            "required": ["tab", "entries"]
        }),
    )
}

/// What `browser_picks` and `browser_pick` say of a pick besides its bundle.
fn pick_properties() -> Value {
    json!({
        "id": { "type": "integer" },
        "tab": { "type": "string", "description": "The tab the pick was made in." },
        "url": { "type": "string" },
        "title": { "type": "string" },
        "summary": { "type": "string", "description": "The element's role and name, or its tag and text." },
        "caption": { "type": "string", "description": "What the user said of it when sending it." },
        "sent": { "type": "boolean", "description": "Whether the user sent it to the agent." }
    })
}

/// `browser_picks`: no arguments; each pick of the session (#496).
fn picks_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "picks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": pick_properties(),
                        "required": ["id", "tab", "url", "title", "summary", "caption", "sent"]
                    }
                }
            },
            "required": ["picks"]
        }),
    )
}

/// `browser_pick`: a pick's id; the pick with its bundle, and its crop as the answer's image.
fn pick_schemas() -> (Value, Value) {
    let mut properties = pick_properties();
    let bundle = json!({
        "type": "object",
        "properties": {
            "tag": { "type": "string" },
            "role": { "type": ["string", "null"] },
            "name": { "type": ["string", "null"] },
            "text": { "type": "string" },
            "locators": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "description": "test id, id, text or css." },
                        "value": { "type": "string" },
                        "unique": { "type": ["boolean", "null"] }
                    },
                    "required": ["kind", "value"]
                }
            },
            "listeners": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "event": { "type": "string" },
                        "on": { "type": "string", "description": "The element, an ancestor, the document or the window." },
                        "script": { "type": ["string", "null"] },
                        "line": { "type": "integer", "description": "From 0." },
                        "column": { "type": "integer", "description": "From 0." },
                        "source_map": { "type": ["string", "null"] }
                    },
                    "required": ["event", "on", "line", "column"]
                }
            },
            "blockers": { "type": "array", "items": { "type": "string" } },
            "page_box": {
                "type": "object",
                "description": "In the document's CSS pixels.",
                "properties": {
                    "x": { "type": "number" },
                    "y": { "type": "number" },
                    "width": { "type": "number" },
                    "height": { "type": "number" }
                },
                "required": ["x", "y", "width", "height"]
            }
        },
        "required": ["tag", "text", "locators", "listeners", "blockers", "page_box"]
    });
    if let Some(properties) = properties.as_object_mut() {
        properties.extend([("bundle".to_string(), bundle)]);
    }
    (
        json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "A pick's id, as the user's line or browser_picks names it."
                }
            },
            "required": ["id"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": properties,
            "required": ["id", "tab", "url", "title", "summary", "caption", "sent", "bundle"]
        }),
    )
}

/// The write tools' arguments; each answers with what it did, and in which tab.
fn browser_write_schemas(verb: &str) -> (Value, Value) {
    let element =
        json!({ "type": "string", "description": "A ref from browser_snapshot, such as e3." });
    let done = json!({
        "type": "object",
        "properties": {
            "did": { "type": "string", "description": "What the tool did, as the Agent chip says it." },
            "tab": { "type": "string" },
            "url": { "type": "string" },
            "title": { "type": "string" }
        },
        "required": ["did", "tab"]
    });
    let arguments = match verb {
        "navigate" => browser_arguments(
            json!({
                "url": { "type": "string", "description": "An http or https URL." },
                "new_tab": { "type": "boolean", "description": "Open the page in a new tab, which leaves the user's focus where it is." }
            }),
            &["url"],
        ),
        "click" => browser_arguments(
            json!({
                "ref": element,
                "x": { "type": "number", "description": "A point of the viewport, in CSS pixels, with y." },
                "y": { "type": "number" },
                "button": { "type": "string", "enum": ["left", "right", "middle"] },
                "count": { "type": "integer", "minimum": 1, "maximum": 3 }
            }),
            &[],
        ),
        "type" => browser_arguments(
            json!({
                "text": { "type": "string" },
                "ref": element,
                "submit": { "type": "boolean", "description": "Press Enter after the text." }
            }),
            &["text"],
        ),
        "press" => browser_arguments(
            json!({
                "key": { "type": "string", "description": "Enter, Tab, Ctrl+A, Shift+ArrowLeft." }
            }),
            &["key"],
        ),
        "scroll" => browser_arguments(
            json!({
                "dy": { "type": "number", "description": "Pixels down; negative is up." },
                "dx": { "type": "number", "description": "Pixels right; negative is left." },
                "ref": element
            }),
            &[],
        ),
        // `back`.
        _ => browser_arguments(json!({}), &[]),
    };
    (arguments, done)
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
