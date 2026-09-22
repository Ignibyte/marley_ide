# TICKET-418 — The simple rail, designed in React (+ parity re-baseline)

- **Ticket:** LOCAL #418 (feature, M31)
- **Tags:** rail, redesign, react-poc, parity, simple-rail
- **Created:** 2026-08-12
- **Status:** closed (2026-08-12 — shipped: the POC rail redesigned + verified, MARLEY-PARITY re-baselined, captures 34–38 + geometry sheet landed; ports queued as #419–#421)

## Summary

The design ticket for the M31 rail redo (Chad's call, 2026-08-12): replace the ancestry-lit,
four-level rail with a simplistic ChatGPT-style rail, designed and visually settled in
marley-web (`LeftRail.tsx`) BEFORE any Rust. Target grammar (beautifului.dev Sidebar Nav +
Task Rows, captured): quiet flat rows, exactly ONE selected row with the rounded fill,
left-edge dot indicators for "open/mounted elsewhere", small-caps section headers, per-file
editor rows, and an add-project affordance. Because the current POC deliberately encodes the
ancestry model (LeftRail.tsx:166-169) and MARLEY-PARITY.md freezes the rail as Zone A
("Marley is right"), this ticket also re-baselines the parity contract for the rail: the POC
becomes the design source; new reference captures replace the old rail shots. marley-web +
docs only — no Rust.

## Acceptance

Headline: the new rail exists in the POC at localhost:5173 with single-selection highlight,
dot indicators, per-file rows, and an add-project affordance; MARLEY-PARITY.md's rail rows
point at the new captures and name Rust port targets for #419–#421. Full EARS in the queued
spec (`docs/planning/pipeline/queued/418-simple-rail-react-design.spec.md`).
