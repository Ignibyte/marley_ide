# TICKET-498 — B4: Draw annotations on the page, for Chad and for the agent

- **Ticket:** LOCAL #498 (feature, prong 3 B4; prong 3 wave 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/498-annotations.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B4; docs/marley/browser-handoff.md
- **Status:** open

## Summary
Pillar B. Boxes and notes drawn by Marley over the page, never injected into it, stored in page
coordinates and placed at each frame from its scroll offsets, so they stay on what they mark.
Chad draws them in an annotate mode; an agent draws them through Marley's MCP server on an
element by its ref, and reads them all back.

## Acceptance
An annotation Chad draws stays on its page content as the page scrolls; an agent's
`browser_annotate` on a ref draws a box around that element; `browser_annotations` lists them.
