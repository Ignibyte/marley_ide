# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-544](open/TICKET-544-blocks-survive-a-rewrap.md) | bug | prong 1 T0 · a resize that rewraps the terminal moves every block's rows; the bars, pills and `terminal_read` follow them again (the plan's D2) |
| [TICKET-503](open/TICKET-503-terminal-urls-open-in-the-browser.md) | feature | prong 3 with prong 1 · a local URL in a terminal opens a Browser tab; the terminal offers a dev server's URL |
| [TICKET-504](open/TICKET-504-browser-tabs-in-the-rail.md) | feature | prong 3 · Browser tabs as rows of their project in the rail |
| [TICKET-505](open/TICKET-505-pick-fix-check.md) | feature | prong 3 · a pick re-found after the fix: before and after in the tray, and a tool |
| [TICKET-506](open/TICKET-506-recording-to-playwright-test.md) | feature | prong 3 · clicks and typing recorded as locators; a recording drafted as a Playwright test |
| [TICKET-507](open/TICKET-507-browser-context-per-project.md) | feature | prong 3 · a browser context per project, its cookies kept across restarts |
| [TICKET-508](open/TICKET-508-approvals-inbox.md) | feature | prong 2 · every agent's pending permission prompt in one list in the rail |
| [TICKET-509](open/TICKET-509-per-turn-diffs.md) | feature | prong 2 · Claude Code's turns in a terminal as snapshots, each turn's diff in a review view |
| [TICKET-510](open/TICKET-510-worktree-agents.md) | feature | prong 2 · the + starts an agent in a new worktree on its own branch, its own rail project and ports |
| [TICKET-511](open/TICKET-511-review-and-merge-a-worktree.md) | feature | prong 2 · review a worktree agent's branch, merge it, remove the worktree |
| [TICKET-486](open/TICKET-486-keep-the-terminal-size-across-launches.md) | bug | prong 1 T0 · the first terminals of a launch open at the last session's size (#485's limit) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-466](open/TICKET-466-fish-shell-integration.md) | feature | prong 1 T0c · shell integration for fish; waits for fish on a machine that can run its tests (the dev box has none) |
| [TICKET-446](open/TICKET-446-marley-crate-license-files.md) | chore | licensing: `LICENSE-APACHE` and `LICENSE-MIT` in every Marley crate; waits for Chad's copyright line for the MIT text |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
| [TICKET-417](open/TICKET-417-318-after-capture-battery.md) | chore | gpui-era, macOS harness; obsolete in the Zed fork (the overlays it captured no longer exist) — close or re-scope when the block terminal's overlays land |
| [TICKET-271](open/TICKET-271-headless-pixel-capture-gpui.md) | chore | gpui-era; the fork builds gpui from the tree, so the wait is over — re-scope as "headless pixel captures for validate" when a UI slice needs it |
