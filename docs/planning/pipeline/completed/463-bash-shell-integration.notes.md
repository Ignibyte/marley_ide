# Shell integration for bash, and its injection at spawn — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-463-shell-integration-scripts.md
- **Pipeline spec:** 463-bash-shell-integration.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad: "ok lets go for it").
- **Split.** #463 as first written covered three shells. The dev box has bash 5.3.15 (Chad's
  `$SHELL`) and zsh 5.9.2, and no fish. #463 is now the injection and bash, #465 is zsh
  (queued), and #466 is fish (Deliberate, until a machine with fish can run its tests).
- **Recall (§18.3).**
  - The gpui era's zsh script: `init` and `bootstrapped` at start, `preexec` with the command,
    `precmd` with exit and `pwd`, and `__marley_quote` escaping `\` first, then `;`, newline,
    tab, CR and ESC.
  - #461's Explore sweep: the injection point after `insert_zed_terminal_env`, gated like
    Zed's activation scripts. The traps: alacritty picks `$SHELL` for `Shell::System`, macOS
    makes it a login shell, `template.env` is reused by clones, and `ShellKind` does not tell
    bash from zsh.
- **Discovery.** `TerminalBuilder::new`'s body is a future run by `cx.background_spawn`
  (`terminal.rs:1407`), so the scripts can be written there without blocking the UI.

### Design
- **`marley_terminal::shell_integration`** (a new public module):
  - `BASH_INTEGRATION`, the script (`include_str!`);
  - `install_in(dir) -> io::Result<()>`, which writes `marley.bash` when its content differs;
  - `for_program(program, dir) -> Option<ShellIntegration { args, env }>`, where `bash` gives
    `["--rcfile", <dir>/marley.bash]` and `MARLEY_SHELL_INTEGRATION=1`, and any other program
    gives `None`;
  - `is_bash(program)`, which compares the file stem.
- **Zed hunk** (`crates/terminal/src/terminal.rs`, in the builder's future after
  `insert_zed_terminal_env`), gated by `task.is_none() && !is_remote_terminal && !no_pty`:
  - map `Shell::System` to `get_system_shell()` and `Shell::Program(p)` to `p`;
  - call `install_in(paths::data_dir().join("shell_integration"))`, logging a failure and
    starting the shell unchanged;
  - if `for_program` knows the program, rewrite the shell to `WithArguments` and extend `env`.
- **The script:**

  ```bash
  [ -r "$HOME/.bashrc" ] && . "$HOME/.bashrc"
  if [ -z "${__MARLEY_HOOKS-}" ]; then __MARLEY_HOOKS=1
    __marley_quote() { __MARLEY_REPLY=$1; …escapes… }
    __marley_precmd() { local status=$?; __marley_quote "$PWD"; printf '\033Pqprecmd;exit=%d;pwd=%s\033\\' "$status" "$__MARLEY_REPLY"; return $status; }
    __marley_preexec() { local line; line=$(HISTTIMEFORMAT= builtin fc -ln -0); …trim…; __marley_quote "$line"; printf '\033Pqpreexec;command=%s\033\\' "$__MARLEY_REPLY"; }
    PROMPT_COMMAND: prepend __marley_precmd (array or string)
    PS0="${PS0-}"'$(__marley_preexec)'
    printf '\033Pqinit;id=%d\033\\' "$$"; printf '\033Pqbootstrapped;subshell=0\033\\'
  fi
  ```
- **File manifest.**
  - Marley: `crates/marley_terminal/shell_integration/marley.bash`,
    `crates/marley_terminal/src/{shell_integration.rs, marley_terminal.rs}`, and
    `crates/marley_terminal/Cargo.toml` if a dependency is needed.
  - Zed: `crates/terminal/src/terminal.rs` (the hunk and tests), with its row updated first,
    and `crates/terminal/Cargo.toml` if `paths` is not already a dependency.

### Test plan
| REQ | Test |
|---|---|
| 001 | PTY: `TerminalBuilder` with `TerminalMode::interactive()`, `Shell::Program("bash")` and `HOME` a scratch dir; after `init` arrives, `echo hi\r` gives a Finished block (`echo hi`, exit 0, output "hi"), then `false\r` gives exit 1, and `echo 'a;b'\r` keeps its command |
| 002 | PTY: a scratch `.bashrc` that echoes a marker; the marker appears in the terminal |
| 003 | unit: `for_program` for bash, `/usr/bin/bash`, `zsh`, `sh` and fish; the Zed gate is covered by Zed's terminal suite (tasks and explicit arguments run unchanged) |
| 004 | unit: `install_in` writes, leaves an identical file untouched (mtime), and rewrites a changed one |
| 005 | `script/gates.sh --diff` |

Negative checks: `PS0` not set (no block opens); `$?` taken after the quote call (exit 0 for
`false`); `~/.bashrc` not sourced (REQ-002 fails).

### Risks
- **The user's `PROMPT_COMMAND` machinery** (bash-preexec, starship, direnv) runs after the
  user's rc; prepending keeps `$?` for them too.
- **History.** A line history drops gets the previous line's text, as scoped out.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] the script · [x] the installer · [x] the ledger rows first · [x] the Zed
  hunk and helper · [x] fmt and clippy.
- **Built.**
  - `crates/marley_terminal/shell_integration/marley.bash`, as designed. Before any Rust, it
    was checked under a real interactive bash 5.3 on a pseudo-terminal (`script -qfc "bash
    --rcfile … -i"`) with a scratch `HOME`. The run showed the user's `.bashrc` marker, `init`
    and `bootstrapped`, `preexec;command=echo hi` before the output, `precmd;exit=1` for
    `false`, and `command=echo "a\;b"` escaped.
  - `marley_terminal::shell_integration`: `BASH_INTEGRATION`, `BASH_FILE`, `MARKER_VARIABLE`,
    `ShellIntegration`, `install_in(dir)` and `for_program(program, dir)`, with 2 unit tests.
  - `crates/terminal`:
    - `paths`, and `tempfile` for tests;
    - the hunk after `insert_zed_terminal_env`, gated by no task, not remote and a PTY, and
      `cfg(unix)`;
    - `marley_shell_integration(shell, env)`, which resolves `System` to `$SHELL`, takes a
      `Program` as it is, and leaves `WithArguments` alone. It asks `for_program` first and
      writes the scripts only for a known shell, logging a failure and starting the shell
      unchanged.
