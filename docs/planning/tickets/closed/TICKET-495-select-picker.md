# TICKET-495 — B1d: Marley draws the page's `<select>` lists

- **Ticket:** LOCAL #495 (feature, prong 3 B1d; split from #493)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/495-select-picker.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B1
- **Status:** closed

## Summary
Headless Chromium draws no `<select>` popup, so a press on a select in the Browser tab shows
nothing. Marley finds the select under the press, shows its options in a list of its own under
it, and sets the chosen option in the page in an isolated world, with the page's `input` and
`change` events.

## Acceptance
Pressing a `<select>` shows its options with the current one marked; choosing one sets the
page's value and fires its change event; Escape or a press elsewhere closes the list.
