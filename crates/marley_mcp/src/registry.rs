//! The tool-family REGISTRY (D4) — first-class `(family, verb)` typed data, ONE source of truth. Three
//! families: `fleet` (read) and `session` (write) from L1, and `terminal` (read, #491), the one the server
//! lists today. Adding `editor`/`browser` later is a new `Family` variant + a [`REGISTRY`] row + its
//! `tool_schemas`/`dispatch` arm — additive, no rework of permissions (REQ-011). PURE.

use std::collections::BTreeSet;

use crate::clients::{Principal, permits};
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
    /// The ports each project's processes listen on, read by the app (#521).
    Ports,
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
            Self::Ports => "ports",
        }
    }

    /// Whether `tools/list` lists this family's tools. The fleet and session families wait for prong
    /// 2's C1 to feed them; a client that names one of their tools still reaches it.
    #[must_use]
    pub const fn is_served(self) -> bool {
        matches!(self, Self::Terminal | Self::Browser | Self::Ports)
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
        description: "List Marley's terminals: each one's id, its `terminal_id` (the \
                      `MARLEY_TERMINAL_ID` its programs see), title, project, working directory, \
                      the command running in it, how many blocks it holds, and `self` for the \
                      terminal this call comes from.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "blocks",
        tier: Tier::Read,
        grant_class: "",
        description: "List a terminal's blocks (the calling terminal's when `terminal` is left \
                      out), the commands run in it, oldest first: each command, \
                      whether the shell's own hook reported it, its exit code, working directory, \
                      start time and duration, whether it still runs, and whether its output is \
                      still in the scrollback. Secrets in commands come back as \
                      `[redacted: <kind>]`, counted in `redacted`, unless the user turned \
                      redaction off.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "Read one block's output as text, from the calling terminal when `terminal` is \
                      left out: at most 2,000 lines, the end kept when there are more. Secrets come back as `[redacted: <kind>]`, counted in \
                      `redacted`, unless the user turned redaction off.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "find",
        tier: Tier::Read,
        grant_class: "",
        description: "Find the line of a block's output that matches a query in words, such as \
                      \"where the server refused the connection\", from the calling terminal \
                      when `terminal` is left out: the line's number and up to three \
                      candidates. A line holding every word of the query answers at once; the \
                      System One model ranks what the words leave open. When it is not sure, \
                      read the block with terminal_read. Secrets come back as \
                      `[redacted: <kind>]`. Listed while the user turns it on \
                      (marley.system_one.uses.terminal_find).",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "screen",
        tier: Tier::Read,
        grant_class: "",
        description: "Read what a terminal's screen shows now, from the calling terminal when \
                      `terminal` is left out: its rows, the cursor, whether a full-screen \
                      program has the alternate screen, the program in the foreground (none \
                      while the shell waits at its prompt), and who controls it: `generation`, \
                      which terminal_type needs, `taken_over`, and whether the user approved \
                      writes to this program. Secrets come back as `[redacted: <kind>]`.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "type",
        tier: Tier::Write,
        grant_class: "terminal.write",
        description: "Type into the program running in a terminal's foreground (psql, a \
                      debugger, a REPL, a dev server's prompt): `text` as a paste, then each of \
                      `keys` by name (escape, ctrl-c, up, tab), then Enter when `submit` is \
                      true, at most 4,096 bytes. Give the `generation` terminal_screen gave; a \
                      write is refused when the program changed since, at the shell's prompt, \
                      into an agent CLI, while the user has taken over, or when the user denies \
                      it or does not answer within 25 seconds: Marley asks before the first \
                      write to each program unless the user chose otherwise.",
    },
    browser_read(
        "tabs",
        "List Marley's Browser tabs, one per page, across every project's browser (each project \
         has a Chromium and a profile of its own): each tab's id, which the other browser tools \
         take as `tab`, its title and URL, whether it loads, its project, whether the user \
         focused it last, and `default`, the one the tools act on when a call names no tab: the \
         one the user focused last in the project you run in.",
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
        "find",
        "Find the element of the page that matches a query in words, such as \"the sign in \
         button\": its ref, which browser_click takes, and up to three candidates. An element \
         whose role and name hold every word of the query answers at once; the System One model \
         ranks what the words leave open. When it is not sure, read the page with \
         browser_snapshot. The refs are a new snapshot's, as browser_snapshot's are. Listed while \
         the user turns it on (marley.system_one.uses.browser_find).",
    ),
    browser_read(
        "console",
        "A page's latest console messages and uncaught errors, oldest first, at most 200, \
         with secrets as `[redacted: <kind>]` unless the user turned redaction off.",
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
         and whether the user sent it to you. Secret-looking text is redacted.",
    ),
    browser_read(
        "pick",
        "Read an element the user picked in a Browser tab, by its id from the user's line or \
         browser_picks, as it was at the pick: its locators, the most durable first, with \
         whether each finds it alone; its role and name; the listeners on it and its ancestors \
         with their scripts, lines and columns, and, through each script's source map, the file \
         in the project and the line they were written at; what would block a click on it; its \
         box in the page; its HTML, without scripts, field values, secret-looking attribute values \
         or URL queries, at most 4,096 characters; sixteen of its computed styles; its siblings' \
         texts and the page's selection; on a React dev build, the components around it and the \
         file and line it was written at; the page around it as an image; and its latest \
         check from browser_check_pick, when one ran. Secret-looking text is redacted.",
    ),
    browser_read(
        "recordings",
        "List the recordings the user saved with Record this in a Browser tab: each one's id, \
         its tab, the page's URL and title, when it was saved, how many seconds it spans, and \
         how many frames and entries it holds.",
    ),
    browser_read(
        "recording",
        "Read a recording the user saved with Record this, the minute before it, by its id \
         from browser_recordings: what the user did (presses with their places, keys by name, \
         typing as counts, and each click, fill and key press with the target's locators; a \
         fill keeps an ordinary field's text, at most 1,000 characters, and only the fact of it \
         for a password or other secret field), what the agent did, the console, the requests \
         with secret-looking URL values hidden, navigations, accessibility snapshots, and the \
         frames; `frame` gives one frame, from 1, as an image. Secret-looking text is redacted.",
    ),
    browser_read(
        "draft_test",
        "Draft a Playwright test from a recording, by its id from browser_recordings, for the \
         project it was recorded in, writing nothing: the test replays each click, fill and key \
         press with the most durable locator that found its target alone at the event \
         (getByTestId, getByRole with the exact name, getByLabel, getByPlaceholder, getByText, \
         then a CSS path), sets baseURL from the first action's origin, and expects the URL \
         after each navigation an action caused. A password or other secret field's fill reads \
         an environment variable, which the test names when it is unset. Answers the test, a \
         path for it in the project (the testDir its playwright.config sets, else tests/), the \
         variables it reads, what it skipped and why, and the command that runs it. Write the \
         file with your own tools.",
    ),
    browser_read(
        "annotations",
        "List the boxes and notes drawn over a page in a Browser tab, by the user or an agent: \
         each one's id, its box in page coordinates (the document's CSS pixels, which stay on \
         the content as the page scrolls), its note, who drew it and when.",
    ),
    browser_write(
        "annotate",
        "Draw a box with a note over a page in a Browser tab, for the user to see: around an \
         element by its ref from browser_snapshot, scrolled into view, or over an area of the \
         viewport; it stays on that content as the page scrolls, marked as the agent's. \
         `clear` removes the agent's own boxes instead.",
    ),
    browser_write(
        "check_pick",
        "Check an element the user picked, by its id from the user's line or browser_picks, \
         after a change such as your fix: find it again in the pick's own tab by the most \
         durable locator that still finds anything (its test id, id, role and name, text, then \
         CSS path; of several matches, the one nearest its old box), scroll it into view when it \
         is off screen, crop it as the pick was cropped, and say what changed since the pick: \
         its box, its computed styles, its text, role and name, or its HTML. Answers whether and \
         by what it was found, the changes, the element as it is now, and the new crop as the \
         image. Secret-looking text is redacted. The user's tray shows the check's verdict.",
    ),
    browser_write(
        "navigate",
        "Load an http or https URL in a Browser tab, or in a new tab with `new_tab`; with no \
         `tab`, in the tab the user focused last in the project you run in, opening one there \
         when it has none. A new tab opens in your project's browser, with that project's \
         cookies and logins. Answers once the page has loaded, with the tab's id.",
    ),
    browser_write(
        "open_url",
        "Open a URL a program in `directory` asked to open, as Marley's `BROWSER` opener does \
         (#561): in a Browser tab of the project whose folder holds `directory`, with the focus, \
         when `marley.terminal_links` sends that URL to a Browser tab (a local http or https \
         URL, by default), bringing forward a tab of that project already on it. A local HTML \
         page, a `file:` URL or a path to a `.html` or `.htm` file, opens the same way (#586); a \
         folder or any other file does not. It never opens the system browser: `opened: false` \
         says why it opened nothing.",
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
    ToolSpec {
        family: Family::Ports,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List the TCP ports the processes of each open project listen on, its dev \
                      servers among them: for each, the project, the project folder that holds \
                      the process's working directory, the address and port, the URL that \
                      reaches it, the pid, the process's name and its working directory. A \
                      listener outside every project is not listed.",
    },
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
/// it starts the server; no setting takes it away yet.
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

/// The tools listed and called only while the user turns them on (#567): each is the System One
/// use of the same name, on while its mode is not `off`.
pub const CONDITIONAL_TOOLS: [&str; 2] = ["browser_find", "terminal_find"];

/// Whether the tool `name` is off: one of [`CONDITIONAL_TOOLS`] that `enabled` leaves out.
#[must_use]
pub fn is_off(name: &str, enabled: &BTreeSet<String>) -> bool {
    CONDITIONAL_TOOLS.contains(&name) && !enabled.contains(name)
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
    tools_list_for(&Principal::Marley, &BTreeSet::new())
}

/// The `tools/list` result for `principal` (#524).
///
/// Marley's own bearer lists every served tool, and an outside client only the tools its grant's
/// list names. A conditional tool is listed while `enabled` holds it (#567).
#[must_use]
pub fn tools_list_for(principal: &Principal, enabled: &BTreeSet<String>) -> Value {
    let tools: Vec<Value> = REGISTRY
        .iter()
        .filter(|spec| {
            let name = spec.name();
            spec.family.is_served() && permits(principal, &name).is_ok() && !is_off(&name, enabled)
        })
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
        Family::Ports => ports_list_schemas(),
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
        "find" => find_schemas(),
        "console" | "network" => entries_schemas(verb),
        "annotations" => annotations_schemas(),
        "recordings" => recordings_schemas(),
        "recording" => recording_schemas(),
        "draft_test" => draft_test_schemas(),
        "picks" => picks_schemas(),
        "pick" => pick_schemas(),
        "check_pick" => check_pick_schemas(),
        "open_url" => open_url_schemas(),
        _ => browser_write_schemas(verb),
    }
}

