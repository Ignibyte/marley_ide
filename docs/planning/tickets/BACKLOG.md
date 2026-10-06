# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-664](open/TICKET-664-rusty-memory-tab.md) | feature | Rusty in Marley R8 · the Memory tab over Rusty's four memory tools |
| [TICKET-665](open/TICKET-665-rusty-skills-tab.md) | feature | Rusty in Marley R8 · the Skills tab: pending skills to approve, skills and scripts to edit, a script run in a terminal |
| [TICKET-666](open/TICKET-666-rusty-settings-on-the-settings-page.md) | feature | Rusty in Marley R8 · every Rusty setting on the Rusty's Server page, credentials masked |
| [TICKET-667](open/TICKET-667-rusty-secrets-tab.md) | feature | Rusty in Marley R8 · the Secrets tab behind Rusty's PIN; no secret in Zed's logs |
| [TICKET-668](open/TICKET-668-test-runs-act-on-nothing-of-the-users.md) | chore | the e2e harness · a run's profile keeps none of the user's push, harness, fleet or System One settings or keys |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
