# Press an agent's Enter after its paste has landed — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-594-enter-after-an-agents-paste.md
- **Pipeline spec:** 594-enter-after-an-agents-paste.spec.md

## Phase 1 — Plan
- **Request:** found in #593's run of #525's scenario: every `terminal_type` with `submit` left
  its line in Python's REPL as a continuation (`...`), none ran, while the tool answered `typed
  13 bytes`.
- **Classification / tier:** bug, one Marley crate, three call sites.
- **Recall (§18.3):** F-claude-481-the-rich-inputs-check-raced-the-echo-of-its-paste-001 noted
  that the rich input's paste and carriage return arrive close together; its stand-in reads
  lines, so no check saw the Enter taken as text. Nothing else on this seam. The brain
  (consultation afc1d0c617a045d5afcc3e8c42e0c1e6) had nothing on it.
- **Discovery:** `terminal_drive.rs` `write` (paste, `try_keystroke` per key, `input(b"\r")` in
  one update), `rich_input.rs` `send` (paste and `\r` in one update), `review_notes.rs` (the same
  inside a deferred window update). CPython 3.14.7's `_pyrepl.commands.perform_bracketed_paste`
  loops on `console.getpending()` until the end marker and inserts everything read, the marker
  removed.
- **Decisions:** D1 (200 ms), D2 (only what follows a paste waits), D3 (answer after the Enter).

### Design
- Approach: `pub(crate) const AFTER_PASTE` and `pub(crate) fn paste_then(terminal:
  &Entity<Terminal>, text: &str, after: impl FnOnce(&mut Terminal) + 'static, cx: &mut App) ->
  Task<()>` in `terminal_drive.rs`: `terminal.paste(text)` now, then a spawned task that waits on
  the background executor's timer and runs `after` through the terminal's weak handle (a closed
  terminal logs and skips). `write` becomes `async fn write(typing: &Typing, cx: &AsyncApp)`:
  keys and Enter in one `after`, awaited through `paste_then` when there is text, run at once when
  there is none; then the last write, the notify and the answer. `type_into` and
  `ask_then_type` await it; `ask_then_type`'s allowed branch decides in its update and writes
  after it. `rich_input::send` and `review_notes` call `paste_then(.., |terminal|
  terminal.input(b"\r".to_vec()), cx).detach()`.
- File manifest (all Marley crate): `crates/marley_workbench/src/terminal_drive.rs`,
  `crates/marley_workbench/src/rich_input.rs`, `crates/marley_workbench/src/review_notes.rs`.
  No ledger row.

### Visual check plan
| REQ | The scenario and the shot |
|---|---|
| REQ-001 | #525's scenario: each submitted write checked on the screen (`terminal-screen` rows hold `42`, `again`, `handed back`, …); 525-03, 525-04, 525-10. |
| REQ-002 | Review: keys go in the same `after` as Enter, after the pause. |
| REQ-003 | A new step: `exec -a claude python3 -q` in a second terminal, Ctrl-G, `print(6 * 7)`, Enter; 525-12 shows `42`. |
| REQ-004 | #522's scenario, run once; its shots show the stand-in's `got:` lines. |
| REQ-005 | Review: `type_into` answers after `write` returns, which is after `after` ran. |

### Risks
- A program slower than 200 ms to read its paste would still take the Enter as text; the pause
  is a time, not a signal. The spec keeps a signal out of scope.
- `terminal_type` calls now take 200 ms longer when they paste and press keys.

## Phase 2 — Code
- Built: `AFTER_PASTE` and `paste_then` in `terminal_drive.rs`; `write` is async, its keys and
  Enter in one `after` closure, through `paste_then` when the write has text and at once when it
  has none, then the last write and the answer. `ask_then_type`'s update now only decides
  (approving on Allow), and the write follows it. `rich_input::send` and `review_notes` detach
  `paste_then` with Enter as the `after`. No deviation from the plan.
- Review of the diff: a terminal closed during the pause makes the weak update fail, which is
  logged and skips the keys; nothing panics. The take-over check is not repeated after the
  200 ms pause, so a take-over inside it would still get the keys; it is 200 ms after the user
  allowed the write, and the notes keep it. `terminal_type` with text alone now answers 200 ms
  later. The allowed branch still sets `approved` before anything is typed, as before.
- Gate: `just gate-diff`, GATE GREEN [diff], 16 passed, 0 failed (log in the scratchpad).

## Phase 3 — Test
- **The scenario:** `script/e2e/525-agent-types-into-a-terminal.sh` (`compositor sway`), now
  with each submitted write checked on the screen (`screen_shows`: a row that is the output, not
  the line typed) and a rich-input step. Run 1 passed every write's check and failed the rich
  input's, which looked the terminal up by a `claude` title: after `exec -a claude python3`,
  Python keeps the shell's pid, so the title stays `repo — bash -q` and the screen reports no
  program. Run 2 looks it up by the project's name (the only terminal), shoots the rich input
  open and checks the text waits there before Enter: exit 0, every check passed. The focus
  report: nothing on Hyprland; the run's sway stopped.
- **525-03-typed:** after Allow, `>>> print(6 * 7)` then `42` then `>>>`; the bar "Stand-in
  agent typed into python3: "print(6 * 7)" <enter>" with Take Over (REQ-001).
- **525-04 … 525-10:** `again`, `handed back`, `asked again` and `never asked` each printed
  under its line; the denied `print('denied')` and the unanswered `print('unanswered')` appear
  nowhere (checked on the screen too).
- **525-11-shell-refused:** Ctrl-D left Python (`$` again, the tab back to `repo — bash`); the
  write at the shell's prompt refused.
- **525-12-new-program-asks:** a new Python's first write shows the card again; Deny.
- **525-13-rich-input-open / 525-14-rich-input-ran:** Python under the name `claude`: the agent
  bar (Claude Code) and the rich input above it holding `print(6 * 7 + 1)`, not yet on the
  screen; after Enter, `>>> print(6 * 7 + 1)` and `43` (REQ-003).
- **#522's scenario, once:** exit 0, its three checks passed. 522-05-sent shows the stand-in's
  `got:` for `File: notes.txt`, `Line: 3` and `User comment: "Say three in lower case"`, the last
  line ended by the Enter that now comes 200 ms later; 522-07-sent-mark shows the note marked
  Sent (REQ-004).
- **REQ-002, REQ-005:** the review in Phase 2; the keys share the Enter's `after`.
- **Found, for #525 (not this ticket):** in 525-11 the bar "Stand-in agent typed into python3 …
  Take Over" stays under the shell after Python exits. The footer reads the stored last write
  without asking whether its program still runs. Filed as #595. (As first written here, this
  said Ctrl-I at the shell's prompt would take over; #595's Plan read `toggle_control`, which
  calls `drive()` first and so drops the stale write: the key reaches the shell.)
- **Seen, not in scope:** the agent row in 525-13 and 522-05 is titled with the user and host
  name that the host's bash sets as the terminal's title; shots stay in the scratchpad.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Fixed, #594); `docs/marley_architecture/marley_workbench.md`, the rich
  input's Enter, `write` and review notes' `deliver`, each through `paste_then`. No Zed path.
- **Knowledge:** F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001,
  PR-claude-keys-after-a-paste-go-in-a-later-write-001. #525's completed notes gained its
  after-the-fact visual check (#593, #594, #595).
- **Brain:** consultation afc1d0c617a045d5afcc3e8c42e0c1e6 closed with a decision, the 200 ms
  pause (follow-up 2026-10-29).
- **Filed:** TICKET-595 (the drive bar and Ctrl-I outlive their program), top of the Queue.
- **Closed:** TICKET-594 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **Commit:** with #525's scenario and the fixture's `terminal-screen` and `terminal-type`.
