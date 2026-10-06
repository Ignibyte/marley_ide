# T3 Code: what it does, and what Marley lacks

T3 Code (`pingdotgg/t3code`, by T3 Tools Inc., the team of Theo Browne and Julius Marminge) calls
itself "an agent harness control surface" in its README and "the open-source control plane for
coding agents" on [t3.codes](https://t3.codes). A Node server on the machine that holds the code
runs each coding agent headless through the most structured channel its vendor offers: Claude Code
through the Claude Agent SDK, Codex through `codex app-server`, Cursor through its SDK, OpenCode and
Pi through their own servers, and Grok, Antigravity and any ACP Registry agent over ACP. Web,
Electron desktop and native iOS and Android clients drive that server over one authenticated
WebSocket. Around the conversation it adds worktrees, checkpoints and turn diffs, a pull request
client for six git hosts, a preview browser that agents drive, remote access over Tailscale, SSH or
its own relay (T3 Connect), push and Live Activities on the phone, scheduled and webhook-triggered
prompts, and an MCP server through which one agent starts, messages and waits on others.

It is MIT licensed, eight months old (first commit 2026-02-07), and moves fast: 4,866 commits, 1,274
of them in the 30 days to 2026-10-06 (about 41 a day, 474 with an agent co-author line), about 410
author names, 25,825 GitHub stars, and about 839,000 lines of non-test TypeScript. Its `AGENTS.md`
claims "over 400,000 users" and says most contributions are written by T3 Code itself. Omarchy
ships it: `omarchy-install-ai-t3-code` installs the `t3code-bin` package from the AUR, and
`omarchy-theme-set-t3code` retints it at every theme switch.

Chad asked on 2026-10-06: "research t3 code app and see what kind of stuff it does we dont and file
that research". This folder is that look: seven reports, one per area, each read from T3 Code's
docs and its source, and this summary.

The difference that shapes every report: T3 Code never shows an agent's TUI. The conversation is
T3's own chat view, fed by the agent's SDK events, and its terminal is a drawer of plain shells.
Marley keeps Claude Code's and Codex's own TUIs in its terminals and reads their state from hooks,
its plugin and Codex's App Server. So much of what T3 builds (a composer, a message queue, steering,
rewind, `/goal`, skills menus, model switching) is a chat client's job that the TUIs already do for
Chad in Marley's terminals, and that Zed's Agent Panel does for its own threads. The useful finds sit
around the conversation: what happens at a usage limit, at a restart, across sessions, between
agents, and in the tools Marley gives its agents.

## The source, and the licence

- T3 Code's source was read from a blobless clone of `github.com/pingdotgg/t3code`, `main` at
  `17c0878941` (2026-10-06).
- T3 Code is **MIT**: `Copyright (c) 2026 T3 Tools Inc.`. The vendored `native/libghostty-vt` is MIT
  from Mitchell Hashimoto and Ghostty's contributors, and two mobile modules are MIT from Expo and
  Bluesky. The source may be read and adopted (CONSTITUTION §20); code taken into a Marley crate
  keeps the MIT notice, and the Marley file's header names the T3 file. T3 Code is TypeScript,
  Effect-TS, Electron and React Native, so in practice Marley reimplements in Rust and cites the
  path.
