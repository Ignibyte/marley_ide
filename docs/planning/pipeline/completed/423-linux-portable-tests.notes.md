# 423 Linux-portable tests — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-423-linux-env-sensitive-tests.md
- **Pipeline spec:** 423-linux-portable-tests.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/goal` batch item 3 (Chad 2026-08-14): #423 **auto approved**,
  autonomous through `/commit`. Queue-top promotion (the Queue is now empty).
- **Classification / tier:** chore (test-lane portability), work pipeline, ONE
  shippable slice — the six members ship together because "done" is a lane
  property (the box drops its `-E` filter + retry forgiveness), not six
  independent greens.
- **Recall (§18.3) + pre-read (gathered during 365's gate wait —
  scratchpad/423-preread.md, reproduced here so it survives):**
  - **Members 1–2 do NOT depend on FS case-sensitivity** (the ticket's framing
    is off-target): the twin spelling is the macOS `/var → /private/var`
    SYMLINK — `root_a = TempDir path`, `root_b = canonicalize()`, guarded by a
    deliberate loud `assert_ne!` (headless_drive.rs:8572-8582). On Linux
    `/tmp` isn't a symlink → identity → the guard itself fires. Portable fix
    direction: CREATE the alias (`std::os::unix::fs::symlink`), keep the
    guard.
  - Member 3 (`write_after_disconnect_errors`, integration.rs:82,
    `#[serial]`): waits ChildExited (≤5 s pump) then retries writes ≤50×20 ms
    expecting `Err(SessionError::Disconnected)`. Linux failure mode to
    confirm from the adapter source (sweep): exit-event race vs writes
    never erroring.
  - Member 6 (`teardown_is_instant_when_child_already_reaped`,
    integration.rs:142): first assert `reaped.is_some()` failed in 34 ms on
    the box — the EIO-before-reap-event race; TICKET flags possible REAL
    `marley_terminal` pump gap; spec D2 makes it a triage.
  - Member 4: trybuild .stderr pins rustc's E0277 help-span rendering into
    the `offset_newtype!` macro; already uses `...` elision; Linux variance
    exceeds it.
  - Member 5: 4 s fixed pump window (200×20 ms) — the exact anti-pattern the
    PR-…-subprocess-wait-finite-deadline rule (prevention-rules.md:537 block)
    bans; fix = deadline ceiling + early exit.
  - Dev-box state: sweep FINISHED 2026-08-13 20:14:32 (4784 tested / 16
    missed = the 12b78c8 slice / 848 unviable / 54 timeouts, rc=3; scope
    inactive) — the box is IDLE for this pipeline's ssh verification runs.
    `NEXTEST_RETRIES=1` currently masks the flaky class there; runner is
    do-not-modify from this repo (drop rider goes to Chad at complete).
  - #348 notes own the teardown-bound design (the `reaped` latch); #264 the
    pump idioms; #412 the git-marks fixture lineage.
- **Box pre-flight (for REQ-006):** checkout `dev:~/builds/marley` (Ubuntu
  x86_64, zsh 5.9) currently at 514e5b9; sync topology = mini pushes to the
  bare `dev:git/marley.git` (`git push dev main`), box pulls from it — the
  mini is 2 commits ahead right now (b6ff4ba, 48ae7de); P4 pushes + pulls
  before its ssh suite runs. `cargo` needs the login env on non-interactive
  ssh (`source ~/.cargo/env` or full path).
- **Discovery:** Explore agent (time-boxed) mapping: terminal_blocks'
  ChildExited production + EIO arm + Disconnected mapping;
  alacritty_terminal's child-watcher/EIO handling (ADOPTION — registry
  source); trybuild's normalization surface. Report lands in Phase 2 (design
  folds it).
- **Decisions:** spec D1–D4 (portable-by-construction; member-6 triage-first;
  no runner edits — ssh verification only; deadline ceilings keep green
  fast). EARS REQ-001..006 incl. the box-suite receipt REQ.

## Phase 2 — Design

### Sweep report (Explore, registry ADOPTION — the design inputs)
- **Marley uses alacritty's `tty` module but NOT its event loop** (pty_os.rs:13-14
  imports `tty::{ChildEvent, EventedPty}`; session.rs:336 is Marley's OWN read
  pump). Child exit = SIGCHLD self-pipe, edge-triggered: `next_child_event`
  reads the pipe; `WouldBlock` → `None` WITHOUT `try_wait`
  (alacritty_terminal-0.26.0 `tty/unix.rs:384-403`); Marley's
  `poll_child_exit` latches `child_reaped` on `Some` (pty_os.rs:92-100, :35).
