---
pipeline_id: e2b6d2cc-de26-49de-af8b-b5ae620ca43e
ticket: docs/planning/tickets/closed/TICKET-463-shell-integration-scripts.md
status: Phase 4 — Complete PASS
title: Shell integration for bash, and its injection at spawn
type: feature
slice: prong 1, T0c (bash; zsh is #465, fish #466)
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/464-zeds-terminal-keeps-anchored-blocks.spec.md]
---

## Title
Blocks exist once a shell sends Marley's hooks (#462, #464), and no shell sends them yet. Zed's
spawn of a local interactive bash now injects Marley's integration, so every command typed into
a Marley terminal becomes a block.

## Scope
### In
- **The script** (`crates/marley_terminal/shell_integration/marley.bash`, embedded with
  `include_str!`):
  - it sources the user's `~/.bashrc`, which `--rcfile` would otherwise skip;
  - it installs the hooks once: `__marley_precmd` first in `PROMPT_COMMAND` (array or string),
    keeping `$?`, and `__marley_preexec` appended to `PS0`;
  - it sends `init` and `bootstrapped`;
  - every frame is AnsiCQuoted (`q`), its values escaped as the decoder expects.
- **The installer** (`marley_terminal::shell_integration`): `install_in(dir)` writes the scripts
  when their content differs, `for_program(program, dir)` returns the arguments and environment
  for a program it knows, and `bash` is the only one known yet.
- **The injection** (`crates/terminal`, one hunk in `TerminalBuilder::new`): for a local
  interactive shell (no task, not remote, with a PTY) given as `Shell::System` or
  `Shell::Program`, the program is resolved (`$SHELL` for `System`). If Marley knows it, the
  shell becomes `WithArguments` with Marley's arguments, the environment gains
  `MARLEY_SHELL_INTEGRATION`, and the scripts live in `<data dir>/shell_integration`.

### Out (explicitly deferred)
- zsh (#465) and fish (#466).
- A shell given `WithArguments` by the user: the user chose its arguments.
- macOS, where alacritty makes the shell a login shell, which `--rcfile` does not reach.
- Remote terminals and tasks.
- The exact command text for a line that history drops (`HISTCONTROL=ignorespace`, history
  off): its block then carries the previous line's text, or none.

## Reference (§20)
N/A — Marley-specific: Marley's own hook frames, emitted by Marley's own scripts (the gpui era's
zsh script is the precedent, and is Marley's code). Warp is not read; the behavior map records
only that its blocks come from shell hooks. Bash's own documented facilities are used: `--rcfile`,
`PROMPT_COMMAND`, `PS0`.

### Prior art
- **Behavior maps:** `docs/warp_architecture/subsystems/00-overview.md:44`.
- **Published material:** the bash manual. `--rcfile` replaces `~/.bashrc` for interactive
  non-login shells. `PROMPT_COMMAND` (an array since 5.1) runs before each primary prompt. `PS0`
  (since 4.4) is expanded and printed after a command is read and before it runs.
- **Code we already ship.**
  - The gpui era's zsh script and its `__marley_quote` escaping
    (`/srv/stacks/marley/crates/marley_app/src/shell_integration.rs`).
  - `marley_terminal::dcs`'s AnsiCQuoted decoder, R24's unescaped-separator split.
  - Zed's `util::shell::get_system_shell()`.
  - `TerminalBuilder::new`'s future, which runs on the background executor.

## UI proof
UI-AFFECTING.
- **Driven tests:** a real interactive bash from `TerminalBuilder` with `HOME` set to a scratch
  directory: typed `echo hi` leaves a Finished block, exit 0, output "hi"; `false` leaves exit
  1; a command containing `;` keeps its text.
- **Live drive:** Marley on this checkout, `echo hi` typed into a fresh terminal, with the log
  showing the hooks applied (nothing draws blocks until T1). It needs keys, so it runs only while
  Chad is away; otherwise the Test phase records why.

## Locked-In Decisions
- D1 — The scripts live in `marley_terminal`, a Marley-owned path, rather than Zed's `assets/`,
  so they need no ledger rows and carry Marley's license.
- D2 — Injection is for `Shell::System` and `Shell::Program` only. A user's own arguments stand.
- D3 — bash's preexec is `PS0`, printed between reading a command and running it, so the frame
  lands in stream order before the command's output. The command text comes from history.
- D4 — The scripts are written to `<data dir>/shell_integration`, only when their content
  differs, from the builder's background future.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a local interactive bash starts in a Marley terminal, it shall send `init`, and each typed command shall leave a block with its text, exit code and output | PTY tests |
| REQ-002 | WHERE the user's `~/.bashrc` exists, bash shall still source it | PTY test with a scratch `HOME` whose `.bashrc` sets a marker |
| REQ-003 | WHEN the shell is a task, remote, or given explicit arguments, the terminal shall start it unchanged | unit tests of the decision; Zed's terminal suite |
| REQ-004 | The installer shall write each script once and rewrite it only when its content changed | unit tests (`install_in`) |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the script, the installer, the injection hunk (the ledger row first); fmt and
  clippy.
- **P3 Test** — REQ-001 to REQ-005, with negative checks, and the live drive.
- **P4 Complete** — CHANGELOG and docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
