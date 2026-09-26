# Blocks keep working over ssh — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-526-blocks-over-ssh.md
- **Pipeline spec:** 526-blocks-over-ssh.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** Chad approved all seven items of the Warp once-over on 2026-09-25; on item 3,
  Warpify subshells, "this would be awesome". As asked: blocks keep working after `ssh` into a
  host first, `docker exec` later, by installing Marley's shell integration in the remote shell at
  connect, with the per-terminal nonce (#474) travelling with it.
- **Classification:** feature, size M; this spec is the ssh slice, and `docker exec` is the next
  ticket (D8). Marley crate `marley_terminal` (the scripts, the bootstrap, the decoder, the block
  model) and `marley_workbench` (autosuggestions, `terminal_blocks`); one Zed hunk, the Rerun
  check in `terminal_view`'s element.
- **Recall (§18.3):**
  - AD-claude-474-a-blocks-command-is-trusted-only-with-the-terminals-nonce-001 and
    PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: a frame is output
    until its nonce says otherwise, and "an action on a `precmd` field adds the nonce to `precmd`
    first". AD-474 rejected a nonce on `precmd` because nothing acted on its fields yet; this
    ticket acts on them (which shell waits at the prompt), so every frame gets one (D4).
    F-claude-474-rerun-would-have-run-a-command-that-output-printed-001 already names "a remote
    host over ssh" as a source of forged frames.
  - AD-claude-463 and L-claude-463-proving-a-shell-script-before-wiring-it-001: bash's hooks through
    `--rcfile`, `PROMPT_COMMAND` and `PS0`; prove a script under the real shell on a
    pseudo-terminal before any Rust.
  - AD-claude-465 and L-claude-465-zsh-reads-an-unquoted-replacement-by-context-001: zsh's
    `ZDOTDIR` hand-back; quote `${var//pattern/replacement}` in zsh.
  - AD-claude-484-autosuggestions-read-the-typed-command-from-the-grid-001: suggestions come from
    verified blocks, then the history file the `history` frame names, read from the local disk.
  - L-claude-483-a-scenario-brings-its-own-shell-001 and L-claude-480-an-e2e-fake-acts-out-the-program-001:
    a scenario pins its shells; a fake on the PATH can act out a program when the real one cannot
    run in the scenario.
- **Discovery (the seams, checked 2026-09-25):**
  - `crates/marley_terminal/shell_integration/marley.bash`: the nonce copied into
    `__MARLEY_NONCE` and unset (lines 10-13); `~/.bashrc` sourced (15-17), which a login bash on a
    host would not read; the hooks' guard (19); `__marley_precmd` (36-41) and its frame without a
    nonce (39); `__marley_preexec` (45-52), the one frame with `nonce=` (50-51); `init` (62),
    `history` (64-67), `bootstrapped;subshell=0` (68).
  - `crates/marley_terminal/shell_integration/marley.zsh`: the `ZDOTDIR` hand-back (10-15), the
    nonce (20-23), `__marley_install` at the first prompt (66-79), which prints `init`, `history`
    and `bootstrapped`.
  - `crates/marley_terminal/src/shell_integration.rs`: `NONCE_VARIABLE` (44), `new_nonce` (48),
    `install_in` (69), which writes the two scripts, `for_program` (89).
  - `crates/terminal/src/terminal.rs`: `marley_integration_dir` (81); `marley_shell_integration`
    (90); the nonce for a local terminal only (1209-1219) and the integration for a local
    interactive shell only (1221-1228); `apply_shell_hook` (1817), which decodes a frame and hands
    it to the blocks. Nothing here changes.
  - `crates/marley_terminal/src/dcs.rs`: `PreexecValue` with its `nonce` (37), `PrecmdValue`
    without one (47), `DcsHook` (56); the decode arms `init` (161), `precmd` (177), `history` (202).
  - `crates/marley_terminal/src/anchored.rs`: `AnchoredBlock` (34); `AnchoredBlocks` (68), one
    `nonce`; `with_nonce` (84); `apply` (97), where `command_verified` compares the preexec's nonce
    (112); `at_prompt` (172); `history_file` (192).
  - `crates/terminal_view/src/terminal_element.rs`: Rerun offered while the last block is finished
    (1632), and in `marley_block` (2320) for a verified, non-empty command (2343): with a remote
    shell at its prompt, a local block would qualify.
  - `crates/marley_workbench/src/autosuggest.rs:56` `suggestion`: every verified block's command
    (63), then the history file (66).
  - `crates/marley_workbench/src/mcp.rs:428` `block_entry`, which #516 is editing for redaction.
  - `crates/marley_remote/src/marley_remote.rs:104` `ssh_command`; only the workspace's member
    list (`Cargo.toml:146`) names the crate, so no pane builds an ssh command through it.
  - OpenSSH 10.5p1 on the box: `ssh -G` prints the configuration after `Host` and `Match` and
    exits; `RemoteCommand` runs "with the user's shell"; sshd's `SetEnv` overrides the default
    environment and is allowed inside `Match`, whose criteria include `LocalPort`; `Tag` names a
    configuration block (`ssh_config(5)`).
- **Decisions:** D1 to D8 in the spec. D2's `Tag marley-plain` came from reading `ssh_config(5)`:
  a per-host opt-out in the user's own ssh config, with no Marley setting needed yet.

### Design
- **The wrapper** (bash and zsh, defined inside each script's hook guard, only when no `ssh`
  function exists: `declare -F ssh` in bash, `${+functions[ssh]}` in zsh). It walks the arguments
  with the option letters of the installed `ssh(1)`: those taking a value (`B b c D E e F I i J
  L l m O o P p Q R S W w`) skip it, attached or not; `--` ends the options; the first other word is
  the destination, and a word after it is a remote command. It runs plain `command ssh "$@"` for a
  remote command, an unknown option, any of `N T W f G V O Q s n`, stdin or stdout not a terminal,
  or a host whose `command ssh -G "$@"` prints a `remotecommand` other than `none` or a `tag
  marley-plain`. Otherwise it mints a connection nonce (`od -An -N16 -tx1 /dev/urandom`, spaces
  and newlines taken out, 32 hex digits like `new_nonce`), prints
  `ESC P q remote;host=<destination>;session=<connection nonce>;nonce=<terminal nonce> ESC \`, and
  runs `command ssh -t "$@" "sh -c '<bootstrap>' marley <connection nonce>"`, each `'` of the
  bootstrap written `'\''`.
- **The bootstrap** (`ssh-bootstrap.sh`, which `install_in` writes, built in Rust from
  `BASH_INTEGRATION` and `ZSH_INTEGRATION` so the three cannot drift). POSIX sh:
  `dir=$(mktemp -d "${TMPDIR:-/tmp}/marley.XXXXXX")` or `exec "$SHELL" -l`; the two scripts
  written with quoted here-documents; then by `${SHELL##*/}`: bash as
  `MARLEY_SHELL_NONCE=$1 __MARLEY_LOGIN=1 __MARLEY_CLEANUP=$dir exec bash --rcfile "$dir/marley.bash" -i`,
  zsh as `ZDOTDIR=$dir/zsh MARLEY_SHELL_NONCE=$1 __MARLEY_CLEANUP=$dir exec zsh -l` (with
  `MARLEY_ZSH_ZDOTDIR` when the host set a `ZDOTDIR`), anything else as `rm -rf "$dir"; exec "$SHELL" -l`.
- **The scripts' remote mode.** `__MARLEY_LOGIN` and `__MARLEY_CLEANUP` are read and unset first,
  as the nonce is. With `__MARLEY_LOGIN`, bash sources `/etc/profile` and the first readable of
  `~/.bash_profile`, `~/.bash_login` and `~/.profile` in place of `~/.bashrc`. bash removes the
  directory at the end of the rcfile, which it has read whole; zsh in `__marley_install`, at the
  first prompt. Both report `bootstrapped;subshell=1`, and neither defines the `ssh` function, so
  a nested ssh runs plain and announces nothing (the terminal would refuse an announcement that
  does not carry its own nonce anyway).
- **Every frame's nonce.** The scripts add `nonce=${__MARLEY_NONCE-}` to `init`, `precmd`,
  `history` and `bootstrapped`. `dcs.rs` decodes an optional `nonce` on each hook and the new
  `remote` hook (`host`, `session`, `nonce`, the first two required).
- **The block model.** `AnchoredBlocks` gains `connection: Option<(nonce, host)>` and the shell of
  the staged prompt. `shell_of(frame nonce)`: the terminal's nonce is `Local`, the connection's is
  `Remote(host)`, anything else `Unknown`. In `apply`: a `remote` frame from `Local` opens the
  connection; `InitShell` from the connection's shell finishes the running block with no exit
  code; `Precmd` from `Local` closes the connection before it applies; `Preexec` records the
  block's `shell` and `host`, and `command_verified` means a known shell; `History` from `Local`
  only. `rerun_offered(block)`: the block is verified and finished, and the shell at the staged
  prompt is the block's.
- **The element** (the Zed hunk): the per-view `rerun` flag becomes `anchored.rerun_offered(block)`
  for each block element.
- **Autosuggestions:** `suggestion` keeps the blocks of the prompt's shell, and the history file
  only when that shell is `Local`.
- **`terminal_blocks`:** `host` in `block_entry` and in the schema (`registry.rs`).
- **File manifest.**
  - Marley crates: `crates/marley_terminal/shell_integration/marley.bash`, `marley.zsh`;
    `crates/marley_terminal/src/shell_integration.rs` (`install_in`, the bootstrap);
    `crates/marley_terminal/src/dcs.rs`; `crates/marley_terminal/src/anchored.rs`; the gpui-era
    `apply.rs` and `session.rs` follow the new fields; `crates/marley_workbench/src/autosuggest.rs`,
    `mcp.rs`; `crates/marley_mcp/src/registry.rs` (the schema); `script/e2e/526-blocks-over-ssh.sh`
    (Test).
  - Zed crate: `crates/terminal_view/src/terminal_element.rs` (the Rerun check).
- **Ledger rows:** extend `crates/terminal_view/src/terminal_element.rs`'s row with the #526 change
  to the #474 Rerun check. `vendor/` and `crates/terminal/src/terminal.rs` are unchanged.

### E2E plan
`script/e2e/526-blocks-over-ssh.sh`, `compositor sway`. Setup: a local HOME with `PS1='$ '`; a
remote HOME with a `.bash_profile` that sources its `.bashrc` (`PS1='remote$ '`), a `.zshrc`
(`PROMPT='zsh-remote%% '`) and `forged.txt` (a `preexec` frame with a made-up nonce and a
`precmd`); host and client keys made with `ssh-keygen -N ''`; `sshd -f <config> -D -e` in the
background on three free loopback ports with `UsePAM no`, `StrictModes no`,
`PasswordAuthentication no`, `AuthorizedKeysFile <scratch>`, `SetEnv HOME=<remote home>
TMPDIR=<remote tmp>`, and `Match LocalPort` blocks adding `SHELL=/usr/bin/zsh` and `SHELL=/bin/sh`;
a client config with `Host e2e`, `e2e-zsh`, `e2e-sh` and `e2e-plain` (`Tag marley-plain`),
`HostKeyAlias`, `UserKnownHostsFile` holding the host key, `StrictHostKeyChecking yes`,
`LogLevel ERROR`; the stand-in MCP client of #491. Teardown stops sshd.

| REQ | Step | Shot or log |
|---|---|---|
| REQ-001, REQ-002 | `echo local`; `ssh -F cfg e2e`; `echo on the far side`; `false` | `526-01-remote-blocks` |
| REQ-003 | the pointer on the remote `false` block, Rerun clicked | `526-02-remote-rerun` |
| REQ-003 | the pointer on the local `echo local` block | `526-03-no-local-rerun` |
| REQ-004 | `cat forged.txt`, the pointer on its block | `526-04-forged` |
| REQ-005 | `ech` (local history only), then `echo on` | `526-05-remote-suggestions` |
| REQ-006, REQ-011 | `exit`; the pointer on the remote block, then the local one | `526-06-back-local` |
| REQ-001 | `ssh -F cfg e2e-zsh`, a command, `exit` | `526-07-zsh` |
| REQ-007 | `command ssh -F cfg e2e`, a command, `exit`; `ssh -F cfg e2e-plain`; `ssh -F cfg e2e true` | `526-08-plain`; the log |
| REQ-008 | `ssh -F cfg e2e-sh`, `echo $0`, `exit` | `526-09-other-shell` |
| REQ-009 | `ls <remote tmp>` after the first connection | the log |
| REQ-010 | the stand-in client's `terminal_blocks` | the log: `host` `e2e` on the remote blocks, null on the local |
| REQ-011 | `script/e2e/484-autosuggestions.sh` | its shots |
| REQ-012 | `just gate-diff` | the gate's exit |

Not reachable: a network host (the scenario's sshd is local, which exercises the same ssh client,
server and remote shells), and hosts whose login shell is not POSIX.

### Risks
- **An unprivileged sshd** on OpenSSH 10.5 (split into `sshd-session` and `sshd-auth`) may refuse
  to run as the user. Test checks it first; the fallback is a fake `ssh` first on the PATH that
  logs its arguments and runs the remote command with `sh -c` under the scratch remote HOME, which
  proves everything but the server.
- **A login shell that is not POSIX** (a router's CLI) receives `sh -c '...'` as a command and the
  session fails. `command ssh` and `Tag marley-plain` skip the wrapper; the Code phase confirms
  that `ssh -G` prints the tag.
- **The option table** drifts with OpenSSH: an option the wrapper does not know makes it run plain
  ssh, which fails safe.
- **The remote command** shows in the host's process list for the moment before `exec`, with the
  connection nonce (D3); the terminal's own nonce never leaves.
- **The scripts change for local shells too** (a nonce on every frame): #484's scenario and a run
  of the scripts under `script -qfc` (L-claude-463) guard it.
- **T2's path links** will need a block's host: a remote block's directory is on the host, which
  `host` on the block now says.
