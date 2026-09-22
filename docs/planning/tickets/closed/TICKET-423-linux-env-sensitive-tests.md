# TICKET-423 — chore: make the 4 Linux-environment-sensitive tests portable (dev-box mutation lane exclusions)

- **Ticket:** LOCAL #423 (chore, M-unset)
- **Tags:** test-lane, portability, linux, mutation, 407-followup, dev-box
- **Created:** 2026-08-13
- **Provenance:** dev-box mutation-lane handoff (2026-08-13, TICKET-407 crash fallout)
- **Status:** closed (2026-08-14 — shipped; pipeline `423-linux-portable-tests`, gate GREEN [diff]; box 2169/2169 with NO exclusions)

## Closed — 2026-08-14

All six members (plus two more the no-exclusion box run exposed —
`torture_command_round_trips_exactly` and
`workspace_two_real_sessions_are_independent`, the same fixed-window class as
member 5) fixed at source. Two ticket diagnoses were corrected by
design-phase probes: members 1–2 rode the macOS `/var` symlink, not
case-sensitivity (fix: `alias_twin_roots` builds its own symlink), and member
4 was rustc 1.94-vs-1.96 skew, not platform rendering (fix:
`rust-toolchain.toml` pin). Members 3+6 were REAL `marley_terminal` gaps —
see `AD-claude-423-pty-exit-order-independence-is-grace-plus-state-latch-001`.

**RIDER for Chad (the completion condition owned by the box lane):** the box
runner's `-E` exclusion filter and `NEXTEST_RETRIES=1` are now dead weight —
drop both from the runner (owned outside this repo; verified green without
them at `dev:~/builds/logs/423-verify.log`, snapshot 29be05c ≙ the shipped
commit). The box checkout was returned to `main` after delivery.

## Description

Full `cargo mutants` sweeps moved off the 16 GB Mac to the dev box (2026-08-13
policy; see TICKET-407 notes for the lane doc). The box-side runner excludes
four tests via a nextest `-E` filter because they are environment-sensitive on
Linux (2154/2158 pass otherwise):

1. `headless_drive::fold_entry_survives_alias_root_twin_instance_close_headless`
   — fixture needs two spellings of one directory; impossible on
   case-sensitive ext4.
2. `headless_drive::git_marks_entry_survives_alias_root_twin_instance_close_headless`
   — same fixture.
3. `marley_terminal` integration `write_after_disconnect_errors` — Linux PTY
   semantics differ.
4. `marley_text_offsets` compile_fail `ui_mixed_offsets_must_not_compile` —
   rustc/platform-sensitive snapshot.

While excluded, box results carry a blind spot: a mutant whose ONLY killer is
one of these four reads MISSED on the box (the 407 addendum cross-checks each
box-missed mutant against them; this ticket is the durable fix).

## Flaky candidate — 5th member (added 2026-08-13, sweep-restart fallout)

`marley` (marley_app) integration `echo_command_produces_a_block_with_its_output`
— real-zsh-over-PTY block test with a 4 s pump window (200 × 20 ms,
`crates/marley_app/tests/integration.rs:46`). NOT excluded by the lane filter;
it FLAKES instead: timed out once under lane load (2026-08-13 10:15 box
baseline → rc=4, killed a sweep restart) while passing ≥6 other box suite
runs, then 10/10 solo and 3/3 full-suite retests at lane conditions
(threads=8). Same PTY-timing class as (3). Two failure surfaces: an rc=4
baseline (recover: restore rotated seed, restart) and — rarer — false-CAUGHT
on a would-be-surviving mutant (hidden-survivor class the 407 cross-check
cannot see). Fix direction: deadline-based wait (~15 s ceiling; success still
exits early so green runs stay fast) rather than platform-gating. Evidence:
`dev:~/builds/logs/baseline-flake-20260813-101350.log`.

## Flaky candidate — 6th member (added 2026-08-13, second rc=4 baseline)

`marley_terminal::integration teardown_is_instant_when_child_already_reaped`
(`crates/terminal_blocks/tests/integration.rs:142`) — its FIRST assert
(`reaped.is_some()`) failed in 34 ms: the pump bailed before observing
`ChildExited` after `exit 0`. Read: the Linux PTY exit race — the child's
exit closes the PTY master (read → EIO) before the reap event surfaces —
i.e., the probabilistic sibling of excluded member (3), load/CPU-quota
modulated (fails under the box's systemd scope, 13/13 green unscoped).
**Triage note for design: this may be a REAL `marley_terminal` pump gap on
Linux** (EIO on the master should imply/queue a ChildExited-equivalent, or
the latch must be set from the reap path regardless of event delivery) —
decide production fix vs test tolerance. Evidence:
`dev:~/builds/logs/baseline-flake-20260813-103429.log`.

Lane mitigation since 2026-08-13 (Chad-approved runner amendment):
`NEXTEST_RETRIES=1` on the box masks this whole flaky class there (flaky →
retried → pass; real kills fail twice). Members 5–6 stay open here as the
durable fixes; the four excluded members still need their exclusions
dropped.

## The work

Make each portable rather than platform-gated where possible:
- Twin-spelling fixtures (1, 2): probe filesystem case-sensitivity at runtime
  and construct the alias-twin scenario accordingly; skip-with-reason ONLY if
  the semantics under test genuinely cannot exist on a case-sensitive FS (the
  current failure is fixture-construction, not feature absence — decide which
  at design).
- PTY semantics (3): assert the platform-appropriate write-after-disconnect
  error contract instead of the macOS-specific one.
- compile_fail snapshot (4): normalize or version/platform-tolerate the
  expected output.

Completion requires dropping the box-side `-E` exclusions — the runner is
owned by the dev-box lane (do-not-modify from this repo); coordinate with Chad
when the fixes land. Done = full suite green on BOTH platforms with no
exclusion filter.

Notes: headless_drive tests spawn a real zsh (the box has zsh installed).
References: TICKET-407 (mutation audit + lane doc in its notes), the
2026-08-13 handoff.
