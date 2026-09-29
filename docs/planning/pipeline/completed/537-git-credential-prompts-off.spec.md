---
pipeline_id: a56e8438-e73a-4470-b16c-8f9ed22f8224
ticket: docs/planning/tickets/open/TICKET-537-git-credential-prompts-off.md
status: Phase 4 — Complete PASS
title: "Git credential prompts off in the terminals Marley opens for agent CLIs"
type: feature
slice: prong 1 T7 (the agent CLIs Marley starts, #440 and #450), before #510's worktree agents
references: [docs/planning/pipeline/completed/440-rail-agent-clis.spec.md, docs/orca_architecture/02-worktrees-and-review.md]
---

## Title
The terminal Marley opens to start an agent CLI gets `GIT_TERMINAL_PROMPT=0` and
`GCM_INTERACTIVE=never` in its environment when its shell starts, so git, and Git Credential
Manager where it is installed, fail with a message instead of waiting on a prompt the agent
cannot answer. Every other terminal keeps git's prompts.

## Scope
### In
- **The variables.** `GIT_TERMINAL_PROMPT=0`: git does not prompt on the terminal, for example for
  HTTP authentication (git(1)), and a command that needs a username fails with `fatal: could not
  read Username for '<url>': terminal prompts disabled`. `GCM_INTERACTIVE=never`: Git Credential
  Manager does not open a window of its own, the value Orca sets for the same reason. Credential
  helpers still run before any prompt would (gitcredentials(7)), so a stored or cached credential
  keeps working.
- **Which terminals.** Every terminal Marley opens to start an agent CLI, all through
  `agents::start_in_terminal`: the rail's Agent CLIs entries (#440), the New Agent picker's CLI
  entries (#450), #510's worktree agents (through `start_cli_with_prompt`), and #527's launch
  configs' Agent items. A launch config's Terminal item runs a command, not an agent, and gets
  none. The variables stay for the terminal's life, so the shell under the agent keeps them after
  the agent exits.
- **The seam.** `Project::create_terminal_shell` takes no environment. Zed's project crate gains
  `create_terminal_shell_with_env`, which hands extra variables to the same internal builder
  after the directory's environment and the `terminal.env` setting; its other callers pass none.
  The launcher's terminal factory and `start_in_terminal` take the variables; New Terminal and
  the Playwright script's terminal pass none; the agent launches pass the list, which
  `marley_agent` keeps beside `launch_input`.

### Out (explicitly deferred)
- An agent typed by hand at a plain shell's prompt: that terminal was not opened for an agent, and
  its environment was fixed when its shell started. Orca draws the same line.
- Zed's own agent terminals and ACP agents: the Agent Panel's terminal threads run
  `terminal_init_command` in a terminal from `Project::create_terminal_shell`
  (`crates/agent_ui/src/agent_panel.rs` 2087, 2138), and Zed's crates start the ACP agents. A
  follow-up can pass the same list through the new function.
- SSH's own prompts. ssh asks for a key's passphrase on the terminal itself, and these variables do
  not reach it. Chad's answer (notes, 2026-09-26) asks for the passphrase in Marley through
  `SSH_ASKPASS`: that is #596, split out as a feature of its own.
- `GIT_ASKPASS` and `core.askPass` (Orca also sets `GIT_ASKPASS` empty when none is set): an
  askpass program opens a window the user can see, and this box has none configured. Orca's
  indexed git config for GCM (`credential.interactive`, `credential.guiPrompt`), which
  `GCM_INTERACTIVE=never` covers.
- A terminal restored after a restart (`create_terminal_shell_restoring`, #575) is a plain shell
  again and gets no variables. #540 plans to resume Claude Code sessions in such terminals;
  the restore opens them, not `start_cli`, so #540 should pass the same list where it opens them.

## Reference (§20)
Orca (MIT, read at `1c2cf120e3`): `src/shared/terminal-git-credential-guard.ts`
(`applyTerminalGitCredentialPromptGuard`: only a terminal launched for a recognized agent, or
marked unattended, gets the guard; a user's terminal keeps git's behavior) and
`src/shared/git-credential-prompt-env.ts` (`gitCredentialPromptGuardEnv`: `GIT_TERMINAL_PROMPT=0`,
`GCM_INTERACTIVE=never`, the askpass variables and two GCM config keys), applied where a PTY's
environment is put together (`src/main/ipc/pty/host-env/assembly.ts:66`). Marley keeps the rule
(agent terminals only) and the two variables. Upstream Zed: the terminal's environment as
`create_terminal_shell` builds it (the directory's environment, then `terminal.env`), which this
extends for agent terminals only. Warp: N/A, the once-over has no such item.

### Prior art
- **Behavior maps.** `docs/orca_architecture/02-worktrees-and-review.md` §2.2 (credential prompts
  forced off in Orca's setup terminal), §2.13 (terminals whose launch command is a known agent)
  and §3 item 8 (the seam: the environment of agent launches from the rail and the agent bar).
- **Published material.** git(1), `GIT_TERMINAL_PROMPT` (the installed manual page);
  gitcredentials(7): helpers run first, then `GIT_ASKPASS`, `core.askPass`, `SSH_ASKPASS`, then
  the terminal. GCM is not installed here; `GCM_INTERACTIVE=never` is taken from Orca's guard,
  whose comment says GCM can open its own window past the terminal guards.
- **Code we already ship.**
  - `agents::start_cli` (`crates/marley_workbench/src/agents.rs:188`) opens the terminal through
    `Launcher::terminal_factory`, by default `Project::create_terminal_shell` (61), then types the
    agent's name after the shell's ready handshake.
  - `Project::create_terminal_shell` (`crates/project/src/terminals.rs:284`) and
    `create_local_terminal` (295) both call `create_terminal_shell_internal` (312), which builds
    the environment from the directory's (`resolve_directory_environment`) extended by
    `settings.env` (377, 378). `TerminalBuilder::new` (`crates/terminal/src/terminal.rs:1153`)
    then adds Zed's variables, Marley's nonce (1209) and Marley's shell integration (1221).
    Nothing between takes per-terminal variables.
  - `create_terminal_task` (terminals.rs 64) does take `SpawnInTerminal::env`, but a task terminal
    runs one command, skips Marley's shell integration, and is not the shell an agent runs in.
  - `marley_agent::launch_input` (`crates/marley_agent/src/marley_agent.rs:90`) is the program's
    name and Enter, nothing else (#440). Typing the variables ahead of it would show in the
    terminal and in the shell's history.

## UI proof
UI-AFFECTING (what an agent's terminal shows when a push needs credentials).
`script/e2e/537-git-credential-prompts-off.sh` (`compositor sway`, for the rail's +). Setup: a
scratch repository with one commit; a local HTTP server that answers every request with 401 and
`WWW-Authenticate: Basic`, so git asks for credentials; stand-ins `claude` and `codex` first on
the PATH that print the two variables, run `git -c credential.helper= push` to that server, print
the exit status, then show that a `store` helper still answers `git credential fill` for the same
host, and wait. Steps: Claude Code from the project's + in the rail (`537-01-rail-agent`); Codex
from the New Agent picker, Ctrl+Alt+N (`537-02-picker-agent`); a New Terminal (Ctrl+~) where the
same push is typed by hand, and git's `Username for` prompt waits (`537-03-plain-terminal`), then
Ctrl+C. The server's log in the run log shows each request.

## Locked-In Decisions
- D1: Agent terminals only, for the terminal's whole life (Orca's rule). A plain terminal keeps
  git's prompts. The agent's shell keeps the variables after the agent exits, so a push typed
  there later fails with the same message, which names the cause.
- D2: The variables enter the terminal's environment when its shell starts, through a new
  `Project::create_terminal_shell_with_env` in `crates/project/src/terminals.rs`: a small
  additive Zed touch with its row in the ledger. They are never typed: the launch line stays what
  `launch_input` makes (the program's name, #440, and the flags #532's setting adds once it
  lands).
- D3: Exactly the two variables. They come after `terminal.env`, so they win over a user's value
  of the same name in an agent's terminal and nowhere else.
- D4: The list is data in `marley_agent` (`GIT_PROMPTS_OFF`), so every launcher takes it from one
  place.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts an agent CLI from the rail's + or the New Agent picker, the agent's terminal shall hold `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never` in its environment. | Shots `537-01-rail-agent` and `537-02-picker-agent` (the stand-ins print both) |
| REQ-002 | WHEN an agent in such a terminal runs a git command that needs credentials no helper holds, git shall fail at once with its "terminal prompts disabled" message instead of waiting on the terminal. | Shots `537-01-rail-agent`, `537-02-picker-agent` (`fatal: could not read Username ...: terminal prompts disabled`, `push exited 128`); the server's log |
| REQ-003 | WHERE a credential helper holds credentials for the remote, git in an agent's terminal shall still use them without a prompt. | Shot `537-01-rail-agent` (the `store` helper's answer printed) |
| REQ-004 | WHEN Marley starts the agent, the text typed into the terminal shall be the launch line `launch_input` makes, after a worktree's setup command and `&&` where #585's Setup box asked for one, and shall hold none of the variables. | Shot `537-01-rail-agent` (the typed line is `claude`) |
| REQ-005 | WHERE a terminal was not opened for an agent CLI, git shall prompt for credentials as before. | Shot `537-03-plain-terminal` |

## Phase Plan
- **P1 Plan:** promote the pair, recall, consult the brain (and ask Chad the SSH question in the
  notes), confirm the design.
- **P2 Code:** the touchpoint row first; `create_terminal_shell_with_env`; `GIT_PROMPTS_OFF`; the
  factory type, `start_cli` and New Terminal; the test factory in
  `marley_workbench_tests.rs`; fmt and clippy clean (Zed's `./script/clippy` for `project`); a
  review of the diff.
- **P3 Test:** write and run the scenario, read every shot; `script/gates.sh --diff` green. No other
  ticket's scenario runs (2026-09-29 workflow).
- **P4 Complete:** CHANGELOG; prong 1's T7 row in `docs/marley/three-prong-plan.md` and D5
  (agents in terminals) of `docs/marley/workbench-shell.md`; the touchpoint row checked against
  what shipped; the ledger; close, archive, commit.
