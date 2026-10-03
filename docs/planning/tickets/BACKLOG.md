# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-642](open/TICKET-642-dictation-and-rusty-tools-off-by-default.md) | feature | prong 1 T7d, prong 2 C2 · the dictation mic waits for `marley.voice.enabled` and `marley.rusty_tools` defaults to false (Chad, 2026-10-02: "Both off by default") |
| [TICKET-640](open/TICKET-640-agent-state-source-progress-quota.md) | feature | prong 2 C1 · a harness session's state source (detected drawn weaker, never in the inbox), progress and quota on its rail row (rustal-harness MREQ-005 to MREQ-007) |
| [TICKET-641](open/TICKET-641-ssh-links-that-know-they-are-dead.md) | feature | prong 2 · ssh keepalive on every link Marley starts; a remote terminal whose link died stays dimmed with input off and reattaches on a backoff to two minutes (herdr's habits, the harness's withdrawn TICKET-094) |
| [TICKET-643](open/TICKET-643-rusty-switch-and-connection.md) | feature | Rusty in Marley R1 · `marley.rusty` (off by default): Marley starts `rusty-mcp` or connects to the service, status and the embedding provider in a Rusty settings section; `rusty_tools` moves in (after #642) |
| [TICKET-644](open/TICKET-644-brain-view-in-the-rail.md) | feature | Rusty in Marley R4 · the rail's Projects and Brain switch; Today, search on Enter and the vault tree with writes through Rusty's tools |
| [TICKET-645](open/TICKET-645-brain-page-tab.md) | feature | Rusty in Marley R2 · a brain page in a center tab: wikilinks, properties, back and forward, preview tabs, Edit in a buffer |
| [TICKET-646](open/TICKET-646-knowledge-panel.md) | feature | Rusty in Marley R3 · the right-dock Knowledge panel: backlinks, outgoing links, tags and brain search |
| [TICKET-647](open/TICKET-647-brain-graph-tab.md) | feature | Rusty in Marley R5 · the Graph tab: whole or local, filters, decision edges, Ely's force layout off the main thread |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
