# Threads and ports under a closed project — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-617-rows-under-a-closed-project.md
- **Pipeline spec:** 617-rows-under-a-closed-project.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones".
- **Batch:** wave 2 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-606 deferred a closed project's threads and ports "since they need its workspace"; the store queries need only its path list and host.
  - `group_threads` needs a workspace for `ThreadEntry.workspace`, the agent's icon and name, and live statuses; each has a way round for a closed group.
  - `project_folders` takes roots from the group's workspaces only, so a closed group gets none today.
  - F-claude-606-closed-headers-would-have-shared-their-element-ids-001: closed rows take their ids from `closed_id`.
- **Discovery:** one Explore sweep for the wave (2026-09-30) over the rail, the ports scan, the
  thread store, terminal views and tooltips; the spec's Prior art cites what applies here, and the
  Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted** from `queued/`; every seam re-verified. Line numbers moved since /spec:
  `group_threads` 6287, `push_closed` 6759, `open_closed_project` 3461, `open_thread` 2393 (it
  now shares `open_thread_with`, #616), `project_folders` 521 and `attribute` 556 in `ports.rs`.
- **Recall (§18.3):**
  - AD-claude-606: a closed project was listed header only, its threads and ports deferred.
  - F-claude-606-closed-headers-would-have-shared-their-element-ids-001: a closed group's ids come
    from `closed_id`; its rows take theirs from the thread key and the port as everywhere.
  - #616: `archive_thread` and `delete_thread` need no workspace, so a closed project's thread
    row keeps both in its menu as they are.
  - The brain (`rusty-cli brain ask`, consultation 9f43fda308fb41679328d0da638aea1c): nothing on
    this seam.
- **Prior art:** Zed's sidebar opens a closed group's thread (`open_workspace_and_activate_thread`,
  `load_agent_thread_in_workspace`): the thread's folders under the group's key, then the Agent
  Panel loaded if the new workspace has none yet. Marley calls the same public pieces
  (`find_or_create_workspace`, `AgentPanel::load`, `add_panel`, `load_agent_thread`).

### Approach
- **`ports.rs`:** `project_folders` takes a group with no workspace and no host from its key's
  path list (D2). `project_names` stops leaving closed groups out, so `ports_list` names their
  ports as the rail does.
- **`rail.rs`, the snapshot:** `push_closed` gets the group's threads and ports.
  `group_threads(group, listed: Option<&Entity<Workspace>>, names_from: &Entity<Project>, cx)`:
  for a closed group, `listed` is none, the entry's workspace a `WeakEntity::new_invalid()`, and the
  icon and name come from the shown workspace's project (D4). `ThreadEntry` gains
  `closed: Option<ProjectGroupKey>`.
- **`rail.rs`, opening:** `open_closed_project` becomes `open_closed(key, paths, window, cx) ->
  Task<Option<Entity<Workspace>>>`, its toast kept, with `open_closed_project` calling it for the
  header. A new `in_workspace(workspace, closed: Option<(ProjectGroupKey, PathList)>, then, window,
  cx)` runs `then` in the shown workspace, or once the closed project has opened.
  `open_thread_with` and `open_port` go through it; `show_logs` becomes a method and does too. The
  thread's load moves into `load_thread(workspace, thread, window, cx)`, which loads the Agent
  Panel with `AgentPanel::load` when the workspace has none yet and adds it if it is still missing.
- **`rail.rs`, rendering:** the header shows its chevron when a closed project has rows and the
  filter is off; `render_port_row` and `PortMenu` carry the group's closed key. `open_row`'s Port
  arm passes it.
- **`marley_rail`:** `cycle_row` passes over rows whose project is closed (D5).

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_workbench/src/ports.rs` | Marley | closed groups' folders and names |
| `crates/marley_workbench/src/rail.rs` | Marley | threads and ports under a closed header; opening through `in_workspace`; the chevron |
| `crates/marley_rail/src/marley_rail.rs` | Marley | `cycle_row` passes over closed projects |
| `script/e2e/617-rows-under-a-closed-project.sh` | e2e | the scenario |

No Zed crate changes.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | `repo` and `repo-b` open; a stand-in thread in `repo-b`; `python3 -m http.server` started in `repo-b`; `repo` shown; restart with no path | `closed.png`: `repo-b` dimmed, its thread and port rows |
| REQ-004 | click `repo-b`'s chevron, then again | `folded.png`, then the rows back |
| REQ-002 | click the thread row | `thread.png`: `repo-b` open, the thread in its Agent Panel |
| REQ-003 | show `repo`, restart again, double-click the port row | `port.png`: `repo-b` open, a Browser tab on the port |
| REQ-005 | no scenario: Next Thread is bound to a chord the sway run does not need; review of `cycle_row` | — |

### Risks and decisions
- Loading the Agent Panel ourselves races Zed's own initialization; both add it only when the
  workspace still has none, as Zed's sidebar relies on.
- A closed remote project's ports are not scanned (its folders are on the other host); its threads
  are listed.

## Phase 2 — Code (2026-09-30)
- **Built:**
  - `ports.rs`: `project_folders` gives a closed local group its key's folders; `project_names`
    names closed groups too, so `ports_list` does.
  - `marley_rail::cycle_row` passes over a closed project's rows.
  - `rail.rs`: `ThreadEntry.closed`; `group_threads` takes an optional listed workspace and the
    shown one for agent names; `listed_threads` shared by open and closed groups; `push_closed`
    lists threads and ports; `header_folds` gives a closed header with rows its chevron, whose
    click stops before the header's own click opens the project; `open_closed` returns the opened
    workspace, `open_closed_project` detaches it; `in_workspace` runs an action in a row's
    workspace or once a closed project has opened; `open_thread_with`, `open_port` and
    `show_logs` go through it; `load_thread` and `show_thread` load the Agent Panel when a just
    opened workspace has none yet. The archived menu of a closed header opens the project first.
- **Deviations from the plan:** none in substance. Show Logs now shows the row's project too,
  since it goes through `in_workspace`; before, a project not shown got its logs terminal unseen.
- **Review:**
  - Re-entrancy: `in_workspace` updates the workspace inside the rail's update as the openers did
    before; the deferred path updates it from a spawned task.
  - Provenance: `load_thread` is written against public API (`AgentPanel::load`, `add_panel`,
    `load_agent_thread`), the order the API sets; no Zed function body was carried over.
  - Errors: a project that does not open shows #606's toast; a panel that does not load is logged,
    as Zed's sidebar logs it.
- **Clippy found:** `obfuscated_if_else` in `project_folders` (now an `if`), and
  `render_project_row` at 104 lines (the fold test moved into `header_folds`).
- **The scenario** `script/e2e/617-rows-under-a-closed-project.sh` is written ahead of Test so one
  gate run covers it.
- **Gate:** `just gate-diff` GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/617-rows-under-a-closed-project.sh` (sway): a server in `repo-b` before
  Marley starts; `repo-b` handed over (#513) and given two threads of #605's stand-in agent, "first
  617" then "second 617"; `repo` shown and Marley started again with no path, twice.
- **Run 1:** every criterion's shot was taken, but `thread.png` showed the Agent Panel's
  "Failed to Launch: Loading or resuming sessions is not supported by this agent": #605's stand-in
  declares no `loadSession`, so no thread of it can be opened after a restart. A scenario limit,
  not the change. The stand-in now keeps each session's messages beside it and answers
  `session/load` by replaying them.
- **Run 2:** the thread loaded, but it was the thread the restored panel shows anyway, so the shot
  could not tell the click's work from Zed's restore. The scenario now makes two threads and clicks
  the older.
- **Run 3, the shots read:**
  - `before.png`: `repo-b` open with its terminal, "second 617", "first 617" and the port row
    `:55117 python3`.
  - `closed.png` (REQ-001): `repo-b`'s header dimmed, with its chevron; under it "second 617" and
    "first 617" (Stand-in · idle, the stand-in's icon) and the port row; `repo` shown below.
  - `folded.png` (REQ-004): `repo-b`'s chevron points right and its rows are gone; `repo` is still
    the shown project, so the chevron's click did not open `repo-b`. `unfolded.png`: the rows back.
  - `thread.png` (REQ-002): `repo-b` open (title bar, its terminal row), the Agent Panel on
    "first 617" with its message and the stand-in's answer, the row selected.
  - `closed-again.png`: `repo-b` closed again after the second restart.
  - `port.png` (REQ-003): `repo-b` open, a Browser tab on `http://127.0.0.1:55117/` showing
    "Served from repo-b", its row selected.
- **REQ-005** (Next and Previous Thread pass over a closed project's threads): reviewed in
  `cycle_row`; no scenario, since the run needs no keyboard chord for it.
- **Not reached:** Show Logs and Restart on a closed project's service row (the scenario's server is
  no unit; both go through the paths the port row's Open and #615's scenario exercise), and a
  closed remote project's threads.
- **Log:** `acp_thread` "Failed to prevent idle sleep" twice: the headless sway has no portal.
  Pre-existing, not in scope.
- **No source change in Test;** only the scenario changed, so the gate runs again at Complete.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; `workbench-shell.md`; `marley_workbench.md` (a Rows under a
  closed project bullet after Closed projects); the guide, the guide page (Quit and come back) and
  the walkthrough (a check in 2.2).
- **Knowledge:** L-claude-617-a-stand-in-agent-without-loadsession-cannot-prove-a-thread-reopens-001,
  AD-claude-617-a-closed-projects-rows-open-the-project-first-001. No `F-` block: the review and
  clippy findings were style, and Test found only scenario limits.
- **Brain:** consultation 9f43fda308fb41679328d0da638aea1c closed with `brain decide`.

