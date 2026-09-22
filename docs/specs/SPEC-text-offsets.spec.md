---
spec_id: text-offsets
component: marley_text_offsets
bucket: REIMPLEMENT
milestone: M0
status: draft
title: Type-safe Char/Byte offset newtypes
goal: Give the editor and terminal a shared, mistake-proof vocabulary for text positions so a char index can never be silently used where a byte index is meant.
reuses: [serde, get-size2]
spec_source: "behavior-level contract — type-safe text-position values: a position counted in Unicode chars vs one counted in bytes that the type system keeps from ever being mixed, plus a forward-only streaming byte→char converter over a borrowed string. Observable I/O only; no fork module/type/file names."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_text_offsets` is the lowest-level text-position primitive shared across Marley's editor, terminal, search, completer, and AI layers. It provides two distinct value types — `CharOffset` (a position counted in Unicode `char`s) and `ByteOffset` (a position counted in bytes) — that the type system keeps from ever being mixed, eliminating the classic editor byte-vs-char index bug class. It additionally provides `CharCounter`, an incremental forward-only converter from byte offsets to char offsets over a borrowed `&str`. The crate has no internal dependencies; it is the common vocabulary every other text crate consumes.

Per **seam-contracts §1 (binding)**, this crate is the **sole owner** of `CharOffset` and `ByteOffset` in the entire workspace: there is exactly one of each, both with a **private** wrapped field, and **no other crate may declare its own offset newtype** — downstream crates (`marley_editor`, `marley_completer`, `marley_syntax`, `marley_terminal`) import these. The char↔byte conversion seam splits by role: the stateful, random-access whole-buffer converter is owned by `marley_editor::Buffer` (`char_to_byte`/`byte_to_char`); the streaming, forward-only converter for decode/scan paths that do not hold a `Buffer` is this crate's `CharCounter` (seam-contracts §1.1).

## Public surface (the contract)
All in `crates/marley_text_offsets/src/lib.rs`. The wrapped field of each newtype is **private**; the per-newtype API below is the only way to construct or read an offset. (The macro that emits the identical per-newtype API for both flavors is an internal implementation detail — implementer's choice — not part of the observable contract.)

- `pub struct CharOffset(/* private */ usize);` — offset counted in `char`s. Derives `Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, GetSize`.
- `pub struct ByteOffset(/* private */ usize);` — offset counted in bytes. Same derives.
- Per-newtype API (identical for both flavors):
  - `pub const fn zero() -> Self`
  - `pub const fn as_usize(self) -> usize` — the only read accessor for the private field.
  - `pub fn empty_range(self) -> core::ops::Range<Self>`
  - `pub fn add_signed(self, delta: isize) -> Self` — debug-asserts on overflow; wraps in release.
  - `pub fn range(r: core::ops::Range<usize>) -> core::ops::Range<Self>`
  - `impl From<usize> for Self`
  - `impl Add<Self>`, `impl Add<usize>`, `impl Sub<Self>`, `impl AddAssign<Self>`, `impl AddAssign<usize>`, `impl SubAssign<Self>`
- `pub struct CharCounter<'a>` — incremental forward-only byte→char converter:
  - `pub fn new(s: &'a str) -> Self`
  - `pub fn char_offset(&mut self, byte: ByteOffset) -> CharOffset`

`CharOffset` and `ByteOffset` are **non-interchangeable**: any expression mixing the two flavors in arithmetic, assignment, or comparison must fail to compile (seam-contracts §1).

## EARS Requirements
R1. The system shall expose two newtypes, `CharOffset` and `ByteOffset`, each wrapping a **private** `usize` that is readable only via `as_usize()`; the wrapped value is never exposed as a public tuple field, and construction is exclusively through the documented API (`From<usize>`, `zero()`, `range()`, and the arithmetic operators / `add_signed`).

R2. The system shall make `CharOffset` and `ByteOffset` non-interchangeable, such that any expression mixing the two flavors in arithmetic, assignment, or comparison fails to compile.

