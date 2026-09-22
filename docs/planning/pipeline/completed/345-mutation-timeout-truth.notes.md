# gate:5 truth — mutants under nextest + a Timeout must prove it was a hang — Notes

- **Forge ticket:** #345 `8af264da-17ea-4974-8dd0-bceb389d6730`
- **AAR:** `7a4476ce-188c-41dc-ab1c-d72133650546`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-345-mutation-timeout-truth.md
- **Pipeline spec:** 345-mutation-timeout-truth.spec.md
- **pipeline_id:** `3ee9f0c7-9b0d-4cb0-97c5-d2b965da5854`

<!-- Working scratch. Excluded from gate:14 doc-todos. -->

## Phase 1 — Plan

- **Request:** gate:5 (mutation, MSI ≥ 100) counts a `Timeout` mutant as CAUGHT. Sound for a genuine
  hang (Infection/Stryker: a hang IS detection); UNsound when the timeout is just "the suite was slow",
  which launders a genuinely undetected mutant into MSI 100. Fix two ways: (D1) run mutants under nextest
  (`--test-tool=nextest` — fail-fast, process-per-test) so a killed mutant dies in seconds instead of
  riding to the deadline; (D2) audit each remaining `Timeout`'s `log_path` for failing tests and print
  `N timeout(s); M mislabeled` so a mislabel is VISIBLE, never silently absorbed.
- **Classification / tier:** work pipeline, `chore`, gate-is-test (D5 — `scripts/gates.sh` gate:5 only,
  NO `.rs`). One shippable slice.
- **Forge recall (§18.3):** bulletins EMPTY. `knowledge-context` (Plan) surfaced 13 nodes into the AAR —
  top PR `a7199501` (0.84 semantic) + PR `9ba80fa6` + AD `bb680dcc` (0.81) in the gate-integrity/§0
  neighborhood; distilled lessons b7eb5905/1ae8fd1a; recent failures dd799805/713a661b. Consult the top
  gate-integrity PR/AD at Design before touching the classifier.

### Re-verification of the spec's confident sentences (all on `d90ff9c`)

- **★ D3-RIDES-348 is FALSE → downgraded to D3-SYNERGY-NOT-DEPENDENCY (spec edited at promotion).** The
  spec + shelf note claimed #345 is "HARD-ordered after #348 (rides its `.config/nextest.toml`)". CLINCHING
  EVIDENCE: (1) `.config/nextest.toml` does NOT exist today; (2) gate:3 `tests_g` (gates.sh:80-82) already
  runs `cargo nextest run --workspace` on every gate and passes WITHOUT it → nextest needs no #348
  deliverable; (3) `cargo mutants --test-tool <cargo|nextest>` supported @ 27.1.0 (`--help`). So the runner
  swap + audit STAND ALONE; #348's terminate ceiling is additive (a per-test hang → nextest kill →
  honest `CaughtMutant`, shrinking the Timeout residue). The goal's 345-before-348 order is valid.
  **Shelf-note correction owed at Phase 5** (integrity-five-shelf.md:16 + 48-50 still say "HARD-ordered").

- **Doctest risk measured ZERO (re-measured, not assumed).** `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0
  cargo test --doc -p marley_lsp -p marley_editor` → `Doc-tests marley_lsp: running 0 tests` and
  `marley_editor: 0 tests`. nextest drops doctests, but there are none to drop. D4 holds: a future
  sole-killer doctest surfaces as a MISSED mutant (fails CLOSED — the §0-correct direction).

