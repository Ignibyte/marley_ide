# TICKET-447 — Rustal's Rust quality gates on the Marley crates

- **Ticket:** LOCAL #447 (chore, quality gates)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/447-rustal-quality-gates.spec.md
- **Source ticket:** Chad's goal of 2026-09-22 ("lets mimic the quality gates on rustal … make sure high quality rust items are here")
- **Status:** closed

## Summary
Chad wants Marley's gate to match the Rust gates of his rustal repo, with mutation testing kept out of the per-change loop. Marley already runs fmt, clippy, tests, coverage, audit, deny, unused dependencies, secrets and the meta-gates; rustal adds a strict lint table (pedantic, nursery and cargo lints plus denies such as `missing_docs`, `unwrap_used` and `expect_used`), manifest formatting, spelling, a check that no test suite is empty, TODO scanning in Rust source, a rustdoc warning check, a semgrep pass, and a sturdier receipt. This ticket brings each of those to the seven `crates/marley_*` crates without touching Zed's own crates, and fixes every hit the lint table finds.

## Acceptance
`script/gates.sh --diff` is green with rustal's lint table on every Marley crate and the new gates in place; each new check fails when its rule is broken. Full EARS in the spec.

## Resolution
Shipped 2026-09-22: all 679 lint hits fixed, gates 17 to 20 added, the runner's modes and receipt
hardened, `script/gates.sh --diff` green at 100% line coverage. TICKET-444 was folded in: the
`let_underscore_must_use` lint this ticket added made its eight discards compile errors.
Zed's dylint lints moved to TICKET-448.
