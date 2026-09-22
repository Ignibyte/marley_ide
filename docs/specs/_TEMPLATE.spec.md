---
spec_id: <kebab-slug>
component: <marley crate or module, e.g. marley_core>
bucket: REIMPLEMENT | INVENT
milestone: M0 | M1 | M2 | M3 | M4 | M5
status: draft
title: <one line>
goal: <one sentence — the outcome, in plain language>
reuses: [<permissive crates pulled in, e.g. gpui, alacritty_terminal, ropey>]
spec_source: <docs/warp_architecture/... the behavior was spec'd from — or "net-new (INVENT)">
clean_room: "behavior-only; no AGPL source read"
browser_testable: no | yes
visual_acceptance: <what it must look/behave like on screen, or N/A>
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
---

## Purpose
What this component is and the problem it solves, in 2–4 sentences.

## Public surface (the contract)
The key public types / functions / traits a consumer calls. Real Rust names. This is the interface other components and tests bind to.

## EARS Requirements
Numbered, one testable clause each (the clause IS the test). Use the EARS forms:
- **Ubiquitous** — `R1. The system shall <response>.`
- **Event** — `R2. WHEN <trigger>, the system shall <response>.`
- **State** — `R3. WHILE <state>, the system shall <response>.`
- **Optional** — `R4. WHERE <feature is present>, the system shall <response>.`
- **Unwanted** — `R5. IF <condition>, THEN the system shall <response>.`
Combine when layered: `R6. WHILE <state>, WHEN <trigger>, the system shall <response>.`
Every clause must be unambiguous and checkable — no "gracefully", no "appropriately".

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | … | planned |

## Visual / Behavioral Acceptance
For UI components: what the AXUIElement + screenshot harness asserts (window/pane present, element tree, near-identical-to-Warp layout). Else `N/A`.

## Test Plan
- **Unit:** each R# → a test name. 100% coverage on this component's touched lines.
- **Integration:** the cross-crate seams exercised.
- **Visual:** the headed-launch assertions (if `browser_testable`/visual).
- **Regression:** what must keep passing.

## Mutation Targets
What `cargo-mutants` must kill (MSI 100% on the testable surface). Note any ACCEPTED-UNTESTABLE lines with the reason (e.g. a GPU/IO path with no unit harness) — a closed decision, not a gap.

## Dependencies
- REUSE (permissive): <crates>
- Marley components: <other specs this depends on>

## Out of scope / deferred
Explicitly what this spec does NOT cover (and which milestone picks it up).

## Clean-room provenance
Spec'd from `<spec_source>` — behavior only, no AGPL/fork source read. REUSE crates are MIT/Apache.
