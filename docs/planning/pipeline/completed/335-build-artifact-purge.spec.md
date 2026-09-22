---
pipeline_id: a57ef69a-d613-477a-ab89-abfeff417139
ticket: forge#335 (fc1a613d-3273-47ed-ab0c-83713de67b8b) · local docs/planning/tickets/open/TICKET-335-build-artifact-purge.md
aar_id: PENDING — aar-open runs at /work promotion (this Phase-1 draft was produced docs-only, no forge calls)
status: Phase 5 — Complete PASS
title: purge build artifacts at pipeline end — and stop cargo-mutants leaking multi-GB temp trees
type: chore
milestone: M24
references:
  - scripts/gates.sh
  - scripts/selftest/bundle-app.sh
  - .claude/commands/commit.md
  - CONSTITUTION.md
---

## Title
Build-artifact hygiene, two halves: **(1) fix the leak at source** — `mutation_g()`
(scripts/gates.sh:233) runs `cargo mutants` in a bare subshell (:249) with NO trap, so an
interrupted run (INT/TERM, machine sleep-kill, agent stop) strands the copy-mode scratch trees
cargo-mutants builds in the system temp dir (`$TMPDIR/cargo-mutants-Marley-*.tmp`; the tempfile
crate's Drop never runs on signal death). Measured 2026-07-16: **3 orphans totalling 8.6G with no
live process**. **(2) a purge script + advisory /commit wiring** — `scripts/purge-build-artifacts.sh`
reclaims the VERIFIED-safe artifact classes at rest, with `--dry-run`, a live-process guard, and a
hard never-touch guard on `target/debug/deps` (the ~27G/464k-file live cache that keeps
`cargo check` at ~7s — purging it would cost every subsequent pipeline minutes per gate). The
/commit skill's closeout invokes the purge AFTER a successful commit, advisory + non-fatal.
Re-measured 2026-07-21 while drafting: `target/debug` is now **52G** (11G of it
`debug/incremental`), `llvm-cov-target` 6.7G — the problem compounds; the ticket's classes hold.

## Scope
### In
- **(1) LEAK FIX — scripts/gates.sh `mutation_g()` only.** Per-invocation scratch discipline +
  cleanup on EXIT/INT/TERM scoped to THIS invocation's trees (D1). Leading mechanism (D-OPEN-SCRATCH,
  Phase 2 verifies empirically): redirect the scratch location by exporting a per-invocation
  `TMPDIR` under `target/` (e.g. `target/mutants-scratch/<pid>/`) for the `cargo mutants` child —
  cargo-mutants has NO dedicated scratch-dir flag (27.1.0 `--help` swept; prior-art leg 2) but
  places its copy via the tempfile crate → `std::env::temp_dir()` → honors POSIX `$TMPDIR`, and the
  published docs pin that **`target/` is never copied into the tree copy by default** (no recursion
  risk). Cleanup then `rm -rf`s exactly the dir this invocation created — exact discrimination, no
  pattern-matching against strangers. Exit-code transparency is binding (REQ-007): the 0/2/3
  threshold semantics, the 1/4 fail-closed arm (:251-257), and both early-return diff paths
  (:242-246, :259) stay byte-identical.
- **(2) NEW `scripts/purge-build-artifacts.sh`.** Safe-list = EXACTLY the ticket's verified table
  (D2): orphaned cargo-mutants scratch trees (system `$TMPDIR/cargo-mutants-*` AND the new
  in-target scratch dir), `target/llvm-cov-target`, `target/release` (bundle-app.sh:12 defaults
  `PROFILE=debug`; `target/release` is ABSENT today and nothing missed it — proven regenerable),
  `target/tests/trybuild`, `target/debug/incremental`, `target/doc`, `mutants.out*` (repo root,
  gitignored). Flags/behavior: `--dry-run` reports the same paths+bytes the real run reclaims
  (REQ-002, one shared enumeration path — D6); refuses the mutants-tree class while a
  `cargo-mutants` process is alive (`pgrep`-based, D3); **never** touches `target/debug/deps`
  (absent from the list AND an explicit deny-guard, REQ-004); per-path + total byte report;
  shellcheck-clean (`gate:11` globs `scripts/*.sh` — gates.sh:111 — so the new file is auto-bound).
- **(3) /commit WIRING — `.claude/commands/commit.md` closeout.** One advisory step in the
  existing `## Closeout` section (commit.md:26-28), invoked AFTER a successful commit, non-fatal
  shape (`scripts/purge-build-artifacts.sh || true`-style) — a purge failure or refusal NEVER
  fails the commit (REQ-005/006). Deliberately NOT a Stop hook, NOT pre-gate (D4).
