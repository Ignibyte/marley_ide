# Shell integration for zsh — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-465-zsh-shell-integration.md
- **Pipeline spec:** 465-zsh-shell-integration.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] promote · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad's goal: "lets continue on those",
  T1 and zsh; autonomous).
- **Classification / tier:** feature, small; `marley_terminal` and #463's hunk in Zed's
  `terminal`.
- **Pre-flight:** #467 and #468 committed; no other active pipeline; README marker present;
  cargo idle; zsh 5.9.2 at `/usr/bin/zsh`.
- **Recall (§18.3).**
  - `AD-claude-463-bash-loads-marleys-hooks-through-rcfile-prompt-command-and-ps0-001` and
    #463's lesson on running the script under the real shell on a pseudo-terminal first.
  - `F-claude-467-a-marley-shells-title-showed-its-rcfile-001`: check the title of a shell
    Marley starts differently.
  - Brain: consultation `57fff85e4de94e4ca4ff2053eb6f2288`, nothing on this seam.
- **The script, run first** (`script -qfc "zsh -i"` with a scratch `HOME` and Marley's
  `ZDOTDIR`, frames read with `cat -v`):
  - the scratch `.zshenv` and `.zshrc` markers print; at the first prompt `init`,
    `bootstrapped` and a `precmd` arrive, then per command `preexec` with the line and `precmd`
    with its exit code. `false` reports 1 though a hook the `.zshrc` added returns 3 (zsh gives
    each hook the command's status; Phase 2).
  - With `MARLEY_ZSH_ZDOTDIR` set, the `.zshrc` there runs and `$ZDOTDIR` reads it afterwards;
    without it, `$ZDOTDIR` is empty.
  - **Found:** the first prototype sent `;` unescaped. Its file held `\;` where `\\;` was meant.
    Testing the forms then showed a real difference: at a script's top level an unquoted
    `r=${v//;/\\;}` leaves `;` bare while `r="${v//;/\\;}"` escapes it; inside a function both
    escape. The script quotes every expansion, which reads the same everywhere, and a rerun
    escapes `;` and `\` as the decoder expects.

### Design
- **`marley.zsh`** (installed as `zsh/.zshenv`): the prototype above, which restores
  `ZDOTDIR`, sources the user's `.zshenv`, and in an interactive shell defines
  `__marley_quote`, `__marley_precmd`, `__marley_preexec` and `__marley_install`, the last
  appended to `precmd_functions`.
- **`shell_integration.rs`**: `ZSH_INTEGRATION`, `ZSH_DIR` (`zsh`), `ZSH_FILE` (`.zshenv`),
  `ZSH_ZDOTDIR_VARIABLE` (`MARLEY_ZSH_ZDOTDIR`); `install_in` writes both scripts, each only
  when its content changed; `for_program(program, dir, user_zdotdir)` gains the zsh arm;
  `shown_arguments` passes no `ZDOTDIR`.
- **`terminal.rs`**: `marley_shell_integration` reads the user's `ZDOTDIR` from `env`, else the
  process's, and returns the shell as it was when the integration adds no arguments.
- **File manifest.**
  - Marley: `crates/marley_terminal/shell_integration/marley.zsh` (new),
    `crates/marley_terminal/src/shell_integration.rs`.
  - Zed: `crates/terminal/src/terminal.rs` (the hunk and the PTY tests); its ledger row first.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: `for_program` for `zsh` and `/usr/bin/zsh`, with and without a user `ZDOTDIR`; `install_in` writes `zsh/.zshenv`; the spawn hunk's unit test: zsh stays `Program("zsh")` and its env gains the three variables |
| 002 | PTY: a scratch `HOME` with `.zshenv` and `.zshrc` markers; a second run with `ZDOTDIR` set to a scratch directory whose `.zshrc` marks, then `echo "[$ZDOTDIR]"` reads it |
| 003 | PTY: `echo hi` (exit 0, output `hi`), `false` (exit 1), `echo 'a;b'` (output `a;b`) |
| 004 | PTY: `title(false)` trimmed ends `— zsh` |
| 005 | `script/gates.sh --diff` |

### Risks
- **shellcheck does not read zsh**, so gate:11 cannot lint the script; the PTY tests run it
  under zsh, and `zsh -n` checked its syntax.
- **A user's `.zshrc` that assigns `precmd_functions`** removes the installer (Out).

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] ledger row first · [x] `marley.zsh` · [x]
  `shell_integration.rs` · [x] `terminal.rs` · [x] fmt · [x] review.
- **Built.**
  - `crates/marley_terminal/shell_integration/marley.zsh`, the prototype.
  - `shell_integration.rs`: `ZSH_INTEGRATION`, `ZSH_DIR`, `ZSH_FILE`, `ZSH_ZDOTDIR_VARIABLE`;
    `install_in` writes both scripts through `write_if_changed`; `for_program` takes the user's
    `ZDOTDIR` and has the zsh arm; `shown_arguments` passes none. Tests for zsh's start and for
    both scripts' install.
  - `terminal.rs`: the user's `ZDOTDIR` from `env`, else the process; the shell returned as it
    was when the integration adds no arguments; `build_marley_shell_terminal` shared by bash and
    zsh, `require_zsh`, `run_in`; the unit test
    `marley_shell_integration_gives_zsh_marleys_zdotdir_and_hands_on_the_users` and the PTY tests
    `marley_zsh_reports_each_typed_command_as_a_block` and
    `marley_zsh_reads_the_users_zdotdir_and_keeps_it`. The bash unit test now takes `sh` as the
    shell left alone, since zsh is no longer.
- **Deviation: D2's reason.** zsh restores `$?` before each `precmd` hook (a second hook saw 1
  after `false` though the first returned 3), so going first does not decide the exit code. It
  decides where the command's output ends: a user hook that prints before the prompt would
  otherwise print inside the block. The PTY test's `.zshrc` adds such a hook.
- **Review.**
  - `std::env::var("ZDOTDIR")` is read only when the terminal's environment has none, so a
    project's environment wins, as it would for the shell itself.
  - An empty `ZDOTDIR` the user set is handed on as empty, and the script's `${…+set}` test
    restores it as set and empty, as zsh would have seen it.
  - The zsh file is written into its own directory, which holds nothing else, so zsh finds no
    other startup file there even if the restore were skipped.
  - Upstream: the hunk grew inside #463's function; the ledger row was updated first.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 · [x] REQ-002 · [x] REQ-003 · [x] REQ-004 · [x] negative checks ·
  [x] live drive · [x] gate.
- **Tests.**
  - `marley_terminal`: `zsh_starts_with_marleys_zdotdir_and_carries_the_users_own` (REQ-001);
    `install_writes_the_scripts_leaves_identical_ones_and_rewrites_changed_ones` for both
    scripts; the shell-integration tests, 5 passed.
  - `terminal`: `marley_shell_integration_gives_zsh_marleys_zdotdir_and_hands_on_the_users`
    (REQ-001); `marley_zsh_reports_each_typed_command_as_a_block` (REQ-002 without a
    `ZDOTDIR`, REQ-003, REQ-004), whose `.zshrc` adds a `precmd` hook that prints;
    `marley_zsh_reads_the_users_zdotdir_and_keeps_it` (REQ-002 with one). The eight `marley_`
    tests passed.
- **Negative checks** on `marley.zsh`, each restored by checksum:
  - Marley's `precmd` put last: the first PTY test fails (the user hook's line lands in the
    block's output).
  - `ZDOTDIR` left at Marley's directory: the first PTY test fails (the user's `.zshrc` is not
    read).
  - The user's `ZDOTDIR` not restored: the second PTY test fails.
  - The `;` replacement unquoted: both pass. Inside `__marley_quote` zsh escapes the unquoted
    form too (Phase 1); the quotes are for code at a script's top level, which this script
    does not escape at.
- **Gate.** Three reds before the green, each fixed at the source:
  - gate:2, `clippy::too_long_first_doc_paragraph` on `for_program`'s doc
    (L-claude-465-a-doc-opens-with-one-short-line-001);
  - gate:2, Zed's `clippy.toml` bans `std::process::Command::status` in `require_zsh`; it scans
    `PATH` now (L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001);
  - gate:4, 45 missed lines in `shell_integration.rs`, all on doc comments and a derive. A
    fresh coverage run showed a second instrumented copy of `marley_terminal` whose regions
    sat on the file's #467 lines: `marley_workbench`'s test executable from #468's coverage run,
    still in `llvm-cov-target/debug/deps` and read by cargo-llvm-cov, linked the old
    `marley_terminal`. Removing that executable (only it carried `shell_integration.rs`) gave
    the tree's own report. The gate's part is TICKET-469.
  - Then: `GATE GREEN [diff]`, 20 passed, the receipt matching the tree.
- **Live drive.** The debug `marley` on a copy of Chad's profile with
  `"terminal": { "shell": { "program": "zsh" } }` added, on hidden workspace 9, shot by its
  toplevel with no input sent. The three terminals read `marley_ide — zsh`, `crates — zsh` and
  `cpeppers — zsh` in their tabs and rows; the shown one sits at zsh's default prompt with no
  new-user menu, though Chad has no zsh files (the spec's Out). The live zsh processes started
  with `MARLEY_SHELL_INTEGRATION=1` and `ZDOTDIR=<data dir>/shell_integration/zsh`, and no
  `MARLEY_ZSH_ZDOTDIR`, as he has no `ZDOTDIR`. What a process's environment cannot show, the
  restore inside the shell, the PTY tests cover.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Added); `docs/marley_architecture/terminal_blocks.md` (zsh in
  `shell_integration.rs`); `docs/marley/three-prong-plan.md` (T0: zsh shipped); the
  `crates/terminal/src/terminal.rs` row in `docs/marley/zed-touchpoints.md`, written before the
  edit, describes the hunk as shipped.
- **Ledger (§19).** `AD-claude-465-zsh-loads-marleys-hooks-through-a-zdotdir-that-hands-back-the-users-001`,
  `F-claude-465-gate4-counted-lines-from-a-stale-executable-001`,
  `L-claude-465-zsh-reads-an-unquoted-replacement-by-context-001`,
  `L-claude-465-a-doc-opens-with-one-short-line-001`,
  `L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001`. TICKET-469 opened and queued for
  the gate's part of the F block.
- **Brain.** Consultation `57fff85e4de94e4ca4ff2053eb6f2288` closed by
  `decisions/zsh-loads-marleys-hooks-through-a-zdotdir-that-hands-back-the-users`, follow-up
  2026-10-07.
- **Ticket** closed; the pipeline archived; one commit.
