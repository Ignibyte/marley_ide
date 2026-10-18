# Triage and repair the golden set — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-806-triage-and-repair-the-golden-set.md
- **Pipeline spec:** 806-triage-and-repair-the-golden-set.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "yep lets work through see if they are actual bugs or regression
  spec issues".
- **Recall:** #797's notes: the 29 failed at the same check on `206b4ed3ed` and on the merge; two
  read from shots (532: the project's + moved under the Default group; 520: a split opens Home
  since #785). #695 left 7. L-claude-803: a chip's point follows the output's line count after a
  clear, blocks stacking from the bottom.
- **Plan:** one full run on head `b0fb8af84a`; the failures split by area among read-only triage
  agents that read each run's log, shots, scenario and the tickets since the scenario's own; a
  verdict per scenario; repairs to stale scenarios, one run each; bugs filed.

## Phase 2 — Code
### The triage (five read-only agents on #797's run, `scratchpad/regress-797`, checked against the
passing run of 2026-10-08, `regress/20261008-203157`)
All 29 are STALE: each scenario still clicks or expects what a later, deliberate change moved.
| Cause (ticket) | Scenarios |
|---|---|
| The rail's Default group above the projects (#700 `a252347cae`, named by #743 `309370d75e`), and terminal rows two lines tall (#796 `206b4ed3ed`): fixed rail points land on a group header or the wrong row | 503, 504, 507, 511, 521, 549, 561, 565, 566, 579, 581, 586 |
| The right dock's panels as tabs (#747 `fb109b8906`): browser toolbar, chip and card buttons about 240 px further right | 505, 518, 523, 524, 571 (and 503's strip offer) |
| A split opens Home (#785 `5020ee34f8`) | 520, 575, 577 |
| The row's + menu: New Terminal, New Browser Tab, New Agent…, Reopen Thread… (#744 `358f910a61`, #772 `eeff2d27d8`), and a menu opening with nothing chosen (Zed #64365 via #695) | 508, 510, 532, 568, 585, 587 |
| `claude --version` probed at start (#648 `701b020874`): stand-ins log it as a launch or read stdin | 508, 510, 532, 569, 585, 587 |
| Permission requests as their own push, `needs approval`, `lock` (#752 `ad27c1d635`) | 535 |
| Two settings rows above Redact Secrets (#673 `ac4e4c56e9`, #759 `6594e7f7fa`): its toggle below the window | 516 |
- **Not a scenario's fault, noted:** in a fresh profile that opens a project directly, the rail's
  Default group lists no Home until it is shown (`511-01-menu`); once something opens there, Home
  and its rows appear (`565-04-masked`). #782's REQ-008 holds ("WHILE a group holds Home's page"),
  so this is not filed; it is put to Chad beside #794.
- **A wrong claim corrected:** #796's notes say no golden scenario clicks the rail at fixed
  coordinates; twelve do.

### The repairs, run 2 (`regress/20261017-202946`, the debug build at `b0fb8af84a`)
**19 of 29 pass:** 503, 504, 507, 510, 511, 518, 520, 521, 523, 524, 565, 566, 569, 571, 577,
579, 581, 586, 587. Repairs: rail points under the Default group and two-line rows; browser,
chip and card points at the window's edge; a terminal opened with `workspace: new terminal` after
each split; the + menu's New Agent… and New Agent in Worktree with Home first; every stand-in
`claude` quiet on `--version`; 535's `needs approval` push; 565's row once System One calls opens
in Default; 516's settings page scrolled.
**10 still fail, each further on than before:**
- 516: the toggle's point after the scroll was unknown; measured since at (1548, 873).
- 508, 532, 568: the + menu's New Agent… picker asks where to start ("Where should Claude Code
  start?", the project first); the helper now presses Return a second time.
- 535 (`no banner reached the user's own bus`), 575 (`the right terminal's id held`), 549 (`nothing
  was typed at the waiting agent's prompt`), 561 (`the local URL from outside reached xdg-open`),
  505 (`Delete not found, and no new crop`): with a second triage agent.
- 585: exits in setup of its prompt; SETUP_BOX, AGENT_ROW_Y and WORKTREE_STEP to measure.
- **Parked 2026-10-17** for #807 and #808, which Chad asked for next; resumes at the 10 above.
