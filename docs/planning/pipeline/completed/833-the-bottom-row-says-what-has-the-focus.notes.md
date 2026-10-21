# The bottom row says what has the focus — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-833-the-bottom-row-says-what-has-the-focus.md
- **Pipeline spec:** 833-the-bottom-row-says-what-has-the-focus.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #832; Chad's overnight goal.
- **Classification / tier:** feature, L, scoped to M: the panel group (#862) and the cards with the
  debugging row (#863) are split off and queued before #794.
- **Recall (§18.3):**
  - PR-claude-831: a view made during the workspace's construction (a status item is) must not
    read the workspace in `new`.
  - F-claude-438-a: read the rail's kept snapshot, not render-only state.
  - The brain (consultation 2d66a901): nothing on this seam.
- **Discovery** (an Explore map, this session): `status_buttons.rs:30-56` (registration,
  `insert_item_after` Debug), `StatusItemView` (`workspace/src/status_bar.rs:42-59`), Zed's items
  (`zed.rs:635-653`), `ActiveFileName` (`workspace/src/active_file_name.rs:52-77`),
  `AnchoredBlock`/`AnchoredBlocks` (`marley_terminal/src/anchored.rs`), `Terminal::marley_anchored`,
  rail `command_snapshot` (`rail.rs:~9375`) → `CommandSnapshot` (`marley_rail.rs:214`),
  `bookmarks::is_bookmarked` (`bookmarks.rs:84`), `working_directory_label` (`marley_rail.rs:1992`),
  inbox (`InboxEntry` `marley_rail.rs:590`, `InboxAnswer` :517, rail `open_inbox_entry` :2276,
  `answer_inbox` :2624), `MarleyHome::rail()` (`home_page.rs:296-313`), `thread_tab::ThreadTab`,
  `agents::thread_agent_name` (`agents.rs:159`).
- **Decisions:** D1 to D4 in the spec.

### Design
- **`focus_row.rs`** (new, Marley): `pub struct FocusRow { workspace: WeakEntity<Workspace>,
  active: Option<Box<dyn ItemHandle>>, item_subscription: Option<Subscription>, rail:
  Option<(WeakEntity<Rail>, Subscription)> }`.
  - `set_active_pane_item`: keep the item; observe its entity (`cx.observe` on the item's
    `to_any()` downcast for `TerminalView`'s terminal, `Editor`, or `ThreadTab`) so state changes
    redraw; `cx.notify()`.
  - `render`: find the rail (as `MarleyHome::rail()`, observed once, kept), then build the
    left part from the active item's kind, then the needs-you part from `rail.inbox()`.
  - File: `item.project_path(cx)` → path; extension in a small rounded chip.
  - Terminal: `TerminalView` → `terminal.read(cx)`: title (`tab_content_text`), `working_directory`
    with `working_directory_label` against the project's first folder, `marley_anchored().live()`:
    last finished block (`command`, `exit_code`) and whether the last is running (or reuse
    `rail::command_snapshot`, made `pub(crate)` with a doc), block count, bookmarked count from
    `MarleyBlockMarks`; `tmux <n>` from `TmuxHosts` when any session is listed.
  - Thread tab: `ThreadTab` → its conversation's thread → agent name and status (through the
    helper the rail's `live_statuses` uses, or `thread_tab::active_conversation`).
  - Needs-you: `rail.read(cx).inbox()` first entry: `agent · project: ask` truncated, `Allow`
    when its answers allow, `Open`; `+n more waiting`. Buttons call `rail.update(cx, |rail, cx|
    rail.answer_inbox(&key, InboxAnswer::Allow, window, cx))` / `open_inbox_entry`.
- **`rail.rs`**: `pub(crate) fn inbox(&self) -> &[InboxEntry]`; `open_inbox_entry` and
  `answer_inbox` become `pub(crate)`; `command_snapshot` becomes `pub(crate)` if reused.
- **`status_buttons.rs`**: in the `observe_new`, make `FocusRow` and insert it after Debug.
- **File manifest:** all Marley: `focus_row.rs` (new), `status_buttons.rs`, `rail.rs`,
  `marley_workbench.rs` (`mod focus_row`), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-002/004 | `repo`; terminal runs `true`, `false`; stand-in Claude in a second terminal waits; focus back on the first terminal | `833-01-terminal` |
| REQ-002 | `sleep 30` typed in the terminal | `833-02-running` |
| REQ-001 | Ctrl+P `main.rs` | `833-03-file` |
| REQ-005 | click Allow in the row | `833-04-allowed` |
| REQ-002 | theme Marley Day; terminal focused | `833-05-day` |
| REQ-003 | a Zed agent thread tab needs a model; read in the diff unless the profile runs one |

### Risks
- Observing the active item's entity: switching tabs must drop the previous subscription.
- The inbox's Allow runs the same path as the rail's (#508); the row must not offer Allow for an
  entry whose answers do not include it.

## Phase 2 — Code
- **Built:** `focus_row.rs` (new): `FocusRow` (`StatusItemView`), `Focus { None, File, Terminal,
  Thread }` set in `set_active_pane_item`, which also observes the terminal or the conversation;
  `render_file` (extension chip + project path), `render_terminal` (`>_`, the tab title capped at
  200 px, the folder label, `rail::command_snapshot` for `<cmd> · exit <n>` or `running: <cmd>`,
  live block and bookmark counts, `tmux <n>`), `render_thread` (agent name through
  `thread_agent_name`, the status as the rail derives it, the title), `render_waiting` (the
  rail's first inbox entry, capped at 320 px, Allow when its answers allow, Open, `+n more
  waiting`). The rail is found from the window's `MultiWorkspace` and observed, as Home does; the
  tmux hosts are observed. `status_buttons.rs` inserts it after Debug. `rail.rs`: `inbox()`, and
  `open_inbox_entry`, `answer_inbox`, `command_snapshot` made `pub(crate)`. `thread_tab.rs`:
  `ThreadTab::conversation`.
- **Deviations:** REQ-005 is shown with Open: a Claude Code terminal's request (#542's stand-in)
  takes no answer in place, so neither the rail nor the row offers Allow for it; Allow uses the
  same `answer_inbox` path. The terminal's last block is the rail's `command_snapshot` (the last
  live block, running or finished), not a separate "last finished" search.
- **Probe runs found a real bug:** with Zed's language-server message in the row, the waiting
  chip's Open was drawn over by the items after it. The root had `min_w_0`, so it shrank below its
  content. The root now keeps its width (`flex_none`), with the title and the request capped and
  truncating, so Zed's message is what shortens (F block at Complete). The probes also moved the
  scenario off rail-row clicks, whose points change between runs.
- **Gate:** clippy findings fixed at source (doc paragraph, two redundant closures,
  `map_or_else`, underscore field, a missing doc); `script/gates.sh --diff` 17/17.

## Phase 3 — Test
- **Scenario:** `script/e2e/833-the-bottom-row-says-what-has-the-focus.sh` (sway, the gated tree),
  exit 0, six shots, read as row crops and the whole window.
- `833-01-terminal`: `>_ repo — bash`, `false · exit 1` in red, `no command running`, `2 blocks`
  (REQ-002 met).
- `833-02-running`: `>_ repo — sleep 30`, `running: sleep 30` in the accent, `3 blocks` (REQ-002
  met).
- `833-03-waiting`: b's stand-in focused: `>_ b — python3 …` (capped), `0 blocks`, and the pink
  chip "Claude Code in b: Permission for Write: README.md" with Open (REQ-004 met; no Allow, as
  the rail's own card offers none for this request).
- `833-04-file`: `rs src/lib.rs`, then the chip, then Zed's items (REQ-001 met).
- `833-05-opened`: after Open in the row, b's terminal with the stand-in's "b: waiting" is the
  active tab, and the row says so (REQ-005 met). The chip and its Open stand clear of Zed's
  search, LSP and diagnostics items, whose message truncates.
- `833-06-day`: Marley Day, the same row in Day's colors (REQ-002, REQ-004 met).
- REQ-003 (a thread tab): needs a Zed agent thread with a model; read in the diff. It reads the
  agent name, status and title the way the rail does.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #833); the guide's panels section (What has the focus); the
  architecture note (What has the focus, at the left); look-plan W1.4 done with #862 and #863.
  No Zed path touched.
- **Knowledge:** F-claude-833-a-status-item-shrank-below-its-content-and-was-drawn-over-001,
  PR-claude-833-a-status-item-caps-its-parts-instead-of-shrinking-001.
- **Brain:** decision on consultation 2d66a901 (follow-up 2026-11-04).
- **Ticket** closed; #862, #863 filed; pipeline archived; committed on `marley/workbench-shell`.
