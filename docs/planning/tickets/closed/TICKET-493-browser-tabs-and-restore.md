# TICKET-493 — B1b: Browser tabs as pages

- **Ticket:** LOCAL #493 (feature, prong 3 B1b; restore went to #494, the select picker to #495)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/493-browser-tabs-and-restore.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B1
- **Status:** closed

## Summary
Every page in Marley's Chromium becomes a Browser tab of its own: a page the page opens (a new
window, a link to `_blank`) appears beside the one that opened it and takes the focus, a page an
agent opens appears without taking it, Ctrl+T opens a blank one, and closing a tab closes its
page. Each tab shows its own page with its own address bar, history, loading and dialogs, and
the agent tools act on the tab the user focused last, or on the one they name.

## Acceptance
A link to `_blank` opens a second Browser tab next to the first; an agent's new page opens a
tab without the focus; closing a tab closes its page; Ctrl+T opens a blank page with the address
bar focused.
