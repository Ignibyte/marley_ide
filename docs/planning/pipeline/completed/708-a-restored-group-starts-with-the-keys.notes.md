# A restored group starts with the keys — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-708-a-restored-group-starts-with-the-keys.md
- **Pipeline spec:** 708-a-restored-group-starts-with-the-keys.spec.md

## Phase 1 — Plan
- **Request:** #700's Test phase found that after a relaunch nothing has focus (L-700); #702 found
  the mechanism. Chad asked the session to work through every open item.
- **Classification:** bug, the Marley layout, Marley crate only.
- **Recall (§18.3):**
  - F-702 and PR-702: a workspace made behind the shown one takes the window's focus;
    `keep_focus` gives it back once the workspace's task is done.
  - L-700: the symptom after a relaunch, worked around in #700's and #701's scenarios with a click.
  - Zed's `restore_multiworkspace` restores only the active workspace; Marley's `groups::reopen`
    then reopens each group's workspace with `open_workspace_by_id(…, Some(window))`, behind the
    shown one: the same mechanism, at restart.
- **Discovery:** `groups::reopen` (groups.rs, #601) spawns one task per group; `rail::keep_focus`
  and `ensure_groups` (#702).

### Design
- `groups.rs`: `pub(crate) fn keep_focus(focused: Option<WeakFocusHandle>, window, cx)`, moved
  from `rail.rs` unchanged; `reopen` takes `window.focused(cx)` (weak) at its start, and each
  group's task, once `opened` resolves, updates the window (`handle`) to call `keep_focus`.
- `rail.rs`: `ensure_groups` calls `groups::keep_focus`.
- **File manifest:** `crates/marley_workbench/src/groups.rs`, `crates/marley_workbench/src/rail.rs`
  (Marley crate); the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Rusty on; trusts repo; clicks Home's header (Home shown); `quit_marley` from the palette; `open_path ""`; `launch_marley`; with no click, Ctrl+Shift+P | 708-02-palette: the palette open |
| 002 | — | The gate |

### Risks
- A user who clicks during the reopen keeps that focus: `keep_focus` acts only when the handle it
  took lost the focus, and then gives it back, so a click in the first moments may be undone once.

## Phase 2 — Code
- **Built:** `groups::keep_focus`, moved from `rail.rs` (#702) and shared; `groups::reopen` takes
  `window.focused(cx)` (weak) at its start, and each group's task, once `open_workspace_by_id`
  resolves, updates the window to call `keep_focus`; a failed reopen still drops its record and
  now returns before the give-back. `rail::ensure_groups` calls `groups::keep_focus`.
- **Deviations:** none.
- **Review of the diff:** the give-back runs in its own window update after the reopen's task,
  never inside another entity's update; a window that closed meanwhile logs and ends. With nothing
  focused when `reopen` starts, nothing is given back (the Test phase checks that case through the
  scenario's first key).
- **Gate (`708-gate-1.log`):** GATE GREEN [diff].

### Phase 2, revised in Test
The first build (`groups::keep_focus` in `reopen`, gate `708-gate-1.log` green) failed the visual
check, so the code changed in Test and the gate ran again (below). The Code phase's design was
replaced:
- **Removed:** `keep_focus` and its uses (rail `ensure_groups`, `groups::reopen`); `groups.rs` is
  back to its committed state.
- **Zed `workspace.rs`:** `pub fn marley_is_shown` (over `owns_window_chrome`); an early return
  unless `owns_window_chrome()` at the head of `Workspace::new`'s focus-lost listener; in
  `new_local`'s `OpenMode::Add` arm and `open_workspace_by_id`'s requesting-window branch, the
  window's focus taken before `Workspace::new` and given back right after `MultiWorkspace::add`.
- **Zed `zed.rs`:** `initialize_workspace`'s closing focus also requires `marley_is_shown()`.
- **Ledger:** the `workspace.rs` and `zed.rs` rows extended before each edit.

## Phase 3 — Test
- **Scenario:** `script/e2e/708-a-restored-group-starts-with-the-keys.sh`, under `compositor sway`,
  with no database and Rusty on (the stand-in `rusty-mcp`): Enter on the trust prompt, Home's
  header clicked, a quit from the palette, a relaunch with no path, then Ctrl+Shift+P with no click.
- **Red, run 1 (`shots-708a`):** with Rusty on, the trust prompt took no Enter at start and the
  quit timed out. #702's fix covered Home alone.
- **Diagnosis, temporary logs (all removed):**
  - A focus log in `Rail::refresh` (`diag-708.sh`, a start with Rusty on): the project's pane, then
    Home's pane, then Rusty's pane, then the prompt; and `keep_focus` then gave the focus to the
    project's pane, taken before the prompt opened, over the prompt.
  - A backtrace on every `Window::focus` (in gpui, removed): four places move it.
    1. `Workspace::new` focuses its new pane (workspace.rs).
    2. `zed::initialize_workspace` focuses every new workspace unless it has a modal (zed.rs).
    3. The prompt, opening, records the focus it took as the one to give back on close
       (modal_layer.rs), which was a hidden pane.
    4. On the prompt's close, every workspace's focus-lost listener ran, and the last registered,
       a hidden one, won.
  - `focus_lost_restore_target` gives the old focus's nearest focusable ancestor, not the old
    focus: with only the listener guard, the Home-only start ended on the prompt's parent and Enter
    missed.
- **Fix:** the three Zed hunks above, `keep_focus` removed.
- **Diagnostics after the fix:** both starts trusted on Enter: Rusty on with no database
  (`diag-708.sh`), and Home only on the copied profile (`diag-trust.sh`, the #702 case).
- **Gate after the fix:** `708-gate-2.log`, GATE GREEN [diff].
- **Final runs:**
  - **708-01-before-quit (`shots-708c`):** Enter trusted the project, and Home's header took the
    click. Home is shown with its page (Start, New Agent with Claude Code, Codex, Gemini CLI and
    OpenCode, Recent Projects, Agents at Work, Configure); the rail lists Home, Rusty and repo.
  - **708-02-palette (REQ-001):** after the quit and the relaunch, Ctrl+Shift+P with no click
    opened the command palette over Home's page; the rail lists Home, Rusty and repo, closed.
  - **#702's scenario again (`shots-702e`), since its fix was replaced:** the scripted agent's
    check passes (the start's Enter and the palette took their keys); 702-01 shows one marked row
    for the thread in its tab and 702-02 the tab in front again.
  - Chad's Hyprland untouched in each run.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Fixed: keys after a relaunch and with Rusty on);
  `docs/marley_architecture/marley_workbench.md` (#702's focus paragraph now names #708's Zed
  hunks); the `workspace.rs` and `zed.rs` rows of `docs/marley/zed-touchpoints.md`, checked against
  the hunks that shipped (the accessor, the listener guard, the two give-backs, the `zed.rs` clause).
- **Knowledge appended:** F-claude-708-702s-focus-fix-restored-a-stale-handle-and-missed-the-modals-record-001,
  PR-claude-708-a-workspace-added-in-the-background-never-takes-focus-001 (supersedes PR-702),
  L-claude-708-focus-lost-restores-an-ancestor-not-the-old-focus-001.
- **Brain:** no `rusty` MCP server in this repository's sessions; nothing recorded there.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** `708-gate-2.log`, GATE GREEN [diff], on the tree committed.
