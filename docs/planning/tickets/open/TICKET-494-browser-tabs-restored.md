# TICKET-494 — B1c: Browser tabs return after a relaunch

- **Ticket:** LOCAL #494 (feature, prong 3 B1c; split from #493)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/494-browser-tabs-restored.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B1
- **Status:** open

## Summary
Browser tabs are saved with the workspace, through Zed's `SerializableItem`, in a table of
their own that holds each tab's page and URL, never in the layout's codec. When Marley
relaunches, each tab comes back: on its page while Marley's Chromium still runs it, and at its
saved URL otherwise.

## Acceptance
After a relaunch the Browser tabs return in their places, each on its page when that page
still lives, or reopened at its URL when it does not.
