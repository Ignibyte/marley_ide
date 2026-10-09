---
pipeline_id: f968d30a-5991-4acb-94bb-5c1e9cc52821
ticket: docs/planning/tickets/open/TICKET-704-editors-read-and-open.md
status: Phase 4 — Complete PASS
title: Editors over MCP: list, read and open
type: feature
slice: Marley's MCP server (docs/planning/intake/zed-control-over-mcp.md), the second of #703 to #707
references:
  - docs/planning/intake/zed-control-over-mcp.md
  - docs/planning/pipeline/completed/703-agent-control-layer.spec.md
---

## Title
An agent sees Zed's open editors and opens files in them through Marley's MCP server. The first
tool that acts in a new area brings the per-area mode and the once-per-session question.

## Scope
### In
- **`editor_list`** (read): every open editor in every window, each with its id, its file's
  absolute path, its project, whether it is dirty, its language, whether it is its pane's active
  tab, and its cursor and selections as line and column ranges (1-based).
- **`editor_read`** (read): an open editor's text by id or path, unsaved edits included. It is
  paged (`start_line`, at most 12,000 bytes or 2,000 lines a page) and redacted.
  - A file matching `marley.agent_control.secret_globs` is refused unless the user removes the
    pattern. The defaults are `.env*`, `*.pem`, `*.key`, `id_rsa*`, `id_ed25519*`,
    `*credentials*`, `.netrc` and `*.p12`.
  - Each read is listed in Agent Activity (#703).
- **`editor_open`** (act): a file inside an open project's folders, at an optional line and
  column, in that project's workspace, its tab in front.
- **The area mode** `marley.agent_control.editors`: `off`, `ask_every`, `ask_first` (the default) or
  `allow`.
  - `off` refuses the area's tools, reads too.
  - Under `ask_first` an act tool's first call from an agent asks: Allow for This Session, Always
    for This Project, or Deny. The question names the agent, the tool and what it acts on.
  - A session is the caller's terminal, or else its client name, and its project, for as long as
    Marley runs. "Always" is kept per area and project across restarts.
  - `ask_every` asks each time, and `allow` never asks.
- **The rename:** `editor_open`/`editor_wait` (#649, unlisted, for `marley-edit`) become
  `prompt_open`/`prompt_wait`, so the `editor_*` names are the agents'.

### Out (explicitly deferred)
- Editing and saving (#705); threads (#706); actions (#707).
- The rail mark while an agent acts.

## Reference (§20)
Upstream Zed, workspace and editor: `Workspace::items_of_type::<Editor>`, `Editor::buffer`'s
singleton buffer, `Buffer::{file, is_dirty, language}`, the selections, and `Workspace::open_abs_path`
then `go_to_singleton_buffer_point`, read through public APIs with no Zed hunk. No Warp analog read.

### Prior art
- **The code we ship:**
  - `mcp.rs`: `terminal_read`'s pager (`page`, `PAGE_BYTES`, `MAX_READ_LINES`, `Page::fill`),
    `for_agents` (redaction) and `terminals(cx)`'s walk of every window.
  - `agent_editor.rs`: `editor_open`'s `window_of` → `open_abs_path`.
  - `settings_change::ask_user`, the Apply/Decline question with a 25-second wait.
  - `agent_activity::gate` (#703).
  - `util::paths::PathMatcher` for the globs.
- **The intake** (`zed-control-over-mcp.md`) and an Explore pass (2026-10-09) on editors, buffers
  and opening at a line.

## UI proof
`script/e2e/704-editors-read-and-open.sh`, under `compositor sway`, with #703's scripted MCP
client. The repo holds `src/main.rs` and `.env`.

Shots:
- `704-01-asked`: the client's `editor_open` on `src/main.rs` line 3 shows the question naming
  e2e-agent, editor_open and the file.
- `704-02-opened`: after Allow for This Session, `main.rs` is in front with the cursor on line 3;
  a second `editor_open` ran without asking (checked).
- `704-03-activity`: Agent Activity lists the opens and the reads. `editor_list` named `main.rs`,
  active, with its line 3 cursor; `editor_read` returned the buffer's unsaved text; `editor_read`
  on `.env` was refused `secret_file` (each checked).

## Locked-In Decisions
- **D1:** the agents' tools take the `editor_*` names; the internal pair becomes `prompt_*`.
- **D2:** the session key is the caller's terminal, else its client name, plus its project; the
  app never sees MCP session ids.
- **D3:** "Always for This Project" is kept in Zed's key-value store per area and project.
- **D4:** `editor_open` opens only files inside an open project's folders.
- **D5:** a refused secret file is named in the refusal so the user knows which pattern to drop.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `editor_list`, the system shall answer every open editor with its id, path, project, dirty state, language, active state and selections. | The client's reply |
| REQ-002 | WHEN an agent calls `editor_read` on an open editor, the system shall answer its current text, unsaved edits included, paged and redacted, and list the read in Agent Activity; a file matching a secret glob shall be refused with `secret_file`. | The client's replies; shot 704-03 |
| REQ-003 | WHILE `marley.agent_control.editors` is `ask_first`, an agent's first `editor_open` in a session shall ask the user, and later calls in that session shall not. | Shots 704-01, 704-02; the second call's reply |
| REQ-004 | WHEN the user allows `editor_open`, the system shall open the file in its project at the line given, its tab in front. | Shot 704-02 |
| REQ-005 | The `marley-edit` prompt editor shall keep working through `prompt_open` and `prompt_wait`. | The review of the diff (the program and the routing) |
| REQ-006 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the rename; the registry rows and schemas; `editor_tools.rs`; `agent_control.rs`
  (the modes, the question, the session and project memory); the settings; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check, every shot read.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close, archive,
  commit.
