# Claude Code sessions resumed after a Marley restart — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-540-session-resume-after-restart.md
- **Pipeline spec:** 540-session-resume-after-restart.spec.md

## Phase 1 — Plan
- **Request:** from the Orca survey Chad asked for on 2026-09-25 (report 05 §3 item 3, report 01
  §3 item 7): Claude Code sessions resumed with `claude --resume <id>` in the session's first
  folder after a Marley restart, the id from #519's `SessionStart`, kept per terminal. Chad's call
  on the survey's open question 9: it waits, because the embedded rustal-harness may keep
  terminals alive instead.
- **Classification / tier:** feature, prong 2 (sessions). Rust in `marley_workbench` (Marley
  crate) and two files of Zed's `terminal_view` crate; one Marley setting. Deliberate, and after
  #519.
- **Recall (§18.3):**
  - L-claude-494-zed-item-ids-change-at-each-launch-001: "An item's `ItemId` is its entity id, new
    at each launch … A table keyed by anything but the item id outlives its items." It settles D1:
    the session goes on the terminal's own row, which Zed's own save and cleanup keep right.
  - L-claude-455-driving-a-real-open-and-restore-through-new-local-001 and #494's scenario: a
    scenario proves a restore with `quit_marley` then `launch_marley` on the same profile.
  - L-claude-491-ctrl-q-in-a-terminal-goes-to-the-shell-001: quit through the palette, which
    `quit_marley` does.
  - L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001: a stand-in agent leads
    the terminal's foreground only when started with `exec -a claude`, so the rail sees it.
  - L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001: the fake prints its
    frames itself rather than relying on a real Claude Code's hooks.
  - Brain: not consulted at drafting (read-only overnight drafting); the promotion runs
    `brain_ask`.
