---
pipeline_id: a83d37ca-d850-4e43-acb4-8a4cbd83ec8e
ticket: docs/planning/tickets/closed/TICKET-454-rail-switcher.md
status: Phase 4 — Complete PASS
title: A switcher over recent terminals and threads
type: feature
slice: workbench shell W6e
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/457-rail-filter.spec.md, docs/planning/tickets/open/TICKET-459-rail-cycle-actions.md]
---

## Title
`ctrl-tab` in the rail or the Agent Panel opens a switcher over the window's terminals and
threads, the most recently shown first, with the row shown before the current one selected.
Each further `ctrl-tab` moves down the list, and releasing `ctrl` opens the selection, as Zed's
thread switcher does from its sidebar and its Agent Panel. The center panes keep Zed's tab
switcher.

## Scope
### In
- **The order** (`marley_rail`, pure):
  - `window_row`: the terminal or thread row that holds the window's focus. That is the
    focused Agent Panel's thread, else the displayed workspace's active terminal while it holds
    focus, whatever the fold and the filter.
  - `switcher_rows`: every terminal and thread row of the window. The rows the window showed
    come first, most recent first, then the rest in the rail's order, whatever the fold and the
    filter.
- **Recency** (`marley_workbench::rail`): the rail notes each change of the window's row with a
  counter, kept in memory and pruned to the rows that exist. The switcher's own focus is no
  row, so opening it notes nothing.
- **The switcher** (the rail's own `switcher` module, `rail_switcher.rs`, new): a view the rail
  puts in the `MultiWorkspace`'s sidebar overlay and focuses, as Zed's thread switcher is put.
  - It needs at least two entries. It opens on the second, or on the last for
    `select_last`.
  - `agents_sidebar::ToggleThreadSwitcher` moves the selection one down, or one up with
    `select_last`, wrapping at the ends. Its key context is Zed's `ThreadSwitcher`, so Zed's
    bindings drive it.
  - The release of the modifiers it opened with, Enter or a click confirms. The rail then opens
    the entry as a click on its row does. Escape and focus-out cancel, and focus goes back where
    it was.
- **The keys:** the rail's `toggle_thread_switcher` opens it. Zed's `AgentPanel` binding already
  routes `ctrl-tab` there. The Marley keymap binds `ctrl-tab` and `ctrl-shift-tab`, with
  `select_last`, to `agents_sidebar::ToggleThreadSwitcher` in `MarleyRail && !Picker`, as
  Zed binds them in `ThreadsSidebar`.

### Out (explicitly deferred)
- Next and Previous Project and Thread (`cycle_project`, `cycle_thread`): TICKET-459.
- `ctrl-tab` in the center panes stays Zed's tab switcher over the active pane's items. A
  window-wide switcher from the center is a product decision for Chad.
- The preview while cycling that Zed's switchers show, hover selection, arrow keys in the
  switcher, and keeping recency across restarts.

## Reference (§20)
- **Upstream Zed.** The Threads Sidebar's thread switcher (`crates/sidebar/src/thread_switcher.rs`,
  `crates/sidebar/src/sidebar.rs:6049-6194`), reached through `Sidebar::toggle_thread_switcher`
  from the `MultiWorkspace` root (`crates/workspace/src/multi_workspace.rs:2098-2104`).
  - It lists threads and Agent Panel terminals, the most recently accessed first
    (`sidebar.rs:5835-5857`).
  - It opens only with two entries or more (`:6067`), on index 1, or on the last for
    `select_last` (`thread_switcher.rs:206-213`).
  - Repeated `ctrl-tab` cycles with wrap (`:252-270`, `:312-323`).
  - It confirms when the modifiers it opened with are released (`:325-343`), on Enter and on a
    click, and cancels on Escape and focus-out (`:220-224`, `:279-310`).
  - It shows through `set_sidebar_overlay` (`sidebar.rs:6185`, `multi_workspace.rs:405-408`,
    `:2182-2195`) and is bound to `ctrl-tab` in `AgentPanel`, `ThreadsSidebar` and
    `ThreadSwitcher` (`assets/keymaps/default-linux.json:225-226`, `:756-757`, `:771-772`).
    Zed's docs: "This works from both the Agent Panel and the Threads Sidebar."
    (`docs/src/ai/parallel-agents.md:30`)
  - Zed's tab switcher keeps `ctrl-tab` in the `Workspace` context over the active pane's
    items (`crates/tab_switcher/src/tab_switcher.rs:121-138`, `:528-558`, `:202-223`).

  Marley keeps the hook, the overlay, the action, Zed's keys and the open, cycle, confirm and
  cancel rules. It differs in its entries (the rail's center terminals and its threads), in
  its recency (the window's row, not Zed's access maps), and it leaves out the preview.
- **Warp:** N/A. `docs/warp_architecture/` and `docs/planning/design-notes/` describe no
  keyboard switcher. `session-tabs-vs-sidebar.md:32` describes clicking a row to switch, which
  the rail already does.

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (the
  `MultiWorkspace` and its sidebar); nothing on a Warp keyboard switcher.
- **Published material:** Zed's parallel-agents docs (`docs/src/ai/parallel-agents.md`).
- **Code we already ship.**
  - `Sidebar::toggle_thread_switcher`, which `SidebarHandle` runs in `window.defer`
    (`multi_workspace.rs:225-250`). The rail may therefore update the `MultiWorkspace` from it.
  - `MultiWorkspace::set_sidebar_overlay` draws the overlay above everything, occluding the
    mouse. It neither focuses nor dismisses it, so the owner does both.
  - `zed_actions::agents_sidebar::ToggleThreadSwitcher { select_last }`.
  - gpui's `on_modifiers_changed` and `ModifiersChangedEvent`, `Modifiers::modified` and
    `is_subset_of`. Modifier events reach only the focused path (`window.rs:6073-6091`).
    `VisualTestContext::simulate_modifiers_change` drives them in tests
    (`test_context.rs:950-956`).
  - `ui::ThreadItem` and `ui::ListItem` for the rows. The rail's own handlers open a row
    (`activate_terminal`, `open_thread`).
  - Zed's `ThreadSwitcher` view cannot be reused: its entries are Agent Panel threads and
    terminals, not center terminals. A pane's `activation_history` counts per workspace, so it
    cannot order rows across the window's projects.
- **The shadow sweep for `ctrl-tab` and `ctrl-shift-tab`** covered every keymap in
  `assets/keymaps/`.
  - On the rail's path (`Workspace > MarleyRail menu`), only `Workspace`'s
    `tab_switcher::Toggle` pair matches, at a shallower depth, so a `MarleyRail` binding wins
    there. TextMate's `Pane` binding is not on the path, and neither `menu` nor `Editor` binds
    the keys, so the filter field inside the rail gets the rail's binding too.
  - The PTY never receives `ctrl-tab` (`crates/terminal/src/mappings/keys.rs`), and no
    Terminal context binds it.

## UI proof
UI-AFFECTING.
- **Driven tests.**
  - `ToggleThreadSwitcher` from the Agent Panel and from the rail opens the switcher on the
    second entry, or on the last with `select_last`; with fewer than two entries nothing opens.
  - Repeated toggles cycle, with wrap. Releasing `ctrl` after `simulate_modifiers_change`
    opens the selection, and so do Enter and a click. Escape and focus-out change nothing and
    close it.
  - The order follows the window's rows, and opening the switcher itself notes nothing.
  - With Zed's default keymap and the Marley keymap bound, `ctrl-tab` from the rail opens it.
    From a center terminal it does not: that key stays Zed's tab switcher.
  - Unit tests cover the pure order.
- **Live drive:** open two projects' terminals and a thread, then press and hold `ctrl-tab`
  and release; screenshot the open switcher and the result. It needs keys, so it runs only
  while Chad is away from the desk; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — Zed's split of the key. The Agent Panel (Zed's binding) and the rail (the Marley
  keymap) open the rail's switcher, and the center panes keep Zed's tab switcher. The draft's
  "`ctrl-tab` in the Marley layout" was narrowed here; the center is Chad's call.
