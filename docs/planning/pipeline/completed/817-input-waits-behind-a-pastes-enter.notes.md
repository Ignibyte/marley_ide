# Input waits behind a paste's Enter — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-817-keys-typed-after-enter-reach-an-agent-before-it.md
- **Pipeline spec:** 817-input-waits-behind-a-pastes-enter.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19: "lets do 817". Split from #816.
- **Classification / tier:** bug; one Zed crate (`terminal`, a field and two methods, a hunk in
  `input`), one Marley crate (`marley_workbench`, `paste_then`).
- **Recall (§18.3):**
  - F-claude-594 / PR-claude-keys-after-a-paste-go-in-a-later-write-001: the 200 ms stays.
  - #816 (this session): the shell's editor writes paste and Enter together; F-claude-816 named
    the window for agents.
  - #641: `marley_input_held` refuses input for a remote terminal whose link is down; the new
    queue sits after that check.
  - The brain (consultation 1f574e4d347249568a54591bd3a588f6): #816's decision only.
- **Discovery:** `Terminal::input` (terminal.rs ~2780) is the path of typed keys
  (`try_keystroke`), pastes and Marley's writes; `paste_then` (terminal_drive.rs ~326) is used by
  rich_input (agents), terminal_drive's write, send_selection, delegation, resume, limits and
  review_notes.
- **Decisions:** D1 to D3 in the spec.

### Design
- `terminal.rs` (Zed crate): a field `marley_queued_input: Option<(Instant, Vec<u8>)>`, the hold's
  deadline and the bytes; `marley_queue_input(until)` starts a hold or moves its deadline later;
  `marley_release_input(owed)` takes the queue, runs `owed(self)` and then `self.input(queued)`.
  In `input`, after the #641 check: while a hold's deadline is ahead, the bytes join the queue
  and nothing else happens; past it, the queue is released first and the input goes on.
- `terminal_drive.rs` (Marley): `paste_then` sets the hold right after its paste, deadline
  `AFTER_PASTE * 10`, and its task calls `marley_release_input(after)`.
- File manifest: `crates/terminal/src/terminal.rs` (Zed, its touchpoint row extended first),
  `crates/marley_workbench/src/terminal_drive.rs` (Marley),
  `script/e2e/817-input-waits-behind-a-pastes-enter.sh` (new).

### Visual check plan
- REQ-001, REQ-002: under sway, a stand-in `claude` (python, reads lines, prints and logs
  `got: <line>`) first on the PATH of the scenario's `.bashrc`; `claude` run from the shell's
  editor; Ctrl-G opens the agent's editor; `wtype -d 20 "first" -k Return "second" -k Return`;
  `817-01-agent` shows both lines; `expect` reads the log's last two lines, `got: first`,
  `got: second`. Once on the installed build: it should log `got: firstsecond`.
- REQ-003: `just e2e script/e2e/816-the-shells-enter-goes-with-its-paste.sh`.
- REQ-004: the diff's deadline branch in `input`.

### Risks
- The stand-in reads the line in the terminal's cooked mode, as Claude Code does not; the queue's
  order is what it shows, and Claude Code reads the same bytes in the same order.

## Phase 2 — Code
- Built to the manifest. `terminal.rs` (its touchpoint row extended first): the field
  `marley_queued_input`, both struct literals, `marley_queue_input(until)` (a hold, or a later
  deadline for one on), `marley_release_input(owed)` (takes the queue, runs `owed`, then
  `self.input(queued)`), and in `input`, after the #641 refusal, `input.into()` once, then the
  queue's check: bytes join it while its deadline is ahead; past the deadline the queue is let
  out first and this input follows. `terminal_drive.rs`: `paste_then` sets the hold right after
  its paste (`AFTER_PASTE * QUEUE_DEADLINE_TIMES`, 2 s) and its task calls
  `marley_release_input(after)`; the doc comment says so.
- Review: the release takes the queue before `owed` runs, so the owed `\r` is written directly
  and the queued bytes then go through `input`'s usual path (`keyboard_input_sent`,
  `note_input`, `write_input`) in order (D3); a hold nobody releases ends at its deadline on the
  next input (REQ-004); the #641 refusal still comes first; #816's shell path sets no hold.
  Zed hunk: a field, two methods, one block in `input`, each with `// Marley:`.
- Check runs: the installed build logged `got: firstsecond` then `got: ` (the bug); the debug
  build logged `got: first`, `got: second`.
- The gate: `just gate-diff` green, 17 of 17.

## Phase 3 — Test
- `just e2e script/e2e/817-input-waits-behind-a-pastes-enter.sh` twice on the debug build after the
  green gate, under sway; focus report "1 Marley windows before the run, 1 after" each time. Both
  pass. `817-01-agent` (cropped beside the installed build's): the stand-in's tty echoes `first`,
  then `second`, each on its own line, and it prints `got: first`, `got: second`; the agent bar
  reads Claude Code and the toast says Ctrl-G opened the editor (REQ-001, REQ-002). The installed
  build's shot: `firstsecond`, `got: firstsecond`, then an empty `got:`.
- #816's scenario on the same build: both shells pass (REQ-003).
- REQ-004 rests on the review (the deadline branch); no scenario drops a release.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #817); `docs/marley_architecture/marley_workbench.md`, the
  `terminal_drive` section's `paste_then` bullet; the `zed-touchpoints.md` row for `terminal.rs`
  read against what shipped. No slice status moves.
- Ledger appends: `F-claude-817-keys-typed-after-an-agents-prompt-joined-it-001`,
  `PR-claude-a-delayed-write-holds-the-terminals-input-001`.
- Brain: consultation `1f574e4d347249568a54591bd3a588f6` closed with `brain decide` (follow-up
  2026-11-19).
- Ticket closed; its `BACKLOG.md` row left at promotion.
