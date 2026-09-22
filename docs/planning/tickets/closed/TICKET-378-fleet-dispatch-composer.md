# TICKET-378 — Fleet dispatch composer — session.send as mailbox data + delivery-state chips

- **Forge ticket:** #378 (b40bc0c2-76dd-448f-867b-534cc983408b) (feature, M24)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/378-fleet-dispatch-composer.spec.md
- **Source ticket:** sprint #35 "M24 — Fleet Layer 2"; fleet-control-plane.md §7 item 5 (the
  dispatch composer) / orchestration-shell.md §4-§5 (the dispatch plane, `session.send`)
- **Status:** closed

## Summary
Layer-2 gated-writes ②: compose a brief to ONE seat as mailbox DATA — never keystrokes (the
two-planes rule) — sent as a proposed `session.send` MCP `tools/call` through `marley_forge_client`
(contract-first fixtures; live Forge-side support is an external follow-up), receipt-tracked
through the shipped-but-unwired `marley_fleet::dispatch` delivery-state machine
(`deposited → claimed → started`, advanced by `seat_events` echoes via the shipped monotone-join
`DeliveryState::observe`), and rendered as a delivery-state chip on the seat's fleet-rail card. The
v1 composer reuses the inline-draft input idiom (#177 `renaming_tab` / #204 `naming_workflow`)
targeted from the rail — no new modal machinery. Kills the evidence-night failure class verbatim
(fleet-control-plane §2): dispatch-by-keystroke with no receipt — seats sitting idle on unsent
briefs.

## Acceptance
A seat-targeted inline draft; confirming it dispatches exactly one `session.send` carrying
`SendRequest {id, text}` as data (never a PTY write); an Accepted receipt renders the seat's chip
at `deposited`; `seat_events` echoes advance it `deposited → claimed → started` through the
existing `marley_fleet::dispatch` machine (duplicates/skips safe, never regressing); any failure
(transport, RPC error, `isError`, or a `Refused{reason}` receipt) surfaces its reason and
PRESERVES the draft; the wire shape is fixture-pinned as a well-formed MCP `tools/call` — the
fixtures being the proposed v1 contract. Full EARS criteria (REQ-001..007) live in the pipeline
spec.
