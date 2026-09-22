# TICKET-334 — `new_tab_inherits_live_cwd_headless` is load-flaky (a 5s wall-clock bound on a real PTY spawn)

- **Forge ticket:** #334 `188dfbaa-dedf-4894-b950-221d6b7f9338` (bug, test-flake/m21-followup/331-adjacent/gate)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `5010ba35-7110-459f-a1bd-300b9c8f0c9c`
- **Pipeline doc:** ../../pipeline/active/334-headless-poll-load-flake.spec.md
- **Source ticket:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #331-adjacent test-flake, surfaced in #331 Validate)
- **Status:** closed

## Summary
`new_tab_inherits_live_cwd_headless` (headless_drive.rs) polls a REAL PTY for its shell-integration `pwd`
report with a FIXED 200-iteration budget (`while polls < 200 { sleep(25ms); advance_clock(40ms);
run_until_parked() }`) — a hard ~5-second wall-clock ceiling. On an idle box the PTY handshake completes in
~0.1s, but under load (box at ~8.2, and `cargo mutants` drives `cargo test`, so ~599 lib tests run as parallel
threads in one process — several spawning their own PTYs — with no nextest process isolation) it can exceed 5s
and the poll gives up with `last saw None` (the shell never reached its integration handshake). Because the
mutation gate treats a failed baseline as exit 4 = "not a valid measurement" and fails CLOSED, this flake can
BLOCK the commit gate non-deterministically — most likely exactly when the machine is busy — and will bite CI on
a shared runner. Both polls in the test (the `reported` cd-tracking poll and the `inherited` new-tab poll) have
the identical shape and exposure. Fix: replace the fixed-iteration budget with an event-driven
poll-until-deadline (a generous FINITE ceiling) so a slow box waits LONGER instead of failing.

## Acceptance
Under load / no per-test isolation, the test's polls wait long enough for the real PTY to report `pwd` rather
than failing the baseline (event-driven generous ceiling, not a fixed small bound); the `last_seen` diagnostic
is preserved; both polls are hardened. Test-harness-only (no app behavior change).
