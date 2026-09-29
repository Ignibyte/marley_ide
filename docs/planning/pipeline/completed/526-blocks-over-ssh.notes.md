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

### Changed at promotion (2026-09-29; each item overrides the design below)
- **Checklist** (no task tool): pre-flight ✓ (no other active pipeline); recall ✓; the brain ✓
  (consultation f2398555666d48b08f4f4cac344ca3d3: nothing on this seam); promoted ✓; the seams re-verified by an
  Explore agent at a2df981b80 ✓ (line numbers moved; five gaps found, below).
- **The remote command is one line with no backslash, `!` or newline**, so a login shell that is
  fish or tcsh parses it as sh does: `sh -c 'b=$(printf %s <base64> | base64 -d 2>/dev/null) &&
  eval "$b" || exec "${SHELL:-/bin/sh}" -l' marley <connection nonce>`, the bootstrap base64 inside.
  `install_in` writes it to `ssh-remote-command`, and `for_program` names that file in
  `MARLEY_SSH_COMMAND`, which the scripts read and unset as they do the nonce. A host without
  `base64` starts its login shell plain.
- **The bootstrap's folder goes at the top of each script** on the host (bash reads its rcfile
  whole; zsh keeps its open `.zshenv` readable after the unlink), so a profile that runs `exec
  tmux` or `exit` still leaves nothing (REQ-009).
- **zsh defines `ssh` at the first prompt** (`__marley_install`), after the user's files, so a user's
  own `ssh` function is seen.
- **The nonce travels in a `Signed` hook**, which `decode_hook` wraps around any frame but `preexec`
  that carries `nonce=`, so no existing hook's fields change; `Remote { host, session }` is the new
  hook. `apply.rs` (the gpui-era path) unwraps `Signed` and ignores `Remote`.
- **Frames from a shell started before the update carry no nonce on `precmd`**, so its prompt's
  shell is unknown and it offers no Rerun until it restarts.

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

### For the quality pass
- No tests (§7, since 2026-09-29): the drafted scenario (a local sshd on loopback ports) waits
  for the quality pass.

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

## Phase 2 — Code
- **Checklist** (no task tool): the ledger row (`terminal_element.rs`) ✓; the scripts ✓; the
  bootstrap and its command file ✓; `dcs.rs` ✓; the block model ✓; the gpui-era readers ✓; the
  element's Rerun ✓; autosuggestions ✓; `terminal_blocks`' `host` ✓; the review ✓; the gate ✓.
- **Built as the changes at promotion say.** Hosts are kept beside the blocks (`hosts`, by
  index, as `times` is) rather than in `AnchoredBlock`, whose literals in Zed's crates' tests would
  all have changed. Clippy asked for `PromptShell` in place of `Option<Option<&str>>`,
  `name_and_fields` out of `decode_hook` (past 100 lines), and shorter first doc paragraphs.
- **Found and fixed before the gate: ssh reading the host's config as its stdin.** The first bash
  wrapper ran `command ssh` inside the `while read` loop over `ssh -G`'s output, so an interactive
  ssh started there would have read the here-string instead of the terminal; shellcheck (SC2095)
  named it. The loop now only decides, and ssh runs after it, in both scripts. An F-block below.
- **Checked by hand, no test:** `shellcheck` on `marley.bash` (gate:11's flags), `bash -n` and
  `zsh -n` on both scripts, `sh -n` on a replica of the bootstrap; the remote command is one line of
  about 22 KB with no backslash, `!` or newline.
- **The review**, against each criterion: the local shell's frames now all carry its nonce, so
  local blocks, Rerun and suggestions behave as before (REQ-011); a shell started before the update
  signs no `precmd`, so its prompt's shell is unknown and it offers no Rerun until it restarts;
  only the local shell opens a connection, so output cannot announce one (REQ-004); the local
  `precmd` ends it (REQ-006); the host's `init` ends the `ssh` block without an exit code
  (REQ-002); the plain cases run `command ssh "$@"` untouched (REQ-007, REQ-008).
- **The gate:** `just gate-diff` green: 16 passed, 0 failed, `GATE GREEN [diff]`, the receipt
  written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md`; `docs/marley_architecture/terminal_blocks.md` (blocks over
  ssh); `docs/marley/guide.md` ("Blocks over ssh"). The `terminal_element.rs` row names the
  per-block Rerun.
- **Knowledge:** F-claude-526-the-ssh-wrapper-ran-ssh-inside-a-loop-reading-its-config-001,
  AD-claude-526-blocks-over-ssh-ride-in-the-ssh-command-with-a-connection-nonce-001.
- **Brain:** consultation f2398555666d48b08f4f4cac344ca3d3 closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-526 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7): the drafted scenario (a local sshd) waits for the quality pass.

---
## Phase 3 — Test (the visual check, after the fact)
- **Why after:** #526 shipped (dd9fa55a1f) while the workflow had no visual check; Chad brought it
  back the same day (7e589cb0a1), and this ticket was checked before its release install.
- **The scenario:** `script/e2e/526-blocks-over-ssh.sh`, `compositor sway`. A stand-in `ssh`
  written in C, first on the PATH: `-G` prints a plain config, anything else runs its last argument
  on a pty of its own (`forkpty`) in a "far" HOME whose `.bash_profile` sets `far$ `, and relays the
  terminal to it, staying in the foreground as ssh does. The real bootstrap, the host's scripts,
  the connection's nonce and the terminal's two shells run; only the network and sshd do not.
  Three runs; the stand-in was first a shell script that exec'd the far shell, which hid the
  title's problem (below).
- **Checks (all pass):** local blocks verified with no host; `echo on the far side` and `false`
  verified blocks of `far` with their exit codes; the `ssh far` block ended with no exit code; the
  bootstrap's folder empty on the far side; ssh given `-t far sh -c 'b=…`; after `exit`, a local
  block again.
- **The shots, read:** `526-01-local` (the local blocks' bars and pills); `526-02-connected` (the
  `ssh far` block ends and the `far$ ` prompt from the login files); `526-03-far-blocks` (far
  blocks with ✓ and exit 1 pills, `ls -A $TMPDIR` empty, the tab and rail row titled `repo — ssh
  far`); `526-03b-far-rerun` (the pointer on the far `false`: Copy and Rerun); `526-03c-local-no-
  rerun` (the pointer on the local `echo local` while the far shell waits: Copy only);
  `526-04-back` (after `exit`, the local prompt and `echo back home` as a block).
- **Found and fixed: the title showed the bootstrap.** The terminal's title lists the foreground
  process's arguments, and for Marley's ssh that was `ssh -t far sh -c '<22 KB of base64>' marley
  <connection nonce>`: the tab and the rail row full of base64, and the connection's nonce on
  screen. `shown_arguments` now drops the remote command Marley added (known by its start,
  `SSH_COMMAND_START`) and the `-t` before it, so the title reads `ssh far`. An F-block.
- **Not reached:** a real sshd and network; a host whose login shell is fish or tcsh (the command's
  one-line form is what covers it).
