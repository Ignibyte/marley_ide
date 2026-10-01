# A task's run as a block — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-621-a-tasks-run-as-a-block.md
- **Pipeline spec:** 621-a-tasks-run-as-a-block.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, first batch (#619 to #623): T2 and the rest of T4.
- **Recall (§18.3):**
  - `terminal.rs` loads shell hooks only for non-task, local, PTY terminals: a task never makes a block today.
  - AD-claude-441: the Marley layout routes tasks to the center through `TerminalPanel::spawn_task`, keeping Zed's rerun rules.
  - `register_task_finished` is where the exit code arrives and the summary line is appended.
- **Discovery:** one Explore sweep for T2 and T4 (2026-09-30) over the block model, the
  terminal's links, the failure shapes, the task spawn path and the diagnostics store; the spec's
  Prior art cites what applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: the PTY terminal's `blocks` are built in the `Terminal` literal
  (`terminal.rs:1554`), where `task: Option<TaskState>` is in scope (`TaskState.spawned_task:
  SpawnInTerminal` with `command_label` and `cwd`); `register_task_finished` (3553) sets the
  task's status from the exit code before it appends the summary lines; `AnchoredBlocks`'s
  `finish_running(line, ExitCode)` closes the running block; `command_verified` gates the typed
  Rerun (#474, the element) and history uses, so a task block keeps it false.
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation
  b8f4722205b24e03a9ff875d73152d71): nothing on this seam.

### Design
- **`crates/marley_terminal/src/anchored.rs`:** `open_task(command, pwd, line)` (finishes any
  running block, pushes a local host and a running block with no prompt line, its folder in
  `prompt.pwd`, unverified) and `finish_task(line, exit)`.
- **`crates/terminal/src/terminal.rs`** (Zed crate, `// Marley:` hunks): in the literal's `blocks`,
  a local task terminal's blocks open the task's block at absolute line 0, stamped; in
  `register_task_finished`, once the status is set and before the summary is appended, the block
  finishes at the cursor's absolute line (the next line when the cursor is past column 0), with the
  exit code, stamped, and the terminal notifies.
- **Ledger:** the `terminal.rs` row's clause.
- **Manifest:** `anchored.rs` (Marley), `terminal.rs` (Zed), the touchpoints row, the scenario.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-002 | `repo/.zed/tasks.json` with `fails` (`echo building; exit 3`) and `passes` (`echo ok`); each from `task: spawn` | `failed.png`, `passed.png` |
| REQ-003 | a right-click on the failed task's block | `menu.png` |

### Risks
- A task with `show_command` prints its command after the run in Zed; the block ends before the
  summary lines, so they sit outside it.

## Phase 2 — Code (2026-09-30)
- **Built:** `AnchoredBlocks::open_task` and `finish_task`; in `terminal.rs`, `marley_task_block`
  (a local task's `command_label` and `cwd`) taken before the literal moves `task`, the block
  opened at line 0 and stamped; in `register_task_finished`, after the status, the block finished
  at the cursor's absolute line (one more past column 0), stamped, the terminal notified. The
  ledger clause.
- **Deviations:** none.
- **Review:** the finish reads `self.term` and `self.blocks` while `task` borrows `self.task`,
  disjoint fields; a remote task opens no block, and its finish then finds no running block and
  changes nothing.
- **Clippy:** clean the first time.
- **Gate:** `just gate-diff` GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/621-a-tasks-run-as-a-block.sh` (sway), two runs.
- **Run 1:** `failed.png` and `passed.png` passed; `menu.png` did not: the right-click at the
  view's top landed on empty rows, since the content sits against the bottom edge (#476), and the
  menu had no Block section. The click moved onto the block's row.
- **Run 2, the shots read:**
  - `failed.png` (REQ-001, REQ-002): the `fails` tab with its error icon; one block holding
    `building` and `exit` (bash's own word as it exits) with the red bar, the wash and the `exit 3`
    pill; Zed's two summary lines below it, outside the block; the rail's row `fails` with
    `/usr/bin/bash -i -c… · exit 3 · 0 s`.
  - `menu.png` (REQ-003): the right-click on the block: Zed's terminal menu and the Block section
    (Send to Agent, Bookmark, Find in Block, the copies; Reinput and Reinput with sudo disabled,
    since a task's command is no shell's report), the block outlined as selected with its hover
    buttons.
  - `passed.png` (REQ-001): the `passes` tab with its check; one block `ok` with the green bar and
    the check pill; the rail's row `done · 0 s`.
- **Seen:** the block's command is Zed's `command_label`, the shell-wrapped form its own summary
  line prints (`/usr/bin/bash -i -c '…'`), so the rail's row shows it so.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Blocks); `terminal_blocks.md`; the plan's T4 row; the
  touchpoints clause.
- **Knowledge:** L-claude-621-a-short-terminals-rows-sit-at-the-bottom-001.
- **Brain:** consultation b8f4722205b24e03a9ff875d73152d71 closed with `brain decide`.

