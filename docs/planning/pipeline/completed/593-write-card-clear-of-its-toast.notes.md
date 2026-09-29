# Keep the terminal write card's Allow and Deny clear of its toast — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-593-write-card-clear-of-its-toast.md
- **Pipeline spec:** 593-write-card-clear-of-its-toast.spec.md

## Phase 1 — Plan
- **Request:** #525's after-the-fact visual check, shot 525-02-ask: the card reads "Stand-in
  agent wants to type into python3: "print(6 * 7)" <enter>" at the left, and the toast ("Stand-in
  agent wants to type into python3. Allow it or deny it under the terminal." with Show) covers
  the right end, where Deny and Allow are. The scenario's click there hit the toast, and the
  write was refused after 25 seconds.
- **Classification / tier:** bug, one Marley file, a layout change.
- **Recall (§18.3):** nothing in `docs/planning/knowledge/` on a toast over a control. #525's
  notes describe the card and the toast (REQ-002) but not where each lands. The brain
  (consultation 5bc349896e124e2ba85362af235b9010) had nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/terminal_drive.rs` `footer`, the `pending` arm:
  `bar` with the label (truncating), `div().flex_1()`, Deny, Allow.
- **Decisions:** D1 (Allow, Deny, then the text), D2 (the toast unchanged).

### Design
- Approach: in the pending arm, the children become Allow, Deny, then the label; the spacer
  goes, since nothing follows the text. The label keeps `.truncate()`, so a long write is cut at
  the row's end, not pushed past the buttons.
- File manifest: `crates/marley_workbench/src/terminal_drive.rs` (Marley crate). No ledger row.

### Visual check plan
| REQ | The scenario and the shot |
|---|---|
| REQ-001 | 525-02-ask: the first write waits; the card's buttons at the left, the toast at the right. |
| REQ-002 | A click on Allow with the toast up; 525-03-typed shows `42`, the stand-in prints `typed … bytes into python3`. |
| REQ-003 | Under `ask_every_write`, a click on Deny; 525-08-denied, the stand-in prints the refusal. |

### Risks
- A terminal in a pane narrower than the toast is still covered at its left end; the toast's
  Show and its close stay usable, and the scope says so.

## Phase 2 — Code
- Built: the pending arm of `footer` now adds Allow, Deny, then the label; the spacer went. A
  comment on the arm says why the buttons lead. No deviation from the plan.
- Review of the diff: the buttons keep their ids and handlers; `div` is still used by the drive
  bar's spacer, so no import changed. The label, last in the row, still truncates.
- Gate: `just gate-diff`, GATE GREEN [diff], 16 passed, 0 failed (log in the scratchpad).

## Phase 3 — Test
- **The scenario:** `script/e2e/525-agent-types-into-a-terminal.sh` (#525's after-the-fact
  check, `compositor sway`): a Python REPL in the project's terminal and a stand-in agent through
  the plugin's bridge, with the fixture's new `terminal-screen` and `terminal-type`. Its clicks
  on Allow (290, 954) and Deny (345, 954) now aim at the card's left end. One run on the new
  debug build; the focus report: the run's sway stopped, nothing on Hyprland.
- **525-02-ask:** the card reads `Allow  Deny  Stand-in agent wants to type into python3:
  "print(6 * 7)" <enter>` from the terminal's left edge; the toast, "Stand-in agent wants to type
  into python3. Allow it or deny it under the terminal." with Show, sits at the right, clear of
  both buttons (REQ-001).
- **525-03-typed:** after the click on Allow with the toast up, the card is gone, the bar reads
  "Stand-in agent typed into python3: …" with Take Over, and the stand-in printed `typed 13 bytes
  into python3` (REQ-002).
- **525-07-asks-every-write / 525-08-denied:** under `ask_every_write` the card asks again with
  the toast up; after the click on Deny the stand-in printed "the user refused this write to
  python3" and `print('denied')` appears nowhere on the screen (REQ-003).
- **525-09-timed-out:** an unanswered write, refused after 25 seconds; no card, no toast.
- **Found, for #525 (not this ticket):** every submitted line sits in the REPL as a continuation
  (`...`), none ran: `write` sends Enter in the same burst as the paste, and Python's REPL reads a
  bracketed paste to its end marker with whatever came after it. The scenario's last step failed
  on it (Ctrl-D does not leave a non-empty buffer). Filed as #594, which carries the scenario.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Fixed, #593); `docs/marley_architecture/marley_workbench.md`'s #525
  section, the card's order. No Zed path touched.
- **Knowledge:** F-claude-593-the-write-toast-covered-the-cards-allow-and-deny-001.
- **Brain:** consultation 5bc349896e124e2ba85362af235b9010 closed with no decision (a layout
  fix).
- **Filed:** TICKET-594 (the Enter after an agent's paste), top of the Queue; it carries #525's
  scenario and the fixture's two commands, which wait for it.
- **Closed:** TICKET-593 moved to `tickets/closed/`; it never had a BACKLOG row.