/// `browser_open_url` (#561): the URL and the program's folder; whether a tab opened, and in
/// which project, or why not.
fn open_url_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The http or https URL the program opens, or a local HTML page as a file: URL or a path." },
                "directory": { "type": "string", "description": "The program's working directory, an absolute path." }
            },
            "required": ["url", "directory"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "opened": { "type": "boolean" },
                "project": { "type": "string", "description": "The project whose Browser tab shows the URL." },
                "reason": { "type": "string", "description": "Why no tab opened." }
            },
            "required": ["opened"]
        }),
    )
}

/// The schema of the `tab` argument every browser tool but `browser_tabs` takes (#493).
fn tab_argument_schema() -> Value {
    json!({
        "type": "string",
        "description": "A tab's id from browser_tabs. Left out: the tab the user focused last in \
                        the project you run in (your terminal's, else your folder's); for a \
                        caller in no project of Marley's, the tab the user focused last."
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
                                "description": "Whether the user focused this tab last, anywhere."
                            },
                            "project": {
                                "type": ["string", "null"],
                                "description": "The project the tab belongs to, as Marley's rail names it."
                            },
                            "default": {
                                "type": "boolean",
                                "description": "The tab the tools act on when a call of yours names none."
                            }
                        },
                        "required": ["id", "title", "url", "loading", "focused", "project", "default"]
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

/// The schema of a find's `query`, for `browser_find` and `terminal_find` (#567).
fn query_schema(example: &str) -> Value {
    json!({
        "type": "string",
        "maxLength": 200,
        "description": format!("What to find, in words, such as \"{example}\".")
    })
}

/// The answer's fields both find tools give (#567).
fn found_properties() -> Value {
    json!({
        "query": { "type": "string" },
        "source": {
            "type": "string",
            "enum": ["rules", "model", "none"],
            "description": "What found the answer: the query's words (rules), the System One model, or nothing."
        },
        "sure": { "type": "boolean", "description": "Whether the answer can be acted on without a look." },
        "present": {
            "type": ["string", "null"],
            "enum": ["found", "absent", "unsure", null],
            "description": "The model's reading of whether anything matches, when it was asked."
        },
        "verify": { "type": "boolean", "description": "The candidates are the model's suggestions: look before acting." },
        "next": { "type": ["string", "null"], "description": "The tool to read with when the answer is not sure." },
        "note": { "type": ["string", "null"], "description": "Why the model was not asked or did not answer, when it was not or did not." }
    })
}

/// `browser_find` (#567): a query; the matching element's ref, and up to three candidates.
fn find_schemas() -> (Value, Value) {
    let mut output = found_properties();
    if let Some(properties) = output.as_object_mut() {
        properties.extend([
            ("tab".to_string(), json!({ "type": "string" })),
            (
                "ref".to_string(),
                json!({ "type": ["string", "null"], "description": "The element's ref, when the answer is sure." }),
            ),
            (
                "candidates".to_string(),
                json!({
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "ref": { "type": "string" },
                            "role": { "type": "string" },
                            "name": { "type": "string" },
                            "probability": { "type": ["number", "null"] }
                        },
                        "required": ["ref", "role", "name"]
                    }
                }),
            ),
        ]);
    }
    (
        browser_arguments(
            json!({
                "query": query_schema("the sign in button"),
                "full": { "type": "boolean", "description": "Look among every node, not only the interactive ones." }
            }),
            &["query"],
        ),
        json!({
            "type": "object",
            "properties": output,
            "required": ["tab", "query", "source", "sure", "candidates"]
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

/// A box in page coordinates, the document's CSS pixels.
fn page_box_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "x": { "type": "number" },
            "y": { "type": "number" },
            "width": { "type": "number" },
            "height": { "type": "number" }
        },
        "required": ["x", "y", "width", "height"]
    })
}

