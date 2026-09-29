# TICKET-555 — Send a block to the agent, and "Ask the agent" under a failed block

- **Ticket:** LOCAL #555 (feature, prong 2 with prong 1: a block handed to the CLI agent in a terminal; after #549 and #554)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/555-send-a-block-to-the-agent.spec.md
- **Source ticket:** The Warp blocks note of 2026-09-25, recommendation 2 (`docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md`), with Chad's answer of 2026-09-26 to its first question: "copy as context" means both, Send to Agent (the block into the agent's prompt, this ticket) and Copy as Markdown (#554). Specced because Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** closed

## Summary
Agents read blocks over Marley's MCP tools, but Chad cannot point an agent at one: Zed's Add
to Agent Thread reaches the Agent Panel, never Claude Code in a center terminal. Warp attaches
a selected block to its agent's input and, after a failed command, offers to attach the output
as context. Marley adds Send to Agent to the block menu and to a selected block
(`ctrl-shift-enter`): it types a reference into the agent's terminal the way a pick's Send
does, `[terminal 42 block 7: cargo build, exit 101; terminal_read terminal=42 block=7] `,
unsent, and focuses that terminal; a block whose output is short goes inline as the Markdown
form instead, so it works where the plugin's bridge is not installed. The target follows #549:
one agent terminal takes it, several open the picker, never the block's own terminal, and a
target waiting on a permission gets nothing. What leaves Marley passes #516's redactor. Under
the newest failed block, while the shell waits at its prompt, an "Ask the agent" chip does the
same in one click.

## Acceptance
Send to Agent on a long failed block puts the reference at the agent's prompt, unsent, and
`terminal_read` with those arguments returns the block; a short block arrives as its Markdown;
the chip appears under a failed block and a click sends it; a token in a sent block reaches the
agent redacted while the terminal keeps it exact; a waiting agent gets nothing and a toast
says why.
