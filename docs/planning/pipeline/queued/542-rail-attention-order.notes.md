# The rail puts what needs Chad first — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-542-rail-attention-order.md
- **Pipeline spec:** 542-rail-attention-order.spec.md

## Phase 1 — Plan
- **Request:** from the Orca survey Chad asked for on 2026-09-25 (report 01 §3 item 5, report 05
  §3 item 4): projects and rows ordered needs you first, then done and unseen, working, not
  reporting, idle; a collapsed project's summary ("2 working, 1 waiting"); the order held while the
  pointer is over the rail.
- **Classification / tier:** feature, prong 2 (attention). Rust in `marley_rail` and
  `marley_workbench` (Marley crates) and the Marley settings file in a Zed crate. Queue, after #519.
- **Recall (§18.3):**
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: a scenario that clicks by
    coordinates breaks when things move. Rows that move are this ticket's point, so the scenario
    reads its targets from its own shots and reruns #500's rail scenario, which clicks rail rows.
  - L-claude-487-a-headless-seat-has-no-devices-until-a-client-adds-them-001: under sway the seat's
    pointer is a helper that stays connected; `pointer_to` moves it for the hover.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001: for the Add Project popover.
  - L-claude-479-drive-a-file-chooser-through-the-test-platforms-path-prompt-001: the default
    settings use the system path prompt; the scenario sets `use_system_path_prompts` false to type
    paths into Zed's own prompt.
  - PR-claude-pump-state-change-must-set-dirty-to-repaint-001 (gpui era): every state change that
    moves a row must redraw, including the hold's end.
  - Brain: not consulted at drafting (read-only overnight drafting); the promotion runs
    `brain_ask`.
