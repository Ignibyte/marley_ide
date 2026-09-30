# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-601](open/TICKET-601-projectless-groups-survive-a-restart.md) | feature | workbench shell · the rail: #600's groups come back after a restart with their names, order and items |
| [TICKET-602](open/TICKET-602-drag-to-reorder-the-rail.md) | feature | workbench shell · the rail: drag headers and rows to reorder them within their group; the order is saved; ties within #542's attention classes follow it |
| [TICKET-603](open/TICKET-603-service-aware-port-rows.md) | feature | workbench shell · port rows: a row names the systemd service behind its listener, and Stop stops the unit (`systemctl --user stop`, or `systemctl stop` through polkit with the sudo command to copy when refused) |
| [TICKET-604](open/TICKET-604-port-row-click-does-not-open.md) | feature | workbench shell · port rows: one click marks the row; Open, a double-click or Enter opens its URL in a Browser tab |
| [TICKET-605](open/TICKET-605-archive-thread-rows.md) | feature | workbench shell · thread rows: an Archive button on hover and a right-click Archive Thread, as Zed's thread history archives |

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
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
