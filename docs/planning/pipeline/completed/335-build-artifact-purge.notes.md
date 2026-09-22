# purge build artifacts at pipeline end + stop the cargo-mutants temp-tree leak — Notes

- **Forge ticket:** #335 (fc1a613d-3273-47ed-ab0c-83713de67b8b)
- **AAR:** PENDING — aar-open runs at /work promotion (Phase 1 drafted docs-only, no forge calls)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-335-build-artifact-purge.md
- **Pipeline spec:** 335-build-artifact-purge.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** Forge #335 — a PRE-EXISTING ticket (filed 2026-07-16 with measurements) adopted into
  sprint #35 "M24 — Fleet Layer 2" as a chore. Two halves: stop `cargo-mutants` stranding
  multi-GB copy-mode scratch trees in `$TMPDIR` on interrupt (3 orphans / 8.6G found 2026-07-16
  with no live process), and add a verified-safe-list purge script wired advisorily into the
  /commit closeout. Hard constraint from the ticket: `target/debug/deps` (~27G / 464k files) is
  the LIVE cache that keeps `cargo check` at ~7s and must NEVER be purged; cargo-sweep for that
  class is explicitly a separate future decision.
- **Classification / tier:** chore, M24, full pipeline — but a **GATE-IS-TEST ticket**
  (CONSTITUTION §7: bash/config only, no `.rs` → verification = gate exit codes + negative
  smokes, no unit tests to invent; no cov/MSI surface). Edit surface: `scripts/gates.sh`
  (`mutation_g` only), NEW `scripts/purge-build-artifacts.sh`, `.claude/commands/commit.md`
  (closeout step). No hooks, no crates, no gate-semantics changes.