R3. WHEN `zero()` is called on a newtype, the system shall return the offset whose wrapped value is `0`.

R4. WHEN `as_usize()` is called on an offset, the system shall return the wrapped `usize` value unchanged.

R5. WHEN an offset is constructed via `From::<usize>::from(n)`, the system shall produce an offset whose `as_usize()` equals `n`.

R6. WHEN `empty_range()` is called on an offset `o`, the system shall return the zero-length range `o..o`.

R7. WHEN `range(start..end)` is called, the system shall return `Self::from(start)..Self::from(end)`.

R8. WHEN `add_signed(delta)` is called and `wrapped_value.checked_add_signed(delta)` is `Some(v)`, the system shall return the offset whose wrapped value is `v`.

R9. IF `add_signed(delta)` would carry the wrapped value outside `0..=usize::MAX`, THEN the system shall panic via `debug_assert!` in debug builds and return the two's-complement wrapped result in release builds.

R10. WHEN two offsets of the same flavor are combined with `+` or one offset is combined with a `usize` via `+`, the system shall return an offset whose wrapped value is the sum of the operands.

R11. WHEN `+=` is applied with another offset of the same flavor (`AddAssign<Self>`) or with a `usize` (`AddAssign<usize>`), the system shall increase the wrapped value in place by that amount.

R12. WHEN one offset is subtracted from another of the same flavor with `-`, the system shall return an offset whose wrapped value is the difference; and WHEN `-=` (`SubAssign<Self>`) is applied, the system shall decrease the wrapped value in place by that amount.

R13. WHEN two offsets of the same flavor are compared, the system shall order and equate them strictly by their wrapped `usize` values (`Ord`, `PartialEq`).

R14. WHEN an offset is serialized and then deserialized through `serde`, the system shall produce an offset equal to the original.

R15. WHERE the `get-size2` derive is present, the system shall report `get_heap_size() == 0` for any offset, reflecting that it holds no heap allocation.

R16. WHEN `CharCounter::new(s)` is constructed over `&str s`, the system shall initialize the converter at byte position `0` and char position `0`.

R17. WHEN `char_offset(b)` is called with a `ByteOffset` `b` that lies on a UTF-8 char boundary at or after the converter's current byte position, the system shall return the `CharOffset` equal to the number of `char`s in `s[..b.as_usize()]`.

R18. WHEN `char_offset(0)` is called, the system shall return `CharOffset::zero()`; and WHEN `char_offset(b)` is called with `b.as_usize() == s.len()`, the system shall return the total `char` count of `s`.

R19. WHILE `char_offset` is invoked with monotonically non-decreasing byte offsets, the system shall resume scanning from its last recorded position rather than from the start of `s`, advancing only over the newly spanned bytes.

R20. WHEN `s` contains multi-byte UTF-8 scalar values, the system shall return a `CharOffset` strictly less than `b.as_usize()` for any `b` past such a scalar value, reflecting char-vs-byte divergence.

R21. IF `char_offset(b)` is called with a `ByteOffset` `b` whose `as_usize()` is strictly less than the converter's current byte position — a non-monotonic, backward call, which is the misuse this crate exists to prevent — THEN the system shall `debug_assert!` and panic in debug builds, and in release builds shall not scan backward but return the `CharOffset` recorded at the converter's current position (a defined, non-UB result).