- **Member 6 = REAL pump gap (D2 triage: production fix).** Marley's read-error
  arm treats EIO as generic failure → `read_failed` → ONE `poll_child_exit`
  try → `Err(Disconnected)` with the latch UNSET (session.rs:362-386). The
  slave fds close with the child (unix.rs:227-229/277-278), so EIO can beat
  the SIGCHLD byte. **alacritty's event_loop.rs:283-292 handles exactly this**:
  `#[cfg(target_os="linux")] if err.raw_os_error()==Some(libc::EIO) { continue }`
  — loop back for the inevitable `Exited`. Apache-2.0, adoptable.
- **Member 3 = structural, not errno-mapping.** Linux masters ACCEPT
  post-close writes (bytes queue in the flip buffer, no reader) → all 50
  retried writes return `Ok` → deterministic fail. macOS surfaces EIO →
  passes. `write_bytes`/`classify_write` (session.rs:119-133, :283-308)
  consult ONLY per-call errno; `child_reaped` is unreachable through the
  `PtyChannel` trait (session.rs:97-106). Fix seam: a SESSION-LEVEL
  disconnect latch (set on `ChildExited` and the EIO-read arm) consulted by
  `write_bytes` — the contract becomes platform-UNIFORM (D1's strong form),
  not a cfg split.
- **Member 6's test mechanics:** `reaped` is `pump_until`'s `Option<()>`,
  `None` on any pump `Err` (integration.rs:44) — the EIO-first ordering fails
  BOTH its asserts from the one root cause (latch unset → full HUP→KILL loop
  → the 1 s bound also blows).
- **trybuild 1.0.117 (member 4): NO wildcards** — `...` is diagnostic-
  continuation formatting, not elision (normalize.rs:647/:658). The
  portability lever is the built-in `Normalization` ladder
  (normalize.rs:44-66: RustLib/CargoRegistry/PathDependencies/
  DependencyVersion/…), matched against ANY rung. Residual Linux variance is
  therefore likely TOOLCHAIN rendering (box rustc vs mini rustc) — design
  diffs the box's actual output before choosing regenerate vs restructure.
- **Versions:** alacritty_terminal 0.26.0 (exact, locked); trybuild 1.0.117.
### Architecture (approach)
Reference §20 re-confirmed N/A (POSIX-reality pump correctness + test-lane
portability; the EIO-continue idea is ADOPTED from Apache-2.0
alacritty_terminal — cited in the code comment, not translated from any
copyleft source).

1. **Pump order-independence (member 6, REQ-003) — non-blocking grace.** In
   `pump`'s read-error arm, recognize EIO (`raw_os_error()==Some(EIO)`, the
   existing const) as "probable child exit" (the slave fds die WITH the child):
   set `read_eio` alongside `read_failed`. On `poll_child_exit()==None &&
   read_failed`: if `read_eio`, start/consult `eio_since: Option<Instant>`
   (new session field) — within `EIO_EXIT_GRACE` (1 s) return `Ok(events)` so
   the NEXT pump (app tick 16 ms; test loops) retries and picks up the
   SIGCHLD byte; past the grace, `Err(Disconnected)` (the old no-exit
   contract, deadline-bounded). NEVER sleep in pump (the #147/TICKET-023
   frame-tick rule) — the grace is measured ACROSS pump calls, not slept.
   Platform-UNIFORM (no cfg): macOS EIO takes the same harmless path.
   `eio_since` clears on a successful read and on the exit event.
2. **Write latch (member 3, REQ-002) — state, not errno.** New session field
   `child_exited: bool`, set where `SessionEvent::ChildExited` is pushed
   (:381 site). `write_bytes` checks it FIRST → `Err(Disconnected)`. Why:
   Linux masters ACCEPT post-close writes (flip-buffer, no reader) — errno
   alone cannot carry the post-exit contract; macOS EIO/BrokenPipe classify
   stays as the pre-observation belt (`classify_write` untouched). The
   contract strengthens to: after ChildExited is OBSERVED, the very first
   write fails, both platforms.
