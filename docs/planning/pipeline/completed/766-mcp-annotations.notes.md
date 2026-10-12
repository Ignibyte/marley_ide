# MCP annotations on every Marley tool — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-766-mcp-annotations.md
- **Pipeline spec:** 766-mcp-annotations.spec.md

## Phase 1 — Plan (queued, 2026-10-11)
- **Request:** the second slice of the T3 Code survey's item 10, drafted in #756's notes ("Slice
  2") and split out as #766.
- **Classification:** chore. `marley_mcp` only; the guide.
- **Recall:** #756 added eight browser tools (66 in the registry); `tools_list_for`
  (`registry.rs:777-795`) builds each entry from `ToolSpec` and `tool_schemas`; the IDE server
  lists five tools in `ide.rs:268-311`. The served families are `is_served` (`registry.rs:72-86`):
  `fleet`, `session` and `prompt` are not, so `fleet_snapshot`, `session_surface_to_human`,
  `prompt_open` and `prompt_wait` are not listed and need no guide row.
- **The guide, checked 2026-10-11:** `guide.md` lacks `browser_draft_test`, `browser_check_pick`
  and `browser_open_url`; the in-app guide lacks only unserved tools.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/` at `d500498012`; the BACKLOG row removed; the ticket in
  progress. Brain: consultation 35294bbddf934b3ca5fffd1a488037fa, nothing on this seam.
- **Seams checked again** (read directly; the registry is #756's from today): `tools_list_for`
  `registry.rs:777-795`, `tool_schemas` 801, `is_served` 72-86; the IDE server's `tool_list`
  `ide.rs:267-312` (two tools always, three with `parts.selection`).

### Design (as built)
- `registry.rs`: `const DESTRUCTIVE: &[&str]` and `const IDEMPOTENT: &[&str]` (the spec's lists);
  `fn annotations(spec: &ToolSpec) -> Value` from the tier, the family (open world for the
  browser's page tools) and the lists; `fn tool_title(spec)` ("Browser: click", the family's
  words and the verb with spaces); `tools_list_for` adds `"annotations"`.
- `ide.rs`: each of the five entries gains `{"readOnlyHint": true, "destructiveHint": false,
  "idempotentHint": true, "openWorldHint": false}` and a title.
- The guide's browser tables gain the three rows.
- A registry test already pins names; a new assertion that every listed tool has annotations
  keeps compiling (no new tests run, §7).

### Visual check plan
| REQ | The check | The proof |
|---|---|---|
| 001, 002 | `mcp_agent annotations`, a new fixture command printing each tool's hints; checks for a read, a destructive and a non-destructive write | The run's log |
| 003 | — | The review |
| 004 | A check over the registry's served names against the guide | The run's log |
| 005 | — | The gate |

## Phase 2 — Code (2026-10-11)
- **Built:** `registry.rs`: `DESTRUCTIVE`, `IDEMPOTENT`, `BROWSER_RECORDS`, `annotations`,
  `tool_title`, and `annotations` in each `tools_list_for` entry (`session_surface_to_human` is in
  `DESTRUCTIVE` for when its family is served); `ide.rs`: `reads(title)` on the five tools. The
  guide: `browser_draft_test`, `browser_check_pick` and `browser_open_url` rows, and a paragraph on
  the annotations.
- **Deviations from the design:** reads state `destructiveHint: false` and `idempotentHint: true`
  rather than leaving those hints out (D2, every tool states all four, settles the spec's two
  readings); the open world is every browser tool but the six over Marley's own records
  (`BROWSER_RECORDS`), reads of a page included.
- **The review:** the lists name tools by their wire names, checked against the registry's
  pinned list; outside clients see the same annotations on the tools they are served.
- **Clippy:** clean on the first run; `cargo fmt` clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The run:** `script/e2e/766-mcp-annotations.sh` (N/A — no UI delta; one shot), exit 0.
  `766-01-marley`: Marley open on the scratch repository. The client's `tools/list` through the
  bridge: "tools: 60, annotated: 60"; `browser_tabs` read-only, `terminal_type` destructive,
  `browser_scroll` a non-destructive open-world write, `editor_list` closed-world,
  `browser_color_scheme` titled "Browser: color scheme", `browser_click` destructive and
  open-world. "listed: 60, missing from the guide: none".

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #766); the guide (three rows and the annotations
  paragraph); `marley_mcp.md`.
- **Knowledge appended:** AD-claude-766-tool-annotations-are-derived-from-the-registry-row-001.
  No bug found in Code or Test.
- **Brain:** consultation 35294bbddf934b3ca5fffd1a488037fa closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed.

