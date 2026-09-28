# TICKET-508 — One approvals inbox in the rail

- **Ticket:** LOCAL #508 (feature, prong 2 (attention); built on #519)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/508-approvals-inbox.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 6 of the list after the browser waves). Revised the same night from the Orca survey's notes on #508 (`docs/orca_architecture/README.md`, "What it changes in the queued sprint"; report 01 item 2, report 04 item 1, report 06 item 3).
- **Status:** closed

## Summary
Agents ask for permission in different places: Claude Code in a terminal, the Zed Agent and external agents in the Agent Panel. The rail gains one "needs you" list at its top, oldest first, each entry naming the agent, its project, what it asks (`Bash: rm -rf build`) and how long it has waited. An Agent Panel entry is answered in place with the offered Allow and Deny, through the panel's own answer path, which ends in `AcpThread::authorize_tool_call`. A terminal entry comes from #519's PermissionRequest events, and a click opens its terminal. A pick sent from the Browser tab is refused, not pasted, while its terminal's agent waits. Answering a terminal agent from the rail and the harness's questions are later slices.

## Acceptance
A pending Agent Panel tool call and a Claude Code permission prompt in a terminal both show in the rail's inbox, oldest first; Allow or Deny on the first answers it and removes it; a click on the second shows its terminal, and it leaves once the agent moves on; a pick is not pasted into a terminal whose agent waits.
