# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-547](open/TICKET-547-claude-code-events-slice-2.md) | feature | prong 2 C1 · #519's second slice: the plugin update chip, `fleet_snapshot` fed from the rail's seats, `no update in N m` |
| [TICKET-520](open/TICKET-520-terminal-identity.md) | feature | prong 2 · each terminal knows its id and project, and Marley's tools know which terminal called them |
| [TICKET-503](open/TICKET-503-terminal-urls-open-in-the-browser.md) | feature | prong 3 with prong 1 · a local URL in a terminal opens a Browser tab; the terminal offers a dev server's URL while its port listens |
| [TICKET-504](open/TICKET-504-browser-tabs-in-the-rail.md) | feature | prong 3 · Browser tabs as rows of their project in the rail |
| [TICKET-518](open/TICKET-518-fuller-pick-bundle.md) | feature | prong 3 · the pick bundle gains HTML, styles and the React component with its source; before #505 |
| [TICKET-505](open/TICKET-505-pick-fix-check.md) | feature | prong 3 · a pick re-found after the fix: before and after in the tray, and a tool |
| [TICKET-506](open/TICKET-506-recording-to-playwright-test.md) | feature | prong 3 · clicks and typing recorded as locators; a recording drafted as a Playwright test (D3 asks Chad about typed text) |
| [TICKET-507](open/TICKET-507-browser-context-per-project.md) | feature | prong 3 · a Chromium and a profile per project; worktrees share their project's |
| [TICKET-523](open/TICKET-523-saved-playwright-scripts.md) | feature | prong 3 · Playwright scripts saved in Marley and run on a Browser tab |
| [TICKET-524](open/TICKET-524-trusted-outside-browser-access.md) | feature | prong 3 with prong 2 · trusted outside clients drive Marley's browser, slice 1 of 3 |
| [TICKET-539](open/TICKET-539-headless-chrome-user-agent.md) | chore | prong 3 · Marley's Chromium introduces itself as Chrome, not HeadlessChrome |
| [TICKET-521](open/TICKET-521-ports-per-project.md) | feature | prong 3 · listening ports per project in the rail, with Open, Copy and Stop |
| [TICKET-508](open/TICKET-508-approvals-inbox.md) | feature | prong 2 · every agent's pending permission prompt in one list in the rail; on #519 |
| [TICKET-509](open/TICKET-509-per-turn-diffs.md) | feature | prong 2 · Claude Code's turns as snapshots, each turn's diff in Zed's commit view; on #519 |
| [TICKET-532](open/TICKET-532-agent-permission-modes.md) | feature | prong 2 · Claude Code's bypass or Codex's full access as a setting, shown on the rail row |
| [TICKET-510](open/TICKET-510-worktree-agents.md) | feature | prong 2 · New Agent in Worktree: an agent on its own branch and worktree, under its project in the rail |
| [TICKET-511](open/TICKET-511-review-and-merge-a-worktree.md) | feature | prong 2 · review a worktree agent's branch and merge it, deferring to the Rustal workflow where it runs |
| [TICKET-522](open/TICKET-522-review-notes-to-the-agent.md) | feature | prong 2 · review notes sent to the agent working in the diffed tree |
| [TICKET-531](open/TICKET-531-pr-state-on-rail-rows.md) | feature | workbench · pull request state and diff counts on the rail's project rows |
| [TICKET-527](open/TICKET-527-project-launch-configs.md) | feature | workbench · a project's launch configs (`.zed/marley.json`) in the rail's + |
| [TICKET-525](open/TICKET-525-agent-drives-a-running-program.md) | feature | prong 2 with prong 1 · an agent reads and types into a running program, with takeover |
| [TICKET-526](open/TICKET-526-blocks-over-ssh.md) | feature | prong 1 T0c · blocks keep working over ssh |
| [TICKET-528](open/TICKET-528-block-filter.md) | feature | prong 1 T1 · filter a block's output |
| [TICKET-529](open/TICKET-529-sticky-command-header.md) | feature | prong 1 T1 · a long block's command stays in view while scrolled |
| [TICKET-530](open/TICKET-530-runnable-markdown-commands.md) | feature | prong 1 · shell commands in the Markdown preview go to the terminal |
| [TICKET-536](open/TICKET-536-agent-aware-copy-and-paste.md) | feature | prong 1 T7 · copy and paste that know an agent is running |
| [TICKET-537](open/TICKET-537-git-credential-prompts-off.md) | feature | prong 1 T7 · git credential prompts off for the agents Marley starts |
| [TICKET-538](open/TICKET-538-notifications-with-content.md) | feature | prong 1 T7b · notifications that say what happened; after #519 |
| [TICKET-542](open/TICKET-542-rail-attention-order.md) | feature | prong 2 · the rail puts what needs Chad first; after #519 |
| [TICKET-543](open/TICKET-543-remote-terminals-survive-a-drop.md) | feature | prong 2 · remote terminals that survive a dropped link (tmux on the host) |
| [TICKET-533](open/TICKET-533-harness-contract-alignment.md) | chore | prong 2 · the harness's contract requests answered, and the plan's harness text corrected |
| [TICKET-541](open/TICKET-541-spawn-in-adapters-ratchet.md) | chore | gate · process spawns held to adapter modules by a ratchet |
| [TICKET-545](open/TICKET-545-activation-token-hand-off.md) | feature | the app · a second launch hands its launcher's activation token to the running Marley, so a compositor that checks tokens brings the window forward (#513's follow-up) |
| [TICKET-486](open/TICKET-486-keep-the-terminal-size-across-launches.md) | bug | prong 1 T0 · the first terminals of a launch open at the last session's size (#485's limit) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-534](open/TICKET-534-harness-sessions-in-the-rail.md) | feature | prong 2 C1 · harness sessions in the rail, the read side; waits for the harness's M9 to exit, and for #533 and #508 |
| [TICKET-535](open/TICKET-535-phone-push-notifications.md) | feature | prong 2 remote · agent events pushed to the phone through ntfy; the phone path starts when Chad picks it, and it needs #519 |
| [TICKET-540](open/TICKET-540-session-resume-after-restart.md) | feature | prong 2 · Claude Code sessions resumed after a restart; waits on how the embedded harness keeps processes alive, so it cannot start a second Claude on a live session |
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-466](open/TICKET-466-fish-shell-integration.md) | feature | prong 1 T0c · shell integration for fish; waits for fish on a machine that can run its tests (the dev box has none) |
| [TICKET-446](open/TICKET-446-marley-crate-license-files.md) | chore | licensing: `LICENSE-APACHE` and `LICENSE-MIT` in every Marley crate; waits for Chad's copyright line for the MIT text |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
| [TICKET-417](open/TICKET-417-318-after-capture-battery.md) | chore | gpui-era, macOS harness; obsolete in the Zed fork (the overlays it captured no longer exist) — close or re-scope when the block terminal's overlays land |
| [TICKET-271](open/TICKET-271-headless-pixel-capture-gpui.md) | chore | gpui-era; the fork builds gpui from the tree, so the wait is over — re-scope as "headless pixel captures for validate" when a UI slice needs it |
