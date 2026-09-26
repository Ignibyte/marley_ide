# TICKET-505 — Pick, fix, check: the same element after the agent's fix

- **Ticket:** LOCAL #505 (feature, prong 3, after B3a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/505-pick-fix-check.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 2 of the list after the browser waves)
- **Status:** open

## Summary
A pick sends an element to the agent, but nothing shows whether the agent's fix worked. Marley re-finds the picked element by its locators after a change, crops it again, and shows before and after side by side in the pick tray; an agent gets the same through a tool, so it can check its own fix.

## Acceptance
After the page changes, a pick's Check re-finds the element and shows the before and after crops in the tray; a tool returns the new crop and what changed.
