# Bounded PTY teardown + the nextest kill-switch — Notes

- **Forge ticket:** #348 `477c649a-a702-493b-94f2-9a2a516dbce8`
- **AAR:** `2a241d0b-0f77-4f1c-8076-d077336d88c0`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-348-pty-teardown-bound.md
- **Pipeline spec:** 348-pty-teardown-bound.spec.md
- **pipeline_id:** `97d7778e-4fa2-412c-85de-012722981158`

<!-- Working scratch. Excluded from gate:14 doc-todos. -->

## Phase 1 — Plan

- **Request:** no test may ever hang a gate again. (D1) a `.config/nextest.toml` terminate ceiling covering
  every nextest lane; (D2/D3/D5) a bounded session teardown replacing the implicit unbounded child reap
  delegated to alacritty's `tty::Pty::Drop`, via a pure clock-injected `reap_step` + a `pty_os` shim.
- **Classification / tier:** work pipeline, `bug`, M22. TWO artifacts: a config file (D1) + a `.rs` source
  fix with a pure-seam cov/MSI-100 obligation (D2/D3). One shippable slice.
- **Forge recall (§18.3):** bulletins EMPTY. `knowledge-context` (Plan) surfaced 13 nodes into the AAR —
  top PR `004a7a9f` (0.81, structural rank 1 — the house bounded-wait idiom), PR `a7199501`, distilled
  lessons `1828c2ab`/`feecfa55`, 2 recent failures. The bounded-wait PR is the governing prior rule.

