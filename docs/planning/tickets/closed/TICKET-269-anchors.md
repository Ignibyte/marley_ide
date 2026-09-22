# TICKET-269 — B4 anchors: a delta-log anchor layer on marley_editor

- **Forge ticket:** #269 21696db9-3828-40ec-8d10-b7985d8cb8a7 (feature, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** a7bc5872-7fde-4970-8382-ca79c045b374
- **Pipeline doc:** ../../pipeline/active/269-anchors.spec.md
- **Source ticket:** sprint #29 (forge)
- **Status:** closed

## Summary
Positions that survive edits: `Anchor{version, offset, bias}` + a
per-buffer delta log fed by the one shared apply path (so undo/redo rebase
too), `edits_since(version)` (the B3 prerequisite), and a pure rebase fold
(`resolve_anchor`) implementing the public patch arithmetic — no CRDT, no
SumTree, no logical clock (deferred to a real multi-writer milestone).
Pure-core in crates/editor at cov/MSI 100. Prereq for B5 multi-cursor + B6
LSP (an async result lands where it was aimed, not where the text used to
be).

## Acceptance
The bias convention proven at every edge (before/after/covering/insert-at);
fold == stepwise; undo/redo rebase; edits_since exact; always clamped;
mutants traced + killed. Full EARS in the spec.
