# TICKET-484 — Autosuggestions from history, accepted with →

- **Ticket:** LOCAL #484 (feature, prong 1: T3a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/484-autosuggestions.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T3)
- **Status:** closed

## Summary
Chad, 2026-09-23: "warp has an awesome auto complete tell where you type something and from
history and use the right arrow to auto complete. we need that". As you type a command, Warp
shows the rest of the most recent matching command from history as dim ghost text after the
cursor, and → takes it. This is the history ghost text of the plan's T3 (the prompt editor),
as a first slice on the shell's own prompt.

## Acceptance
While the shell waits at its prompt with typed text that starts a command in the history, the
terminal shows the rest of the newest such command dimmed after the cursor; → at the end of the
line types it; typing on filters or clears it. The EARS criteria come with the spec.
