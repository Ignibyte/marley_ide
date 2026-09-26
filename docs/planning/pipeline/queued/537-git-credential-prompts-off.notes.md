# Git credential prompts off in the terminals Marley opens for agent CLIs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-537-git-credential-prompts-off.md
- **Pipeline spec:** 537-git-credential-prompts-off.spec.md

## Phase 1 — Plan
- **Request:** Chad asked on 2026-09-25 for every item decided that day to be specced; this is
  item 8 of report 02's list (`docs/orca_architecture/02-worktrees-and-review.md` §3), in the
  survey README's smaller items. The brief for this draft: agent terminals get
  `GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never`, so an agent's `git push` fails with a
  message instead of hanging; find where Marley launches agents and sets terminal environment.
- **Classification / tier:** feature, S. One small additive Zed touch
  (`crates/project/src/terminals.rs`) and a few lines in `marley_agent` and `marley_workbench`.
- **Recall (§18.3):**
  - #440 (the rail's Agent CLIs entries: `start_cli` writes only the program's name, once the
    shell is ready) and #450 (the New Agent picker, the same `start_cli`).
  - PR-claude-enforce-security-invariants-in-code-not-just-docs-001: "agent terminals only" is
    enforced where the terminal is made (the factory's argument), and the scenario's plain
    terminal checks the other side. The ledger has no entry on git credentials, askpass or
    `GIT_TERMINAL_PROMPT`.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001: the + menu starts on its
    first entry; count the Downs to Claude Code under "Agent CLIs".
  - Brain: not consulted in this drafting pass, which was read-only; `/pipeline:plan` runs
    `brain_ask` at promotion.
- **Discovery (opened and checked):**
  - `crates/marley_workbench/src/agents.rs`: `TerminalFactory` (37), `Launcher` (50, default
    `Project::create_terminal_shell` at 61), `launcher` (70), `start_cli` (188 to 221), the picker's
    `Start::Cli` (340); `crates/marley_workbench/src/rail.rs`: `new_terminal` (582, the same
    factory at 589) and `new_agent` (619, `start_cli`).
  - `crates/marley_workbench/src/marley_workbench_tests.rs:61`: the one test that sets its own
    factory (`display_only_terminal_in`); gate:2 builds it, so its signature follows.
  - `crates/marley_agent/src/marley_agent.rs`: `launch_input` (90), `send_payload` (84).
  - `crates/project/src/terminals.rs`: `create_terminal_task` (64, with `SpawnInTerminal::env`),
    `create_terminal_shell` (284), `create_local_terminal` (295), `create_terminal_shell_internal`
    (312; the environment at 377 and 378: `env_task.await`, then `env.extend(settings.env)`).
  - `crates/terminal/src/terminal.rs`: `TerminalBuilder::new` (1153), `insert_zed_terminal_env`
    in its future, Marley's nonce (1209) and the shell integration gated on `task.is_none()`
    (1221).
  - Other callers of `create_terminal_shell`: `crates/agent_ui/src/agent_panel.rs` (2087 and 2155,
    the Agent Panel's terminal threads with `terminal_init_command`, 2160),
    `crates/terminal_view/src/persistence.rs:286` (restore), `crates/terminal_view/src/terminal_panel.rs`
    (580, 756, 952). None changes.
  - git's manual pages on this box: git(1) `GIT_TERMINAL_PROMPT` (line 1897 of the roff);
    gitcredentials(7), the askpass order.
  - This box: no `GIT_ASKPASS`, `SSH_ASKPASS` or `core.askPass`; no global credential helper; no
    Git Credential Manager; an SSH agent is running; `origin` remotes here are SSH URLs.
  - Orca: `src/shared/terminal-git-credential-guard.ts`, `src/shared/git-credential-prompt-env.ts`,
    `src/main/ipc/pty/host-env/assembly.ts:66` ("unattended agents must fail instead of looping on
    OS credential prompts; user terminals keep normal Git behavior").
- **Decisions:** D1 to D4 in the spec.
- **Open question for Chad (ask at promotion):** SSH remotes. An agent's push over SSH still stops
  on ssh's own questions: an unknown host key, or a key with a passphrase that no agent holds.
  `GIT_SSH_COMMAND='ssh -o BatchMode=yes'` in agent terminals would make those fail at once too,
  at the price of overriding a repository's `core.sshCommand`. *Default: leave SSH alone; this
  box's keys are in its SSH agent.*

### Design
- **Zed, `crates/project/src/terminals.rs`.** `pub fn create_terminal_shell_with_env(&mut self,
  cwd: Option<PathBuf>, env: HashMap<String, String>, cx: &mut Context<Self>) -> Task<Result<
  Entity<Terminal>>>`, a `// Marley: #537` comment on it. `create_terminal_shell_internal` gains an
  `extra_env` parameter and extends the environment with it right after `settings.env`;
  `create_terminal_shell` and `create_local_terminal` pass an empty map. The map type is the one
  the builder takes there. Its row in `docs/marley/zed-touchpoints.md` is written first: what
  changed, why (agent terminals need variables of their own and nothing in the path takes them),
  and at a merge keep the parameter and the function, or move onto an upstream way to pass a
  terminal's environment and drop the row.
- **`crates/marley_agent/src/marley_agent.rs`.** `pub const GIT_PROMPTS_OFF: [(&str, &str); 2] =
  [("GIT_TERMINAL_PROMPT", "0"), ("GCM_INTERACTIVE", "never")];` with a doc comment on why.
- **`crates/marley_workbench/src/agents.rs`.** `TerminalFactory` takes the variables;
  `Launcher::default` uses `Project::create_terminal_shell_with_env`; `start_cli` passes
  `GIT_PROMPTS_OFF`. **`rail.rs`**'s `new_terminal` passes an empty map. The test factory takes the
  argument and ignores it.
- **File manifest.** Zed: `crates/project/src/terminals.rs`. Marley:
  `crates/marley_agent/src/marley_agent.rs`, `crates/marley_workbench/src/agents.rs`, `rail.rs`,
  `marley_workbench_tests.rs`; `script/e2e/537-git-credential-prompts-off.sh` at Test.
- **Ledger rows.** The new touchpoint row. At Complete: an AD for "agent terminals only, for the
  terminal's life" (D1) and why the variables go in at spawn and are never typed (D2).

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002, REQ-003, REQ-004 | setup: `repo` with one commit; a Python server on a free port that answers every request with 401 and `WWW-Authenticate: Basic realm="e2e"` and logs each request; a `store` credentials file for a second fake host, its line built at run time; stand-ins `claude` and `codex` in `$E2E_WORK/bin` (one script) that print `GIT_TERMINAL_PROMPT=... GCM_INTERACTIVE=...`, run `git -c credential.helper= push http://127.0.0.1:<port>/fixture.git HEAD:main`, print `push exited <status>`, print the `username=` line of `git -c credential.helper='store --file=<file>' credential fill` for the second host, then `exec -a claude sleep 600`; `PATH=$bin:$PATH` for Marley and `terminal_env PATH` for its terminals; the scenario's HOME. Steps: trust the repository; click the project's +; Down to Claude Code under "Agent CLIs"; Return; settle 6 | `537-01-rail-agent`: the typed `claude`, the two variables, git's `terminal prompts disabled`, `push exited 128`, `username=agent` |
| REQ-001, REQ-002 | Ctrl+Alt+N; type `Codex`; Return; settle 6 | `537-02-picker-agent`: the same lines in a new center terminal |
| REQ-005 | Ctrl+~ for a New Terminal; type the same push by hand; settle 3 | `537-03-plain-terminal`: `Username for 'http://127.0.0.1:<port>':` waiting; then Ctrl+C |

The run log gets the server's request log (one request per push before git gives up, or the prompt
reached) and Marley.log. Not reachable by a scenario: a real GitHub push (no network in the run)
and Git Credential Manager (not installed), whose variable the stand-ins print but nothing reads.

### Risks
- Sibling tickets drafted the same night touch the same seams (their queued specs, 2026-09-25):
  #520 (terminal identity) changes `create_terminal_shell_internal` too, to set `MARLEY_PROJECT`
  and to add `create_terminal_shell_restoring`; whichever of #520 and this ticket lands second
  merges the two new parameters. #510 and #527 plan a helper shared with `start_cli`; the list
  belongs in it. #532 (agent permission modes) makes `launch_input` take the mode, which REQ-004
  allows for. #540 (session resume) opens agent sessions in restored terminals and should pass the
  list there.
- A user's `.bashrc` that exports `GIT_TERMINAL_PROMPT=1` would win inside the agent's shell,
  since it runs after the terminal's environment is set. That is the user's own choice.
- The stand-ins' stored credential must not look like a secret to gate:10's gitleaks: the
  scenario builds the `store` line from parts at run time, so no URL with a password sits in the
  file.
- `git push` over HTTP to the 401 server: git's first request is `GET /info/refs?service=
  git-receive-pack`; a 401 there is enough for git to ask for credentials, so the server needs no
  git code.
