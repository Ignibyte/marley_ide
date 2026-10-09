# One rail row for a thread in a center tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-702-one-rail-row-for-a-thread-in-a-center-tab.md
- **Pipeline spec:** 702-one-rail-row-for-a-thread-in-a-center-tab.spec.md

## Phase 1 — Plan
- **Request:** #697's Test phase saw the thread listed twice; Chad was asked, and told the session
  to proceed on the default (keep the thread row). Run by a fork of the session.
- **Classification:** bug, the rail, Marley crate only.
- **Recall (§18.3):** #674 (every center tab has a row; terminals and Browser tabs are skipped
  there for rows of their own); #697 (`ThreadTab`, `activate_for`, the rail's `show_thread`).
  Nothing in the ledgers on this seam.

### Design
- **`thread_tab.rs`:** `ThreadTab::thread_key(&self, cx) -> String`, the conversation's
  `parent_id().to_key_string()`.
- **`rail.rs`:** `member_tabs` skips `ThreadTab`; `active_rows` returns the active item as a tab
  only when it is not a `ThreadTab`; `note_focus` sets `Focus::thread` to the active `ThreadTab`'s
  key, else the panel's thread as today.
- **File manifest:** `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/
  thread_tab.rs` (Marley crate); the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | A Marley thread on the scripted agent, "hello"; `marley: open thread in center` | 702-01-one-row |
| 003 | Clicks the terminal's row, then the thread row | 702-02-from-row |
| 004 | — | The gate |

### Risks
- None beyond the marked row: the panel's own thread still marks its row when the panel holds
  focus.

## Phase 2 — Code
- **Built (by the session's fork, finished by the session after the fork hit its turn limit):**
  - `thread_tab.rs`: `ThreadTab::thread_key`, the conversation's `parent_id().to_key_string()`,
    the key the rail's thread rows use (`live_threads`, rail.rs, and the listed rows).
  - `rail.rs`: `member_tabs` skips a `ThreadTab`, as it skips terminals and Browser tabs;
    `active_rows` takes no tab row for an active `ThreadTab`; `note_focus` marks the thread row
    of the active `ThreadTab` (`active_thread_tab`), else the panel's focused thread as before.
- **Deviations:** none.
- **Review of the diff:** reads only, from the rail's snapshot pass; the key format matches the
  thread rows' (`to_key_string` on both sides). Clicking the thread row still goes through
  `show_thread`, which brings the tab forward (`thread_tab::activate_for`).
- **Gate (`702-gate-1.log`):** GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/702-one-rail-row-for-a-thread-in-a-center-tab.sh`, #697's setup and
  scripted agent under `compositor sway`. Since #700 the rail lists Home on top, so the project's
  + is at (236, 136), the terminal row at y 176 and the thread row at y 267 (under the panel's
  draft row), read from the probe runs.
- **Red: no key reached Marley after start (`shots-702a`, `-b`).** The trust prompt stayed up
  through Enter, and the palette never opened, so the scripted agent never ran.
  - A diagnostic scenario in the scratchpad (shots at 2, 8, 10 and 16 s, Enter twice) showed the
    prompt up from 2 s and both Enters lost; sway listed one Marley window, focused, so the keys
    reached the window.
  - A temporary focus log in `Rail::refresh` showed the sequence: the project's pane, then the
    prompt (focused), then, 3 s in when #700's `ensure_groups` made Home with `with_group`, the new
    workspace's pane (`Workspace::new` focuses its center pane, even in `OpenMode::Add`), then
    another handle of the new workspace's setup. The prompt never got the focus back, and the
    window draws only the project, so keys went nowhere. #700's and #701's scenarios had met this
    and clicked their way around it (L-claude-700-a-restored-folderless-workspace-starts-unfocused-001).
  - Tried and dropped: a guard in Zed's focus-lost listener (`owns_window_chrome`) and a focus
    restore in `new_local`'s `OpenMode::Add` arm. Neither brought the keys back, since the new
    workspace's setup moves focus after `new_local`'s window update, so both Zed hunks and their
    ledger row were reverted. No Zed path is touched.
  - **Fix (`rail.rs`):** `ensure_groups` takes the window's focused handle (weak) before it makes a
    group, and the group's `then` runs `keep_focus`, which focuses that handle again when it still
    exists and lost the focus. `then` runs once `new_local`'s whole task is done.
  - The diagnostic then showed Enter trusting the project (the prompt gone, no Restricted Mode in
    the title). The temporary log was removed.
- **Gate after the fix:** `702-gate-2.log`, GATE GREEN [diff].
- **Final run (`shots-702d`): every shot shows its criterion.**
  - **702-00-before (REQ-005):** the scenario's Enter trusted the project and the palette and the
    + menu took their keys: the title reads "repo" with no Restricted Mode, and the thread "hello"
    is in the panel with "Noted: hello". The rail lists Home, then repo with its terminal and the
    thread row, marked.
  - **702-01-one-row (REQ-001, REQ-002):** after `marley: open thread in center`, the center has
    "repo — bash" and "hello" (the thread, "Noted: hello"), and the panel is on "New Marley
    Thread". The rail lists repo — bash, the panel's draft (New Agent Thread · Marley · idle) and
    one row for the thread (hello · Marley · idle), marked. There is no tab row for it.
  - **702-02a-terminal:** the terminal's row clicked: the terminal in front, its row marked.
  - **702-02-from-row (REQ-003):** the thread row clicked: the "hello" tab is in front again with
    the thread, and its row is marked.
  - The run reports Chad's Hyprland untouched: one Marley window before and after.
- **Not in scope, still open:** a restored folderless workspace shown at start (Home or Rusty with
  no tab) still starts with nothing focused (L-700's own case, a restored window, not a group made
  later). This fix doesn't reach it. Noted for a ticket.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Fixed: keys right after start; one rail row for a thread in a
  center tab); `docs/marley_architecture/marley_workbench.md` ("One row for a thread in a center
  tab, and the keys at start"). No Zed path is touched, so no touchpoint row.
- **Knowledge appended:** F-claude-702-a-group-made-at-start-took-the-windows-focus-001 and
  PR-claude-702-making-a-workspace-in-the-background-keeps-the-focus-001.
- **Brain:** this repository's sessions have no `rusty` MCP server; nothing recorded there.
- **Ticket:** closed; BACKLOG had no row left.
- **Gate:** `702-gate-2.log`, GATE GREEN [diff], on the tree committed.
