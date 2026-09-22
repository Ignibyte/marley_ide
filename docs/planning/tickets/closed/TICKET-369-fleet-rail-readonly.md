# TICKET-369 — Fleet rail: read-only render of the FleetSnapshot (state chips + question-cards + staleness)

- **Forge ticket:** #369 (279d5673-899d-466a-be94-d6c3417c9251) (feature, M23)
- **Owner:** ca2d1534-e512-4d84-92e8-b2f73b4955c9 (M23 train session)
- **AAR:** ca93ce71-629a-42b1-a6d0-438b16727ebf
- **Pipeline doc:** ../../pipeline/active/369-fleet-rail-readonly.spec.md
- **Source ticket:** sprint "M23 — Fleet Control Plane: Layer 1" — slice ③ of the Layer-1 sequence in
  docs/marley_architecture/orchestration-shell.md §12 (① envelope/reducer #367 → ③ this rail)
- **Status:** closed (shipped 2026-07-20; gate GREEN --diff; see pipeline/completed/369-fleet-rail-readonly.notes.md)

## Summary
The shell grows a READ-ONLY fleet rail rendering `marley_fleet::FleetSnapshot` — insight before
control (the tmux intake's promotion rule). Each seat renders as a card: title + a state chip from
the closed vocabulary (Starting/Working/Idle/Waiting/Error/Done — Error visually ≠ Idle), transport
as a subtle hint, opaque label chips rendered generically (Marley renders strings, never interprets
them — a Forge-specific string in the render path is a defect), a Waiting seat's question as a
render-only card (answering is a Layer-2 receipted verb), staleness dimming from the reducer's
absence-of-heartbeat flag + a last_event relative time, and attention ordering (Waiting/Error seats
surface first, preserved from the snapshot). Zero writes, no dispatch UI, no session verbs. The rail
is fully provable from a fixture/demo feed — the live ② client feed may not exist yet (Layer 0 is
external and unshipped). Depends on #367 (the pure `marley_fleet` crate: `Session` +
`FleetSnapshot`); the rail renders those types and re-derives nothing.

## Acceptance
The fleet rail opens beside the existing shell surfaces without regressing them, renders a demo
FleetSnapshot with all six state chips (Error ≠ Idle), question-cards only for Waiting-with-question
seats (visibly not-yet-actionable), verbatim opaque label chips, staleness dimming + relative time,
snapshot (attention) order preserved, and a non-blank empty state. Full EARS criteria live in the
pipeline spec.