- **Discovery:**
  - #519's spec (drafted the same night, `519-claude-code-events-in-the-rail.spec.md`): a seat per
    terminal in `marley_workbench::agent_events`, keyed by the terminal's id; the row reads
    `working`, `waiting`, `idle` or `failed`, and `no update in N m` once a working seat has gone 30
    minutes without an event while Claude Code is in the foreground (its D7); a row with no events
    keeps the quiet timer; its Out leaves "attention order, summaries and unread marks" to this
    ticket.
  - `crates/marley_rail/src/marley_rail.rs`: `ProjectSnapshot` (30-42), `TerminalSnapshot` (46-59)
    and `TerminalAgent` (63-68, a kind and `AgentStatus`), `ThreadSnapshot` (72-83), `ThreadStatus`
    (87-97), `thread_status` (118-132), `thread_attention` (140-152), `Focus` (156-167),
    `RailSnapshot` (171-178), `Selection` (182-191, projects by window index), `ProjectRow`
    (195-209), `walk` (347-369, the one order the rows, the selection and the keyboard read),
    `row_shows` (373-383), `rail_rows` (500-540), `switcher_rows` (574-607),
    `hidden_rows_need_the_user` (610-620), `has_attention` (636).
  - `crates/marley_agent/src/marley_agent.rs`: `AgentStatus` (96-113, working or waiting),
    `WAITING_AFTER` (117, two seconds), `agent_status` (121-128), `status_line` (131).
  - `crates/marley_workbench/src/rail.rs`: `refresh` (299-328, redraws only when the snapshot
    changed), `note_ended_runs` (348-372), `note_output` (375-387, the quiet timer),
    `move_project` (869-885), `toggle_expanded` (887-903), `render_project_row` (1016-1111: the
    disclosure, the name, the attention dot, the menu, Move Project Up and Down at 1084-1110),
    `render_thread_row` (1236-1266), `render_terminal_row` (1268-1370), `terminal_snapshot`
    (1666-1722), `build_snapshot` (1734-1846), `thread_status_mark` (1959-1987), `render`
    (2140-2222: `last` is the group's window index, so Move Project Down keeps its meaning).
  - `crates/marley_fleet/src/attention.rs`: `AttentionReason` (Error, Question, Stale), `rank`, a
    stable sort, the clock injected; a model for ranking, not a seam the rail can call (it ranks
    fleet seats, which nothing feeds yet).
  - `crates/sidebar/src/sidebar.rs`: threads sorted by time (1800-1804); the collapsed group's
    spinner and waiting warning (2409-2437); `on_hover` with `cx.listener` (6304, 6608).
  - `crates/gpui/src/elements/div.rs:1655`: `on_hover(listener(&bool, …))` on a stateful element;
    the rail's root is `v_flex().id("marley-rail")`, which is stateful.
  - `crates/recent_projects/src/sidebar_recent_projects.rs:392`: Add Project's Open Local Folders
    dispatches `workspace::Open { create_new_window: Some(false) }`, which opens a folder as another
    project in the same window.
  - Orca (MIT): `src/renderer/src/components/sidebar/smart-attention.ts` (416 lines; the classes and
    "the most demanding pane wins"), `worktree-card-agent-summary.ts` (130 lines; the state order
    and `not reporting`), `src/shared/agent-status-freshness.ts` (30 minutes).
- **Decisions:** D1 to D7 in the spec.

### Design
- **The pure part (`marley_rail`).**
  - The terminal row's state comes from #519's seat: working, waiting (with its `Question`), idle,
    failed, and its `no update in N m` decay of a silent working seat, beside today's quiet-timer
    `status` for a terminal whose agent sends no events. A failed run counts as needing Chad until
    the terminal is focused: through #538's mark once #538 has landed, and until then tracked the
    way `note_ended_runs` tracks threads.
  - `Attention { NeedsYou, DoneUnseen, Working, NotReporting, Idle }` with `const fn` class
    functions for a thread row and a terminal row, and the summary's words.
  - `RailSnapshot` gains `order: RailOrder { Attention, Window }` and
    `held: Option<HeldOrder>` (the project indices and each project's row selections, in the order
    shown when the hold began). `walk` sorts a stable copy of the projects by class and each
    project's rows by class, or follows `held` with new rows and projects at the ends.
  - `ProjectRow` gains `summary: Option<String>`, set only for a collapsed project with counted
    agents.
- **The gpui side (`marley_workbench::rail`).** `build_snapshot` fills the new fields from #519's
  per-terminal state; the rail's root gets `on_hover`: entering stores the rail's current order as
  `held`, leaving clears it and calls `refresh`. `render_project_row` draws the summary after the
  name in a muted small label. The `no update` decay is #519's, and so is the redraw when a seat
  crosses the 30 minutes.
- **Settings.** `rail_order` on `MarleySettingsContent`, read by `MarleySettings` and copied into
  the snapshot; a change refreshes the rail through the settings observer the layout switch already
  uses (`marley_workbench.rs:259`).
- **File manifest.** `crates/marley_rail/src/marley_rail.rs` (Marley crate);
  `crates/marley_workbench/src/rail.rs` and `marley_workbench.rs` (Marley crate);
  `crates/settings_content/src/marley.rs` (Zed crate path, Marley's file; its touchpoint row
  updated); `script/e2e/542-rail-attention-order.sh` and its fake (Test). After #515, a Rail section
  on the Marley page (`crates/settings_ui/src/marley_page.rs`, #515's file).
- **Ledger rows.** The settings file's touchpoint row. At Complete, a lesson if the hover's end
  fires in some case the scenario finds surprising (a menu opened from the rail, a drag).

### E2E plan
Shared fixtures: three scratch repositories, `a`, `b` and `c`, opened into one window in that
order through the rail's Add Project with Zed's typed path prompt (`use_system_path_prompts`
false); in each, a terminal running a fake `claude` started as #481's scenario starts one, which
prints the #519 frame its trigger file names; `b` also has a plain shell as its first terminal.
The pointer's targets are read from the first shot, not fixed (L-claude-498).

| REQ | Scenario part | Shot |
|---|---|---|
| REQ-001, REQ-003 | `a` working (a `UserPromptSubmit` then `PreToolUse`), `b` waiting (`PermissionRequest`), `c` finished (`Stop` while another terminal holds focus); the pointer parked off the rail | `542-01-order`: `b`, `c`, `a` from the top; under `b` the waiting agent above the shell |
| REQ-004 | Collapse `b` (its disclosure, a click) | `542-02-collapsed`: `b`'s header reads `1 waiting` |
| REQ-005 | Expand `b`; move the pointer onto the rail; `a` turns waiting | `542-03-held`: the order as in `542-01-order` |
| REQ-006, REQ-002, REQ-009 | Select `a`'s terminal row with the keyboard, then move the pointer off the rail | `542-04-released`: `a` then `b` (both needs you, window order), then `c`; `a`'s row still selected |
| REQ-007 | Only with `E2E_LONG=1`, as #519's `519-08-no-update`: `a` back to working, then silent for 31 minutes | `542-05-not-reporting`: `a`'s row reads `no update in 31m`, below the working and done rows |
| REQ-008 | `rail_order: window` | `542-06-window-order`: `a`, `b`, `c`, rows in tab order |

The run also reruns `script/e2e/500-browser-from-the-rail.sh`, which clicks the project's + in the
rail at fixed coordinates (`PLUS_X=236`, `PLUS_Y=96`, measured from its first run), and reads its
log and shots, not only its exit code (L-claude-498).

### Risks
- **#519's seat.** The classes read the seat `agent_events` keeps; `build_snapshot` copies its state
  into the terminal's snapshot, so a change in how #519 stores it touches the workbench and not
  `marley_rail`.
- **The 30-minute redraw.** The not-reporting class moves a row only when the rail redraws after the
  30 minutes pass; #519 owns that redraw for its `no update` text, and the order follows it. If the
  text lags, so does the order.
- **The hover's end.** A context menu or a popover opened from the rail takes the pointer off the
  rail's element; the hold then ends while the menu is open, and a row may move behind the menu.
  The menu acts on its row by identity (Move Project and Close use the key and the view), so the
  action stays right; Test looks at it once.
- **Rows moving under the keyboard.** Out of scope by choice; if it bothers Chad, holding the order
  while the rail has keyboard focus is one more condition on `held`.
- **Two quiet signals.** Agents without the plugin keep the two-second quiet timer, which says
  nothing about waiting; D4 sorts them as idle so a pause never jumps the queue.