R22. IF `char_offset(b)` is called with a `ByteOffset` `b` that does not lie on a UTF-8 char boundary of `s` (`!s.is_char_boundary(b.as_usize())`), THEN the system shall `debug_assert!` and panic in debug builds, and in release builds shall return the `CharOffset` for the greatest char boundary at or before `b` — never panicking and never splitting a scalar value (a defined, non-UB result).

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | Two newtypes exist; field is private; read only via `as_usize()`, construct only via documented API (`From<usize>`/`zero()`/`range()`/arithmetic) — no public tuple field (R1) | planned |
| 2 | Mixing `CharOffset` and `ByteOffset` fails to compile (R2) | planned |
| 3 | `zero()` returns wrapped `0` for both flavors (R3) | planned |
| 4 | `as_usize()` returns the wrapped value unchanged (R4) | planned |
| 5 | `From<usize>` round-trips through `as_usize()` (R5) | planned |
| 6 | `empty_range()` yields `o..o` (R6) | planned |
| 7 | `range(a..b)` yields `Self::from(a)..Self::from(b)` (R7) | planned |
| 8 | `add_signed` returns checked result when in range (R8) | planned |
| 9 | `add_signed` overflow debug-panics; release wraps (R9) | planned |
| 10 | `+` over `Self` and `usize` sums wrapped values (R10) | planned |
| 11 | `+=` over `Self` and `usize` increments in place (R11) | planned |
| 12 | `-` and `-=` subtract wrapped values (R12) | planned |
| 13 | `Ord`/`PartialEq` order and equate by wrapped value (R13) | planned |
| 14 | serde serialize→deserialize round-trips to an equal offset (R14) | planned |
| 15 | `get_heap_size()` is `0` (R15) | planned |
| 16 | `CharCounter::new` starts at byte 0 / char 0 (R16) | planned |
| 17 | `char_offset(b)` equals char count of `s[..b]` on boundary (R17) | planned |
| 18 | `char_offset(0) == zero`; `char_offset(len) == total chars` (R18) | planned |
| 19 | Monotonic calls resume from last position, no full rescan (R19) | planned |
| 20 | Multi-byte string yields `CharOffset < ByteOffset` past a wide scalar (R20) | planned |
| 21 | Backward (non-monotonic) `char_offset` debug-panics; release returns current `CharOffset`, no backward scan (R21) | planned |
| 22 | Non-char-boundary `char_offset` debug-panics; release rounds down to the nearest boundary at/before `b`, never splits a scalar (R22) | planned |

## Visual / Behavioral Acceptance
N/A — pure non-UI type-safety primitive; no window, pane, or AXUIElement surface. `browser_testable: no`; no `visual_acceptance` clause, so quality-bar gate 15 does not apply to this spec.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement:
  - `r1_private_field_read_only_via_as_usize`, `r3_zero_is_zero` (both flavors), `r4_as_usize_identity`, `r5_from_usize_roundtrip`, `r6_empty_range_is_o_to_o`, `r7_range_constructs_self_bounds`, `r8_add_signed_in_range`, `r9_add_signed_release_wraps` (+ `#[should_panic]` `r9_add_signed_debug_overflow_panics`), `r10_add_self_and_usize`, `r11_add_assign_self_and_usize`, `r12_sub_and_sub_assign`, `r13_ord_and_eq_by_value`, `r14_serde_roundtrip`, `r15_get_heap_size_zero`, `r16_char_counter_new_at_origin`, `r17_char_offset_equals_char_count`, `r18_char_offset_zero_and_end`, `r19_monotonic_resumes_without_rescan`, `r20_multibyte_char_lt_byte`.
  - R1 is additionally backed by the R2 compile-fail snapshot (there is no public tuple-field access path to assert at runtime — its absence is enforced by the private field + `trybuild`).
  - R2 is a `trybuild` compile-fail case (`tests/ui/mixed_offsets_fail.rs` + `.stderr`) asserting the type error.
  - R19 is verified by a counter-instrumented `CharCounter` test variant (or a property test) asserting bytes scanned across two monotonic calls equals the spanned span, never the full length.
  - R21 (non-monotonic backward call): `#[should_panic]` `r21_backward_call_debug_panics` (debug build asserts the `debug_assert!` fires) **plus** `r21_backward_call_release_returns_current` (a `cfg(not(debug_assertions))` test asserting the release result equals the `CharOffset` at the current position and that no backward scan occurred, via the instrumented byte-counter).
  - R22 (non-char-boundary byte): `#[should_panic]` `r22_non_boundary_debug_panics` **plus** `r22_non_boundary_release_rounds_down` (a `cfg(not(debug_assertions))` test over a multi-byte string asserting the result equals the char count at the greatest boundary ≤ `b` and that `s[..result_boundary]` never splits a scalar).
  - 100% line coverage on every touched line of `lib.rs`.
