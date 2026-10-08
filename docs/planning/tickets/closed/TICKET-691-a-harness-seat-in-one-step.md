# TICKET-691 — A harness seat in one step

- **Ticket:** LOCAL #691 (feature, phase 2 item 6 of the manager plan)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [691-a-harness-seat-in-one-step.spec.md](../../pipeline/completed/691-a-harness-seat-in-one-step.spec.md)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2, item 6
- **Status:** closed

## Summary
Setting up a manager or any other seat is one form. `marley: new harness seat` asks for a name,
the agent (Claude Code or Codex), a folder and a role. Marley then runs harness TICKET-109's
`rh seat add` and `rh seat start` through the command it follows the harness with, so a harness
over SSH gets them over SSH, and opens the new session's tab. A refusal shows the harness's code
and reason in the form.

## Acceptance
With `marley.harness_writes` on and a harness followed, the form adds and starts a seat and opens
its tab. A refused seat, such as the reserved role `foreman`, keeps the form open with the
harness's reason.
