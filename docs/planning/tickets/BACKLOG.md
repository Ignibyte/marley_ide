# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-613](open/TICKET-613-move-a-terminal-to-another-project.md) | feature | rail (wave 2) · move a terminal to another project, its shell kept |
| [TICKET-614](open/TICKET-614-container-ports-in-the-rail.md) | feature | rail (wave 2) · container ports named by their container, Stop stopping it |
| [TICKET-615](open/TICKET-615-restart-a-service-from-its-port-row.md) | feature | rail (wave 2) · restart a service from its port row, its state and logs |
| [TICKET-616](open/TICKET-616-delete-and-unarchive-threads-from-the-rail.md) | feature | rail (wave 2) · delete a thread, and unarchive threads, from the rail |
| [TICKET-617](open/TICKET-617-rows-under-a-closed-project.md) | feature | rail (wave 2) · a closed project's threads and ports under its header |
| [TICKET-618](open/TICKET-618-port-row-lines-and-header-tooltip.md) | bug | rail (wave 2) · a port row's clipped lines, and a header tooltip over its menu |
| [TICKET-612](open/TICKET-612-settings-edit-after-marley-writes.md) | bug | workbench · a hand edit to settings.json after Marley writes the file does not reload |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-578](open/TICKET-578-restored-browser-tab-draws-its-reopened-page.md) | bug | prong 3 · a restored Browser tab sometimes draws nothing (2 of 6 runs of #576's scenario); waits for a reproduction that logs the restored page's screencast frames in a blank run |
| [TICKET-534](open/TICKET-534-harness-sessions-in-the-rail.md) | feature | prong 2 C1 · harness sessions in the rail, the read side; its blockers (#533, #508, the harness's M9) are done, and it is built in wave 4 of `design-notes/remaining-work-2026-09-30.md` |
| [TICKET-573](open/TICKET-573-english-at-the-prompt-second-stage.md) | feature | prong 1 T3 · a System One reading for typed lines #557's local rules leave open, in the hint slot after 250 ms without typing, never in Enter's path; opt-in, redacted; waits on #557 and #565, and on a listed project or a local provider |
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-540](open/TICKET-540-session-resume-after-restart.md) | feature | prong 2 · Claude Code sessions resumed after a restart; waits on how the embedded harness keeps processes alive, so it cannot start a second Claude on a live session |
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-466](open/TICKET-466-fish-shell-integration.md) | feature | prong 1 T0c · shell integration for fish; waits for fish on a machine that can run its tests (the dev box has none) |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
