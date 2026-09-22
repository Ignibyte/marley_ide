---
pipeline_id: 0aa03374-c7ef-4556-a807-1c1cd76ab571
ticket: docs/planning/tickets/open/TICKET-440-rail-agent-clis.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active for Phase 2 Design
title: Agent CLIs in rail terminals (New Agent, recognition, status)
type: feature
slice: workbench shell W4
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/queued/438-marley-layout-and-rail.spec.md]
---

## Title
Run agent CLIs the Warp way: New Agent opens a center terminal at the project root and starts
the chosen CLI once the shell is ready. Any terminal running an agent CLI, however it was
started, shows as an agent row with the CLI's own title and a working, waiting or exited
status.

## Scope
### In
- The project `+` gains New Agent: the agent CLIs found on `PATH`, from a list `marley_agent`
  owns (Claude Code `claude`, Codex `codex`, Gemini `gemini`, OpenCode `opencode`).
- Launch: an interactive shell at the project root, then the CLI written after the shell's
  startup handshake (`Terminal::start_init_command_startup_handshake`,
  `write_init_command_after_startup`, `crates/terminal/src/terminal.rs:2142-2225`).
- Recognition: `Terminal::foreground_process_command_name` (argv[0], `terminal.rs:2890`)
  classified by `marley_agent::agent_kind_of`, which grows the Gemini and OpenCode kinds.
- Row label: the title the CLI sets over OSC (`breadcrumb_text`, `BreadcrumbsChanged`),
  falling back to the agent's display name; the agent's glyph replaces the prompt glyph.
- Status: `marley_agent::agent_status_from` over live facts: working while output arrives
  (`Wakeup`), waiting after a quiet spell or a bell, exited when the shell is back in the
  foreground.
- `marley_agent` stays pure and at the full bar; the timing (quiet threshold as a duration,
  not pump ticks) moves to a pure function it owns.

### Out (explicitly deferred)
- Configurable agent profiles (a list under the `marley` settings block).
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
  Wakeup}`; `marley_agent::{agent_kind_of, launch_command, agent_status_from}`; `which`-style
  lookup through `util` (Zed resolves binaries on `PATH` in several places; design names the
  helper).

## UI proof
UI-AFFECTING.
- **Driven tests:** New Agent lists only CLIs present in a test `PATH`; a terminal whose
  foreground argv is `claude` turns into an agent row; a breadcrumb title becomes the label;
  status goes working, then waiting after the quiet interval on the executor clock, then
  exited. The `marley_agent` additions get unit tests at MSI 100.
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

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The New Agent submenu shall list exactly the known agent CLIs found on `PATH` | driven test with a temp `PATH` |
| REQ-002 | WHEN New Agent > a CLI is chosen, a center terminal shall open at the project root and the CLI's command shall be written once the shell's startup handshake completes | driven test on the handshake seam |
| REQ-003 | WHEN a terminal's foreground argv[0] names a known agent CLI, its row shall show that agent's glyph and name | driven test + `agent_kind_of` unit tests |
| REQ-004 | WHILE an agent terminal has a breadcrumb title, its row label shall be that title | driven test |
| REQ-005 | WHILE output keeps arriving, an agent row shall read working; WHEN output stops for the quiet interval or the bell rings, it shall read waiting | driven test with `advance_clock` |
| REQ-006 | WHEN the shell returns to the foreground, the agent row shall read exited and go back to a plain terminal row | driven test |
| REQ-007 | `marley_agent`'s new kinds and timing function shall be at 100% lines and MSI 100 | `script/gates.sh --diff` |

## Phase Plan
- **P2 Design** — the `marley_agent` additions, the PATH lookup, the handshake seam for tests,
  the status inputs the snapshot carries.
- **P3 Implement** — `marley_agent`, the menu, recognition, status.
- **P3.5 Inspect** — critics: false agent detection, command injection through the launch
  path (fixed argv only), clock handling in tests.
- **P4 Validate** — tests, `script/gates.sh --diff`, the live drive.
- **P5 Complete** — CHANGELOG, `docs/marley_architecture/marley_agent.md`, ledger, close,
  archive.