/// What `browser_recordings` and `browser_recording` say of a recording besides its entries.
fn recording_properties() -> Value {
    json!({
        "id": { "type": "string" },
        "tab": { "type": "string" },
        "url": { "type": "string" },
        "title": { "type": "string" },
        "recorded_at": { "type": "integer", "description": "Seconds since the Unix epoch." },
        "seconds": { "type": "number", "description": "How long it spans." },
        "frames": { "type": "integer" }
    })
}

/// `browser_recordings`: no arguments; each recording, without its entries (#499).
fn recordings_schemas() -> (Value, Value) {
    let mut item = recording_properties();
    if let Some(properties) = item.as_object_mut() {
        properties.extend([(
            "entries".to_string(),
            json!({ "type": "integer", "description": "How many entries it holds." }),
        )]);
    }
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "recordings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": item,
                        "required": ["id", "tab", "url", "title", "recorded_at", "seconds", "frames", "entries"]
                    }
                }
            },
            "required": ["recordings"]
        }),
    )
}

/// `browser_recording`: a recording's id and, with `frame`, one frame as the answer's image.
fn recording_schemas() -> (Value, Value) {
    let mut properties = recording_properties();
    if let Some(object) = properties.as_object_mut() {
        object.extend([
            (
                "entries".to_string(),
                json!({
                    "type": "array",
                    "description": "What happened, oldest first: each entry's `kind` (click, scroll, key, typed, navigation, action, console, request, snapshot, agent or frame), `at_ms` from the recording's start, and its details. An action is a click, fill or press with the target's locators, most durable first, each marked `unique` when it found the element alone.",
                    "items": { "type": "object", "properties": { "kind": { "type": "string" }, "at_ms": { "type": "integer" } }, "required": ["kind", "at_ms"] }
                }),
            ),
            (
                "snapshot".to_string(),
                json!({ "type": "string", "description": "The page's accessibility snapshot when it was saved." }),
            ),
        ]);
    }
    (
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "A recording's id from browser_recordings." },
                "frame": { "type": "integer", "minimum": 1, "description": "A frame to see, from 1, as an entry of kind frame names it." }
            },
            "required": ["id"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": properties,
            "required": ["id", "tab", "url", "title", "recorded_at", "seconds", "frames", "entries", "snapshot"]
        }),
    )
}

