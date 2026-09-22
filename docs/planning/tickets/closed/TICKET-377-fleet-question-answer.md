# TICKET-377 — Fleet structured interrupts: answer the Waiting seat's question (receipted verb)

- **Forge ticket:** #377 (d81f8c67-7b4f-4c9d-8f83-b57c73796727) (feature, M24)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/377-fleet-question-answer.spec.md
- **Source ticket:** sprint #35 "M24 — Fleet Layer 2"; makes the #369 render-only question-card
  actionable (Layer-2 gated writes ①, orchestration-shell.md §12)
- **Status:** closed

## Summary
`fleet_rail::question_card` (#369) renders a Waiting seat's structured question read-only — prompt +
option chips, Waiting-gated, with an honest "answered in Layer 2" hint. This ticket is that Layer 2:
picking an option dispatches exactly ONE receipted answer via `marley_forge_client`, contract-first —
a proposed MCP `tools/call` wire (arguments `{session_id, choice}`) frozen by fixture-server tests in
the #368/#372 livewire pattern (live ucsosv2 support for the tool = external follow-up). A local
pending/answered chip shows until the seat's own event stream flips its state — truth stays with the
reducer (no optimistic state forgery: the send path never patches a `Session`/`FleetSnapshot`). A
failed or refused send surfaces the reason and re-arms the card; while an answer is in flight,
further picks are no-ops. The answer is DATA on the control plane, never keystrokes — the in-shell
slice of the mission-control AskUserQuestion loop (flagship loop #1). Sweep headline:
`marley_fleet::verbs` ships NO answer verb yet (Send/Read/Open/Surface + `Receipt<T>` only), while
the write lane it needs is already whole in `marley_forge_client` (`tool_call_request` +
fail-closed `parse_write_ack`).

## Acceptance
Picking an option on a Waiting seat's card dispatches exactly one well-formed answer `tools/call`
naming the seat + the picked option string (fixture-recorded — the frozen proposed contract); the
pending gate makes in-flight re-picks no-ops; a success receipt shows a local pending→answered chip
with the reducer snapshot untouched until the event stream flips it (the chip then clears); any
failure class (HTTP, JSON-RPC error, `isError` refusal) surfaces and re-arms; non-Waiting seats (and
Waiting without a question) keep offering no affordance. Pure seams (answer state machine, pick
gate, chip + clear decisions, request builder, receipt parse) at cov/MSI 100; the app.rs click/send
glue masked. Full EARS criteria live in the pipeline spec
(../../pipeline/queued/377-fleet-question-answer.spec.md).
