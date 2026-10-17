# TICKET-797 — Merge upstream Zed through 2026-10-17

- **Ticket:** LOCAL #797 (chore, upstream sync)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** [797-merge-upstream-zed.spec.md](../../pipeline/completed/797-merge-upstream-zed.spec.md)
- **Source ticket:** Chad, 2026-10-17: "Lets also look any zed releases and bring that code down so we arent behind"
- **Status:** closed

## Summary
The fork carries upstream Zed's main of 2026-10-08 (`9ab0715969`, #695). Since then upstream
main has 110 more commits through `56934ec056` (2026-10-17), which hold everything in v1.24.3
and v1.25.0-pre: Zed cuts its releases from main. Most of them move the agent protocol to ACP
v2. A trial merge (`git merge-tree`) conflicts in two files, `crates/agent_servers/src/acp.rs`
and `crates/agent_ui/src/conversation_view.rs`. The merge goes first in the agent-components
batch because #798 and #800 edit the agent files upstream is rewriting.

## Acceptance
The fork builds on upstream's main of 2026-10-17 with every Marley hunk kept, the gate is
green, and the golden e2e set passes as it did after #695.
