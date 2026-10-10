# Thread tabs after a restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-736-thread-tabs-after-a-restart.md
- **Pipeline spec:** 736-thread-tabs-after-a-restart.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - #494 (Browser tabs restored) and #576 (no `UNIQUE(item_id)` in an items table).
  - AD-claude-609: the Agent tab was kept out of `SerializableItem` until a store served its history; here Zed's thread store does.
  - `rusty/graph_store.rs:215` is the simplest template: one JSON `state` column.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `rusty/graph_store.rs`: `persistence::MarleyRustyGraphTabsDb` (a JSON `state` column,
    `PRIMARY KEY(workspace_id, item_id)`, no `UNIQUE(item_id)`), a global map loaded at `init` so
    `deserialize` reads it at once, `save_tab`, `cleanup` through `workspace::delete_unloaded_items`;
    `rusty/graph_tab.rs`'s `impl SerializableItem` and `register_serializable_item` (104).
  - `ConversationView::new(…, resume_session_id, thread_id, work_dirs, title, …)`; the panel takes
    the session id from `ThreadMetadataStore::entry(id).session_id`
    (`create_agent_thread_with_server`, `agent_panel.rs:4592`).
  - `ThreadId` derives `Serialize`/`Deserialize`; `ConversationView::{parent_id, agent_key,
    title, root_thread, marley_own_folders}`; `Agent::id()`; `AcpThread::work_dirs()`.
  - `Workspace::database_id()` (7628).

### Design
- **`thread_tab.rs`** (Marley crate):
  - `SavedThreadTab { thread_id, agent, folders, own_folders, title }` (serde), and a nested
    `mod persistence` with `MarleyThreadTabsDb` (table `marley_thread_tabs`), `save_tab` and
    `all_tabs`, after `graph_store`; `SavedThreadTabs`, a global map loaded in `init`.
  - `impl SerializableItem for ThreadTab`: kind `MarleyThreadTab`; `serialize` takes the view's
    thread id, agent id, its root thread's `work_dirs()` (else the record's folders), the flag and
    the title; `should_serialize` on `UpdateTab`; `cleanup` drops rows for items not loaded;
    `deserialize` refuses a tab whose thread has no record, waits for the panel, re-joins the
    folder when the tab was #734's and the folder needs it, and builds the view with the record's
    session id.
  - `open_thread`'s view building moves into `build_view(…)`, shared by a new tab and a restored
    one.
- **File manifest:** `thread_tab.rs` only (Marley crate); the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Two bound keys: a thread in the repo, "hello"; a thread on `other/`, "noted"; quit; launch | 736-01-before, 736-02-after |
| 002 | "again" in the restored root thread | 736-03-typed and the agent log |
| 004 | `read <other>/notes.md` in the restored `other/` thread | 736-04-folder |
| 003 | — | The review of the diff |

### Risks
- **A trust prompt after the relaunch.** The repo was trusted in the first launch; if Zed asks
  again, the scenario's Enter answers it, and the shot shows it.
- **Session load replay.** The scripted agent replays from its own log; a real agent replays from
  its own store, as the Agent Panel's history relies on.

## Phase 2 — Code
- **Checklist** (no TaskCreate here): `thread_tab.rs` ✓, guide ✓, scenario (before the gate) ✓,
  gate ✓.
- **Built (`thread_tab.rs`):**
  - `ThreadSpec` and `build_view`, shared by a new tab (`open_thread`) and a restored one: the
    panel's connection store, `ThreadStore::global` for Zed's agent, the record's session id.
  - `ThreadTab::saved`: the thread id, the agent's id, the root thread's `work_dirs()` (else the
    record's folders), the `marley_own_folders` flag and the title; `None` before Zed has folders
    for the thread.
  - `impl SerializableItem for ThreadTab` (`MarleyThreadTab`): `serialize` on `UpdateTab`;
    `cleanup` through `delete_unloaded_items`; `deserialize` refuses a tab whose thread has no
    record, waits up to ten seconds for the workspace's Agent Panel, re-joins a #734 folder that
    needs a hidden worktree (the tab holds it), and builds the view.
  - `SavedThreadTab`, `SavedThreadTabs` (loaded in `init`), `save_tab`, `saved_tab`, and the
    `persistence` module with `MarleyThreadTabsDb` (`marley_thread_tabs`), after `graph_store`.
  - `init` registers the item.
- **The guide:** "After a restart" in the thread-tab section.
- **Deviations:** none from the design.
- **Review of the diff:** REQ-001 the item is registered and saved on every tab update; REQ-002 the
  restored view resumes the record's session; REQ-003 no record → `Err`, so Zed shows no tab and the
  cleanup drops the row; REQ-004 a #734 tab's folder joins again before the view. Re-entrancy: the
  restore reads the workspace through `read_with` in its own task, never inside an update.
  Provenance: the store follows Marley's `graph_store`; the view's construction is #734's.
- **Gate:** `736-gate-1.log` RED, gate:2 only: `useless_let_if_seq` (the hidden worktree as an `if`
  expression now) and `redundant_closure_for_method_calls` (`Workspace::panel::<AgentPanel>`).
  `736-gate-2.log`: GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/736-thread-tabs-after-a-restart.sh`, under `compositor sway`. A scripted
  agent that loads a session by replaying the prompts its log holds for it, and answers `read`
  through Marley's `fs/read_text_file`. Two keys bound to `marley::NewAgentThread`: Marley on
  `other/` and Marley in the project's root. The first run (`shots-736a`) passed every check but
  its last shot came from the wrong tab: Ctrl+PageUp stayed in the message box, so the read ran in
  the root thread (and still succeeded, since the restored `other/` tab had re-joined the folder
  to the project). The scenario clicks the tab instead (`shots-736b`).
- **The Test phase's run (`shots-736-test`), after `just build` and `736-gate-2.log` green: every
  check passes.**
  - **736-01-before (REQ-001):** the repo's tabs repo — bash, first, hello; "hello" answered
    "Noted: hello"; the rail lists hello and first under repo.
  - **736-02-after (REQ-001):** after `zed: quit` and a new launch on the same profile, the same
    three tabs in the same order, hello in front with "hello" and "Noted: hello" replayed. The
    check: the agent log holds a `session/load`.
  - **736-03-typed (REQ-002):** "again" answered "Noted: again" under the restored turn; the log
    holds the prompt.
  - **736-04-folder (REQ-004):** the first tab, its turn "first" / "Noted: first" replayed, and
    `read …/other/notes.md` answered "Read: # notes from other": its folder joined the project
    again.
  - **REQ-003:** the review of the diff.
  - Focus report: one Marley window before and after on Chad's Hyprland, no rule added.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: Thread tabs come back after a restart); the architecture
  note (`marley_workbench.md`, "Restored after a restart", and #697's line about serializing); the
  slice line in `workbench-shell.md`; the guide came with Phase 2. No Zed path is touched.
- **Knowledge appended:** L-claude-736-a-restored-item-waits-for-its-workspaces-panels-001. No bug
  was found in Code or Test beyond clippy's two findings and the scenario's tab switch.
- **Brain:** no `rusty` MCP server in this repository's sessions; no brain loop ran.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** `736-gate-2.log`, GATE GREEN [diff], 17 passed, on the tree committed.