- **(4) Receipt consequence, named:** editing gates.sh changes the worktree the
  `gate_state_hash` receipt binds (:391-394); any prior receipt is invalidated and the gate re-runs
  before /commit — a normal consequence of touching the gate, not a blocker.

### Out (explicitly deferred)
- **`target/debug/deps` (~27G) and any cache-trimming of it — cargo-sweep is a SEPARATE future
  decision** (the published tool for that class; deliberately not adopted here — D5).
- Any Stop-hook / pre-gate / scheduled (launchd) automation of the purge — advisory post-commit only.
- Unverified purge candidates: `target/flycheck0` (rust-analyzer), `target/tmp` (empty),
  `target/Marley.app` (bundle-app.sh rebuilds it anyway), `~/.cargo` registry/git caches — future
  table entries once verified, refused by this script until then.
- Changing any gate SEMANTICS: `--jobs 2`, mutation scope, MSI floors, exit-code thresholds, the
  `#345` timeout audit — all verdict-affecting behavior is untouched (hygiene only).
- CI/runner-fleet disk management (the macOS-only runner IS this machine; nothing remote).

## Reference (§20)
**N/A — Marley-specific build tooling.** No reference-app behavior to match: this is repo-local
gate/disk hygiene (bash + cargo tooling), not terminal/editor UX. Clean-room §20 untouched — no
Warp (AGPL) / Zed (GPL) source is relevant or consulted.

### Prior art
1. **Behavior maps — N/A.** docs/warp_architecture/ + docs/zed_architecture/ map product behavior
   (terminal, editor, cockpit); neither maps a build-artifact-hygiene surface. Stated per the sweep
   rule.
2. **Published — cargo-mutants 27.1.0 (installed; `--help` + mutants.rs book):**
   - **No dedicated scratch-dir flag exists.** `-d/--dir` is the SOURCE crate dir ("Create
     mutants.out within this directory"), not the scratch location. The book: "By default,
     cargo-mutants copies your tree to a temporary directory before mutating and building it" —
     placement is the tempfile crate's default = `std::env::temp_dir()` = **POSIX `$TMPDIR`**,
     which the observed orphan naming (`$TMPDIR/cargo-mutants-Marley-*.tmp`) empirically confirms.
     → **The knob is `TMPDIR` itself**, settable per-invocation in `mutation_g` — this is the
     finding that simplifies the trap (D1/D-OPEN-SCRATCH).
   - **`target/` is excluded from the copy by default** "regardless of gitignore settings"
     (`--copy-target` is opt-in) → pointing `TMPDIR` under `target/` cannot recurse. (`--gitignore`
     filtering defaults off since 25.0.2 — irrelevant given the unconditional target exclusion,
     noted for completeness.)
   - **`--in-place` exists and is REJECTED** (D1): it mutates the REAL source tree — an interrupt
     mid-mutant leaves MUTATED SOURCE in the working tree, strictly worse than a leaked temp copy.
   - **`--leak-dirs`** ("Don't delete the scratch directories, for debugging") proves normal-exit
     deletion is built in — the leak is interrupt-shaped, exactly what a trap addresses.
   - Exit semantics 0/2/3 (completed) vs 1/4 (invalid) are already load-bearing in `mutation_g`
     (:251-257) and must survive the trap unchanged (REQ-007).
   - **cargo-sweep** = the published tool for the deps/ class — named and explicitly DEFERRED (D5).
3. **OUR OWN repo (the seams this ticket edits):**
   - `mutation_g()` (scripts/gates.sh:233-294): `rm -rf mutants.out` up front (:236); DIFF mode
     early-returns BEFORE any cargo-mutants child on an empty diff (:242-246); the child runs in a
     bare subshell `( cargo mutants "${margs[@]}" >/dev/null 2>&1 ); rc=$?` (:249) — **no trap
     anywhere in gates.sh** (grep) → signal death strands the tempfile scratch. rc is captured
     immediately; cleanup must slot after capture (REQ-007). `--jobs 2` (:238) means up to 2
     scratch build dirs per run — consistent with multi-orphan observations.
   - Receipt: `gate_state_hash > "$GITDIR/ignibyte-gate-receipt"` on FULL/DIFF green (:391-394);
     `enforce-commit-gate.sh` replays it — the gates.sh edit invalidates old receipts (normal).
   - `shellcheck_g` (:111) runs `shellcheck -S info -e SC1091 .claude/hooks/*.sh scripts/*.sh` —
     the new purge script is inside the glob, auto-bound to gate:11.
   - `scripts/selftest/bundle-app.sh:12` — `PROFILE="${1:-debug}"` → the self-test bundle builds
     DEBUG by default; nothing requires `target/release` to persist (and it is absent today).
   - `/commit` skill (.claude/commands/commit.md): gate → confirm-complete → stage → commit → PR →
     `## Closeout` (:26-28) — the closeout IS the post-successful-commit seam (D4).
   - `.gitignore` already covers `/mutants.out`, `/mutants.diff`, `mutants.out*/`; the new scratch
     lives under already-ignored `target/` — no gitignore edit needed.
   - Fresh measurements (2026-07-21): target/debug 52G (deps dominant + incremental 11G),
     llvm-cov-target 6.7G, doc 36M, tests 166M, mutants.out+.old 3.2M, release ABSENT, zero
     `$TMPDIR` orphans right now (cleaned since 2026-07-16), no live cargo-mutants process.

## Locked-In Decisions
- **D1 — Fix the leak AT SOURCE, in `mutation_g`, via per-invocation scratch discipline + an
  EXIT/INT/TERM cleanup scoped to THIS invocation's trees.** Cleanup discriminates by ownership —
  it removes only what this invocation created, never a concurrent run's live tree. **Leading
  mechanism (D-OPEN-SCRATCH):** per-invocation `TMPDIR=target/mutants-scratch/<pid>` for the
  cargo-mutants child (published leg: tempfile honors TMPDIR; target/ never copied → no recursion),
  making discrimination exact (`rm -rf` the one dir we created) and stranding any residue inside
  the purge script's domain as defense-in-depth. **Alternative kept open:** system-`$TMPDIR` +
  own-tree discrimination (mtime/pid matching) if the redirect probe misbehaves. **Rejected:**
  `--in-place` (interrupt leaves mutated SOURCE); a blanket `rm -rf $TMPDIR/cargo-mutants-*` in the
  trap (kills a concurrent run's live tree — that blanket sweep belongs to the purge script, where
  the live-process guard fronts it).
- **D2 — Purge safe-list = EXACTLY the ticket's verified table** (orphaned cargo-mutants trees,
  llvm-cov-target, release, tests/trybuild, debug/incremental, doc, mutants.out*) — nothing else,
  however tempting; unverified candidates are Out. `target/debug/deps` is protected twice:
  structurally absent from the list AND an explicit deny-guard refusing any path under it.
