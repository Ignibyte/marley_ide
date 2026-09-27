# TICKET-524 — Trusted outside clients drive Marley's browser

- **Ticket:** LOCAL #524 (feature, prong 3 with prong 2's MCP server; first of three slices)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/524-trusted-outside-browser-access.spec.md
- **Source ticket:** Chad, 2026-09-25: "marley should also expose its browser to trusted outside so they can drive for the user." Decided the same day as its twin, saved Playwright scripts (#523). Prior art in the Orca survey: report 04 §2.2, §2.13 and §3.2 item 3 (paired devices, per-device tokens, revoke), report 03 §2.11 (agents driving the browser from outside the app).
- **Status:** closed

## Summary
Two doors reach Marley's browser today. Marley's MCP server takes a bearer that changes at each start and sits in a file only the user can read, so only the user's own processes that read Marley's data folder get in, and they all get every tool. Chromium's DevTools port takes no credential at all (CDP has none) and listens on 127.0.0.1, where any local process can reach it; on the dev box that includes processes of more than a dozen other system users. Chad wants trusted outside programs, such as another agent host or a sandboxed agent, to drive the browser for him. This first slice gives them a front door: the user names each client and chooses whether it may only read pages or also act in them; each client gets its own token, in an endpoint file of its own, and reaches only the browser tools its grant allows through Marley's MCP server; a tab a client drives shows the client's name; and the user can cut a client off at once. Loopback only. The second slice closes the side door (Chromium's own endpoint off TCP, behind a relay that takes the same client tokens, for Playwright clients), and the third reaches other machines through an SSH tunnel.

## Acceptance
A client allowed by name lists and calls only the browser tools its grant's list names; a read-only client's write call is refused; a tab a client drives shows the client's name and a Cut Off button; cutting a client off refuses its next request and ends its sessions; tokens change at each start; Marley's own bridge keeps every tool. Since the promotion (2026-09-27): what the server reads before it authenticates is bounded in size and time, a session belongs to the bearer that opened it, and the bridge tells a refused token as a refusal.
