# Harden new_tab_inherits_live_cwd_headless (poll-until-deadline) — Notes

- **Forge ticket:** #334 `188dfbaa-dedf-4894-b950-221d6b7f9338`
- **AAR:** `5010ba35-7110-459f-a1bd-300b9c8f0c9c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-334-headless-poll-load-flake.md
- **Pipeline spec:** 334-headless-poll-load-flake.spec.md
- **pipeline_id:** 9509166f-d7c8-443d-b3db-0a97263876ef
- **Live on:** `506218d` (after #359; TENTH + FINAL of the goal /work 341…334)

## Phase 1 — Plan
- **Request:** Fix the load-flaky `new_tab_inherits_live_cwd_headless` — a fixed 200-iteration (~5s wall-clock)
  poll budget on a real PTY spawn that fails the `cargo mutants` baseline under load (exit 4 → fails the commit
  gate CLOSED). Ticket options: (a) event-driven generous ceiling, (b) env-knob budget, (c) warm-shell gate.
- **Classification / tier:** work pipeline; a test-flake (gate-blocking) bug; test-harness-only (no app code).
- **Forge recall (§18.3):** bulletins none. `aar-open` → `5010ba35`. `knowledge-context` (Plan) logged 13
  surfacings — the poll/timing/mock-clock PR family (`9605e19b`, `004a7a9f`, `fcefaae2`, `340b7a13`, `a7199501`)
  + timing ADs (`cf751352`, `9a5e768a`, `503b0a80`) + recent failures (`2a776e48`, `dd799805`). The governing
  lesson is the #349/#359 mock-clock pump idiom — HONORED here (the loop already advances the clock; only the
  ceiling is wrong).
- **★ Recon (on `506218d`) — four decisive findings:**
  1. **The test + its two polls (headless_drive.rs:2032).** `reported` poll :2083 (`while polls < 200 && !reported`)
     tracks the live prompt pwd after a real `cd`; `inherited` poll :2102 (`while polls2 < 200 && !inherited`)
     waits for the ⌘T new tab's PTY to spawn in the inherited cwd, asserting with `last saw {last_seen:?}`. Each
     body = `sleep(25ms) + advance_clock(40ms) + run_until_parked()`; 200 × 25ms = the 5s wall-clock budget. The
     loop ALREADY does real-sleep + advance_clock + park (the #349 idiom) — the defect is purely the fixed bound.
  2. **★ NO env-knob idiom exists (ticket option b's premise is false).** grep of headless_drive.rs for
     `env::var|MARLEY_*|"CI"|BUDGET|poll_until|deadline|nextest` → 0 (only one unrelated doc-comment). There is
     nothing "the other timing-sensitive drives" do to reuse. → option (b) would INTRODUCE the first such knob and
     couple to detecting the runner (nextest vs cargo test) — fragile. Event-driven (a) is chosen (D1).
  3. **★ `#[cfg(test)] mod headless_drive`** (lib.rs:55) → this is a "gate-is-test" change (D3): cargo-mutants
     skips `#[cfg(test)]` code (no MSI surface); the mutation gate `--in-diff` on a test-only crate diff finds no
     mutable lines → PASSES (gates.sh:239/254; the no-viable-mutants-fails-closed hardening is FULL-mode only);
     llvm-cov counts headless_drive.rs by execution (NOT in gates.sh:217's ignore-regex) → a single-path helper
     stays cov 100. So the plan's "extract a pure helper for cov/MSI-100" is DISSOLVED by the facts (a #339-class
     dissolution) — manufacturing a pure env-budget fn would add an uncovered branch + runner coupling for zero
     MSI benefit. The honest validation is the finiteness unit + the passing test + the gate.
  4. **★ The flake CLASS = 5 tests (D5).** The identical `while _ < 200` real-sleep poll recurs in
     `new_tab_inherits_live_cwd_headless` (2 polls, real-PTY — in scope), `mouse_modes_track_decset_headless`
     (:2166, real-PTY DECSET), `syntax_async_large_file_lands_off_thread_headless` (:1922,:1948, worker-thread),
     and 2× `t349_*` (:6456, :6517, worker-thread). A shared deadline helper is their natural home; rewiring all
     kills the whole gate-blocking class. Breadth (all-5 vs ticket-only-1) ratified at design.
