---
pipeline_id: fe5dac6e-3c33-42bd-9fbc-a1b0bfc682eb
ticket: docs/planning/tickets/open/TICKET-915-thread-composer-and-details.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The thread's composer and details panel
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md]
---

## Title
In a thread Marley hosts, the composer shows a chip above its text for each thing the draft
mentions, and an agent that offers a plan mode gets a Plan / Act segmented switch. The details
panel draws each section as a card; its first card holds the agent with a live state line, the
context bar, and two tiles, Folder and Mode; the terminal peek ends with "Open in a tab".

## Scope
### In
- `crates/agent_ui/src/conversation_view/thread_view.rs` (Zed crate, under `MarleyThreadHost`):
  - `render_message_editor`: a row of chips above the editor, one per mention in the draft, in
    the order they appear: the mention's name (`MentionUri::name`) in the buffer font on
    `element_background`, 24 px tall, 7 px radius. No chips, no row.
  - The bottom row: where the session's modes include one whose id is `plan`, a two-segment
    switch, Plan and Act, in place of Zed's mode dropdown. Plan sets the plan mode; Act sets the
    last other mode this thread used, else the first other mode the agent lists. Act carries a
    chevron that opens Zed's mode menu, so every mode stays one click away. An agent with no plan
    mode keeps Zed's dropdown.
- `crates/agent_ui/src/mode_selector.rs` (Zed crate): `all_modes()`, a read of the connection's
  modes for the switch.
- `thread_tab.rs` (Marley), `render_details`:
  - Each section a card: 12 px radius, `elevated_surface_background`, a border, its label in
    small capitals inside, as the board's.
  - The first card, THIS THREAD: the agent's tile and name, and under it the state line in the
    state colour: "Working · <what the newest running tool call says>", "Working", "Needs you"
    while a tool call waits for permission, or "Idle"; then the context bar (#825); then a
    two-tile grid, Folder (the thread's folder name, in the buffer font) and, when the agent has
    modes, Mode ("Plan", or "Act · <mode name>").
  - ITS TERMINAL: under the peek, an "Open in a tab" link that opens a terminal tab in the
    window's active pane over the same terminal the agent ran (`TerminalView` over the call's
    `terminal::Terminal`).

### Out (explicitly deferred)
- Removing a mention from its chip: a chip shows what the message carries; the crease in the text
  stays the way to edit it.
- The model button as "Opus 5.5 · High" text in the composer: a follow-up ticket with the rest of
  the composer's looks (Zed's model selector stays).
- Checks as gate and visual-check results: #864's Out ("A task run's result among the checks").
- The plan inside the answer's flow: #864's Out.
- A block reference chip ("block · cargo test…") and @-mentioning a block: the AgentTerminal
  board's items, a follow-up ticket (no block mention kind exists in Zed's mention menu).
- The answer's header, labels, chips, send and width buttons: TICKET-914.
- The bottom row for a thread (model, mode, elapsed, context, files): TICKET-907. Its
  `thread_activity` helper says what a thread's agent does; whichever of 907 and 915 lands second
  draws the details panel's state line from it.

## Reference (§20)
Upstream Zed (`agent_ui`'s message editor, mention set and mode selector): mentions stay Zed's
creases and Zed's mode machinery sets the mode, so the agent sees exactly what Zed's menu would
send. Marley adds the chips' row, the switch's drawing and the panel's layout.

### Prior art
- `thread_view.rs:5004-5170` `render_message_editor`: the editor in a column, the bottom row's
  left group (add context, follow, width buttons) and right group (`config_options_view`, else
  `mode_selector` and `model_selector`, then send); #825's card hunk at :5044.
- `mention_set.rs:190` `MentionSet::mentions()`; `message_editor.rs:758` `mention_set()`;
  `acp_thread/src/mention.rs:21` `MentionUri` and `:336` `name()`.
- `mode_selector.rs:46` `mode()`, `:50` `set_mode` (also saves the agent's default mode), `:24`
  `menu_handle()`, `:79` the menu built from `connection.all_modes()`;
  `acp_thread/src/connection.rs:372` `AgentSessionModes { current_mode, all_modes, set_mode }`.
- `thread_tab.rs:697-787` `render_details`; `:757` the agent's icon, name and model; `:848`
  `terminal_peek` finds the newest `ToolCallContent::Terminal` and its `terminal::Terminal`
  (`terminal.inner()`); `:868` `peek_section`; `:960` `section_label`; `:121` the thread's
  `folder`.
- `acp_thread::AcpThread::status()` (`ThreadStatus::{Idle, Generating}`), its tool calls'
  `status()` (`WaitingForConfirmation`) and labels: the state line's sources.
- `terminal_view::TerminalView::new` over an existing terminal entity: how the thread view embeds
  a call's terminal (the card's `entry.terminal(terminal)`).
- Grep for "context chip" and "last block" in `agent_ui` and `marley_workbench` (the review's):
  none.

## UI proof
Scenario `script/e2e/915-thread-composer-and-details.sh` (sway): #864's set-up, Marley's own agent
on the scripted ACP agent (`MARLEY_ASSISTANT_ADAPTER`) in a scratch crate. The scripted agent
offers three modes (`default`, `plan`, `bypassPermissions`), and for the prompt it runs a command
through the client's terminal that prints a line a second for six seconds, then answers.
- `915-01-composer`: "@" and the crate's `src/main.rs` picked, a word typed: the chip
  `main.rs` above the text, Plan / Act with Act lit (REQ-001, REQ-002).
- `915-02-plan`: Plan clicked: Plan lit, the details panel's Mode tile "Plan" (REQ-002, REQ-004).
- `915-03-working`: Act clicked and the prompt sent, during the command: the panel's cards, the
  state line "Working · <the command's label>", Folder and Mode tiles, the peek with Open in a
  tab (REQ-003, REQ-004, REQ-005).
- `915-04-tab`: Open in a tab clicked: a terminal tab with the command's output (REQ-005).
- `915-05-day`: Marley Day, the composer with a chip and the panel (REQ-001 to REQ-004).

## Locked-In Decisions
- D1 — The chips are a reading of the draft's mentions, not a second editor: they change as the
  creases change and carry no actions.
- D2 — Plan / Act appears only for an agent that offers a mode with id `plan`; the switch goes
  through Zed's `ModeSelector::set_mode`, so the agent and the saved default mode behave as Zed's
  menu makes them.
- D3 — Act remembers the thread's last non-plan mode in the thread view, not in settings.
- D4 — The state line reads the thread itself (status, the newest in-progress or waiting tool
  call), never a seat: Marley's agent and Zed's agents have no seat.
- D5 — Open in a tab shows the same terminal entity, so it stays live while the agent runs;
  closing the tab leaves the agent's terminal running.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the draft holds mentions, the composer shall show a chip for each above its text, in Marley Night and Marley Day. | `915-01-composer`, `915-05-day` |
| REQ-002 | WHERE the agent offers a plan mode, the composer shall show a Plan / Act switch with the current one lit, and a click shall set that mode. | `915-01-composer`, `915-02-plan` |
| REQ-003 | The details panel shall draw each section as a card. | `915-03-working`, `915-05-day` |
| REQ-004 | The details panel's first card shall show the agent's state line in its state colour and the Folder and Mode tiles. | `915-02-plan`, `915-03-working` |
| REQ-005 | WHEN Open in a tab under the terminal peek is clicked, a terminal tab shall open over the terminal the agent ran. | `915-03-working`, `915-04-tab` |
| REQ-006 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the chips' row, the switch, the panel's cards, state line, tiles and link, the
  ledger rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
