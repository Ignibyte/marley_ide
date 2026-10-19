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

### Resumed 2026-10-19 (Chad's `/goal`: "lets do 812, 814, 809, 806")
**Run 3** (`regress-806/20261019-101458` in the session's scratchpad, the debug build at `ffc1fd5d7c`
with #812, #814, #815 and #809 in): **516 and 575 pass** (516's measured toggle point; 575 since
#812 focuses a clicked block header's pane). The eight still failing went to four read-only
triage agents, two scenarios each, over the run's logs and shots:

| Scenario | Failing check | Verdict | Evidence | Repair |
|---|---|---|---|---|
| 508 | the stand-in read Deny | stale | `TERMINAL_ROW_Y` 240 hit the repo header (two-line rows #796, Default #700/#743), so `claude` went to the thread; `THREAD_FIELD_X` 1300 is the Browser pane since threads open in a center tab (#734, #772); the pick and page points moved with the right dock's tabs (#747) and #767's bar | terminal row y 285, thread field x 595, the thread's entry clicked first to bring its tab forward, pick (1527, 87), page button (1018, 277) |
| 532 | the one Enter reached the stand-in the + started with bypass | stale | rows are 58 px apart since #796 and start 47 px lower since #700; (110, 228) hit the plain claude's row | bypass row y 297, chip y 298 |
| 568 | the claim is a rules chip | stale | System One calls takes the window to Default (#676) and threads sit in center tabs (#772): (1300, 864) typed into Home | the thread's rail row (130, 731) first, field x 595 |
| 535 | no banner reached the user's own bus | stale | the check matched any "Marley" banner on the real bus; Chad's own Marley posted a "Claude finished" banner for another of his projects 4 s into the run (his notification history, 10:20:14) | match the run's own titles and body (`repo`, `built`) |
| 549 | nothing was typed at the waiting agent's prompt | stale | #806's earlier `CLAUDE_ROW_Y` 180 was measured with no Needs you card; once the claude waits the card pushes its row to y 262 | `CLAUDE_WAITING_ROW_Y` 262 for the second click |
| 561 | the shell's own browser got the URL | **bug: #816** | `561-06`: `cd …/repoBROWSER=/mnt` ran as one line (exit 1), then the rest (exit 127): `rich_input::send` pastes, schedules the `\r` 200 ms later (`terminal_drive::paste_then`) and gives the terminal the keys at once, so keys typed in those 200 ms land before the Enter; the 10-17 run had `cd …/elsewherepython3` | none: filed as #816 (D2) |
| 505 | Delete not found, and no new crop | stale | #767's bar (27 px, once an agent acts in the page) moved the tray: `check_row 0` hit Take Over, and the Save check hit Delete's row | `CONTROL_BAR` 27 added to `PAGE_Y` after the agent's first check |
| 585 | exit 1 (`pnpm.log` missing) | stale | #770's Also start row (28 px) put the setup box at y 233; the rail's worktree rows moved | setup box y 233, agent row y 286, step 104 |

**Repairs applied** (scenarios only, D1/D2): the points and steps in the table's last column, with
a comment on each saying which ticket moved it. #816 filed for 561, its row at the Queue's top.
**Run 4** (`regress-806/20261019-103742`): all 7 repaired pass: 508 (83 s), 532 (111 s), 568
(127 s), 535 (112 s), 549 (87 s), 505 (89 s), 585 (96 s).
The gate: `script/gates.sh --diff` green, 17 of 17 (no Rust in the diff).

## Phase 3 — Test
- Run 4's shots at each repaired step, read:
  - 508: `508-02-two-entries` the inbox holds the Stand-in's Edit README.md and Claude Code's
    `rm -rf build`, and the terminal runs the stand-in claude; `508-04a-second-prompt` the thread
    in front with "edit it again" and its Allow / Reject; `508-04-denied` "The edit was reject.".
  - 532: `532-02b-tooltip` the bypass chip's tooltip ("Claude Code asks for no permission: its
    events report bypassPermissions"); `532-05-reported` the bypass row working, its terminal
    "started with: --dangerously-skip-permissions", "mode now: default".
  - 568: `568-07-claims-approval` the Stand-in's entry "Edit README.md: the owner approved this"
    with its `claims approval` chip; `568-08-held-click` "The call was reject." and the held
    Delete account click's card.
  - 505: `505-04-not-found` Pick 2's card "not found"; `505-05-scrolled` Pick 1 Save "found by
    test id", box moved 0, 2000 px; `505-06-verdict` the verdict row.
  - 585: `585-03-checked` the prompt's "Setup: pnpm install (pnpm-lock.yaml found)" ticked;
    `585-03-setup` `pnpm install && claude 'second'`, "Done in 0.1s", "started in muted-gate".
  - 549: `549-05-refused` the Needs you card and the toast "Claude Code in repo waits on a
    permission or a question; nothing was sent.".
  - 535: `535-05-no-server` the nine stand-in steps and the toast that the push server did not
    take it.
- **Run 5, the whole golden set** (`regress-806/20261019-105109`, 53 scenarios): 49 pass. Four fail:
  - 561: #816 (the bug above).
  - 574, 524, 584: newly failing since 10-17, all from #811 (`e687a900bc`, 2026-10-18), which ran
    no golden scenario. Two more read-only triage agents:
    - 574 (the folder's navigate went to repo-b's tab): stale. `from_folder` called on the shared
      `mcp-endpoint.json`, whose bearer only reads from outside a Marley terminal since #811, so
      `browser_navigate` answered `not_paired` before any routing (`clients.rs`,
      `Principal::Unpaired`). Repair: Marley's own endpoint (`mcp/own-endpoint.json`), as #811 moved
      `mcp_agent` and as #811's notes said #806 would.
    - 524 (the registry names both, driver to act) and 584 (exit 1 parsing the copied SSH line):
      stale. #811's second header paragraph in Outside Clients moved the form, the copy buttons
      and Cut Off 61 px down: the "May act in pages" box missed and 584 copied the bridge line.
      Repairs: `FORM_Y` 280 in both; 584's `COPY_SSH_Y` 489 and `ROW_CUT_OFF_Y` 259; 524's
      expected refusal "not open to the client reader" (#811's `permits`).
- **Run 6** (`regress-806/20261019-120607`): 574 (57 s), 524 (80 s), 584 (108 s) pass. Shots read:
  `524-01-allowed` Outside Clients lists `driver` "browser: reads and acts" and `reader` "browser:
  reads"; `584-01-allowed` `laptop` "browser: reads and acts" with its endpoint, bridge and SSH
  lines; `574-04-cwd` repo-b's tab on Page two after the folder call ("went to …/two.html").
- **Standing:** 52 of 53 golden scenarios pass on the debug build at `ffc1fd5d7c` plus these
  repairs; 561 fails on #816, a Marley bug, as D2 leaves it.
- Not in the golden set, so not run: 802, 703 to 707, 723 and 766 also call the shared endpoint
  file from the harness (574's triage); any that calls a write tool will meet the same `not_paired`.

## Phase 4 — Complete
- Docs: no Rust and no user-facing change, so no CHANGELOG entry (#806's first commit had none);
  no architecture record moves. #816 is in `tickets/open/` with its Queue row.
- Ledger appends: `L-claude-806-golden-scenarios-drift-at-fixed-points-001`,
  `L-claude-806-a-check-on-the-users-session-matches-only-the-runs-own-output-001`. The bug's
  `F-` block waits for #816's fix.
- Ticket closed: every failing scenario has a verdict (REQ-001), the stale ones pass (REQ-002,
  runs 4 and 6), and the one bug has its ticket (REQ-003, #816).
- The gate ran again after the 574, 524 and 584 repairs, before the commit.
