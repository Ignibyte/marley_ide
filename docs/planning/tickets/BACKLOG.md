# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-661](open/TICKET-661-rusty-off-leaves-no-trace.md) | chore | Rusty in Marley · with the switch off, no `rusty:` command in the palette and only the switch on the settings page |
| [TICKET-662](open/TICKET-662-rusty-favourites.md) | feature | Rusty in Marley R4b · favourites above the Brain tree, a star on a page's tab, favourites first in the page picker |
| [TICKET-663](open/TICKET-663-rusty-quick-capture-and-import.md) | feature | Rusty in Marley · capture a line or a URL and import a vault from the palette; longer deadlines for those calls |
| [TICKET-664](open/TICKET-664-rusty-memory-tab.md) | feature | Rusty in Marley R8 · the Memory tab over Rusty's four memory tools |
| [TICKET-665](open/TICKET-665-rusty-skills-tab.md) | feature | Rusty in Marley R8 · the Skills tab: pending skills to approve, skills and scripts to edit, a script run in a terminal |
| [TICKET-666](open/TICKET-666-rusty-settings-on-the-settings-page.md) | feature | Rusty in Marley R8 · every Rusty setting on the Rusty's Server page, credentials masked |
| [TICKET-667](open/TICKET-667-rusty-secrets-tab.md) | feature | Rusty in Marley R8 · the Secrets tab behind Rusty's PIN; no secret in Zed's logs |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
