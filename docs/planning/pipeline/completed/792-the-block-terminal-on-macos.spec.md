---
pipeline_id: 5e4ab248-84bf-4189-be24-73012ecffcde
ticket: docs/planning/tickets/closed/TICKET-792-the-block-terminal-on-macos.md
status: Phase 4 — Complete PASS
title: "The block terminal on macOS"
type: bug
slice: macOS
references: [docs/planning/pipeline/completed/788-ports-on-macos.spec.md]
---

## Title
On the Mac (#787), a new zsh terminal in Marley shows zsh's plain prompt under "Last login: …".
No prompt editor docks, and no block forms. `terminal_run` refuses with "a program runs in the
terminal's foreground".

On macOS, the vendored alacritty starts a shell that has no arguments through `/usr/bin/login
-flp <user> /bin/zsh -fc 'exec -a -zsh <shell>'`, so the shell is the terminal's grandchild. Two
probes on the Mac (#792's notes) show the integration loads there: zsh opens Marley's `.zshenv`,
and its hooks write `init`, `history`, `bootstrapped` and `precmd` frames. Yet Marley's prompt
never comes, and `terminal_drive::program_of` takes `login` as the shell. With the shell started
directly (`/bin/zsh`), the prompt editor docks and `terminal_run` forms a block with exit 0.

## Scope
### In
- `crates/marley_terminal/src/shell_integration.rs`: on macOS, the integration for zsh and fish
  adds `-l`. The shell then starts directly, as Marley's own child, and stays a login shell
  (`/etc/zprofile`'s `path_helper`, the user's `.zprofile`). `shown_arguments` already leaves the
  integration's arguments out of the title.
- The crate's own tests take the expected arguments from the same platform rule, so they hold on
  both systems.

### Out (explicitly deferred)
- bash on macOS. Its integration already adds `--rcfile`, so it starts directly. As a non-login
  shell it does not read `.bash_profile`; that is a question for a bash user.
- Why the hooks' prompt is not taken through `login`. The fix removes `login` from the path
  rather than chasing the frames through it.
- Zed's own terminal without Marley's integration (`sh`, nushell), which keeps alacritty's
  `login` wrapper.

## Reference (§20)
Upstream Zed's terminal (`crates/terminal`), with the vendored `alacritty_terminal`:
- a shell with no arguments goes to alacritty's macOS default, `login -flp`;
- a program with arguments is started as given.

Marley keeps both rules. Only its integrated shells on macOS now carry `-l`, so they are started
as given. Published practice on macOS: VS Code's terminal starts the user's shell directly with
`-l` (its `terminal.integrated.profiles.osx`), so that `.zprofile` runs.

### Prior art
- **The code we ship:**
  - `vendor/alacritty_terminal/src/tty/unix.rs:207-232` (`default_shell_command`, macOS only):
    the `login` wrapper, used when `Options.shell` is `None`, with `exec -a -<name>` making a
    login shell.
  - `crates/terminal/src/terminal.rs:109-140` (`marley_shell_integration`): an integration with
    arguments becomes `Shell::WithArguments`, which alacritty starts as given (`Command::new` and
    its args, `unix.rs:257-260`).
  - `marley_terminal::shell_integration::for_program` / `shown_arguments` (:213, :280).
- **AD-465:** zsh loads Marley's hooks through a `ZDOTDIR` and starts "unchanged but for its
  environment". On macOS it now also gets `-l`, so it starts as alacritty's wrapper would have
  made it, a login shell, without the wrapper.
- **Published:** `zsh(1)` (`-l`: act as a login shell; startup files `/etc/zprofile`,
  `$ZDOTDIR/.zprofile`), `fish(1)` (`-l`, `--login`), `login(1)` on macOS (`-f`, `-l`, `-p`).

## UI proof
On the Mac, where no `script/e2e.sh` scenario runs. Marley is reinstalled with the change and
reopened, and its restored terminal starts anew.
- `792-mac-prompt.png`: the terminal with Marley's prompt editor docked ("A command for the
  shell") and no "Last login" line.
- `terminal_run` (MCP) of `echo` in it: a block with exit code 0 and the output.
- The process tree: Marley → `/bin/zsh -l`, and the title "zsh".

On Linux nothing changes (the rule is `cfg(target_os = "macos")`); its run is `just shot`.

## Locked-In Decisions
- D1 — The fix is in the integration's arguments, a Marley crate, not in Zed's terminal hunk or
  in the vendored alacritty. One rule decides both the launch (`for_program`) and the title
  (`shown_arguments`).
- D2 — `-l`, not `--login`: zsh and fish both take `-l`, and zsh treats a leading `-l` as a
  login option before any script.
- D3 — bash is left as it is (D-out). Its `--rcfile` already makes it start directly, and `-l`
  would make bash ignore `--rcfile`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley opens a terminal on macOS whose shell is zsh, the shell shall be Marley's direct child, started as `zsh -l` | the Mac's process tree |
| REQ-002 | WHILE that shell waits at its prompt, Marley's prompt editor shall be docked under the terminal | `792-mac-prompt.png` |
| REQ-003 | WHEN `terminal_run` types a command at that prompt, it shall run and form a block with its exit code | the MCP answer; `terminal_blocks` |
| REQ-004 | WHILE the shell runs, it shall be a login shell: `/etc/zprofile` and the user's `.zprofile` have run | the shell's `[[ -o login ]]` and `$PATH` holding `/opt/homebrew/bin`, through `terminal_run` |
| REQ-005 | WHILE a terminal's title names its shell, it shall leave out the `-l` Marley added | the tab's title "zsh" (shot) |
| REQ-006 | WHILE Marley runs on Linux, zsh and fish shall start as before, with no `-l` | the review of the diff (`cfg`); `just shot`; the gate |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the macOS arguments in `for_program`, the tests' expected values; a review; the
  gate on the box; clippy on the Mac.
- **P3 Test** — on the Mac: reinstall, reopen, the tree, the shot, `terminal_run`; on Linux:
  `just shot`.
- **P4 Complete** — CHANGELOG, the guide's Mac section, `marley_terminal`'s note, the ledger,
  close, archive, commit.
