# TICKET-516 — Secrets hidden from what agents read

- **Ticket:** LOCAL #516 (feature, prong 2 with the MCP server)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/516-secret-redaction-for-agents.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Warp once-over's item 2 (Secret Redaction): "love it need it" (docs/planning/design-notes/warp-once-over-2026-09-25.md)
- **Status:** closed

## Summary
Marley's MCP tools hand agents a terminal's commands and output as they are, so an `env` dump,
a `cat .env`, an `export TOKEN=...` or a key file printed in a terminal goes to whatever model
the agent runs on. The browser's network and console tools already hide secret-looking URL
values (plan D15); the terminal tools hide nothing. Marley redacts what agents read from
terminals and the browser's console: known key shapes (cloud, GitHub, Slack, Stripe, model APIs,
private keys, JWTs, bearer headers, URL passwords) and secret-named assignments become
`[redacted: <kind>]`, with a count in the answer so the agent knows. It is on by default, a toggle
on the Marley settings page, and the user can add patterns of their own.

## Acceptance
An agent reading a terminal block that printed secrets gets them redacted, with the count; with
the setting off it gets them as printed; a user pattern redacts too.
