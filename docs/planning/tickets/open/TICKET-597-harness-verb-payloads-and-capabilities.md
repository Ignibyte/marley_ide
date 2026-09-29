# TICKET-597 — The harness's verb payloads and capabilities in the contract crate

- **Ticket:** LOCAL #597 (chore, prong 2, the fleet contract before C1 and C4)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (no queued spec)
- **Source ticket:** rustal-harness `docs/planning/MARLEY_REQUESTS.md`, MREQ-003 and MREQ-004 (both
  2026-09-26), filed after #533's plan; #533's risk rule gives a request of another shape its own
  ticket.
- **Status:** open

## Summary
The harness asks Marley's contract crate for two more things. MREQ-003: each verb's accepted
payload in `marley_fleet`, starting with `SurfaceAck { surfaced }`, which lives in `marley_mcp`
today, then a read's lines and range and an answer's choice, so a server and an adapter check the
same types (#533 added the send's and the open's). MREQ-004: a typed `capabilities` map on
`Session` (`mode`, `model`, `effort`, as `fleet-control-plane.md` designs it) and an optional
`requires` on `SendRequest`, so an adapter reads and checks them without the harness's
`capability.NAME` labels.

## Acceptance
`marley_fleet` exports `SurfaceAck` and the read's and answer's values with the fields `rh mcp`
returns; `Session.capabilities` and `SendRequest.requires` are optional and absent from the JSON
when unset; Marley's server keeps answering `session_surface_to_human` with the moved type.
