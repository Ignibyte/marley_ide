# The manager in the Agent Panel — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-694-the-manager-in-the-agent-panel.md
- **Pipeline spec:** 694-the-manager-in-the-agent-panel.spec.md

## Phase 1 — Plan
- **Request:** the cron's item 7, unblocked by the harness's TICKET-111, which closed 2026-10-08.
- **Recall:**
  - #683 (a custom agent server in the defaults);
  - #691 (`seat_command`);
  - #685 (the panel shows an ACP agent's posts and permission requests between turns);
  - L-683 (no new fields on Zed's enums).
- **Design:**
  - `Harness.manager_entry: Option<(PathBuf, Vec<String>)>`, the command now in the defaults.
  - `sync_manager_entry(cx)`, called after `seed`, after each page of events, and at the end of
    `follow_setting`. It inserts or removes `agent_servers["Manager"]` with
    `CustomAgentServerSettings::Custom`.
  - **Manifest:** `harness.rs`, the scenario, and the docs.
- **Visual check plan:**
  - Actor `boss`: publish running, wait for a message, output "got it", publish idle, then
    sleep. Designate it with `rh manager <id>`.
  - Open the project's + → New Agent Thread for `694-01-menu`. Choose Manager, check the
    thread runs `rh acp` (the stand-in guard of PR-687: `rh thread show` names the root's
    manager), type "hello manager", and take `694-02-thread`.
  - Check `rh thread show` holds the person's record "hello manager", and the actor's output
    holds "got it".
- **Risks:** this harness's `rh acp` is new. A turn that never ends leaves the panel generating,
  and the shot will show it.

## Phase 2 — Code
- **Built:**
  - `MANAGER_ENTRY`, `Harness.manager_entry` and `sync_manager_entry`.
  - It is called after `seed`, after each page of events, and in `follow_setting` on both paths.
- **Review:** the entry changes only when its command does. The defaults update fires the
  settings observer once more, and that call finds nothing to do.
- **Gate:** run 1 was red (`assigning_clones`); it now uses `clone_from`. Run 2
  (`scratchpad/694-gate-2.log`): **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/694-the-manager-in-the-agent-panel.sh`.
  - Run 1 measured the submenu.
  - Run 2 passed 2 of 2 (`scratchpad/694-e2e-2.log`): `rh acp` runs on the scratch root, and
    "hello manager" is in the thread as the person's message.
- **Shots:**
  - `694-01-menu` (REQ-001): New Agent Thread's submenu reads Zed Agent, Claude Agent, then
    Manager.
  - `694-02-thread` (REQ-002): a Manager thread titled "hello manager", with the message and the
    turn still running. The rail reads "hello man… Manager · w…", and the harness section reads
    boss waiting.
- **Not Marley's (reported to the harness 2026-10-08):**
  - The turn does not end because the harness never hands the message to boss. The same
    happens with no Marley: a fresh root, an actor manager in its message wait, and
    `rh thread send`. `rh thread show` has no handoff, and `rh fleet` shows boss waiting.
  - The harness's ACP ends a prompt only after the hand-off, so the panel keeps the turn
    running.
  - Marley's part, the entry and the message reaching the thread, is shown.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

- **Run 3, after the gate's shellcheck (SC2009):** the guard now uses `pgrep -af`. 2 of 2
  (`scratchpad/694-e2e-3.log`). The gate is green (`scratchpad/694-gate-4.log`).
- **The harness confirmed it:** its hand-off fires only for a manager envelope that reads idle,
  and an actor in a message wait reads waiting. The fix goes into its TICKET-112, logged as a
  TICKET-106 defect.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md` (#694).
  - The guide's "The manager in the Agent Panel".
  - `marley_workbench.md`'s Writes entry.
  - The plan's item 7, marked done.
- **Knowledge:** `AD-claude-694-the-manager-entry-follows-the-role-label-001`.
- **Ticket:** TICKET-694 closed.
