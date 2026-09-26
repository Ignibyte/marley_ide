# TICKET-505 — Pick, fix, check: the same element after the agent's fix

- **Ticket:** LOCAL #505 (feature, prong 3, after #518)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/505-pick-fix-check.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 2 of the list after the browser waves), with the Orca survey's rules for #505 folded in (`docs/orca_architecture/README.md`, "What it changes in the queued sprint"; report 03 §2.7 and item 2)
- **Status:** open

## Summary
A pick sends an element to the agent, but nothing shows whether the agent's fix worked; Orca's
whole check is "click it again". A pick's Check finds the same element again after the page
changed, trying its test id, id, role and name, text, then CSS path, with generated ids and hashed
classes kept out of every locator, the match nearest the old box when several match, and "not
found" when none does, instead of cropping something else. It crops the element again as the
first crop was made (inspect highlight off, the same margin and scale, the element in view) and
shows the before and after crops side by side, opened from the pick's tray row, with what changed
as text: `padding: 8px → 16px`, the box moved or resized, the text. Agents get the same through
`browser_check_pick`, so an agent can check its own fix. It compares against #518's bundle.

## Acceptance
After the page changes, a pick's Check re-finds the element and shows the before and after crops
with what changed; when nothing matches it says so and crops nothing; `browser_check_pick`
returns the new crop and what changed.
