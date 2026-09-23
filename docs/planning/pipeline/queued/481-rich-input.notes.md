# Rich input: a Zed editor for an agent's prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-481-rich-input.md
- **Pipeline spec:** 481-rich-input.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "there is Rich input which im not even sure what that is? DO
  you know?" Answered from Warp's docs: Warp's own editor in place of the agent's prompt box.
- **Recall.** PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001 and
  L-claude-453-a-key-context-test-must-press-a-key-only-that-context-binds-001: the Ctrl-G test
  must fail when the context is gone, so it checks the key reaches the program without an agent.
  The inline assistant (`agent_ui/src/terminal_inline_assistant.rs`) is the template for an
  editor over a terminal.
