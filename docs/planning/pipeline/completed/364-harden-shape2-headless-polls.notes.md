# Harden the Shape-2 worker-thread headless polls — Notes

- **Forge ticket:** #364 `fcd70d73-5015-4c34-bae5-19d150a4d5a8`
- **AAR:** `b05249af-f448-4b87-80dd-bba0c90f9f66`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-364-harden-shape2-headless-polls.md
- **Pipeline spec:** 364-harden-shape2-headless-polls.spec.md
- **pipeline_id:** 46b05b93-46e3-4f85-a6a0-c72bdc417bb1
- **Live on:** `4f85856` (FIFTH + LAST of the goal /work 360,361,362,363,364)

## Phase 1 — Plan
- **Request:** harden the Shape-2 worker-thread headless polls (fixed `for _ in 0..100` ~2.5s budget) against load —
  the #334 follow-up. A test-flake/gate bug; test-harness-only.
- **Classification / tier:** work pipeline; test infrastructure; a probe-first sibling of #334's `poll_until` + a
  mechanical rewire of 7 sites. One shippable slice.
- **Forge recall (§18.3):** bulletins none. `aar-open` → `b05249af`. `knowledge-context` (Plan) logged 13
  surfacings — top prevention rule (`df6ae107`, semantic 0.855) is the #334 `poll_until` PR; the governing prior art
  is #334 itself (read directly).
- **★ Prior-art sweep (required):** (1) behavior maps — N/A (test harness). (2) published — N/A. (3) permissive deps
  — gpui's `VisualTestContext` has `run_until_parked` + `executor().advance_clock`, but NO hybrid real-`sleep` +
  mock-tick DEADLINE poll: the wait straddles a real OS worker thread (needs real time) AND the mock-clock pump
  (needs `advance_clock`), and no single gpui primitive spans both — so #334's `poll_until` owns this seam and #364
  is a Marley-local sibling. Recorded in the spec.
