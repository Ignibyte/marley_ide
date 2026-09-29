# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-564](open/TICKET-564-project-icons-from-the-repo.md) | feature | workbench · the rail's project headers show the repository's favicon or logo, or the icon its index.html declares; nothing fetched |
| [TICKET-542](open/TICKET-542-rail-attention-order.md) | feature | prong 2 · the rail puts what needs Chad first; after #519 |
| [TICKET-543](open/TICKET-543-remote-terminals-survive-a-drop.md) | feature | prong 2 · remote terminals that survive a dropped link (tmux on the host) |
| [TICKET-533](open/TICKET-533-harness-contract-alignment.md) | chore | prong 2 · the harness's contract requests answered, and the plan's harness text corrected |
| [TICKET-588](open/TICKET-588-e2e-cleanup-after-the-compositor-exits.md) | chore | e2e · the runner's cleanup finishes and says so when its headless sway exits partway (#560's golden run) |
| [TICKET-541](open/TICKET-541-spawn-in-adapters-ratchet.md) | chore | gate · process spawns held to adapter modules by a ratchet |
| [TICKET-545](open/TICKET-545-activation-token-hand-off.md) | feature | the app · a second launch hands its launcher's activation token to the running Marley, so a compositor that checks tokens brings the window forward (#513's follow-up) |
| [TICKET-486](open/TICKET-486-keep-the-terminal-size-across-launches.md) | bug | prong 1 T0 · the first terminals of a launch open at the last session's size (#485's limit) |
| [TICKET-596](open/TICKET-596-ssh-passphrases-asked-in-marley.md) | feature | prong 1 T7 · an agent's ssh asks for a key's passphrase in a Marley dialog through `SSH_ASKPASS`; split from #537 (Chad's answer, 2026-09-26) |

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