- D2 — The switcher is the rail's `toggle_thread_switcher`, drawn as the sidebar overlay and
  focused by the rail, like Zed's thread switcher. Its key context is Zed's `ThreadSwitcher`.
- D3 — Recency is the rail's own counter over changes of the row that holds the window's
  focus, in memory and pruned. The order is most recent first, then the rest in the rail's order, whatever the
  fold and the filter.
- D4 — Zed's rules are kept: at least two entries; the second (or the last) selected;
  confirm on the release of the modifiers it opened with, on Enter or on a click; Escape and
  focus-out cancel. The preview is left out, so cancelling has nothing to restore but focus.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `agents_sidebar::ToggleThreadSwitcher` reaches the rail, from the Agent Panel or from the rail, and the window has two terminals and threads or more, the rail shall open a switcher over them with the second selected, or the last for `select_last` | driven tests |
| REQ-002 | WHILE the switcher is open, `ToggleThreadSwitcher` shall move its selection one down, or one up for `select_last`, wrapping at the ends | driven test |
| REQ-003 | WHEN the modifiers the switcher opened with are released, or Enter is pressed, or an entry is clicked, the switcher shall close and the rail shall open that entry as a click on its row does | driven tests with `simulate_modifiers_change` |
| REQ-004 | WHEN Escape is pressed in the switcher or it loses focus, it shall close, open nothing, and leave the order as it was | driven tests |
| REQ-005 | The switcher shall list the rows the window showed most recently first and the others after them in the rail's order, whatever the fold and the filter | unit tests + driven test |
| REQ-006 | WHEN `ctrl-tab` is pressed with the rail focused, the switcher shall open; WHEN it is pressed in a center terminal, the rail's switcher shall not open | driven tests with Zed's default keymap and the Marley keymap |
| REQ-007 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the order in `marley_rail`; the recency and the hook in `rail.rs`; the switcher
  view; the keymap block; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run the tests, with the negative checks of
  `PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001`; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
