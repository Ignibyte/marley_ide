---
pipeline_id: 0fc73235-d273-414e-ba06-f31c96e72322
ticket: docs/planning/tickets/open/TICKET-707-palette-actions.md
status: Phase 4 — Complete PASS
title: Palette actions over MCP
type: feature
slice: Marley's MCP server (docs/planning/intake/zed-control-over-mcp.md), the last of #703 to #707
references:
  - docs/planning/pipeline/completed/706-agent-panel-threads.spec.md
---

## Title
An agent runs Zed's palette actions by name, the ones Marley lists as safe and the ones the user
names; the rest are refused, and a set that could hand an agent the user's consent or run code is
refused even when named.

## Scope
### In
- **`action_list`** (read): the actions an agent may run. Each has its name, its documentation,
  and whether the user allowed it by name, followed by `unknown`: allowlisted names that no
  action is registered under (an upstream rename).
- **`action_run`** (act): `name`, with optional JSON `arguments` and `project` (a folder name or
  path).
  - `App::build_action` builds the action. It is dispatched in the window that holds the
    project's workspace (else the active window), from that window's focus, as the command
    palette does.
  - An action the focused element can't take is refused `not_available`.
- **Who may run what:**
  - Marley's allowlist (navigation, docks and panels, splits, search, formatting and go-to)
    runs under the area's mode.
  - A name in `marley.agent_control.actions_allowed` runs the same way.
  - Anything else is refused `action_not_allowed`, with how to allow it.
  - Refused outright, even when named (`action_refused`): the namespaces `agent`, `acp`,
    `assistant`, `task`, `debugger`, `repl`, `notebook`, `git`, `extensions`, `cli`,
    `auto_update`, `client`, `collab`, `marley` and `rusty`, and any name holding Quit,
    Restart, Reload, CloseWindow, OpenBrowser, OpenZedUrl, SendText, SendKeystroke, Allow,
    Reject, Authorize, Approve, Deny, Delete, Trash, Install, SignIn, SignOut, Share, Run,
    Rerun, Spawn, Push or Discard. Aliases are checked by the built action's own name too.
- **The area mode** `marley.agent_control.actions` (`ask_first`).

### Out (explicitly deferred)
- A Settings UI for `actions_allowed` (it is edited in JSON).
- Key bindings as a channel (`workspace::SendKeystrokes` is refused outright).

## Reference (§20)
Upstream Zed, gpui:
- `App::{build_action, all_action_names, action_documentation}`;
- `Window::{dispatch_action, is_action_available}`;
- command_palette, which dispatches the chosen action from the window's focus.

No Zed hunk beyond the settings paths.

### Prior art
- **Zed's command palette:** lists `window.available_actions` and dispatches with
  `window.dispatch_action`.
- **#704 to #706:** the area modes, the levels, and the activity log.
- **The intake's argument for an allowlist:** a deny list leaks, because upstream adds actions
  at every merge.

## UI proof
`script/e2e/707-palette-actions.sh`, under `compositor sway`. #704's scripted MCP client
(`e2e-agent`) runs the actions.

Shots:
- `707-01-run-asked`: the first run asks (actions area, `ask_first`).
- `707-02-dock-toggled`: after Allow, `workspace::ToggleRightDock` hid the right dock and its project panel.
- `707-03-split`: `pane::SplitRight` ran with no second question: two panes.

The client's replies show the rest:
- `action_list` answers `unknown=[]`;
- `project_panel::Delete` → `action_refused`;
- `workspace::SendKeystrokes` named in `actions_allowed` → `action_refused`;
- `editor::SelectAll` → `action_not_allowed`, until the profile names it, then it runs.

## Locked-In Decisions
- **D1:** an allowlist plus the user's names, with an outright refusal over both.
- **D2:** `action_run` is Act, so under `ask_first` it asks once per session.
- **D3:** dispatch goes from the target window's focus, as the palette does.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `action_list`, the system shall answer the actions it may run, with their documentation, and the allowlisted names no action is registered under. | The client's reply |
| REQ-002 | WHEN an agent runs an allowlisted action and the user allows it, the system shall dispatch it in the target window as the palette would. | Shots 707-01, 707-02, 707-03 |
| REQ-003 | WHEN an agent runs an action neither allowlisted nor named by the user, the system shall refuse it with how to allow it; a named one shall run. | The client's replies |
| REQ-004 | WHEN an agent runs an action in a refused namespace or with a refused word, the system shall refuse it even when the user named it. | The client's replies |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec and the notes' design.
- **P2 Code** — settings, the registry's `action` family, `Area::Actions`, `action_tools.rs`;
  gate.
- **P3 Test** — the visual check.
- **P4 Complete** — docs, ledger, close, commit.
