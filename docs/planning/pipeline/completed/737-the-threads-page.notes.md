# The Threads page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-737-the-threads-page.md
- **Pipeline spec:** 737-the-threads-page.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - Zed's thread history is `ThreadsArchiveView` in its sidebar (`agents_sidebar::ToggleThreadHistory`); the Marley layout's rail replaced the sidebar, so the history went with it.
  - PR-claude-701: a page's handlers that update the workspace are plain closures over a weak handle.
  - The rail's archived threads (#616) match a project by `main_worktree_paths`; the page lists every record regardless.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `ThreadMetadataStore::{try_global, entries, archived_entries, entry}`;
    `ThreadMetadata::{thread_id, agent_id, display_title, folder_paths, updated_at, is_draft,
    archived, remote_connection}`.
  - `agents::{thread_icon, thread_agent_name}` (127, 143); `AgentIcon`.
  - `assistant::Entry::{Marley, Rusty}` and `name()` (150-165), private today.
  - `thread_tab::{activate_for, build_view, needs_worktree}` (#734, #736); `AgentPanel::{
    active_thread_id, is_retained_thread, activate_retained_thread}`.
  - Opening another workspace of the window: `MultiWorkspace::activate(workspace, None, window,
    cx)`, as the rail's `activate_workspace` (2552).
  - `Editor::single_line` and its `EditorEvent::BufferEdited` (as `rusty/brain.rs`).

### Design
- **New `crates/marley_workbench/src/threads_page.rs`** (Marley crate, a new component):
  - `actions!(marley, [OpenThreads])`, registered on every workspace; `open` brings the
    workspace's page forward or adds one to the center; nothing while AI is disabled.
  - `ThreadsPage { workspace, search: Entity<Editor>, agent: Option<AgentId>, archived_open,
    focus_handle, _subscriptions }`: it observes the thread store and the search editor.
  - `rows(cx)`: the store's entries (drafts left out) and archived entries, matched by the search
    (title or folders, case-insensitive) and the chip, newest `updated_at` first, split into
    Marley and Rusty (agent `Marley` or `Rusty`), All conversations, Archived.
  - Each row: the agent's icon, the title, "agent · folder" (`~` for the home folder), and the
    time; a click runs `open_thread` through a weak handle (PR-claude-701).
  - `open_thread`: a tab showing it anywhere in the window comes forward (its workspace shown,
    then `activate_for`); else a panel retaining it shows it; else `thread_tab::open_saved` in the
    page's workspace.
  - `impl Item` (title Threads, an icon), `Focusable`, `Render`.
- **`thread_tab.rs`:** `pub(crate) fn open_saved(workspace, record, window, cx)`: the record's
  agent, folders and session through `build_view`, the folder joining hidden when it needs to; a
  remote project's record refused with a toast.
- **`assistant.rs`:** `pub(crate) fn is_assistant(agent: &AgentId) -> bool`.
- **`marley_workbench.rs`:** the module and its `init`.
- **The guide:** a section for the page.
- **File manifest:** `threads_page.rs` (new), `thread_tab.rs`, `assistant.rs`,
  `marley_workbench.rs` (Marley crate); the guide; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | Three bound keys, a message each; `marley: open threads` | 737-01-page |
| 003 | Types `second` in the search field | 737-02-search |
| 004 | Clears it; clicks the Marley chip | 737-03-chip |
| 005 | Closes the `second/` tab; clicks the All chip; clicks its row | 737-04-open |
| 006 | — | The review of the diff |

### Risks
- **The user's own conversations** appear in the run's page (the profile is a copy); the checks
  look only for the run's titles, which are newest.

## Phase 2 — Code
- **Checklist** (no TaskCreate here): `threads_page.rs` ✓, `thread_tab.rs` ✓, `assistant.rs` ✓,
  `marley_workbench.rs` ✓, guide ✓, scenario (before the gate) ✓, gate ✓.
- **Built:**
  - `threads_page.rs` (new): `OpenThreads`; `open`; `ThreadsPage` (a search editor, the chosen
    agent, the archived switch; it observes the thread store and the editor); `sections` (drafts
    left out, newest `updated_at` first, the search over the title and the folders' paths, the
    chip); chips, sections and rows; `open_thread` (a tab anywhere in the window comes forward, a
    panel holding the thread shows it, else `thread_tab::open_saved`); `Item`, `Focusable`,
    `Render`.
  - `thread_tab.rs`: `open_saved` (the record's agent, folders, title and session through
    `build_view`; a folder needing it joins hidden; a conversation with no folder takes the
    workspace's default folder; a remote project's is refused with a toast).
  - `assistant.rs`: `is_assistant`.
  - `marley_workbench.rs`: the module and its `init`. The guide: "The Threads page".
- **Deviations:** none from the design.
- **Found before the gate:** compile errors (the `Empty` element's import, a row id from a
  `String`, a borrow held by `items_of_type` in an `if let`) and one clippy finding
  (`option_if_let_else`); all fixed at the source.
- **Review of the diff:** REQ-001/002 `sections`; REQ-003 `matches`; REQ-004 the chips; REQ-005
  `open_saved`; REQ-006 `open_thread`'s first two branches. Re-entrancy: the row's click is a plain
  closure over a weak handle (PR-claude-701); `open_thread` reads each workspace outside any update
  and updates one at a time. Provenance: Marley's own page over Zed's public store; the search rule
  follows `ThreadsArchiveView`'s fields (title, then folders) without its code.
- **Gate:** `737-gate-1.log`: GATE GREEN [diff], 17 passed.

## Phase 3 — Test
- **Scenario:** `script/e2e/737-the-threads-page.sh`, under `compositor sway`. #736's scripted agent
  runs the Marley entry and a custom agent server `Scripted` from the run's settings; three bound
  keys start Marley in the root and Scripted on `other/` and `second/`. The first run (`shots-737a`)
  showed the page right but missed the chips and the row (guessed coordinates); the second set
  them from its shots.
- **The Test phase's run (`shots-737-test`), after `just build` and `737-gate-1.log` green: every
  check passes.**
  - **737-01-page (REQ-001, REQ-002):** the page in the repo: the search field, the chips All,
    Scripted, Marley, Claude Agent, Rusty (the last two from the profile's own archived
    conversations); MARLEY AND RUSTY with "marley first · Marley · …/repo"; ALL CONVERSATIONS with
    "second plan · Scripted · …/second" and "other notes · Scripted · …/other", each with its
    time; Show archived (15).
  - **737-02-search (REQ-003):** "second" in the field: only "second plan" under All
    conversations.
  - **737-03-chip (REQ-004):** the Marley chip chosen: only "marley first", and Show archived (4)
    for Marley's archived ones.
  - **737-04a-before-open:** the `second/` tab closed: its tab and rail row gone, the page still
    listing it.
  - **737-04-open (REQ-005):** its row clicked: a "second plan" tab in the repo with "second plan"
    and "Noted: second plan" loaded, its rail row back; the check: a `session/load`.
  - **REQ-006:** the review of the diff.
  - Focus report: one Marley window before and after on Chad's Hyprland, no rule added.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: The Threads page); the architecture note
  (`marley_workbench.md`, "The Threads page"); the slice line in `workbench-shell.md`; the guide
  came with Phase 2. No Zed path is touched.
- **Knowledge appended:** AD-claude-737-marleys-thread-history-is-a-page-over-zeds-store-001.
  The compile and clippy errors in Phase 2 were caught before any run; no bug reached a scenario.
- **Brain:** no `rusty` MCP server in this repository's sessions; no brain loop ran.
- **Ticket:** closed; its BACKLOG row left at promotion.
- **Gate:** `737-gate-1.log`, GATE GREEN [diff], 17 passed, on the tree committed.