### ★ Founding-sentence re-examination (the headline finding)
The spec's premise — "an 11-hour gate wedged on `resize_real_pty_succeeds`, SLOW >39,240s at 0.0% CPU,
blocked not slow." My earlier "may be the syspolicyd fault (same 0% signature)" flag is **refined, not
upheld**, using the first-hand syspolicyd wedge from #345's gate:
- **syspolicyd wedge (observed on #345):** the nextest LOG FREEZES with NO test started — the test binaries
  hang in `_dyld_start` (pre-`main`) at 0% CPU while the Gatekeeper daemon trio (syspolicyd/amfid/taskgated)
  sits idle at 0%. nextest never lists a running test.
- **#348 incident:** nextest was ALIVE and reporting ONE specific test (`resize_real_pty_succeeds`) as SLOW
  for 39,240s. A binary that reached the point of *running* that test is PAST syspolicyd.
- **Conclusion: the premise HOLDS.** The hang was IN the test's PTY teardown, not the OS loader. The
  **distinguishing signature** — nextest-reports-one-running-test-SLOW (real hang) vs frozen-log-no-test
  (syspolicyd) — is the durable lesson (both look like "0% CPU, no progress" from a distance).

### ★ The unbounded wait is REAL and located (spec's D2/D3 justified — confirmed at P1, not deferred)
Reading the vendored `alacritty_terminal-0.26.0/src/tty/unix.rs` (permissive dep — ADOPTION, outside the
§20 wall):
```
impl Drop for Pty {                        // unix.rs:309
    fn drop(&mut self) {
        ... libc::kill(self.child.id() as i32, libc::SIGHUP);   // :313
        let _ = self.child.wait();                              // :319  ← UNBOUNDED blocking waitpid
    }
}
fn next_child_event(&mut self) -> Option<ChildEvent> {          // :384
    ... match self.child.try_wait() {                           // :395  ← non-blocking (WNOHANG) poll
        Ok(exit_status) => Some(ChildEvent::Exited(...)), ... }
}
```
So alacritty's `Pty::Drop` sends SIGHUP then **blocks forever** on `Child::wait()` if the child ignores HUP
or races — the exact 11-hour hang. The source fix is justified.

### The fix mechanism (all handles confirmed present at P1)
- **Fix site:** the `OsPtyChannel` teardown (`crates/terminal_blocks/src/pty_os.rs`). Today
  `TerminalSession::shutdown(self)` is EMPTY (session.rs:353); dropping the session drops `OsPtyChannel`
  → drops `pty: tty::Pty` → alacritty's blocking `Pty::Drop`. `resize_real_pty_succeeds`
  (tests/integration.rs:108) is exactly this: spawn a live `/bin/sh`, resize, `session.shutdown()`.
- **The crate itself is clean:** grep across `crates/terminal_blocks/src` finds ZERO `impl Drop`,
  `thread::spawn`, `waitpid`, or `.wait()` — the only unbounded wait is delegated to alacritty (spec claim
  holds on `1e4c092`). Reads are O_NONBLOCK; writes are `PUMP_RETRY_BUDGET`-bounded (=8, session.rs:32).
- **pid handle:** alacritty `Pty::child()` (unix.rs:110) is a PUBLIC `&Child` accessor → `Child::id()` is
  the pid. So `self.pty.child().id()` gives it; `rustix` (already a pty_os dep — used for `tcsetwinsize`)
  sends `SIGHUP`/`SIGKILL`.
- **poll primitive:** `self.pty.next_child_event()` (already used by our `poll_child_exit`, pty_os.rs:75) →
  `try_wait()` (non-blocking). This is the reap poll.
- **Success path is clean (no leak):** `try_wait()` reaping the child CACHES the status in `std::process::
  Child`, so alacritty's later `child.wait()` in Drop returns immediately — no hang, no forget.
- **Give-up path (D5):** if the child survives SIGKILL past the second deadline (pathological D-state), the
  Child is un-reaped → alacritty's `Pty::Drop` would STILL block on `child.wait()`. So the give-up path MUST
  neutralize that Drop — `mem::forget` / `ManuallyDrop` the `tty::Pty` — and log the leak. A leaked
  fd/zombie is recoverable; a hang is not (D5 zombie-over-hang). **P2 decides the forget granularity + the
  P3.5 "does our try_wait consume the event alacritty's Drop needs" question (answer sketch: on success it
  is FINE — the cached status makes Drop's wait return; the concern is only the give-up path, handled by
  forget).**

### Decisions confirmed (spec)
D1-NEXTEST-TERMINATE (60s/terminate-after 3), D2-BOUNDED-TEARDOWN (HUP→deadline→KILL→deadline→give-up),
D3-PURE-STEP-SEAM (`reap_step` in session.rs, cov/MSI 100; pty_os shim ACCEPTED-UNTESTABLE), D4-TESTS-STAY-
UNDER-COVERAGE (§0), D5-ZOMBIE-OVER-HANG — all hold. EARS REQ-001..006 confirmed against the live code
(the resize test at integration.rs:108, its gate:4 lane, the O_NONBLOCK/retry-budget paths untouched).

### EARS AC — confirmed hold
REQ-001 (nextest ceiling terminates+FAILs a hung test), REQ-002 (same under llvm-cov nextest),
REQ-003 (bounded teardown with a live child < deadlines), REQ-004 (pure `reap_step` truth table),
REQ-005 (`resize_real_pty_succeeds` byte-identical intent + green), REQ-006 (read/write paths untouched)
— all map to confirmed code sites; none re-derived.

### §20 + Prior art
§20 = N/A (Marley test-infra + process hygiene; no reference-app analog). The ONE source READ is
alacritty's `tty/unix.rs` (permissive dep — adoption, explicitly outside the wall; confirmed `Pty::Drop`
blocks on `child.wait()`). Prior art: (1) the nextest book documents `slow-timeout` + `terminate-after` as
exactly this backstop; macOS has no `timeout(1)`, so it lives in nextest config. (2) our own
`PUMP_RETRY_BUDGET` bounded-wait idiom (session.rs) — the teardown adopts the same stance; `rustix` already
ships the `kill` surface. (3) checked gpui — no process-reap owner.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Architecture / approach.** Two artifacts, both in the `marley_terminal`/`terminal_blocks` adapter layer
(the gpui app → `alacritty_terminal` PTY seam). (1) `.config/nextest.toml` — a repo-root config, no code.
(2) A bounded child-reap in the `PtyChannel` teardown: the DECISION logic is a pure `reap_step` in
`session.rs` (pure crate → cov/MSI 100), the EXECUTION is an `impl Drop for OsPtyChannel` in `pty_os.rs`
(the ACCEPTED-UNTESTABLE shim — already `mutants::skip` + cov-excluded, gates.sh:217). §14: no panic on the
teardown path (a reap failure logs + proceeds, never unwraps); process-spawn/signal stays confined to the
`pty_os` adapter. §20 = N/A confirmed; the one dependency source read is alacritty's `tty/unix.rs`
(adoption) — done at P1.

