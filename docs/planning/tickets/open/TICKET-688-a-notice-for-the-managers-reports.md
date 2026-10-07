# TICKET-688 — A notice for the manager's reports

- **Ticket:** LOCAL #688 (feature, prong 2 C)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** (set at promotion)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2; rustal-harness
  TICKET-106 (the thread) and TICKET-111 (`rh acp`), its D179
- **Status:** open

## Summary
The Agent Panel shows a manager's report sent between turns but raises nothing for it (#685).
The harness keeps each thread record's author, kind and time and serves the thread by cursor and
subscription (its TICKET-106); it sends no desktop notice of its own and leaves that to Marley.
Marley follows the manager's thread and, for a manager record of kind report or confirmation that
arrives while its thread is not in front, raises a notice: a desktop notification and the manager
thread's rail row marked, the same way Marley raises an agent that needs the user. Waits on the
harness's TICKET-106.

## Acceptance
With a harness manager posting a report while the user looks at another tab, Marley shows a desktop
notice naming the manager and the report's first line, and the manager thread's row is marked
until the user opens it; a report while the thread is in front raises no notice.