- **★ outcomes.json schema confirmed on a REAL retained artifact (`mutants.out.old/outcomes.json`).** Each
  `.outcomes[]` row carries `.summary`, `.log_path`, `.diff_path`, `.scenario` (+ `.phase_results`).
  Top-level carries `caught`/`missed`/`timeout`/`unviable`/`total_mutants` counts and `.success`.
  Summaries seen: `CaughtMutant`, `MissedMutant`, `Timeout`, `Success` (the baseline), `Unviable`. The
  gate:5 jq (`CaughtMutant`|`Timeout` = caught; `MissedMutant` = missed) is correct; `Success`/`Unviable`
  are excluded from both — unchanged.
  - **`log_path` is RELATIVE to `mutants.out/`** (not CWD, not absolute). The retained Timeout row's
    `log_path` = `log/crates__syntax__src__fold.rs_line_75_col_16.log`, and the file exists at
    `mutants.out.old/log/…`. **→ the audit must grep `mutants.out/$log_path`.** (Design must resolve the
    dir once; a future cargo-mutants `--output` would move it, so read the dir from the same place the
    gate already writes it.)
  - **A real MISLABEL is in that retained output** — proof of the whole thesis beyond the #337 hand-audit.
    The Timeout row is the `fold.rs:75:16 delete-! in collect_fold_regions` mutant; grepping its log finds
    `test fold::tests::fold_regions_all_seven_kinds_document_order ... FAILED`. It was CAUGHT by a failing
    assert but labeled `Timeout` because plain `cargo test` didn't fail-fast. Under `--test-tool=nextest`
    this mutant dies in seconds as `CaughtMutant` — the swap fixes exactly this. The audit is the belt for
    any residue the runner swap leaves.

- **REQ-002 target still valid.** `crates/marley_app/src/font_zoom.rs` exists on `d90ff9c` — a small pure
  file, viable as the synthetic `--in-diff` clean-reclassify smoke target. (Note: it lives in `marley_app`,
  which is coverage-excluded but mutation-INCLUDED, so its mutants are real gate:5 targets.) No substitution
  needed. If a P4 run shows its set is slow, fall back to a `crates/syntax` or `crates/editor` pure fn file.

### For Phase 2 (do NOT resolve at Plan)

- **`--jobs 2` × nextest's own parallelism = CPU oversubscription** (the #334 load-flake family, exit 4
  under load). Decide at P2 whether to pin nextest test-threads for the mutants baseline (e.g.
  `--test-tool nextest` + a bounded thread count) — **WITH a measured flaky run as evidence**, not
  preemptively. The exit-4 case already fails CLOSED (gates.sh:247-251), so a flake is a red gate, not a
  laundered pass — but a flaky gate is its own harm.
- **The audit's defensive shape (P3.5 lens):** a `Timeout` row whose `log_path` file is MISSING/unreadable
  must NOT crash the gate — the audit degrades to "counted, log unavailable", never a non-zero from `grep`.
- **The commit receipt:** editing `scripts/gates.sh` STALES the `gate_state_hash` (the hook lib counts
  `scripts/*.sh` as bar-defining). Phase 4 MUST run the MODIFIED gate to mint a fresh receipt — and that
  run IS the REQ-002/004 proof. The #350 docs-don't-stale shortcut does NOT apply here.

### Decisions confirmed (spec)

D1-NEXTEST-RUNNER, D2-AUDIT-NOT-RECLASSIFY, D4-FAIL-CLOSED-ON-DOCTEST-DRIFT, D5-GATE-IS-TEST — all hold as
written. D3 downgraded (above). EARS REQ-001…006 confirmed against the live gates.sh (`:236` margs,
`:247-251` exit contract, `:258-259` the Timeout∈caught jq). §20 = N/A (Marley gate integrity; no
reference-app analog).

### Prior art (§20 sweep)
1. **Published** — cargo-mutants documents `--test-tool nextest` as the supported runner swap; the
   `Timeout∈caught` stance is the Infection/Stryker MSI convention (survives; gains an audit).
2. **OUR OWN CODE** — gate:3 (`tests_g`, gates.sh:74-83) has run nextest + a separate `cargo test --doc`
   since inception; gate:5 joining nextest CONVERGES the runners rather than adding one. The #337
   log-reading that found the mislabels is the audit, done by hand once — this ticket automates it.
3. Checked gpui/ropey/regex — not their seam (no owner).

## Phase 2 — Design

