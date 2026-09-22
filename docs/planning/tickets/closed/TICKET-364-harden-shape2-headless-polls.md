# TICKET-364 — Harden the Shape-2 worker-thread headless polls against load

- **Forge ticket:** #364 `fcd70d73-5015-4c34-bae5-19d150a4d5a8` (bug, test-flake/gate/334-followup)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `b05249af-f448-4b87-80dd-bba0c90f9f66`
- **Pipeline doc:** ../../pipeline/active/364-harden-shape2-headless-polls.spec.md
- **Source ticket:** the follow-up goal `/work 360,361,362,363,364` (the #334 follow-up)
- **Status:** closed

## Summary
#334 replaced the fixed-iteration real-PTY polls with an event-driven `poll_until` (a wall-clock deadline that
drives the mock-clock pump) + proved finiteness via `poll_until_respects_a_finite_ceiling_headless`. Seven MORE
headless polls share the same fixed-budget flake class but in a DISTINCT shape — Shape-2: a `for _ in 0..100 { if
probe() { break; } sleep(25ms); advance_clock(40ms); run_until_parked(); }` PROBE-FIRST loop waiting on an
off-thread PARSE landing (not a PTY spawn). Under load the ~2.5s budget can lapse before the parse lands → the
`cargo mutants` baseline fails closed (exit 4) → a non-deterministic commit-gate block. #364 adds a probe-first
deadline sibling `poll_until_pre` (mirroring `poll_until`, leaving it untouched) and rewires all 7 Shape-2 sites to
it — a generous finite ceiling, single-path for coverage, `#[cfg(test)]` so it enumerates no mutants.

## The 7 Shape-2 sites (grep-confirmed on `4f85856`, all in crates/marley_app/src/headless_drive.rs)
`syntax_async_large_file_lands_off_thread_headless` (two polls: the initial land + the post-keystroke catch-up),
`t349_bracket_match_reads_the_cached_tree_on_a_large_file_headless`,
`t349_edit_invalidates_the_cached_tree_and_falls_back_headless`, and the three #363 drives
`ladder_reads_the_cached_tree_on_a_hit_headless`, `sticky_headers_read_the_cached_tree_on_a_hit_headless`,
`ladder_falls_back_on_a_cache_miss_headless`. EXCLUDED (false positives): the `for _ in 0..100 {
dispatch_for_test("add-cursor-…") }` action loops in the #360 `add_cursor_below/above_follows_*_headless` drives —
deterministic dispatch loops with no probe/sleep/pump, not polls.

## Acceptance
A probe-first deadline helper evaluates the probe BEFORE the first pump tick (a pre-satisfied probe returns without
advancing the clock); it has a generous FINITE ceiling and gives up (not-met) rather than hanging; `syntax_async`'s
"cache lags immediately after the keystroke" assertion still holds (its standalone pre-tick read is preserved); ALL
7 Shape-2 polls use the helper (no raw probe-first `for _ in 0..100` remains); the #334 Shape-1 `poll_until` sites +
finiteness test stay green. Test-harness-only — the diff is headless_drive.rs alone.
