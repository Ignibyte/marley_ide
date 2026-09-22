---
pipeline_id: 46b05b93-46e3-4f85-a6a0-c72bdc417bb1
ticket: forge#364 (fcd70d73-5015-4c34-bae5-19d150a4d5a8) · local docs/planning/tickets/open/TICKET-364-harden-shape2-headless-polls.md
aar_id: b05249af-f448-4b87-80dd-bba0c90f9f66
status: Phase 5 — Complete PASS
title: Harden the 7 Shape-2 worker-thread headless polls (probe-first for-0..100 fixed budget) against load — a probe-first poll_until sibling
type: bug
milestone: M22
references: [headless_drive.rs:105 POLL_CEILING / :117 poll_until (the #334 Shape-1 template, body-then-probe) / :2143 poll_until_respects_a_finite_ceiling_headless (the finiteness proof) / :89 tick_pump / :1918 syntax_async_large_file_lands_off_thread_headless (the LAG assertion :1970-1976, 2 polls :1952/:1978) / :6677 t349_bracket / :6738 t349_edit / :6820 ladder_reads / :6891 sticky_headers_read / :6963 ladder_falls_back / lib.rs:55 #[cfg(test)] mod headless_drive, #334 poll_until]
---

## Title
Add a probe-first deadline helper `poll_until_pre` (mirroring #334's body-first `poll_until`, leaving it untouched)
and rewire all 7 Shape-2 worker-thread headless polls to it — a generous finite ceiling in place of the load-flaky
fixed `for _ in 0..100` (~2.5s) budget, preserving `syntax_async`'s standalone pre-tick LAG assertion.

## Scope
### In
- **`poll_until_pre<T: Default>(vcx, ceiling, probe) -> (bool, T)` (headless_drive.rs, `#[cfg(test)]`):** the
  PROBE-FIRST twin of `poll_until` — evaluate the probe ONCE before the loop, then `while !met && now < deadline {
  sleep(25ms); tick_pump(vcx); probe }`. Same signature as `poll_until`, same single-path shape (no early return),
  same `(met, observed)` return. Leaves `poll_until` byte-for-byte unchanged.
- **Rewire all 7 Shape-2 sites** to `poll_until_pre(&mut vcx, POLL_CEILING, |vcx| (…probe…, …diag…))`, deleting each
  raw `for _ in 0..100 { if probe { break } sleep; advance_clock; run_until_parked }` loop. The `#363` sites'
  `cache_matches_live` closures fold into the probe (closing over `window`); `syntax_async`'s `key(…)` probe folds
  in, and its STANDALONE LAG read (`simulate_keystrokes("x")` → direct `key()` → `assert_ne!`) stays exactly as-is
  BETWEEN the two rewired polls.
- **A finiteness unit for the new helper** (`poll_until_pre_respects_a_finite_ceiling_headless`, mirror
  `poll_until_respects_a_finite_ceiling_headless`) + **a pre-satisfied-probe unit** (a probe met on the first call
  returns `met=true` without advancing the clock — the probe-first guarantee).

### Out (explicitly deferred)
- Any change to `poll_until` itself or its Shape-1 real-PTY call sites (#334 — UNCHANGED; REQ-334-unchanged).
- Any app-code / marley_syntax change (the diff is headless_drive.rs ONLY — no product behavior changes).
- The 2 false-positive `for _ in 0..100 { dispatch_for_test("add-cursor-…") }` ACTION loops (#360 add-cursor
  drives) — deterministic, no probe/sleep/pump, NOT polls.
- Merging `poll_until` + `poll_until_pre` into one flagged fn (a DRYer alternative considered + rejected at Design —
  it refactors shipped #334 code for ~4 saved lines; the sibling leaves #334 untouched).

## Reference (§20)
N/A — Marley's own headless test harness (the hybrid real-`sleep` for the real worker OS thread + mock-clock
`advance_clock` for the pump timer). No Warp/Zed source read or behavior matched — this is internal test
infrastructure.

### Prior art
- **Our own code (the fix IS a #334 sibling):** `poll_until` (headless_drive.rs:117) is the exact template — the
  `deadline = now + ceiling`, the `while !met && now < deadline { sleep; tick_pump; probe }` loop, the `(met,
  observed)` return, the single-path/no-early-return coverage discipline, and `POLL_CEILING = 30s`.
  `poll_until_respects_a_finite_ceiling_headless` (:2143) is the finiteness-proof template. #364 mirrors both, with
  the ONE delta being probe-BEFORE-the-first-tick.
- **Permissive deps:** gpui's `VisualTestContext` owns `run_until_parked` + `executor().advance_clock`, but NOT a
  hybrid real-sleep + mock-tick deadline poll (checked — the #334 `poll_until` owns this seam because the wait
  straddles a real OS worker thread AND the mock-clock pump; neither gpui primitive alone spans both). No external
  owner; the fix is a Marley-local sibling.
- **Behavior maps / published:** N/A — test infrastructure, no reference-app behavior.

## Locked-In Decisions
- **D1 — a probe-first SIBLING `poll_until_pre`, not a flag on `poll_until`.** Leaves the shipped #334 `poll_until`
  byte-for-byte unchanged (REQ-334-unchanged trivial; zero risk to the Shape-1 sites). The ~4 duplicated loop lines
  are the price of not touching proven code; both fns stay single-path. (Design re-confirms against a shared-inner
  alternative.)
- **D2 — same signature + single-path shape as `poll_until`.** `fn poll_until_pre<T: Default>(vcx, ceiling, mut
  probe: impl FnMut(&mut VisualTestContext) -> (bool, T)) -> (bool, T)`. Body: `let deadline = now + ceiling; let
  mut last = probe(vcx); while !last.0 && now < deadline { sleep(25ms); tick_pump(vcx); last = probe(vcx); } last`.
  No early return, no timeout tail → coverage stays 100 (every line runs on a passing drive; the pre-satisfied +
  deadline paths carried by the 2 units).
- **D3 — `syntax_async`'s LAG assertion is a STANDALONE read, untouched.** The "cache lags immediately after the
  keystroke" `assert_ne!` (:1970-1976) calls `key()` DIRECTLY between the two loops, not through either loop — so it
  is independent of the loops' ordering. Rewiring the two loops to `poll_until_pre` leaves the LAG read verbatim;
  probe-first is chosen to keep each rewired loop a zero-behavior-change transplant of its predecessor.
- **D4 — no new mutation surface, `#[cfg(test)]`.** `mod headless_drive` is `#[cfg(test)]` (lib.rs:55) → cargo-mutants
  (normal build) never sees `poll_until_pre` → gate:5 unaffected, no `mutants::skip` needed (exactly `poll_until`).
  headless_drive.rs is coverage-INCLUDED (not in the gate:4 ignore-regex) → the single-path shape carries cov 100.
- **D5 — reuse `POLL_CEILING` (30s).** The same generous finite wall-clock ceiling #334 established; a parse landing
  is faster + less contended than a PTY spawn, so 30s is comfortably generous. One ceiling constant, both helpers.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-PROBE-FIRST | The helper shall evaluate the probe BEFORE the first pump tick — a probe already met on entry returns `met=true` without advancing the clock. | unit: a probe that returns met on its first call → `poll_until_pre` returns `(true, …)`; assert the mock clock did not advance (or the probe ran exactly once). |
| REQ-FINITE-CEILING | WHEN the probe never reports met, the helper shall give up at the ceiling (return not-met) rather than hang. | unit `poll_until_pre_respects_a_finite_ceiling_headless` — always-unmet probe + a 50ms ceiling → returns `(false, …)`, no hang (mirror #334's). |
| REQ-LAG-PRESERVED | `syntax_async`'s "cache lags immediately after the keystroke" assertion shall still hold. | the rewired `syntax_async_large_file_lands_off_thread_headless` runs green — the standalone pre-tick `assert_ne!` between the two `poll_until_pre` polls passes. |
| REQ-ALL-SITES-REWIRED | ALL 7 Shape-2 polls shall use the helper — no raw probe-first `for _ in 0..100` loop remains. | grep: zero `for _ in 0..100` in the 7 named fns; all 7 drives run green through `poll_until_pre`. |
| REQ-334-UNCHANGED | The #334 Shape-1 `poll_until` + its sites + `poll_until_respects_a_finite_ceiling_headless` shall be unchanged. | `git diff` shows `poll_until` untouched; the Shape-1 sites + finiteness test stay green. |

## Phase Plan
- **P2 Design** — settle `poll_until_pre`'s exact body (single-path, probe-first) vs the shared-inner alternative
  (recommend the sibling, D1); the probe-closure folding for each of the 7 sites (esp. the `#363` `cache_matches_
  live`→`|vcx| (…, ())` and `syntax_async`'s `key`→`(met, (n,v,cache))`); confirm the LAG read stays standalone;
  the 2 units.
- **P3 Implement** — add `poll_until_pre` + the 2 units; rewire the 7 sites (headless_drive.rs only).
- **P3.5 Inspect** — ★ probe-first ordering (pre-tick) correct; the LAG read untouched + still between the two polls;
  ALL 7 rewired (no raw loop; the 2 action loops untouched); `poll_until` byte-identical; single-path cov; no
  app/marley_syntax change (diff = headless_drive.rs only); no new mutants (`#[cfg(test)]`).
- **P4 Validate** — RUN the 2 units + all 7 rewired drives + the #334 Shape-1 sites/finiteness (regression) + the
  `--diff` gate (cov 100; gate:5 unaffected — `#[cfg(test)]`).
- **P5 Complete** — CHANGELOG + a headless-harness doc note (the Shape-1 `poll_until` / Shape-2 `poll_until_pre`
  pair + when each applies).
