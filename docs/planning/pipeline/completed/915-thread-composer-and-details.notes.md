# The thread's composer and details panel — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-915-thread-composer-and-details.md
- **Pipeline spec:** 915-thread-composer-and-details.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 5 (the agent thread).
- **Classification / tier:** feature, M. `agent_ui` hunks for the composer, Marley's
  `thread_tab.rs` for the panel.
- **Recall (§18.3):**
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the panel reads the root
    thread view from Marley's own tab, outside that view's render; keep it so (no read from
    inside a thread view callback).
  - PR-claude-701-an-items-handlers-that-update-its-workspace-are-not-listeners-001: Open in a tab
    adds an item to the workspace from the thread tab's own click; write it as a plain closure
    over a weak workspace, deferred out of the tab's update.
  - F-claude-893-a-toolbar-item-read-the-workspace-001: nothing here is a toolbar item, but the
    same rule holds for the details panel's render: take the workspace from the tab's own handle.
  - #864's notes: the peek is `last_n_non_empty_lines(4)` of the call's inner terminal; the
    scripted agent's terminal calls run a real shell command.
  - L-claude-798-driving-menus-and-the-palette-in-a-scenario-001: the @ menu is driven by keys
    (`wtype`), confirmed with Return.
- **Discovery:**
  - `crates/agent_ui/src/conversation_view/thread_view.rs:5004-5170` (the composer),
    `:616` (`mode_selector` field), `:13524-13551` (the mode keys).
  - `crates/agent_ui/src/mode_selector.rs:16-90` (`ModeSelector`).
  - `crates/agent_ui/src/message_editor.rs:758` (`mention_set`) and `mention_set.rs:182-195`
    (`creases`, `mentions`).
  - `crates/marley_workbench/src/thread_tab.rs:697-960` (the panel and its sections).
- **Decisions:** D1–D5 in the spec.

- **Promoted 2026-10-25:** seams re-read after #914: `render_message_editor`'s composer and
  bottom row, `ModeSelector` (`mode`, `set_mode`, `mode_name`; `all_modes` private), the draft's
  creases through `MentionSet::mention_uri_for_crease`, `thread_tab.rs` `render_details`,
  `terminal_peek`, `peek_section`, `section_label`; #907's `thread_activity` lands first, so the
  state line reads it.

### Design
- Composer (Zed crate, `thread_view.rs`):
  - `render_marley_context_chips(&self, cx) -> Option<AnyElement>`: the mentions in crease order
    (the editor's creases sorted by their start), each `MentionUri::name()`; `None` when there are
    none or `MarleyThreadHost` is unset. Placed as the first child of the editor's column.
  - `render_marley_plan_switch(&self, cx) -> Option<AnyElement>`: needs the mode selector and a
    mode with id `plan` in `all_modes()`; two buttons in a 3 px padded group on
    `element_background`, the lit one on `element_selected`. `ThreadView` gains
    `marley_act_mode: Option<acp::SessionConfigValueId>`, set when the current mode changes to a
    non-plan mode. In the right group, the switch replaces `.children(self.mode_selector.clone())`
    when it draws.
- Zed crate, `mode_selector.rs`: `pub fn all_modes(&self) -> Vec<acp::SessionConfigSelectOption>`.
- Panel (Marley crate, `thread_tab.rs`):
  - `section_card(label, body) -> Div`: the card shape used by every section.
  - `state_line(thread, cx) -> (SharedString, Color)` from `status()` and the newest tool call:
    a `WaitingForConfirmation` call is "Needs you" in the needs-you colour; a running call
    "Working · <label as plain text>" in the warning colour; generating with no call "Working";
    else "Idle", muted.
  - `tiles(folder, mode)`: a two-column grid; the mode from the root thread view's mode
    selector (`mode()` and the option's name).
  - `open_peek_in_tab(terminal: Entity<terminal::Terminal>, workspace: WeakEntity<Workspace>,
    window, cx)`: a `TerminalView` over the terminal, added to the active pane, deferred.
- Ledger rows in `docs/marley/zed-touchpoints.md`: `thread_view.rs`'s row gains the chips' row
  and the Plan / Act switch (#915); a new row for `mode_selector.rs` (`all_modes`, a read the
  switch needs).
- File manifest: Zed crate: `thread_view.rs`, `mode_selector.rs`. Marley crate: `thread_tab.rs`.

### Visual check plan
- REQ-001, REQ-002: focus the composer, type "@", "main", Return, then " look"; `915-01-composer`;
  read the chip and the switch.
- REQ-002, REQ-004: click Plan; `915-02-plan`; read the switch and the Mode tile.
- REQ-003 to REQ-005: click Act, send the prompt, settle two seconds; `915-03-working`; read the
  cards, the state line's words and colour, the tiles and the link.
- REQ-005: click Open in a tab; `915-04-tab`; the output's lines in a terminal tab.
- All in Day: switch the theme live; `915-05-day`.

### Risks
- An agent that reports its modes as ACP config options gets Zed's `config_options_view`, not the
  mode selector. The scripted agent offers plain session modes; Claude Code's ACP adapter may use
  config options. If so, the switch must read the mode option from the config options too; check
  Claude Code's adapter in the Code phase and either cover both or record the gap.
- `set_mode` also writes the agent's default mode to settings (Zed's behaviour). A Plan click thus
  makes Plan the next thread's default too, as Zed's menu does; kept, and named in the CHANGELOG.
- A `TerminalView` made over the agent's terminal is a second view of one entity; check that
  closing the tab does not stop the agent's command (`TerminalView`'s drop does not kill its
  terminal while another holder keeps it).

