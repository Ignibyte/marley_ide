---
pipeline_id: 6b193214-fae6-4d9b-8635-74834e3e2f2d
ticket: docs/planning/tickets/closed/TICKET-596-ssh-passphrases-asked-in-marley.md
status: Phase 4 — Complete PASS
title: "An agent's ssh asks for a key's passphrase in a Marley dialog"
type: feature
slice: prong 1 T7 (the agent CLIs Marley starts, #440, #450, #537)
references: [docs/planning/pipeline/completed/537-git-credential-prompts-off.spec.md]
---

## Title
The terminal Marley opens for an agent CLI gets `SSH_ASKPASS`, pointing at a helper script Marley
makes for that terminal, and `SSH_ASKPASS_REQUIRE=force`. When ssh in that terminal needs a key's
passphrase (or any other answer), the helper asks Marley over a private socket, and Marley shows
Zed's password dialog over the agent's terminal with the prompt ssh gave. The typed text goes back
to ssh only. Cancel gives ssh an empty answer, so the command fails at once. An agent can no
longer hang on a prompt it cannot answer, and the user types the passphrase without leaving
Marley (Chad, 2026-09-26: "a way the user can type it in, preferably").

## Scope
### In
- **The variables.** Agent terminals get `SSH_ASKPASS=<the terminal's helper script>` and
  `SSH_ASKPASS_REQUIRE=force` (ssh(1): force uses the askpass program even with a terminal),
  beside #537's `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never`. #537's list gains
  `GIT_ASKPASS=` (empty): git falls back to `SSH_ASKPASS` for its own prompts (gitcredentials(7)
  step 3, checked on this box), and an empty `GIT_ASKPASS` stops that, so git's HTTP prompts still
  fail at once as #537 decided, and only ssh asks through Marley.
- **Which terminals.** Every terminal `agents::start_in_terminal` opens for an agent: the rail's
  Agent CLIs, the New Agent picker's CLI entries, #510's worktree agents and #527's launch
  configs' Agent items. A launch config's Terminal item, New Terminal, the Playwright script's
  terminal and a restored terminal get none. A remote project's terminal gets none: its ssh runs
  on the other machine, where the helper's path means nothing.
- **The helper.** Zed's own: `askpass::PasswordProxy` makes a 0700 temporary folder holding a
  socket and `askpass.sh`, which runs Marley's executable as `marley --askpass=<socket>` (the
  hidden mode `crates/zed/src/main.rs` already has). One proxy per agent terminal, alive exactly
  as long as the terminal: it goes when the terminal entity is released, and its folder with it.
- **The dialog.** Zed's `git_ui_core::askpass_modal::AskPassModal`: a header, ssh's prompt, and
  one line whose text is masked unless the prompt is a yes/no question. Marley brings the agent's
  terminal to the front first (`browser::reveal_terminal`), then opens the dialog in the
  workspace that holds it, headed `ssh for <agent> in <project>`.
- **Cancel.** The dialog's Cancel (Escape) drops the answer; the proxy closes the connection
  without writing, the helper prints nothing, and ssh reads an empty passphrase, skips the key
  and fails (`Permission denied`, git exits 128) with no further wait.
- **The helper program at start.** `askpass::set_askpass_program` is called once, when Marley
  starts, with its own executable, so an install that replaces the file while Marley runs does
  not leave later helpers pointing at `marley (deleted)`, as #583 found for the relay.

### Out (explicitly deferred)
- Plain terminals, and an agent typed by hand at a plain shell's prompt: #537's line stands.
- Git's own HTTP credential prompts through the dialog: #537 decided they fail at once, and
  `GIT_ASKPASS=` keeps that. A later ticket can route them through the same proxy if Chad wants
  it.
- The GPG signing wrapper the proxy also makes: Zed's git panel uses it through `gpg.program`;
  agent terminals do not set it.
- Proving which process asked. Any process in the agent's terminal can run `$SSH_ASKPASS` with a
  prompt of its own choosing, as with every askpass program; the dialog names the terminal and
  shows the prompt verbatim, and the user decides. Checking the asker's parent with
  `SO_PEERCRED` is a possible later hardening.