- Other sources: the repository's `docs/` (user guides, internals, runbooks), its release notes on
  [GitHub Releases](https://github.com/pingdotgg/t3code/releases), and the site. Nothing was taken
  from third-party write-ups.
- These reports are the prior-art leg for any ticket in their areas (§20): cite the report and the T3
  path it names.

## The reports

| Report | Area |
|---|---|
| [01-agents-and-providers.md](01-agents-and-providers.md) | how each provider runs, provider instances and accounts, permission and plan modes, switching model or provider in a thread, fork, rewind, restart continuation, usage limits, the Usage page, past sessions, provider updates, text generation |
| [02-threads-and-composer.md](02-threads-and-composer.md) | the sidebar's shelves, settling and snooze, threads with no project, one prompt to several models, search, context chips, thread references, large pastes, queue and steer, visual replies, notifications, themes and Omarchy's hook |
| [03-git-worktrees-and-prs.md](03-git-worktrees-and-prs.md) | checkpoints as hidden refs, the diff panel, rewind with files, worktree naming, setup and cleanup, git actions, the Pull Requests page, six hosts, viewed marks, linked PRs, stacks, the PR watch that wakes the agent |
| [04-remote-and-mobile.md](04-remote-and-mobile.md) | the environment and its clients, `t3 serve` and the user service, version negotiation, pairing and scopes, Tailscale, routes, SSH environments, T3 Connect, the phone app, push, Live Activities, the browser on another device |
| [05-terminal-browser-and-capture.md](05-terminal-browser-and-capture.md) | the server-owned terminal and its history, the `libghostty-vt` renderer, the preview browser and its 19 agent tools, file choosers and downloads, the annotator, video recording, ports, SnapShots, the Devices panel |
| [06-orchestration-mcp-and-automations.md](06-orchestration-mcp-and-automations.md) | the per-session MCP server and its 80 tools, the privilege ceiling, instructions and result sizes, delegated tasks, completion pointers, secrets, scheduled tasks, webhooks, the cancelled-work note, "Hit every surface" |
| [07-engineering-and-releases.md](07-engineering-and-releases.md) | the process architecture, Effect services, the event-sourced orchestrator, performance discipline, tests and replay fixtures, test data copied one way, the contributor contract, release trains, `t3 triage`, Codex protocol bindings, two months of release notes |

Seven agents wrote them on 2026-10-06, read-only against the checkout above; nothing was built or
run. Each checked Marley's side against `docs/marley/guide.md`, `CHANGELOG.md` and the code under
`crates/`, and against Zed's own crates before calling something missing.

## What T3 Code does that Marley lacks, first

Ranked by use to Chad's day: Claude Code and Codex in Marley's terminals on Omarchy, with Marley
rebuilt and restarted often. Size: S a day, M a few days, L a week or more.

1. **Restarts that lose nothing.** T3's threads outlive a restart for every provider, and behind a
   setting (off by default) it continues a turn the restart cut off with "Continue where you left
   off.", naming the background work that died with it (`RestartContinuation.ts`,
   `RestartBackgroundNote.ts`). Its terminals
   replay up to 5,000 lines or 8 MiB of history with terminal queries stripped
   (`apps/server/src/terminal/Manager.ts`). Marley's #540 resumes Claude Code only, leaves a cut-off
   turn idle, and every terminal comes back empty. For Marley: resume Codex too (`codex resume`,
   the id from the App Server), an "Interrupted · Continue" chip on a resumed row, then saved
   scrollback replayed dimmed above the new prompt, with blocks redrawn from saved records rather
   than from replayed bytes. S for the first two, M to L for scrollback. Reports 01 (item 2), 06
   (item 5), 05 (item 1).
2. **A usage limit as a state, with continue at reset.** T3 marks a thread Limited with the reset
   time and offers Resume at reset, Snooze until reset and an auto-resume setting, with guards
   against double or stale recoveries (`UsageLimitRecoveryWorker.ts`). In Marley a limited agent
   looks failed or idle. For Marley: `limited · resets in 1 h 35 m` on the row and in Needs you, and
   Continue at Reset, Codex first since its App Server reports the window and takes `turn/start`.
   M. Report 01 (item 1).
3. **Past sessions, searchable, with Resume.** T3 imports past Claude Code and Codex conversations
   and searches every thread's messages from the palette (`AgentSessionScanner.ts`,
   `ThreadSearch.ts`). Marley shows only live terminals; the way back to last week's session is
   `/resume` in the right folder, one CLI at a time. For Marley: a pure parser for both CLIs'
   session files, a Sessions list per project, and `marley: search agent sessions` with T3's ranking
   and snippets; Enter opens `claude --resume` or `codex resume` in the session's folder. This is
   also the unbuilt half of Orca report 01's item 7. M. Reports 01 (item 4), 02 (item 2).
