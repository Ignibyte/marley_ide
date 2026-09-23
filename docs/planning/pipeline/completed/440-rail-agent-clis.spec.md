---
pipeline_id: 0aa03374-c7ef-4556-a807-1c1cd76ab571
ticket: docs/planning/tickets/open/TICKET-440-rail-agent-clis.md
status: Phase 4 — Complete PASS
title: Agent CLIs in rail terminals (New Agent, recognition, status)
type: feature
slice: workbench shell W4
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/438-marley-layout-and-rail.spec.md, docs/planning/pipeline/completed/439-rail-zed-threads.spec.md]
---

## Title
Run agent CLIs the Warp way: New Agent opens a center terminal at the project root and starts
the chosen CLI once the shell is ready. Any terminal running an agent CLI, however it was
started, shows as an agent row with the CLI's own title and a working, waiting or exited
status.

## Scope
### In
- The project `+` menu lists the agent CLIs `marley_agent` knows (Claude Code `claude`, Codex
  `codex`, Gemini CLI `gemini`, OpenCode `opencode`) that the rail's search path holds, found
  with `which::which_in` over `PATH`. They appear as entries under an "Agent CLIs" header,
  after New Terminal and New Agent Thread, so starting one is a single click; with none found,
  the section is absent.
- Launch: a center terminal in the project's working directory, where New Terminal starts one
  (`terminal_view::default_working_directory`). The CLI's program name and a carriage return
  are then written once the shell's startup handshake completes, or after its timeout
  (`Terminal::start_init_command_startup_handshake`, `write_init_command_after_startup`,
  `crates/terminal/src/terminal.rs:2142-2225`), as Zed's Terminal Threads start their init
  command. A terminal with no PTY gets it at once.
- Recognition: `Terminal::foreground_process_command_name` (argv[0], `terminal.rs:2890`)
  classified by `marley_agent::agent_kind_of`, which grows the Gemini and OpenCode kinds.
- An agent row: the agent's icon in place of the terminal's, labelled with the title the CLI
  sets over OSC (`breadcrumb_text`), else the agent's display name. The second line reads the
  agent's name and status, for example "Claude Code · waiting".
- Status: working while output arrives (`Wakeup`), waiting once output has been quiet for
  `marley_agent::WAITING_AFTER` (2 s) or the bell rang. A per-terminal quiet timer, re-armed on
  each output, redraws the row when the interval passes, on gpui's executor clock. When the
  agent leaves the foreground, the row is a plain terminal row again.
- `marley_agent` stays pure and at the full bar. Its tick-based status goes (the fork has no
  16 ms pump, and no crate uses it) for `agent_status(quiet_for, bell)` over a duration.

### Out (explicitly deferred)
- Configurable agent profiles (a list under the `marley` settings block).
- An attention dot for a CLI agent that goes quiet while unseen: the bell's dot (W2) is the
  signal for now; the rest is W6 polish.
- Sending prompts or answers to an agent from the rail (prong 2, `session.send`).
- Harness seats and Rusty agent sessions as rows (prong 2, C1 and C2).

## Reference (§20)
- **Warp:** a coding agent runs in a terminal session that the session list shows by its live
  title (observed, `docs/planning/design-notes/session-tabs-vs-sidebar.md` G1; Warp's agent
  model at the behavior level, `docs/warp_architecture/subsystems/04-agent-ai-mcp.md`).
- **Upstream Zed:** Terminal Threads start `agent.terminal_init_command` in a fresh shell
  after the startup handshake (`crates/agent_ui/src/agent_panel.rs:2066-2220`); the rail
  keeps that launch behavior for center terminals.