## Phase 2 — Code
- **Built:** `ModeSelector::all_modes`; `MessageEditor::marley_mention_names` (the draft's creases
  through `crease_items_with_offsets`, sorted by offset); in `thread_view.rs`
  `render_marley_mention_chips` above the editor, `render_marley_plan_act` before the option and
  mode menus, `MarleyModeControl` and `marley_last_act_mode`; `ConfigOptionsView::value_ids_of`,
  `value_of`, `set_value_of`. In `thread_tab.rs`: `detail_card`, `detail_tile`, `state_line`,
  `TerminalPeek`, Open in a tab through `open_terminal_tab`.
- **Deviations:**
  - Zed's mode menu stays beside the switch rather than becoming a chevron on Act: it is the only
    way to reach a third mode (Bypass, Accept Edits), and it shows which Act mode is set.
  - The switch also covers an agent that reports its mode as a config option (F-claude-915),
    which the plan named as a risk; three accessors in `config_options.rs` and a ledger row.
  - The opened terminal tab takes the command as its custom title: an agent's terminal runs no
    shell, so the tab and its rail row had no name.
  - The chips are named `render_marley_mention_chips` and the switch `render_marley_plan_act`
    (the design said `context_chips` and `plan_switch`).
- **Review:** the details panel reads the root thread view from Marley's tab, outside any thread
  view render (F-claude-865 holds); Open in a tab updates the workspace from a plain click closure
  on the window's root, not from inside the thread tab's update (PR-claude-701 holds).
- **Gate:** `just gate-diff` 17/17 green, run again after each change.

## Phase 3 — Test
- **Scenario:** `script/e2e/915-thread-composer-and-details.sh`, sway, 7 shots, every one read.
  - `915-01-details`: the cards: THIS THREAD with "Working · Edit notes.md" in the working colour,
    the context line, Folder "config" (Marley's own agent runs in its folder) and Mode
    "Act · Default"; FILES CHANGED; ITS TERMINAL with the command, two lines and Open in a tab;
    CHECKS. REQ-003, REQ-004 met.
  - `915-02-chips`: the chip `notes.md` above "Look at [notes.md]"; Plan / Act with Act lit
    beside the "Default" menu. REQ-001, REQ-002 met.
  - `915-03-plan`: Plan lit, the menu and the status bar read "Plan", the Mode tile "Plan".
  - `915-05-day`: the same in Marley Day, cards on the light surface, chip legible.
  - `915-04-terminal`: a tab "sh -c echo Compiling; echo Finished" with the two lines as a block;
    its rail row named the same. REQ-005 met.
  - `915-06-options`: the second thread (config option, values Ask and Plan): the switch beside
    the "Ask" option menu, the Mode tile "Act · Ask"; `915-07-options-plan`: Plan lit, the menu
    and the tile "Plan". REQ-002 met for both shapes.
- **Fixes found in Test:** the first full run showed the opened tab with no title (fixed with the
  custom title); reading Claude Code's adapter showed its modes come as a config option
  (F-claude-915), fixed and given its own shots. Gate rerun green after each.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md`, the look plan's TICKET-825 row, the ledger rows for
  `thread_view.rs`, `message_editor.rs`, `mode_selector.rs` and `config_options.rs`.
- Knowledge appended: F-claude-915-plan-act-read-only-the-session-modes-001,
  L-claude-915-an-acp-agent-reports-modes-two-ways-001.
- Brain: decision recorded (Plan / Act over both mode shapes, Zed's menu kept).
