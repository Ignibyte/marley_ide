---
pipeline_id: b445fd82-8883-4177-848f-0c0517427ca4
ticket: docs/planning/tickets/open/TICKET-526-blocks-over-ssh.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Blocks keep working over ssh"
type: feature
slice: prong 1 T0c (shell integration; Warp once-over item 3), ssh first
references: [docs/planning/design-notes/warp-once-over-2026-09-25.md, docs/planning/pipeline/completed/463-bash-shell-integration.spec.md, docs/planning/pipeline/completed/465-zsh-shell-integration.spec.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md]
---

## Title
After `ssh` into a host from a Marley terminal, the commands typed there become blocks, as local
ones do. Marley's shell integration travels in the ssh command and runs in the remote bash or zsh
for that session only, and the #474 nonce's guarantee holds over the connection: a block is
offered Rerun only by the shell that ran it.

## Scope
### In
- **The `ssh` function** in Marley's bash and zsh scripts (`crates/marley_terminal/shell_integration/`),
  defined in the local shell only (on the host the same scripts leave it out, so nested ssh runs
  plain). For an interactive login it mints a connection nonce (128 bits from `/dev/urandom`), prints a
  `remote` frame naming the destination as typed and the connection nonce, carrying the
  terminal's nonce, and runs `command ssh -t <the user's arguments> <bootstrap>`, where the
  bootstrap is `sh -c '<script>' marley <connection nonce>`, quoted for the remote login shell.
  Anything else runs `command ssh` with the arguments as typed: a remote command, `-N`, `-T`,
  `-W`, `-f`, `-G`, `-V`, `-O`, `-Q`, `-s`, `-n`; an option it does not know; stdin or stdout not
  a terminal; a host whose configuration sets `RemoteCommand` or carries `Tag marley-plain` (both
  read with `ssh -G`), the per-host opt-out in the user's own ssh config. A user's own `ssh`
  function is left alone, and `command ssh` skips the wrapper.
- **The bootstrap**, a POSIX sh script that `shell_integration::install_in` writes beside the two
  scripts, built from them: it makes a temporary directory (`mktemp -d`), writes the bash and zsh
  scripts there, and by the host's `$SHELL` starts bash with `--rcfile` or zsh as a login shell
  with Marley's `ZDOTDIR`, the connection nonce in `MARLEY_SHELL_NONCE` as locally. The scripts
  remove the directory once read. Another shell, or a failed `mktemp`, gets `exec "$SHELL" -l`, the
  login shell plain ssh would give.
- **The remote bash** reads the files a login bash reads (`/etc/profile`, then the first of
  `~/.bash_profile`, `~/.bash_login`, `~/.profile`) in place of `~/.bashrc`; the remote zsh reads
  the user's files through the `ZDOTDIR` hand-back #465 made.
- **Every frame carries its shell's nonce**: the scripts add `nonce=` to `init`, `precmd`,
  `bootstrapped` and `history` as well as `preexec`, and `marley_terminal::dcs` decodes it on each
  hook, and the new `remote` hook.
- **The block model** (`marley_terminal::anchored`): a frame comes from the local shell when it
  carries the terminal's nonce, from a connection's remote shell when it carries a nonce a
  verified `remote` frame announced, and from no known shell otherwise. Each block keeps its
  shell and host. A connection's nonce is accepted from its announcement until the local shell's
  next `precmd`. The running `ssh` block ends at the remote shell's first frame, with no exit
  code. A history file is taken from the local shell only.
- **Rerun** is offered for a verified block only while the shell that ran it waits at its prompt
  (`terminal_view`'s #474 hunk asks the block model).
- **Autosuggestions** at a prompt draw on that shell's own commands, and on the history file only
  at the local shell's prompt.
- **`terminal_blocks`** gives each block's `host`: the destination as typed, or null for the local
  shell.

### Out (explicitly deferred)
- `docker exec` into a container's bash or zsh, the next slice: the same bootstrap through
  `docker exec -it <container> sh -c`. Then the rest of Warp's list (`podman`, `poetry shell`,
  `gcloud compute ssh`).
- Nested ssh from the remote host, which Warp's legacy wrapper does not bootstrap either.
- fish (#466); remote hosts whose shell is neither bash nor zsh.
- The terminals of Zed's remote projects (`is_remote_terminal`), which get no integration and no
  nonce (#474).
- A setting and an offer before the first wrap, as Warp's "Always ask"; they belong on the
  Marley page (#515). Until then `command ssh` and `Tag marley-plain` are the ways around it.
- A host label drawn on remote blocks, T2's path links against a remote block's directory, and
  the ssh session drawn as one block holding the remote ones (stage two, T5).
- `marley_remote`, which no crate in the fork calls.

## Reference (§20)
- **Warp, Warpify subshells** (https://docs.warp.dev/terminal/warpify/subshells/; the Warp
  once-over, `docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 3): for bash, zsh or
  fish sessions nested in a running shell, local, in a container or over SSH, Warp runs a setup
  script so blocks work there; `docker exec`, `poetry shell` and SSH-like commands are recognized
  by default; an rc-file line printing a DCS makes it automatic. **Warp's legacy SSH wrapper**
  (https://docs.warp.dev/terminal/warpify/ssh-legacy/) brings blocks "to a remote session without
  installing anything on the remote host": a shell function (`warp_ssh_helper`) authenticates with
  the system `ssh` and bootstraps the remote shell; bash or zsh only; for zsh "Warp creates a
  temporary ZDOTDIR to bootstrap the shell and removes it once setup finishes"; nested ssh is not
  bootstrapped; `command ssh` bypasses it; a `RemoteCommand` in the ssh config breaks it. Warp's
  current SSH extension (https://docs.warp.dev/terminal/warpify/ssh/) installs a companion server
  under `~/.warp*/` after asking. Marley matches the legacy wrapper: a function, nothing
  installed, a temporary directory removed after start, bash and zsh, `command ssh` to bypass,
  and it skips a host with `RemoteCommand` instead of failing. No Warp code.
- **Upstream Zed:** the local terminal's spawn path (`TerminalBuilder::new`), where #463 and #465
  load the integration and #474 gives the nonce; Zed's own remote-project terminals are left as
  they are.

### Prior art
- **Behavior maps and reports.** The once-over, item 3 ("blocks stop at the first `ssh` or `docker
  exec`, and much of the work on this box is SSH. The #474 nonce would have to travel with the
  bootstrap"). Orca report 04 §2.10: Orca's SSH worktrees upload a Node relay to `~/.orca-remote/`
  that owns the remote PTYs, which is Warp's extension model and the one this ticket declines.
  Orca report 05 §2.3: Orca's wrappers hand zsh's `ZDOTDIR` back before the user's files run and
  read then unset `ORCA_SHELL_FEATURES` first (`src/main/zsh-startup-wrapper-builder.ts`,
  `src/main/daemon/daemon-bash-shell-ready-rcfile.ts`), the same shape as Marley's scripts, and
  inject across local, daemon and relay spawn paths.
- **Published material.** kitty's ssh kitten (https://sw.kovidgoyal.net/kitty/kittens/ssh/): ssh
  runs "a POSIX sh (or optionally Python) bootstrap script" as the remote command, which reads a
  compressed tarball over the TTY, guarded by a one-time password, installs it under
  `~/.local/share/kitty-ssh-kitten`, and starts the login shell with integration; it is opt-in,
  through an alias. Marley's scripts are small enough to ride in the command itself, with no TTY
  round trip and nothing left on the host. OpenSSH `ssh(1)`: a command after the destination runs
  instead of a login shell, `-t` forces a PTY, and `ssh -G` prints a host's effective
  configuration without connecting.
- **The code we already ship.**
  - `marley_terminal::shell_integration` (`install_in`, `for_program`, `new_nonce`) and the two
    scripts; Zed's `terminal` loads them for local interactive shells only
    (`marley_shell_integration`) and gives the nonce (#474).
  - `marley_terminal::anchored::AnchoredBlocks`: `with_nonce`, `apply` (a `Preexec` finishes a
    running block and opens one; a `Precmd` finishes it and stages the prompt), `command_verified`;
    `marley_terminal::dcs` (the fields, the unescaped split).
  - The Rerun check in `terminal_view`'s `marley_block` (`command_verified` and the last block
    finished); `marley_workbench::autosuggest::suggestion` (verified commands, then the history
    file); `mcp.rs`'s `block_entry`.
  - `marley_remote` parses targets and builds an `ssh` argv with a `--` guard, but nothing in the
    fork depends on it (only the workspace's member list names it), so the wrapper parses the
    user's own `ssh` arguments.
  - Nothing to adopt for the transfer: Zed's own remote development runs its `remote_server` on
    the host (report 04 §2.10; `crates/remote/src/transport/ssh.rs`, `build_command`), a program
    on the host again.

## UI proof
UI-AFFECTING. `script/e2e/526-blocks-over-ssh.sh` (`compositor sway`, for the hovers and clicks on
Rerun). A real `sshd` of the scenario's own, run as the user on `127.0.0.1` on a free port, with a
scratch host key, a scratch client key, `UsePAM no`, `StrictModes no` and `SetEnv
HOME=<scratch remote home>`, so no file of the user's is read or written; `Match LocalPort` blocks
give a second port a zsh and a third `/bin/sh`. The client config (`-F`) pins the host key. Shots:
`echo local` at the local prompt, then `ssh -F <config> e2e`, `echo on the far side` and `false`
(`526-01-remote-blocks`: the ssh block's bar ending at the remote prompt, a check and `exit 1`
pills); the pointer on the remote `false` block, Rerun shown, clicked (`526-02-remote-rerun`: a
new `false` block); the pointer on the local `echo local` block, no Rerun (`526-03-no-local-rerun`);
`cat forged.txt`, a frame with a wrong nonce, and the pointer on its block (`526-04-forged`); a
prefix of a local command typed at the remote prompt, no suggestion, and a prefix of a remote one,
suggested (`526-05-remote-suggestions`); `exit`, the pointer on the remote block, no Rerun, then on
the local one, Rerun (`526-06-back-local`); the zsh port (`526-07-zsh`); `command ssh -F <config>
e2e` and a host tagged `marley-plain`, no remote blocks (`526-08-plain`); the `/bin/sh` port, a
plain login (`526-09-other-shell`).
The run log: `ssh -F <config> e2e true`'s output as plain ssh gives it, the stand-in MCP
client's `terminal_blocks` with each block's `host`, and the scratch remote `/tmp` with no
`marley.*` directory left. #484's scenario runs again for local autosuggestions.

## Locked-In Decisions
- D1 — The integration rides in the ssh command. The `ssh` function adds `-t` and a POSIX sh
  bootstrap as the remote command, which writes Marley's scripts into a temporary directory and
  starts the login shell with them; the directory goes once the shell has read it, and nothing is
  installed on the host. Rejected: typing a bootstrap into the remote shell after login (it needs
  the remote prompt found by its text, and leaves the bootstrap on screen and in the remote
  history); a program on every host (Warp's extension, Orca's relay, Zed's `remote_server`).
- D2 — Only an interactive login is wrapped: no remote command, none of the flags that make a
  session non-interactive, terminals on stdin and stdout, and no `RemoteCommand` for the host.
  Everything else, `command ssh`, a user's own `ssh` function and a host tagged `marley-plain` in
  the user's ssh config run as the user wrote them; the tag is the per-host opt-out until the
  Marley page has a setting.
- D3 — A connection nonce, never the terminal's. The wrapper mints one per connection and
  announces it in a `remote` frame carrying the terminal's nonce, so output cannot announce one;
  the terminal accepts it until the local shell's next prompt. The remote command shows in the
  host's process list for the moment before `exec`, so the terminal's own nonce stays local, and
  a connection nonce seen there dies with the connection.
- D4 — Every frame carries its shell's nonce, the local ones included, so the terminal knows which
  shell waits at the prompt. PR-claude-474 foresaw it: an action on a `precmd` field adds the
  nonce to `precmd` first.
- D5 — Rerun and autosuggestions follow the shell at the prompt. A block is offered Rerun only
  while the shell that ran it waits at its prompt, since a remote command rerun locally, or a
  local one on the host, runs where it never ran; suggestions draw on that shell's commands, and
  a history file is read only for the local shell, since a remote shell's path names a file on
  the host.
- D6 — The `ssh` block ends at the remote shell's first frame, without an exit code: its output is
  the connection's banner. One block holding the session's blocks waits for stage two (T5).
- D7 — On the host, bash reads a login bash's files in place of `~/.bashrc`, since ssh gives a
  login shell and `--rcfile` makes bash skip them; zsh starts as a login shell with Marley's
  `ZDOTDIR`, which #465's script hands back.
- D8 — ssh first; `docker exec` next with the same bootstrap.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs `ssh` for an interactive login from Marley's bash or zsh to a host whose shell is bash or zsh, the system shall start that shell with Marley's integration, and each command typed there shall become a block with its exit code and output. | Shots `526-01-remote-blocks`, `526-07-zsh` |
| REQ-002 | WHEN the remote shell's first frame arrives, the system shall end the `ssh` block there without an exit code. | Shot `526-01-remote-blocks` |
| REQ-003 | WHILE the remote shell waits at its prompt, the system shall offer Rerun for a block the remote shell reported and for no block of the local shell. | Shots `526-02-remote-rerun`, `526-03-no-local-rerun` |
| REQ-004 | WHEN a frame that carries neither the terminal's nonce nor the connection's arrives, its block shall be offered no Rerun. | Shot `526-04-forged` |
| REQ-005 | WHILE the remote shell waits at its prompt, autosuggestions shall come from the commands that shell reported, and from no local history. | Shot `526-05-remote-suggestions` |
| REQ-006 | WHEN the connection ends and the local shell reports its prompt, the system shall stop accepting the connection's nonce: the remote blocks lose Rerun and the local blocks have it again. | Shot `526-06-back-local` |
| REQ-007 | WHEN `ssh` is given a remote command or a non-interactive flag, runs without terminals, targets a host whose configuration sets `RemoteCommand` or carries `Tag marley-plain`, or is run as `command ssh`, the system shall run ssh with the user's arguments only. | Shot `526-08-plain`; the run log |
| REQ-008 | WHERE the host's shell is neither bash nor zsh, or the host cannot make a temporary directory, the system shall start the host's login shell as plain ssh does. | Shot `526-09-other-shell` |
| REQ-009 | WHEN the remote shell has started, the bootstrap's temporary directory shall be gone from the host. | The run log |
| REQ-010 | WHEN an agent calls `terminal_blocks`, each block shall name the host its shell ran on, or none for the local shell. | The run log |
| REQ-011 | WHILE no ssh connection runs, local blocks, Rerun and autosuggestions shall behave as before. | Shot `526-06-back-local`; #484's scenario |
| REQ-012 | The diff gate shall be green. | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; the design and the test plan in the notes; the scripts proven under the
  real shells on a pseudo-terminal first (L-claude-463).
- **P2 Code** — the touchpoints rows first; the scripts and the bootstrap; the decoder and the block
  model; the Rerun hunk; autosuggestions and `terminal_blocks`; fmt and clippy clean; a review of
  the diff (security lens: no frame is trusted without a nonce the terminal gave or a verified
  frame announced, and the terminal's nonce never leaves the machine).
- **P3 Test** — write and run the scenario, read every shot; #484's scenario again;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley/three-prong-plan.md` (T0c), the `marley_terminal`
  notes under `docs/marley_architecture/` (§21), ledger capture (§19), close, archive, commit.