- **Decisions:** D1 event-driven poll-until-deadline (not env-knob); D2 a shared `poll_until` helper (generous
  FINITE `POLL_CEILING`, per-iteration body verbatim); D3 gate-is-test (no manufactured app seam; cov-by-execution
  + mutants-skip-cfg-test + diff-no-mutable-lines); D4 preserve the `last_seen` diagnostic; D5 rewire the whole
  same-class set (design ratifies breadth).
- **Prior art:** gpui test-support owns `run_until_parked` + `advance_clock` but NOT a real-IO deadline poll; `std`
  `Instant` owns the deadline → the thin helper is ours. Our own fixed-iter idiom (5 sites) is what this factors.
- **§20:** N/A (Marley's own headless harness timing; no reference-app behavior).
- **EARS:** REQ-SLOW-BOX-WAITS, REQ-FINITE-CEILING, REQ-KEEP-DIAGNOSTIC, REQ-BOTH-POLLS (see spec).

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** Entirely within `#[cfg(test)] headless_drive.rs` (M16 #264 driven-test lane). A
shared, deadline-bounded poll helper replaces the fixed 200-iteration budget; the per-iteration body is the
existing `tick_pump` (:89 = `advance_clock(40ms) + run_until_parked()`) preceded by the `sleep(25ms)` — verbatim.
No app-crate (`src/**` non-test) touch → §14 typed-errors/no-panic-on-input rules N/A (test harness); §20 N/A
(Marley's own harness timing; confirmed — no reference-app behavior).

**★ Recon settled the shapes (two, NOT one).** All 5 sites read:
- **Shape 1** (`while <var> < 200 && !flag { <var>+=1; sleep(25); advance(40); park; flag = probe() }` —
  body-THEN-probe, 200 iters = 5s, REAL-PTY): `new_tab_inherits_live_cwd_headless` :2083 (`reported`) + :2102
  (`inherited`), and `mouse_modes_track_decset_headless` :2164 (`on`). The bodies are byte-identical (`sleep(25);
  tick_pump`). ⇒ these map onto the helper directly.
- **Shape 2** (`for _ in 0..100 { if probe() { flag=true; break; } sleep(25); advance(40); park; }` — probe-FIRST
  then break, 100 iters = 2.5s, WORKER-THREAD): `syntax_async_large_file_lands_off_thread_headless` :1916 + :1943,
  `t349_bracket_match…` :6451, `t349_edit_invalidates…` :6512. DIFFERENT structure (probe-first), a smaller
  budget, and they wait on a background PARSE, not a PTY spawn. `syntax_async`'s probe-first is load-bearing (its
  "the cache LAGS immediately after the keystroke" assertion depends on checking BEFORE the first tick). ⇒ NOT
  byte-identical; forcing them through this helper would change their semantics.

**★ D5 breadth — FINAL.** Rewire the Shape-1 REAL-PTY set now (the ticket's test + `mouse_modes`, same 5s-budget
body-then-probe flake, same file, mechanical): **3 polls across 2 tests.** DEFER the Shape-2 worker-thread set to
a follow-up ticket (a distinct class — probe-first, 2.5s, parse-not-PTY; syntax_async's structure is load-bearing).
This honors "rewire what matches byte-identically; anything that differs → follow-up." (Filed at Phase 5.)

**★ The helper (D1/D2/D3).**
```
const POLL_CEILING: Duration = Duration::from_secs(30);   // generous + FINITE; healthy box still exits ~0.1s
fn poll_until<T: Default>(
    vcx: &mut VisualTestContext,
    ceiling: Duration,
    mut probe: impl FnMut(&mut VisualTestContext) -> (bool, T),
) -> (bool, T) {
    let deadline = Instant::now() + ceiling;
    let mut last = (false, T::default());
    while !last.0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(25));
        tick_pump(vcx);
        last = probe(vcx);
    }
    last
}
```
- **SINGLE-PATH RETURN (cov 100):** no early `return`, no timeout tail — every line runs on the success path of a
  passing drive; on timeout the SAME lines return `(false, …)`. llvm-cov (headless_drive.rs is cov-by-execution,
  NOT in gates.sh:217's ignore-regex) sees 100%. The deadline-exceeded outcome is exercised by the finiteness unit.
- **GENERIC `T: Default`** so a probe can observe a `pwd` (`Option<String>`, for the `last_seen` diagnostic) OR
  nothing (`()`, mouse_modes). `T::default()` = `None` / `()`. (Non-generic `-> (bool, Option<String>)` is an
  acceptable simpler fallback if inference frictions; either is single-path → cov 100.)
- **REUSES `tick_pump`** (the #349/#359 mock-clock idiom — the loop advances the clock, honored not fought). The
  defect was ONLY the fixed ceiling.
- **`POLL_CEILING = 30s`**: 6× the old 5s → ample for a ~8-load box's PTY spawn+rc+shell-integration handshake;
  event-driven so a fast box is unaffected; FINITE so a genuinely-broken shell fails bounded, not hangs the suite.

**★ The rewires.**
- `new_tab_inherits_live_cwd_headless` — both polls become, each with the SAME inline probe (the closure captures
  `window` [WindowHandle is Copy], `focused_pwd` [Fn], `is_sub` [Fn] by ref):
  `let (reported, _) = poll_until(&mut vcx, POLL_CEILING, |vcx| { let p = focused_pwd(&window, vcx); (is_sub(&p), p) });`
  `let (inherited, last_seen) = poll_until(&mut vcx, POLL_CEILING, |vcx| { let p = focused_pwd(&window, vcx); (is_sub(&p), p) });`
  The `inherited` assert is UNCHANGED (`… last saw {last_seen:?}` reads the returned observed value — D4 preserved).
  The `polls`/`polls2` counters + `let mut reported/inherited/last_seen` are removed.
- `mouse_modes_track_decset_headless` — `let (on, _) = poll_until(&mut vcx, POLL_CEILING, |vcx| { let m =
  modes(&window, vcx); (m.click && m.sgr, ()) });` (T = `()`). The two asserts below are unchanged.

**★ The finiteness unit (REQ-FINITE-CEILING).** `poll_until_respects_a_finite_ceiling_headless` — `boot` a trivial
project, then `let (met, seen): (bool, Option<String>) = poll_until(&mut vcx, Duration::from_millis(50), |_| (false,
None));` (the annotation resolves T); assert `!met` + `seen.is_none()`; `reap_sessions`. Proof of finiteness =
the test RETURNS (a hang → harness timeout → fail) AND `met == false` (it gave up rather than spin). ★ NO
wall-clock UPPER-bound elapsed assertion — that would reintroduce the exact load-flake class being removed; the
`!met` return IS the finiteness proof. The tiny 50ms ceiling / 25ms sleep ⇒ ~2 iterations then deadline → exits;
deterministic, no real PTY observed.

**File manifest (1 file):**
- `crates/marley_app/src/headless_drive.rs` — ADD `const POLL_CEILING` + `fn poll_until<T: Default>` near
  `tick_pump` (:89); REWIRE `new_tab_inherits_live_cwd_headless` (2 polls) + `mouse_modes_track_decset_headless`
  (1 poll); ADD the `poll_until_respects_a_finite_ceiling_headless` unit. No other file (CHANGELOG/doc at P5).

**Regression Test Plan (gate-is-test — D3; no new app cov/MSI surface):**
| # | Proves | Test |
|---|---|---|
| T1 | REQ-FINITE-CEILING | `poll_until_respects_a_finite_ceiling_headless` — always-unmet probe + 50ms ceiling → `(false, None)`; deterministic, no PTY. |
| T2 | REQ-SLOW-BOX-WAITS + REQ-KEEP-DIAGNOSTIC + REQ-BOTH-POLLS | `new_tab_inherits_live_cwd_headless` (rewired) still passes on a healthy box; both polls via the helper; `inherited` assert still prints `last saw {last_seen:?}`. |
| T3 | REQ-BOTH-POLLS (the ratified sibling) | `mouse_modes_track_decset_headless` (rewired) still passes. |
- **Uncoverable-honestly:** the flake itself is load/scheduling dependent → NOT deterministically reproducible in
  the gate. T2/T3 prove the rewire did not BREAK the tests; the finiteness unit + the event-driven structure prove
  the HARDENING. cov 100 via single-path + execution (D3); mutation gate passes via test-only-diff-no-mutable-lines.

**Risks / decisions.** (a) probe closure borrows — `window` is `Copy` (capture by value/ref), `focused_pwd`/`is_sub`/
`modes` captured by ref, only `vcx` is `&mut` (the arg) → no borrow conflict. (b) `POLL_CEILING` FINITE (30s) — no
suite hang. (c) single-path return → cov 100 (D3). (d) D5: only byte-identical Shape-1 real-PTY sites rewired;
Shape-2 worker-thread → follow-up (their probe-first is load-bearing). (e) helper is `#[cfg(test)]` → mutants-skip
(no MSI) + cov-by-execution. (f) the finiteness unit carries NO wall-clock upper bound (would re-add flake). §20 N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

**Built (1 file, `crates/marley_app/src/headless_drive.rs`), exactly to the manifest:**
- `const POLL_CEILING: Duration = Duration::from_secs(30)` + `fn poll_until<T: Default>(vcx, ceiling, probe) ->
  (bool, T)` inserted after `tick_pump` (:95). Single-path as designed (`let deadline = Instant::now() + ceiling;
  let mut last = (false, T::default()); while !last.0 && Instant::now() < deadline { sleep(25); tick_pump(vcx);
  last = probe(vcx); } last`). Reuses `tick_pump`. Plain-backtick docs (with `[`tick_pump`]` /
  `[`poll_until_respects_a_finite_ceiling_headless`]` intra-doc links — both resolve, same module).
- `new_tab_inherits_live_cwd_headless`: the `reported` + `inherited` polls each replaced by a `poll_until` call
  with the inline probe `|vcx| { let p = focused_pwd(&window, vcx); (is_sub(&p), p) }`. Both asserts UNCHANGED
  (the `inherited` one still prints `last saw {last_seen:?}` — `last_seen` is now the helper's returned observed).
  The `polls`/`polls2`/`let mut reported`/`let mut inherited`/`let mut last_seen: Option<String>` counters removed.
- `mouse_modes_track_decset_headless`: the `on` poll replaced by `poll_until(&mut vcx, POLL_CEILING, |vcx| { let m
  = modes(&window, vcx); (m.click && m.sgr, ()) })` (T = `()`). Both asserts below UNCHANGED. `polls`/`let mut on`
  removed.
- `poll_until_respects_a_finite_ceiling_headless` (new `#[gpui::test]`, inserted before the `// M17 #280`
  mouse_modes block): boot a trivial project, `poll_until(&mut vcx, Duration::from_millis(50), |_vcx| (false,
  None))` with the required `(bool, Option<String>)` annotation, assert `!met` + `seen.is_none()`, reap. NO
  wall-clock upper bound (would re-add the flake class).

**Borrow shape (confirmed compiling):** each probe closure captures `window` (WindowHandle is `Copy`) +
`focused_pwd`/`is_sub`/`modes` (by ref) and takes `vcx: &mut VisualTestContext` as its arg → no borrow conflict
with `poll_until`'s own `&mut vcx`. The two `new_tab` probes are separate inline closures (a closure can't be
reused across two `poll_until` calls) — identical bodies, intentional.

**Checks:** `cargo fmt --all` clean; `cargo check -p marley --all-targets` CLEAN (only the pre-existing `block
v0.1.6` future-incompat note, a transitive dep — not our code); `cargo clippy -p marley --all-targets -- -D
warnings` **exit 0** (no unused-var/`mut` from the counter removal). Diff = ONLY headless_drive.rs.

**Deviations from design:** none. D5 held (only the 3 Shape-1 real-PTY polls rewired; the Shape-2 worker polls
untouched → Phase-5 follow-up).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 critics (Agent general-purpose, parallel) over the diff + my own step-2 review. Verdict: CLEAN — no HIGH/MEDIUM;
4 LOW, all no-action. No source change.**

**Critic 1 — helper correctness / finiteness / single-path-cov. VERDICT: correct, finite, single-path.**
- (a) event-driven CONFIRMED (exits on met-probe OR 30s deadline); real hardening (30s > old 5s, not cosmetic);
  first-iter sleep→tick→probe ordering identical to the old loop; `tick_pump` preserved verbatim.
- (b) ★ FINITE CONFIRMED via gpui-0.2.2 source: `advance_clock` mutates only `dispatcher.state.time` (the MOCK
  clock; `dispatcher.rs:71`, `executor.rs:393`) — it has NO path to `std::time::Instant` (the real OS monotonic
  clock the deadline reads), and the unconditional real `sleep(25ms)` advances real time every pass → after ≤30s
  the loop MUST exit. (My own check reached the identical conclusion from the same source.)
- (c) single-path CONFIRMED (no early return/timeout tail; every line runs on both paths; `T: Default` holds for
  `Option<String>`=None + `()`=()); the passing drives cover the lines, the finiteness unit covers the
  deadline-FALSE outcome.
- (d) finiteness unit rigorous (annotation needed+present; `!met`+`is_none`; no wall-clock upper bound = correct;
  boots+reaps).
- LOW-1 (deadline-at-top bounds iteration count, not an intra-iteration hang) — inherent to any deadline-poll +
  identical to the old loop's structure; `run_until_parked`/probe are bounded; NO CHANGE. LOW-2 (`!met` is a
  near-tautology; the real proof is the harness returning) — already stated in the :2147-2150 comment; NO CHANGE.

**Critic 2 — rewires behavior-preserving / scope / provenance. VERDICT: ship.**
- (a) all 3 rewires byte-identical predicates + byte-identical asserts; ★ `last_seen` binds to the helper's
  returned observed (`.1` = the last probe's `p`; since the 30s deadline passes at entry the body always runs ≥1
  iteration → never the `None` default) → D4 diagnostic fidelity preserved. Per-iteration body byte-equivalent
  (`tick_pump` == the old inline advance+park); the ONLY change is the ceiling.
- (b) probes re-read live state each iteration (accessor called INSIDE the closure); `window` is `Copy`
  (`window.rs:4761`); no borrow conflict (only `vcx` is `&mut`, supplied by `poll_until`).
- (c) scope tight — exactly 5 diff hunks (helper + 3 rewires + finiteness unit); the 4 Shape-2 worker polls
  (:1952/:1978/:6487/:6548) UNTOUCHED; no `while _ < 200`/`polls2`/stray `advance_clock` remain in the 2 rewired
  tests (REQ-BOTH-POLLS holds).
- (d) provenance clean — only `#[cfg(test)] headless_drive.rs` (lib.rs:55) → zero app surface; ★ gate-14 runs
  rustdoc WITHOUT `--cfg test` (`gates.sh:162-163`) so the whole cfg(test) module + its intra-doc links are
  configured OUT of the doc build → no rustdoc-warning path at all; `git status` = the 1 src + 3 docs; no
  Zed/Warp, no secrets, no debug prints/TODO.
- LOW-1 (timeout-path iteration count differs) — semantically equivalent (both = "last value observed"), not a
  defect. LOW-2 (`mutants::skip` insertion-trap) — N/A (no adjacent skipped shim; whole module is cfg(test) →
  mutants-skip wholesale).

**My own step-2 review (independent):** confirmed finiteness via gpui source (clocks independent); read all 3
rewires + the finiteness unit in the live file (predicates + asserts preserved, `last_seen` bound to observed);
grep-confirmed scope (0 `while _ < 200` remain, 4 `for _ in 0..100` worker polls untouched, exactly 4 `poll_until`
sites). Agrees with both critics.

**Findings acted on:** none — no HIGH/MEDIUM from either critic or my own review; all 4 LOWs are honest-boundary
observations explicitly requiring no change. `failure-record`: none (the FLAKE this ticket FIXES was the
pre-existing #334 bug itself; the fix introduced no new defect). No source edit at Inspect.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests RUN (the Phase-3 tests; Phase 4 runs them):**
- T1/T2/T3 targeted: `cargo nextest run -p marley -E 'test(poll_until_respects_a_finite_ceiling) +
  test(new_tab_inherits_live_cwd) + test(mouse_modes_track_decset)'` → **3 passed, 732 skipped** —
  `mouse_modes_track_decset_headless` 0.099s, `poll_until_respects_a_finite_ceiling_headless` 0.124s,
  `new_tab_inherits_live_cwd_headless` 0.148s. ★ T1 returns in 0.124s (NOT 30s) — the deadline-exit fires with the
  50ms ceiling, proving finiteness (returns not-met, no hang). T2/T3 pass — the rewired polls exit after the first
  met probe on this healthy box; the `last saw {last_seen:?}` diagnostic is intact.
- App regression: `cargo nextest run -p marley` → **733 passed, 2 skipped** (was 732 for #359 → +1, the new
  finiteness unit; the 2 rewired tests were already counted). No regression; the `search_open…` flake did not
  recur; the untouched Shape-2 `syntax_async_large_file_lands_off_thread_headless` still passes (1.187s).

**Live drive:** NONE — stated deliberately. The proof is the headless `cargo nextest` run through the REAL tests
(real PTYs, the real `poll_until`), which is the exact subject of #334; a live synthetic drive is not needed and
is off-limits (chad may be at the machine). No UI/render/input surface changed (test-harness only).

**Full `--diff` gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15 PASS**, receipt
`60b157b184366cdcb7eed776100e68feea91aa37`. gate:3 tests green (incl. T1/T2/T3); gate:4 coverage ≥100% lines
(headless_drive.rs cov-by-execution — the single-path helper's lines all run in the 3 tests); gate:5 mutation MSI
≥100% (the cfg(test)-only diff has no mutable crate lines → passes per gates.sh:239/254); gate:14 docs PASS
(rustdoc runs without `--cfg test` → the cfg(test) module + its intra-doc links are configured out); gate:15
visual/AX PASS. No red, no pre-existing exclusions. The mutation-baseline (the very scenario #334 fixes) ran clean
— the 30s ceiling held.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete

**Docs (§21).**
- **CHANGELOG.md** — a `### Fixed` entry: the load-flaky headless test no longer blocks the commit gate (the
  fixed 200-iter/~5s budget → exit-4-fails-closed under load; the event-driven `poll_until` 30s FINITE ceiling
  fix; test-harness-only; the Shape-2 follow-up).
- **docs/marley_architecture/marley_visual_harness.md** — a "third lane rule (M21 #334)" in the #264 headless-lane
  paragraph: real-subprocess waits poll event-driven to a generous FINITE deadline (`poll_until`, `POLL_CEILING =
  30s`, independent of the mock clock), reusing `tick_pump` as the per-iteration body; the Shape-2 worker polls
  are a separate follow-up.

**Capture (forge wired).**
- `aar-submit` 5010ba35, outcome completed, effectiveness 4 (1 novel, 13 verdicts). Headline lessons: (a) ★ the
  ticket's option (b) ("scale off an env knob the way the other timing drives do") RESTED ON A FALSE PREMISE —
  recon grep=0 found NO such idiom; option (a) event-driven is the honest + more robust fix (a #339-class "verify
  the ticket's confident sentence before building on it"). (b) ★ a flake fix in `#[cfg(test)]` code is a
  GATE-IS-TEST change — mutants skips it, the `--in-diff` mutation gate passes on no-mutable-lines, llvm-cov counts
  it by execution → the plan's "extract a pure helper for cov/MSI" DISSOLVED (would add an uncovered branch +
  runner coupling). (c) ★ prove finiteness by the helper RETURNING not-met (always-unmet probe + tiny ceiling),
  NEVER by a wall-clock upper-bound assertion (that re-adds the flake). (d) ★ finiteness rests on TWO INDEPENDENT
  clocks — `std::time::Instant` (real OS) vs gpui's mock `advance_clock` (`dispatcher.state.time`), verified in
  gpui-0.2.2 dispatcher.rs:71 / executor.rs:393. (e) ★ TWO SHAPES, not one — the flake class spans 5 tests in 2
  shapes; reading each sibling body before swapping kept `syntax_async`'s load-bearing probe-first ordering intact
  (deferred to #364). (f) single-path return → cov 100 (both critics + my own review: unanimous ship, 0
  HIGH/MEDIUM).
- `prevention-rule-record` **PR-claude-event-driven-deadline-poll-not-fixed-budget-001** (df6ae107): a headless
  test waiting on a real subprocess/worker must poll event-driven to a generous FINITE deadline (not a fixed
  iteration budget → exit-4-fails-closed under load); prove finiteness by returning not-met, never by a wall-clock
  upper bound; keep the helper single-path for cov; verify a "reuse the existing idiom" claim before building on it.
- `failure-record`: none SHIPPED — the flake was the pre-existing #334 bug this ticket FIXES; inspect found 0
  HIGH/MEDIUM (4 no-action LOWs).
- **Follow-up: forge #364** (`fcd70d73-5015-4c34-bae5-19d150a4d5a8`, test-flake/gate/334-followup) — harden the
  Shape-2 worker-thread polls (`syntax_async_large_file` ×2, `t349_bracket`, `t349_edit`): same fixed-budget flake
  class, distinct probe-first `for _ in 0..100` shape (can't reuse `poll_until` as-is; syntax_async's probe-first
  is load-bearing).

**Close + archive.** forge #334 → done; TICKET-334 → closed/ (`status: closed`); the spec/notes pair archived
active/ → completed/; spec `status: Phase 5 — Complete PASS`.

**Status: Phase 5 — Complete PASS.** Run `/commit` to deliver (LOCAL — push un-OK'd). ★ FINAL ticket of the goal
/work 341…334 (10/10).
