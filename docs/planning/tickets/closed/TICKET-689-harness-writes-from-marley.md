# TICKET-689 — Harness writes from Marley

- **Ticket:** LOCAL #689 (feature, phase 2 item 4 of the manager plan)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [689-harness-writes-from-marley.spec.md](../../pipeline/completed/689-harness-writes-from-marley.spec.md)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2, item 4
- **Status:** closed

## Summary
Since #534 Marley follows the harness's fleet read-only. This ticket adds the harness's write
verbs to a session's tab:
- answer the question it waits on (`session_answer`);
- send it text (`session_send`);
- list the commands that watch it (`session_surface_to_human`);
- open a new session from a declared profile (`session_open`).

`marley.harness_writes` turns these on, off by default. With it on, the embedded harness is
followed under `--grant write`. A `marley.harness` command keeps its own arguments, and a write
it is not granted is refused, with the harness's reason shown in the tab.

## Acceptance
With `marley.harness_writes` on:
- a question's option clicked in the tab answers it;
- text sent from the tab reaches an actor waiting for a message;
- Views lists the commands that watch the session;
- `marley: open harness session` with a profile name opens a session and its tab.

Off, the tab is read-only as before.
