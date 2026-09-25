# TICKET-492 — B2: The agent sees and drives the browser

- **Ticket:** LOCAL #492 (feature, prong 3 B2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/492-browser-tools-for-agents.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B2, D15 and D17
- **Status:** closed

## Summary
The `browser_*` tools on Marley's MCP server (#491) give an agent what Chad sees in the
Browser tab: the page's URL, title, focus, selection and the very frame on his screen, an
accessibility snapshot that includes cross-site iframes, and the recent console and network.
The same tools act in that tab (navigate, click, type, press, scroll, back) while Chad
watches, with a chip in the tab naming the agent's last action: the Cursor-like half of
Chad's 2026-09-24 goal.

## Acceptance
Through the plugin's bridge, a client reads the frame Chad sees and a snapshot with refs
(an iframe's field included), reads the console and network with secrets redacted, and
navigates, clicks and types in the tab, which shows each action and the Agent chip; other
URL schemes are refused.