- Zed's Agent Panel threads and ACP agents, and #540's resumed sessions in restored terminals
  (both get none today, as with #537).

## Reference (§20)
Upstream Zed: its git operations already ask this way. `crates/git/src/repository.rs:4055` wraps
a push, pull or fetch in an `AskPassSession` and sets `GIT_ASKPASS`, `SSH_ASKPASS` and
`SSH_ASKPASS_REQUIRE=force` on git's process; `crates/git_ui/src/git_panel.rs:4669`
(`askpass_delegate`) opens `AskPassModal` in the workspace for each prompt. Marley keeps the
helper, the socket, the hidden `--askpass` mode and the dialog as they are, and applies them to a
terminal's environment instead of one git process. Where Zed's session keeps ssh waiting on a
cancel and kills git itself, Marley does not own the ssh process, so a cancel closes the
connection instead. Orca (MIT, `1c2cf120e3`) is the other reference for the variables:
`src/shared/git-credential-prompt-env.ts:93` sets `GIT_ASKPASS` and `SSH_ASKPASS` empty so its
own git calls cannot prompt, and `src/shared/terminal-git-credential-guard.ts:48` leaves both
alone in agent terminals. Marley takes the empty `GIT_ASKPASS` and, where Orca switches ssh's
prompt off, asks for it instead. Warp: N/A, no such item in the once-over.

### Prior art
- **Behavior maps.** `docs/orca_architecture/02-worktrees-and-review.md` §2.2 and §3 item 8 (the
  credential guard in agent terminals, #537's source). `docs/zed_architecture/` has no entry on
  askpass. `docs/warp_architecture/`: nothing on ssh prompts.
- **Published material.** ssh(1), `SSH_ASKPASS` and `SSH_ASKPASS_REQUIRE` (force, prefer,
  never); OpenSSH's `readpass.c` behavior, observed on this box (OpenSSH 10.5p1): an askpass
  answer that is empty, or a helper that exits non-zero, gives an empty passphrase, and ssh
  skips the key. gitcredentials(7): `GIT_ASKPASS`, then `core.askPass`, then `SSH_ASKPASS`, then
  the terminal; checked here: with `SSH_ASKPASS` set and `GIT_ASKPASS` unset, `git credential
  fill` asked the askpass program for a username and password, and with `GIT_ASKPASS=` it failed
  with `terminal prompts disabled`.
- **Code we already ship.** The whole seam is Zed's:
  - `crates/askpass/src/askpass.rs`: `PasswordProxy::new` (267) binds the socket inside a spawned
    task and writes `askpass.sh` (`generate_askpass_script`, 480); `AskPassDelegate::
    new_with_cancellation` (70) runs each prompt on the foreground; `askpass::main` (414) is the
    helper's side; `set_askpass_program` (471) fixes the executable; `EncryptedPassword`
    (`encrypted_password.rs`) zeroizes on drop.
  - `crates/zed/src/main.rs:225`: `marley --askpass=<socket>` runs `askpass::main` and returns
    before any app starts.
  - `crates/git_ui_core/src/askpass_modal.rs`: `AskPassModal::new(operation, prompt, tx,
    cancellation, window, cx)`, masked unless `yes/no` or `Username`; Cancel drops `tx`; a
    dropped cancellation sender closes it.
  - Marley's own: `agents::start_in_terminal` (`crates/marley_workbench/src/agents.rs:339`),
    `browser::reveal_terminal` (`browser.rs:7696`), `mcp::terminals` (`mcp.rs:719`),
    `system_one::project_of` and `project_name` (`system_one.rs:1097`),
    `marley_browser::service::socket_fits` (`service.rs:279`).
  No Zed crate changes: Marley calls these as they are.

## UI proof
UI-AFFECTING (a new dialog over an agent's terminal, and what that terminal shows after it).
`script/e2e/596-ssh-passphrases-asked-in-marley.sh` (`compositor sway`, keys only, for a seat
nothing else sees while a passphrase is typed). Setup: a user-level `sshd` on a free loopback
port with its own host key and `authorized_keys`, a bare repository it serves, a
passphrase-protected ed25519 key whose passphrase is made at run time, a known-hosts file
holding the server's key; stand-ins `claude` and `codex` first on the PATH that print the
variables, show git's HTTP prompt still failing, push over ssh with that key, print the exit
status and wait; `push-over-ssh`, the same push for a plain terminal. Steps: Claude Code from the
project's + in the rail, the dialog read (`596-01-dialog`), the passphrase typed (`596-02-typed`),
Enter and the push through (`596-03-pushed`); Codex from the picker, the dialog (`596-04-codex-dialog`), Escape, the push
failed (`596-05-cancelled`); a New Terminal where `push-over-ssh` waits on ssh's own prompt
(`596-06-plain-terminal`), then Ctrl+C. The run log holds `sshd`'s log and a search of Marley's
log and the terminals' screens for the passphrase.

## Locked-In Decisions
- D1: Zed's askpass as it is: `PasswordProxy`, the `--askpass` mode and `AskPassModal`, with no
  Zed crate changed. A proxy per agent terminal, so each prompt knows its terminal by the socket
  it came in on, and ssh's prompts in one terminal come one at a time (the proxy serves one
  connection before the next).
- D2: Agent terminals of local projects only (#537's rule), for the terminal's life. The proxy
  lives in a release callback of the terminal entity.
- D3: `GIT_ASKPASS=` joins #537's list, so git's own prompts keep failing at once and only ssh
  asks through Marley.
- D4: Cancel closes the connection unanswered; ssh reads an empty passphrase and fails at once.
  Zed's `AskPassSession`, which holds the connection open until its caller kills the command, is
  not used.
- D5: The dialog opens over the agent's terminal: its tab is brought to the front in the
  workspace that holds it, so the user sees what asks before typing a secret. It is headed
  `ssh for <agent> in <project>`, the prompt below it verbatim. When a password dialog is already
  open in that workspace, a new request is refused (logged; ssh reads an empty answer) rather than
  toggled, since Zed's `toggle_modal` would close the open one and cancel both.
- D6: A proxy whose socket path would not fit `sun_path` is dropped, and that terminal gets no
  `SSH_ASKPASS` (ssh asks on the terminal, as before), with the reason logged. An error making
  the proxy reaches the launch's prompt (`Could not start the agent`).
- D7: The helper program is fixed when Marley starts (`askpass::set_askpass_program`), once per
  process.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts an agent CLI in a local project, the agent's terminal shall hold `SSH_ASKPASS` naming a helper script, `SSH_ASKPASS_REQUIRE=force` and an empty `GIT_ASKPASS`, beside #537's two variables. | Shot `596-01-dialog` (the stand-in prints them) |
| REQ-002 | WHEN ssh in an agent's terminal asks for a key's passphrase, Marley shall show a dialog over that terminal headed with the agent and the project, showing the prompt ssh gave, with the typed text masked. | Shots `596-01-dialog`, `596-02-typed` |
| REQ-003 | WHEN the user types the passphrase and confirms, ssh shall receive it and the push shall succeed. | Shot `596-03-pushed` (`push exited 0`, the new branch); `sshd`'s `Accepted publickey` |
| REQ-004 | WHEN the dialog is answered, the passphrase shall appear in no terminal's screen and in no log line of Marley's. | The scenario's search of the terminals' screens and Marley's log (run log) |
| REQ-005 | WHEN the user cancels the dialog, the ssh command shall fail at once, with no prompt left waiting in the terminal. | Shot `596-05-cancelled` (`Permission denied`, `push exited 128`) |
| REQ-006 | WHILE an agent's terminal holds `SSH_ASKPASS`, git's own credential prompts shall still fail at once with `terminal prompts disabled`, and no dialog shall open for them. | Shot `596-01-dialog` (the stand-in's `git credential fill` line, printed before its push) |
| REQ-007 | WHERE a terminal was not opened for an agent CLI, ssh shall ask for the passphrase on the terminal as before. | Shot `596-06-plain-terminal` |

## Phase Plan
- **P1 Plan:** this spec, the design in the notes.
- **P2 Code:** `GIT_PROMPTS_OFF` gains `GIT_ASKPASS`; `marley_agent::ssh_dialog_title`;
  `askpass` in `marley_workbench`'s manifest; the proxy, the dialog and the variables in
  `agents.rs`; `start_in_terminal` takes the agent instead of the variables; `launch.rs` passes
  it; `set_askpass_program` at init; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test:** write and run the scenario, read every shot.
- **P4 Complete:** CHANGELOG; prong 1's T7 row; `docs/marley_architecture/marley_workbench.md`
  (Agent CLIs); the ledger; close, archive, commit.
