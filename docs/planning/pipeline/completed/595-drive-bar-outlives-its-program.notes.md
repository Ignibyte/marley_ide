# End the drive bar with the program an agent typed into — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-595-drive-bar-outlives-its-program.md
- **Pipeline spec:** 595-drive-bar-outlives-its-program.spec.md

## Phase 1 — Plan
- **Request:** #594's run of #525's scenario, shot 525-11-shell-refused: after Ctrl-D leaves
  Python, the bar "Stand-in agent typed into python3: "print('never asked')" <enter>" with Take
  Over stays under the shell's `$`.
- **Classification / tier:** bug, one Marley file, a render check.
- **Recall (§18.3):** #525's notes describe `drive()` resetting the drive on a new program, and
  `footer` reading `Drives` directly (it gets `&App`). As filed, #595 claimed Ctrl-I would take
  over at the shell's prompt; `toggle_control` calls `drive()` first, which drops the last write
  once the program changed, so it does not. The claim is corrected in the ticket, and at
  Complete in #594's and #525's notes. The brain (consultation
  320ca8b1288d4a6ebfa65d167d693e1e) had nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/terminal_drive.rs` `footer`, the arm after the
  pending card: `let written = drive.last_write.as_ref()?;`.
- **Decisions:** D1.

### Design
- Approach: in `footer`, the bar's arm takes the last write only while `drive.program ==
  foreground_program(&view, cx)`; the taken-over arm is unchanged. The terminal re-renders on the
  shell's prompt output, so the bar goes at the next frame.
- File manifest: `crates/marley_workbench/src/terminal_drive.rs` (Marley crate). No ledger row.

### Visual check plan
| REQ | The scenario and the shot |
|---|---|
| REQ-001 | #525's scenario: 525-11-shell-refused after Ctrl-D: no bar. |
| REQ-002 | 525-03-typed and 525-10-never-ask still show the bar while Python runs. |
| REQ-003 | New: `ech`, Ctrl-I at the shell's prompt; 525-11b-tab-completes shows `echo`; the line is then cleared with Ctrl-U. |

### Risks
- `foreground_program` runs on each footer render while a drive exists: one `tcgetpgrp` call on
  the pty (`pty_info.rs`), not a process scan, and only for a terminal an agent has read or
  typed into.

## Phase 2 — Code
- Built: `footer`'s bar arm filters the last write on `drive.program ==
  foreground_program(&view, cx)`, with a comment on why. #525's scenario gained the Tab step
  (`ech`, Ctrl-I, `echo` on the screen) before the gate, since the receipt covers scenarios.
- Review of the diff: the comparison reads the same two values `drive()` compares, from `&App`,
  so the footer mutates nothing; `view` is borrowed before the bar's click handler takes it. The
  pending card and the taken-over arm are untouched.
- Gate: `just gate-diff`, GATE GREEN [diff], 16 passed, 0 failed (log in the scratchpad).

## Phase 3 — Test
- **Run 1: a panic.** The first write's footer render panicked, "cannot read
  terminal_view::TerminalView while it is already being updated", at `foreground_program`
  (`terminal_drive.rs:109`) called from the new check in `footer`: the footer renders inside
  `TerminalView::render`, so the view is leased and cannot be read. Marley died; the stand-in's
  next call found no Marley. The gate and the review had passed it.
- **The fix:** `foreground_program` split into itself and `program_of(&Terminal)`, which the
  footer calls on `context.terminal`, the entity the footer context already carries and which is
  not leased while the view renders. `just gate-diff` again: GATE GREEN [diff], 16 passed.
- **Run 2:** exit 0, every check passed, no `panicked` in any of the run's logs. The focus
  report: nothing on Hyprland; the run's sway stopped.
- **525-03-typed and 525-10-never-ask:** while Python runs, the bar under it reads "Stand-in
  agent typed into python3: …" with Take Over (REQ-002).
- **525-11-shell-refused:** after Ctrl-D, the shell's `$` and no bar under the terminal
  (REQ-001).
- **525-11b-tab-completes:** `ech` and Ctrl-I at the prompt: `$ echo ` (REQ-003; #525's
  REQ-012), cleared with Ctrl-U before the next step.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Fixed, #595); `docs/marley_architecture/marley_workbench.md`, the footer's
  bar and `program_of`. No Zed path.
- **Knowledge:** F-claude-595-the-drive-bar-outlived-the-program-it-named-001,
  F-claude-595-the-footer-read-its-own-view-while-it-rendered-001,
  PR-claude-a-render-hook-reads-its-context-not-its-view-001. #594's and #525's notes corrected:
  Ctrl-I at the shell's prompt was never taken over.
- **Brain:** consultation 320ca8b1288d4a6ebfa65d167d693e1e closed with no decision.
- **Closed:** TICKET-595 moved to `tickets/closed/`; its BACKLOG row went at promotion.
