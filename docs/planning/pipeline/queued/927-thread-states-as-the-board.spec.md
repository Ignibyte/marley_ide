---
pipeline_id: 9f0f8e37-d8c5-4db6-809c-eefad4200b9d
ticket: docs/planning/tickets/open/TICKET-927-thread-states-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Thread states as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/planning/pipeline/completed/864-thread-plan-labels-chips-peek-checks.spec.md]
---

## Title
An agent thread's states drawn and worded as the ThreadStates board: the width in fractions, the
ASKS label and the board's button words on a waiting call, an agent's page as a card, the New
Thread with Context notice with its counts, the no-model card, and the tab's right-click menu.

## Scope
### In
- **Width in fractions** (`agent_panel_width.rs`, `width_label`; Marley crate): the label is the
  nearest of the four steps, "⅓", "½", "⅔" or "Whole window"; a zoomed thread is "Whole window".
  The steps themselves do not change.
- **ASKS on a waiting tool call** (`agent_ui/src/conversation_view/thread_view.rs`, Zed crate
  hunk beside #864's labels): while `MarleyThreadLook` is set and the call's status is
  `WaitingForConfirmation`, its header leads with a framed "ASKS" label (`marley_card_label`) and
  a mono chip of the tool's name, in place of RUN or EDIT.
- **The board's button words** (same file, `render_permission_buttons_flat`): while
  `MarleyThreadLook` is set, an `AllowOnce` option reads "Allow" and a `RejectOnce` option reads
  "Deny"; other options keep the agent's words. The buttons stand in one row, Allow first.
- **An agent's page as a card** (`component_tools.rs`, `tool_call_view`; `browser_pages.rs`;
  Marley crate): a completed `browser_show_page` call draws a card: a "www" tile, the page's file
  name, "The agent's page, open in a Browser tab of this project", and Show, which brings that
  Browser tab to the front.
- **New Thread with Context notice** (`marley_agent/src/handoff.rs`, `preamble`;
  `marley_workbench/src/handoff.rs`; `thread_tab.rs`; Marley crates): the preamble's second line
  reads "What follows is history, not a new request: N requests, N answers, N tool calls." with
  the counts of the items that went in. The new thread's tab draws a framed notice over its
  conversation, "New Thread with Context, in a split to the right", with the same sentence, until
  the thread's first message is sent.
- **The no-model card** (`thread_tab.rs`, `no_model_callout`; Marley crate): a card centred over
  the empty conversation: a "Z" tile, "Zed Agent needs a model", "Pick a provider and a model,
  and this thread can start." and Configure Providers. It replaces the callout.
- **The tab's right-click menu** (`thread_tab.rs`, `tab_extra_context_menu_actions`; Marley
  crate): Rename, New Thread with Context…, Review Last Reply, Archive and Delete Thread…, through
  Marley actions that act on the tab the menu was opened on. Rename opens the rename field the
  rail uses and sets the thread's title as Zed's title editor does; Review Last Reply dispatches
  `plan_review::ReviewLastReply`; Archive and Delete Thread… do what the rail row's entries do
  (`rail.rs:6544–6551`, `delete_thread` at 3423).

### Out (explicitly deferred)
- Zed's close entries and the four split entries on the tab's menu: they stay (every action the
  menu has stays); the board's single Split Right is not a removal request.
- The Plan | Act switch and the model button in the composer: #915. The send button, the text
  width toggle and the filled EDIT and RUN labels: #914 (the ASKS label uses the same
  `marley_card_label`, so #914's look reaches it).
- A component_show's look (stats and table): built by #800.