- **Discovery:**
  - #519's spec (drafted the same night, `519-claude-code-events-in-the-rail.spec.md`): its frame
    carries `session_id` and `cwd` on every event, `source` on `SessionStart` and `reason` on
    `SessionEnd`; `agent_events` keeps a seat per terminal and ends it on `SessionEnd`, on Claude
    Code leaving the foreground, or on the terminal closing; its Out names session resume as a
    later ticket, this one.
  - `crates/terminal_view/src/persistence.rs`: `TerminalDb` (409-455), the `terminals` table keyed
    by `(workspace_id, item_id)` with `working_directory`, `working_directory_path` and
    `custom_title` added by later migrations (415-454); `save_working_directory` (471-501),
    `get_working_directory` (503-509), `save_custom_title` (511-535), `get_custom_title` (537-543).
  - `crates/terminal_view/src/terminal_view.rs`: the fields `needs_serialize` (172) and
    `custom_title`; `set_custom_title` (448-456) and `mark_needs_serialize` (458-461);
    `serialize` (1933-1964): nothing for a task terminal (1941-1943) or a clean view (1945-1947),
    else the folder and the title; `deserialize` (1970-2027): the saved folder when non-empty,
    else the workspace's default, then `create_terminal_shell(cwd)`; `cleanup` (1923-1931) calls
    `delete_unloaded_items`.
  - `crates/terminal/src/terminal.rs:3059-3097`: `working_directory` is the foreground process's
    folder (`None` for a remote terminal); `foreground_process_command_name` (3071) is what the
    rail matches to an agent.
  - `crates/marley_workbench/src/agents.rs`: `STARTUP_TIMEOUT` (45); `start_cli` (188-222): the
    center terminal, the startup handshake raced against the timeout, then
    `write_init_command_after_startup(marley_agent::launch_input(kind))`, which refuses to write
    once the terminal took other input.
  - `crates/marley_workbench/src/rail.rs`: `terminal_snapshot` (1666-1722) reads the foreground
    command; the rail notices an agent leaving through `foreground_command`.
  - `crates/db/src/db.rs:138,166`: the database file is `db.sqlite` in a `0-<scope>` folder under
    `paths::database_dir()`; `sqlite3` is installed for the scenario's damaged-id part.
  - Orca (MIT): `src/shared/agent-session-resume.ts` (`getAgentResumeArgv`, which gives
    `['claude', '--resume', id]` at line 268); `src/shared/agent-resume-launch-command.ts` (drops a
    `--resume`, `-r`, `--continue` or `-c` already in the base command);
    `src/renderer/src/components/terminal-pane/pty-connection/cold-restore-resume-startup.ts` (the
    resume typed into a new shell), named in report 05 §2.2.
  - rustal-harness: `docs/TERMINALS.md` ("`shutdown` alone stops the Rust runtime and leaves tmux
    terminals alive"), `docs/RESTORE.md` (a restore makes a fresh instance: "Previously live work
    has unknown present status; restore neither terminates that work nor asserts it completed"),
    `docs/REMOTE.md` (a lost connection makes the view stale; `Ctrl-b r` reconnects to the original
    pane and process), `docs/ROADMAP.md` M9 (the fleet contract over `rh mcp`) and M10 (a foreman
    that restarts a stalled agent "as a new incarnation").
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.** #519's `agent_events` receives each terminal's `SessionStart` and `SessionEnd`
  frames (`session_id`, `cwd`, `source`, `reason`); the resume bookkeeping beside it keeps, per
  terminal view, the session (id and first folder) by the spec's rules, and hands it to the view
  through
  a new `TerminalView::set_marley_agent_session`
  (a Marley hunk in `terminal_view.rs`, shaped like `set_custom_title`). `serialize` writes the two
  columns beside the folder; `deserialize` reads them, prefers the session's folder for the new
  shell, and sets the field on the new view so the next save carries it under the new item id.
  The workbench's `observe_new` for `TerminalView` sees each restored view; one with a session
  and no live agent yet is a restore. It checks the setting and the UUID, claims the id for this
  launch (a set of ids resumed so far), and runs the same handshake-then-type path as `start_cli`,
  factored into one function both call, with `claude --resume <id>`.
- **Dropping the session.** On `SessionEnd` with `prompt_input_exit` or `logout`, and when the rail
  sees the foreground leave Claude Code while Marley runs, the workbench clears the field (which
  marks the view for saving). `other` keeps it (D4).
- **The quit.** Zed saves each item as it changes and again when the workspace closes; a
  `SessionEnd` that Claude Code prints as its terminal dies at the quit reaches a Marley that no
  longer reads it, and reads `other` in any case. Test confirms the order with the fake, which
  prints `SessionEnd` (`other`) on SIGHUP.
- **File manifest.**
  - `crates/terminal_view/src/persistence.rs` (Zed crate): one migration adding the two columns;
    `save_marley_agent_session` and `get_marley_agent_session`. Touchpoint row.
  - `crates/terminal_view/src/terminal_view.rs` (Zed crate): the field, its setter and getter, the
    columns in `serialize` and `deserialize`, the session's folder preferred for the new shell.
    Touchpoint row (the file already has Marley rows; this one is added).
  - `crates/marley_workbench/src/agents.rs` (Marley crate): the shared launch path, the resume.
  - `crates/marley_workbench/src/marley_workbench.rs` (Marley crate): `resume_agents` on
    `MarleySettings`, and the per-terminal session bookkeeping from #519's events (or a new
    `sessions.rs` beside it, if the bookkeeping grows past a screen).
  - `crates/settings_content/src/marley.rs` (Zed crate path, Marley's file): `resume_agents`.
    Its touchpoint row updated.
  - `script/e2e/540-session-resume-after-restart.sh` and its fake (Test).
- **Ledger rows.** Three touchpoint rows (the two `terminal_view` files and the settings file's
  update), written before the first edit (§14). At Complete, a lesson on the quit's order if Test
  finds anything surprising.

### What an embedded harness changes
- **A restart is not a death there.** With rustal-harness embedded, a seat's terminal runs in the
  harness's own tmux server, and Marley shows it through a display-only Zed terminal fed by the
  capture stream (the three-prong plan's D10, slice C3). The harness's runtime and its tmux
  terminals outlive the client (`docs/TERMINALS.md`), so after a Marley relaunch the Claude Code
  process is still running and Marley reattaches. A resume there would start a second Claude Code
  on a live session, which D5 forbids.
- **What still needs a resume.** A reboot, a crashed tmux server, or a harness restore: a restore
  makes a fresh instance in which "previously live work has unknown present status" and old
  execution ids are not reused (`docs/RESTORE.md`). Only then does a conversation come back by
  `claude --resume <id>`, and only when the harness reports the old process gone. Losing contact is
  never evidence of exit (Orca's rule, report 04 §2.9), so a seat the harness cannot account for is
  shown as such, not resumed.
- **Where the id lives then.** The harness journal, per actor, rather than Zed's terminal row;
  Marley reads it through `rh mcp` (M9) and offers Resume on a seat the harness reports exited. M10's
  foreman already restarts a stalled agent "as a new incarnation" under a bounded policy; a resume
  by session id would be the natural form of that restart for Claude Code.
- **So:** this ticket serves Marley's own terminals. If the harness is embedded first, the slice
  becomes "Resume on a seat the harness reports exited", with the session id from the harness's
  journal and the same `claude --resume` line.

### E2E plan
Shared fixtures: a scratch repository with a `sub` folder; a HOME of the scenario's own for the
terminals (`terminal_env HOME`), whose `.bashrc` gives a plain prompt; the fake `claude` first on
the PATH, which reads its session id from a file the scenario writes, logs `argv` and `pwd` to
`$E2E_WORK/claude.log`, prints the frames, and on SIGHUP prints `SessionEnd` (`other`).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | Terminal A: `cd sub`, `marley: new agent` → Claude Code; the fake reports id `11111111-…`; terminal B: start the fake, then its exit trigger; `quit_marley`, `launch_marley` | `540-01-running` before the quit; `540-02-resumed` after; the log's second entry for A reads `claude --resume 11111111-…` |
| REQ-002 | The same relaunch | The log's `pwd` for the resume is `<repo>/sub`; the shot shows `resumed … in <repo>/sub` |
| REQ-003 | The same relaunch, terminal B | `540-03-plain-shell`: B at a plain prompt; no resume entry for B |
| REQ-004 | `quit_marley`, `launch_marley` again | `540-04-second-relaunch`: A resumed a second time; a third log entry |
| REQ-005 | Before a quit, B's fake reports A's id too; relaunch | The log: one `--resume` for the id; the other terminal at a plain prompt |
| REQ-006 | Between a quit and a launch, `sqlite3` sets A's saved id to `11111111; touch pwned` in the profile copy's `db/0-*/db.sqlite` | `540-05-damaged`: a plain prompt in A; no log entry; no `pwned` file in A's folder |
| REQ-007 | `marley.resume_agents: false` in the profile copy's settings; relaunch | No resume entry; plain prompts |

Not reachable by a scenario: a real Claude Code resuming a real session (that needs an account and
a model call). Test resumes one real session by hand once, on Chad's machine, and notes what
Claude Code printed.

### Risks
- **The quit's order.** If Zed saved a terminal after Claude Code's `SessionEnd` arrived and cleared
  the session, nothing would resume. D4 keeps `other`, and the fake's SIGHUP frame tests it; if
  Claude Code sends `prompt_input_exit` on a hangup, the rail's foreground check has to decide
  instead.
- **Claude Code refusing the folder.** `claude --resume <id>` looks up
  `~/.claude/projects/<the folder, encoded>/`; a session started in a folder that no longer exists
  fails there. The shell then starts in the workspace's default folder (Zed's fallback), and Claude
  Code's own error shows in the terminal.
- **The trust prompt.** Claude Code asks to trust a folder it has not seen. A resumed session's
  folder has been trusted before, so no prompt is expected; Test checks it once.
- **Typing into a shell.** Zed's `write_init_command_after_startup` writes only if the terminal has
  taken no other input, so a user who types first keeps a plain shell. The command is the program,
  one flag and a checked UUID.
- **One session in two terminals.** Chad can run `claude --resume` of one session in two
  terminals himself; both rows then hold the id, and D5's one-resume rule keeps a relaunch from
  starting two agents on it.
