---
pipeline_id: 4d2062df-6c4b-4fd3-9760-7d528e1d70fa
ticket: docs/planning/tickets/closed/TICKET-467-shell-title-hides-marleys-arguments.md
status: Phase 4 — Complete PASS
title: A Marley shell's title leaves out Marley's arguments
type: bug
slice: prong 1 T0c
references: [docs/planning/pipeline/completed/463-bash-shell-integration.spec.md, docs/planning/knowledge/architecture-decisions.md]
---

## Title
#463 starts each local bash with `--rcfile <data dir>/shell_integration/marley.bash`, and Zed's
terminal title lists the foreground process's arguments, so every shell's tab and rail row
shows Marley's rcfile path. The title now leaves out the arguments Marley's integration added.

## Scope
### In
- **The arguments shown** (`marley_terminal::shell_integration`). A pure function takes a
  process's argv and the integration directory and returns the arguments to show: all of them,
  less the one run of arguments `for_program` gives that program at that directory.
- **The title** (`terminal::Terminal::title`). The arguments after the process name come from
  that function, with the directory the builder installs the scripts in.

### Out (explicitly deferred)
- The process info itself: `argv` stays as the operating system reports it, so the foreground
  command name and anything else reading it see the real process.
- zsh (#465) adds no arguments (`ZDOTDIR`); fish (#466) would go through the same function.

## Reference (§20)
- **Upstream Zed:** `Terminal::title` (`crates/terminal/src/terminal.rs:3060-3105`) names a
  shell's tab `<cwd name> — <process name> <arguments>`, read from the foreground process.
  Marley keeps that title, live cwd and running command included, and leaves out only the
  arguments it added itself.
- **Warp:** N/A. The title is Zed's; nothing in Warp's shell integration is copied or needed.

### Prior art
- **Behavior maps:** none on this seam.
- **Published material:** bash has no environment variable that names an interactive shell's
  startup file (`BASH_ENV` is for non-interactive shells, `ENV` for POSIX mode), so the
  argument cannot be avoided for bash; zsh's `ZDOTDIR` can (#465).
- **Code we already ship.**
  - `task::Shell::WithArguments::title_override` gives a terminal a fixed title. Rejected: it
    would freeze the title and lose the live cwd and running command.
  - `foreground_process_command_from_argv` (`terminal.rs:3519`) reads only `argv[0]`, so agent
    detection is unaffected by the arguments.
  - `marley_terminal::shell_integration::for_program` already computes the exact arguments
    added; reusing it keeps one definition.

## UI proof
UI-AFFECTING: the title shows in each shell's tab and in the rail's terminal rows.
- **Driven test:** a real bash spawned through `TerminalBuilder` with Marley's integration;
  after the user's `.bashrc` runs, the loaded process info names `bash`, and `title(false)`
  holds no `--rcfile`.
- **Live drive:** the debug `marley` on a copy of Chad's profile, on hidden workspace 9, shot by
  its toplevel (`grim -T <stableId>`), with no input sent; the tab and the rail row read
  `marley_ide — bash`.

## Locked-In Decisions
- D1 — The arguments hidden are exactly the ones `for_program` returns for the process's
  `argv[0]` at the directory the builder installs into, matched as one contiguous run. No
  field is added to `Terminal`; the title recomputes them.
- D2 — Only the title changes. `ProcessInfo::argv` keeps the real arguments.
- D3 — Zed's own quirks in the title stay, such as the space after a shell's name when it has
  no arguments.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the foreground process is a shell Marley started with its integration, the terminal's title shall name it without the arguments the integration added | unit tests; PTY test |
| REQ-002 | WHEN a process's arguments do not hold the integration's arguments as one run, the title shall show every argument unchanged | unit tests |
| REQ-003 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — `shown_arguments` in `marley_terminal`, the title hunk and the shared directory
  helper in `terminal.rs`; fmt and clippy clean; a review of the diff.
- **P3 Test** — the unit tests and the PTY test run, with a negative check; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
