---
pipeline_id: 345bba4e-d995-4972-a30a-cd8b121d92c5
ticket: docs/planning/tickets/closed/TICKET-772-reopen-thread-and-threads-in-tabs.md
status: Phase 4 — Complete PASS
title: Reopen Thread…, and every thread opens in a tab
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/744-one-new-agent.spec.md
  - docs/planning/pipeline/completed/737-the-threads-page.spec.md
  - docs/planning/pipeline/completed/697-an-agent-thread-in-a-center-tab.spec.md
---

## Title
One way back to an old conversation, and no thread in Zed's Agent Panel in the Marley layout. The
second half of #744's split. Chad, 2026-10-11, of "New Agent from Thread": "my thought is that you
would re open it". Today a rail thread row, the rail's Archived Threads, the Needs you inbox's
Manager entry and a notification's click show a thread in Zed's Agent Panel on the right, and
the Threads page does when the panel holds it.

## Scope
### In
- **Reopen Thread…** (`marley::ReopenThread`, `marley: reopen thread`): a picker over the saved
  conversations Zed keeps, unarchived and not drafts, newest first: the agent's icon, the title,
  "agent · folder" and the time; typing matches title and folder. Enter opens the conversation in
  a tab of the group the picker opened in, or brings its tab forward where it is open. It sits
  under New Agent… in a rail row's `+` and in every pane's `+`.
- **One opening path** in the Marley layout, `thread_tab::show_here`: the thread's tab forward;
  else, when an Agent Panel holds it (shown or kept), the thread moves out of the panel into a tab
  of that panel's workspace; else `false`, and the caller opens it from Zed's store in a tab.
- Its callers: the Threads page (`threads_page::open_thread`'s panel branch), the rail's thread
  rows and Archived Threads (`rail::show_thread`, an archived record unarchived first), the
  inbox's Manager entry (a listed Manager thread through the same path; none starts one in a tab),
  and a notification's click (`MarleyThreadHost::reveal`).
- `harness::manager_in_front` also counts a shown thread tab on the Manager entry, so its unread
  records clear when the Manager is read in a tab.
- In the Marley layout the thread tab's menu has no Move Thread to Panel, the palette lists neither
  Move Thread to Panel nor Open Thread in Center, and a rail thread row has no Open in Center.
- The guide, the in-app page and the walkthrough.

### Out (explicitly deferred)
- Claude Code and Codex sessions from their own files in Reopen Thread…: #754.
- Zed's own ways into the right dock that start no thread (a dock restored open, `workspace: toggle
  right dock`, the panel's settings page): #747 and #748.
- Zed's layout: every opener keeps today's panel behavior there.

## Reference (§20)
Upstream Zed, `agent_ui`: the Agent Panel's thread history (`ThreadsArchiveView`), its retained
threads (`activate_retained_thread`, `is_retained_thread`, `clear_base_view`) and
`ThreadMetadataStore`. Behavior kept: Zed's store is the list, Zed's `ConversationView` draws the
thread, a running thread keeps its connection when it moves. Marley changes where it shows in its
own layout: a tab instead of the panel. Zed's picker (`picker`, `fuzzy`) is the list.

### Prior art
- **The code we ship:** `thread_tab::open_in_center` (#697) already moves the panel's active
  thread into a tab (`clear_base_view` then a `ThreadTab` on the same view); `activate_for`
  brings a tab forward; `open_saved` (#737) opens a store record in a tab. `threads_page::open_thread`
  is the funnel with a panel branch. `AgentPanel::activate_retained_thread(id, false, …)` makes a
  kept thread the panel's view without focusing it. The New Agent picker (#735, `agents.rs`) is
  the picker pattern (`Picker::uniform_list`, `fuzzy::match_strings`). `MarleyNewItemMenu`
  (#744) holds the pane's `+`. `routing.rs`'s `CommandPaletteFilter` use and
  `assistant::filter_talk` hide actions by state.
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (pickers in the
  modal layer). T3 Code's report 02 item 2 (past sessions with Resume) is #754's.
- **Published material:** ACP's `session/load`, as #736 uses it.

## UI proof
`script/e2e/772-reopen-thread-and-threads-in-tabs.sh`, `compositor sway`, with #744's fixtures (the
Marley entry on a scripted agent that answers "Noted: <prompt>" and replays a session on
`session/load`).
- `772-00-plus`: the repo row's `+` with Reopen Thread… under New Agent… (REQ-007).
- `772-01-reopen`: a Marley thread asked "first", its tab closed, then the repo row's `+` → Reopen
  Thread…: the picker lists it with its title, agent, folder and time (REQ-001).
- `772-02-reopened`: Enter: a tab in the repo with "first" and its answer; `session/load` logged
  (REQ-002).
- `772-03-again`: Reopen Thread… on it again: the same tab forward, one Marley tab (REQ-003).
- `772-04-row`: the tab closed, the thread's rail row clicked: a tab in the repo, no Agent Panel
  (REQ-004).
- `772-05-tab-menu`: the thread tab's right-click menu without Move Thread to Panel (REQ-006).

## Locked-In Decisions
- **D1 — One path that never shows the panel** in the Marley layout: the tab forward, else out of
  the panel into a tab, else a tab from the store. A thread is never drawn twice (#697's rule).
  Rejected: leaving the panel branch for threads the panel holds (the panel would open on the right,
  the thing Chad asked to end).
- **D2 — Reopen Thread… is a picker**, not the Threads page: keys, Enter, the group it was asked
  from. Unarchived and non-draft conversations only; the page and Archived Threads reach the rest.
- **D3 — An archived conversation opened from the rail is unarchived first,** as the panel's load did
  (#616), so its row comes back under its project.
- **D4 — The Manager with no thread starts in a tab** (`thread_tab::start` on the Manager entry),
  not in the panel; `agents::start_thread` goes with its last caller.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Reopen Thread… opens, the picker shall list the saved unarchived conversations, newest first, each with its title, agent, folder and time. | Shot 772-01 |
| REQ-002 | WHEN a conversation with no open tab is chosen, the system shall open it in a tab of the picker's group with its earlier turns. | Shot 772-02 and `session/load` in the log |
| REQ-003 | WHEN a conversation whose tab is open is chosen, the system shall bring that tab forward and open no second one. | Shot 772-03 |
| REQ-004 | WHILE the Marley layout is used, WHEN a thread row in the rail is clicked, the system shall show the thread in a tab and not show the Agent Panel. | Shot 772-04 |
| REQ-005 | WHILE the Marley layout is used, WHEN a thread an Agent Panel holds is opened from Reopen Thread…, the Threads page, the rail, the inbox's Manager entry or a notification, the system shall move it into a tab of that panel's workspace without showing the panel. | The review of the diff |
| REQ-006 | WHILE the Marley layout is used, the thread tab's menu and the palette shall not offer Move Thread to Panel, and a rail thread row shall not offer Open in Center. | Shot 772-05; the review |
| REQ-007 | The rail row's `+` and every pane's `+` shall list Reopen Thread… under New Agent…. | Shot 772-01; the review for the pane |
| REQ-008 | The change shall pass `script/gates.sh --diff`. | The gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `thread_tab.rs` (`show_here`, the reveal, the menu and palette), `threads_page.rs`
  (the picker and the funnel), `rail.rs` (the `+`, `show_thread`, the row menu, the Manager),
  `agents.rs` (the pane's `+`), `harness.rs` (`manager_in_front`), the in-app guide; a review;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check.
- **P4 Complete** — CHANGELOG, guide, walkthrough, architecture note, ledgers, close, archive, commit.