- **Deviation:** the helper checks `for_program` before `install_in`, not after, so a zsh user's
  terminal writes nothing.

## Phase 3 — Test (2026-09-23)
- **REQ-001, REQ-002:** `marley_bash_reports_each_typed_command_as_a_block`. It runs a real
  interactive `bash` from `TerminalBuilder`, with `HOME` a scratch directory whose `.bashrc`
  echoes a marker:
  - the marker appears;
  - typed `echo hi` gives a Finished block with exit 0 and output "hi";
  - `false` gives exit 1;
  - `echo 'a;b'` keeps its command and its output "a;b".
- **REQ-003:** `marley_shell_integration_rewrites_bash_and_leaves_other_shells` covers explicit
  arguments and zsh unchanged, with nothing added to the environment, and bash rewritten with
  `--rcfile` and the marker. Zed's terminal suite (tasks, the interactive `System` shell)
  passes: `cargo nextest run -p terminal -p marley_terminal`, 267 passed.
- **REQ-004:** `install_writes_the_script_leaves_an_identical_one_and_rewrites_a_changed_one`.
  A read-only identical script still installs, so no write was attempted.
- **Negative checks** (restored by sha256 checksum):
  - N1, no `PS0` hook: the bash test fails, since no block opens.
  - N2, `$?` read after the quote call: the bash test fails on `false`'s exit.
  - N3, `~/.bashrc` not sourced: the bash test fails, since the marker never appears.
  - N4, explicit arguments rewritten: the decision test fails.
- **Live drive: not run.** Chad was at the desk (`IdleHint=no`, workspace 1 active). A
  fresh-profile window would have landed on the workspace in use, and nothing is drawn until
  T1. The PTY test runs the same `TerminalBuilder` path with a real bash. The window check
  (the seeded terminal's `bash --rcfile …/marley.bash` in `ps`) is owed to the next drive.
- **After the first green: shellcheck.** gate:11 covered only the hooks and the gate scripts,
  so the script Marley ships to users was never linted. By hand it had two warnings and five
  infos:
  - `HISTTIMEFORMAT= builtin fc` (SC1007);
  - `PROMPT_COMMAND` as an array in one branch and a string in the other (SC2178, SC2128);
  - frames ending in `\033\\'`, read as an escaped quote (SC1003);
  - the single-quoted `$(…)` in `PS0` (SC2016).

  All were fixed at the source, with no suppression: `HISTTIMEFORMAT=''`; both branches
  array-shaped, since `PROMPT_COMMAND[0]` is the string in older bash; `\134` for the closing
  backslash; and `\$(…)` in double quotes. gate:11 now lints
  `crates/marley_terminal/shell_integration/marley.bash`, and CONSTITUTION §0's table says so.
  The same `script -qfc` run showed the same frames. The four integration tests pass again, and
  the gate runs again on the changed tree.
- **Gate:** `script/gates.sh --diff`, scope the Marley crates plus `terminal` and
  `marley_terminal`: GATE GREEN [diff], 20 of 20, twice, the second time on the
  shellcheck-clean script with gate:11 linting it. 564 tests ran over the scope. Coverage was
  100% of lines (2887) and functions (298). The receipt matches.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:**
  - `CHANGELOG.md` (Added).
  - `docs/marley_architecture/terminal_blocks.md` (`shell_integration.rs`).
  - `docs/marley/three-prong-plan.md`: the T0 row.
  - `CONSTITUTION.md`: gate:11's row.
  - The two ledger rows.
- **Knowledge:** `AD-claude-463-bash-loads-marleys-hooks-through-rcfile-prompt-command-and-ps0-001`
  and `L-claude-463-proving-a-shell-script-before-wiring-it-001`. No `F-` block: the shellcheck
  findings were caught before any commit.
- **Brain:** consultation `b4976182ea8d45aa8bf643bede62228c` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #463 closed. #465 (zsh) is queued, and #466 (fish) waits in Deliberate.
- **Live check (after the commit, 11:29):** Marley built from `d95c13ee81` and opened on this
  repository at Chad's request. `ps` shows both restored terminals' shells as
  `/usr/bin/bash --rcfile /home/cpeppers/.local/share/marley/shell_integration/marley.bash`.
  The script was already there with the same content, so the app did not rewrite it. The owed
  window check is done.
