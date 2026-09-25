# TICKET-496 — B3a: Pick an element in the Browser tab and send it to the agent

- **Ticket:** LOCAL #496 (feature, prong 3 B3a; prong 3 wave 2)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/496-element-picker.spec.md
- **Source ticket:** docs/marley/three-prong-plan.md, prong 3 B3a; docs/marley/browser-handoff.md
- **Status:** closed

## Summary
Pillar A, the element picker. Chad turns on pick mode in a Browser tab, sees Chromium's own
inspect highlight follow the pointer, and clicks the element he means. Marley captures a durable
bundle for it at once (the nearest interactive element, ranked locators, the accessibility role
and name, its listeners with their script locations, what blocks a click on it, its box, and a
crop of the frame) and stages it in a tray in the tab. Chad captions it and sends it: Marley
types a reference into the terminal he used last, and the agent reads the whole bundle through
Marley's MCP server.

## Acceptance
Pick mode highlights the element under the pointer; a click stages its bundle in the tray; a
captioned pick sent to the agent lands in the last-used terminal as a reference; `browser_pick`
answers with the bundle.
