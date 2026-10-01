# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-632](open/TICKET-632-the-embedded-harness.md) | feature | prong 2 · the embedded harness: Marley starts `rh serve` on a root of its own and follows it (D19) |
| [TICKET-633](open/TICKET-633-rusty-tools-for-zeds-agents.md) | feature | prong 2 C2 · Rusty's tools for Zed's agents when `rusty-mcp` is installed (D11) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-540](open/TICKET-540-session-resume-after-restart.md) | feature | prong 2 · Claude Code sessions resumed after a restart; waits on how the embedded harness keeps processes alive, so it cannot start a second Claude on a live session |
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-631](open/TICKET-631-block-gaps-and-density.md) | feature | prong 1 T5 · gaps between blocks and a density setting, split from #630: rows that are no grid line need a display-row map through 17 sites of Zed's terminal element; waits for Chad to weigh the look against the upstream diff |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