### Prior art
- **Behavior maps:** the Warp agent subsystem map above; the gpui-era agent cockpit, whose
  pure model this ticket reuses (`crates/marley_agent`, Marley's own code).
- **Published material:** `docs/src/ai/terminal-threads.md` (Claude Code's
  `preferredNotifChannel: terminal_bell`, Amp's `AMP_FORCE_BEL`, OpenCode's title updates).
- **Code we already ship:** the startup-handshake pair and `foreground_process_command_name`
  in `crates/terminal`; `breadcrumb_text` and `terminal::Event::{BreadcrumbsChanged, Bell,
  Wakeup}`; `marley_agent::{agent_kind_of, launch_command, agent_status_from}`.
- **Re-swept at promotion (2026-09-22).** The Agent Panel's terminal threads show the full
  launch sequence (`agent_panel.rs:2170-2212`): write at once to a non-PTY terminal; else
  start the handshake and race it against a timeout; then `write_init_command_after_startup`,
  which refuses once the user has typed or the child has exited. `which = "8"` is already a
  workspace dependency, used by `askpass`, `auto_update` and `node_runtime`, and its
  `which_in` takes an explicit search path. `TerminalPanel::add_center_terminal` returns the
  new `Terminal`'s weak handle. `Terminal::take_pty_write_log` (test-support) shows what a
  test terminal was sent. `ui::IconName` carries `AiClaude`, `AiOpenAi`, `AiGemini` and
  `AiOpenCode`. `BackgroundExecutor::now` is the clock the tests advance. No crate but its
  own uses `marley_agent`.

## UI proof
UI-AFFECTING.
- **Driven tests:** New Agent lists only CLIs present in a test `PATH`; a terminal whose
  foreground argv is `claude` turns into an agent row; a breadcrumb title becomes the label;
  status goes working, then waiting after the quiet interval on the executor clock, then
  exited. The `marley_agent` additions get unit tests.
- **Live drive:** New Agent > Claude Code in a real project, screenshot the agent row with
  Claude's title; let it finish a prompt, screenshot the waiting status; start `codex` by hand
  in another terminal and screenshot its row.

## Locked-In Decisions
- D1 — Agents launch in a shell, not as the terminal's program, so the terminal survives the
  CLI and restores as a shell.
- D2 — Recognition reads argv, never the process name (Claude Code's binary here is named
  after its version, `~/.local/share/claude/versions/2.1.280`).
- D3 — The CLI list and classification live in `marley_agent` (pure); the rail only renders.
- D4 — The waiting threshold is time-based on gpui's executor clock, so tests drive it with
  `advance_clock`.
- D5 — The launch writes only a program name from `marley_agent`'s fixed list, never text a
  user or a file supplied, so nothing reaches the shell but a known command.
- D6 — Recognition and the PATH lookup go through seams the tests replace, as the terminal
  factory does (#438). A display-only terminal has no foreground process, and a test's `PATH`
  must not be the machine's.
- D7 — An agent's quiet spell reads as waiting but raises no dot; a bell still does (W2).
- D8 (revised at promotion) — The CLIs are entries in the `+` menu under a header, not a
  submenu: one click to start one, which is what the original complaint asked for ("I cant
  figure out even how to start a new agent"), and every entry carries a selector a test can
  click.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The project's `+` menu shall list, under an Agent CLIs header, exactly the known agent CLIs found on the search path, and no such section when none is found | driven test with a temp search path |
| REQ-002 | WHEN an agent CLI is chosen from the `+` menu, a center terminal shall open in the project's working directory and the CLI's command shall be written once the shell's startup handshake completes | driven test on the handshake seam |
| REQ-003 | WHEN a terminal's foreground argv[0] names a known agent CLI, its row shall show that agent's glyph and name | driven test + `agent_kind_of` unit tests |
| REQ-004 | WHILE an agent terminal has a breadcrumb title, its row label shall be that title | driven test |
| REQ-005 | WHILE output keeps arriving, an agent row shall read working; WHEN output stops for the quiet interval or the bell rings, it shall read waiting | driven test with `advance_clock` |
| REQ-006 | WHEN a terminal's foreground program stops being a known agent CLI, its row shall go back to a plain terminal row | driven test |
| REQ-007 | WHEN an agent is started from the `+` menu, the only text written to the shell shall be that agent's program name and a carriage return | driven test on the terminal's write log |
| REQ-008 | `marley_agent`'s new kinds and timing function shall be at 100% lines, and the diff gate green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — the design: the `marley_agent` additions, the PATH lookup, the handshake seam
  for tests, the status inputs the snapshot carries.
- **P2 Code** — `marley_agent` first (the kinds, names, icons' identity, `agent_status`,
  `WAITING_AFTER`), then `marley_rail`'s agent rows, then the menu, launch, recognition and
  timers in `marley_workbench`. The review of the diff checks for false agent detection,
  command injection through the launch path (a fixed program name only), and timer and clock
  handling.
- **P3 Test** — tests, `script/gates.sh --diff`, the live drive.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_agent.md`, ledger, close,
  archive, commit.
