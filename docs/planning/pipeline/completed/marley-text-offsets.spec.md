---
pipeline_id: d91ec9ea-f771-4ecd-babd-a635f5a00e89
ticket: forge#2 (b17cd9a8-1ca3-4f9d-95d3-39ae8fdb26b7) · local docs/planning/tickets/open/TICKET-001-text-offsets.md
aar_id: 49f16b7b-26ec-4d47-910a-6e8f16e0cf59
status: Phase 5 — Complete PASS
title: marley_text_offsets — type-safe Char/Byte offsets + streaming converter
type: feature
milestone: M0
references:
  - ../../../specs/SPEC-text-offsets.spec.md
  - ../../../specs/standards/seam-contracts.md
  - ../../../specs/standards/quality-bar.spec.md
---

## Title

TICKET-001 — `marley_text_offsets`. The lowest-level text-position primitive: two
non-interchangeable newtypes `CharOffset`/`ByteOffset` (private `usize`, the
type system forbids mixing) + `CharCounter`, a forward-only streaming byte→char
converter. The shared offset vocabulary every downstream text crate imports
(seam-contracts §1 — this crate is its **sole owner**). The **first real Marley
crate**, and the one that first proves the FULL gate green end-to-end.

**The contract is authoritative in [`docs/specs/SPEC-text-offsets.spec.md`](../../../specs/SPEC-text-offsets.spec.md)
(EARS R1–R22, public surface, test plan, mutation targets).** This pipeline doc
adopts it verbatim — it does not re-derive or override it. Design expands the
implementation approach; it may not change the contract.

## Scope

### In
- `pub struct CharOffset(/* private */ usize)` and `ByteOffset(...)`, derives
  `Clone,Copy,Debug,Default,PartialEq,Eq,PartialOrd,Ord,Hash,Serialize,Deserialize,GetSize`.
- Per-newtype API (identical for both, emitted by an internal macro — implementer's
  choice, not part of the contract): `zero()`, `as_usize()`, `empty_range()`,
  `add_signed()`, `range()`, `From<usize>`, `Add<Self>/Add<usize>`, `Sub<Self>`,
  `AddAssign<Self>/AddAssign<usize>`, `SubAssign<Self>`.
- `CharCounter<'a>`: `new(&'a str)`, `char_offset(ByteOffset) -> CharOffset` —
  forward-only, monotonic-resume (R19), with defined release behavior for the
  backward (R21) and non-boundary (R22) misuse cases.
- Deps (REUSE, MIT/Apache): `serde` (derive), `get-size2` (GetSize derive),
  `num-traits` (checked/signed arithmetic helpers).

### Out (per spec "Out of scope / deferred")
- The whole-buffer random-access char↔byte converter (owned by
  `marley_editor::Buffer`, seam-contracts §1.1) — a later M1 spec.
- The `SumTree`/buffer dimension integration; the dual-offset buffer summary.
- A backward/char→byte converter — `CharCounter` is forward-only.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — The spec is the contract.** R1–R22 of SPEC-text-offsets.spec.md are the
  acceptance criteria and the test list, 1:1. No deviation without amending that
  spec.
- **D2 — Private fields, sole ownership.** The wrapped `usize` is private; the
  only read path is `as_usize()`. No other crate declares an offset newtype
  (seam-contracts §1). R2 (mixing flavors fails to compile) is enforced by the
  private field + a `trybuild` compile-fail snapshot.
- **D3 — Coverage 100% on touched, MSI 100, NO accepted-untestable lines.** The
  `debug_assert!` arms (R9 overflow, R21 backward, R22 non-boundary) are covered
  by `#[should_panic]` tests in debug AND `cfg(not(debug_assertions))` tests
  asserting the defined release result — so every line is testable (AD-claude-
  coverage-model-001). Mutation kills the exact set the spec's Mutation Targets
  enumerate.
- **D4 — This ticket first proves the FULL gate.** Unlike 000 (no `.rs`), 001
  adds `crates/marley_text_offsets/src/lib.rs` (+ tests), so `enforce-commit-gate`
  requires a FULL/`--diff` receipt → `scripts/gates.sh` must print
  `GATE GREEN [full]` (coverage 100 + mutation MSI 100 + the static set), which
  writes the receipt the commit needs.

## Acceptance Criteria (EARS)
**All of R1–R22** in [SPEC-text-offsets.spec.md](../../../specs/SPEC-text-offsets.spec.md)
§"EARS Requirements" / §"Acceptance Criteria", verified by the spec's named tests
(one `#[test]` per clause: `r1_…`–`r22_…`, the R2 `trybuild` case, the R19
no-rescan counter test, the R21/R22 debug+release pairs). Plus the quality bar:

| # | Bar | Verify |
|---|---|---|
| AC-A | All R1–R22 tests pass | `cargo nextest run -p marley_text_offsets` |
| AC-B | R2 mixing fails to compile | `trybuild` `tests/ui/mixed_offsets_fail.rs` + `.stderr` |
| AC-C | 100% line coverage on touched | gate:4 `cargo llvm-cov nextest --fail-under-lines 100` |
| AC-D | Mutation MSI 100 (the spec's enumerated mutants killed) | gate:5 `cargo mutants` |
| AC-E | `#![deny(missing_docs)]` + rustdoc clean | gate:14 |
| AC-F | deny/audit/machete clean with serde+get-size2+num-traits | gate:8/7/9 |
| AC-G | FULL `scripts/gates.sh` → `GATE GREEN [full]` + receipt | gate run at /commit |

## Phase Plan
- **P2 Design** — the macro that emits the per-newtype API (internal); the
  `add_signed` checked/wrapping split (R8/R9); the `CharCounter` resume invariant
  + the R21/R22 release-defined fallbacks; the instrumented byte-counter for the
  R19 no-rescan test; the exact `Cargo.toml` deps; the test manifest (one row per R).
- **P3 Implement** — write `lib.rs` (now phase-gated) + `Cargo.toml` deps.
- **P3.5 Inspect** — critics: contract-fidelity vs the spec, the unsafe-free
  arithmetic correctness (overflow/wrap), the converter misuse arms, clean-room.
- **P4 Validate** — write the R1–R22 tests + trybuild + release-cfg tests; RUN
  them; FULL `scripts/gates.sh` green (coverage 100 + MSI 100).
- **P5 Complete** — AAR capture; archive; close forge #2.
- **/commit** — FULL gate writes the receipt; commit on `ticket-001-text-offsets`.