**File manifest:**
- `.config/nextest.toml` — NEW. `[profile.default]` + `slow-timeout = { period = "60s", terminate-after = 3 }`.
- `crates/terminal_blocks/src/session.rs` — ADD the pure seam: `ReapStage`, `ReapAction`, `REAP_DEADLINE_MS`,
  `reap_step(...)` (+ its `#[cfg(test)]` truth-table tests). No change to `shutdown`/read/write.
- `crates/terminal_blocks/src/pty_os.rs` — `pty: tty::Pty` → `pty: ManuallyDrop<tty::Pty>`; ADD
  `impl Drop for OsPtyChannel` (the bounded-reap executor, `#[cfg_attr(test, mutants::skip)]`); the accessor
  sites (`self.pty.child()`, `self.pty.next_child_event()`, `self.pty.file()`) work unchanged through Deref.
- `crates/terminal_blocks/Cargo.toml` — `rustix` features `["termios"]` → `["termios", "process"]` (for
  `rustix::process::kill_process` + `Signal`). [Fallback: a single `libc::kill` in the skip'd shim if the
  rustix 0.38 `process` API differs — confirm at implement.]

### D1 — the nextest ceiling (exact)
```toml
[profile.default]
slow-timeout = { period = "60s", terminate-after = 3 }
```
DEFAULT profile → applies to EVERY nextest invocation with no `--profile`: gate:3 (`cargo nextest run`),
gate:4 (`cargo llvm-cov nextest`), gate:15 (visual harness), and #345's `cargo mutants --test-tool=nextest`.
Semantics (nextest standard): a test is marked SLOW every `period` (60s); after `terminate-after` periods
(3 × 60s = 180s) nextest sends SIGTERM then SIGKILL, and the terminated test FAILS the run. The exact 180s
is not load-bearing — REQ-001's smoke (a `loop {}` test that nextest TERMINATES → non-zero exit) is the
proof the profile LOADED and took effect (a config typo would be silently ignored, and the loop test would
hang the smoke → visible). [Confirm the precise SIGTERM→SIGKILL grace from the nextest book at implement;
does not change the design.]

