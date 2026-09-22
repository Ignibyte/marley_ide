---
pipeline_id: 9c233977-aee6-4715-94a5-68aacf81fe50
ticket: docs/planning/tickets/open/TICKET-423-linux-env-sensitive-tests.md
status: Phase 5 — Complete PASS
title: The dev-box lane's blind spot dies — 4 excluded tests made portable, 2 flaky members made durable
type: chore
milestone: M29
references:
  - docs/planning/pipeline/parked/407-full-gate-audit-debt.notes.md
  - docs/planning/pipeline/completed/348-pty-teardown-bound.notes.md
  - docs/planning/pipeline/completed/412-git-marks-clear-on-close.notes.md
  - docs/planning/pipeline/completed/264-headless-drive.notes.md
---

## Title
The dev-box mutation lane (the 2026-08-13 sweep home) excludes four tests via a
nextest `-E` filter and forgives two more via `NEXTEST_RETRIES=1` — so a mutant
whose ONLY killer is one of those six reads MISSED (or false-CAUGHT) on the box.
This pipeline makes each portable/durable at the source so the lane can drop
BOTH mitigations: the twin-alias fixtures stop depending on macOS's `/var`
symlink, the PTY tests assert platform-appropriate contracts (and any REAL
Linux pump gap gets triaged + fixed, not tolerated), the trybuild snapshot
stops pinning one platform's rustc rendering, and the two flaky PTY-timing
tests get deadline-based waits. Done = full suite green on BOTH platforms with
no exclusion filter and no retry forgiveness needed for these six.

## Scope
### In
1. **Members 1–2 (twin-alias fixtures)** —
   `fold_entry_survives_alias_root_twin_instance_close_headless`
   (headless_drive.rs:8578) + `git_marks_…` (:8816): construct the two
   spellings by CREATING a symlink alias explicitly (portable on any unix)
   instead of riding the macOS `/var → /private/var` tmp symlink; keep the
   loud `assert_ne!` environmental guard.
2. **Member 3 (PTY write-after-exit)** — `write_after_disconnect_errors`
   (terminal_blocks/tests/integration.rs:82): assert the platform-appropriate
   disconnect contract (design confirms the exact Linux vs macOS error-surface
   mapping from the pump/adapter source).
3. **Member 6 (teardown EIO race — TRIAGE FIRST)** —
   `teardown_is_instant_when_child_already_reaped` (:142): decide REAL pump
   gap (Linux EIO-on-master-before-reap-event ⇒ the reap/exit latch must not
   depend on event-delivery order — a production fix in `marley_terminal`) vs
   test tolerance; fix at whichever source the triage names.
4. **Member 4 (trybuild snapshot)** —
   `ui_mixed_offsets_must_not_compile` (marley_text_offsets): make the
   expected stderr platform/rustc-tolerant (normalization/elision per
   trybuild's own capabilities — sweep) without weakening the contract (the
   mix must still FAIL to compile with E0277).
5. **Member 5 (echo pump window)** —
   `echo_command_produces_a_block_with_its_output`
   (marley_app/tests/integration.rs:46): deadline-based wait (~15 s ceiling,
   early-exit on success — green runs stay fast) per the ticket + the
   PR-…-poll-EVENT-DRIVEN-to-a-finite-wall-clock-deadline rule.
