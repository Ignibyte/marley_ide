# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-583](open/TICKET-583-chromium-devtools-off-tcp.md) | feature | prong 3 · Chromium's DevTools off TCP behind Marley's relay: a 0600 Unix socket for Marley, a CDP WebSocket that takes a client's token for Playwright clients; slice 2 of #524 |
| [TICKET-584](open/TICKET-584-outside-clients-from-other-machines.md) | feature | prong 3 · #524's clients from another machine, through `ssh -L` or `tailscale serve` to a fixed loopback port; slice 3 of #524 |
| [TICKET-539](open/TICKET-539-headless-chrome-user-agent.md) | chore | prong 3 · Marley's Chromium introduces itself as Chrome, not HeadlessChrome |
| [TICKET-521](open/TICKET-521-ports-per-project.md) | feature | prong 3 · listening ports per project in the rail, with Open, Copy and Stop |
| [TICKET-565](open/TICKET-565-system-one-layer.md) | feature | prong 2 · the System One layer: TypeSafe, compatible, rules and replay providers; compiled-in question sets, masked state, the call log and replay, modes and switches, the Decisions view; off by default |
| [TICKET-566](open/TICKET-566-stop-kind.md) | feature | prong 2 · what a stopped Claude Code turn needs (done · checked, done · claimed, asks you, blocked, still going) as the row's word from #519's seats; code decides the clear cases, the model the rest; on #565 |
| [TICKET-567](open/TICKET-567-find-tools.md) | feature | prong 2 with prong 3 · browser_find and terminal_find: a match by words first, the model ranks what the words leave open, over tagged refs and lines; not listed while off; on #565 |
| [TICKET-569](open/TICKET-569-stalled-or-looping-agents.md) | feature | prong 2 · a flag on a working agent's row when it loops or stalls, from local facts first (repeats, the process tree's CPU, Marley's own waits) and a System One kind second; never a stop; on #565 |
| [TICKET-571](open/TICKET-571-pause-before-a-consequential-click.md) | feature | prong 3 with prong 2 · an agent's click that pays, deletes, sends or changes an account waits on Allow or Refuse in the Browser tab, by default only for agents with no prompt of their own; rules first, a System One noul second; on #565 |
| [TICKET-508](open/TICKET-508-approvals-inbox.md) | feature | prong 2 · every agent's pending permission prompt in one list in the rail; on #519 |
| [TICKET-568](open/TICKET-568-inbox-order-and-risk-chips.md) | feature | prong 2 · #508's inbox ordered by level then age, with risk chips from code's rules and chips the model may add but never remove; approves nothing; on #508 and #565 |
| [TICKET-570](open/TICKET-570-who-answers-a-question.md) | feature | prong 2 · each inbox entry marked for you, the manager, could proceed or unclear, from #568's chips first and a System One choice second; refines #568's order within a level; Marley answers nothing; on #565 |
| [TICKET-509](open/TICKET-509-per-turn-diffs.md) | feature | prong 2 · Claude Code's turns as snapshots, each turn's diff in Zed's commit view; on #519 |
| [TICKET-532](open/TICKET-532-agent-permission-modes.md) | feature | prong 2 · Claude Code's bypass or Codex's full access as a setting, shown on the rail row |
| [TICKET-510](open/TICKET-510-worktree-agents.md) | feature | prong 2 · New Agent in Worktree: an agent on its own branch and worktree, under its project in the rail |
| [TICKET-560](open/TICKET-560-conflict-chip.md) | feature | prong 2 · a chip on worktree rows: commits behind the base and the files a merge would stop on, from three read-only git calls; after #510 |
| [TICKET-511](open/TICKET-511-review-and-merge-a-worktree.md) | feature | prong 2 · review a worktree agent's branch and merge it, deferring to the Rustal workflow where it runs |
| [TICKET-522](open/TICKET-522-review-notes-to-the-agent.md) | feature | prong 2 · review notes sent to the agent working in the diffed tree |
| [TICKET-531](open/TICKET-531-pr-state-on-rail-rows.md) | feature | workbench · pull request state and diff counts on the rail's project rows |
| [TICKET-527](open/TICKET-527-project-launch-configs.md) | feature | workbench · a project's launch configs (`.zed/marley.json`) in the rail's + |
| [TICKET-525](open/TICKET-525-agent-drives-a-running-program.md) | feature | prong 2 with prong 1 · an agent reads and types into a running program, with takeover |
| [TICKET-526](open/TICKET-526-blocks-over-ssh.md) | feature | prong 1 T0c · blocks keep working over ssh |
| [TICKET-554](open/TICKET-554-block-selection-and-menu.md) | feature | prong 1 T1 · a selected block and the block menu: Copy Command, Both, as Markdown, Reinput, Reinput with sudo |
| [TICKET-555](open/TICKET-555-send-a-block-to-the-agent.md) | feature | prong 2 with prong 1 · a block sent into a terminal agent's prompt, and Ask the agent under a failed block; after #549 and #554 |
| [TICKET-528](open/TICKET-528-block-filter.md) | feature | prong 1 T1 · filter a block's output |
| [TICKET-558](open/TICKET-558-save-as-workflow.md) | feature | prong 1 T4 · Save as Workflow writes a block's command as a task in tasks.json with `{{name}}` parameters; runnable from Zed's task picker; after #528 |
| [TICKET-559](open/TICKET-559-block-bookmarks-and-find-in-block.md) | feature | prong 1 T1 · bookmarks on blocks with Alt+Up/Down and scrollbar ticks; find within a block on Zed's search bar |
| [TICKET-529](open/TICKET-529-sticky-command-header.md) | feature | prong 1 T1 · a long block's command stays in view while scrolled |
| [TICKET-530](open/TICKET-530-runnable-markdown-commands.md) | feature | prong 1 · shell commands in the Markdown preview go to the terminal |
| [TICKET-536](open/TICKET-536-agent-aware-copy-and-paste.md) | feature | prong 1 T7 · copy and paste that know an agent is running |
| [TICKET-537](open/TICKET-537-git-credential-prompts-off.md) | feature | prong 1 T7 · git credential prompts off for the agents Marley starts |
| [TICKET-538](open/TICKET-538-notifications-with-content.md) | feature | prong 1 T7b · notifications that say what happened; after #519 |
| [TICKET-551](open/TICKET-551-command-end-from-outside.md) | feature | prong 1 T7b · a long command's end as a notification and on its rail row, with the password check; never agent terminals |
| [TICKET-572](open/TICKET-572-running-command-errors.md) | feature | prong 1 T7b · a dev server that prints an error and keeps running gets a notification and a red mark on its row, recovery clears it; error shapes first, a System One noul for the open lines; after #551, on #565 |
| [TICKET-556](open/TICKET-556-terminal-run.md) | feature | prong 2 with prong 1 · `terminal_run`: an agent runs a command at the user's prompt as a block, behind Warp's allow and deny lists, with the agent mark and Ctrl-I takeover |
| [TICKET-553](open/TICKET-553-agent-commands-in-history.md) | feature | prong 1 T7 · whether an agent's commands enter the shell history and the suggestions, as a setting; after #556 |
| [TICKET-557](open/TICKET-557-inline-assist-and-english-at-the-prompt.md) | feature | prong 1 T3 · Inline Assist proven by a scenario; English at the prompt by local rules, a hint, Ctrl+Shift+Enter to the agent and the exit-127 button |
| [TICKET-552](open/TICKET-552-codex-and-opencode-notifications.md) | feature | prong 1 T7b · the agent bar sets up Codex's and OpenCode's notifications in a click, as it connects Claude Code |
| [TICKET-563](open/TICKET-563-terminal-shortcut-note.md) | feature | prong 1 T7 · a toast the first time one of Marley's keys is taken from a terminal program, once per action per data directory |
| [TICKET-564](open/TICKET-564-project-icons-from-the-repo.md) | feature | workbench · the rail's project headers show the repository's favicon or logo, or the icon its index.html declares; nothing fetched |
| [TICKET-542](open/TICKET-542-rail-attention-order.md) | feature | prong 2 · the rail puts what needs Chad first; after #519 |
| [TICKET-543](open/TICKET-543-remote-terminals-survive-a-drop.md) | feature | prong 2 · remote terminals that survive a dropped link (tmux on the host) |
| [TICKET-533](open/TICKET-533-harness-contract-alignment.md) | chore | prong 2 · the harness's contract requests answered, and the plan's harness text corrected |
| [TICKET-541](open/TICKET-541-spawn-in-adapters-ratchet.md) | chore | gate · process spawns held to adapter modules by a ratchet |
| [TICKET-545](open/TICKET-545-activation-token-hand-off.md) | feature | the app · a second launch hands its launcher's activation token to the running Marley, so a compositor that checks tokens brings the window forward (#513's follow-up) |
| [TICKET-486](open/TICKET-486-keep-the-terminal-size-across-launches.md) | bug | prong 1 T0 · the first terminals of a launch open at the last session's size (#485's limit) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-578](open/TICKET-578-restored-browser-tab-draws-its-reopened-page.md) | bug | prong 3 · a restored Browser tab sometimes draws nothing (2 of 6 runs of #576's scenario); waits for a reproduction that logs the restored page's screencast frames in a blank run |
| [TICKET-534](open/TICKET-534-harness-sessions-in-the-rail.md) | feature | prong 2 C1 · harness sessions in the rail, the read side; waits for the harness's M9 to exit, and for #533 and #508 |
| [TICKET-573](open/TICKET-573-english-at-the-prompt-second-stage.md) | feature | prong 1 T3 · a System One reading for typed lines #557's local rules leave open, in the hint slot after 250 ms without typing, never in Enter's path; opt-in, redacted; waits on #557 and #565, and on a listed project or a local provider |
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-540](open/TICKET-540-session-resume-after-restart.md) | feature | prong 2 · Claude Code sessions resumed after a restart; waits on how the embedded harness keeps processes alive, so it cannot start a second Claude on a live session |
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-466](open/TICKET-466-fish-shell-integration.md) | feature | prong 1 T0c · shell integration for fish; waits for fish on a machine that can run its tests (the dev box has none) |
| [TICKET-446](open/TICKET-446-marley-crate-license-files.md) | chore | licensing: `LICENSE-APACHE` and `LICENSE-MIT` in every Marley crate; waits for Chad's copyright line for the MIT text |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
| [TICKET-417](open/TICKET-417-318-after-capture-battery.md) | chore | gpui-era, macOS harness; obsolete in the Zed fork (the overlays it captured no longer exist) — close or re-scope when the block terminal's overlays land |
| [TICKET-271](open/TICKET-271-headless-pixel-capture-gpui.md) | chore | gpui-era; the fork builds gpui from the tree, so the wait is over — re-scope as "headless pixel captures for validate" when a UI slice needs it |
