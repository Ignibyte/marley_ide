# Orca: what it does, and what Marley takes from it

Orca (`stablyai/orca`, "the ADE for working with a fleet of parallel agents") is an Electron and
TypeScript app that runs coding-agent CLIs side by side, each in its own git worktree, with an
embedded browser, a diff review that sends notes back to the agent, a phone companion, SSH
worktrees and a CLI that agents drive it with. It is MIT licensed, six months old (first commit
2026-03-16), and moves at about 60 commits a day, most of them written by agents.

Chad asked on 2026-09-25 for a thorough look at it, "we will be taking what it does well and
bring it in here", with its source on disk. This folder is that look: seven reports, one per area,
each read from Orca's docs and its source, and this summary.

## The source, and the licence

- Orca's source is checked out at `/srv/stacks/orca-refs/orca` (a blobless clone of `main` at
  `1c2cf120e3`, 2026-09-25: `git log` works, older blobs fetch on demand), with the plugin index
  (`orca-plugins`) and the workflow skills (`orca-workflow-skills`) beside it.
- Orca is **MIT**: `Copyright (c) 2026 Lovecast Inc.`. Unlike Warp's AGPL source, which Marley never
  reads (CONSTITUTION §20), Orca's source may be read and adopted. Code taken from it into a Marley
  crate keeps Orca's copyright and permission notice, which is MIT's one condition; name the Orca
  file in the Marley file's header. Most of what is worth taking is design, and Orca is TypeScript
  and Electron, so in practice Marley reimplements in Rust and cites the Orca path.
- These reports are the prior-art leg for any ticket in their areas (§20): cite the report and the
  Orca path it names.

## The reports

| Report | Area |
|---|---|
| [01-agents-and-sessions.md](01-agents-and-sessions.md) | launching agent CLIs, agent state from hooks, notifications and unread, session history and resume, hibernation, usage and accounts, orchestration |
| [02-worktrees-and-review.md](02-worktrees-and-review.md) | worktree creation, setup and teardown hooks, ports, removal and squash-aware branch cleanup, the diff viewer, notes back to the agent, commit, PR and merge, hosted integrations |
| [03-browser-and-design-mode.md](03-browser-and-design-mode.md) | the browser pane, Design Mode (grab, annotate, markup), profiles and cookies, link routing and ports, the agent browser commands, computer use, the emulator |
| [04-remote-control-and-mobile.md](04-remote-control-and-mobile.md) | the phone app, pairing and encryption, the relay, push, the headless runtime, SSH worktrees, remote ports, ephemeral VMs, the security model |
| [05-terminal-and-workspace.md](05-terminal-and-workspace.md) | the terminal engine, terminals that outlive the app, shell integration, links, tabs and splits, session restore, Cmd-J, the sidebar, the editor, drops, dictation, onboarding, settings, plugins |
| [06-cli-automations-skills.md](06-cli-automations-skills.md) | the `orca` CLI and how agents reach the app, terminal identity, receipted input, orchestration and its mailbox, automations, skills, the session archive, artifacts, hooks |
| [07-engineering-and-changelog.md](07-engineering-and-changelog.md) | process architecture, durable state, crash and hang capture, tests, architecture rules as tests, evidence documents, release engineering, telemetry, and three months of release notes as the real feature list |

Seven agents wrote them on 2026-09-25, read-only against the checkout above; nothing was built or
run. Two corrections to what the agents were told: Marley's pick sends no HTML and no computed CSS
(report 03 checked `PickBundle`), and Marley does send telemetry. Zed's `report_event` queues
events whenever `telemetry.metrics` is on (the default), and `flush_events_inner` posts them to
`api.zed.dev/telemetry/events`.

## What Orca does well that Marley lacks, first

Ranked by what each buys Marley against its cost (S a day, M a few days, L a week or more).