6. **Both-platform verification** — the full suite runs green locally (macOS)
   AND on the dev box over ssh (`ssh dev`, repo at the lane's builds checkout)
   with NO `-E` exclusion for these tests; receipts in notes.

### Out (explicitly deferred)
- The box RUNNER itself (`marley-mutants` script/scope) — do-not-modify from
  this repo; DROPPING the `-E` filter + `NEXTEST_RETRIES` is coordinated with
  Chad after this lands (the ticket's completion rider).
- The #407 timeout-classification/addendum work — parked pipeline resumes
  after this queue.
- Any broader PTY refactor beyond what member 6's triage strictly names.

## Reference (§20)
N/A — Marley-specific test-lane portability; no reference-app behavior is
matched. (The PTY disconnect CONTRACT is Marley's own #348-era design; the
platform semantics beneath it are POSIX, not a product behavior.) No Warp/Zed
source read.

### Prior art
- **Our permissive deps (the highest-yield leg):**
  - `alacritty_terminal` owns the PTY seam Marley's `terminal_blocks` adapts —
    the sweep (Explore, registry source) maps how it surfaces child exit vs
    the Linux master-EIO race and what marley_terminal consumes vs re-derives;
    design's member-3/6 decisions cite those exact sites (summary in notes
    Phase 1/2 — the sweep report).
  - `trybuild` (the member-4 harness) ships its own stderr normalization +
    `...` elision; the fix uses ITS mechanism rather than hand-rolled snapshot
    munging (exact capabilities per the sweep).
  - `tempfile` + `std::os::unix::fs::symlink` cover the twin-alias fixture
    portably — no new dep.
- **Published material:** POSIX PTY semantics — on Linux, closing the slave
  side (child exit) makes master reads return EIO and writes EIO/EPIPE; BSD/
  macOS returns EOF-ish 0-reads and different write errnos. The teardown race
  (exit-closes-master BEFORE the reaper observes the child) is a documented
  PTY class, which is why the fix direction is order-independence, not timing.
- **Behavior maps:** N/A — no user-facing behavior.
- **Our own code:** the #348 teardown-bound design (the `reaped` latch under
  test), #264's pump/poll idioms, the PR-…-subprocess-wait rule (event-driven
  poll to a finite wall-clock deadline — the member-5 shape), the #412
  git-marks fixture lineage (member 2's test).

## React-first (parity)
N/A — no UI delta: test-lane portability + (at most) an order-independence fix
inside the terminal pump's exit/reap bookkeeping; nothing the user sees
changes.

## Locked-In Decisions
- D1 — **Portable-by-construction over platform-gating**: every member runs
  the SAME assertions on both platforms wherever semantics allow; `cfg` splits
  only where the underlying contract genuinely differs (member 3's errno
  mapping may be such a case — design decides from the adapter source, and any
  split asserts BOTH arms, never skips one).
- D2 — **Member 6 is a triage, not a patch**: if the Linux EIO-before-reap
  race can leave `reaped` unset while the child IS gone, that is a REAL pump
  gap — fix in `marley_terminal` (latch from the reap path / EIO implies
  exit-pending) with a regression test; only if the mechanism proves sound is
  test tolerance acceptable.
- D3 — **No lane-runner edits from this repo**; both-platform verification
  runs the suite directly (`cargo nextest run --workspace` locally; the same
  over `ssh dev` in the lane's checkout) — the exclusion-drop itself is
  Chad's runner amendment, listed as the completion rider.
- D4 — **Flaky-window fixes keep green-fast**: deadline ceilings (~15 s)
  with early exit on success; no unconditional sleeps.
- D5 *(added at Design, from the box probe)* — **Pin the workspace toolchain**
  (`rust-toolchain.toml`, `channel = "1.96.0"`): the member-4 variance is
  rustc 1.94 (box) vs 1.96 (mini) diagnostic-rendering drift, not a platform
  property. The pin kills the class for every future ui test and makes box
  mutation verdicts same-compiler-honest; the committed `.stderr` already
  matches 1.96.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the twin-alias fixtures run on a filesystem where tmp is NOT a symlink (Linux ext4), the tests shall construct two distinct spellings of one directory (created symlink) and pass their original census assertions. | Both tests green on the dev box (ssh run) AND locally; the `assert_ne!` guard retained. |
| REQ-002 | WHEN `write_after_disconnect_errors` runs on either platform, it shall assert that post-exit writes surface the platform-appropriate `SessionError::Disconnected` contract within its deadline. | Green on both platforms (local + ssh dev). |
| REQ-003 | WHEN the child exits and the master read errors BEFORE the exit event is observed (the Linux race), the session teardown shall still observe the reap (`reaped.is_some()`) — order-independent. | Member-6 test green on the box under load conditions (repeated runs); if triage found a pump gap: a regression test pinning the order-independence, mutation-covered. |
| REQ-004 | WHEN `ui_mixed_offsets_must_not_compile` runs under the box's rustc, the compile-fail contract (E0277 on the mixed-offset expression) shall hold without pinning platform-specific rendering. | trybuild green on both platforms; the .stderr still demands E0277 + the mixing span. |
| REQ-005 | WHEN the suite runs under lane-load conditions, `echo_command_…` and member 6 shall wait event-driven to a finite wall-clock deadline (~15 s ceiling, early-exit on success) and never a fixed iteration budget. | Code review vs the PR rule + green under `--test-threads` load locally and on the box; green runs stay <5 s for these tests. |
| REQ-006 | WHEN the full workspace suite runs on the dev box WITHOUT the lane's `-E` exclusions, it shall be green (0 failures across the six members' targets). | `ssh dev` suite run receipt pasted in notes (P4). |

## Phase Plan
- **P2 Design** — fold the sweep: exact exit-event/EIO mechanism, member-6
  triage verdict, member-3 error mapping, trybuild normalization choice, the
  symlink fixture shape; file manifest + test plan.
- **P3 Implement** — per manifest (test files + any triaged `marley_terminal`
  fix).
- **P3.5 Inspect** — critics incl. a PTY-order-race lens; provenance.
- **P4 Validate** — full local suite + gate `--diff` green; ssh-dev suite runs
  (with and without load) green; receipts.
- **P5 Complete** — archive, ledger capture, close ticket; hand Chad the
  exclusion-drop rider.
