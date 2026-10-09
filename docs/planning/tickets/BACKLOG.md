# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-712](open/TICKET-712-lsp-tools-for-agents.md) | feature | `editor_diagnostics`, `editor_definition`, `editor_references`, `editor_hover` for every agent, over Zed's LSP store |
| [TICKET-713](open/TICKET-713-rail-cleanup.md) | chore | the rail: unregister a swapped sidebar, refresh only on row changes, a silent swap, save after restore |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-710](open/TICKET-710-the-marley-agent-removes-a-seat.md) | feature | `seat_remove` for the Manager; waits on the harness's `rh seat stop` and `rh seat remove` (asked of rustal-harness 2026-10-09) |
| [TICKET-714](open/TICKET-714-undo-a-turn.md) | feature | from MonoCode (usemono.dev), for future use: undo a turn (small to medium) |
| [TICKET-715](open/TICKET-715-usage-meter-and-resume-at-reset.md) | feature | from MonoCode (usemono.dev), for future use: a usage meter, and resume at the reset (medium) |
| [TICKET-716](open/TICKET-716-delegation-tools.md) | feature | from MonoCode (usemono.dev), for future use: delegation tools on marley's mcp server (medium) |
| [TICKET-717](open/TICKET-717-second-opinion-and-handoff.md) | feature | from MonoCode (usemono.dev), for future use: second opinion and handoff (medium) |
| [TICKET-718](open/TICKET-718-failed-checks-to-an-agent.md) | feature | from MonoCode (usemono.dev), for future use: failed checks to an agent, then a github inbox (medium, then large) |
| [TICKET-719](open/TICKET-719-scheduled-runs.md) | feature | from MonoCode (usemono.dev), for future use: scheduled runs (medium to large) |
| [TICKET-720](open/TICKET-720-mcp-manager-and-cli-update-checks.md) | feature | from MonoCode (usemono.dev), for future use: an mcp manager and agent cli update checks (small to medium) |
| [TICKET-721](open/TICKET-721-a-quick-prompt.md) | feature | from MonoCode (usemono.dev), for future use: a quick prompt (small) |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
