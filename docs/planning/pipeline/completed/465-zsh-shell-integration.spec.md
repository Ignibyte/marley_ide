---
pipeline_id: d9b47e4a-578a-4354-898f-82a75d68316b
ticket: docs/planning/tickets/closed/TICKET-465-zsh-shell-integration.md
status: Phase 4 — Complete PASS
title: Shell integration for zsh
type: feature
slice: prong 1 T0c
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/463-bash-shell-integration.spec.md]
---

## Title
bash reports its prompts and commands to Marley since #463; zsh does not yet. A local
interactive zsh that Zed spawns now loads Marley's integration through `ZDOTDIR`, with the
user's own startup files still read, so every command typed in zsh becomes a block.

## Scope
### In
- **The script** (`crates/marley_terminal/shell_integration/marley.zsh`, installed as
  `<data dir>/shell_integration/zsh/.zshenv`). It puts the user's `ZDOTDIR` back, or unsets it,
  and sources the user's `.zshenv`, so zsh reads the rest of the user's files from where it
  would have. In an interactive shell it installs the hooks at the first prompt: `precmd`
  first in `precmd_functions`, `preexec` after the user's, and the `init` and `bootstrapped`
  frames. Values are escaped as the bash script escapes them.
- **The start** (`marley_terminal::shell_integration`). `for_program` gives zsh no arguments and
  the environment `ZDOTDIR=<dir>/zsh`, `MARLEY_SHELL_INTEGRATION=1`, and
  `MARLEY_ZSH_ZDOTDIR=<the user's ZDOTDIR>` when the shell would have inherited one.
  `install_in` writes the zsh file beside the bash one.
- **The spawn** (`terminal::marley_shell_integration`). The user's `ZDOTDIR` is read from the
  terminal's environment, else Marley's own; a shell whose integration adds no arguments stays
  the `System` or `Program` shell it was.

### Out (explicitly deferred)
- fish (#466).
- OSC 133 alongside Marley's frames (the plan's D5 calls it cheap; not needed by any block yet).
- A user who assigns `precmd_functions` outright in `.zshrc` drops Marley's installer, and the
  shell starts without blocks.
- A user with no zsh startup files at all no longer sees zsh's new-user menu
  (`zsh-newuser-install`) in Marley's terminals: zsh looks for startup files in `ZDOTDIR` before
  reading any, and Marley's holds a `.zshenv`. Kept: the menu has no place in an editor's
  terminal, and it still runs in any other terminal.

## Reference (§20)
- **Warp:** the plan's D5 (`docs/marley/three-prong-plan.md`) names `ZDOTDIR` as the way Warp
  and Kitty load zsh hooks, with the user's own files still sourced. Observed behavior only; no
  code from either.
- **Upstream Zed:** the spawn path (`TerminalBuilder::new`) and #463's hunk in it; Zed itself
  has no shell integration.

### Prior art
- **Behavior maps:** none beyond D5.
- **Published material:** the zsh manual, "Startup/Shutdown Files": zsh reads `.zshenv`,
  `.zprofile`, `.zshrc` and `.zlogin` from `$ZDOTDIR`, else `$HOME`, taking the value of
  `ZDOTDIR` at the moment it reaches each file, so a `.zshenv` that restores it hands the rest to
  the user's location. Hook arrays: `precmd_functions` and `preexec_functions` (`add-zsh-hook`
  writes the same arrays); `preexec` gets the line as typed in `$1`.
- **Code we already ship.**
  - #463's `shell_integration.rs` (the embedded script, `install_in`, `for_program`) and the
    injection hunk in `terminal.rs`; zsh takes the same seams.
  - The gpui era's zsh rc (`/srv/stacks/marley/crates/marley_app/src/shell_integration.rs`,
    Marley's own): `add-zsh-hook preexec`/`precmd` and `__marley_quote`. It did not source the
    user's files and wrote into a private `ZDOTDIR`; its expansions were unquoted, which a
    script's top level reads differently (the notes' Phase 1).
  - `shown_arguments` (#467) keeps working: zsh gets no arguments.

## UI proof
UI-AFFECTING: blocks are not drawn yet (T1), but the terminal the user types into starts
differently and its title must stay as it was.
- **Driven tests** (PTY, `crates/terminal`): a real zsh through `TerminalBuilder` with a scratch
  `HOME`: its `.zshenv` and `.zshrc` run; `echo hi`, `false` and `echo 'a;b'` become finished
  blocks with their exit codes and output; a user's `ZDOTDIR` is where zsh reads `.zshrc` from
  and what `$ZDOTDIR` reads afterwards; the title names `zsh` alone.
- **Live drive:** the debug `marley` with `terminal.shell` set to zsh in the profile copy, on
  hidden workspace 9, captured by toplevel with no input: the shell starts at its prompt and the
  tab reads `<dir> — zsh`; the shell's process started with Marley's `ZDOTDIR` and the marker.
  The restore happens inside the shell, which the PTY tests read.

## Locked-In Decisions
- D1 — One file, `.zshenv`, in Marley's `ZDOTDIR`: it restores the user's `ZDOTDIR` and hands
  the other startup files back to zsh, rather than a Marley copy of each file that sources the
  user's.
- D2 — The hooks install at the first prompt, so the user's files have run: `precmd` goes
  first, so its frame marks where the command's output ends before another hook prints, and
  `preexec` goes last. zsh gives every `precmd` hook the command's `$?`, whatever the order.
- D3 — zsh needs no argument, so its `Shell` is not rewritten; only the environment grows.
- D4 — The zsh PTY tests need zsh and fail without it, as the bash ones need bash; no silent
  skip.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Zed spawns a local interactive zsh as the `System` or `Program` shell, the shell shall start with Marley's `ZDOTDIR` and its marker, and keep the user's `ZDOTDIR` in `MARLEY_ZSH_ZDOTDIR` when it had one | unit tests |
| REQ-002 | WHEN the zsh starts, it shall read the user's own `.zshenv` and `.zshrc` from the user's `ZDOTDIR`, else `$HOME`, and leave `ZDOTDIR` as the user had it | PTY tests |
| REQ-003 | WHEN a command is typed in that zsh, the terminal shall hold a finished block with the command, its exit code and its output | PTY test |
| REQ-004 | The zsh terminal's title shall name the shell without anything Marley added | PTY test |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the script, `shell_integration.rs`, the spawn hunk; fmt and clippy clean; a
  review of the diff.
- **P3 Test** — the unit and PTY tests, with negative checks; the live drive;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