**Architecture / approach.** No Marley runtime is touched — this is the test *bar*, not the app.
`scripts/gates.sh`'s `mutation_g()` (lines 228-270) is gate:5. The full function, verified on `d90ff9c`:
the base `margs=( --jobs 2 --no-times )` (:233); diff mode prepends `--in-diff mutants.diff` (:242); the
run at :244 (`rc=$?`); the exit-code contract 0|2|3 (:249-252); the `outcomes.json` exists-check with
diff-mode early return (:253-256); the `caught`/`missed`/`total`/`msi` arithmetic (:257-269). **Shell
posture: `set -uo pipefail`, NO `errexit`** (gates.sh:35) — a mid-function non-zero does NOT abort; only
an explicit `|| return 1` fails the gate. This is what makes the defensive audit safe.

§20 = N/A (Marley gate integrity; no reference-app analog) — confirmed. Prior art = our own gate:3
nextest/doctest split (gates.sh:74-83) that gate:5 now converges onto.

**File manifest (ONE file, no `.rs`, no new file):**
- `scripts/gates.sh` — (a) `--test-tool=nextest` into the `margs` base array (:233); (b) a Timeout-audit
  block inserted after the `outcomes.json` exists-check (after :256); (c) the :258 comment gains the
  audit's rationale.

### The exact edits

**(a) D1 — the runner (line 233):**
```
  local -a margs=( --jobs 2 --no-times --test-tool=nextest )
```
One base-array add covers BOTH modes: diff prepends `--in-diff mutants.diff` to `"${margs[@]}"` (:242),
so nextest carries into full and diff alike. Nothing else in margs changes.

**(b) D2 — the Timeout audit, inserted immediately AFTER line 256 (the `fi` of the outcomes-exists
check), BEFORE `local caught missed total msi`:**
```
  # ── #345 Timeout truth audit (additive; changes no count) ────────────────────
  # A Timeout is COUNTED caught by the convention below. But a Timeout whose log shows a
  # FAILED test was caught by ASSERT and merely blew cargo-mutants' wall clock (not a hang).
  # Surface that mislabel instead of absorbing it. --test-tool=nextest (fail-fast) makes it
  # rare at the source; this is the belt for any residue. log_path is RELATIVE to mutants.out/.
  local timeouts=0 mislabeled=0 lp
  while IFS= read -r lp; do
    [ -z "$lp" ] && continue
    timeouts=$((timeouts + 1))
    if [ -f "mutants.out/$lp" ] && grep -qE 'FAILED|panicked|assertion' "mutants.out/$lp"; then
      mislabeled=$((mislabeled + 1))
    fi
  done < <(jq -r '.outcomes[]|select(.summary=="Timeout")|.log_path' mutants.out/outcomes.json)
  if [ "$timeouts" -gt 0 ]; then
    echo "mutation: ${timeouts} timeout(s); ${mislabeled} mislabeled (log shows failing tests — caught-by-assert, perf not detection)"
  fi
```
Placement rationale: after :256 means both diff-mode early returns (`:239` no changed crate lines, `:254`
no mutable lines) return BEFORE the audit — it runs only on a real completed run with an `outcomes.json`.
It reads only; it prints one optional line; it never touches `caught`/`missed`/`total`/`msi` (REQ-006).

**Exactly how grep's exit code is neutralized (the P3.5 defensive contract):**
- `grep -qE` non-match returns 1, but it sits in an `if [ -f … ] && grep …; then` — the `if` **consumes**
  that exit status; with no `errexit`, nothing aborts.
- The `&&` short-circuits: if `[ -f "mutants.out/$lp" ]` is false (log MISSING/unreadable), grep is never
  run — the Timeout is still counted, just not classified mislabeled. No crash. (REQ-003 missing-log case.)
- The loop is fed by **process substitution `< <(jq …)`, NOT `jq … | while`** — so the `while` runs in the
  CURRENT shell and `timeouts`/`mislabeled` survive (a pipe would subshell them away to 0). `pipefail` has
  no pipe to bite here.
- `set -u`: `timeouts`/`mislabeled`/`lp` are all initialized before use.
- After the block, `$?` is 0 (a not-taken `if` yields 0; a taken one ends on a successful `echo`), so the
  following `local …` and the arithmetic proceed unaffected.