3. **Twin-alias fixtures (members 1–2, REQ-001) — explicit symlink.** Replace
   the `/var`-symlink dependence: `real = tmp/root-real`,
   `alias = symlink(real) at tmp/root-alias`; `root_a = alias`,
   `root_b = real`. Guards: `assert_ne!(root_a, root_b)` (distinct spellings)
   + `assert_eq!(root_a.canonicalize()?, root_b.canonicalize()?)` (one dir) —
   the environmental assumption stays LOUD. Uniform on macOS (drop the tmp
   trick entirely).
4. **Deadline waits (member 5 + member 6's own window, REQ-005).** The echo
   test's fixed 200×20 ms budget → wall-clock deadline (~15 s ceiling, 20 ms
   cadence, early-exit on success) per the PR-…-finite-wall-clock rule;
   member 6's `pump_until` window checked to the same rule at implement.
5. **trybuild (member 4, REQ-004) — PROBE VERDICT: toolchain pin (D5).** The
   box probe found the real variance: **box rustc 1.94.0 vs mini 1.96.0**
   (no `rust-toolchain.toml` anywhere — each machine rides its own stable),
   and rustc's diagnostic rendering (the trait-impl help annotations) moved
   between those minors. Fix: **pin the workspace with `rust-toolchain.toml`
   (channel = "1.96.0")** — the strongest form of the ticket's
   "version-tolerate": the variance class dies for every current AND future
   ui test, and the box's mutation verdicts become same-compiler-honest vs
   the gate. The committed `.stderr` already matches 1.96 (it gates green on
   the mini), so NO snapshot change; the box auto-installs 1.96 via rustup on
   its next cargo invocation (one-time re-toolchain/rebuild; the sweep is
   FINISHED so nothing is disturbed; the seed's caught-list stays valid as
   data). Rejected: regenerate-under-1.94 (breaks the 1.96 mini),
   restructure-the-fixture (cannot guarantee stability across future rustcs),
   box-side rustup-without-pin (drifts again).

### Box probe receipts (design evidence, dev @514e5b9)
- rustc: box `1.94.0 (4a4ef493e 2026-03-02)` vs mini `1.96.0 (ac68faa20
  2026-05-25)`.
- Member 3 REPRODUCED deterministically: `assertion left == right failed —
  left: Ok(()), right: Err(Disconnected)` (integration.rs:102) — Linux
  accepts post-exit master writes; the state-latch design is confirmed.
- Member 6 PASSED this (unloaded) run — consistent with its load/CPU-quota-
  modulated profile (fails under the systemd scope; 13/13 green unscoped);
  the root cause stands from source analysis (EIO-before-SIGCHLD).
- Member 4 FAILED with the rendering diff (the `CharOffset implements Add`
  help-annotation shape) + trybuild's `TRYBUILD=overwrite` bless hint.

### File manifest
| File | Change |
|---|---|
| `crates/terminal_blocks/src/session.rs` | `child_exited` + `eio_since` fields; EIO-grace arm in `pump`; latch check in `write_bytes`; `EIO_EXIT_GRACE` const; `#[cfg(test)]` deadline-rewind helper; unit tests over the scripted mock channel for every new branch (mutation-killable). |
| `crates/terminal_blocks/tests/integration.rs` | member 3: post-observed-exit write asserts `Err(Disconnected)` on the FIRST write (strengthened, uniform); member 6: semantics unchanged (the pump fix makes it hold), window checked vs the PR rule. |
| `crates/marley_app/src/headless_drive.rs` | both twin-alias fixtures → explicit-symlink shape with the two guards. |
| `crates/marley_app/tests/integration.rs` | echo test → deadline wait. |
| `rust-toolchain.toml` (NEW, workspace root) | `channel = "1.96.0"` — the D5 pin; the committed `.stderr` (already 1.96-shaped) is untouched. |

