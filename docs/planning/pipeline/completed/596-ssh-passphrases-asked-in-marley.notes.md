# An agent's ssh asks for a key's passphrase in a Marley dialog — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-596-ssh-passphrases-asked-in-marley.md
- **Pipeline spec:** 596-ssh-passphrases-asked-in-marley.spec.md

## Phase 1 — Plan
- **Request:** Chad's answer on #537 (2026-09-26): "passphrases have always been a pain so either
  the agents need passphrase-less or a way the user can type it in, preferably". The top of the
  Queue on 2026-09-29; Chad asked for the last two unheld tickets to be finished that night.
- **Classification / tier:** feature, M. Marley crates only (`marley_agent`, `marley_workbench`);
  no Zed crate changes, so no touchpoint row.
- **Pre-flight:** cargo 1.98.1, gate, e2e and shear present; hooks wired; no active pipeline;
  README marker present; cargo idle; `/mnt/fast` at 90% (101G free).
- **Recall (§18.3):**
  - #537 (completed): agent terminals get their variables at spawn through
    `Project::create_terminal_shell_with_env` and `agents::start_in_terminal`; its notes name
    this ticket as the ssh half and say the box has no `SSH_ASKPASS` or `core.askPass`.
  - AD-claude-583-chromium-on-its-pipe-behind-marleys-relay-001: Zed's hidden modes
    (`--askpass`) are the model for running Marley's own executable as a helper, and
    `relay_executable` exists because `current_exe` reads `marley (deleted)` after an install
    replaced the file (D7 here).
  - PR-claude-check-a-unix-socket-path-against-sun-path-001: a socket under a folder the user
    chooses (`$TMPDIR` here) is checked against `sun_path` before use (D6).
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001: the scenario's passphrase is made
    at run time and its checks read the run's own files.
  - Brain (consultation eb40743a930f4cb79b1b3b6ed58a89f7): nothing on this seam; only unrelated
    due follow-ups came back.