/// `browser_annotations`: the tab's annotations (#498).
fn annotations_schemas() -> (Value, Value) {
    (
        browser_arguments(json!({}), &[]),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "annotations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "box": page_box_schema(),
                            "note": { "type": "string" },
                            "maker": { "type": "string", "enum": ["user", "agent"] },
                            "made_at": { "type": "integer", "description": "Seconds since the Unix epoch." }
                        },
                        "required": ["id", "box", "note", "maker", "made_at"]
                    }
                }
            },
            "required": ["tab", "annotations"]
        }),
    )
}

/// `browser_draft_test` (#506): a recording's id; the drafted test and where it goes.
fn draft_test_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "A recording's id from browser_recordings." }
            },
            "required": ["id"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "test": { "type": "string", "description": "The Playwright test, a TypeScript file's text." },
                "path": { "type": "string", "description": "Where the test goes: in the recording's project, when it names one." },
                "project": { "type": ["string", "null"], "description": "The root of the project the recording was made in." },
                "env": {
                    "type": "array",
                    "description": "The environment variables the test reads, one per secret field it fills.",
                    "items": { "type": "string" }
                },
                "skipped": {
                    "type": "array",
                    "description": "What the test leaves out, and why.",
                    "items": { "type": "string" }
                },
                "start": { "type": "string", "description": "The page the test starts on." },
                "run": { "type": "string", "description": "The command that runs it, from the project's root." },
                "note": { "type": ["string", "null"] }
            },
            "required": ["id", "test", "path", "env", "skipped", "start", "run"]
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