- **★ Recon (on `4f85856`) — fully de-risked:**
  1. `poll_until` (:117) = `deadline = now + ceiling; let mut last = (false, T::default()); while !last.0 && now <
     deadline { sleep(25ms); tick_pump(vcx); last = probe(vcx); } last` — body-THEN-probe (Shape-1), single-path,
     `POLL_CEILING = 30s` (:105), `tick_pump` (:89) = `advance_clock(40ms) + run_until_parked()`. Finiteness proven
     by `poll_until_respects_a_finite_ceiling_headless` (:2143, always-unmet probe + 50ms ceiling → not-met, no
     hang). #364 adds the probe-FIRST twin.
  2. **★ `syntax_async` LAG is a STANDALONE read, NOT load-bearing on loop order.** The `assert_ne!` (:1970-1976)
     calls `key()` DIRECTLY between the two `for _ in 0..100` loops (:1952 land, :1978 catch-up), right after
     `simulate_keystrokes("x")` and before the catch-up loop — with no `advance_clock` in between, so the cache
     provably lags. It does NOT go through either loop → independent of body-vs-probe-first. Rewiring the two loops
     to `poll_until_pre` leaves the LAG read verbatim. Probe-first is chosen anyway to make each rewired loop a
     zero-behavior-change transplant.
  3. The other 6 sites (t349 ×2, #363 ×3, + syntax_async's 2 loops) are plain probe-first-until-`cache==live`; a
     probe-first deadline helper with a generous finite ceiling works uniformly. The #363 sites use a `cache_matches_
     live` closure taking `(&window, &mut vcx)` — the probe folds to `|vcx| (cache_matches_live(&window, vcx), ())`
     closing over `window` (a `Copy` `WindowHandle`); `poll_until`'s `FnMut(&mut VisualTestContext) -> (bool, T)`
     signature fits unchanged.
  4. **cov/MSI:** `mod headless_drive` is `#[cfg(test)]` (lib.rs:55) → cargo-mutants (normal build) never enumerates
     `poll_until_pre` (no `mutants::skip` needed, exactly like `poll_until`); headless_drive.rs is NOT in the gate:4
     ignore-regex (:217) → coverage-INCLUDED, carried by the single-path shape + the 2 units + the 7 drives.
- **Scope confirmed (grep):** exactly 7 Shape-2 probe-first polls (syntax_async ×2, t349_bracket, t349_edit, +
  #363's ladder_reads/sticky_headers_read/ladder_falls_back); the :5886/:5950 `dispatch_for_test("add-cursor-…")`
  loops are #360 ACTION loops (no probe/sleep/pump) — EXCLUDED.
- **Decisions:** D1 sibling (not a flag — leaves #334 untouched); D2 same signature + single-path; D3 the LAG read
  stays standalone; D4 no new mutation surface (`#[cfg(test)]`); D5 reuse `POLL_CEILING` (30s). (See spec.)
- **Prior art:** #334's `poll_until` + finiteness proof (the sibling template). §20 N/A.
- **EARS:** REQ-PROBE-FIRST, REQ-FINITE-CEILING, REQ-LAG-PRESERVED, REQ-ALL-SITES-REWIRED, REQ-334-UNCHANGED (see
  spec).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** A probe-first sibling of #334's `poll_until` + a mechanical rewire of the 7 Shape-2
sites — all in `crates/marley_app/src/headless_drive.rs` (`#[cfg(test)]`). No app/marley_syntax change: this is
test infrastructure, so §14's product concerns (typed errors, no-panic on input paths, adapter-confined spawns) are
N/A — a test helper MAY panic (the finiteness/assert path IS the test). §20 CONFIRMED N/A (Marley's own hybrid
real-`sleep` + mock-clock-`advance_clock` poll; no Warp/Zed source or behavior). The design MATCHES the #334
`poll_until` prior art exactly, with the single delta being probe-before-first-tick.

**★ The helper (D1/D2 — settled against the code).** A SIBLING `poll_until_pre`, NOT a flag/shared-inner: leaving
`poll_until` byte-for-byte untouched makes REQ-334-unchanged trivial and risks nothing in the Shape-1 real-PTY
sites; the ~4 duplicated loop lines are the accepted price. (A shared `poll_until_inner(probe_first: bool)` would
save them but refactors shipped code AND adds a branch mutant surface were it not `#[cfg(test)]` — the sibling is
strictly lower-risk here.) Body (single-path, no early return — coverage stays 100 like `poll_until`):
```
fn poll_until_pre<T: Default>(
    vcx: &mut VisualTestContext,
    ceiling: std::time::Duration,
    mut probe: impl FnMut(&mut VisualTestContext) -> (bool, T),
) -> (bool, T) {
    let deadline = std::time::Instant::now() + ceiling;
    let mut last = probe(vcx); // PROBE FIRST (pre-tick) — the ONE delta vs poll_until
    while !last.0 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(25));
        tick_pump(vcx);
        last = probe(vcx);
    }
    last
}
```
Coverage of every line by a passing suite: the `deadline` + pre-`probe` + `last` run on EVERY drive; the `while`
body runs whenever the pre-probe isn't met (all 7 real drives — the off-thread parse is never instant); the
while-false-on-entry (pre-satisfied) path is carried by the REQ-PROBE-FIRST unit; the deadline-exceeded exit by the
finiteness unit. Plain-backtick doc mirroring `poll_until`'s, noting the ONE difference (probe-before-first-tick,
for the Shape-2 worker-parse polls whose predecessors checked the probe at the loop top).

**★ The 7 rewires (each: delete the raw `for _ in 0..100 {…}` + keep the `assert!`).**
- **`syntax_async_large_file_lands_off_thread_headless` (2 polls).** Probe = the existing `key(&window, vcx)` →
  `(nonce, version, cache)`; met = `cache == Some((nonce, version))`. Loop 1 (:1952) → `let (landed, _) =
  poll_until_pre(&mut vcx, POLL_CEILING, |vcx| { let (n, v, c) = key(&window, vcx); (c == Some((n, v)), (n, v, c))
  }); assert!(landed, …)`. ★ The STANDALONE LAG block (`simulate_keystrokes("x")` → direct `key()` → `assert_ne!`,
  :1970-1976) stays VERBATIM between the two polls (D3 — it never went through a loop; it is independent of loop
  ordering). Loop 2 (:1978, `caught_up`) → the identical `poll_until_pre` fold. The `key` closure stays as-is.
- **`t349_bracket` (:6677) / `t349_edit` (:6738) / `ladder_reads` (:6820) / `sticky_headers_read` (:6891) /
  `ladder_falls_back` (:6963).** Each already defines a `cache_matches_live = |window, cx| bool` closure; keep it,
  wrap: `let (hit/landed, _) = poll_until_pre(&mut vcx, POLL_CEILING, |vcx| (cache_matches_live(&window, vcx),
  ())); assert!(hit, …)`. `T = ()` (no diagnostic; the assert message is static). CONFIRMED (recon): none of these
  6 has a pre-tick assertion — each just `assert!(hit/landed)` after the loop. The `cache_matches_live` closure and
  every post-loop assertion/step stay unchanged.

**★ The 2 new units.**
- **REQ-PROBE-FIRST** `poll_until_pre_returns_without_ticking_when_pre_satisfied_headless`: capture a `let mut calls
  = 0usize;` counter in the probe: `let (met, _) = poll_until_pre(&mut vcx, POLL_CEILING, |_vcx| { calls += 1;
  (true, ()) });` → `assert!(met)` + `assert_eq!(calls, 1, "the pre-satisfied probe returned before any tick")`.
  Call-count == 1 proves the loop body (and thus `tick_pump`) never ran — the probe-first, no-tick guarantee. (A
  clock-read is harder + noisier headless; the call count is the crisp proof.) Boots a window like the #334 unit.
- **REQ-FINITE-CEILING** `poll_until_pre_respects_a_finite_ceiling_headless`: a verbatim mirror of
  `poll_until_respects_a_finite_ceiling_headless` (:2143) — boot, `let (met, seen): (bool, Option<String>) =
  poll_until_pre(&mut vcx, Duration::from_millis(50), |_vcx| (false, None));` → `assert!(!met)` +
  `assert!(seen.is_none())` + `reap_sessions`. The always-unmet probe + tiny ceiling → the loop gives up (finite),
  not a hang; also covers `poll_until_pre`'s deadline-exceeded exit + the while-body (the pre-probe is unmet → the
  loop runs at least once before the 50ms deadline).

**File manifest (1 file):**
- `crates/marley_app/src/headless_drive.rs` — add `poll_until_pre` (near `poll_until` :117) + the 2 units (near
  `poll_until_respects_a_finite_ceiling_headless` :2143); rewire the 7 Shape-2 poll sites. NOTHING else.

**★ Regression Test Plan.**
| # | Proves | Test |
|---|---|---|
| U1 | REQ-PROBE-FIRST | `poll_until_pre_returns_without_ticking_when_pre_satisfied_headless` — a first-call-met probe returns `(true, …)` with `calls == 1` (no tick). |
| U2 | REQ-FINITE-CEILING | `poll_until_pre_respects_a_finite_ceiling_headless` — always-unmet probe + 50ms ceiling → `(false, None)`, no hang (mirror #334's). |
| T1 | REQ-LAG-PRESERVED | the rewired `syntax_async_large_file_lands_off_thread_headless` runs green — its standalone pre-tick `assert_ne!` (LAG) between the two `poll_until_pre` polls still passes. |
| T2 | REQ-ALL-SITES-REWIRED | all 7 rewired drives (`syntax_async`, `t349_bracket`, `t349_edit`, `ladder_reads`, `sticky_headers_read`, `ladder_falls_back`) run green; grep shows zero `for _ in 0..100` in those fns. |
| T3 | REQ-334-UNCHANGED | `poll_until` byte-identical (git diff); the Shape-1 sites (:2117/:2128/:2206) + `poll_until_respects_a_finite_ceiling_headless` stay green. |

- **cov/MSI:** `poll_until_pre` single-path → cov 100 via U1 + U2 + the 7 drives; `#[cfg(test)] mod headless_drive`
  (lib.rs:55) → cargo-mutants (normal build) enumerates NO mutants (no `mutants::skip`, exactly like `poll_until`);
  confirm at Validate via the `--diff` gate. **Uncoverable-honestly:** none — every `poll_until_pre` line runs on a
  passing suite (the 3 paths: pre-satisfied via U1, deadline via U2, met-after-ticks via the 7 drives).

**Risks / decisions.** (a) D1 sibling over shared-inner (recommend + settled — #334 untouched, lower risk). (b) ★ D3
the `syntax_async` LAG block stays STANDALONE between the two rewired polls (never fold it into a poll). (c) the
probe closure fits both styles via `T: Default` (`key`→`(n,v,cache)` diag, `cache_matches_live`→`()`). (d) single-
path cov (the pre-satisfied + deadline paths on the 2 units). (e) the pre-satisfied unit proves no-tick via a probe
CALL-COUNT (==1), not a clock read. (f) `#[cfg(test)]` → no mutation surface (no skip). §14 N/A (test harness); §20
N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**Built to the manifest — ONE file (`crates/marley_app/src/headless_drive.rs`), all via the Edit tool. `cargo
check`/`clippy -D warnings`/`fmt` clean (only the pre-existing `block v0.1.6` dep note).**

- **`poll_until_pre`** added immediately after `poll_until` (probe-first: `let mut last = probe(vcx);` before the
  `while !last.0 && now < deadline { sleep(25ms); tick_pump(vcx); last = probe(vcx); }`). Plain-backtick doc noting
  the ONE delta vs `poll_until`. `poll_until` / `POLL_CEILING` / `tick_pump` left byte-for-byte unchanged.
- **All 7 Shape-2 sites rewired** to `poll_until_pre(&mut vcx, POLL_CEILING, |vcx| …)`:
  - `syntax_async` — 2 polls, inline `key(&window, vcx)` probe → `(cache == Some((n,v)), (n,v,cache))`; the
    STANDALONE LAG block (`simulate_keystrokes("x")` + direct `key()` + `assert_ne!`) left VERBATIM between them (D3).
  - the 5 cache sites (`t349_bracket`/`t349_edit`/`ladder_reads`/`sticky_headers_read`/`ladder_falls_back`) — the
    `cache_matches_live` closure kept, wrapped `|vcx| (cache_matches_live(&window, vcx), ())`; each flag name +
    assert message preserved. (t349_edit + ladder_falls_back are byte-identical block+assert → one `replace_all`.)
- **2 units** added after `poll_until_respects_a_finite_ceiling_headless`:
  `poll_until_pre_returns_without_ticking_when_pre_satisfied_headless` (a first-call-met probe → `calls == 1`, no
  tick) + `poll_until_pre_respects_a_finite_ceiling_headless` (always-unmet + 50ms ceiling → not-met, no hang).

**Verification:** `grep "for _ in 0..100"` → only the 2 #360 add-cursor ACTION loops remain (confirmed
`dispatch_for_test("add-cursor-below"/"above")`, no probe/sleep) + the one in the `poll_until_pre` doc backticks;
9 `poll_until_pre(` calls (7 rewires + 2 units); no orphaned `let mut hit/landed/caught_up = false`; diff =
headless_drive.rs only.

**★ Deviation from design (1, sound):** DROPPED the `T: Default` bound from `poll_until_pre` (design said mirror
`poll_until`'s `<T: Default>`). Rationale: `poll_until_pre` seeds `last` from the FIRST `probe(vcx)` call (probe-
first), so it NEVER calls `T::default()` — the bound is genuinely unused. Keeping it would have needlessly required
`BufferVersion: Default` for `syntax_async`'s observed-tuple `T = (u64, BufferVersion, Option<(u64, BufferVersion)>)`
(a real compile risk). Dropping it is correct AND a natural consequence of probe-first; documented in the fn's
rustdoc ("Unlike `poll_until` there is no `T: Default` bound…"). `poll_until` keeps its `<T: Default>` (it seeds
`last` from a default before the body-first probe).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 critics (Agent general-purpose), parallel — Critic 1 = the helper + `syntax_async` (the LAG + probe-first
correctness + the dropped bound); Critic 2 = the 5 cache rewires + the 2 units + cov/mutation/scope. Both READ-ONLY
+ `cargo check` (HARD RULE: never git checkout/stash/restore/reset).**

### Findings ledger

**F1 [MEDIUM → FIXED] the pre-satisfied unit's `calls == 1` did NOT discriminate probe-first from body-first**
(Critic 2) — headless_drive.rs `poll_until_pre_returns_without_ticking_when_pre_satisfied_headless`. **REAL.**
Traced: for a probe met on its first call, BOTH helpers reach `calls == 1` — probe-first `poll_until_pre` probes
(met) then skips the loop; body-first `poll_until` seeds `(false, default)`, enters the loop, ticks once, THEN
probes (met) and exits. A call count can NEVER discriminate the two (both call the probe the same number of times
for a given "met on call N"; the ONLY difference is the TICK count — probe-first saves exactly one tick). So the
unit passed against the very body-first helper #364 replaces — it did not pin the probe-first invariant it exists
to prove (the same class as #362's vacuous drive / #363's tautological equivalence). **FIX (source, §0):** verified
gpui exposes `BackgroundExecutor::now()` (executor.rs:350 → `TestDispatcher::now` = `start_time + time`, the MOCK
clock that `advance_clock` — inside `tick_pump` — moves; `poll_until_pre`'s `deadline` uses the SEPARATE std wall
clock). Added `let before = vcx.cx.executor().now(); … let advanced = vcx.cx.executor().now() - before;
assert_eq!(advanced, Duration::ZERO, "…probe-first, not body-first")`. A zero mock-clock advance is the crisp proof
the probe ran BEFORE any tick — a body-first poll would show +40 ms. RAN both units → PASS (`advanced == ZERO`
holds; the assertion genuinely discriminates). Recorded as a prevention rule (the ordering-invariant-via-call-count
anti-pattern).

**F2 [LOW → REJECTED] the finiteness unit's `assert!(seen.is_none(), "…carried through")` is trivially true**
(Critic 2) — the always-`None` probe makes `seen` `None` regardless of whether the value is threaded. **REJECTED as
consistent-with-precedent:** it is a verbatim mirror of #334's shipped `poll_until_respects_a_finite_ceiling_headless`
(:2159), and the LOAD-BEARING assertion of that unit is the finiteness proof (`!met`, no hang), which IS real and
solid. Strengthening the "carried through" claim would need a drive asserting a non-trivial observed value (none of
the 7 rewires use the diagnostic — all discard it `_`), diverging from the established #334 idiom for no real gain.
Left as-is.

### Verified CLEAN (no change)
- **Probe-first helper correct + single-path** (Critic 1): the body probes ONCE before the loop, then `sleep +
  tick_pump + probe` byte-identical to `poll_until`; no early return; `while !last.0 && now < deadline` stops on met
  OR deadline; no off-by-one.
- **★ The dropped `T: Default` bound is SOUND + REQUIRED** (Critic 1): `poll_until_pre` never calls `T::default()`
  (seeds `last` from the first probe); `BufferVersion` (editor/types.rs:22) derives `Copy/Eq/Ord/Hash` but NOT
  `Default`, so `poll_until<T: Default>` could NOT accept `syntax_async`'s observed tuple — dropping the bound was
  necessary, not cosmetic.
- **★ The `syntax_async` LAG preserved** (Critic 1): `simulate_keystrokes` (gpui test_context.rs) only
  `run_until_parked`s (never `advance_clock`s), so between the keystroke and the direct `key()` read the mock clock
  never moves → the cache stays stale → `assert_ne!` holds; the LAG block is a direct read BETWEEN the two rewired
  polls, structurally independent of the helper's ordering — untouched.
- **The 5 cache rewires byte-correct** (Critic 2): each `cache_matches_live` closure unchanged; flag names + assert
  messages preserved; no orphan `let mut …=false`; the `replace_all` hit EXACTLY the 2 identical `landed` sites
  (t349_edit + ladder_falls_back), no third; 9 `poll_until_pre` calls; downstream test bodies (incl. t349_edit /
  ladder_falls_back's edit-then-assert-stale) untouched.
- **cov/mutation/scope** (Critic 2): `poll_until_pre` single-path (pre-probe + loop via the 7 drives, deadline +
  pre-satisfied via the 2 units) → cov 100; `#[cfg(test)] mod headless_drive` (lib.rs:55) → no mutants (no skip,
  like `poll_until`); headless_drive.rs NOT in the gate:4 ignore-regex; the only remaining `for _ in 0..100` are the
  2 #360 add-cursor ACTION loops (correctly excluded) + the doc backtick; `poll_until` + its Shape-1 sites +
  finiteness test unchanged.

**Post-fix checks:** `cargo nextest -p marley -E 'test(poll_until_pre)'` → 2 passed (incl. the strengthened
mock-clock-delta assertion); `cargo fmt` clean; `cargo check` clean. Diff = headless_drive.rs only.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**The tests were written in Phase 3 + the F1 inspect fix; Phase 4 RAN them + the full gate. All green.**

**1. The 2 units + the 7 rewired drives** (`cargo nextest -p marley -E 'test(poll_until_pre) + test(syntax_async…)
+ test(t349_bracket…) + test(t349_edit…) + test(ladder_reads…) + test(sticky_headers_read…) + test(ladder_falls_
back…)'`): **8 tests, 8 passed** (2 units + 6 drive fns — `syntax_async`'s 2 polls are one fn).
- `poll_until_pre_returns_without_ticking_when_pre_satisfied_headless` ✓ (REQ-PROBE-FIRST — the strengthened
  `advanced == Duration::ZERO` mock-clock assertion holds: probe-first does not tick)
- `poll_until_pre_respects_a_finite_ceiling_headless` ✓ (REQ-FINITE-CEILING)
- `syntax_async_large_file_lands_off_thread_headless` ✓ (REQ-LAG-PRESERVED — the standalone pre-tick `assert_ne!`
  between the two rewired polls still passes)
- `t349_bracket`/`t349_edit`/`ladder_reads`/`sticky_headers_read`/`ladder_falls_back` ✓ (REQ-ALL-SITES-REWIRED)

**2. Regression** (`cargo nextest -p marley`): **748 passed, 2 skipped, 0 failed.** The #334 Shape-1 `poll_until`
sites + `poll_until_respects_a_finite_ceiling_headless` + every other headless drive stay green (REQ-334-UNCHANGED —
`poll_until` untouched). The known `search_open…` flake did not fire.

**3. NO LIVE SYNTHETIC DRIVE — stated.** #364 is TEST-HARNESS-ONLY: the diff is a `#[cfg(test)]` poll helper
(`poll_until_pre`) + 7 mechanical loop→helper rewires + 2 units. No app / render / input / shim path is touched (no
product behavior changes at all — the rewires only replace a fixed 100-iter budget with a 30s deadline). The 7
headless drives that RAN exercise the real reroute end-to-end; a `drive.swift` run would show nothing (no UI
delta) AND is off-limits (chad may be at the machine). The headless drives are the complete proof.

**4. THE FULL `--diff` GATE → GATE GREEN [diff], 15/15, first run.** Receipt
`df1ef410d4252ede279d7cc79cd346d715c61da5`. All 15 PASS incl.:
- **gate:4 coverage ≥ 100%** — `poll_until_pre` single-path fully exercised (pre-probe + loop by the 7 drives, the
  deadline exit by the finiteness unit, the pre-satisfied skip by the pre-satisfied unit); headless_drive.rs is
  cov-INCLUDED (not in the ignore-regex).
- **gate:5 mutation MSI 100%** — the diff is entirely `#[cfg(test)] mod headless_drive` → cargo-mutants (normal
  build) enumerates NO mutants in it (no `mutants::skip` needed, like `poll_until`); NO app/marley_syntax change →
  no product mutation surface added. No survivor.
- **gate:14 docs** — the plain-backtick `poll_until_pre` rustdoc + the `[`poll_until`]` intra-doc link resolve.
- **gate:6 miri / gate:15 visual** — green (no product change).

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21):** CHANGELOG.md `### Fixed` entry (the 7 Shape-2 polls no longer flake the gate; `poll_until_pre`
probe-first sibling) + docs/marley_architecture/marley_visual_harness.md — UPDATED the #334 "third lane rule"
paragraph: its closing "the Shape-2 polls are a separate follow-up — they can't reuse `poll_until` as-is" now
records #364's `poll_until_pre` sibling (the two poll helpers = body-first `poll_until` for real-PTY spawns +
probe-first `poll_until_pre` for worker-parse landings). Both DOCS-only → no receipt stale.

**Capture (forge wired):** aar-submit `b05249af` — outcome completed, effectiveness 4 (13 verdicts, 1 novel
finding, distillation/confidence-drift/pattern-emergence enqueued). HEADLINE lessons:
- ★ The #334 sibling transplant held — a probe-first twin + a mechanical 7-site rewire; recon confirmed
  `#[cfg(test)]` → no mutation surface + single-path cov BEFORE writing code.
- ★★ The [MEDIUM] catch (Critic 2): the pre-satisfied unit's `calls == 1` did NOT discriminate probe-first from
  body-first (both reach `calls == 1`; only the TICK differs — a call count can NEVER prove an ordering, only the
  gated SIDE EFFECT can). The test passed against the very helper #364 replaces. FIX: assert `executor().now()`
  mock-clock delta == 0 (probe-first = 0; body-first = +40ms). → `PR-claude-ordering-invariant-not-provable-by-
  call-count-001` (`5ab1707a`) — same family as #362's vacuous-drive + #363's tautological-equivalence.
- ★ The dropped `T: Default` bound — probe-first seeds `last` from the first probe (never `T::default()`), so the
  bound is unused; keeping it would have forced `BufferVersion: Default` for `syntax_async`'s observed tuple (a
  compile risk). A design refinement discovered at implement, documented in the fn's rustdoc.
- ★ The pre-flight SCOPE-DELTA discovery: the ticket named 4 sites (pre-#363); a grep found 9 raw `for _ in 0..100`
  → 7 real Shape-2 polls (the 4 + #363's 3) + 2 FALSE POSITIVES (the #360 add-cursor ACTION loops — dispatch ×100,
  no probe/sleep). "Grep, don't trust the stale count" — the pre-flight caught both the under-count and the false
  positives.
- ★ The LAG-is-standalone finding (Critic 1 traced `simulate_keystrokes` to gpui source: `run_until_parked` only,
  no `advance_clock`) dissolved the ticket's "probe-first is load-bearing for the LAG" framing — the LAG is a
  direct read INDEPENDENT of the loop's ordering; probe-first is the faithful zero-behavior-change transplant, not
  a LAG requirement. (A locked assumption dying to evidence — recorded, not hidden.)

**failure-record:** NONE — F1 caught + fixed in-phase (gate never red). **architecture-decision-record:** none — an
application of the #334 poll-helper pattern. **follow-up ticket:** none — this is the LAST ticket of the goal.

**Close + archive:** forge #364 closed (`ticket-close` → done); TICKET-364 → `tickets/closed/`; the pipeline pair →
`pipeline/completed/`; spec Phase 5 PASS.

**Status: Phase 5 — Complete PASS.**
