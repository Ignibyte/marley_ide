---
pipeline_id: 97d7778e-4fa2-412c-85de-012722981158
ticket: forge#348 (477c649a-a702-493b-94f2-9a2a516dbce8) · local docs/planning/tickets/open/TICKET-348-pty-teardown-bound.md
aar_id: 2a241d0b-0f77-4f1c-8076-d077336d88c0
status: Phase 5 — Complete PASS
title: Bounded PTY teardown + the nextest kill-switch — no test may ever hang a gate again
type: bug
milestone: M22
references: [the 11-hour zombie gate (ticket evidence — SLOW >39,240s at 0.0% CPU), OsPtyChannel "its Drop reaps the child" (pty_os.rs:18-19), TerminalSession::shutdown is EMPTY (session.rs:353), NO Drop/thread/waitpid anywhere in terminal_blocks src (grep-verified), the O_NONBLOCK io + PUMP_RETRY_BUDGET bounded-wait house idiom (session.rs write_bytes), .config/nextest.toml DOES NOT EXIST, cargo-nextest 0.9.138, #334 (the load-flaky PTY sibling), #345 (rides this ticket's nextest config)]
---

## Title
A gate started at 20:41 was still "running" 11 hours later, wedged on ONE test:
`marley_terminal::integration resize_real_pty_succeeds`, SLOW >39,240s at **0.0% CPU** under
`cargo llvm-cov nextest` — blocked, not slow — and nextest reported SLOW forever without ever killing it.
A hung gate that reports nothing is the worst failure mode: no red, no green, just silence. Two fixes,
one systemic and one at the source: a nextest terminate ceiling so NO test can ever stall a gate again,
and a bounded session teardown so the PTY lifecycle simply has no unbounded wait left in it.

## Scope
### In
- **The kill-switch (systemic floor):** create `.config/nextest.toml` —
  `[profile.default] slow-timeout = { period = "60s", terminate-after = 3 }` — SLOW warnings at 60 s,
  SIGTERM/SIGKILL at 180 s, and the terminated test FAILS the run. One knob covers every nextest lane:
  gate:3 (`cargo nextest run`, gates.sh:83), gate:4 (`cargo llvm-cov nextest`, gates.sh:216), gate:15
  (visual harness, gates.sh:323), and — once #345 lands — every `cargo mutants --test-tool=nextest` run.
- **The source fix (the actual bug):** the session lifecycle's ONLY unbounded wait. Recon @ f044546:
  `shutdown(self) {}` is empty (session.rs:353), `terminal_blocks` has **zero** `Drop` impls, threads,
  or `waitpid` calls; reads are `O_NONBLOCK`; writes are `PUMP_RETRY_BUDGET`-bounded. The reap is
  delegated: `OsPtyChannel` holds `tty::Pty` "so … its `Drop` reaps the child" (pty_os.rs:18-19) —
  i.e. **alacritty_terminal's `Pty::Drop`** (leading candidate: SIGHUP + a blocking `waitpid`; confirm
  by reading the vendored source at implement — a permissive dep, reading it is ADOPTION under §20).
  Replace the implicit blocking reap with an EXPLICIT bounded teardown in `shutdown`/pty_os:
  HUP → poll-reap (WNOHANG / `next_child_event`) under a ~2 s deadline → SIGKILL → poll-reap under a
  second ~2 s deadline → give up, log, and proceed (a leaked zombie is strictly cheaper than a hang;
  document it). By the time alacritty's `Drop` runs, the child is already dead + reaped, so any
  blocking wait inside it returns immediately.
- **The pure seam (cov/MSI 100):** the escalation/deadline logic as a pure step function in session.rs
  (e.g. `reap_step(state, now, child_status) -> ReapAction` — HUP-first, escalate-at-deadline,
  give-up-at-second-deadline), with injected clock + poll results, exactly the house
  pure-seam-plus-skip'd-shim shape. The pty_os shim (ACCEPTED-UNTESTABLE, mutants::skip, cov-excluded)
  only executes the returned action with real syscalls.
- The app inherits the fix for free: tab-close/quit can no longer hang Marley on a wedged child.
### Out (explicitly)
- Excluding real-PTY tests from coverage (§0 — the ticket forbids it; the tests stay, bounded).
- #334's 5 s load-flake bound (sibling ticket); #335's cargo-mutants temp trees; any change to
  `#[serial]` usage (nextest is process-per-test; the serial locks are per-process and uncontended);
  the gate's overall wall-clock budget.