/// What #518 adds to a pick's bundle: the element's HTML, styles, sibling texts, the page's
/// selection and the React component.
fn element_context_properties() -> Value {
    json!({
        "html": {
            "type": "string",
            "description": "The element's HTML without scripts, field values, secret-looking attribute values or URL queries and fragments; at most 4,096 characters, ending ' (truncated)' when cut."
        },
        "styles": {
            "type": "object",
            "description": "Sixteen of its computed styles, by their CSS names.",
            "additionalProperties": { "type": "string" }
        },
        "nearby_text": {
            "type": "array",
            "description": "Its siblings' texts, the nearest first, before and after in turn; at most ten of 200 characters.",
            "items": { "type": "string" }
        },
        "selected_text": {
            "type": ["string", "null"],
            "description": "The page's selection at the pick, at most 500 characters; none when it lay in a field."
        },
        "component": {
            "type": ["object", "null"],
            "description": "On a React dev build: the components around the element and where it was written.",
            "properties": {
                "chain": {
                    "type": "array",
                    "description": "The components around it, the outermost first, at most six.",
                    "items": { "type": "string" }
                },
                "source": {
                    "type": ["object", "null"],
                    "description": "Where it was written: React's debug source, or React 19's debug stack through the scripts' source maps.",
                    "properties": {
                        "from": { "type": "string", "enum": ["debug source", "debug stack"] },
                        "source": { "type": "string", "description": "The source as React or the source map names it." },
                        "file": {
                            "type": ["string", "null"],
                            "description": "The file in the user's project that holds it, relative to its worktree."
                        },
                        "line": { "type": "integer", "description": "From 1." },
                        "column": {
                            "type": ["integer", "null"],
                            "description": "From 1 for a debug stack; as the JSX transform wrote it for a debug source."
                        }
                    },
                    "required": ["from", "source", "line"]
                }
            },
            "required": ["chain"]
        }
    })
}

/// `browser_pick`: a pick's id; the pick with its bundle and its latest check, and its crop as the
/// answer's image.
fn pick_schemas() -> (Value, Value) {
    let mut properties = pick_properties();
    if let Some(properties) = properties.as_object_mut() {
        properties.extend([
            ("bundle".to_string(), bundle_schema()),
            ("check".to_string(), check_schema()),
        ]);
    }
    (
        pick_id_schema(),
        json!({
            "type": "object",
            "properties": properties,
            "required": ["id", "tab", "url", "title", "summary", "caption", "sent", "bundle"]
        }),
    )
}

/// The arguments of a tool that takes a pick: its id.
fn pick_id_schema() -> Value {
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
    })
}

/// A pick's latest check (#505), as `browser_pick` gives it.
fn check_schema() -> Value {
    let bundle = nullable(bundle_schema());
    json!({
        "type": ["object", "null"],
        "description": "The pick's latest check from browser_check_pick; none before one ran.",
        "properties": {
            "found_by": {
                "type": ["string", "null"],
                "description": "The kind of locator that found the element again: test id, id, role and name, text or css; none when nothing did."
            },
            "changes": {
                "type": "array",
                "description": "What changed since the pick, a line each.",
                "items": { "type": "string" }
            },
            "bundle": bundle,
            "checked_at": { "type": "integer", "description": "When it ran, in milliseconds since the Unix epoch." }
        },
        "required": ["found_by", "changes", "checked_at"]
    })
}