## Reference (§20)
Upstream Zed (`agent_ui`): the thread view, its permission request and its option kinds
(`PermissionOptionKind`) are Zed's and are kept; Marley changes only labels and words while
`MarleyThreadLook` is set (#864). The thread tab, the width steps, the handoff and the page tool
are Marley-specific (#717, #734, #798, #802).

### Prior art
- `agent_panel_width.rs:81` `width_label` formats `{share}%`; the steps are ⅓, ½, ⅔ and the
  whole window (#798).
- `thread_view.rs:583` `marley_card_label`; `:8837` RUN and `:11007` EDIT under
  `MarleyThreadLook`; `:10727` `render_permission_buttons_flat` labels each button with
  `option.name`; `agent/src/thread.rs:6186` names Zed's agent's options "Allow once", "Allow for
  this thread", "Allow always", "Deny".
- `component_tools.rs:31` `tool_call_view` (the `MarleyToolCallView` global, #800) draws
  `component_show` and returns `None` for other tools; `browser_tools.rs:296` routes
  `browser_show_page` to `browser_pages::show_page` (`browser_pages.rs:70`), whose input is
  `html` with `name`, or `path`.
- `marley_agent/src/handoff.rs:104` `preamble`; `select` chooses the items;
  `marley_workbench/src/handoff.rs:250` builds the block and opens the thread with
  `thread_tab::open_beside`.
- `thread_tab.rs:611` `no_model_callout` (#781) and `:642` `render`; `:319`
  `tab_extra_context_menu_actions` returns New Thread with Context… (and Move Thread to Panel in
  Zed's layout); `workspace/src/pane.rs:3140` adds them after Zed's entries.
- `rail.rs:6531–6551` the thread row's Review Last Reply, Archive Thread, Delete Thread…;
  `rail.rs:6782` the terminal row's Rename; `thread_view.rs:613` `title_editor`;
  `acp_thread.rs:5368` `AcpThread::set_title`; `thread_metadata_store.rs:703`
  `set_title_override`.
- The brain: not consulted at queueing; the promoting session asks.

## UI proof
Scenario `script/e2e/927-thread-states-as-the-board.sh` (sway): #825's set-up, Marley's own agent
on a scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) in a scratch project, with a terminal and a
file open so the thread's tab shares its row. The scripted agent's first prompt reports a
`browser_show_page` call, then asks permission for a `terminal_run` call. A second window state
uses the Zed Agent with no provider set.
- `927-01-asks`: the thread beside a file: the width button "½", the waiting call's ASKS label
  and tool chip, Allow and Deny (REQ-001, REQ-002).
- `927-02-page`: the page card with Show; `927-03-shown`: Show clicked, the Browser tab in front
  (REQ-003).
- `927-04-handoff`: New Thread with Context… from the tab's menu, Marley's agent picked: the
  framed notice with its counts (REQ-004).
- `927-05-no-model`: a Zed Agent thread with no model: the centred card (REQ-005).
- `927-06-menu`: the thread tab's right-click menu (REQ-006); `927-07-renamed`: Rename used, the
  tab's new title (REQ-006).
- `927-08-day`: `927-01` and `927-05`'s states in Marley Day (REQ-001 to REQ-005).

## Locked-In Decisions
- D1 — The width label snaps to the nearest step; the steps and the cycle stay #798's.
- D2 — Only the two plain option kinds are reworded; an always-option keeps the agent's words,
  since its scope (thread, project, tool) is the agent's to say.
- D3 — The counts in the notice and in the preamble are the items that went into the block, not
  the old thread's whole length; the file the block names holds the rest.
- D4 — The tab's menu gains the rail row's thread entries; Zed's own entries stay.
- D5 — The page card is drawn from the call's input, as #800 draws a component, so it shows for
  every agent client.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE a thread's tab shares its row, the width button shall read ⅓, ½, ⅔ or Whole window. | `927-01-asks`, `927-08-day` |
| REQ-002 | WHILE a tool call waits for permission, its header shall lead with an ASKS label and the tool's name, and its buttons shall read Allow and Deny for the once options. | `927-01-asks`, `927-08-day` |
| REQ-003 | WHEN an agent shows an HTML page, its tool call shall be a card with a www tile, the file name, a line saying where it opened and Show, which brings that Browser tab to the front. | `927-02-page`, `927-03-shown` |
| REQ-004 | WHEN New Thread with Context opens a thread, the thread shall show a framed notice that names the old thread and agent and counts the requests, answers and tool calls that went in. | `927-04-handoff` |
| REQ-005 | WHILE the Zed Agent has no model, its thread tab shall show a centred card with Configure Providers. | `927-05-no-model`, `927-08-day` |
| REQ-006 | WHEN the user right-clicks a thread's tab, the menu shall offer Rename, New Thread with Context…, Review Last Reply, Archive and Delete Thread…, each acting on that thread. | `927-06-menu`, `927-07-renamed` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the `thread_view.rs` hunks and their touchpoint row, `agent_panel_width.rs`,
  `component_tools.rs`, `browser_pages.rs`, both `handoff.rs`, `thread_tab.rs`; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