1. **Agent state from Claude Code's hooks, carried in-band.** Orca knows what each agent is doing:
   the prompt, the tool in flight and its input, the question it waits on, its last message, its
   children, its session id. It gets all of it from hooks it writes into 16 CLIs' configs, which
   `curl` each event to a loopback server. Marley's rail guesses from 2 s of quiet. Marley's plugin
   can register the same Claude events and answer each with a `terminalSequence` OSC 777 under a
   reserved title carrying base64 JSON. The event then arrives in the very terminal it belongs to,
   over SSH too, with no server, endpoint file or relay. This one item feeds the approvals inbox,
   per-turn diffs, notifications that say something, attention order in the rail, session resume
   and a non-empty `fleet_snapshot`. M. Reports 01 (item 1), 05 (item 1), 06 (item 3).
2. **Terminal identity.** Every Orca terminal starts with its own id in the environment, so an agent
   knows which terminal and worktree it is. Marley's terminals carry nothing that names them, and
   its bridge forwards no caller, so agent tools act on "the tab the user focused last, in any
   project". `MARLEY_TERMINAL_ID` and `MARLEY_PROJECT` at spawn, forwarded by the bridge as a
   header, scope every tool to its caller. S. Reports 06 (item 1), 03 (item 4).
3. **The approvals inbox answers, not only lists.** Claude Code's `PermissionRequest` hook may
   return allow or deny itself. A hook of type `mcp_tool` on Marley's server can wait for the
   answer from the rail (and later the phone) and fall back to the TUI's own prompt on timeout.
   Orca never managed this: its phone types "1" or Esc into the terminal. M after item 1.
   Reports 04 (item 1), 01 (item 2).
4. **A fuller pick.** Orca's grab carries up to 4 KB of outerHTML, 16 computed styles, the React
   component chain and the JSX source file and line, with secret-looking values redacted. Marley's
   pick has the locators, listeners and crop but none of those, and on a React app its
   source-mapped listeners land in react-dom. S to M, and #505 needs it. Report 03 (item 1).
5. **Worktree removal that never loses work.** When `git branch -d` refuses, Orca proves the
   branch merged anyway, squash merges included (`git merge-tree --write-tree`, `git cherry`, a
   patch-id match), and deletes it with a compare-and-swap `update-ref`. Otherwise it keeps the
   branch and says so. About 280 lines to port for #511. Report 02 (item 2).
6. **Ports that belong to a project.** Orca reads `/proc/net/tcp`, maps each listener to its
   process's working directory and so to a worktree, and keeps a dev server's printed URL only
   while something listens on its port. That covers #503's offer, #504's port rows and #510's port
   offsets, and gives agents a `ports_list` tool. M. Reports 03, 05 (item 5), 02 (item 7).
7. **Agent sessions that come back.** Orca relaunches an agent with `claude --resume <id>` after a
   restart, using the session id its hooks reported, and lists every past session per project with
   Resume. Marley loses every terminal agent at quit. M. Reports 05 (item 3), 01 (item 7), 06
   (item 5).
8. **One Marley per data directory, and a crash you can read.** Report 07 found what #502 left
   open. On the `dev` channel two Marleys can run on one profile, and whichever quits first deletes
   the other's `mcp-endpoint.json`, which is also written in place rather than atomically. The
   fixes: an instance guard, an endpoint file written durably and removed only by its owner, local
   minidumps (`ZED_GENERATE_MINIDUMPS=1`, nothing uploaded) and a rollback copy of the binary. S.
   Report 07 (items 2, 4).
9. **Regression, not just proof.** Orca runs a golden set of end-to-end suites as blocking gates
   and routes each change to the specs that cover its paths. Marley's scenarios each run once, at
   their own ticket's Test. A route table from paths to scenarios and a golden list would make them
   a suite. S to M. Machine checks through `marley_mcp`, so that only failing shots need reading,
   would amend §7. Report 07 (item 1).
10. **Architecture rules that cannot rot.** Orca's ratchet tests carry five devices: a shrink-only
    allowlist, a failure on a stale entry, a pinned count, an anti-vacuity check, and a planted
    violation the check must catch. Marley's grep gates have none of them, and §14's "spawns stay
    in adapter modules" is checked by nothing. S per rule. Report 07 (item 3).