### Regression test plan
| # | Proves | Test |
|---|---|---|
| T1 | REQ-003 pump order-independence | NEW session units (mock channel): (a) read=EIO + poll=None → `Ok` (grace active), then poll=Some → `ChildExited` + latch set — kills "grace→immediate-Err" mutants; (b) grace EXPIRY via the `#[cfg(test)]` rewind helper → `Err(Disconnected)` — kills "grace never expires"; (c) eio_since clears on successful read. |
| T2 | REQ-002 write latch | NEW unit: after a pump that surfaced ChildExited, `write_bytes` → `Err(Disconnected)` with a mock that would ACCEPT the write (the Linux shape — proves state beats errno); existing errno-path units untouched. |
| T3 | REQ-002/003 end-to-end | members 3+6 integration tests themselves, run on BOTH platforms (local + ssh box). |
| T4 | REQ-001 | members 1–2 with the symlink fixture, both platforms. |
| T5 | REQ-004 | member 4 trybuild green both platforms (post-verdict fix). |
| T6 | REQ-005 | member 5 (+6-window) deadline shape review + green under `--test-threads` load both platforms; green runs < 5 s. |
| T7 | REQ-006 | ssh box: full workspace suite, NO `-E` exclusions → 0 failures; receipt pasted. |
- Uncoverable: none new (ssh runs are verification, not gated tests).

### Risks
- **Grace latency**: a dead child's `Disconnected` (absent the exit event) now
  surfaces up to 1 s later — only on the pathological no-SIGCHLD path; the
  normal path surfaces ChildExited the pump AFTER the EIO (16 ms).
- **Write-latch semantics**: writes into a pane whose child exited now
  fail-fast by state — correct (typing into a dead pane), and the app's
  write-error handling already routes `Disconnected`.
- **macOS behavior shift**: EIO used to `Err` immediately; now graced ≤1 s
  first. The only caller that could notice is a test asserting immediate
  Disconnected-on-EIO-without-exit — sweep at implement (grep
  `Disconnected` asserts) and adjust windows if any.
- **Member-4 unknown** until the probe lands; worst case = restructure the ui
  fixture (kept minimal).

## Phase 3 — Implement
- **React-first: N/A** (spec: no UI delta).
- **Built (to manifest):**
  - `session.rs`: `child_exited` + `eio_since` fields (+doc comments carrying
    the Linux flip-buffer/EIO-order rationale), `EIO_EXIT_GRACE = 1 s` const,
    the EIO-grace arm in `pump` (non-blocking, measured across pumps;
    `eio_since` cleared on live read + exit), latch set at the ChildExited
    push, `write_bytes` latch-check-first, `expire_eio_grace_for_test`
    (`#[cfg(test)]` rewind — no sleeps in units).
  - `terminal_blocks/tests/integration.rs`: member 3 strengthened — asserts
    the exit event SURFACES (15 s deadline) then the FIRST write is
    `Err(Disconnected)` (uniform, by state; the errno-retry loop is gone);
    member 6 window 5→15 s (D4) + rationale comment, semantics unchanged.
  - `headless_drive.rs`: both twin-alias fixtures → created-symlink shape
    (`root-alias` → `root-real`) with BOTH guards (distinct spellings; one
    canonical dir); comments rewritten to the #423 story.
  - `marley_app/tests/integration.rs`: echo wait → 15 s wall-clock deadline,
    20 ms cadence, early exit (+`Instant` import).
  - `rust-toolchain.toml` (NEW): `channel = "1.96.0"` + the D5 rationale;
    rustup synced the pinned toolchain locally — SAME binary as stable
    (commit ac68faa20), so the warm cache survived (5.18 s check).
- **Deviations:** none from the design; member 3's precondition window also
  widened 5→15 s (D4 consistency, not in the manifest's line item).
- **Existing-test audit (the design risk):** all pre-existing session units
  use `BrokenPipe`/zero-write/raw-EIO on the WRITE path — none assert
  immediate-Disconnected on an EIO READ, so the grace changes no existing
  test's expectation. (The validate-phase units will extend `MockPtyChannel`
  with a raw-errno read push for the EIO arms.)
- **Checks:** `cargo check -p marley_terminal -p marley --all-targets` clean
  under the pin; `cargo fmt --all --check` clean.

## Phase 3.5 — Inspect
- **Critics:** 2 spawned (pump/latch correctness+state; provenance+portability+
  simplification) — table below as reports land.
- **Lead-found (empirical run BEFORE the critics reported):**
  | Sev | Finding | Fix |
  |---|---|---|
  | HIGH (caught pre-commit) | The symlink fixture rewrite kept `boot(cx, &root_a)` — but the ORIGINAL conflated config-dir and root_a (both were the tempdir); with root_a now a symlink SUBDIR, boot read an empty config → zero projects → `index out of bounds` in both twin tests. | `boot(cx, cfg.path())` in both + a comment naming the conflation; both tests GREEN (0.27 s). A textbook hidden-coupling break: the fixture's two roles (config home vs project root) were one path by accident. |
