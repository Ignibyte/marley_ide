# Palette actions over MCP — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-707-palette-actions.md
- **Pipeline spec:** 707-palette-actions.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-09, full control of Zed over MCP; the intake's fifth ticket.
- **Classification:** feature; Marley crates plus the settings paths (rows extended).
- **Recall (§18.3):**
  - AD-704 (areas, asking once per session), AD-705 (levels) and AD-706 (the panel's own paths).
  - The intake's refused set.
  - A static scan of `actions!` (a regex over the macros) proved unreliable for checking names,
    so the names are checked at runtime: `action_list`'s `unknown`.
- **Discovery:**
  - `App::build_action(name, Option<Value>)`, `all_action_names`, `action_documentation`;
  - `Window::dispatch_action(Box<dyn Action>, cx)`, `is_action_available(&dyn Action, cx)`;
  - Marley's own action namespaces are `marley` and `rusty`. They include
    `marley::ResumeAgentControl` and Rusty's Secrets, so both are refused outright.

### Design
- **Settings (rows first):**
  - `MarleyAgentControlContent.actions: Option<MarleyAgentControlMode>` and
    `actions_allowed: Option<Vec<String>>`;
  - `default.json` (`"actions": "ask_first"`, `"actions_allowed": []`);
  - the Agent Control section's Actions dropdown.
- **`registry.rs`:** `Family::Action` (`action`, served), with `action_list` (Read) and
  `action_run` (Write, `action.write`) in `action_schemas`; count 52. `dispatch.rs` defers it.
  `mcp.rs` grants `action.write` and routes `action_*`.
- **`agent_control.rs`:** `Area::Actions`.
- **`action_tools.rs` (new):**
  - `ALLOWED`: about 40 names (docks, panels, splits, pane navigation, search, go-to, format,
    folds, finders).
  - `REFUSED_NAMESPACES` and `REFUSED_WORDS`; `refused(name)` takes the namespace before `::`
    and checks the local name with `contains`.
  - `user_allowed(cx)`; `list`; `run`:
    - refused → `action_refused`;
    - not allowed → `action_not_allowed`, with `next` naming `actions_allowed`;
    - `build_action` failing → `bad_action`;
    - the built action's `name()` checked against the refusal again;
    - the target workspace activated in its window;
    - `window.is_action_available` false → `not_available`;
    - else `window.dispatch_action`.
- **File manifest:**
  - Marley crates: `marley_mcp` (`registry.rs`, `dispatch.rs`); `marley_workbench`
    (`action_tools.rs` new, `agent_control.rs`, `mcp.rs`, `marley_workbench.rs`).
  - Zed paths: `settings_content/src/marley.rs`, `default.json` and `marley_page.rs`.
  - Docs: the touchpoints and the guide.
  - The scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `action_list` | the reply holds `unknown=[]` and `workspace::ToggleLeftDock` |
| 002 | `action_run workspace::ToggleLeftDock` in the background | 707-01-run-asked |
| 002 | Allow for This Session | 707-02-dock-toggled |
| 002 | `action_run pane::SplitRight` (no question) | 707-03-split |
| 003 | `editor::SelectAll` → `action_not_allowed`; the profile names it; run → `ran` | the replies |
| 004 | `project_panel::Delete` → `action_refused`; `workspace::SendKeystrokes`, named in the profile → `action_refused` | the replies |

### Risks
- **An allowlisted name upstream lacks or renamed:** `unknown` shows it, and the scenario
  expects none.
- **`is_action_available` from the window's focus:** right after the question is answered,
  focus may sit on the notification. Docks and splits are workspace actions, which are
  available from anywhere in the workspace, so the shots show whether this holds.

## Phase 2 — Code
- **Built:**
  - **Settings:** `actions` and `actions_allowed`, their `default.json` lines, and the Actions
    dropdown (the rows were extended first).
  - **`registry.rs`:** `Family::Action` (served); `action_list` (Read) and `action_run` (Write,
    `action.write`) in `action_schemas`; count 52. `dispatch.rs` defers it.
  - **`mcp.rs`:** grants `action.write` and routes `action_*`.
  - **`agent_control.rs`:** `Area::Actions`.
  - **`action_tools.rs` (new):** `ALLOWED` (37 names), `REFUSED_NAMESPACES`, `REFUSED_WORDS`,
    `refused`, `user_allowed`, `list`, `target`, `run` and `dispatch_in`.
  - **`guide.md`:** the tools' lines.
- **Review:**
  - **A workspace switch** (`MultiWorkspace::activate`, when `project` names one not shown) left
    the availability check and the dispatch on the old frame. When the switch happens, both now
    wait for `window.on_next_frame`.
  - **Nothing focused:** the window's active pane is focused first, so the workspace's handlers
    are in the path.
  - **The refusal runs twice:** on the name given, and on `action.name()` after the build, which
    covers deprecated aliases.
  - **Dispatch** is deferred by gpui (`Window::dispatch_action`), so it doesn't run inside the
    `MultiWorkspace` update.
- **Gate:**
  - `707-gate-1.log` RED (rustdoc: the module doc linked the private `ALLOWED` and `refused`).
  - `707-gate-2.log` GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/707-palette-actions.sh`, under `compositor sway`. The client is
  #704's, rewritten for `action_list`, `action_run` and `editor_open`. The profile sets the
  editors area to `allow` and names `editor::SelectAll` and `workspace::SendKeystrokes`.
- **Run a (`shots-707a`): every check passed, but 707-02 was a red.**
  - `workspace::ToggleLeftDock` changed nothing on screen. Marley's rail is not the left dock, and
    the left dock is empty here, so the shot showed no criterion.
  - The scenario now toggles the right dock, which holds the project panel. No source change.
- **Run b (`shots-707b`): the split failed, and the tool was right.** The second toggle reopened
  the dock, and the project panel took the focus. From there `pane::SplitRight` is
  `not_available`, which `action_run` reported. The scenario now checks that refusal, clicks the
  editor, and splits from there.
- **Run c (`shots-707c`): every check passes, every shot shows its criterion.**
  - **Checks:**
    - `action_list` → `actions=38 unknown=[]`: all 37 allowlisted names are registered, plus
      `editor::SelectAll named`, and `SendKeystrokes` is left out as refused;
    - `editor_open` opened `main.rs` with no question;
    - both toggles ran, the second without a question;
    - the split from the panel → `refused not_available`; from the editor → `ran
      pane::SplitRight`;
    - `editor::SelectAll` ran;
    - `editor::DuplicateLineDown` → `action_not_allowed`;
    - `project_panel::Delete`, `zed::Quit` and the named `workspace::SendKeystrokes` →
      `action_refused`.
  - **707-01-run-asked (REQ-002):** "e2e-agent wants to use action_run", "run
    workspace::ToggleRightDock", the three buttons; the project panel is open on the right.
  - **707-02-dock-toggled (REQ-002):** the right dock and its project panel are gone; the editor
    takes the width.
  - **707-03-split (REQ-002):** two `main.rs` editors side by side, the dock back on the right,
    and the rail's Files listing both.
  - Chad's Hyprland untouched.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md` (Added);
  - `docs/marley_architecture/marley_workbench.md` (a section for `action_tools.rs`) and
    `marley_mcp.md`;
  - the guide (Phase 2).
  - The three touchpoint rows describe the shipped `actions` and `actions_allowed`, their
    defaults, and the dropdown.
- **Knowledge appended:**
  - AD-claude-707-palette-actions-run-from-an-allowlist-under-a-hard-refusal-001;
  - L-claude-707-marleys-rail-is-not-the-left-dock-and-an-opened-dock-takes-the-focus-001.
  - No source bug was found in Test: both reds were the scenario's.
- **Brain:** no `rusty` MCP server in this repository's sessions; the decisions are in the ledger.
- **Ticket:** closed; the BACKLOG row left at promotion. The intake's five tickets (#703 to
  #707) are all shipped.
- **Gate:** `707-gate-3.log`, GATE GREEN [diff], on the tree committed.