**(c) The :258 comment** keeps its first line and gains a back-reference:
```
  # Timeout counts as CAUGHT — Infection/Stryker MSI convention: a hang IS detection.
  # The #345 audit above surfaces the mislabel case (a Timeout whose log shows a FAILED test
  # was caught by assert, not a hang); --test-tool=nextest fails fast so that case is now rare.
  # The convention and the MSI arithmetic are unchanged — the audit is visibility only.
```

### The proof base is REAL (REQ-002/003 evidence)
`mutants.out.old/outcomes.json` holds a live mislabel: the `crates/syntax/src/fold.rs:75:16 delete-! in
collect_fold_regions` mutant is `summary:"Timeout"`, yet its log
(`mutants.out.old/log/crates__syntax__src__fold.rs_line_75_col_16.log`) contains
`test fold::tests::fold_regions_all_seven_kinds_document_order ... FAILED`. Caught by assert, laundered by
the wall clock — exactly #337. Under `--test-tool=nextest` (fail-fast) it dies in seconds as `CaughtMutant`.
This is the audit's real-world regression fixture (REQ-003 can reuse this exact log shape).

### font_zoom.rs target confirmed (REQ-002/004)
`cargo mutants --list -f crates/marley_app/src/font_zoom.rs` → ~12 mutants across two pure fns
(`clamp_font_size`, `zoom_step`) — arithmetic/comparison/negation only, fast to kill under nextest. Ideal
small deterministic target for the clean-reclassify (REQ-002) and the neuter-an-assert MISSED smoke
(REQ-004).

### ★ The oversubscription measurement (`--jobs 2` × nextest threads) — MEASURED, not assumed
Probe: `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 cargo mutants -f crates/marley_app/src/font_zoom.rs
--jobs 2 --no-times --test-tool=nextest` (a faithful proxy — the runner/parallelism behavior is identical
whether mutants are selected by `-f` or `--in-diff`).

**RESULT (measured 2026-07-18, on `d90ff9c`):**
```
Found 12 mutants to test
ok       Unmutated baseline
12 mutants tested: 12 caught
PROBE EXIT rc=0        (total run ~2.7 min incl. baseline build)
outcomes: total=12 caught=12 missed=0 timeout=0 unviable=0
```
- **rc=0 — NO exit-4 flake.** `--jobs 2` × nextest's own thread pool did NOT oversubscribe into the #334
  load-flake family. The baseline passed under nextest, and all 12 mutants completed cleanly.
- **DECISION: pin NOTHING** (§0 — no preemptive complexity). No `NEXTEST_TEST_THREADS`, no `--test-threads`
  cap. The margs stay `( --jobs 2 --no-times --test-tool=nextest )`. If a future load-heavy run flakes
  exit 4, that fails CLOSED (gates.sh:247-251 — a red gate, not a laundered pass), and the pin can be
  added THEN, with that run as evidence. It is not needed now.
- **Doubles as REQ-002 evidence:** font_zoom.rs's full 12-mutant set → all `CaughtMutant`, **0 Timeouts**,
  in ~2.7 min. That is the "reclassify honestly, 0 Timeouts, faster than the recorded slow run" proof for
  the clean case. (The `--in-diff`-specific path is re-exercised by the P4 modified-gate run.)

### Regression Test Plan (all shell exit-code / smoke — gate-is-test, NO unit tests)
| # | Proof | How |
|---|---|---|
| REQ-001 | mutants run under nextest, both modes | read the `margs` line; the P2 probe's cargo-mutants output names the nextest runner; a P4 diff-mode gate run confirms |
| REQ-002 | #337 trio reclassifies honestly | the P2 `-f font_zoom.rs` probe (and a P4 `--in-diff` gate run) → full set `CaughtMutant`, 0 Timeouts, faster than the recorded slow run |
| REQ-003 | `N timeout(s); M mislabeled` printed from log_path; zero-timeout prints nothing; missing-log is safe | shell smoke: feed the audit a synthetic `mutants.out/outcomes.json` + a `log/…` fixture (one Timeout row whose log has `FAILED` → `1 timeout(s); 1 mislabeled`; a no-Timeout json → no extra line; a Timeout row whose log file is absent → `1 timeout(s); 0 mislabeled`, no crash) |
| REQ-004 | a genuinely MISSED mutant still fails the gate red | neuter one `font_zoom` assert so a mutant survives → gate:5 (diff-scoped) exits non-zero; restore |
| REQ-005 | exit 0/2/3 contract + fail-closed 1/4 byte-identical | diff review (:249-252 untouched) + REQ-002 (exit 0) and REQ-004 (exit 2) runs |
| REQ-006 | `MUT_MSI_MIN` + MSI arithmetic untouched | diff review — the only additions are the margs token + the audit block; :259-269 unchanged |

