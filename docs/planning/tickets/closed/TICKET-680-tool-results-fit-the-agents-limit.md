# TICKET-680 — Tool results that fit the agent's limit

- **Ticket:** LOCAL #680 (feature, prong 2 C: Marley's MCP server)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** ../../pipeline/completed/680-tool-results-fit.spec.md
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 2;
  `docs/t3code_architecture/06-orchestration-mcp-and-automations.md`, §3 items 1 and 2
- **Status:** closed

## Summary
`terminal_read` answers up to 2,000 lines and 256 KiB of a block's output, and the answer carries
the output twice (the text block and `structuredContent`), so a long build log is far over Claude
Code's MCP output limit (a warning at 10,000 tokens, a cut at 25,000): Claude Code moves the result
to a file, and the agent has no way to ask for the part it needs. This ticket pages block output
newest first in pages that fit, gives Marley's server a short `instructions` text in its
`initialize` result so agents know which tool to reach for, and gives refusals a code and next
steps instead of a sentence alone. It is the first item of phase 1 (the Marley agent), whose
tools will answer through the same server.

## Acceptance
A long block reads as pages of at most 12,000 bytes of output, newest first, each naming the
`before` value of the page ahead of it, with nothing repeated or skipped; `initialize` carries
instructions under 2 KB; a wrong terminal, block or page is refused with a code and the next step.
