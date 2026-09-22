---
pipeline_id: 9509166f-d7c8-443d-b3db-0a97263876ef
ticket: forge#334 (188dfbaa-dedf-4894-b950-221d6b7f9338) · local docs/planning/tickets/open/TICKET-334-headless-poll-load-flake.md
aar_id: 5010ba35-7110-459f-a1bd-300b9c8f0c9c
status: Phase 5 — Complete PASS
title: Harden new_tab_inherits_live_cwd_headless — event-driven poll-until-deadline replaces the fixed 5s wall-clock budget on a real PTY spawn
type: bug
milestone: M21
references: [headless_drive.rs:2032 new_tab_inherits_live_cwd_headless / :2083 the `reported` cd-poll (200 iters) / :2102 the `inherited` new-tab poll (200 iters, asserts `last_seen`) / :2085,:2104 sleep(25ms)+advance_clock(40ms)+run_until_parked, lib.rs:55 `#[cfg(test)] mod headless_drive`, gates.sh:235-243 mutation --in-diff (empty/no-mutable-lines → pass) / :217 cov --ignore-filename-regex (headless_drive NOT excluded → cov-by-execution), the same-class sibling polls :2166 mouse_modes_track_decset_headless (real-PTY DECSET) + :1922/:1948 syntax_async_large_file + :6456/:6517 t349_* (worker-thread), #331 (surfaced-in-validate), #349/#359 mock-clock pump lesson]
---

## Title
`new_tab_inherits_live_cwd_headless` polls a REAL PTY for its shell-integration `pwd` with a FIXED 200-iteration
(~5s wall-clock) budget. Under load the handshake exceeds 5s → the poll gives up with `last saw None` → the
`cargo mutants` baseline fails → exit 4 (unmeasurable) fails the commit gate CLOSED. Replace the fixed budget
with an event-driven poll-until-deadline helper (a generous FINITE ceiling) so a slow box waits LONGER instead
of failing. Test-harness-only; no app behavior change.

## Scope
### In
- A shared `poll_until`-style helper in `#[cfg(test)] headless_drive.rs`: loop `sleep(25ms) + advance_clock(40ms)
  + run_until_parked()` while a caller predicate is unmet AND `Instant::now() < deadline`, where `deadline =
  Instant::now() + POLL_CEILING` (a generous FINITE const, e.g. 30–60s — design picks). Returns whether the
  predicate was met (and the last observed value for the diagnostic). The per-iteration body is UNCHANGED — only
  the CEILING changes from a fixed small iter-count to a generous deadline.
- Rewire BOTH polls in `new_tab_inherits_live_cwd_headless` (`reported` :2083, `inherited` :2102) through the
  helper, preserving the exact asserts — especially the `inherited` assert's `last saw {last_seen:?}` message.
- A deterministic finiteness unit: the helper with an always-unmet predicate + a TINY ceiling returns "not met"
  within a bounded time (proves the ceiling is finite / cannot hang forever; also exercises the deadline-exceeded
  outcome for region coverage).

### In (ratified at design — the byte-identical Shape-1 real-PTY sibling)
- `mouse_modes_track_decset_headless` (:2164, `on`) — the SAME shape as the ticket's polls (`while <var> < 200 &&
  !flag`, body-THEN-probe, 200-iter/5s, a REAL-PTY wait), byte-identical body → rewired through the same helper. It
  is the identical gate-blocking flake in the same file; NOT silent widening (design D5 confirmed the body matches).

### Out (explicitly deferred)
- **The Shape-2 worker-thread polls** (`syntax_async_large_file_lands_off_thread_headless` :1916/:1943,
  `t349_bracket_match…` :6451, `t349_edit_invalidates…` :6512) — a DISTINCT class: `for _ in 0..100`, probe-FIRST
  then break, 100-iter/2.5s, waiting on a background PARSE not a PTY. `syntax_async`'s probe-first ordering is
  load-bearing (its "cache LAGS immediately after the keystroke" assertion checks BEFORE the first tick), so they
  are NOT byte-identical to the helper and are not forced through it. → a Phase-5 follow-up ticket.
- Any change to APP code (`src/**` non-test). This is entirely `#[cfg(test)] headless_drive.rs`.
- Any change to the ASSERTIONS' meaning (the test still proves the same behavior — only the WAIT hardens).
- Reproducing the flake deterministically in the gate (impossible — it is load/scheduling dependent; the
  finiteness unit + the still-passing test + the `--diff` gate are the proof).
- The LIVE self-test harness (CGEvents/screencapture) — off-limits (chad may be at the machine) + irrelevant here.
- An env-knob / CI-detection budget (ticket option b) — dissolved: no such idiom exists to reuse (see D3), and
  event-driven is strictly more robust (it waits on the actual event regardless of WHY the box is slow).

## Reference (§20)
N/A — Marley-specific. This is Marley's own headless driven-test harness (the gpui `test-support` lane, M16
#264) and its real-PTY timing, with no Warp/Zed product-behavior analog. No copyleft source read.

### Prior art
- **Our permissive deps (highest-yield leg):** checked gpui `test-support` — it owns `run_until_parked` +
  `executor().advance_clock` (both already used here), but NOT a real-wall-clock deadline poll for EXTERNAL IO: a
  real PTY is a real subprocess off gpui's mock clock, so "wait until a real subprocess reports within a deadline"
  is not a gpui primitive. `std` owns the deadline (`std::time::Instant` — adopt it for `now < deadline`). No crate
  we ship owns a "retry-a-predicate-until-deadline while pumping the mock clock" seam → the helper is ours (thin).
- **Our own code:** the fixed-iter poll idiom (`while polls < 200 { sleep; advance_clock; run_until_parked }`)
  recurs in 5 headless tests — this ticket factors the shared, deadline-bounded version. The #349/#359 mock-clock
  lesson is HONORED, not fought: the loop ALREADY advances the clock + parks (that is why the pwd lands at all);
  the defect is purely the CEILING (a fixed small budget), not a vacuous `run_until_parked`.
- **Behavior maps / published material:** N/A — a standard "poll-until-condition-or-timeout" testing pattern; no
  reference-app behavior involved.

## Locked-In Decisions
- **D1 — APPROACH = event-driven poll-until-deadline (ticket option a), NOT an env-knob budget (option b).** The
  loop runs until the predicate is met OR a generous FINITE wall-clock deadline elapses. A fast box exits in
  ~0.1s (unchanged); a slow/loaded box waits LONGER (up to the ceiling) instead of failing at a fixed 5s. This is
  robust to ANY slowness cause (load, no per-test isolation, cold shell) — it waits on the actual event.
- **D2 — a shared helper in `#[cfg(test)] headless_drive.rs`** (name/signature at design — likely
  `poll_until(vcx, window, ceiling, predicate) -> (bool, Option<String>)` or a closure `FnMut(...) -> (bool,
  Option<String>)`). The per-iteration body (`sleep(25ms) + advance_clock(40ms) + run_until_parked`) is verbatim;
  only the bound becomes `Instant::now() < deadline`. `POLL_CEILING` a generous FINITE const (design picks
  ~30–60s, commented — big enough that a loaded box finishes; finite so a genuinely-broken shell fails bounded,
  not hangs the suite).
- **D3 — this is a "gate-is-test" change; NO app-crate cov/MSI seam is manufactured (the plan's pure-helper
  hypothesis is DISSOLVED by the facts).** `mod headless_drive` is `#[cfg(test)]` (lib.rs:55): (i) cargo-mutants
  SKIPS `#[cfg(test)]` code → no MSI surface, no surviving-mutant risk; (ii) the mutation gate's `--in-diff` on a
  test-only crate diff finds no mutable lines → PASSES (gates.sh:239/254 — the "no viable mutants fails closed"
  hardening is FULL-mode only); (iii) llvm-cov counts headless_drive.rs by EXECUTION (it is NOT in gates.sh:217's
  ignore-regex), and a single-path-return helper has every line run by the real test → cov stays 100. Extracting a
  pure env-budget fn would add an uncovered branch AND couple to detecting the runner — a net negative. Validation
  = the finiteness unit + the still-passing test + the `--diff` gate, per §7's gate-is-test path.
- **D4 — the `last_seen` diagnostic is preserved verbatim** (the `inherited` assert's `wanted {sub:?}, last saw
  {last_seen:?}` made the original diagnosis quick — the helper returns the last observed value so the assert is
  unchanged).
- **D5 — the flake CLASS is broader than the one test (5 sites); D4-breadth ratified at design.** Recommend
  rewiring all identical-shape real-sleep polls in the file (they block the same baseline); the ticket's test is
  the required minimum.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-SLOW-BOX-WAITS | WHEN the box is under load / running under `cargo test` (no per-test process isolation), the `new_tab_inherits_live_cwd_headless` polls shall wait for the real PTY to report `pwd` up to a generous FINITE deadline rather than failing at a fixed small bound. | The polls route through the deadline helper (review the diff); the test still passes on a healthy box (headless run) with no behavior/assert change; the ceiling is a generous FINITE const (not a fixed 200-iter budget). |
| REQ-FINITE-CEILING | WHEN the awaited event never occurs, the poll helper shall return "not met" within a bounded time (it shall NOT hang indefinitely). | A deterministic unit: the helper with an always-unmet predicate + a tiny ceiling returns `(false, …)` within the bounded time (asserts finiteness + exercises the deadline-exceeded outcome). |
| REQ-KEEP-DIAGNOSTIC | The `inherited` assertion shall still print the last observed pwd (`last saw {last_seen:?}`) on failure. | Review the assert is byte-preserved; the helper returns the last observed value. |
| REQ-BOTH-POLLS | BOTH polls in the test (`reported` cd-tracking and `inherited` new-tab) — and, per the ratified breadth, any same-class sibling — shall use the hardened deadline wait, not just one. | Grep the file: no `while <var> < 200` real-sleep poll remains in the rewired tests; each routes through the helper. |

## Phase Plan
- **P2 Design** — ratify D1 (event-driven, not env-knob) + the helper signature + `POLL_CEILING` value + D5
  breadth (all-5 vs ticket-only) + the finiteness unit shape + the test plan (finiteness unit + the still-passing
  headless run). Confirm each sibling's per-iteration body matches before planning its swap.
- **P3 Implement** — the `poll_until` helper + rewire `new_tab_inherits_live_cwd_headless`'s two polls (+ ratified
  siblings); the finiteness unit. `cargo check -p marley --all-targets`.
- **P3.5 Inspect** — ★ the budget is genuinely event-driven + generous (a slow box waits longer, not cosmetic);
  the ceiling is FINITE (cannot hang forever); BOTH polls (+ siblings) rewired; the `last_seen` assert intact; NO
  app-code change (test-harness only); the helper's single-path-return keeps cov 100 (no uncovered branch).
- **P4 Validate** — the finiteness unit + the headless RUN of `new_tab_inherits_live_cwd_headless` (+ regression:
  the full `-p marley` suite, incl. the rewired siblings) + the `--diff` gate (⚠️ the mutation gate RUNS the
  baseline — the fix hardens exactly THAT; a test-only diff clears gate:5 via no-mutable-lines). No LIVE drive.
- **P5 Complete** — CHANGELOG (`### Fixed`) + a headless-harness/testing doc note (the deadline-poll idiom + the
  #331→#334 lineage) + AAR capture + close + archive.