- **Empirical state at critic-spawn:** terminal_blocks 128/128 (members 3+6
  green under the pump fix); fixtures 2/2 (post-fix); echo 1/1; fmt clean;
  clippy: one transient `expire_eio_grace_for_test` never-used warning — its
  consumers are the P4 units, written before the gate runs (noted, not
  suppressed).
- **Critic findings ledger (both reports in):**
  | Sev | Finding (critic) | Verdict | Disposition |
  |---|---|---|---|
  | HIGH | EIO-grace ships with zero direct coverage; the cfg(test) helper is consumer-less — REQ-003's "mutation-covered" clause unmet; every grace-branch mutant survives today (correctness) | REAL — and structurally enforced: gate:2 (-D dead_code) + gate:5 CANNOT pass without the consumer | P4 T1 rows write the units (sharpened per the critic: `io::Error::from_raw_os_error(EIO)` mock pushes — the existing `push_read_err(ErrorKind)` never sets a raw errno; extend `MockPtyChannel`), consuming the helper. |
  | MED | `pump`'s doc still said fatal-read-yields-Disconnected unconditionally (doc drift vs the grace) | REAL | FIXED — doc now states non-EIO immediate / EIO after `EIO_EXIT_GRACE`. |
  | MED | `rust-toolchain.toml` untracked — a `git add -u` commit would ship everything EXCEPT the D5 pin | REAL-but-covered | /commit stages with `git add -A`; the commit task now names the file explicitly (belt). |
  | LOW | Post-observed-exit EIO re-armed a fresh 1 s grace per dead pane (the exit event cleared `eio_since`); no correctness break (verified all dead consumers transition-guarded) but a cadence change | REAL (tidy) | FIXED — grace gated on `!self.child_exited`: post-exit EIO is an instant `Err`, the pre-#423 dead-pane cadence. |
  | minor | Toolchain pin lacked a `components` list — llvm-tools NOT installed for 1.96.0 (gate:4 would redden; a fresh minimal-profile rustup would even lose rustfmt/clippy) (provenance) | REAL | FIXED — `components = ["rustfmt","clippy","llvm-tools"]` in the pin + llvm-tools installed on BOTH machines now. |
  | minor | The 10-line twin-root fixture body duplicated verbatim in both tests (the file's idiom for these two tests is a shared helper) | REAL | FIXED — `alias_twin_roots(cfg) -> (alias, real)` beside `seed_two_projects`, guards inside; both tests rewired; 2/2 green. |
  | INFO | `expire_eio_grace_for_test` Instant-underflow — monotonic base is since-BOOT on both OSes, unreachable in a real test run | noted | none. |
  | INFO | `eio_since` survives an idle WouldBlock pump between two EIOs (mock-only edge; real masters' EIO is terminal) | noted | P4 unit keeps it in mind; no code change. |
  | note | app.rs:1899-1913 dead-pane comments were ALREADY stale vs the edge-triggered poll (pre-existing, before this diff) | pre-existing | out of 423's manifest — left; candidate future tidy. |
  - **Clean lenses (verified):** §20 structural independence from alacritty's
    Linux-EIO `continue` (ours: bounded cross-pump grace + expiry contract +
    write latch, no analog; license Apache-2.0 verified in-registry); no
    post-exit write caller exists for the latch to break (~20 sites audited);
    grace lifecycle order (burst-then-die restarts, no stale timestamp);
    Ok(0)/EOF arm untouched (macOS order-independence preserved);
    latch↔`child_reaped` cannot diverge (single poll caller); member-3/6
    soundness under the new pump (pump_until keeps looping through grace
    pumps); toolchain blast radius (only `cargo +nightly miri` is qualified
    and CLI `+` outranks the file; no stable hardcodes; no CI files);
    unix-only symlink confined to the cfg(test) module (no windows lane
    exists); §15 strictly STRENGTHENED (member 3's old form never asserted
    the exit and accepted any-of-50; ceilings widened failure-side only);
    `NEXTEST_RETRIES=1` cannot false-pass the latch test (state-deterministic).
- **Post-fix verification:** fixtures 2/2 (helper-based), terminal 128/128
  (with the `!child_exited` gate + doc fix), fmt clean.

## Phase 4 — Validate
- **Tests added (T1/T2, per the plan + the inspect HIGH):** 5 session units +
  the `push_read_raw_err(i32)` mock extension (ErrorKind-only pushes carry no
  raw errno, so the EIO arm was untestable without it):
  `pump_eio_within_grace_reports_quietly_then_exit_lands`,
  `pump_eio_grace_expiry_disconnects` (consumes the rewind helper — the
  dead-code warning is gone), `pump_bytes_then_eio_restarts_the_grace`,
  `pump_eio_after_observed_exit_disconnects_instantly` (the `!child_exited`
  gate), `write_after_observed_exit_disconnects_by_state` (asserts the
  channel was NEVER touched — recorded_writes empty).
- **Local runs (macOS, pinned 1.96.0):**
  - `cargo nextest run -p marley_terminal`: **133/133** (128 + the 5 new).
  - `cargo nextest run --workspace`: **2168 passed, 7 skipped** (21.2 s).
  - `cargo test --workspace --doc`: ok (2 passed; 17 crates doctest-empty).
  - Members on macOS: 3+6 in the terminal suite run; 1+2
    (`alias_root_twin`) 2/2; 5 (echo) green — all in the workspace run.
- **Live-app capture: N/A — no UI delta** (test-lane + a pump bookkeeping fix
  behind existing events; spec React-first is N/A).
- **Gate round 1: RED (two findings, both fixed at source):**
  - gate:14 — `pump`'s public doc intra-linked the PRIVATE `EIO_EXIT_GRACE`
    (`-D rustdoc::private-intra-doc-links`); de-linked to plain backticks.
  - gate:5 — ONE missed mutant: `<`→`<=` at the grace comparison — the
    boundary (`elapsed == GRACE`) is unpinnable through a live `Instant`.
    Fixed with the repo's pure-decision idiom: `eio_grace_expired(elapsed)`
    (boundary = expired) + `eio_grace_expiry_boundary` unit at GRACE−1ns /
    GRACE / GRACE+1ns — the mutant class dies deterministically.
- **Box round 1 (snapshot `5e7cc5b`): 2166/2168** — ALL SIX 423 members GREEN
  on Linux (trybuild under the pin; echo at 11.9 s under load; both PTY
  members; both twin-alias fixtures). The two failures were NON-member
  siblings of member 5's class, exposed by the no-exclusion run:
  `torture_command_round_trips_exactly` (a `400×20 ms` iteration budget) and
  `workspace_two_real_sessions_are_independent` (an 8 s ceiling; failed at
  8.08 s under cold-build load). Both got the same D4 treatment (15 s
  wall-clock deadlines, early exit) — validate-discovered members, in scope
  via REQ-006.
- **Box round 2 (snapshot `29be05c`, all fixes): 2169 tests run: 2169
  passed, 7 skipped — ZERO failures, NO exclusions, pinned 1.96.0
  (ac68faa20, identical to the mini).** REQ-006 receipt:
  `dev:~/builds/logs/423-verify.log` (12:45 UTC run). The 7 skips are the
  `#[ignore]`d headed tests — identical on both platforms, not lane
  exclusions.
- **Gate round 2:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15
  PASS** — receipt written for `/commit`. Local totals: terminal 134/134,
  workspace 2168/2168 + the boundary unit (the box ran 2169).

## Phase 5 — Complete
- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (TICKET-423);
  `docs/marley_architecture/terminal_blocks.md` gained the exit-order/state-
  latch bullet (after the #348 teardown one). React parity: N/A.
- **Ledger appends:**
  `AD-claude-423-pty-exit-order-independence-is-grace-plus-state-latch-001`,
  `AD-claude-423-the-workspace-pins-its-toolchain-001`;
  `L-claude-423-a-tickets-environmental-diagnosis-is-a-hypothesis-not-a-fact-001`
  (at design) and
  `PR-claude-423-a-fixture-path-playing-two-roles-splits-loudly-001` (at
  inspect) already in.
- **Ticket:** closed with the completion story + the RIDER for Chad (drop the
  box runner's `-E` filter + `NEXTEST_RETRIES=1` — verified dead weight);
  moved to `tickets/closed/`. BACKLOG Queue is the empty placeholder (swept).
- **Archive:** pair → `pipeline/completed/`. Box checkout returns to `main`
  at the post-commit step (with the `423-verify` branch deleted).