/// `browser_check_pick` (#505): a pick's id; whether and how its element was found again, what
/// changed, and the element now, with its crop as the answer's image.
fn check_pick_schemas() -> (Value, Value) {
    let bundle = nullable(bundle_schema());
    (
        pick_id_schema(),
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "integer" },
                "tab": { "type": "string", "description": "The pick's tab, where the check ran." },
                "found": { "type": "boolean" },
                "found_by": {
                    "type": ["string", "null"],
                    "description": "The kind of locator that found it: test id, id, role and name, text or css."
                },
                "changes": {
                    "type": "array",
                    "description": "What changed since the pick, a line each: the box, each computed style, the text, the role, the name, or that only the HTML did.",
                    "items": { "type": "string" }
                },
                "bundle": bundle,
                "checked_at": { "type": "integer", "description": "In milliseconds since the Unix epoch." }
            },
            "required": ["id", "tab", "found", "found_by", "changes", "checked_at"]
        }),
    )
}

/// `schema`, an object's, taking null too.
fn nullable(mut schema: Value) -> Value {
    schema["type"] = json!(["object", "null"]);
    schema
}

/// A pick's bundle: the element as a pick or a check read it.
fn bundle_schema() -> Value {
    let mut bundle = json!({
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
                        "line": { "type": "integer", "description": "In the script, from 1." },
                        "column": { "type": "integer", "description": "In the script, from 1." },
                        "source_map": { "type": ["string", "null"] },
                        "original": {
                            "type": ["object", "null"],
                            "description": "Where the script's source map says the listener was written.",
                            "properties": {
                                "source": { "type": "string", "description": "The source, as its map names it." },
                                "file": {
                                    "type": ["string", "null"],
                                    "description": "The file in the user's project that holds it, relative to its worktree."
                                },
                                "line": { "type": "integer", "description": "From 1." },
                                "column": { "type": "integer", "description": "From 1." }
                            },
                            "required": ["source", "line", "column"]
                        }
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
        "required": ["tag", "text", "locators", "listeners", "blockers", "page_box", "html", "styles", "nearby_text"]
    });
    if let (Some(bundle_properties), Value::Object(added)) = (
        bundle.get_mut("properties").and_then(Value::as_object_mut),
        element_context_properties(),
    ) {
        bundle_properties.extend(added);
    }
    bundle
}

/// The write tools' arguments; each answers with what it did, and in which tab.
fn browser_write_schemas(verb: &str) -> (Value, Value) {
    let element =
        json!({ "type": "string", "description": "A ref from browser_snapshot, such as e3." });
    let mut done = json!({
        "type": "object",
        "properties": {
            "did": { "type": "string", "description": "What the tool did, as the Agent chip says it." },
            "tab": { "type": "string" },
            "url": { "type": "string" },
            "title": { "type": "string" }
        },
        "required": ["did", "tab"]
    });
    if verb == "annotate"
        && let Some(properties) = done.get_mut("properties").and_then(Value::as_object_mut)
    {
        properties.extend([
            (
                "id".to_string(),
                json!({ "type": "integer", "description": "The annotation's id; none for `clear`." }),
            ),
            ("box".to_string(), page_box_schema()),
        ]);
    }
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
        "annotate" => browser_arguments(
            json!({
                "ref": element,
                "x": { "type": "number", "description": "An area of the viewport, in CSS pixels, with y, width and height." },
                "y": { "type": "number" },
                "width": { "type": "number" },
                "height": { "type": "number" },
                "note": { "type": "string", "description": "What the box says." },
                "clear": { "type": "boolean", "description": "Remove the agent's own boxes from the page instead." }
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
        "find" => terminal_find_schemas(),
        "screen" => terminal_screen_schemas(),
        "type" => terminal_type_schemas(),
        _ => terminal_list_schemas(),
    }
}

/// The schema of the `terminal` argument, a terminal's id.
fn terminal_argument_schema() -> Value {
    json!({
        "type": "integer",
        "description": "The terminal's id, from terminal_list; the terminal this call comes from when left out."
    })
}

/// `terminal_list`: no arguments; each terminal.
/// `ports_list` (#521): no arguments; each listener, never its command line, which can carry a
/// token.
fn ports_list_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "ports": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "project": { "type": "string" },
                            "folder": {
                                "type": "string",
                                "description": "The project folder that holds the process's working directory."
                            },
                            "address": { "type": "string" },
                            "port": { "type": "integer" },
                            "url": { "type": "string" },
                            "pid": { "type": "integer" },
                            "name": { "type": "string", "description": "The process's name." },
                            "cwd": { "type": "string" }
                        },
                        "required": ["project", "folder", "address", "port", "url", "pid", "name", "cwd"]
                    }
                }
            },
            "required": ["ports"]
        }),
    )
}

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
                            "terminal_id": {
                                "type": ["string", "null"],
                                "description": "The MARLEY_TERMINAL_ID the terminal's programs see."
                            },
                            "self": {
                                "type": "boolean",
                                "description": "Whether this call comes from this terminal."
                            },
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
            "required": [],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "total": { "type": "integer", "description": "How many blocks the terminal holds." },
                "redacted": {
                    "type": "integer",
                    "description": "How many secrets the listed commands had hidden."
                },
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
            "required": ["block"],
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
                },
                "redacted": {
                    "type": "integer",
                    "description": "How many secrets the command and the whole output had hidden, the part left out included."
                }
            },
            "required": ["terminal", "block", "command", "running", "output", "truncated"]
        }),
    )
}

