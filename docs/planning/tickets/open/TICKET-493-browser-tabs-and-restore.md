# TICKET-493 — B1b: Browser tabs, restore on relaunch, and the select picker

- **Ticket:** LOCAL #493 (feature, prong 3 B1b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/493-browser-tabs-and-restore.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B1
- **Status:** open

## Summary
Every page in Marley's Chromium becomes a Browser tab: a page the page opens (a new window, a
link to `_blank`) or an agent opens appears beside the one that opened it, and closing a tab
closes its page. Browser tabs come back when Marley relaunches, attached to their pages while
Chromium still runs and reopened at their URLs when it does not. And since headless Chromium
draws no `<select>` popup, Marley draws the list of options itself.

## Acceptance
A link to `_blank` opens a second Browser tab; closing it closes its page; after a relaunch
both tabs return on their pages; clicking a `<select>` shows its options, and choosing one
sets it in the page with the page's change event.