- **Forge recall (§18.3):** drafted DOCS-ONLY (no MCP calls in this lane); recall drawn from the
  on-disk record — CONSTITUTION §0 (no weakening a gate; the receipt), §7 (gate-is-test), §15
  (anti-circumvention: REQ-007 pins that the trap can't re-thread a verdict); the gates.sh
  in-file history (#345 timeout-audit block sits inside `mutation_g` — the trap must not disturb
  it); the cross-platform-mutation note (macOS-only runner — this ticket is that one machine's
  disk). Live knowledge-context/bulletins recall re-runs at `/work` promotion.
- **Discovery (the sweep — what Phase 2 builds on):**
  - **`mutation_g` today (gates.sh:233-294):** `rm -rf mutants.out` (:236); DIFF mode
    early-returns before any child on an empty diff (:242-246); the child = bare subshell
    `( cargo mutants "${margs[@]}" >/dev/null 2>&1 ); rc=$?` (:249) — **no trap anywhere in
    gates.sh** → signal death strands tempfile's scratch (Drop never runs). rc semantics
    load-bearing: 0/2/3 completed, 1/4 fail-closed (:251-257). `--jobs 2` (:238) → up to 2
    scratch trees per run. Receipt written on FULL/DIFF green via `gate_state_hash` (:391-394) —
    the gates.sh edit invalidates prior receipts, a normal consequence, not a blocker.
  - **THE PUBLISHED-DOCS FINDING (cargo-mutants 27.1.0 `--help` + mutants.rs):** there is NO
    dedicated scratch-dir flag (`-d/--dir` = source dir, not scratch) — but the copy lands in
    "a temporary directory" via the tempfile crate → `std::env::temp_dir()` → **honors POSIX
    `$TMPDIR`**, confirmed empirically by the observed orphan naming
    `$TMPDIR/cargo-mutants-Marley-*.tmp`. **So the knob is `TMPDIR` itself**, settable
    per-invocation → scratch can be redirected under `target/` (e.g.
    `target/mutants-scratch/<pid>`), making trap discrimination exact and stranding any residue
    inside the purge script's domain. Safe because the book pins **`target/` is never copied
    by default** regardless of gitignore (`--copy-target` opt-in) → no recursion.
    (`--gitignore` filtering defaults off since 25.0.2 — moot given the target exclusion.)
    Also swept: `--in-place` (REJECTED — an interrupt would leave MUTATED SOURCE);
    `--leak-dirs` (proves normal-exit deletion is built in — the leak is interrupt-shaped);
    cargo-sweep (the published tool for the deps class — named + deferred, D5).
  - **Purge-list verification anchors:** `scripts/selftest/bundle-app.sh:12`
    `PROFILE="${1:-debug}"` → the bundle defaults DEBUG; `target/release` is ABSENT today and
    nothing missed it — regenerable, safe. `shellcheck_g` (gates.sh:111) globs `scripts/*.sh` →
    the new script is auto-bound to gate:11. `.gitignore` already covers `/mutants.out`,
    `/mutants.diff`, `mutants.out*/`; the in-target scratch is inside ignored `target/`.
  - **/commit seam:** `.claude/commands/commit.md` — gate → confirm-complete → stage → commit →
    (PR) → `## Closeout` (:26-28). The closeout is the AFTER-successful-commit seam; the wiring
    is a prompt-step (the skill is agent-interpreted), so REQ-006's verify is doc-review + a
    rehearsal transcript, honestly stated.
  - **Fresh measurements (2026-07-21, this machine):** target/debug **52G** (incremental **11G**),
    llvm-cov-target **6.7G**, doc 36M, tests 166M, mutants.out + .old 3.2M, release ABSENT,
    `$TMPDIR` orphans currently ZERO (cleaned since 2026-07-16), no live cargo-mutants process.
    The leak MECHANISM is unchanged (no trap); the growth since 2026-07-16 (34G → 50G+ in target)
    shows the purge classes compound.
- **Decisions:** D1 trap-at-source scoped to this invocation (TMPDIR-redirect leading;
  `--in-place` + blanket-trap-sweep rejected); D2 safe-list = exactly the ticket's verified
  table + double protection on deps; D3 pgrep live-guard; D4 advisory post-commit closeout,
  never Stop-hook/pre-gate; D5 cargo-sweep OUT; D6 dry-run parity by shared enumeration.
- **Open questions (D-OPEN, Phase 2 settles):**
  - **D-OPEN-SCRATCH** — final scratch placement: in-target TMPDIR redirect (leading) vs
    system-`$TMPDIR` + own-tree discrimination. Settle with ONE bounded interrupted probe run
    (confirm placement, no recursion, `--jobs 2` tree count, and that cargo-mutants tolerates a
    TMPDIR inside the workspace).
  - **D-OPEN-REFUSAL-SCOPE** — live-guard refusal: whole-run (lean; a live cargo-mutants means a
    gate is running, which also owns mutants.out + llvm-cov-target) vs mutants-class-only.
  - **D-OPEN-EXIT-SHAPE** — purge exit codes: always-0 vs honest nonzero for refused/failed
    (lean nonzero; the /commit wiring is `|| true`-shaped either way).
  - Trap mechanics detail: subshell-scoped vs function-scoped trap; EXIT-vs-RETURN semantics in
    bash 3.2; rc captured strictly before cleanup (REQ-007).
- **EARS:** REQ-001 no orphan on interrupt (counter-smoke proves the smoke bites); REQ-002
  dry-run/real parity + idempotence; REQ-003 live-process refusal; REQ-004 deps never touched +
  post-purge `cargo check` warm; REQ-005 purge failure never fails a commit; REQ-006 closeout
  invokes purge post-commit only (no Stop hook, no pre-gate); REQ-007 gate verdict byte-identical
  (trap is exit-code-transparent). Full table in the spec.
- **Forge ids:** ticket #335 = fc1a613d-3273-47ed-ab0c-83713de67b8b (sprint #35 "M24 — Fleet
  Layer 2"); pipeline_id = a57ef69a-d613-477a-ab89-abfeff417139; AAR pending at /work.

- **Promotion (/work 376-379,335, 2026-07-21):** queued→active, claimed + in-progress/plan, AAR
  `9bda4bbe`. Phase 1+2 fold — the queued spec's D-OPENs resolved: **D-OPEN-SCRATCH → the in-target
  TMPDIR redirect** (the drafter's key find: cargo-mutants 27.1.0 has no scratch flag but its
  tempfile copies honor POSIX `TMPDIR`; `target/` is never copied → no recursion), so orphans land
  in `target/mutants-scratch/<pid>` — inside the purge domain AND trap-cleanable with an exact name;
  **D-OPEN-REFUSAL-SCOPE → whole-run refusal** (any live cargo-mutants blocks the purge — simplest
  honest guard); **D-OPEN-EXIT-SHAPE → honest nonzero** (0 ok/nothing · 2 refused · 3 deny-guard
  bug) with the `|| true` absorption at the ONE advisory call site. Phase 1+2 PASS.

## Phase 2 — Design
- Architecture / approach; file manifest; regression test plan; risks.

- (Folded above.)

## Phase 3 — Implement
- **gates.sh `mutation_g`:** the scratch dir `target/mutants-scratch/$$`; `TMPDIR="$PWD/$scratch"`
  on the cargo-mutants invocation; normal-path `rm -rf "$scratch"`; the interrupt belt = a
  single-quoted EXIT trap naming exactly this pid's dir ($$ is shell-lifetime-constant — SC2064
  clean) + `trap 'exit 130' INT` / `'exit 143' TERM` so a signal EXITS (an INT trap that doesn't
  exit would swallow Ctrl-C and resume the gate — self-caught while writing). Exit-code semantics of
  the gate untouched (the trap only ever runs `rm -rf`).