11. **Review notes back to the agent.** Zed draws "Send Review to Agent (N)" on its diffs, and
    nothing in the tree handles the action: upstream deleted the handler in February. A handler
    that pastes the notes into the chosen agent's terminal, only while that agent is idle, closes
    the review loop for #509 and #511. S to M. Report 02 (item 5).
12. **Remote control, the Marley way.** Push to the phone (a self-hosted ntfy first), then a
    phone endpoint bound to loopback behind `tailscale serve`, with QR pairing and scoped device
    tokens, then a small web app for the fleet, approvals and prompts. Orca's relay, push gateway
    and 186,000-line phone app are not needed on a tailnet. About a week for push and approvals,
    about four weeks for the phone client. Report 04.

Smaller things worth a day each: copy without an agent's gutter, bracketed pastes whenever an agent
runs, raw image paths for drops (report 05, item 7); agent-grade refusals with next steps and
request ids in `marley_mcp` (report 06, item 7); git credential prompts off for agent terminals
(report 02, item 8); dropping `HeadlessChrome` from the Browser tab's user agent (report 03,
item 10); remote terminals that survive a dropped link through tmux (report 04, item 5).

## What it changes in the queued sprint

- **#503 (terminal URLs to a Browser tab).** A link popover (Browser tab, system browser, Copy)
  with a default asked once, Ctrl+click for the default and Shift+Ctrl+click for the other. URLs
  that a TUI wrapped are stitched back together, and OSC 8 targets are followed. A printed URL is
  offered only while its port listens (a `/proc/net/tcp` check). `0.0.0.0` and `[::]` open as
  loopback. URLs from SSH terminals stay with the system browser. Reports 03 and 05.
- **#504 (Browser tabs in the rail).** A favicon read from the page, a spinner while loading, host
  and port, counts of picks and annotations, a mark when an agent acted on the page; later, rows
  for live ports with no tab. Report 03.
- **#505 (pick, fix, check).** It waits for item 4's fuller pick. Record the box, styles and text
  at pick time. Re-find by test id, id, role and name, text, then CSS path, with hashed CSS-in-JS
  classes left out of every locator. Crop both times under the same conditions, scrolled into view.
  Report what changed as text (`padding: 8px → 16px`). Orca has nothing here: "click it again" is
  its whole check. Report 03.
- **#506 (recording to a Playwright test).** Compute each target's locators in the page when the
  event happens; prefer `getByTestId`, then role, label, placeholder, text, then CSS; write
  password fields as `process.env` placeholders; assert the URL after each navigation. Orca has no
  recorder to copy. Report 03.
- **#507 (a browser context per project).** CDP's `Target.createBrowserContext` contexts are off
  the record: Marley can save and restore their cookies, but logins kept in `localStorage` or
  IndexedDB (Supabase, Firebase) die at a restart. One Chromium unit per project, with its own
  profile directory keyed on the project's root, keeps them. Marley's service already names units
  by profile path. Key storage on a durable id, never a per-boot value; Orca lost cookies at every
  reconnect until it learned that. Report 03.
- **#508 (approvals inbox).** Built on item 1: one "needs you" state and glyph. An entry names the
  agent, the project and what is asked (`Bash: rm -rf build`, or the question and its options),
  with its age, oldest first. Agent Panel entries are answered in place through
  `AcpThread::authorize_tool_call`; terminal entries open the terminal, and item 3 later answers
  them from the rail. Entries clear on the session's next event. Nothing is pasted into a
  terminal whose agent is waiting. Reports 01, 04, 05, 06.
- **#509 (per-turn diffs).** Zed already has what the snapshots need: `GitStore::checkpoint` and
  `diff_checkpoints`, temporary-index trees that touch neither the index nor any ref. A turn opens
  on `UserPromptSubmit` and closes on `Stop`, `StopFailure` or a manual `PostCompact`, and also on
  the next `UserPromptSubmit`: in Orca's recorded sessions an Esc interrupt fires no hook at all.
  Harness-injected prompts and background tasks need their own rules. Reports 02 and 01.
