# A thread's row drags to another group — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-818-drag-a-thread-to-another-group.md
- **Pipeline spec:** 818-drag-a-thread-to-another-group.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19: "i cant drag a rusty or agent across terminal groups. … i opened
  a rusty in one and tried to move it to default and couldnt". Asked which threads should move
  (Rusty and Marley only, every thread, Rusty only): "Every thread".
- **Classification / tier:** bug (a gap #613 left); Marley crate (`rail`, `rail_order`,
  `thread_tab`) and one Zed crate (`agent_ui`, one method).
- **Recall (§18.3):**
  - L-claude-613: `move_item` crosses workspaces; stale handles unless `added_to_workspace`
    re-points them. #613's `TerminalView::added_to_workspace` hunk is the pattern.
  - L-claude-602-style drags in e2e: several pointer steps with short sleeps; one jump reaches no
    target (lessons.md, #613's note).
  - The brain (consultation 31198302f2b34bcaadb71962b7ada968): nothing on this seam.
- **Discovery:** `rail_order.rs` `moves_to` (terminals only) and the header drop's
  `Selection::Terminal` arm; `rail.rs` `move_terminal`, `ThreadEntry { workspace, thread_id,
  agent, work_dirs, title, closed, .. }`, `load_thread(workspace, ThreadToOpen, ..)`;
  `thread_tab.rs` `tab_threads`, `ThreadTab { conversation_view, _folder, .. }` with no
  `added_to_workspace`; `conversation_view.rs` `workspace` field, `as_connected().threads`.
- **Decisions:** D1 to D3 in the spec.

### Design
- `rail_order.rs`: `moves_to` matches `Selection::Terminal(_) | Selection::Thread(_)`; the
  header's drop calls `move_terminal` or `move_thread`.
- `rail.rs`: `move_thread(&self, key, project, window, cx)`, beside `move_terminal`: the target
  group's workspace (not closed); deferred to the window: when the thread's workspace holds a
  `ThreadTab` for its `thread_id` (`items_of_type::<ThreadTab>`, by `parent_id`), `move_item` from
  that tab's pane to the target's active pane; otherwise `load_thread(target, ThreadToOpen ..)`;
  then `activate_workspace(target)`.
- `thread_tab.rs`: `ThreadTab` gains `added_to_workspace`, calling
  `conversation_view.marley_set_workspace(workspace.weak_handle(), cx)`; and a
  `pub(crate) fn tab_for(workspace, thread_id)` for the rail.
- `conversation_view.rs` (Zed crate): `pub fn marley_set_workspace(&mut self, workspace, cx)`:
  sets `self.workspace` and each connected thread view's `workspace`; a no-op for the same one.
- File manifest: `crates/marley_workbench/src/{rail_order.rs, rail.rs, thread_tab.rs}` (Marley),
  `crates/agent_ui/src/conversation_view.rs` (Zed, its touchpoint row first),
  `script/e2e/818-drag-a-thread-to-another-group.sh` (new).

### Visual check plan
- As the spec's UI proof: #807's set-up (scripted adapter, Rusty stand-in), `repo-b` handed over
  as #613 does; `marley: new rusty conversation` and a prompt in `repo`; drag its row (steps, as
  #613's `drag_to_repo_b`) onto Default's header; a prompt in the moved thread; `marley: new
  marley conversation` in `repo`, its row dragged onto `repo-b`'s header; `quit_marley`,
  `launch_marley`. Points measured from the first run's shots.

### Risks
- Workspace handles other than the conversation's and its thread views' (the message editor's
  completion provider) keep the old workspace; they read its project for mentions, which is the
  agent's project anyway.

## Phase 2 — Code
- Built to the manifest. `rail_order.rs`: `moves_to` takes `Selection::Thread(_)` beside
  `Selection::Terminal(_)`; the header's drop matches the selection, `move_terminal` or
  `move_thread`. `rail.rs`: `move_thread`, deferred like `move_terminal`: the source workspace's
  `ThreadTab` for the thread (`thread_tab::tab_for`, by `parent_id`) moves with `move_item` to the
  target's active pane; with no tab (or a closed group's thread, whose workspace handle is gone)
  `load_thread` opens it in the target; then `activate_workspace`. `thread_tab.rs`: `tab_for`,
  and `Item::added_to_workspace` calling `ConversationView::marley_set_workspace`.
  `conversation_view.rs` (Zed, its touchpoint row extended first): `marley_set_workspace` sets the
  view's `workspace` and each connected `ThreadView::workspace`, a no-op for the same workspace.
- Scenario: written to the plan; check runs measured the points (the first guess dragged the
  `repo — bash` terminal row; the Marley row and `repo-b`'s header move once Default holds two
  rows), and the relaunch needed `open_path ""` so the session restores, as #613's does.
- Review: the drop target's `can_drop` still refuses the row's own group and closed groups;
  `added_to_workspace` runs at a first add too, where the workspace is the same and nothing
  changes; the agent's project and the tab's hidden worktree (#734) stay with the source project,
  as D2 says; nothing reads or updates an entity while it is updated (the conversation is updated
  from the tab's update, a different entity).
- The gate: `just gate-diff` green, 17 of 17.

## Phase 3 — Test
- `just e2e script/e2e/818-drag-a-thread-to-another-group.sh` on the debug build after the green
  gate, under sway; focus report "1 Marley windows before the run, 1 after"; the `expect` passes.
  Shots read:
  - `818-01-rusty-in-repo`: `repo` lists `repo — bash`, New Agent Thread (Marley, the cat) and
    hello rusty (Rusty, the butler); both threads' tabs are in `repo`'s pane.
  - `818-00-layout`: `repo-b` handed over, shown; Default's header at y 88, the Rusty row at 385.
  - `818-02-rusty-in-default`: Default lists Home and hello rusty, highlighted; the window shows
    Default, its tabs Home and hello rusty; `repo` lists only its terminal and the Marley thread
    (REQ-001, REQ-003).
  - `818-03-answers`: in that tab, "still here" and "Noted: still here" under the first exchange:
    the conversation carried on; the scripted agent's log holds the prompt (REQ-004).
  - `818-04-marley-in-repo-b`: `repo-b` lists `repo-b — bash` and New Agent Thread; the window
    shows `repo-b` with the thread's tab; `repo` keeps its terminal (REQ-002).
  - `818-05-restored`: after `zed: quit` and a launch with no path, Default lists hello rusty and
    `repo-b` New Agent Thread (REQ-005); `repo` comes back closed, as a group not visited does
    (#617).
- Not run without the fix: Chad's report is the before (the drop did nothing for a thread row).

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #818); `docs/marley_architecture/marley_workbench.md`, the #613
  bullet; the `zed-touchpoints.md` row for `conversation_view.rs` read against what shipped. No
  slice status moves.
- Ledger appends: `L-claude-818-a-moved-item-keeps-its-project-but-not-its-workspace-001`. No
  `F-` block: a gap #613 left by design, not a fault.
- Brain: consultation `31198302f2b34bcaadb71962b7ada968` closed with `brain decide` (follow-up
  2026-11-19).
- Ticket closed; it had no `BACKLOG.md` row (filed at Plan).
