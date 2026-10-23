# TICKET-946 — Forge's board and online agents as the board draws them

- **Ticket:** LOCAL #946 (feature, the Marley look, Forge)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [946-forge-as-the-board.spec.md](../../pipeline/queued/946-forge-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/G-system.md, boards ForgeBoard and ForgeAgents)
- **Boards:** ForgeBoard, ForgeAgents
- **Size:** M
- **Status:** open

## Summary

Forge's sprint board (#842) and its agents in the Fleet panel (#841, #843) work, but they say less
than the ForgeBoard and ForgeAgents boards. The board's kinds are accent text whatever the kind, its
columns have no status dot, and a card names its agent with no tile. The detail pane has no phase
progress, no callout when QA failed, no comment authors, and no way to open the ticket's agent. In
the Fleet panel a Forge agent's row packs its ticket and folder into one line, with no harness and
no time since its last output, and a host's header has no agent count. Nothing shows the
connection: the endpoint, where the token came from, the projects and how fast Forge answers. This
ticket draws all of that. A host's role ("forge-brain · forge-mcp · forge-sched") goes to a
follow-up: Forge's `agent-list` does not report one.

## Acceptance

The board shows coloured kind pills, phase chips, column dots and agent tiles; the detail pane
shows the phase strip, the QA failed callout, comment authors and Open the agent; the Fleet panel's
Forge rows show harness, ticket title, folder, state and output age under hosts with their agent
count, and the Forge source shows its connection. Shot in Marley Night and Marley Day.
