# The block terminal on macOS — Notes

- **Local ticket doc:** [TICKET-792](../../tickets/open/TICKET-792-the-block-terminal-on-macos.md)
- **Pipeline spec:** 792-the-block-terminal-on-macos.spec.md

## Phase 1 — Plan
- **Request:** seen in #788's check on the Mac. Marley's terminal there showed zsh's plain prompt,
  with no prompt editor and no block, and `terminal_run` refused it.
- **Classification / tier:** bug, macOS. Marley crate only (`marley_terminal`), no Zed hunk.
- **Recall (§18.3):**
  - AD-465: zsh gets Marley's hooks through `ZDOTDIR` and starts unchanged but for its
    environment. On macOS that left it to alacritty's `login` wrapper.
  - #467 (`shown_arguments`): the title leaves out the integration's arguments, so a `-l` added
    there stays out of the title by the same rule.
  - The brain (consultation `3aa32ecdbf1c4b1db9762671828b8221`): nothing on the macOS launch.
- **Discovery (probes on the Mac, 2026-10-16):**
  - **The tree.** `marley` → `/usr/bin/login -flp chadpeppers /bin/zsh -fc exec -a -zsh /bin/zsh`
    (pid 83483) → `-zsh` (83485). The tty's foreground group is 83485, while Marley's
    `pid_getter().fallback_pid()` is 83483, so `terminal_drive::program_of` reports a program.
  - **The hooks load under `login`.**
    - `fs_usage` on a Marley restart shows the terminal's zsh opening
      `…/Application Support/Marley/shell_integration/zsh/.zshenv`.
    - Run by hand under `script(1)` the same way (`login -flp … /bin/zsh -fc "exec -a -zsh
      /bin/zsh"`, with `ZDOTDIR` and a nonce), zsh writes
      `init;id=…;nonce=abc`, `history;…`, `bootstrapped;subshell=0` and `precmd;exit=0;…` frames.
    - Yet Marley never reaches `at_prompt` there (shot `792-login.png`: no prompt editor, plain
      prompt), and its log says nothing.
  - **Direct zsh works.** A temporary setting `terminal.shell.program: "/bin/zsh"` made alacritty
    start zsh directly, with Marley's integration (a `Program` still integrates), and:
    - the prompt editor docked ("A command for the shell", shot `792-direct.png`);
    - the rail row showed "done · 0 s";
    - `terminal_run "echo probe-792"` came back as block 0, exit 0, output `probe-792`.

    The setting was taken back out (the backup copied over, `grep -c "TEMP #792"` = 0).
  - `marley_shell_integration` keeps `Shell::System` when the integration adds no arguments,
    which alacritty sends to its `login` wrapper. With arguments it builds `Shell::WithArguments`,
    which alacritty starts as given.
  - The Mac's `.zprofile`, `.zshenv` and `.zshrc` touch none of the hooks; `.zshrc` sources
    iTerm2's own shell integration, which the direct run had too.
- **Decisions:** D1 to D3 in the spec.

### Design
- In `marley_terminal::shell_integration`:
  - `fn login_arguments() -> Vec<String>`: `vec!["-l"]` under `cfg(target_os = "macos")`, else
    empty, with a comment on why;
  - `for_program`'s zsh and fish arms take `args: login_arguments()`;
  - bash is unchanged.
- The tests in that file build the zsh and fish expectations with `login_arguments()`, so they
  hold on both systems. On Linux the values are the same as before.
- **File manifest:** `crates/marley_terminal/src/shell_integration.rs` (Marley crate). Nothing
  else.

### Visual check plan
| REQ | What runs on the Mac | What shows it |
|---|---|---|
| 001 | `just install` with the change; quit and reopen the test Marley (its restored terminal starts a new shell) | `ps`: Marley → `/bin/zsh -l` |
| 002, 005 | the window shot through iTerm | `792-mac-prompt.png`: the prompt editor docked, the tab "ports-788 — zsh" |
| 003 | `terminal_run "echo probe-792b"` through Marley's MCP | the answer: a block, exit 0, the output |
| 004 | `terminal_run '[[ -o login ]] && echo login; echo $PATH'` | "login", and `/opt/homebrew/bin` in the PATH |
| 006 | Linux: `just shot` | Marley starts. The diff is `cfg`-bound |