4. **Agent tools that explain themselves and fit the agent's limit.** T3's MCP server sends usage
   instructions in its `initialize` result, keeps results near 20 KB with offset paging, and refuses
   with typed codes. Marley's `initialize` carries no instructions, and `terminal_read` can return
   256 KiB, more than twice Claude Code's default MCP output limit of 25,000 tokens, so Claude Code
   cuts it or moves it to a file. For Marley: a short instructions text (when to read blocks before
   asking for a paste, when to use the Browser tab instead of a new Chromium), a default near 20 KB
   with `offset` and `next_offset`, and refusals with a code, next steps and a `request_id`
   (Orca report 06's item 7, still open). S. Report 06 (items 1, 2).
5. **Codex gets what Claude Code gets.** T3 treats every provider alike. In Marley several features
   are Claude-only: push (`push.rs` hard-codes `AgentKind::Claude`), resume (#540) and per-turn
   diffs (#509), and Codex's own questions and app-access requests go to the terminal while its
   approvals are answered in the inbox (#651). For Marley: push for Codex and Zed's agent threads
   with T3's per-kind switches (approval, input, finish, failure) and grouping; Codex questions and
   app-access answered from the inbox through the App Server; turn checkpoints taken on the App
   Server's turn start and end. S to M. Reports 04 (item 1), 01 (item 5), 03.
6. **Undo one turn's files.** T3's Edit from here restores files with the conversation, and only in
   a worktree no other session shares (`CheckpointRestoreSafety.ts`). Claude Code's `/rewind` misses
   whatever a shell command changed, and Codex restores no files; Marley's turn commits hold both.
   For Marley: Undo This Turn on a Turns row, a three-way reverse apply that keeps later turns,
   refused while another agent works in the same checkout. M. Reports 03 (item 1), 01 (item 7).
7. **Follow Omarchy's theme.** Omarchy retints T3 Code, Claude Code, VS Code, Obsidian and its
   terminals at every theme switch; Marley keeps its old colours. For Marley: a theme template in
   Omarchy's user templates folder, rendered into Marley's themes folder, which Zed's theme watcher
   reloads; opt-in, so a chosen Zed theme stays. S to M. Report 02 (item 1).
8. **Hand work between agents.** T3 switches provider inside a thread and carries a budgeted
   selection of the history (the latest request and answer, the first request, then newest first),
   framed as context, with a tool to read the rest (`ContextHandoffBudget.ts`). For Marley:
   "Continue in Codex" and "Continue in Claude Code" on an agent row, built on item 3's parsers, and
   an `agent_session_read` tool so one agent can read what the other found. M. Reports 01 (item 3),
   02 (item 3).
9. **Several agents from one prompt, and agents that start agents.** T3 sends one prompt to several
   models, each in its own worktree. Its agents can delegate to a child agent or launch a thread in
   a worktree, never broader than their own permission mode, and a child's completion comes back to
   the parent as a one-line pointer. For
   Marley: agent checkboxes in the New Agent in Worktree prompt (#510), then an `agent_launch` tool
   over the same path with a per-terminal token and a completion pointer typed only into an idle
   caller. S to M, then M. Reports 02 (item 4), 06 (items 3, 4).
10. **The browser tools Marley's agents still lack.** T3's agents can wait for a locator, hover,
    pick an option, drag, upload, answer a dialog, resize to a device preset and switch the colour
    scheme; people can take control of a tab, and file choosers and downloads work. Orca report 03's
    item 11 listed the same tools and none was built. For Marley: those tools under `browser.write`,
    MCP annotations on every tool, file choosers and downloads through CDP, and a Take Over key like
    the terminal's Ctrl-I. S each, M together. Report 05 (items 2 to 4).
11. **A pull request loop for worktree agents.** T3 shows a thread's pull request and checks, opens
    one with generated text, and watches it for the agent, waking it on a failed check, passing
    checks, a new comment or a conflict, with loop guards (`pullRequestWatch.ts`). For Marley: #531's
    PR chip on worktree rows, Push and Open Pull Request with generated text, and a `pr_watch` tool.
    M, and only if worktree branches go to GitHub rather than a local merge (open question 1).
    Reports 03 (items 2, 3, 8), 06 (item 7).
12. **A Usage tab.** T3 reads tokens, models and limits from the CLIs' own transcripts and pools
    accounts per window. For Marley: a center tab over the same parsers, with Codex limits from its
    App Server and Claude's from the status line. M. Report 01 (item 6).
13. **Claude Code's Remote Control, switched on from Marley.** T3's phone app exists because its
    agents have no app of their own. Claude Code has one: `--remote-control` puts a session's
    prompts, approvals and pushes on the Claude app. For Marley: a setting that adds the flag, a
    chip on the row, and Marley's own Claude pushes held back while it is on. S. Report 04 (item 2).

Smaller things worth a day each: Claude Code's `acceptEdits` and `auto` and Codex's automatic review
as permission modes (report 01, item 8); a paste over 32 KiB handed over as a file (report 02,
item 6); `OOMPolicy=continue` on Marley's Chromium units, so one killed renderer does not close a
project's tabs (report 04, item 3); writes to Codex's and OpenCode's config files that show the
change, check the file did not move, keep a backup and rename into place (report 05, item 6);
worktrees started from the fetched base and branches named from the first prompt (report 03, items
4 and 5, which are Orca report 02's items 10 and 11); review notes that carry their hunk (report
03, item 7); subagent rows under an agent row (report 01, item 9); port rows that say which
listeners serve a page and which terminal started them (report 05, item 8); pages an agent draws,
shown in a Browser tab (report 02, item 5).

## For Marley's own workflow

Report 07 found these in how T3 Code is built:

- **Test runs that act on nothing.** `script/e2e.sh` copies Chad's profile and turns off Rusty,
  voice, the IDE link and Codex's App Server, but keeps `marley.push`, the harness settings, fleet
  hosts and System One's key, so a scenario can push to the phone or read real hosts. T3's
  `migrate-dev-db` keeps only what a run needs. S, and worth doing first.
- **The guide changes with the feature.** The guide still says "through #516", its planned table
  lists shipped tickets, and #528, #535, #540, #550 and #556 have no entry. A "Guide:" line in the
  notes template, and a catch-up pass. S.
- **Codex's App Server checked at each Codex release** with a schema diff, since Marley reads
  Codex's state only on 0.155.1 to 0.158.0 and Codex updates itself. S, M with a recorder.
- **A surfaces checklist in the plan template** (both layouts, local and remote projects, groups,
  worktrees, all four agents, an MCP tool or why not, the way back out, docs), after T3's "Hit
  every surface". S.
- **`marley: triage`**, after `t3 triage`: a context file of logs, versions and masked settings,
  handed to Claude Code with a playbook that files a note under `docs/planning/intake/`. S to M.

## Facts about Marley the survey turned up

- Session resume (#540), phone push (#535) and per-turn diffs (#509) cover Claude Code only.
- Orca items never built: Continue in a new session (report 01, item 8), the browser tool gaps
  (report 03, item 11), typed refusals and a worktree tool (report 06), commit and PR text, branch
  names from the first prompt and PR merge through `gh` (report 02, items 10 to 12), and the
  rollback binary (report 07).
- Any client holding the MCP bearer can name any terminal in the `Marley-Terminal` header, so the
  server cannot prove which terminal calls (`crates/marley_mcp/src/transport.rs:585`).
- Marley's tools carry no MCP annotations (`readOnlyHint`, `destructiveHint`), and four tools
  (`browser_check_pick`, `browser_draft_test`, `browser_open_url`, `terminal_run`) are missing from
  the guide's tool tables.
- `crates/marley_workbench/src/agent_notify.rs` rewrites Codex's `config.toml` with a plain write:
  no backup, no check that the file is unchanged since it was read.
- Zed already has the ACP Registry (`crates/project/src/agent_registry_store.rs`), ACP session
  import (`crates/agent_ui/src/thread_import.rs`), and queued and steered messages, rewind and
  Restore Checkpoint in its Agent Panel. None of those is a gap.
- On local worktree work Marley is ahead: T3 Code has no local merge, its cleanup cannot see a
  squash merge, and it has no drift chips, port slots or teardown hooks.

## What not to take

- Running agents headless through SDKs as the main surface, and the composer, queue, steering,
  stash and `/goal` row that come with it: the TUIs and Zed's Agent Panel already give Chad those.
- The event-sourced orchestrator, thread store, command receipts and capability negotiation: they
  keep a GUI conversation alive inside a server Marley does not have. Hand the harness three
  lessons (receipts keyed by a client id, an ordered publish lane, a turn's end apart from its
  finalization) and take the policies, not the store.
- Full access as the default permission mode (`providerPolicy.ts:32`): Marley keeps each CLI's
  prompts unless Chad opts in (#532).
- T3 Connect, the native phone apps, Live Activities, the widget and the multi-route catalog: on a
  tailnet with Claude's app covering Claude Code, a hosted relay and two store apps buy little.
- The Pull Requests page as a review client, the five non-GitHub hosts, stacks and "remove agent
  credits": Chad's repositories are on GitHub, and his rules require the co-author line.
- Adapters for Cursor, Grok, Antigravity, Pi, OpenCode servers and Devin, and Claude reset credits
  read from private endpoints.
- The `libghostty-vt` renderer, server-owned PTYs, cookie import, the Devices panel, SnapShot
  backends other than Hyprland's, and `preview_evaluate` (Marley runs no script an agent supplies).
- Product telemetry (PostHog, on by default in T3), three release trains and hosted manifests.

## Open questions for Chad

Each has the default Marley follows until Chad says otherwise.

1. **Pull requests for agent work.** Do worktree branches go to GitHub as pull requests, or are they
   merged locally (#511) or by the Rustal workflow? *Default: item 11 waits.*
2. **Usage limits.** When the limit resets, should a limited agent continue by itself, or only when
   Chad presses Continue at Reset? *Default: the button only; auto-continue is a setting, off.*
3. **A cut-off turn after a restart.** Continue it by itself, or offer a chip? *Default: a chip.*
4. **Agents that start agents** (item 9): Marley's tool over #510's path, or the harness's job?
   *Default: Marley's, never broader than the caller; nothing built until Chad agrees.*
5. **Claude Code's Remote Control.** Acceptable as the phone path for Claude Code, given that the
   session goes through Anthropic's API? *Default: Marley changes nothing.*
6. **Omarchy's theme.** Follow it whenever Omarchy is present, or only when a setting says so?
   *Default: a setting, off.*
7. **Restored scrollback.** On by default, given that the log holds whatever was printed?
   *Default: on, mode 0600, with a setting to turn it off.*