/// `terminal_screen` (#525): a terminal; its screen and who controls it.
fn terminal_screen_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": { "terminal": terminal_argument_schema() },
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "rows": { "type": "array", "items": { "type": "string" } },
                "cursor": {
                    "type": "object",
                    "properties": {
                        "row": { "type": "integer" },
                        "column": { "type": "integer" }
                    }
                },
                "columns": { "type": "integer" },
                "lines": { "type": "integer" },
                "alternate_screen": { "type": "boolean" },
                "scrolled": {
                    "type": "boolean",
                    "description": "Whether the user scrolled back, so the rows are not the bottom of the screen."
                },
                "program": { "type": ["string", "null"] },
                "generation": { "type": "integer" },
                "taken_over": { "type": "boolean" },
                "approval": { "type": "string", "enum": ["ask_first_write", "ask_every_write", "never_ask"] },
                "approved": { "type": "boolean" },
                "redacted": { "type": "integer" }
            },
            "required": ["terminal", "rows", "generation", "taken_over", "approval", "approved"]
        }),
    )
}

/// `terminal_type` (#525): a terminal, its generation, and what to type; what was typed.
fn terminal_type_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal_argument_schema(),
                "generation": {
                    "type": "integer",
                    "description": "The terminal's generation, from terminal_screen."
                },
                "text": { "type": "string", "description": "Text to type, as a paste." },
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Keys by name after the text: escape, ctrl-c, up, tab."
                },
                "submit": { "type": "boolean", "description": "Press Enter last." }
            },
            "required": ["generation"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "written": { "type": "integer" },
                "program": { "type": "string" },
                "generation": { "type": "integer" }
            },
            "required": ["written", "program", "generation"]
        }),
    )
}

/// `terminal_find` (#567): a terminal, a block and a query; the matching line, and up to three
/// candidates.
fn terminal_find_schemas() -> (Value, Value) {
    let mut output = found_properties();
    if let Some(properties) = output.as_object_mut() {
        properties.extend([
            ("terminal".to_string(), json!({ "type": "integer" })),
            ("block".to_string(), json!({ "type": "integer" })),
            (
                "line".to_string(),
                json!({
                    "type": ["integer", "null"],
                    "description": "The line's number in the block's output, from 1, when the answer is sure."
                }),
            ),
            (
                "candidates".to_string(),
                json!({
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "line": { "type": "integer" },
                            "text": { "type": "string" },
                            "probability": { "type": ["number", "null"] }
                        },
                        "required": ["line", "text"]
                    }
                }),
            ),
            (
                "cut".to_string(),
                json!({
                    "type": "boolean",
                    "description": "Whether the model saw less than the block: its newest lines, up to about 24,000 tokens."
                }),
            ),
        ]);
    }
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal_argument_schema(),
                "block": { "type": "integer", "description": "The block's index, from terminal_blocks." },
                "query": query_schema("where the server refused the connection")
            },
            "required": ["block", "query"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": output,
            "required": ["terminal", "block", "query", "source", "sure", "candidates", "cut"]
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