### Risks
- **A user's `.zprofile` that does something slow or noisy.** It runs now as it would in
  Terminal.app, so the behavior is the platform's.
- **"Last login" goes away.** `login` printed it. Terminal.app shows it, but it carries no
  information Marley needs.

## Phase 2 — Code
- **What was built:** in `marley_terminal::shell_integration`:
  - `login_arguments()` (`-l` under `cfg!(target_os = "macos")`, else none), with its why;
  - `for_program`'s zsh and fish arms take it;
  - the zsh test's two expectations take it too, so the test holds on both systems.

  bash is unchanged.
- **No deviation.** `cfg!` (the macro), not `#[cfg]`, keeps both branches compiled on both
  systems, so nothing is dead code on either.
- **Review:**
  - REQ-001: with arguments, `marley_shell_integration` (Zed hunk, unchanged) builds
    `Shell::WithArguments`, which alacritty starts as given rather than through `login`.
  - REQ-005: `shown_arguments` strips the integration's arguments, now `-l` on macOS, from the
    title.
  - REQ-006: on Linux `login_arguments()` is empty, the old value.
  - No Zed hunk, so no ledger row.
- **Checks:** box clippy on `marley_terminal` green; **GATE GREEN [diff]**, 17 of 17. Mac clippy
  `-D warnings` on `marley_terminal`: recorded in Phase 3 with the install.

## Phase 3 — Test
- **Mac clippy and install.** `cargo clippy -p marley_terminal -- -D warnings` is green on the
  Mac, then `just install` there, at `6cbeb1a651` with this diff. The test Marley was quit and
  reopened, so its restored terminal started a new shell.
- **REQ-001:** the tree is `marley` (90768) → `/bin/zsh -l` (90807), Marley's direct child. No
  `login` is in the path.
- **REQ-005:** `terminal_list` names the tab `ports-788 — zsh`, without `-l`.
- **REQ-003:** `terminal_run "echo probe-792b"` returned block 1, `exit_code` 0, output
  `probe-792b`, with no refusal.
- **REQ-004:** `terminal_run '[[ -o login ]] && echo login-shell; … grep -c homebrew'` returned
  block 2, exit 0, output `login-shell` and `1`. It is a login shell, with Homebrew's folder on
  the PATH that `/etc/zprofile` and `.zprofile` build.
- **REQ-002:** `792-mac-prompt.png` shows Marley's prompt editor docked under the terminal ("A
  command for the shell"). The second command is a block under its header `~/ports-788`, with
  its output. The rail row reads "… · done · 0 s". The new shell printed no "Last login" line;
  the ones above are restored scrollback from the `login` runs. Compare `792-login.png` before
  the change: no prompt editor, zsh's plain prompt.
- **Seen, not in scope:** the first command after the shell's start drew without Marley's
  header, while the second had one. The first prompt is drawn before the hooks install at the
  first `precmd`. To be checked against Linux bash before a ticket.
- **REQ-006:** `just build`, then `COMPOSITOR=sway just shot 792-linux-unchanged`.
  `792-linux-unchanged.png` is Marley as before (`repo — bash`, the trust prompt), and the
  Hyprland Marley was untouched.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md`, Fixed: "The block terminal on a Mac".
  - `docs/marley/guide.md`, On a Mac: a block terminal item, with bash's `.bash_profile` note.
  - `docs/marley/three-prong-plan.md`: the macOS bullet.
  - `docs/marley_architecture/terminal_blocks.md`: `-l` on macOS, and why.
  - No Zed hunk changed, so no touchpoint row.
- **Ledger:**
  - F-claude-792-a-macos-shell-under-login-lost-the-block-terminal-001
  - PR-claude-792-the-shell-marley-integrates-is-the-terminals-own-child-001
  - L-claude-792-probing-a-terminal-launch-on-a-mac-001
  - AD-claude-792-on-macos-zsh-and-fish-start-as-login-shells-of-their-own-001
- **Brain:** consultation `3aa32ecdbf1c4b1db9762671828b8221` closed by
  `decisions/on-macos-marleys-zsh-and-fish-start-as-login-shells-of-their-own`.

