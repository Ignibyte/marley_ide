# TICKET-027 — delete marley_spike (the M0 throwaway)

- **Forge ticket:** #27 `ef3f9837-2d8b-4009-b930-96839267d156` (chore, M1.C — The Wired Cockpit, seq-6)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `69be8de9-21c6-4cb3-a9a3-7e2df3fa7d8d`
- **Pipeline doc:** ../../pipeline/active/delete-marley-spike.spec.md
- **Source ticket:** forge sprint #3 `b295976b-4f32-4edc-be3c-aecd80b02785` (M1.C — The Wired Cockpit)
- **Status:** closed

## Summary
`marley_spike` was the M0 viability spike — explicitly throwaway ("deleted once the M0 gate is
green"). It still ships in the workspace, so every FULL gate run covers + mutates dead code. Delete
the crate and sweep every reference: the workspace glob auto-drops it; clean the `gates.sh`
coverage-exclude regex; DELETE `SPEC-foundation-spike.spec.md` (its `component: marley_spike` +
visual_acceptance is a gate-15 binding) + the as-built doc + the crate-map node/row; reword the
lineage/precedent prose. Its write-first-exemplar role is superseded by `marley_app`.

## Acceptance
No `crates/marley_spike` + no `marley_spike` reference in any manifest/gates.sh/surviving source;
`cargo nextest run --workspace` green (nothing depended on it); gate-15 has no dangling component;
FULL gate GREEN [--diff] with a receipt (the deletion removes `crates/**/*.rs`). Full EARS in the
pipeline spec. This is the M1.C sprint's final ticket (#22–#26 shipped the features).