**Commit-receipt fact (P4):** editing `scripts/gates.sh` STALES the `gate_state_hash` (hook lib counts
`scripts/*.sh` as bar-defining), so P4 MUST run the MODIFIED `--diff` gate to mint a fresh receipt — that
run IS the REQ-001/002/005 proof. The #350 docs-don't-stale shortcut does NOT apply.

### Risks / decisions
- **Oversubscription** — resolved by measurement (above), not assumption. Default stance: pin nothing
  unless the probe flakes (§0).
- **The audit is reporting-only** — it can never *raise* MSI or hide a MISSED; the worst a bug in it can
  do is misprint the advisory line, which a P4 smoke pins. It carries no floor.
- **syspolicyd** — the probe + P4 gate are real mutation runs; a silent 0%-CPU stall is the OS, not the
  code (`ps -o %cpu -p $(pgrep syspolicyd)`), self-clears, never weaken the gate.

## Phase 3 — Implement

Three edits to `scripts/gates.sh` `mutation_g()`, exactly as designed — NO `.rs`, no other file:
1. **margs (line 233):** `( --jobs 2 --no-times )` → `( --jobs 2 --no-times --test-tool=nextest )`. One
   base-array add; diff mode prepends `--in-diff` to it (:242), so nextest carries into both modes.
2. **The Timeout audit block** inserted after the `outcomes.json` exists-`fi` (after :256), before
   `local caught missed total msi`. Process substitution `< <(jq …)` (counters survive), `grep -qE`
   inside `if [ -f … ] && grep …` (non-match exit-1 consumed; missing log skipped, no crash), prints the
   `N timeout(s); M mislabeled` line only when `timeouts>0`. Reads only; touches no count.
3. **The :258 comment** kept its first line + gained a 3-line back-reference (audit-above; convention +
   MSI arithmetic unchanged, visibility only).

