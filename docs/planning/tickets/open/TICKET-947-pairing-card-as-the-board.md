# TICKET-947 — The pairing card as the board draws it

- **Ticket:** LOCAL #947 (feature, the Marley look, AgentAccess)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** none yet
- **Source ticket:** split from #938 (its Out and D6); the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/F-tools.md, board AgentAccess)
- **Boards:** AgentAccess
- **Size:** M
- **Status:** open

## Summary

The AgentAccess board draws an outside program's pairing as a card. It is titled "“<name>” asks
to pair with Marley", followed by "From <folder> over SSH. Check the code matches what the command
printed." and the code in six digit boxes. Under that are three grants as a radio list, each with
its line:

- Read: "The read tools only, as before pairing."
- Read and Write: "Also editors, the Browser tab, threads, palette actions, settings proposals."
- Read, Write and Run: "Also typing and running in terminals, harness seats, starting agents."

The card ends with Refuse and Pair, and "No answer in two minutes pairs nothing."

Marley's pairing question (`clients.rs`, `show_pairing`) shows none of this. "Over SSH" needs the
bridge to say how it was reached: a new `Caller` field in `marley_mcp` and a header from
`marley-mcp-bridge`. The radio list needs the picked grant held in the notification until Pair.

## Acceptance

A pairing request shows the card as the board draws it, a grant picked in the list is the power
the client pairs with, and "over SSH" shows only for a bridge reached over SSH; shot in Marley
Night and Marley Day.
