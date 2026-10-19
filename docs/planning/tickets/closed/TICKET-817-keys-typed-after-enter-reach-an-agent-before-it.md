# TICKET-817 — Keys typed right after Enter in an agent's prompt editor reach the agent before it

- **Ticket:** LOCAL #817 (bug, the prompt editor for agents)
- **Owner:** 6bc788ef-5565-4849-9e73-e341a6348e9f
- **Pipeline doc:** ../../pipeline/completed/817-input-waits-behind-a-pastes-enter.spec.md
- **Source ticket:** split from #816, 2026-10-19.
- **Status:** closed

## Summary
#816 sends the shell's Enter with its paste. An agent's editor (`Target::Agent`) still pastes,
sends its `\r` after `terminal_drive::AFTER_PASTE` (200 ms, which Python's REPL and agent CLIs
need, F-claude-594) and gives the terminal the keys at once, so keys typed in that window, or
queued behind the Return, reach the agent ahead of the Enter and join the prompt just sent. The
other `paste_then` callers (`terminal_type`, review notes, delegation, resume, limits, send
selection) share the window when the person types meanwhile.

## Acceptance
A prompt sent from an agent's editor goes alone, whatever is typed right after it; what is typed
reaches the agent after the Enter.