- **Integration:** none internal (zero internal deps). A downstream seam test in the first consumer (M1 `marley_editor::Buffer`) confirms `CharOffset`/`ByteOffset` flow through `range()` into buffer seeks and that the editor imports these types rather than re-declaring them (seam-contracts §1); out of scope here, named for traceability.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full suite plus the `trybuild` snapshot must stay green; the per-newtype API must remain identical for both flavors (a shared parameterized test runs over each).

## Mutation Targets
`cargo mutants` must kill every viable mutant on the per-newtype API and `CharCounter`:
- arithmetic operator swaps in `Add`/`Sub`/`add_signed` (`+`↔`-`, `+`↔`*`) — killed by R8/R10/R11/R12.
- boundary mutants in `CharCounter::char_offset` (off-by-one in char count, `<`↔`<=` in the resume guard, dropping the position update) — killed by R17/R18/R19/R20.
- the R21 backward-call guard (deleting the `debug_assert!`, flipping the `<` comparison that detects a backward `b`, or replacing the release "return current position" with a re-scan) — killed by `r21_backward_call_debug_panics` + `r21_backward_call_release_returns_current`.
- the R22 boundary guard (deleting the `debug_assert!`/`is_char_boundary` check, or replacing the release round-down with an unchecked slice that would split a scalar) — killed by `r22_non_boundary_debug_panics` + `r22_non_boundary_release_rounds_down`.
- `zero()` returning a non-zero constant — killed by R3.
- `empty_range()`/`range()` swapping start/end bounds — killed by R6/R7.
- comparison-operator mutants in `Ord`/`PartialEq` — killed by R13.
- MSI target: **100%** on the testable surface. No ACCEPTED-UNTESTABLE lines anticipated; the `debug_assert!` arms (R9, R21, R22) are exercised by `#[should_panic]` tests in debug and by the wrap/round-down tests in release, so they are not excluded.

## Dependencies
- REUSE (permissive, MIT/Apache): `serde` (derive + round-trip), `get-size2` (heap-size accounting derive — the maintained successor to `get-size`, whose latest release `0.1.4` is unmaintained; `get-size2` keeps the `GetSize` trait + derive surface). Both are MIT/Apache and pass `cargo deny`'s allowlist. Signed arithmetic uses std `usize::checked_add_signed`/`wrapping_add_signed` (stable), so no `num-traits` dependency is needed.
- Marley components: none (leaf crate; 0 internal deps). This crate is the **upstream** offset vocabulary every other text crate imports (seam-contracts §1).

## Out of scope / deferred
- The `SumTree`/buffer dimension integration that seeks these offsets inside the editor's rope — deferred to the M1 editor/buffer spec.
- A backward (char→byte) or random-access converter — `CharCounter` is forward/incremental only; whole-buffer random-access char↔byte conversion is owned by `marley_editor::Buffer` (seam-contracts §1.1), a separate spec.
- The dual-offset buffer summary carried by the editor — owned by the M1 editor spec, not here.

## Clean-room provenance
Behavior-derived from a fork-reference doc; **IP-counsel sign-off pending** (open item in `clean-build-plan.md`). The `spec_source` above is a behavior-level statement (observable I/O only — no private module/type/static names, no fork file paths). The public surface uses Marley-original identifiers and seam-contracts-blessed names only (`CharOffset`/`ByteOffset` with private fields, `CharCounter`); the per-newtype-API macro is an internal implementation detail and not part of the contract. REUSE crates (`serde`, `get-size2`) are MIT/Apache. The package is named `marley_text_offsets`; no AGPL/fork source was read or transcribed for this spec.
