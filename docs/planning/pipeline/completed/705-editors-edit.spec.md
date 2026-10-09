---
pipeline_id: c7c3cd43-1ab0-4c4b-8c37-9811a3e9f265
ticket: docs/planning/tickets/open/TICKET-705-editors-edit.md
status: Phase 4 — Complete PASS
title: Editors over MCP: edit and save
type: feature
slice: Marley's MCP server (docs/planning/intake/zed-control-over-mcp.md), the third of #703 to #707
references:
  - docs/planning/pipeline/completed/704-editors-read-and-open.spec.md
---

## Title
An agent edits an open editor's text as one undoable, unsaved change, and saves it only after the
user says so.

## Scope
### In
- **`editor_edit`** (act): in an open editor named by id or path, it replaces `old_text` with
  `new_text`.
  - `old_text` must match once; with `replace_all` it may match many times.
  - The change is one transaction marked as an agent's (`BufferEditSource::Agent`), so one undo
    takes it back. It is left unsaved.
  - It answers the lines it changed.
  - A file matching the secret globs is refused.
- **`editor_save`** (sensitive): it writes an open editor's buffer to its file.
- **The sensitive level:** a sensitive tool asks every time, even in a session allowed for acts,
  unless the area is `allow`. `off` refuses it, as it refuses the area's other tools.

### Out (explicitly deferred)
- A review view of an agent's edit, the inline Keep/Reject of Zed's agent: `AgentDiff` is not
  exported from `agent_ui`. For now one undo, Agent Activity and the git gutter show and take back
  a change.
- Edits by line and column ranges; edits to files no editor has open.

## Reference (§20)
Upstream Zed, language and agent: Zed's own agent edits a buffer as `start_transaction`, `edit`,
then `end_transaction_with_source(BufferEditSource::Agent)` (`crates/agent/src/tools/
edit_session.rs`), and saves through `Project::save_buffer`. Marley does the same through public
APIs, with no Zed hunk. The `old_text`/`new_text` shape is the one agents' own edit tools use.

### Prior art
- **The code we ship:** `agent_edit_buffer` in `edit_session.rs`, `Project::save_buffer`,
  `Buffer::text`, and #704's `editor_tools` (`editor_named`, the secret globs) and
  `agent_control::admit`.
- **Published:** Claude Code's Edit tool and Codex's apply-patch both replace exact old text,
  unique unless told otherwise.

## UI proof
`script/e2e/705-editors-edit.sh`, under `compositor sway`, with #704's scripted MCP client.

Shots:
- `705-01-edited`: after `editor_open` (allowed for the session) and `editor_edit`
  `let answer = 42;` → `let answer = 43;`, the editor shows 43 and its tab is unsaved; the file on
  disk still says 42 (checked).
- `705-02-save-asked`: `editor_save` asks, although the session was allowed.
- `705-03-saved`: after Allow, the tab is saved and the file says 43 (checked). Then a second edit
  and one Ctrl+Z bring the text back to 43 (checked through `editor_read`).

## Locked-In Decisions
- **D1:** edits are exact text replacements, unique unless `replace_all`.
- **D2:** a save is sensitive: it asks every time unless the area is `allow`.
- **D3:** no review view in this ticket; undo is the take-back.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `editor_edit` with an `old_text` found once, the system shall replace it in the open buffer as one undoable change and leave it unsaved. | Shot 705-01; the file on disk; the undo check |
| REQ-002 | WHEN `old_text` is found nowhere or more than once without `replace_all`, the system shall refuse with `no_match` or `ambiguous` and change nothing. | The client's reply |
| REQ-003 | WHEN an agent calls `editor_save` while the area is not `allow`, the system shall ask the user every time, and write the file only after Allow. | Shots 705-02, 705-03; the file on disk |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the registry rows and schemas, `agent_control`'s levels, and `editor_tools`'s edit
  and save; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check, every shot read.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close, archive,
  commit.