### D3 — the pure seam `reap_step` (session.rs, cov/MSI 100)
```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReapStage { Hup, Kill }         // which signal we are awaiting the effect of

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ReapAction { Wait, SendKill, GiveUp, Done }

pub(crate) const REAP_DEADLINE_MS: u64 = 2000;  // D2: 2s per stage (HUP then KILL)

/// Pure escalation decision. The shim has ALREADY sent SIGHUP and entered `Hup` before the first call;
/// `waited_ms` is the elapsed time in the CURRENT stage; `exited` is a fresh non-blocking poll result.
pub(crate) fn reap_step(stage: ReapStage, waited_ms: u64, exited: bool) -> ReapAction {
    if exited {
        return ReapAction::Done;
    }
    match stage {
        ReapStage::Hup => {
            if waited_ms >= REAP_DEADLINE_MS { ReapAction::SendKill } else { ReapAction::Wait }
        }
        ReapStage::Kill => {
            if waited_ms >= REAP_DEADLINE_MS { ReapAction::GiveUp } else { ReapAction::Wait }
        }
    }
}
```
This is the WHOLE cov/MSI-100 surface. The REQ-004 truth table kills every mutant: the `if exited` early
return (a `Done` row for each stage), the two `>=` boundaries (rows at `waited_ms` = 1999 / 2000 / 2001 for
BOTH stages — a `<`/`>`/`==`/`<=` mutant dies on at least one), and the `SendKill` vs `GiveUp` arm split
(Hup-past-deadline → SendKill, Kill-past-deadline → GiveUp). The clock is INJECTED (`waited_ms` is a param —
`Instant::now` is never read inside; #345/#350 idiom).

### D2/D5 — the executor `impl Drop for OsPtyChannel` (pty_os.rs, skip'd + cov-excluded)
```rust
// field: pty: ManuallyDrop<tty::Pty>
#[cfg_attr(test, mutants::skip)]
impl Drop for OsPtyChannel {
    fn drop(&mut self) {
        let pid = /* self.pty.child().id() as i32 → rustix Pid::from_raw */;
        // 1. HUP, enter the Hup stage.
        kill(pid, Signal::Hangup);            // rustix::process::kill_process
        let mut stage = ReapStage::Hup;
        let mut started = Instant::now();
        let reaped = loop {
            let exited = self.poll_child_exit().is_some();   // non-blocking try_wait via next_child_event
            match reap_step(stage, started.elapsed().as_millis() as u64, exited) {
                ReapAction::Done   => break true,
                ReapAction::Wait   => std::thread::sleep(Duration::from_millis(20)),
                ReapAction::SendKill => { kill(pid, Signal::Kill); stage = ReapStage::Kill; started = Instant::now(); }
                ReapAction::GiveUp => break false,
            }
        };
        if reaped {
            // child already reaped by try_wait → alacritty's Pty::Drop child.wait() returns INSTANTLY,
            // and it closes the leader/signal/sig_id fds. Run it for fd hygiene.
            unsafe { ManuallyDrop::drop(&mut self.pty); }
        } else {
            // D5 zombie-over-hang: do NOT run alacritty's Pty::Drop (it would BLOCK on child.wait()).
            // Leak the Pty (child + its fds) deliberately + log. A leaked fd/zombie is recoverable; a hang is not.
            log::warn!("PTY child {pid} survived SIGKILL past the reap deadline — leaking to avoid a teardown hang");
        }
        // self.io (the try_clone'd leader fd) drops normally either way.
    }
}
```
- **pid source:** `self.pty.child().id()` — alacritty `Pty::child()` (unix.rs:110, PUBLIC) → `Child::id()`.
- **poll primitive:** `self.poll_child_exit()` (already present, pty_os.rs:75) → `next_child_event()` →
  `try_wait()` (non-blocking, WNOHANG). **SIGCHLD-consume reasoning (P3.5):** `try_wait` reaping the child
  CACHES the `ExitStatus` inside `std::process::Child`, so on the success path alacritty's later
  `child.wait()` returns the cached status without a syscall — our poll does NOT starve alacritty's Drop; it
  ENABLES its instant return. The only path where alacritty's Drop would block is give-up, which we skip via
  the leak.
- **why `impl Drop` (not a `shutdown`-only method):** Drop fires on EVERY teardown path — `shutdown(self)`,
  a tab-close dropping the session, a panic unwind — not just an explicit `shutdown` call. One bounded reap,
  all paths. `shutdown(self){}` stays empty (its doc updates to point at the Drop). The mock channel (unit
  tests) has no real child and no Drop → unit tests unaffected.
- **why `ManuallyDrop`:** a `Drop::drop(&mut self)` cannot move `self.pty` out to `mem::forget` it; making the
  field `ManuallyDrop<tty::Pty>` lets us CHOOSE — run alacritty's Drop (success, `ManuallyDrop::drop`) or skip
  it (give-up, leak). All `self.pty.*` reads work through `Deref`/`DerefMut`.

### Mutation / coverage homes
- `reap_step` + enums + const → `session.rs` (pure crate, NOT cov-excluded) → cov 100 + MSI 100 via the
  REQ-004 truth table.
- `impl Drop for OsPtyChannel` → `pty_os.rs` — `#[cfg_attr(test, mutants::skip)]` on the impl (matching the
  file's other fns) + the whole file is already cov-excluded (gates.sh:217 `--ignore-filename-regex …
  terminal_blocks/src/pty_os\.rs`). VERIFIED: read/write/set_winsize/poll_child_exit already carry the skip.
- `.config/nextest.toml` — config, no cov/MSI.

### Regression Test Plan
| # | Proof | How |
|---|---|---|
| REQ-001 | a hung test is TERMINATED + FAILs under `cargo nextest run` | add a throwaway `#[test] fn zz_hang_smoke(){ loop{ std::hint::spin_loop(); } }`, run `cargo nextest run -p … zz_hang_smoke` → nextest terminates it at the ceiling, run exits non-zero; then DELETE the test + confirm `git status` clean. (Proves the profile LOADED.) |
| REQ-002 | same ceiling under the coverage runner | the same throwaway under `cargo llvm-cov nextest` (the original crime-scene lane) |
| REQ-003 | bounded teardown with a LIVE child | `#[test] #[serial] fn teardown_bounded_with_live_child()` — `spawn_sh()` (live /bin/sh, NOT exited), record `Instant::now()`, `session.shutdown()` (→ Drop → bounded reap), assert `elapsed < 2×REAP_DEADLINE_MS + margin` (~5s). /bin/sh exits on SIGHUP so the Hup stage reaps it well under the first deadline. |
| REQ-004 | the escalation truth table | pure `#[cfg(test)]` over `reap_step`: exited→Done (both stages); Hup {1999→Wait, 2000→SendKill, 2001→SendKill}; Kill {1999→Wait, 2000→GiveUp, 2001→GiveUp}. cov/MSI 100. |
| REQ-005 | `resize_real_pty_succeeds` byte-identical + green | the test body is UNCHANGED (spawn→resize→shutdown); it now goes through the bounded Drop. Run its exact gate:4 lane green. |
| REQ-006 | read/write paths untouched | the existing integration tests (`dcs_hook_stream…`, the disconnect test) green; diff review — O_NONBLOCK read + `PUMP_RETRY_BUDGET` write are not in the diff. |

Uncoverable-by-unit: the real signal syscalls + the give-up leak live in the cov-excluded `pty_os` Drop
(ACCEPTED-UNTESTABLE); REQ-003's live-child integration test exercises the HUP→reap path end-to-end, and the
give-up branch is pathological (a child surviving SIGKILL — not reproducible deterministically), covered by
the pure truth table's `GiveUp` row + the mechanism (mem-leak is straight-line skip'd shim).

### Risks / decisions
- **(a) give-up leak granularity** — leaking the whole `ManuallyDrop<tty::Pty>` leaks its `file`/`signals`/
  `sig_id` fds too, not just the child. Bounded + pathological-only (a SIGKILL-proof child). D5 accepts it:
  recoverable vs a hang. Not optimizing the granularity (a partial forget is more unsafe code for a path that
  should never fire).
- **(b) `terminate-after` timing** — stated as 180s (3×60s) from the nextest standard; REQ-001's terminate
  smoke proves effective behavior regardless of the exact grace period. Confirm the book's SIGTERM→SIGKILL
  grace at implement; it doesn't change the config or the design.
- **(c) the `sleep(20ms)` tick** — in the skip'd shim, bounded by the deadlines (≤ ~200 iterations per stage);
  the pure `reap_step` has no sleep.
- **(d) `#[serial]`** — the real-PTY tests are `#[serial]`; the bounded Drop changes teardown timing only
  (faster + bounded), not the serialization; REQ-003's new test is also `#[serial]`.
- **(e) rustix `process` feature** — adds the `kill_process`/`Signal` surface; if the 0.38 API differs, the
  fallback is one `unsafe { libc::kill(pid, SIGHUP/SIGKILL) }` in the already-unsafe-adjacent skip'd shim.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest — 4 files, no test expansion:
1. **`.config/nextest.toml`** (NEW) — `[profile.default] slow-timeout = { period = "60s",
   terminate-after = 3 }` + a comment explaining the lanes it covers + the §0 note.
2. **`crates/terminal_blocks/src/session.rs`** — added `ReapStage`, `ReapAction`, `REAP_DEADLINE_MS`
   (2000), and the pure `reap_step` after the consts. Updated TWO stale docs (the false-doc lesson):
   the `PtyChannel` trait doc (:36 — was "dropping a session blocks in the child reap"; now "BOUNDED …
   softens the TICKET-023 concern") and `shutdown`'s doc (was "its Drop SIGHUPs then reaps"; now points
   at the bounded reap). No change to read/write/`shutdown` bodies.
3. **`crates/terminal_blocks/src/pty_os.rs`** — `pty: tty::Pty` → `pty: ManuallyDrop<tty::Pty>`
   (`spawn` wraps `ManuallyDrop::new`); added `impl Drop for OsPtyChannel` (`#[cfg_attr(test,
   mutants::skip)]`) — the executor loop driving `reap_step`. All `self.pty.*` accessors unchanged
   (Deref). Imports: `std::mem::ManuallyDrop`, `std::time::{Duration, Instant}`, the `reap_step`/enums.
4. **`crates/terminal_blocks/Cargo.toml`** — `rustix` features `["termios"]` → `["termios", "process"]`.

**Decisions made at implement (deviations from the design's tentatives):**
- **Signal names:** rustix 0.38's variants are `Signal::Hup` / `Signal::Kill` (the design tentatively wrote
  `Hangup`). Confirmed by a clean `cargo check`.
- **Signalling via rustix (not libc):** `rustix::process::kill_process(Pid::from_raw(raw), Signal::…)` —
  keeps the crate's deliberate "zero libc FFI" stance (Cargo.toml comment). `Pid::from_raw` returns
  `Option`, handled by `if let Some(p)`. No libc dep added.
- **Give-up log:** `eprintln!` (the crate has no `log`/`tracing` dep and the give-up path is pathological
  + in the skip'd shim) — no new dependency.
- **Stale-doc fixes** (trait + shutdown) were not in the literal manifest but are the #307/#321/#350
  false-doc-the-compiler-can't-catch class — the trait doc's "dropping … blocks in the child reap" is now
  FALSE, so it had to change.

**Verification:** `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 cargo check -p marley_terminal` → clean (rc=0,
23.6s). `cargo clippy -p marley_terminal --all-targets -- -D warnings` → clean (no warnings). `cargo fmt
--all` applied. The diff is exactly the 4 files (+ the pipeline docs). No mutation run here (Phase 4).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

Two critics (unsafe-soundness + reap-logic) + my own concrete review. The critics ran slowly (the
machine) and were stopped mid-work, but critic-2 (reap logic) surfaced the SEED of the one real finding
before it stopped; I verified + fixed it. Critic-1 (unsafe) was still reading — the unsafe/ManuallyDrop
lens is carried by my own review below.

### ★ F1 — MEDIUM, CONFIRMED + FIXED: the teardown loop can't see a child reaped during normal use
**Finding (critic-2 seed + my verification):** the reap loop polls `poll_child_exit()` →
`next_child_event()`, which is **EDGE-triggered** (alacritty reads ONE `SIGCHLD` self-pipe byte, unix.rs:387;
returns `None` on `WouldBlock` WITHOUT `try_wait`). The pump ALSO calls `poll_child_exit()` every tick
(session.rs:372) and reaps a normally-exited shell there, consuming that byte. So a child that exited during
normal use is INVISIBLE to a later teardown poll → the loop runs the full HUP(2s)+KILL(2s)=**4s**, then
**GiveUp → leaks the pty**. Two harms: (a) a 4s teardown freeze + an fd leak on the COMMON path (a shell
that exited before the tab is closed); (b) it SIGHUP/SIGKILLs a reaped pid that the OS may have **recycled**
— signalling an unrelated process.
**Verified:** confirmed the pump calls `poll_child_exit` (session.rs:372, emits `ChildExited`) and alacritty's
`next_child_event` is edge-triggered (unix.rs:387-391, `WouldBlock`→`None`).
**Fix (at source):** an `OsPtyChannel.child_reaped: bool` latch — set in `poll_child_exit` whenever it returns
`Some` (the only time the edge is observable), read in `Drop`: if already reaped, SKIP the signals + loop
entirely and go straight to `ManuallyDrop::drop` (alacritty's `Pty::Drop` reaps from the cached status at
once). Every edge re-traced: pump-reaped (latch true → clean instant drop); alive-at-drop (latch false →
loop reaps via HUP); exited-between-last-pump-and-drop (latch false → loop's first poll consumes the pending
byte → Done); genuine SIGKILL-survivor (latch false → loop → GiveUp → leak, the intended D5). All in the
`mutants::skip` + cov-excluded shim → no cov/MSI change. clippy `-D warnings` clean.

### Own review (the unsafe/ManuallyDrop lens — clean)
- **(a) double-drop / drop-skip:** `ManuallyDrop::drop(&mut self.pty)` runs on EXACTLY the `reaped==true`
  branch, never on give-up, never twice; `pty` is untouched after. SAFETY comment accurate.
- **(b) panic-safety:** no statement between entry and `ManuallyDrop::drop` can panic (`child().id()`, `as
  i32`, `Pid::from_raw`, `kill_process` [Result discarded], `poll_child_exit` [Option], `reap_step`,
  `Instant::now/elapsed`, `sleep` — none unwrap/panic). So no double-drop-on-unwind path exists.
- **(c) io vs leaked pty:** `self.io` (a `try_clone`d fd, separate from the pty's `file`) drops on both
  paths → no double-close; the leak is the pty's fds only, one-shot + pathological (give-up branch).
- **(d) pid cast + signalling:** `id() as i32` is lossless for a real pid; `from_raw` returns `Option`
  (handled); a stray signal to a dead pid is `ESRCH`, discarded via `let _ =`. (The recycled-pid risk is
  now moot on the reaped path — F1's latch skips the signals there.)
- **(e/f) the core correctness claim:** on success the child is reaped via `try_wait` (through
  `next_child_event`), which caches the `ExitStatus` in the SAME `std::process::Child` alacritty's `Pty::Drop`
  later `wait()`s (one `child: Child` field; one `pty`), so that `wait()` returns the cached status without a
  syscall — no block, no `ECHILD`. Confirmed from std's `Child` semantics + alacritty's single-Child struct.
- **(g) SIGCHLD gating:** covered by F1 (the fix). During the loop, our own HUP/KILL generate a fresh SIGCHLD
  → a fresh pipe byte → the next poll reaps within a tick or two, bounded.
- **(h) loop termination:** the 4 branches all break or advance (sleep 20ms / send-kill+reset) → no spin;
  `SendKill` resets `started` so the Kill stage gets a fresh 2s; total ≈ 4s + tick slack.
- **(i) `reap_step` totality:** exited→Done (any stage); Hup+`>=2000`→SendKill else Wait; Kill+`>=2000`→
  GiveUp else Wait — Phase-4's truth-table target.
- **(REQ-006)** read/write/set_winsize bodies are NOT in the diff (untouched). No `unwrap`/`expect`; no
  Zed/Warp. `.config/nextest.toml` is valid TOML with the exact `[profile.default]` + `slow-timeout` schema.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests added:** `session.rs` — 3 `reap_step` truth-table tests (REQ-004: exited→Done both stages;
Hup 1999→Wait/2000→SendKill/2001→SendKill; Kill 1999→Wait/2000→GiveUp/2001→GiveUp; + a
`REAP_DEADLINE_MS==2000` pin). `integration.rs` — `teardown_is_bounded_with_a_live_child` (REQ-003) and
`teardown_is_instant_when_child_already_reaped` (the F1-latch regression).

**`cargo nextest run -p marley_terminal` → 127 passed, 0 failed, 0.656s** (real output). Highlights:
```
PASS reap_step_done_when_child_exited_at_any_stage / _hup_waits_… / _kill_waits_…   (REQ-004)
PASS integration teardown_is_instant_when_child_already_reaped   0.022s   (F1 latch — NOT 4s)
PASS integration resize_real_pty_succeeds                        0.033s   (the 11-hour hang — GONE, REQ-005)
PASS integration teardown_is_bounded_with_a_live_child           0.047s   (REQ-003)
PASS integration child_exit_reports_exact_code / write_after_disconnect / dcs_hook_stream  (REQ-006, unchanged)
Summary 127 tests run: 127 passed, 0 skipped   [0.656s]
```
The whole suite finished in <1s — no hang, no syspolicyd wedge. The teardown tests spawn real `/bin/sh`
and the reap is now bounded; the F1 regression test proves an already-reaped child tears down in 22ms
(without the latch it would run the ~4s loop).

**REQ-001 — the terminate ceiling FIRES (real output).** A throwaway `#[test] fn zz_terminate_smoke(){
loop { spin_loop() } }` under `cargo nextest run -p marley_terminal --test zz_terminate_smoke`:
```
SLOW [> 60.000s]  zz_terminate_smoke      ← the 60s period LOADED
SLOW [>120.000s]  zz_terminate_smoke
TERMINATING [>180.000s]  zz_terminate_smoke ← terminate-after=3 → 180s
TIMEOUT [ 180.006s] (1/1)  zz_terminate_smoke
Summary [180.009s] 1 test run: 0 passed, 1 timed out, 0 skipped
error: test run failed        (exit rc=100)
```
So the `.config/nextest.toml` LOADED (SLOW at exactly 60s), TERMINATES at 180s, and FAILS the run
(non-zero). The throwaway test was DELETED + `git status` verified clean. **REQ-002** (the same ceiling
under `cargo llvm-cov nextest`): the terminate-after config is **runner-agnostic** — nextest reads
`.config/nextest.toml` for every lane (`nextest run`, `llvm-cov nextest`, `mutants --test-tool=nextest`)
regardless of the wrapping runner, so the one smoke proves the config for all lanes. (This is the SAME
lane — gate:4 llvm-cov nextest — that hosted the original 11-hour hang, now doubly protected: the source
fix bounds the teardown AND the ceiling backstops any residual hang.)

This distinguished cleanly from the syspolicyd fault: nextest was ALIVE and REPORTING the test SLOW
(a real running test), not a frozen log with no test started.

**Mutation (diff-scoped preview, `cargo mutants --list --in-diff`):** exactly 3 candidates, ALL in
`session.rs reap_step`: `replace reap_step -> ReapAction with Default::default()` (UNVIABLE — `ReapAction`
has no `Default` derive → excluded from MSI), and two `>= → <` (both KILLED by the truth table's 2000
boundary). `pty_os.rs` generated ZERO mutants (all `mutants::skip`). → the gate's gate:5 is MSI 100 (2/2).

**The full `--diff` gate — FIRST run RED (4 gates), fixed at source, re-run.** The first `--diff` gate
found four reds; three were real and deterministically fixed, the fourth is a pre-existing flake:
- **gate:13 SAST + gate:6 miri (BOTH from the new `unsafe`).** The give-up path used
  `unsafe { ManuallyDrop::drop(&mut self.pty) }`; the SAST flags `unsafe` whose `// SAFETY:` isn't on the
  SAME line, and the new `unsafe` pulled `terminal_blocks` into the miri set (miri can't run its real-PTY
  tests, and isn't installed). **Fixed at source by REMOVING the unsafe entirely:** `pty:
  ManuallyDrop<tty::Pty>` → `pty: Option<tty::Pty>`; the give-up path is now a SAFE
  `std::mem::forget(self.pty.take())` (take() moves the `Pty` out, forget skips its blocking `Drop`); the
  success path leaves `self.pty` `Some` so it drops normally (alacritty's `Pty::Drop` reaps from cache at
  once). Verified: `grep unsafe crates/terminal_blocks/src` → NONE → SAST clean + miri skip-clean; clippy
  `-D warnings` clean (no `mem_forget` lint — it's restriction-tier, off by default). This is the cleaner
  fix (no unsafe, no SAFETY-line dance, no `miri-exempt` exemption — §0 "fix at source, don't exempt").
- **gate:14 docs (rustdoc `-D warnings`).** The PUBLIC `shutdown` doc had an intra-doc link `[reap_step]`
  to the `pub(crate)` (private) `reap_step` → `private_intra_doc_links` error. Fixed: plain `` `reap_step` ``
  backticks (no link). The two other `[reap_step]` links are on `pub(crate)` items (private→private, OK).
  Verified: `RUSTDOCFLAGS="-D warnings" cargo doc -p marley_terminal --no-deps` → rc=0.
- **gate:4 coverage — the #334 load-flake, NOT this change.** `marley_app
  headless_drive::new_tab_inherits_live_cwd_headless` FAILED (an assertion, not a timeout) under the heavy
  llvm-cov run. It is the documented #334 load-flaky test (a cwd-inheritance timing race), in `marley_app`
  and driven by a MOCK channel — my change is entirely in `terminal_blocks` and cannot affect it. Prior
  tickets' gates passed it; it is intermittent. Re-running the gate (below) on a cleaner load.
- (The Option rewrite is behaviour-identical: `cargo nextest run -p marley_terminal` → 127/127 pass,
  incl. the F1-latch test at 17ms, `resize_real_pty_succeeds` at 41ms, the 3 `reap_step` truth-table tests.)

**The re-run `--diff` gate → GATE GREEN [diff] (15/15).** All four previously-red gates now PASS:
gate:13 SAST (no unsafe left), gate:14 docs (link fixed), gate:6 miri (skip-clean — no unsafe crate),
gate:4 coverage (the #334 flake passed this run — `new_tab_inherits_live_cwd_headless` PASS). gate:5
mutation `2 caught / 0 missed → MSI 100.0%` (the `reap_step` truth table killed both `>=`→`<` mutants;
the `Default::default()` mutant is unviable). Fresh receipt `34ad7e61bb2c66f78dd04f86b19cd0df203c94dc`
written and verified worktree-bound against `gate_state_hash` (editing the `.rs` source staled the old
hash). syspolicyd stayed healthy across both runs (no wedge this time).

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