**Verification:** `bash -n scripts/gates.sh` OK. Gate:11's exact invocation
`shellcheck -S info -e SC1091 .claude/hooks/*.sh scripts/*.sh` → CLEAN (the sole SC1091 info on the
pre-existing `source` at :42 is excluded by the gate's own `-e SC1091`; my edits added zero findings).
`git diff scripts/gates.sh` = exactly the three edits; the `caught`/`missed`/`total`/`msi` arithmetic
(:259-269 pre-change) and the `0|2|3` exit contract (:249-252) are byte-identical (REQ-005/006 hold by
construction). No mutation run here (Phase 4 owns it — the P2 probe already proved the runner: rc=0,
12/12 caught, 0 timeout).

**Deviations from design:** none.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**No findings.** One general-purpose shell-correctness critic + my own concrete six-lens review, both
over `git diff scripts/gates.sh`. The critic returned **"All checks confirmed"** (I stopped it while it
was doing its final scratch-cleanup — its checks had all passed; the tree was verified clean afterward,
no leftover scratch files, mirroring the #307 lesson). My own review verified each lens with a run, not
an assertion:

| Lens | Verdict | Evidence |
|---|---|---|
| (a) counter survival | PASS | the loop is `done < <(jq …)` (gates.sh:269) — process substitution. Reproduced: process-sub form yields `t=3`, the `jq \| while` pipe form yields `t=0` (subshell). Counters `timeouts`/`mislabeled` init before the loop, survive to the `if [ "$timeouts" -gt 0 ]` test. |
| (b) grep exit-code | PASS | `set -uo pipefail`, NO errexit (gates.sh:35). `grep -qE` sits in `if [ -f … ] && grep …; then` → its non-match exit-1 is consumed. Reproduced both the missing-file (short-circuit, grep never runs) and the no-match (grep returns 1) cases → both "survived", no abort. |
| (c) placement | PASS | the block is after the `outcomes.json` exists-`fi`, so both diff-mode early `return 0`s skip it; it runs only on a completed run with an `outcomes.json`. A malformed json makes `jq` emit nothing → zero iterations → no report (and the pre-existing `caught=$(jq …)` handles malformed json exactly as before — no NEW failure mode). |
| (d) no-arithmetic-change (REQ-006) | PASS | `diff` of the `caught=`/`missed=`/`total=`/`msi=`/floor lines between `HEAD:scripts/gates.sh` and the working copy → **byte-identical** (only line numbers shifted +19). The `MUT_MSI_MIN` clamp (:54/:56) unchanged. The audit changes no count and cannot alter pass/fail. |
| (e) margs both modes | PASS | `--test-tool=nextest` is in the BASE `margs` array (:233); diff mode does `margs=( --in-diff mutants.diff "${margs[@]}" )` (:242) → nextest carries into both. No other margs element changed. |
| (f) log_path base | PASS | the audit greps `mutants.out/$lp`; the retained `mutants.out.old/outcomes.json` proves `.log_path` = `log/….log` (relative to the mutants output dir), so `mutants.out/$lp` is the correct base for a live run. |
| (g) shellcheck | PASS | `shellcheck -S info -e SC1091 .claude/hooks/*.sh scripts/*.sh` (the gate's exact command) → CLEAN. |

`set -u` safety: `timeouts`/`mislabeled`/`lp` are all declared (`local timeouts=0 mislabeled=0 lp`) and
`$lp` is only expanded inside the loop body after `read -r lp` assigns it. No word-splitting risk on
`$lp` in practice (cargo-mutants log paths are `crates__…__file.rs_line_N_col_M.log` — no spaces — and
`$lp` is double-quoted at both `[ -f "mutants.out/$lp" ]` and the grep target regardless).

No `failure-record`/`prevention-rule` — nothing was wrong. Lenses covered: correctness/shell-safety,
state-integrity (the no-count-change invariant), simplification (no thread-pinning — the P2 measurement
said none is needed), provenance (N/A — shell, no AGPL).

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

Gate-is-test (D5): no `.rs`, no unit tests — §7 exit-code smokes + the real gate run. All smokes replicate
the gate:5 tail (audit block + arithmetic) VERBATIM from `scripts/gates.sh` against synthetic fixtures
(the same shell; only fixtures injected). Harness ran + self-cleaned; tree left clean.

**REQ-003 — the audit block, three cases (ACTUAL output):**
```
(i)   mislabel:     mutation: 1 timeout(s); 1 mislabeled (log shows failing tests — caught-by-assert, perf not detection)
                    mutation: 2 caught / 0 missed → MSI 100.0% (floor 100%)                     [rc=0]
(ii)  zero-timeout: mutation: 2 caught / 0 missed → MSI 100.0% (floor 100%)   ← NO timeout line  [rc=0]
(iii) missing-log:  mutation: 1 timeout(s); 0 mislabeled (…)   ← [ -f ] short-circuited grep, no crash
                    mutation: 2 caught / 0 missed → MSI 100.0% (floor 100%)                     [rc=0]
```
The mislabel fixture used the real retained log shape (`test fold::tests::x ... FAILED`). All three exactly
as designed: the audit prints only when timeouts>0, resolves `log_path` under `mutants.out/`, and degrades
safely on a missing log.

**REQ-004 — a genuinely MISSED mutant still reds the gate (ACTUAL output):**
```
mutation: 1 caught / 1 missed → MSI 50.0% (floor 100%)
MSI 50.0% < floor 100% — kill more mutants (write tests)              [tail rc=1 — RED]
```
Path chosen: the **isolation-arithmetic** proof (fed a synthetic `MissedMutant` row to the byte-identical
`caught`/`missed`/`total`/`msi`/floor tail) → it returns non-zero, red. This is stronger than arguing from
the diff alone: it shows the floor check actually fires. The audit touches ONLY `Timeout` rows, so a
`MissedMutant` can never be masked by it (REQ-006 in action). Combined with the P2 probe (nextest reports
real outcomes end-to-end — 12/12 caught from a real mutation run), a MISSED under nextest is reported and
reds the gate. (Path (a) — a full forced-missed mutation run — was unnecessary given the byte-identical
arithmetic + the probe; recorded as available if ever doubted.)

**REQ-001 / REQ-002 / REQ-005 — the nextest runner + clean reclassify + exit-0 (from the P2 probe):**
`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 cargo mutants -f crates/marley_app/src/font_zoom.rs --jobs 2
--no-times --test-tool=nextest` → **rc=0, 12 mutants → 12 caught, 0 missed, 0 timeout, ~2.7 min.** That is
REQ-001 (mutants ran under nextest — the runner named in the invocation), REQ-002 (the clean case: full set
`CaughtMutant`, **0 Timeouts**, fast — the #337 laundering gone), and REQ-005 (exit 0 accepted).

**REQ-006 — arithmetic untouched:** proven at inspect (d) — the `caught`/`missed`/`total`/`msi`/floor lines
diff byte-identical vs `HEAD`.

**The full `--diff` gate → GATE GREEN [diff] (15/15).** gate:5 printed exactly the expected
`mutation: no changed crate lines (diff) — nothing to mutate, pass` (the #345 diff touches only
`scripts/` + `docs/`, not `crates/`, so gate:5 in diff mode has nothing to mutate and the audit
correctly does not run). gate:11 shellcheck PASS (my edit is clean under the gate's own
`-S info -e SC1091`). All 15 PASS; coverage ≥100%, miri PASS, visual/AX PASS. Fresh receipt
`8fa5be4a034767a6bac62e70412340b89d2e49c2` written and verified worktree-bound against `gate_state_hash`
(editing `scripts/gates.sh` staled the old hash; this run minted the new one).

**syspolicyd note:** the first `--diff` run wedged on the intermittent macOS syspolicyd fault — every
freshly-built nextest test binary hung in `_dyld_start` at 0% CPU while the Gatekeeper daemon trio sat
idle (an ~9-min stall, log frozen). Recovery (documented, no gate weakened): stopped the stuck run
(which took its hung children with it), probed syspolicyd health with a trivial fresh binary (ran in
<1s → recovered), then re-ran the gate — it flew (cached build → straight to tests) and went green.
Re-running is not weakening the gate. This is the same fault that cost multiple attempts earlier this
session.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**
## Phase 5 — Complete

**Docs (§21):** CHANGELOG.md — a `### Changed` entry (gate:5 under nextest + the Timeout audit; convention
+ arithmetic unchanged). **No `docs/marley_architecture/` testing/gates doc exists** (checked: no
test*/gate*/contribut*/quality*/ci* doc) — per the gate-tooling convention, NOT inventing one; the
CHANGELOG carries it. **Shelf-note corrected** (integrity-five-shelf.md): the #345 table row + the
cross-cutting "#348 → #345 hard order" bullet both rewritten to D3-SYNERGY-NOT-DEPENDENCY (closes the
confident-but-wrong sentence at its source).

**Capture:** aar-submit 7a4476ce (completed, effectiveness 4). Lessons: (a) the D3 "hard order" was a
confident-but-wrong spec sentence, killed by the substrate's own behavior (gate:3 runs nextest daily
without `.config/nextest.toml`) — the batch's recurring win. (b) the retained `mutants.out.old` was gold:
a REAL mislabel (`fold.rs:75` Timeout whose log says `FAILED`) turned "plausible" into "catches a proven
case" and nailed that `log_path` is relative to `mutants.out/` — inspect a retained artifact before
parsing it (→ PR-claude-inspect-a-retained-artifact-before-parsing-it). (c) oversubscription resolved by
MEASUREMENT (rc=0, 12/12, 0 timeout) → pin nothing (§0). (d) syspolicyd recovery = kill stuck run + probe
health with a trivial fresh binary + re-run cached; never weaken, never escalate.

**forge not-wired fallback:** N/A — forge wired; captured via aar-submit + prevention-rule-record.

**Status: Phase 5 — Complete PASS.**