- **NEW `scripts/purge-build-artifacts.sh`:** the verified safe-list (mutants-scratch,
  llvm-cov-target, release, trybuild, incremental, doc, mutants.out*, mutants.diff) + legacy
  system-tmp `cargo-mutants-*` orphans (maxdepth 1); ONE enumeration path shared by `--dry-run` and
  the real run (parity by construction); the pgrep -x live-guard (exit 2); the `*debug/deps*`
  DENY-GUARD inside the removal loop (exit 3 — deps protected twice: absent from the list AND
  unremovable through any future list edit); du-failure tolerated (`|| kb=0`).
- **`.claude/commands/commit.md`:** the Closeout gains the advisory post-commit step with the
  load-bearing `|| true` and the never-before/instead-of-a-commit wording.
- shellcheck (the gate's exact flags) + `bash -n` both clean.
- **Status: Phase 3 — Implement PASS.**
- What was built; deviations from design (with reason).

## Phase 3.5 — Inspect
- **2 critics** (bash-correctness · safety/scope), BOTH probe-driven — the strongest inspect of the
  run. Ledger:
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | HIGH (bash critic, PROVEN LIVE) | The in-repo scratch (`target/mutants-scratch`) made every mutant-run test tempdir REPO-ANCESTORED — `discover_no_git_falls_back` asserts its tempdir has NO `.git` ancestor → the baseline fails inside every cargo-mutants run → exit 4 → gate:5 RED on the first real mutation run (proven with a live nextest run both ways). My "no recursion" reasoning had checked copy-recursion but missed the ancestry consequence entirely. | REAL | scratch relocated OUTSIDE the repo (`${TMPDIR:-/tmp}/marley-mutants-scratch/$$`); the probe re-run PASSES with the fixed shape; a REAL bounded cargo-mutants run through the exact redirect completed `ok Unmutated baseline` (4 mutants: 3 caught, 1 unviable) |
  | F2 | MED (both critics, probe-proven) | The deny-guard's `*debug/deps*` substring missed ANCESTORS — injecting `target/debug` into the list deleted deps transitively while still printing "deps untouched". | REAL | one case now covers name + every ancestor spelling; the contained-copy smoke trips rc=3 with deps intact; the trailer reworded to the structural claim |
  | F3 | MED (safety critic, probe-proven) | A misspelled `--dryrun` silently selected the REAL run (deleted the seeded dir, exit 0). | REAL | strict args: anything ≠ `--dry-run` → usage + exit 64 (smoked) |
  | F4 | MED (safety critic, probe-proven) | No repo-root assertion: a symlinked/sourced invocation resolved the safe-list against the WRONG tree (dry-run listed a seeded foreign `target/release`). | REAL | the gates.sh `BASH_SOURCE` prologue + a `Cargo.toml`+`crates/` root assertion (exit 65) |
  | F5 | MED (safety critic) | The live-guard covered only cargo-mutants; gates 3/4/14 own llvm-cov-target/doc/trybuild for minutes with no mutants process — a concurrent-session purge mid-gate rips a live dir (fail-closed spurious RED; concurrency proven real on this machine during the critique). | REAL | the refusal also matches a live `bash …scripts/gates.sh` (pid(s) named — folding the L6 nit); over-matching = refusal = the safe direction |
  | F6 | LOW ×4 | exit-code header incomplete (1 undocumented) · find matches FILES too · unconditional "deps untouched" trailer · the stale in-target comments after the F1 relocation. | REAL | header lists 0/1/2/3/64/65; `-type d`; the trailer states the structural claim; comments updated |
  | F7 | self-caught at validate | My pgrep compound died under `set -e` (a no-match pgrep killed the subshell before the second check — BOTH refusal smokes returned rc=1 silently). | REAL | per-command `\|\| true` inside the group; both smokes re-run green (rc=3 / rc=2) |
  | F8 | notes | Interrupt-time rm can race dying workers (self-heals via the purge) · "no recursion" is conditional on `--copy-target` staying default-off (help-text-verified) · the residual pgrep→rm TOCTOU bounded by the post-commit-only wiring. | ACCEPTED | recorded |
- **Clean surfaces (probe-verified by the critics):** trap semantics correct incl. the INT-must-exit
  chain and single-quoted invariant `$$`; bash-3.2 `set -u` empty-array hazard correctly dodged (the
  early exit precedes the only `"${targets[@]}"` expansion — negative-control-proven); every listed
  path's rebuildability named; the warm-check trio (`deps`/`.fingerprint`/`build`) entirely outside
  the removal set; the receipt hashes NOTHING the purge touches (a post-commit purge cannot
  invalidate a receipt); dry/real parity structural; `--copy-target` opt-in confirmed via help.
- **Status: Phase 3.5 — Inspect PASS.**
- Critics run; findings table (severity / finding / verdict); fixes.

## Phase 4 — Validate
- **Gate-is-test smokes (all REAL, all pasted in the transcript):** bad-flag → usage + rc=64 ·
  the trap-mechanism SIGINT probe (dir CLEANED, + the counter-smoke: WITHOUT the trap the dir
  remains — the smoke bites) · the deny-guard contained-copy probe (ancestor injected → rc=3 +
  message, deps intact) · the live-guard decoy (a compiled binary named cargo-mutants → rc=2 with
  the pid named; clean after) · the F7 self-caught set-e pgrep death (both smokes silently rc=1 →
  fixed with per-command `|| true` → re-run green) · **the tolerance run: a REAL bounded
  cargo-mutants run through the exact TMPDIR redirect — `ok Unmutated baseline`, 4 mutants (3
  caught, 1 unviable), scratch empty after + removed** (the F1 HIGH verified fixed at the real
  seam) · **the REAL purge: ~19.4 GB reclaimed across 7 items, dry↔real byte-identical, second
  dry = "nothing to reclaim" (idempotent), `target/debug/deps` 586,360 entries before == after,
  and `cargo check --workspace` still 7.7s WARM** (the REQ-004 regression proof).
- **Gate `--diff`: GREEN 15/15** — the run itself exercised the new mutation_g path (scratch mkdir
  + traps + the no-crate-diff arm + EXIT-trap cleanup) AND gate:4 REBUILT the just-purged
  llvm-cov-target — rebuildability proven in the same pass. The full interrupt-mid-REAL-mutation
  integration is standing-exercised by every future .rs ticket's gate (the mechanism itself is
  smoke-proven above).
- **Status: Phase 4 — Validate PASS.**

- Tests RUN (with counts) + gate result; negative smokes; pre-existing notes.

## Phase 5 — Complete
- **Docs:** CHANGELOG (first under Added); the /commit skill carries the wiring (part of the diff).
- **Knowledge:** BF-claude-in-repo-tmpdir-breaks-repo-ancestry-tests-001 +
  PR-claude-relocated-tmpdir-changes-every-tempdirs-ancestry-001; AAR 9bda4bbe submitted.
- **Ticket:** #335 closed (done) + ship comment; local doc → closed; pair → completed/.
- Lessons: the probe-driven critics were the run's strongest (a HIGH proven live BOTH ways before
  any fix); "checked copy-recursion" ≠ "checked ancestry" — a relocated tmpdir changes every
  tempdir's parent chain; and my own refusal smoke caught my own set-e bug (F7) — the smokes
  earn their keep even against the fix's author.

- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
