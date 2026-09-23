# TICKET-477 — The agent bar, with the folder and branch

- **Ticket:** LOCAL #477 (feature, prong 1: T7a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/477-agent-bar.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T7)
- **Status:** open

## Summary
When a CLI agent such as Claude Code runs in a Warp session, a bar appears under it with the
agent's controls on the left and the folder and git branch on the right. Marley gets the same
bar under a terminal whose foreground program is a known agent. This ticket builds the bar
with the agent's name and the folder and branch; #478 to #481 add its buttons.

## Acceptance
With an agent in the foreground, the bar shows below the terminal's grid with the folder and,
inside a repository the project tracks, its branch; without one, no bar. The EARS criteria are
in the spec.
