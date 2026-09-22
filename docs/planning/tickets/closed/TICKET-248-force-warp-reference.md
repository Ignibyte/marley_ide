# TICKET-248 — Force Warp-reference into the pipeline (§20 discipline)

- **Forge ticket:** #248 (d369915f-4728-4836-9f18-730bfc47dfb8) (chore, M15 sprint #28)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** e2221e1e-b048-40cc-a84c-7540722ef1c4
- **Pipeline doc:** ../../pipeline/completed/force-warp-reference.spec.md
- **Source:** the M15 "Editable Editor" train; the FIRST ticket (the rest inherit it). chad 2026-07-11 — force
  Warp-reference in the pipeline.
- **Status:** closed

## Summary
Make "how does Warp do this?" a forced, enforced per-ticket step. Add a required `## Warp Reference (§20)` spec
section (plan-filled, design-confirmed — cite `docs/warp_architecture/…`, or an observed capture in a new durable
`docs/warp_architecture/observed/`, or `N/A — Marley-specific + why`); require design to state how it matches
(the §20 clean-room wall); enforce with a new `enforce-warp-reference.sh` commit hook (mirroring
`enforce-changelog.sh`) + a CONSTITUTION §20 bullet. Gate-is-test (docs + hook, no `.rs`). Fixes the gap that §20
mandates clean-room-from-Warp but nothing forces a per-ticket Warp reference (and the editor isn't specced).

## Acceptance
The template carries the section; the hook blocks a pipeline-spec commit lacking a non-empty `## Warp Reference`
and passes when present; the hook is registered; the plan/design skills + CONSTITUTION §20 require it;
`docs/warp_architecture/observed/` exists. Verified via the hook's exit-code smokes + file/doc checks; static
gate green. Full EARS in the pipeline spec.
