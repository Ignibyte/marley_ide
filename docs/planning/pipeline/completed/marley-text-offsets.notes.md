# marley_text_offsets — Notes

- **Forge ticket:** #2 `b17cd9a8-1ca3-4f9d-95d3-39ae8fdb26b7` (feature, M0), claimed `dc7df9b5-…`.
- **AAR:** `49f16b7b-26ec-4d47-910a-6e8f16e0cf59`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-001-text-offsets.md`.
- **Pipeline spec:** `marley-text-offsets.spec.md` (pipeline_id `d91ec9ea-…`).
- **Branch:** `ticket-001-text-offsets` (stacked on TICKET-000 `b216a20`).

## Phase 1 — Plan

- **Request:** implement `marley_text_offsets` per the authoritative
  `docs/specs/SPEC-text-offsets.spec.md` (R1–R22). Second ticket of the M0 run;
  the first with real `.rs`.
- **Classification / tier:** work pipeline, **feature** (a new leaf crate), single
  shippable slice. Zero internal deps.
- **Forge recall (§18.3):** no text-offset/newtype-specific lessons (first feature
  crate). `knowledge-context` surfaced 7 nodes — all from TICKET-000:
  - **AD-claude-coverage-model-001** (whole-workspace 100% lines) → this crate
    must reach 100% with no accepted-untestable; the `debug_assert!` arms are made
    testable by `#[should_panic]` (debug) + `cfg(not(debug_assertions))` (release)
    pairs, so D3 anticipates NO excluded lines.
  - **AD-claude-receipt-scope-001** → 001 is the first `.rs` commit, so its FULL
    receipt now also binds the gate-defining files (000's fix).
  - **PR-claude-detection-tracks-runner / receipt-binds-gate-definition**,
    failures BF-tests-ran-runner-detection / BF-commit-receipt-scope — context for
    why the gate is the way it is; not a constraint on this crate.
  - `docs-search` reference analogue: oathstar `WORK-entity-contracts-v1` (a typed
    `Role` vocabulary with construction-boundary typed errors) — same family as a
    type-safe offset vocabulary; informs the "private field + typed construction"
    shape, not the contract (the spec is authoritative).
- **Authoritative contract:** SPEC-text-offsets.spec.md adopted 1:1 (D1). The
  pipeline spec references it; it is not duplicated.

### Carry to Design (the non-trivial bits)
- The internal **macro** that emits the identical per-newtype API for both flavors
  (implementer's choice; NOT part of the contract — the spec says so). Must not
  leak the private field.
- `add_signed` (R8/R9): `wrapped.checked_add_signed(delta)` → `Some(v)` returns
  `v`; overflow → `debug_assert!` panic (debug) / two's-complement wrap (release).
  num-traits/`isize` helpers.
- `CharCounter` resume invariant (R16–R20): track `(byte_pos, char_pos)`; on a
  monotonic `char_offset(b)` scan only `s[byte_pos..b]` (R19 — no full rescan).
  Misuse arms: R21 backward `b < byte_pos` → debug_assert + release returns the
  recorded `char_pos`, no backward scan; R22 `!is_char_boundary(b)` → debug_assert
  + release rounds down to the greatest boundary ≤ b (never splits a scalar).
- R19's no-rescan proof: an instrumented byte-counter (or property test) asserting
  bytes-scanned across two calls equals the spanned span, not the full length.
- `Cargo.toml`: serde (`features=["derive"]`), get-size2 (derive), num-traits.
  All MIT/Apache — clear the permissive deny allowlist (verify with cargo deny).

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design

### The pivotal finding — cargo-mutants does NOT mutate macro bodies
Probe (`scratchpad/mutprobe`, `cargo mutants --list`): a `macro_rules!`-emitted
`add_via_macro` generated **0 mutants**; the sibling non-macro `add_plain`
generated 4 (`replace + with -`, `with *`, `-> 0`, `-> 1`). So **any arithmetic
emitted by a macro is mutation-blind** — the spec's enumerated arithmetic mutants
would never be generated.

Spike (`scratchpad/offprobe`): a generic `Offset<M>(usize, PhantomData<M>)` with
`#[serde(transparent)]` + `#[serde(skip)] PhantomData` compiles and round-trips as
a **bare usize** (`"42"`), and `get-size2` reports heap 0 — but a `pub type
CharOffset = Offset<Char>` forces `Offset`/`Char`/`Byte` to be **public** (a
`pub type` alias to a private type is `E0446`), bloating a seam crate whose
contract is exactly `CharOffset`/`ByteOffset`/`CharCounter`.

### D5 — Structure: macro-emitted structs + non-macro logic
- **`macro_rules! offset_newtype!`** emits the two **spec-exact** `pub struct
  CharOffset(usize)` / `pub struct ByteOffset(usize)` (private field) with the full
  per-newtype API + derives. Clean public surface (nothing but the spec's types),
  DRY, and the spec's intended shape ("the macro that emits the identical
  per-newtype API … implementer's choice").
- **The logic that must be mutation-tested is NON-macro:** `CharCounter` (all of
  it) and `add_signed`'s checked/wrapping core are free/inherent fns the macro
  calls — so cargo-mutants mutates them and MSI covers them.
- **Trivial `a±b` ops** (`Add`/`Sub`/`AddAssign`/`SubAssign`, `zero`, `range`)
  stay macro-emitted: **100% line-covered** (gate:4, the r3/r6/r7/r10–r12 unit
  tests execute every line) + **behaviorally unit-tested**, but they generate no
  mutants (cargo-mutants limitation). Disclosed, not hidden: "kill every viable
  mutant" is satisfied vacuously on the macro surface (zero viable) and fully on
  `CharCounter`/`add_signed`. The crate's real complexity (the converter guards)
  is fully mutation-tested.

### D6 — Dependencies (spike-confirmed)
- `serde = { version = "1", features = ["derive"] }` — `#[serde(transparent)]` +
  `#[serde(skip)] PhantomData`… **N/A now** (macro structs have a single `usize`
  field, so plain `#[serde(transparent)]` gives the bare-usize wire form for R14).
- `get-size2 = { version = "0.10", features = ["derive"] }` — `GetSize` derive →
  heap 0 (R15).
- **DROP `num-traits`** — `usize::checked_add_signed(isize)` / `wrapping_add_signed`
  are std (stable, present in 1.96), so num-traits is unused → `cargo machete`
  (gate:9) would fail it. The spec's dep list named it as a hedge; std suffices.

### D7 — add_signed (R8/R9)
`add_signed(delta: isize)`: a non-macro `fn add_signed_usize(v: usize, d: isize)
-> usize { match v.checked_add_signed(d) { Some(x) => x, None => { debug_assert!(
false, "CharOffset::add_signed overflow"); v.wrapping_add_signed(d) } } }`. R8 =
the `Some` arm (checked); R9 debug = the `debug_assert!` fires (`#[should_panic]`);
R9 release = `wrapping_add_signed` (the `cfg(not(debug_assertions))` test). The fn
is non-macro → its `+`/wrap is mutated.

### D8 — CharCounter (R16–R22) — the mutation-load surface
`pub struct CharCounter<'a> { s: &'a str, byte_pos: usize, char_pos: usize }`.
`new(s)` → `{ s, 0, 0 }` (R16). `char_offset(b: ByteOffset) -> CharOffset`:
```
let b = b.as_usize();
if b < self.byte_pos {                  // R21 backward misuse
    debug_assert!(false, "non-monotonic char_offset");
    return CharOffset::from(self.char_pos);          // release: defined, no backward scan
}
let b = if self.s.is_char_boundary(b) { b }
        else {                          // R22 non-boundary misuse
            debug_assert!(false, "char_offset off a UTF-8 boundary");
            floor_char_boundary(self.s, b)           // release: round down, never split
        };
self.char_pos += self.s[self.byte_pos..b].chars().count();   // R17/R18/R20 — scan ONLY the delta (R19)
self.byte_pos = b;
CharOffset::from(self.char_pos)
```
- `floor_char_boundary`: a small non-macro helper (std `str::floor_char_boundary`
  is nightly-only → hand-roll: walk down from `b` while `!is_char_boundary`). Fully
  unit + mutation tested.
- **R19 no-rescan:** the slice `s[byte_pos..b]` spans only the delta; the
  `byte_pos = b` update is what makes the *next* call resume. A dropped/!wrong
  `byte_pos` update yields a WRONG `char_pos` on the second monotonic call (caught
  by `r19` two-call output assert), and the perf contract (bytes scanned = spanned
  span) is asserted by a counter-instrumented variant (a `#[cfg(test)]` wrapper
  that sums `b - byte_pos`).

### File manifest
| File | Change |
|---|---|
| `crates/marley_text_offsets/src/lib.rs` | the `offset_newtype!` macro + 2 structs, the non-macro `add_signed_usize` + `floor_char_boundary` + `CharCounter`, `#![deny(missing_docs)]` doc comments, `#[cfg(test)] mod tests` |
| `crates/marley_text_offsets/Cargo.toml` | add `serde` (derive) + `get-size2` (derive) deps; `[dev-dependencies] trybuild` |
| `crates/marley_text_offsets/tests/ui/mixed_offsets_fail.rs` + `.stderr` | R2 trybuild compile-fail (CharOffset + ByteOffset, etc.) |

(Unit tests live in `#[cfg(test)] mod tests` inside lib.rs per the spec's test
names; the release-mode R9/R21/R22 asserts are `#[cfg(not(debug_assertions))]`
tests in the same module — coverage/mutation run in debug, so the debug
`#[should_panic]` arms are the ones that count toward the floors.)

### Regression test plan (one row per EARS clause — names from the spec)
| Test | R | kind | kills (mutation) |
|---|---|---|---|
| `r1_private_field_read_only_via_as_usize` | R1 | unit | (field privacy — compile/trybuild) |
| `r2` `tests/ui/mixed_offsets_fail.rs` | R2 | trybuild | type error asserted |
| `r3_zero_is_zero` (both flavors) | R3 | unit | `zero` non-zero const |
| `r4_as_usize_identity` | R4 | unit | accessor |
| `r5_from_usize_roundtrip` | R5 | unit | `From` |
| `r6_empty_range_is_o_to_o` | R6 | unit | start/end swap |
| `r7_range_constructs_self_bounds` | R7 | unit | start/end swap |
| `r8_add_signed_in_range` | R8 | unit | **`add_signed_usize` checked arm (non-macro → mutated)** |
| `r9_add_signed_debug_overflow_panics` | R9 | `#[should_panic]` | **debug_assert present** |
| `r9_add_signed_release_wraps` | R9 | `cfg(not(debug_assertions))` | **wrapping arm** |
| `r10_add_self_and_usize` | R10 | unit | (macro op — line-covered) |
| `r11_add_assign_self_and_usize` | R11 | unit | (macro op) |
| `r12_sub_and_sub_assign` | R12 | unit | (macro op) |
| `r13_ord_and_eq_by_value` | R13 | unit | (derived — not mutated) |
| `r14_serde_roundtrip` | R14 | unit | serde transparent |
| `r15_get_heap_size_zero` | R15 | unit | GetSize |
| `r16_char_counter_new_at_origin` | R16 | unit | **CharCounter init** |
| `r17_char_offset_equals_char_count` | R17 | unit | **scan count** |
| `r18_char_offset_zero_and_end` | R18 | unit | **boundary 0/len** |
| `r19_monotonic_resumes_without_rescan` | R19 | unit (+counter) | **`byte_pos` update; resume guard** |
| `r20_multibyte_char_lt_byte` | R20 | unit | **multibyte scan** |
| `r21_backward_call_debug_panics` | R21 | `#[should_panic]` | **backward guard (debug)** |
| `r21_backward_call_release_returns_current` | R21 | `cfg(not(debug_assertions))` | **release no-backward-scan** |
| `r22_non_boundary_debug_panics` | R22 | `#[should_panic]` | **boundary guard (debug)** |
| `r22_non_boundary_release_rounds_down` | R22 | `cfg(not(debug_assertions))` | **`floor_char_boundary`** |

Boundary cases threaded through r17–r22 (b == byte_pos, b == 0, b == len, a wide
scalar) to make the CharCounter guards **non-equivalent** under mutation.

### Risks / reversible-but-load-bearing
- **R-mut:** the macro/cargo-mutants interaction (disclosed in D5). If inspect/Chad
  wants the trivial ops mutation-tested too, refactor them through shared non-macro
  fns (a follow-up; doubles nothing since they'd be shared).
- **R-equiv:** equivalent mutants on the CharCounter `<`/boundary guards. Mitigated
  by the boundary tests above. If a genuinely-equivalent mutant survives gate:5, it
  is the ONE place a documented `mutants::skip` with a `// justification` may be
  needed — flag for validate (the gate has no blanket exclusions, so this would be
  an explicit, reviewed, per-mutant skip, not a baseline).
- **R-trybuild:** the `.stderr` snapshot is rustc-version-sensitive; regen with
  `TRYBUILD=overwrite` if a toolchain bump changes the message (note in the crate).
- **R-floor-cb:** `str::floor_char_boundary` is nightly-only → hand-rolled helper
  (non-macro, mutation-tested).

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement

Wrote the production code (no test module yet — Phase 4). Phase-gate correctly
allowed the `lib.rs` write (Design PASS).

### Built
- `crates/marley_text_offsets/src/lib.rs` — `offset_newtype!` macro → `pub struct
  CharOffset(usize)` / `ByteOffset(usize)` (private field, `#[serde(transparent)]`,
  `GetSize` derive, the per-newtype API); non-macro `add_signed_usize`,
  `floor_char_boundary`, and `CharCounter` (the mutation surface). `#![deny(missing_docs)]`
  satisfied (every public item documented).
- `crates/marley_text_offsets/Cargo.toml` — `serde` (derive) + `get-size2` (derive);
  `[dev-dependencies] trybuild`.

### Verified (sandbox disabled for the dep fetch — the stub workspace had no deps)
- `cargo check -p marley_text_offsets` → **Finished, clean**.
- `cargo clippy -p marley_text_offsets --all-targets -- -D warnings` → **clean**.
- `cargo fmt --check` → **clean**.
- `cargo deny check` → **advisories ok, bans ok, licenses ok, sources ok** (all 45
  transitive deps clear the permissive allowlist — no copyleft).
- `cargo audit -f Cargo.lock` → **exit 0** (no advisories). `cargo machete` →
  **no unused deps** (serde + get-size2 both used).

### Deviations from design (with reason)
1. **num-traits dropped** (D6) — `usize::checked_add_signed`/`wrapping_add_signed`
   are std (1.96); num-traits would be machete-flagged. Confirmed machete-clean.
2. **get-size2 0.10.1 pulls a heavy proc-macro tree** (attribute-derive / manyhow /
   syn → 45 total deps) for the `GetSize` derive. All permissive-licensed (deny
   green), builds fine. Noted as dep-weight to revisit if get-size2 is ever the only
   reason for that tree; the spec mandates get-size2 for R15, so kept.

### Carry to Validate
- Cargo.lock now updated (new deps) — the FULL gate's receipt will bind it (000's
  gate_state_hash fix). Tests (r1–r22 + trybuild + release-cfg) get WRITTEN and RUN
  in Phase 4; the FULL `scripts/gates.sh` must reach `GATE GREEN [full]` (coverage
  100 + MSI 100) — the dep fetch means the gate needs network (run unsandboxed).

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5)

3 critics (contract-fidelity vs R1–R22 · UTF-8 converter correctness · mutation-
readiness/clean-room), each verifying empirically (cargo build, a 30k-case
brute-force boundary sweep, `cargo mutants --list`).

**Verified CORRECT:** all R1–R22 satisfiable by the production code; no missing /
extra public item; no surface leak (`add_signed_usize`/`floor_char_boundary`
private; only CharOffset/ByteOffset/CharCounter exposed — seam-contracts §1); R2
correct by construction (the macro body never names the sibling type → mixing
fails to compile); the slice `s[byte_pos..b]` **proven panic-free** (induction:
`byte_pos` is always a boundary `≤ b ≤ len`; 30,470 cases, debug+release, zero
panics); `floor_char_boundary` returns the greatest boundary ≤ b; no unwrap/expect;
clean-room original (behavior-derived; `offset_newtype!` is internal, not Warp's
`#[macro_export] impl_offset!`).

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| 1 | HIGH | Mutant `i > 0` → `i >= 0` in `floor_char_boundary` is **EQUIVALENT** (differ only at i==0 where `is_char_boundary(0)` is always true) → would survive → gate:5 RED. | **REAL** (proven 0 differing inputs) | Dropped the redundant `i > 0` guard (0 is always a boundary → the walk self-terminates). The `>` is gone → the mutant can't be generated. |
| 2 | HIGH | The `debug_assert!(false, …)` misuse arms (R9 wrap, R21 return-current, R22 round-down) leave the release-defined fallback lines **unreachable in the debug coverage build** → gate:4 < 100% RED. | **REAL** | Restructured all three to **assert-the-invariant + always-compute-the-defined-result**: `add_signed_usize` computes `wrapping_add_signed` then `debug_assert!(checked.is_some())`; `char_offset` does `debug_assert!(b>=byte_pos); b=b.max(byte_pos)` and `debug_assert!(is_char_boundary(b)); b=floor_char_boundary(s,b)`. The defined result is now executed (and debug-covered) on every call; debug still panics on misuse (R9/R21/R22 debug); release still clamps/rounds/wraps (R9/R21/R22 release). |
| 3 | LOW | Spec `reuses`/Dependencies list `num-traits`, unused (std `checked_add_signed` suffices) → would mismatch a deps-match-spec check + machete. | **REAL** | Dropped `num-traits` from SPEC-text-offsets.spec.md (3 spots). |
| 4 | LOW | `b > s.len()` routes through the R22 path with a slightly imprecise assert message. | **ACCEPTED** — spec-conformant (b>len ⇒ !is_char_boundary ⇒ R22; greatest boundary ≤ b is len); message cosmetic. | none. |
| 5 | LOW | Plain `Add`/`Sub` overflow-panic in debug (unlike `add_signed`'s defined wrap). | **REJECTED** — R10–R12 don't mandate defined-on-overflow (only R9 does); standard integer behavior, spec-permitted. | none. |

Post-fix: clippy + fmt clean; `cargo mutants --list` → **17 → 10 mutants, all
killable, no equivalents**.

### Mutation kill-list (MANDATORY for Validate — 10 mutants, MSI 100, zero exclusions)
All target the non-macro code (macro-emitted struct ops generate 0 mutants — line-
covered + unit-tested instead). The private fns need **in-crate** `#[cfg(test)]`.

| Mutant @ line | Killed by |
|---|---|
| `add_signed_usize -> 0` / `-> 1` (L33) | `r8_add_signed_in_range`: `from(5).add_signed(3).as_usize()==8` |
| `floor_char_boundary -> 0` / `-> 1` (L48) | `floor_cb_on_boundary`: `floor_char_boundary("aé",3)==3`; `floor_cb_off_boundary`: `("aé",2)==1` |
| `delete ! ` (L49) | `floor_cb_off_boundary` (mutant returns 2≠1 or loops→timeout=caught) |
| `-= → +=` (L50) | `floor_cb_off_boundary` (i ascends → wrong) |
| `-= → /=` (L50) | `floor_cb_off_boundary` (i/=1 unchanged → infinite loop → Timeout = Caught) |
| `char_offset -> Default::default()` (L193) | `r17`: `new("abc").char_offset(3.into())==3.into()` (mutant 0) |
| `char_pos += → -=` (L213) | `r17` (0−3 underflow panic, debug) |
| `char_pos += → *=` (L213) | `r17` (0×3=0 ≠ 3) |

Plus the spec's debug `#[should_panic]` (r9/r21/r22) for the misuse panics, the
`cfg(not(debug_assertions))` release-behavior tests (r9 wrap / r21 return-current /
r22 round-down — now also debug-COVERED via the restructure), R2 trybuild, R14
serde-bare-usize, R15 heap-0, and the R19 two-call resume assert.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate

### Tests written (all from the spec test plan + the inspect kill-list)
- In-crate `#[cfg(test)] mod tests` in lib.rs: r1, r3–r22 named tests (r3 both
  flavors), `byteoffset_api_smoke` (covers the second macro instantiation),
  `floor_cb_on_boundary` / `floor_cb_off_boundary` + direct `add_signed_usize`
  asserts (the private-fn kills), debug `#[should_panic(expected=…)]` for
  r9/r21/r22, and `#[cfg(not(debug_assertions))]` r9/r21/r22 release-behavior tests.
- `tests/ui/mixed_offsets_fail.rs` + `tests/compile_fail.rs` (trybuild R2): mixing
  the flavors fails to compile — `.stderr` captured `E0277 cannot add ByteOffset to
  CharOffset` + 2× `E0308 mismatched types`.
- `serde_json` dev-dep for the R14 bare-`usize` wire assertion.

### Runs (real output)
- `cargo nextest run -p marley_text_offsets` → **25 tests, 25 passed, 0 skipped**
  (24 unit + the trybuild compile-fail). `cargo test --doc` → 0 (no doctests).
- **`scripts/gates.sh` (FULL) → `GATE GREEN [full]`, 16 passed / 0 failed:**
  - gate:4 coverage **100.00%** — 201/201 lines, 40/40 functions, 367/367 regions
    on lib.rs (the restructure put the wrap/clamp/floor lines on the normal debug
    path; the release-cfg tests verify the release *behavior*).
  - gate:5 mutation **MSI 100.0%** — **10 caught / 0 missed** (the inspect kill-list
    held; no equivalents).
  - gate:6 miri SKIP (no unsafe); gate:15 visual SKIP (marley_text_offsets is
    `visual_acceptance: N/A`); gates 1–3,7–14,16 all PASS.
  - **Receipt written** (`e85012df…`) binding crates/**/*.rs + the gate-defining
    files — the first FULL receipt, satisfying enforce-commit-gate for the commit.
- One fix during validate: `cargo fmt` wrapped a long assert line (no logic change);
  re-ran FULL → green.

**Pre-existing failures:** none. **Skips:** gate:6/15 (correctly N/A for a pure
non-unsafe lib crate); the 3 release-cfg tests compile out of the debug run by
design (their lines are covered by the restructured normal path). No `#[ignore]`,
no lowered floor, no mutation exclusion.

**Phase 4 status:** PASS — FULL gate green (cov 100 / MSI 100), receipt written. → Phase 5 Complete.
