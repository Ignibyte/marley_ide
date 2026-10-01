# Claude Code sessions resumed after a Marley restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-540-session-resume-after-restart.md
- **Pipeline spec:** 540-session-resume-after-restart.spec.md

## Phase 1 — Plan
- **Request:** from the Orca survey (2026-09-25, report 05 §3 item 3, report 01 §3 item 7), held
  Deliberate for the harness question; Chad picked it on 2026-10-01 ("lets do 637, 631, 540"), run
  autonomously (a session goal).
- **Classification / tier:** feature, prong 2 (sessions). Marley crates (`marley_agent`,
  `marley_workbench`) and the setting's three Zed-path files, each with its row.
- **Checklist:** pick ✓, pre-flight ✓ (no active pipeline, README marker present; rustal-os runs a
  mutation pass in its own target folders, so Marley's cargo runs at `nice 19`), recall ✓,
  promote ✓ (the queued pair, re-verified and redesigned), prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-575 and F-claude-575: a restored terminal keeps its `MARLEY_TERMINAL_ID` through
    Marley's table, read from memory at the restore because the terminal panel's cleanup deletes
    rows mid-restore. That id is this ticket's key, so the queued draft's two Zed columns go.
  - F-claude-577: a restored center terminal once opened in the project's folder, so the resume
    goes to the session's folder itself (D3).
  - F-claude-547 (`a scenario's click ran the real claude`): here Marley runs nothing itself, it
    types into the terminal, whose PATH the scenario's `.bashrc` leads with the fake; the
    scenario's HOME keeps any real Claude Code off Chad's configuration.
  - AD-claude-519 and the rail's `note_claude_code` (#547): a frame counts only while Claude Code
    is the foreground; the rail ends a seat whose terminal no longer runs it.
  - Brain (`rusty-cli brain ask`, consultation 1e394b3b2cff4ecdbbeb3b28883b1175): nothing on this
    seam.
- **Re-verification (2026-10-01):** the draft's seams moved. `start_cli` now goes through
  `start_in_terminal` (`agents.rs:357`), whose handshake is `start_init_command_startup_handshake`
  raced against `STARTUP_TIMEOUT`, then `write_init_command_after_startup`, which refuses a
  terminal that took input. `TerminalView::deserialize` asks `MarleyTerminalIdentity::saved` for
  the id and `create_terminal_shell_restoring` builds the terminal with it (#575). `HookEvent`
  (`marley_agent::claude_events`) has `source` and no `reason`, though `event.py` sends it.
  `agent_events::end` (called by the rail) ends the seats whose Claude Code left.

### Design
- **`marley_agent`:** `resume_line(mode, session, folder) -> Option<Vec<u8>>`: `None` unless
  `session` is a lowercase or uppercase 8-4-4-4-12 hexadecimal id; else
  `cd -- <quote_argument(folder)> && claude [mode's arguments] --resume <session>` and Enter
  (`send_payload`), the `cd` left out for an empty folder. `HookEvent::reason`.
- **`marley_workbench::resume` (new):**
  - `persistence::MarleyAgentSessionsDb`: `marley_agent_sessions(terminal_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL, folder TEXT NOT NULL) STRICT`; `save`, `remove`, `all`, `retain`.
  - `Sessions` global: the rows by terminal id (read at `init`, written through), the session ids
    resumed this launch, `quitting`.
  - `init` (after `terminal_ids::init`): reads the rows, removes those whose terminal id no saved
    terminal holds (`terminal_ids::known_ids`), sets `quitting` in `on_app_quit`, and observes new
    `TerminalView`s: `resume_restored`.
  - `on_event(terminal, event, cx)` from `agent_events::on_frame` for the lead's events: the
    `SessionStart` and `SessionEnd` rules of the spec, for a local terminal with an id.
  - `ended(terminal_ids, cx)` from `agent_events::end`: drops their rows unless `quitting`.
  - `resume_restored(view)`: setting on, local terminal, its id has a row, the session not claimed
    this launch, a valid `resume_line` → claim, then in a task the handshake, the timeout, and
    `write_init_command_after_startup`; a refused write (the user typed first) logs and drops the
    claim.
- **`agent_events.rs`:** `on_frame` calls `resume::on_event` after the fold; `end` passes the
  ended terminals' ids (`agent_events` knows the views by id; the terminal ids come from the
  rail's terminals) to `resume::ended`.
- **`terminal_ids.rs`:** `known_ids(cx)`, the terminal ids of the saved terminals.
- **Settings:** `resume_agents` in `MarleySettingsContent` with its doc, `"resume_agents": true`
  in `default.json`, a toggle in the Marley page's Agents section; `MarleySettings::resume_agents`.
- **File manifest:**
  - Marley: `crates/marley_agent/src/marley_agent.rs`, `crates/marley_agent/src/claude_events.rs`;
    `crates/marley_workbench/src/resume.rs` (new), `marley_workbench.rs` (module, init, the
    setting), `agent_events.rs`, `terminal_ids.rs`; `script/e2e/540-session-resume-after-restart.sh`.
  - Zed paths (rows exist; each widened first): `crates/settings_content/src/marley.rs`,
    `assets/settings/default.json`, `crates/settings_ui/src/marley_page.rs`.

### Visual check plan
| REQ | Scenario | Shot or log |
|---|---|---|
| REQ-001, REQ-002 | sway; a repository with `sub`; click the terminal; `cd sub`; `claude` (the fake: `session <id> in <repo>/sub`); `quit_marley`, `launch_marley`, settle | `540-01-running` before the quit; `540-02-resumed` (`resumed <id> in <repo>/sub`); the log's second line `--resume <id>` in `<repo>/sub` |
| REQ-003 | `quit_marley`, `launch_marley` | `540-03-again`; a third log line |
| REQ-004 | type `exit` to the fake (its `SessionEnd`, `prompt_input_exit`); `quit_marley`, `launch_marley` | `540-04-exited`: a plain prompt; no fourth line |
| REQ-005 | `claude` again; `quit_marley`; `profile_setting marley.resume_agents false`; `launch_marley` | `540-05-off`: a plain prompt; no new line |
| REQ-006, REQ-007 | review | the claim and `resume_line`'s checks |
Not reachable by a scenario: a real Claude Code resuming a real session (an account and a model
call); the fake stands in, printing what Claude Code's hooks send.

### Risks
- **The quit's order.** If the rail ended the seat as Claude Code died at the quit, the row would
  go; `quitting`, set in `on_app_quit` before the windows close, holds it. The scenario's
  relaunches prove it.
- **The handshake on a restored terminal.** The view is observed as it is created, before its
  shell is up, so the handshake's marker runs as for a new agent terminal.
- **Typing into a shell.** Only a checked id and one quoted folder; a terminal that took input
  first stays a plain shell.

## Phase 2 — Code (2026-10-01)
- **Built:** `marley_agent::resume_line` and `is_session_id`; `HookEvent::reason`;
  `marley_workbench::resume` (the `marley_agent_sessions` table, `Sessions` with the rows, each
  local terminal view's terminal id by seat, the sessions resumed this launch and `quitting`;
  `init`, `on_event`, `ended`, `resume_restored`); `agent_events::on_frame` hands the lead's events
  to `resume::on_event`, and `end` the ended seats to `resume::ended`; `terminal_ids::known_ids`;
  `agents::STARTUP_TIMEOUT` and `launch_mode` made `pub(crate)`; `resume_agents` in the settings
  content, `MarleySettings`, `default.json` and the Marley page's Agents section; the scenario.
- **Deviations:**
  - `ended` takes the seats (terminal view ids) the rail ends, since no lookup from a view id to
    its view exists: `resume` records each local terminal view's terminal id as the view appears
    and forgets it at release.
  - The handshake sequence is written out in `resume_restored` with `agents::STARTUP_TIMEOUT`,
    not shared with `start_in_terminal`: that one runs in a window's async context, whose entity
    updates return results, and this one in the app's.
  - A refused write (the user typed first) logs and keeps the claim: the session is not resumed
    in another terminal that launch either. The table has no `retain`; `init` removes stale rows
    one by one.
- **Dry run (before the gate):** every check passed (`resumed_in_sub` 2 and 3, `runs` 3 and 4).
  The shots also show Zed's startup handshake command (`printf '%s%s%s\n'
  __zed_init_command_ready_ 1 __`) as a block of its own, before the resumed command's output.
  The same handshake starts every agent CLI (`start_in_terminal`), so it predates this ticket;
  minted as a follow-up at Complete.
- **Review:**
  - REQ-001..003: the row is written at `SessionStart` (startup or resume) and read from memory at
    the restore, so a cleanup mid-restore (F-claude-575) cannot take it; a resume's own
    `SessionStart` rewrites the same row, so the next relaunch resumes again.
  - REQ-004: `SessionEnd` with `prompt_input_exit` removes the row; the rail's ending of a seat
    removes it too, unless `quitting`, set in `on_app_quit` before the windows close.
  - REQ-005: `resume_restored` returns at once with the setting off.
  - REQ-006: the claim (`resumed`) is taken before the task starts, so a second view holding the
    id that launch is passed over.
  - REQ-007: `resume_line` types nothing unless the id is 8-4-4-4-12 hexadecimal, and the folder
    is one `quote_argument` word after `cd --`.
  - Re-entrancy: `on_event` runs inside the view's update and reads the terminal, a different
    entity; `resume_restored` runs in `observe_new` and spawns the terminal updates.
  - A remote terminal has no row (`local_id`).
- **Clippy and dylint found:** the first doc paragraph of `resume_line` and of `resume::init`
  too long; `async {}` in the quit hook with nothing to await (`futures::future::ready`); two
  redundant clones; a fourth `bool` on `MarleySettings` (now `ResumeAgents`, as `RustyTools`
  is); `from_settings` one line past a hundred (`ResumeAgents::from_content` reads the content in
  one line).
- **Gate:** `script/gates.sh --diff` GATE GREEN (17 of 17), the third run; the first two were red
  on the lints above. All at `nice 19`.

## Phase 3 — Test (2026-10-01)
- **Build:** `cargo build -p zed --bin marley` at `nice 19`, after the gate's fixes.
- **Scenario:** `script/e2e.sh script/e2e/540-session-resume-after-restart.sh`, a headless sway,
  exit 0; the four log checks pass (`resumed_in_sub 2`, `resumed_in_sub 3`, `runs 3`, `runs 4`).
  Focus report: no Hyprland window before or after, no rule added; sway stopped with the run's
  Marley.
- **Shots, each read (the terminal area cropped):**
  - `540-01-running`: the `cd sub` block, then `claude` running in `…/repo/sub` with
    `session 11111111-2222-3333-4444-555555555555 in …/repo/sub`; the agent bar reads Claude Code.
  - `540-02-resumed` (REQ-001, REQ-002): after the relaunch, `$ cd -- '<…/repo/sub>'` typed at
    the prompt and the stand-in's `resumed 11111111-… in …/repo/sub`, running; the log's second
    line is `--resume 11111111-… @ <repo>/sub`.
  - `540-03-again` (REQ-003): after a second relaunch, `… && claude --resume 11111111-…` and
    `resumed … in …/repo/sub` again; the log's third line the same.
  - `540-04-exited` (REQ-004): after `exit` (the stand-in's `SessionEnd`, `prompt_input_exit`) and
    a relaunch, the terminal `sub — bash` at a plain `$` prompt with the prompt editor; the log
    still three lines.
  - `540-05-off` (REQ-005): a new session started, `marley.resume_agents` set to `false` between
    the quit and the launch: a plain `$` prompt; the log four lines (the new session's start, no
    resume).
- **Seen on the way, pre-existing — not in scope:** Zed's startup handshake command
  (`printf '%s%s%s\n' __zed_init_command_ready_ 1 __`) shows as a block of its own before the
  resumed command's output, as before every agent CLI Marley starts; minted as #639.
- REQ-006 and REQ-007 rest on the review.

## Phase 4 — Complete (2026-10-01)
- **Docs (§21):** `CHANGELOG.md` (Added); `docs/marley/three-prong-plan.md` C3 (#540 shipped);
  `docs/marley_architecture/marley_workbench.md` (`resume.rs`) and `marley_agent.md`
  (`resume_line`, `HookEvent::reason`). Touchpoint rows checked: `settings_content/src/marley.rs`,
  `settings_ui/src/marley_page.rs` and `default.json` each name `resume_agents` (#540).
- **Ledger:** AD-claude-540-a-claude-code-session-resumes-by-the-terminals-id-001,
  L-claude-540-a-queued-spec-is-redesigned-against-what-shipped-since-001. No bug found beyond
  the lints, so no F- or PR- block.
- **Brain:** `brain decide` on consultation 1e394b3b2cff4ecdbbeb3b28883b1175
  (`decisions/marley-resumes-a-claude-code-session-by-the-terminals-id`).
- **Follow-ups minted:** #639 (the startup handshake shows as a block).
- **Ticket:** closed; archived to `completed/`; committed on `marley/workbench-shell`.