- **Discovery (opened and checked):**
  - Zed's askpass wiring (an Explore pass, then read): `crates/askpass/src/askpass.rs`
    (`PasswordProxy::new` 267 to 396: the tempdir, the socket bound inside `_task`, `askpass.sh`
    written after; the accept loop serves one connection at a time, writes the answer on
    `Continue(Ok)`, writes nothing and drops the stream on `Continue(Err)`, holds it open forever
    on `Break`), `AskPassDelegate::new_with_cancellation` (70), `ask_password` (94: `None` when the
    dialog drops its sender), `set_askpass_program` (471, a `OnceLock`, `debug_panic` on a second
    set), `generate_askpass_script` (480).
  - `crates/zed/src/main.rs:225`: `--askpass` returns before any app starts; before it,
    `keep_activation_token` only reads an environment variable and the relay check only looks at
    the arguments, so a helper run leaves the running Marley alone. The binary is `marley`
    (`crates/zed/Cargo.toml:57`).
  - `crates/git/src/repository.rs:4055`: Zed's git sets `GIT_ASKPASS`, `SSH_ASKPASS` and
    `SSH_ASKPASS_REQUIRE=force`. `crates/git_ui/src/git_panel.rs:4669`: `askpass_delegate`, the
    pattern for opening the dialog (three private copies in Zed; Marley needs its own).
  - `crates/git_ui_core/src/askpass_modal.rs`: `AskPassModal` is `pub`; `marley_workbench`
    already depends on `git_ui_core`. `askpass` itself is not yet a dependency.
  - `crates/workspace/src/workspace.rs:8613`: `toggle_modal` hides an open modal of the same type
    instead of opening a second (D5); `active_modal::<V>` (8602) tells.
  - Marley: `agents.rs` (`start_in_terminal` 339, `agent_env` 328, `start_cli_with_prompt` 288),
    `launch.rs:437` (the Agent item's variables), `browser::reveal_terminal` (7696),
    `mcp::terminals` (719), `system_one::project_of`/`project_name` (1097),
    `marley_browser::service::socket_fits` (279), `marley_agent::GIT_PROMPTS_OFF` (146),
    `AgentKind::display_name` (83), `event_line` (530, which strips control characters from the
    project's name).
  - On this box: OpenSSH 10.5p1, `/usr/bin/sshd` present. A user-level `sshd` on a loopback port
    (own host key, `AuthorizedKeysFile` and `PidFile` in the scratch folder, `UsePAM no`,
    `StrictModes no`) served a `git push` over ssh; with `SSH_ASKPASS_REQUIRE=force` ssh ran the
    askpass program with `Enter passphrase for key '<path>': ` and pushed with its answer; with
    an empty answer the push failed with exit 128 at once. With `SSH_ASKPASS` set and
    `GIT_ASKPASS` unset, `git credential fill` asked the askpass program for a username and a
    password; with `GIT_ASKPASS=` it failed with `terminal prompts disabled` (D3).
    `$TMPDIR` is `/mnt/fast/tmp`, so the proxy's socket path is about 44 bytes.
- **Decisions:** D1 to D7 in the spec.

### Design
- **`crates/marley_agent/src/marley_agent.rs`** (Marley).
  - `GIT_PROMPTS_OFF` becomes three entries, `("GIT_ASKPASS", "")` added, its doc saying why: an
    empty `GIT_ASKPASS` keeps git from falling back to `SSH_ASKPASS` (#596).
  - `pub fn ssh_dialog_title(project: &str, kind: AgentKind) -> String`: `ssh for <display name>
    in <project>`, the project's control characters stripped as `event_line` does.
- **`crates/marley_workbench/Cargo.toml`** (Marley): `askpass.workspace = true`.
- **`crates/marley_workbench/src/agents.rs`** (Marley).
  - `start_in_terminal(workspace, directory, agent: Option<AgentKind>, input, window, cx)`: the
    variables are worked out inside. For an agent in a local project it first makes the proxy,
    then opens the terminal through `add_center_terminal` from its spawned task, then moves the
    proxy into `cx.on_release` of the new terminal (so it drops, and its folder goes, when the
    terminal does) and records the terminal in the proxy's shared slot. `None` gives no variables,
    as New Terminal's and the Terminal items' calls do today.
  - `passphrase_proxy(window, title, terminal_slot, cx) -> anyhow::Result<Option<PasswordProxy>>`:
    an `AskPassDelegate::new_with_cancellation` whose callback, on the foreground, finds the
    terminal's view through `mcp::terminals`, reveals it, and opens `AskPassModal` in its
    workspace (refusing when one is open there, D5); then `PasswordProxy::new` with a callback
    that turns the dialog's answer into `Continue(Ok)` and its absence into `Continue(Err)`
    (D4). `None` when the socket would not fit (D6), logged.
  - `agent_env(script)`: #537's list plus `SSH_ASKPASS` and `SSH_ASKPASS_REQUIRE=force`; the
    script path must be UTF-8 (an error otherwise).
  - `start_cli_with_prompt` passes `Some(kind)`.
- **`crates/marley_workbench/src/launch.rs`** (Marley): passes `Some(kind)` for an Agent item and
  `None` otherwise, instead of the variables.
- **`crates/marley_workbench/src/marley_workbench.rs`** (Marley) `init`: sets the askpass program
  to Marley's executable once per process (a `std::sync::Once`, since the tests call `init`
  more than once and a second set is a `debug_panic`), D7.
- **Ledger rows.** No touchpoint row (no Zed path changes). At Complete: an AD for the proxy per
  agent terminal and the cancel semantics (D1, D4), and the empty `GIT_ASKPASS` (D3).

### Visual check plan
`script/e2e/596-ssh-passphrases-asked-in-marley.sh`, `compositor sway`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| — | setup: `$E2E_WORK/ssh`: `ssh-keygen` host key and an ed25519 agent key with a passphrase built at run time and written to `$E2E_WORK/passphrase`; `authorized_keys`; `sshd_config` (loopback, free port, `UsePAM no`, `StrictModes no`, publickey only); `/usr/bin/sshd -D -E <log>` under `timeout 900`; `known_hosts` with `[127.0.0.1]:<port>`; a bare `remote.git`; `repo` with one commit; the HOME's `.bashrc` with `PS1` and the stand-ins' folder first on PATH | — |
| REQ-001, REQ-006 | stand-ins `claude` and `codex`: print `SSH_ASKPASS` set or unset, `SSH_ASKPASS_REQUIRE=…`, `GIT_ASKPASS=<…>`, #537's two; run `git -c credential.helper= credential fill` for `https://example.invalid` (fails); then `git push` over ssh with `GIT_SSH_COMMAND` naming the key, `IdentitiesOnly=yes`, `IdentityAgent=none` and the known-hosts file, to `HEAD:<name>`; print `push exited <status>`; `exec -a <name> sleep 600` | `596-01-dialog` (the lines above the dialog) |
| REQ-002 | Ctrl+Alt+N, `Claude Code`, Return; settle | `596-01-dialog`: the dialog over Claude Code's terminal, `ssh for Claude Code in repo`, `Enter passphrase for key '…agent_key':` |
| REQ-002 | type the passphrase from `$E2E_WORK/passphrase` | `596-02-typed`: dots only |
| REQ-003, REQ-004 | Return; settle | `596-03-pushed`: `* [new branch] HEAD -> claude`, `push exited 0`; `sshd`'s `Accepted publickey`; the screen and Marley's log searched for the passphrase |
| REQ-005 | Ctrl+Alt+N, `Codex`, Return; settle; Escape; settle | `596-04-cancelled`: `Permission denied (publickey)`, `push exited 128` |
| REQ-007 | palette `workspace: new terminal`; type `push-by-hand`; Return | `596-05-plain-terminal`: `Enter passphrase for key '…':` in the terminal, no dialog; Ctrl+C |

Not reached by a scenario: a real remote (no network in a run); a host-key question and password
authentication (the same dialog, other prompts; the fixture's known-hosts file and publickey-only
server skip them); an install replacing Marley's executable during a run (D7 is read in the diff).

### Risks
- A dialog that outlives its ssh: if the user interrupts ssh in the terminal while the dialog is
  open, nothing tells the proxy, and the dialog stays until answered or dismissed; its answer
  goes to a closed connection (the write's error is logged, never the value). Closing the
  terminal does close it: the proxy drops, its pending request with it, and the dialog's
  cancellation fires.
- Password authentication asks up to `NumberOfPasswordPrompts` (3) times, so a server that
  offers it shows the dialog again after a cancel. Key passphrases, the ticket's case, ask once.
- Two agents in one workspace asking at the same moment: the second is refused (D5) and its ssh
  fails; the agent can retry. Prompts from one terminal queue in its proxy.
- `start_in_terminal` now opens the terminal from its task instead of synchronously; the tests in
  the tree that start an agent and look for its terminal before the executor runs would need a
  `run_until_parked`. No gate runs them (§7); gate:2 builds them.
- The socket name `askpass.sock` in D6's check is Zed's private choice; a comment says so, and an
  upstream rename would only make the check pass or fail on the wrong length.

## Phase 2 — Code
- **Built:**
  - `marley_agent.rs`: `GIT_PROMPTS_OFF` gains `("GIT_ASKPASS", "")`, its doc saying why;
    `ssh_dialog_title(project, kind)`, `ssh for <agent> in <project>`.
  - `marley_workbench/Cargo.toml`: `askpass.workspace = true`.
  - `agents.rs`: `start_in_terminal` takes `agent: Option<AgentKind>` instead of the variables,
    makes the proxy first for an agent in a local project, opens the terminal from its task, and
    moves the proxy into the terminal's `on_release`; `passphrase_proxy` (the delegate, the
    proxy, the `sun_path` check); `ask_passphrase` (finds the terminal in the window's
    workspaces through `mcp::terminals`, refuses when a password dialog is open there, reveals the
    terminal, opens `AskPassModal`); `agent_env(script)`; `fix_askpass_program` in `init`.
  - `launch.rs`: an Agent item passes its kind, anything else `None`.
  - `script/e2e/596-ssh-passphrases-asked-in-marley.sh`, written now so one gate run covers
    it (the receipt binds the scenarios).
- **Deviations:** `start_cli` and `start_cli_with_prompt` take `&Workspace`, `&Window` and
  `&Context<Workspace>` now: all their work moved into a task, and clippy's
  `needless_pass_by_ref_mut` asked for it (their callers pass `&mut`, which coerces). D7 lives in
  `agents::init`, behind a `std::sync::Once`, rather than in `crate::init`.
- **Review of the diff:**
  - REQ-001: the four variables and `GIT_ASKPASS` come from `agent_env` for every
    `start_in_terminal` call with an agent; a remote project gets #537's list without the
    askpass pair.
  - REQ-002: the dialog is Zed's, masked unless the prompt is a yes/no or Username question;
    the heading is `ssh_dialog_title`; the terminal is revealed before the dialog opens.
  - REQ-003 and REQ-004: the answer travels as `EncryptedPassword` to the proxy, which writes it
    to ssh's connection only; nothing in the new code logs a prompt or an answer (each
    `log::warn!` is a fixed sentence or a path).
  - REQ-005: a dismissed dialog drops its sender, `ask_password` gives `None`, the proxy gets
    `Continue(Err)`, writes nothing and drops the connection.
  - REQ-006: `GIT_ASKPASS=` is in the list every agent terminal gets.
  - REQ-007: New Terminal, the Playwright terminal and a launch config's Terminal item never
    reach `agent_env`.
  - Re-entrancy: `ask_passphrase` reads the workspaces, then `reveal_terminal` updates the
    multi-workspace and a pane, then `workspace.update` opens the modal; no update nests in
    another on the same entity. The release callback only drops the proxy.
  - Errors: a proxy that cannot be made fails the launch through its prompt (`Could not start
    the agent`); a socket that would not fit leaves ssh on the terminal and logs why.
  - Provenance: Marley calls Zed's public askpass and modal API; the few lines that open the
    modal follow the only way the API allows (the pattern `git_panel.rs` uses), no Zed body is
    carried over. No Zed path changed, so no touchpoint row.
- **Gate:** run 1 red: gate:2, four clippy findings in the new code (`clone_on_ref_ptr` on the
  `Arc`, `needless_pass_by_ref_mut` on `start_in_terminal`), and the receipt, since the fix
  landed while the run went on. Run 2 red: gate:2, the same lint on `start_cli_with_prompt`.
  Both fixed at the source. Run 3: GATE GREEN [diff].

## Phase 3 — Test
- **Scenario:** `script/e2e/596-ssh-passphrases-asked-in-marley.sh` under `compositor sway`: a
  user-level `sshd` on a free loopback port serving a bare repository to an ed25519 key whose
  passphrase is made at run time; stand-ins `claude` and `codex` first on the terminals' PATH
  (checked before any launch) that print the askpass variables, run `git credential fill` for an
  https host, push over ssh with that key (`-F /dev/null`, no ssh agent, the fixture's known-hosts
  file), print the exit status and wait; `push-over-ssh` for the plain terminal.
- **Run 1 red (the change):** the stand-in printed the variables and git's prompt failed at once,
  but no dialog opened; ssh read an empty answer and the push failed. Marley's log: `ssh asked for
  a passphrase in a terminal that has closed`. `ask_passphrase` looked the terminal up with
  `mcp::terminals` inside `window.update`, and `mcp::terminals` reads every window through its
  handle, where the window being updated reads as gone, so the agent's own window held no
  terminals. Fixed: the lookup runs before the update, and the update only checks that the
  terminal's workspace is one of this window's. Rebuilt.
- **Run 2 red (the scenario):** every check of the Claude Code half passed; the Codex half's check
  looked for `$ codex` in the MCP read of the screen, which starts at git's first line (the shot
  shows the typed line). The check now asks for `Permission denied` and `push exited 128` only.
- **Run 3:** exit 0, every check passes. Every shot read:
  - `596-00-menu`: the project's + menu, Claude Code selected under Agent CLIs.
  - `596-01-dialog` (REQ-001, REQ-002, REQ-006): the dialog at the top of the window headed
    `ssh for Claude Code in repo`, the body `Enter passphrase for key '<path>':` (ssh cuts the
    path at 100 characters), an empty line with the cursor; below, the agent's terminal in front
    with `$ claude`, `SSH_ASKPASS=askpass.sh SSH_ASKPASS_REQUIRE=force`,
    `GIT_ASKPASS=[] GIT_TERMINAL_PROMPT=0`, and `fatal: could not read Username for
    'https://example.invalid': terminal prompts disabled`; no ssh prompt in the terminal.
  - `596-02-typed` (REQ-002): the same dialog, the line holding asterisks only.
  - `596-03-pushed` (REQ-003, REQ-004): the dialog gone; the terminal shows git's transfer,
    `* [new branch] HEAD -> claude` and `push exited 0`. `sshd`'s log: `Accepted publickey`. The
    checks found the passphrase in neither the terminal's screen nor Marley's log.
  - `596-04-codex-dialog` (REQ-002): `ssh for Codex in repo` over Codex's terminal, which shows the
    same variables.
  - `596-05-cancelled` (REQ-005): after Escape the dialog is gone and the terminal shows
    `<user>@127.0.0.1: Permission denied (publickey).` and `push exited 128`; `sshd`'s log shows
    the connection closed before authentication.
  - `596-06-plain-terminal` (REQ-007): a New Terminal where `push-over-ssh plain` waits on ssh's
    own `Enter passphrase for key '<path>':` on the terminal, no dialog; its rail row reads
    `waiting for a password`. Ctrl+C ended it.
  - Pre-existing, not in scope: Ctrl+Alt+N from a terminal shows Marley's toast that the
    program did not get the key, as it did before this change.
- **Focus report:** the run was in its own headless sway; Hyprland had 0 Marley windows before
  and after, and the run added no rule.
- **Not reached:** a real remote (no network in a run); a host-key question and password
  authentication (other prompts through the same dialog; the fixture's known-hosts file and a
  publickey-only server skip them); an install that replaces Marley's executable during a run
  (D7, read in the diff); a remote project's agent terminal (no remote host in a run; the code
  gives it #537's list alone).
- **Gate after the fix:** GATE GREEN [diff] (run 4, over the fixed `agents.rs` and the scenario).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley/three-prong-plan.md` T7;
  `docs/marley_architecture/marley_workbench.md` (Agent CLIs: ssh's passphrases);
  `docs/marley_architecture/marley_agent.md` (`GIT_PROMPTS_OFF`, `ssh_dialog_title`). No Zed path
  changed, so no touchpoint row to check.
- **Knowledge:** `F-claude-596-a-lookup-inside-a-window-update-lost-that-windows-terminals-001`,
  `PR-claude-596-look-up-across-windows-before-updating-one-001`,
  `AD-claude-596-an-agent-terminals-ssh-asks-through-zeds-askpass-001`. Brain: the decision on
  consultation eb40743a (`decisions/marley-an-agent-terminals-ssh-asks-for-passphrases-through-zeds-askpass-dialog`),
  follow-up by 2026-10-29.
- **Closed** TICKET-596, archived the pair.
