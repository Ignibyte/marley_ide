# MCP annotations on every Marley tool — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-766-mcp-annotations.md
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

### Design (draft; checked again at promotion)
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
