# TICKET-434 — Runnables: gutter ▶ spawns a command Block

- **Ticket:** LOCAL #434 (feature, M33)
- **Tags:** fusion, runnables, syntax, terminal-blocks, gutter
- **Created:** 2026-08-15
- **Provenance:** Phase C's first named thread (roadmap: "tree-sitter
  runnables → a gutter run-button → spawn as a first-class command Block");
  shelf: ../../design-notes/m33-tail-and-wedge-shelf.md
- **Pipeline doc:** ../../pipeline/completed/434-runnables-gutter-run-block.spec.md
- **Status:** closed (2026-08-15 — Phase C opened; GATE GREEN [diff] 15/15)

## Summary
The wedge opens: a pure `marley_syntax` runnables API (the 6th node API —
Rust-only, caller-gated like #304/#340: `#[test]` fns and `fn main`) marks
runnable lines; the editor gutter shows a ▶ affordance on those lines; a
click SPAWNS the mapped command (`cargo test <path> <name>` / `cargo run`)
as a first-class command Block in the workspace terminal — status pill,
framed output, the whole shipped block model. Zed has no block model; this
is where terminal-first starts beating it.

## Acceptance
Headline: open a Rust file with a `#[test]`; a ▶ renders in its gutter;
click → a Block runs `cargo test` scoped to that test in the workspace
terminal with a live status pill. Full EARS in the queued spec.