- **#510 (worktree agents).** Zed's worktree service creates only detached HEADs, so the ticket
  adds a `NewBranch` option (a small Zed touch). Create with `--no-track`, and record the base in
  git config as `branch.<b>.base` so #511 knows it without Marley state. Put the first prompt on
  the agent's argv, not a paste. Copy `.worktreeinclude` files. Keep Claude Code's own
  `.claude/worktrees` out of the rail. Check that Claude Code's folder-trust prompt does not stall
  a new worktree. Pair the port offset with discovery (item 6). Reports 02, 01, 06.
- **#511 (review, merge, remove).** Item 5 for removal. For merging (Orca merges only through
  `gh pr merge`): require the main checkout clean and on the base, show how far the base has moved,
  and on a conflict abort there and hand the worktree's agent a conflict prompt. A teardown hook
  that blocks removal when it fails. Report 02.

New tickets these imply, in the order they would land: one Marley per data directory (item 8,
after #512); a fuller pick bundle (item 4, before #505); structured Claude Code events and terminal
identity (items 1 and 2, before #508 and #509); ports per project (item 6); review notes to the
agent (item 11); session resume (item 7); regression routing and a golden set (item 9); the phone
path (item 12).

## What not to take

- The Node terminal daemon as built (60,000 lines, 36 wire versions). Its rules go to the harness
  client (prong 2, C3) instead.
- Hooks written into sixteen CLIs' global configs and a loopback hook server with spool files,
  launch tokens and relays: Marley's opt-in plugin and in-band frames do the job.
- Permission-bypass flags as every agent's default (Orca's onboarding starts "Yolo" on).
- The 39-agent catalog, parsers for 20 transcript formats, account hot-swap and usage read from
  private vendor endpoints.
- The orchestration database (42,000 lines, experimental): rustal-brain and rustal-harness hold
  those roles in Marley's plan. Keep its mechanisms: the worker preamble, mail delivered as a
  one-line pointer typed into an idle agent, receipts, and the `live` / `unverifiable` / `exited`
  vocabulary.
- The relay, the push gateway, forking the phone app, and phone-width resizing of desktop
  terminals.
- Monaco, the plugin system, the settings store, themes, keybindings and translations: Zed is
  ahead on each.
- The emulator pane, computer use (X11-only on Linux), the warm checkout pool, the reliability-gate
  registry (126 gates, none ever blocking), five release channels, and product telemetry.

## Open questions for Chad

Each has the default Marley follows until Chad says otherwise.

1. **Telemetry.** Zed's metrics are on and posted to `api.zed.dev`. Turn them off in Marley's
   defaults? *Default: ask before changing; nothing changes yet.*
2. **#507.** One Chromium per open project (logins in `localStorage` survive, a few hundred MB
   each), or one Chromium with contexts and a cookie jar Marley keeps (lighter, and those logins
   are lost at a restart)? *Default: one Chromium per project.*
3. **#510.** A worktree agent as a row nested under its project (Orca's model, and how Zed already
   groups worktrees), or its own top-level project? *Default: its own project row in the rail, as
   the ticket says, labelled with its parent.*
4. **#511.** Merge commit, squash, or rebase then fast-forward? Push afterwards? *Default: a merge
   commit (`--no-ff`), no push.*
5. **Worktree agents' permissions.** Each CLI's own prompts, routed to the inbox, or an opt-in
   bypass? *Default: the CLI's own prompts.*
6. **Per-turn snapshots.** Session-only, or pinned with refs so they survive a restart? *Default:
   pinned under `refs/marley/turns/`, pruned with the worktree.*
7. **Regression runs.** May golden scenarios pass on a machine check, with shots read only on a
   failure? It amends §7 for regression runs only. *Default: no change until Chad agrees.*
8. **The phone.** Is it on the tailnet, and a web app or a native app? *Default: nothing built
   until Chad picks.*
9. **Terminals across a restart.** Resume Claude Code sessions now (item 7), or wait for the
   harness to keep the processes alive? *Default: resume now; process survival stays with the
   harness.*
