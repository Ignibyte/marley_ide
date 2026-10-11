# TICKET-752 — Codex push and answers

- **Ticket:** LOCAL #752 (feature, prong 2: "Codex gets what Claude Code gets", first half; T3 Code survey item 5)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [752-codex-push-and-answers.spec.md](../../pipeline/queued/752-codex-push-and-answers.spec.md)
- **Source ticket:** [T3 Code survey](../../../t3code_architecture/README.md), item 5; reports 04 (item 1) and 01 (item 5). Chad, 2026-10-11, on "Codex gets what Claude Code gets": "3.) yes".
- **Status:** open

## Summary
Phone push for Codex and for Zed's agent threads, as Claude Code has it, with a switch per kind
of event after T3 Code's: approval, input, finish and failure. Today `push.rs` builds every line with
`AgentKind::Claude`, so a Codex on its App Server and a thread in the Agent Panel or a thread tab
push nothing. The same ticket answers Codex's own questions (`item/tool/requestUserInput`) and its
app-access requests (an MCP elicitation with nothing to fill in) from Needs you through the App
Server, as #651 answers its approvals. Today those go to the terminal.

## Acceptance
A Codex on its App Server and a Zed agent thread push `<project>: <agent> <event>` to the phone
for the kinds switched on, and nothing for a kind switched off. A Codex question with options and
a Codex tool approval are answered from Needs you, and the TUI's prompt closes.
