---
pipeline_id: 9703d032-051a-4672-bfff-834b62879634
ticket: docs/planning/tickets/open/TICKET-697-an-agent-thread-in-a-center-tab.md
status: Phase 4 — Complete PASS
title: An agent thread in a center tab
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), on Chad's 2026-10-08 ask
references:
  - docs/planning/pipeline/completed/696-the-marley-and-rusty-agents-out-of-the-box.spec.md
---

## Title
Move an Agent Panel thread into a tab of the center pane and back, so an agent conversation can
sit in the main content beside terminals and files.

Chad, 2026-10-08: "is there any way that its possible we are able to move the agent panel into
the main content if we wanted? … lets do #3".

## Scope
### In
- `marley: open thread in center` moves the Agent Panel's active thread into a center tab:
  - the same `ConversationView`, still running, with its history and its message box;
  - the panel turns to a new draft (`AgentPanel::clear_base_view`, which keeps the thread
    retained), so the view is never drawn twice;
  - a thread already in a tab brings that tab forward.
- `marley: move thread to panel`, from the active thread tab or the tab's right-click menu,
  gives the thread back to the panel and closes the tab:
  - `activate_retained_thread` while the panel still retains it;
  - otherwise `load_agent_thread` from its history.
- The tab:
  - its title is the thread's title, kept current, and its icon the agent's;
  - it closes itself when the panel makes its thread active again (from the panel's own
    history), so a thread lives in one place.
- The rail:
  - a thread row's right-click menu gains **Open in Center**;
  - clicking a thread row whose thread is in a center tab brings that tab forward.

### Out (explicitly deferred)
- A new, empty draft in the center. A draft has no thread to move, and the panel would draw the
  same draft again. The command says to send a first message.
- Thread tabs across a restart (`SerializableItem`). After a restart the thread is in the panel's
  history.
- A button in the Agent Panel's toolbar, which would be a Zed hunk.

## Reference (§20)
Upstream Zed, workspace and agent panel. A center `Item` that hosts a `ConversationView` is
the pattern of Zed's own `ThreadViewItem` in `agent_ui/src/conversation_view.rs`'s tests. The
panel's retained threads (`clear_base_view`, `activate_retained_thread`, `load_agent_thread`)
are `agent_ui`'s public API, so no Zed hunk is needed. No Warp analog: Warp has no Agent Panel
to move from.

### Prior art
- **The code we ship:**
  - `ThreadViewItem` (Zed's tests) shows a `ConversationView` as an `Item`: it renders the
    view and delegates focus to it.
  - `AgentPanel` keeps a thread that leaves its base view in `retained_threads`. Running or
    unloadable threads are never evicted, and an idle loadable one is evicted beyond
    `max_idle_retained_threads`.
  - `load_agent_thread` activates a retained thread or loads one from storage.
  - `Item::tab_extra_context_menu_actions` adds entries to a tab's right-click menu.
  - `ItemEvent::CloseItem` lets an item close itself.
- **Marley:** the rail's `show_thread` and thread row menu (`rail.rs`), and
  `add_item_to_center` as the Marley layout's terminals use it.
- **Behavior maps:** `docs/zed_architecture/` has no section on moving a panel's content to the
  center. Zed's own feature for this is `agent: open thread as markdown`, a read-only copy, not a
  live thread.

## UI proof
`script/e2e/697-an-agent-thread-in-a-center-tab.sh`, under `compositor sway`. The Marley entry
runs a scripted agent (`MARLEY_ASSISTANT_ADAPTER`) that answers prompts.

Shots:
- `697-01-in-center`: after a first message, the thread in a center tab, and the panel on a new
  draft.
- `697-02-typed`: a second message typed in the tab, with its answer.
- `697-03-back`: the thread back in the panel, the tab gone.
- `697-04-from-rail`: the rail row's Open in Center.

## Locked-In Decisions
- **D1:** the tab hosts the panel's own `ConversationView` entity. It is not a copy, so a running
  turn keeps streaming into it.
- **D2:** `clear_base_view` is the move. The thread stays in the panel's retained threads while it
  can, and moving it back reactivates that same entity.
- **D3:** a thread is drawn in one place. The tab closes itself when the panel shows its entity
  again, and the rail's click on such a row brings the tab forward.
- **D4:** the code lives in a Marley module (`thread_tab.rs`), with the actions registered on the
  workspace. There is no Zed hunk.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs `marley: open thread in center` with a thread active in the Agent Panel, the system shall show that thread in a center tab titled as the thread and show a new draft in the panel. | Shot 697-01 |
| REQ-002 | WHILE a thread is in a center tab, a message typed in the tab shall reach the agent and its answer shall show in the tab. | Shot 697-02; the scripted agent's log |
| REQ-003 | WHEN the user runs `marley: move thread to panel` from the tab, the system shall show the thread in the Agent Panel and close the tab. | Shot 697-03 |
| REQ-004 | WHEN the user chooses Open in Center on a rail thread row, the system shall show that thread in a center tab. | Shot 697-04 |
| REQ-005 | WHEN the active panel thread is an empty draft, the command shall leave it in place and say to send a first message. | Review of the diff |
| REQ-006 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan:** this spec, and the design in the notes.
- **P2 Code:** `thread_tab.rs` and the rail's hooks; a review of the diff; the gate green.
- **P3 Test:** the 697 scenario, with every shot read.
- **P4 Complete:** the CHANGELOG, the guide and the architecture note, the ledger, then close the
  ticket, archive and commit.