## Reference (§20)
N/A — Marley-specific test-infra + process-lifecycle hygiene; no reference-app behavior to match.
Clean-room note: the one source we DO read is `alacritty_terminal`'s `tty` module (Apache/MIT — our
own dependency; adoption, explicitly outside the wall) to confirm what its `Pty::Drop` blocks on.

### Prior art
1. **Published material** — the nextest book documents `slow-timeout` + `terminate-after` as exactly
   this backstop ("a test that cannot finish must fail, not stall"); `timeout(1)` does not exist on
   macOS, which is why this lives in nextest config and not a shell wrapper (the ticket's own note).
2. **OUR PERMISSIVE DEPS** — `alacritty_terminal` owns the Drop-reap semantics today; reading its
   `tty/unix.rs` at implement decides the exact syscall sequence (which handle exposes the pid, what
   `next_child_event` consumes). rustix already ships the `kill`/`waitpid` surface we'd need.
3. **OUR OWN CODE** — `write_bytes`' `PUMP_RETRY_BUDGET` (session.rs) is the house "bounded wait or
   typed error" idiom; the teardown adopts the same stance. Checked gpui: no process-reap owner there.

## Locked-In Decisions
- **D1-NEXTEST-TERMINATE** — `.config/nextest.toml`, default profile, `period = "60s"`,
  `terminate-after = 3` (kill at 180 s ≈ 30× the heaviest legitimate test). Numbers revisit only with
  evidence, never upward silently.
- **D2-BOUNDED-TEARDOWN** — explicit HUP → deadline → KILL → deadline → give-up-and-log escalation;
  never an unbounded wait in session teardown. The deadline pair defaults to 2 s + 2 s.
- **D3-PURE-STEP-SEAM** — the escalation logic is a pure, clock-injected step function (cov/MSI 100);
  pty_os stays an ACCEPTED-UNTESTABLE executor.
- **D4-TESTS-STAY-UNDER-COVERAGE** — no exclusion, no baseline (§0); the bound is the fix.
- **D5-ZOMBIE-OVER-HANG** — if a child survives SIGKILL past the second deadline (pathological D-state),
  leak it deliberately with a log line; a zombie is recoverable, a hang is not.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | terminate + FAIL any test still running at the nextest kill ceiling (config present, default profile) | negative smoke: a throwaway `loop {}` test under `cargo nextest run` is TERMINATED and the run exits non-zero; test then deleted, `git status` clean |
| REQ-002 | apply the same ceiling under the coverage runner | the same smoke via `cargo llvm-cov nextest` (the original crime scene's lane) |
| REQ-003 | complete session teardown within the bounded deadlines with a LIVE (non-exited) child | integration test: spawn `/bin/sh`, do NOT exit it, drop the session, assert wall-clock < HUP+KILL deadlines + margin |
| REQ-004 | drive the reap escalation HUP-first, KILL at the first deadline, give-up at the second | pure `reap_step` truth table (cov/MSI 100) |
| REQ-005 | keep `resize_real_pty_succeeds` byte-identical in intent and green under `cargo llvm-cov nextest` | run the exact original lane; gate:4 green |
| REQ-006 | leave the read/write paths untouched (O_NONBLOCK + retry budget semantics unchanged) | existing integration tests green; diff review |

## Phase Plan
P2 read alacritty `tty/unix.rs` (vendored) — confirm what Drop blocks on + which handle carries the
pid; pick the poll primitive (`next_child_event` vs rustix waitpid/WNOHANG); confirm the deadline pair.
P3 nextest.toml first (the floor lands even if the source fix grows), then the pure step fn, then the
pty_os/shutdown wiring. P3.5 critics on: signal ordering vs SIGCHLD pipe consumption (does our poll
consume the event alacritty's Drop later needs?), the give-up path's fd hygiene, config typos silently
ignored by nextest (prove the profile LOADED — the REQ-001 smoke does). P4 the truth table + the live-
child teardown integration + both smokes + gate. P5 docs (testing-standards note: the ceiling is now
enforced) + AAR. Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md); **all
heavy builds `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`**.