- **D3 — Live-process guard = `pgrep`-based cargo-mutants detection.** While a cargo-mutants
  process is alive the purge refuses AT LEAST the mutants-tree class, with a message naming why
  (REQ-003). **D-OPEN-REFUSAL-SCOPE:** whole-run refusal (lean — a live cargo-mutants means a gate
  is running, which also owns `mutants.out` and `llvm-cov-target`) vs class-scoped refusal.
- **D4 — /commit wiring = one advisory closeout step AFTER a successful commit, non-fatal
  (`|| true`-shape).** Deliberately NOT a Stop hook (fires far too often, and a hygiene failure
  must never block an agent stop), NOT pre-gate (the gate NEEDS warm artifacts; purging before it
  would slow the very loop this ticket protects). A purge failure is stderr-advisory (REQ-005).
- **D5 — cargo-sweep and the 27G deps class = OUT.** A separate future decision with its own
  verification; this ticket's contract is "never touch deps", full stop.
- **D6 — Dry-run parity by construction:** one enumeration path computes the candidate set
  (paths + sizes); `--dry-run` prints it, the real run deletes it — the two can't diverge except
  through filesystem drift between invocations (REQ-002's idempotence smoke bounds that).
- **D-OPEN-EXIT-SHAPE (Phase 2):** purge exit-code contract — always-0 vs distinct nonzero codes
  for refused/failed (lean: honest nonzero, since the /commit wiring is non-fatal by shape anyway
  and scripts deserve a truthful exit).

## Acceptance Criteria (EARS)
GATE-IS-TEST ticket (bash/scripts only, no `.rs`): per CONSTITUTION §7, verification = the gate's
own exit codes + **negative smokes** (inject the drift → red/dirty; revert → green/clean), not
invented unit tests.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a `mutation_g` cargo-mutants run is interrupted (INT/TERM) — or exits by any path — the invocation shall leave no orphaned cargo-mutants scratch tree, neither in the system `$TMPDIR` nor under the in-target scratch dir. | Negative smoke: launch a bounded mutation run, SIGINT it mid-build → assert zero `cargo-mutants-*` residue in both locations; counter-smoke with the fix stashed → orphan APPEARS (proves the smoke bites) → fix restored → clean. Normal-exit: post-green-gate assert clean. |
| REQ-002 | WHEN `purge-build-artifacts.sh --dry-run` runs, it shall report exactly the paths and byte totals the subsequent real run then reclaims; and WHEN the real run is repeated immediately, the second run shall reclaim ~0 (idempotent). | Smoke: seed known-size disposable artifacts into safe-list locations → capture `--dry-run` report → real run → diff report vs reclaimed set (paths + sizes match) → immediate re-run reports/reclaims ~0. Exit codes per D-OPEN-EXIT-SHAPE. |
| REQ-003 | WHILE a cargo-mutants process is alive, the purge shall refuse to remove any cargo-mutants tree (scope per D3) and shall report the live process as the reason. | Negative smoke: with a live name-faithful decoy (or a real bounded run) in flight → purge refuses the mutants class + names the pid; after the process exits, the identical invocation proceeds. |
| REQ-004 | WHEN the purge runs (any flags), it shall never remove or modify anything under `target/debug/deps`; and WHEN `cargo check --workspace` runs post-purge, it shall stay warm (seconds — zero external-dep recompiles). | Structural: deps absent from the safe-list + deny-guard smoke (feed a deps path → refused). Post-purge: a deps sentinel (inode/mtime sample) unchanged + `time cargo check --workspace` bounded with no `Compiling <external-dep>` lines. |
| REQ-005 | IF the purge fails or refuses (any nonzero exit), the /commit flow shall still complete successfully — the wiring is advisory and non-fatal. | Negative smoke: force a purge failure (injected unreadable/failing path) inside a rehearsed closeout → the commit flow's result is unchanged; review: the wiring step is `|| true`-shaped and cannot propagate. |
| REQ-006 | WHEN /commit completes a successful commit, its closeout shall invoke `scripts/purge-build-artifacts.sh` (advisory); the purge shall NOT be wired pre-gate, pre-commit, or into any Stop hook. | Review of the commit.md diff (step sits in `## Closeout`, after the commit step) + one closeout rehearsal transcript showing the invocation; negative: grep proves no purge reference in gates.sh's static path or `.claude/hooks/`. |
| REQ-007 | WHEN `mutation_g` completes — green (0/2/3), fail-closed (1/4), or either early-return diff path — the cleanup shall not alter the gate verdict: exit semantics byte-identical to today, and a cleanup failure shall be stderr-advisory, never a verdict change. | Negative smoke: force cleanup failure (read-only scratch parent) on a green bounded run → gate still green; a red-path run → still red; review pins rc captured BEFORE cleanup runs; DIFF-mode empty-diff early return still prints its pass message. |

## Floors (constitution)
No `.rs` → no cov/MSI surface; the binding floors are: gate:11 shellcheck `-S info` clean on the
new script + the gates.sh edit; gate:10 gitleaks; gate:14 docs; and the FULL/DIFF gate green
end-to-end (the receipt re-established after the gates.sh edit). §0 anti-circumvention applies:
the cleanup/purge may not weaken, reorder, or re-thread any gate verdict (REQ-007 is the pin).

## Phase Plan
- **P2 Design** — empirically probe D-OPEN-SCRATCH (one bounded interrupted run with the TMPDIR
  redirect; confirm placement + no recursion + `--jobs 2` tree count); settle
  D-OPEN-REFUSAL-SCOPE + D-OPEN-EXIT-SHAPE; exact trap placement in `mutation_g` (subshell-scoped
  vs function-scoped; rc-capture ordering); the purge script's enumeration fn + report format; the
  smoke-script plan per REQ (where each smoke lives, how the counter-smoke stashes the fix);
  commit.md closeout wording.
- **P3 Implement** — gates.sh `mutation_g` delta; NEW `scripts/purge-build-artifacts.sh`;
  commit.md closeout step. No `.rs`, no hooks, no gate-semantics changes.
- **P3.5 Inspect** — adversarial: can the trap eat or reorder rc? any path where the purge sees
  `target/debug/deps` (symlinks? `..` traversal? env-injected paths)? TOCTOU between pgrep and
  rm? concurrent-gate interaction? does the DIFF empty-diff early return still bypass cleanly?
  shellcheck posture; §20 provenance (trivially N/A but stated).
- **P4 Validate** — RUN the REQ-001..007 smokes for real (counter-smokes included — inject the
  drift, watch it bite, revert); `scripts/gates.sh` green (FULL or --diff; the mutation gate's
  own run doubles as the REQ-001 normal-exit assert); fresh receipt after the gates.sh edit.
- **P5 Complete** — CHANGELOG; note the new script in the repo docs where scripts are indexed;
  AAR (lesson candidates: TMPDIR-is-the-knob; trap-vs-rc ordering); archive; close forge #335.
