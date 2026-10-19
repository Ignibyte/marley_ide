# New tmux Terminal and New Agent in tmux — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-819-new-tmux-terminal-and-agent.md
- **Pipeline spec:** 819-new-tmux-terminal-and-agent.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19 (item 1 of two): "on the + on the left panel if tmux is on the box
  auto detected. THen lets add a New Terminal Agent." Asked what the + should do: "new tmux agent
  and new tmux terminal session basically".
- **Classification / tier:** feature; one Marley crate (`marley_workbench`: `tmux_sessions.rs`,
  `rail.rs`).
- **Recall (§18.3):** #808's tmux sessions (L-claude-808: scratch `TMUX_TMPDIR`, kill the server
  in teardown); #820's `new_entries`; #806's stand-ins quiet on `--version` (#648 probes it). The
  brain: nothing on this seam.
- **Discovery:** see the spec's Prior art.
- **Decisions:** D1 to D3 in the spec.

### Design
- `tmux_sessions.rs`: `pub(crate) fn installed(cx) -> bool` (`which::which_in("tmux",
  agents::launcher(cx).search_path, "/")`); `follow` takes `tmux_hosts.unwrap_or_else(||
  installed.then(|| vec![LOCAL]))`; `open`'s Place::Here branch becomes `open_here(session,
  words, ..)`, shared with `pub(crate) fn new_local_session(folder: &Path, program: Option<&Path>,
  workspace, window, cx)`, which names a session, runs `new-session -A -s <name> -c <folder>
  [program]`, and lists soon.
- `rail.rs` `new_entries`: when `installed` and the workspace's project is local, New tmux
  Terminal after New Terminal, and after New Agent in Worktree a submenu New Agent in tmux of
  `installed_clis`, each by `which_in(kind.program(), ..)`'s path; the folder is the first visible
  worktree's, else the home folder.
- File manifest: `crates/marley_workbench/src/{tmux_sessions.rs, rail.rs}` (Marley);
  `script/e2e/819-new-tmux-terminal-and-agent.sh` (new); Phase 4: the `tmux_hosts` doc (settings
  content comment, guide).

### Visual check plan
- As the spec's UI proof. The scenario's `teardown` kills its tmux server.

### Risks
- A pane takes the tmux server's environment: a server started by Marley's attach inherits that
  terminal's; one already running keeps its own (Out).

## Phase 2 — Code
- Built: `tmux_sessions.rs` `installed` (`which_in("tmux", launcher search path)`), `follow`'s
  unset `tmux_hosts` → `["local"]` while installed, `new_local_session(folder, command, ..)` and
  `open_here` (the Place::Here words, now shared with `open`); `rail.rs` `new_entries` gains New
  tmux Terminal after New Terminal and `tmux_agent_entries` (New Agent in tmux) after New Agent in
  Worktree, both behind `tmux_folder` (tmux installed, a local project; its first folder, else
  the home folder); `start_tmux_in` shows the workspace and starts the session (`new_tmux_session`
  was #808's name already).
- Deviation (D3 changed): the first check run started the agent by the path Marley's search path
  gave, and Marley's PATH comes from the login shell, so the real Claude Code started (at its
  first-run theme screen, HOME the run's scratch folder; the run's teardown killed its tmux
  server). The agent is now typed into the session's shell (`; send-keys -t <name> <cli> Enter`),
  which runs it by the shell's PATH, the stand-in's in the scenario and the user's own otherwise.
- The scenario: the stand-in on the terminals' `.bashrc` PATH (a heredoc; shellcheck refused a
  `printf` with a literal `$PATH`), waiting as `exec -a claude sleep 900` so tmux and the rail name
  its pane `claude` (#808's trick); points measured from a check run.
- Review: a user's own `tmux_hosts` list wins (D1); `[]` stays off; the entries need a local
  project, since `remote::open_words` opens only there; errors reach a prompt through
  `tmux_sessions::report`.
- The gate: `just gate-diff` green, 17 of 17 (after the shellcheck fix).

## Phase 3 — Test
- `just e2e script/e2e/819-new-tmux-terminal-and-agent.sh` on the debug build after the green gate,
  under sway, tmux on `$XDG_RUNTIME_DIR/marley-e2e-819` (gone after teardown); focus report "1
  Marley windows before the run, 1 after"; both `expect`s pass. Shots read:
  - `819-01-plus`: `repo`'s `+`: New Terminal, New tmux Terminal, New Browser Tab, New Agent…,
    Reopen Thread…, New Agent in Worktree ›, New Agent in tmux ›; "TMUX ·" already under the
    project, with no `tmux_hosts` set (REQ-001, REQ-004).
  - `819-02-tmux-terminal`: a tab `this machine · marley-…` attached to the session (tmux's status
    line `marley-… 1:repo`), `pwd` printing the run's `…/repo`; the rail lists the tab under
    `repo` and `TMUX · this machine 1 session` with it, `1 window · attached` (REQ-002, REQ-004).
  - `819-03-tmux-agent`: a second session's tab where tmux typed `claude` and the stand-in printed
    `claude (stand-in) in …/repo`; under `TMUX · this machine 2 sessions` its row reads `Claude
    Code · 1 window · attached` with Claude's icon (REQ-003).

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Added, #819); `docs/marley/guide.md`, the tmux section;
  `docs/marley_architecture/marley_workbench.md`, the tmux section; the `tmux_hosts` doc comment
  in `crates/settings_content/src/marley.rs` (its default), which the Settings window shows, so
  the gate ran again before the commit.
- Ledger appends: `F-claude-819-a-check-run-started-the-real-claude-code-001`,
  `PR-claude-819-marley-starts-an-agent-cli-by-name-in-a-shell-001`.
- Brain: consultation `6cb891734df3413e9fd5ad05874a3afb` closed with `brain decide` (follow-up
  2026-11-19).
- Ticket closed; its `BACKLOG.md` row left at promotion.
