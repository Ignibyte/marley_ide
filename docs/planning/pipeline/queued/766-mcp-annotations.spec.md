---
pipeline_id: c6fc52f0-21e6-4cc2-96a2-e70aa9484662
ticket: docs/planning/tickets/open/TICKET-766-mcp-annotations.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: MCP annotations on every Marley tool
type: chore
slice: Marley's MCP server; the second slice of the T3 Code survey's item 10 (#756 the first)
references:
  - docs/planning/pipeline/completed/756-the-browser-tools-agents-lack.notes.md
  - docs/t3code_architecture/06-orchestration-mcp-and-automations.md
---

## Title
Each tool Marley's MCP server lists, and the IDE server's five, carries MCP's annotations, so a
client can tell a read from an action and an action that destroys from one that does not; the
guide lists every tool the server serves. Chad, 2026-10-11: "18.) build now", item 10's slices.

## Scope
### In
- **`tools/list`** (`registry::tools_list_for`) gives each tool an `annotations` object (MCP
  2025-06-18): `title`, `readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`.
  - Read-tier tools: `readOnlyHint: true`, `openWorldHint` false but for the browser's reads of
    a page (the web is open), and no destructive or idempotent hint (they do not apply).
  - Write tools: `readOnlyHint: false`; `destructiveHint` true for the ones that change or send
    what the user cannot take back from Marley (`terminal_type`, `terminal_run`, `editor_edit`,
    `editor_save`, `settings_change`, `keymap_change`, `seat_add`, `seat_stop`, `seat_remove`,
    `thread_post`, `thread_answer`, `action_run`, `browser_click`, `browser_type`,
    `browser_press`, `browser_navigate`, `browser_back`, `browser_select`, `browser_drag`,
    `browser_upload`, `browser_dialog`), false for the rest (`editor_open`, `browser_scroll`,
    `browser_hover`, `browser_resize`, `browser_color_scheme`, `browser_annotate`,
    `browser_check_pick`, `browser_open_url`); `idempotentHint` true for `editor_open`,
    `browser_resize`, `browser_color_scheme`, `browser_open_url`; `openWorldHint` true for the
    browser tools that act in a page.
  - `title`: the tool's name in words, such as "Browser: click".
- The IDE server's five tools (`ide.rs`) are read-only, closed-world.
- The guide's tool tables gain the rows they lack: `browser_draft_test`, `browser_check_pick`,
  `browser_open_url`.
- **Docs:** `marley_mcp.md`.

### Out (explicitly deferred)
- Changing what any tool does, or which tools a client is served.
- Outside clients' lists (#524): their tools carry the same annotations, nothing more.

## Reference (§20)
N/A — Marley-specific: Marley's own MCP server. The annotations are MCP 2025-06-18's
(`ToolAnnotations`), as T3 Code's MCP server sets them (T3 survey report 06 §2.1).

### Prior art
- **The code we ship:** `crates/marley_mcp/src/registry.rs` (`ToolSpec`, `REGISTRY`,
  `tools_list_for`, `Tier`), `crates/marley_mcp/src/ide.rs` (the IDE server's `tools/list`).
  The annotations are computed from the row (its tier and family) and two short lists, so no row
  of the 66 changes.
- **Published material:** MCP 2025-06-18, `Tool.annotations` (`ToolAnnotations`): the hints are
  advice to clients; `destructiveHint` and `idempotentHint` mean something only when
  `readOnlyHint` is false; the defaults are `readOnlyHint: false`, `destructiveHint: true`,
  `idempotentHint: false`, `openWorldHint: true`, so each tool states all four.
- **Behavior maps:** the T3 survey's report 06 §2.1.

## UI proof
N/A — no UI delta: the annotations are in `tools/list`, which no Marley screen draws, and the
guide's rows are text. The run is `just shot`, and a client's `tools/list` is printed and
checked in `script/e2e/766-mcp-annotations.sh`.

## Locked-In Decisions
- **D1:** annotations are derived from the row's tier and two explicit lists in the registry,
  not written on each of the 66 rows. Rejected: a field on `ToolSpec` (66 edits, and a new tool
  could leave it at a wrong default).
- **D2:** every tool states all four hints, since MCP's defaults (destructive, open-world) are
  the wrong guess for most of Marley's tools.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `tools/list` shall give every tool an `annotations` object with a title and the four hints. | The client's `tools/list` (checked) |
| REQ-002 | A read-tier tool shall carry `readOnlyHint: true`; a write tool `readOnlyHint: false` with its destructive and idempotent hints as the lists say. | The client's `tools/list` (checked) |
| REQ-003 | The IDE server's `tools/list` shall mark its five tools read-only. | The review of the diff |
| REQ-004 | The guide shall list every tool the server serves. | The review (a check over the registry's names) |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — `registry.rs` (`annotations`, the lists, `tools_list_for`), `ide.rs`, the guide;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — `just shot`, and the scenario printing a client's `tools/list` annotations.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
