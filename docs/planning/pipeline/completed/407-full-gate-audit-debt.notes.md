# 407 FULL-gate audit debt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-407-full-gate-audit-debt.md
- **Pipeline spec:** 407-full-gate-audit-debt.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

> **PARKED 2026-08-14** (Chad /goal: close #6, then #365 + #423 auto-approved).
> The remaining 407 scope — REQ-004 timeout classification, the REQ-002 sabotage
> smoke, the mutation_g gate-split re-plumb, the deferred full-gate re-run
> (REQ-005 as amended) — is gated on the dev-box sweep (restart #4,
> `marley-mutants-20260813-114903.scope`, log
> `~/builds/logs/mutants-20260813-114903-514e5b9.log`) finishing, so the active
> lane is handed to #365/#423 per §3's one-active rule. The REQ-003 killer-test
> slice already shipped (12b78c8). **Resume:** move this spec+notes pair back to
> `active/` when the sweep lands and continue from the Phase-3 "Still open" list.
> Note: #423 (queued next) fixes the very tests the box lane excludes/retries —
> land it before reading the sweep's addendum numbers as final.
>
> **UPDATE 2026-08-14 (mid-goal check):** the sweep FINISHED —
> `=== finished rc=3 2026-08-13 20:14:32 ===`, `4784 mutants tested: 16
> missed, 3866 caught, 848 unviable, 54 timeouts` (scope inactive). The 16
> missed are the survivors the same-evening REQ-003 slice killed (12b78c8,
> per-site scoped re-runs 0 missed). **The parking condition is MET** — when
> the #365/#423 queue completes, resume here: REQ-004 now has its real
> timeout list (54) to classify, and the 16-missed cross-check against the
> #423 excluded/flaky tests can run per the addendum caveat.

## Phase 1 — Plan
- **Request:** `/work 407 auto approved` — the Deliberate-row idle-machine
  ticket, picked explicitly; user pre-approved autonomous-through-commit.
- **Classification / tier:** bug (audit debt), work pipeline, one shippable
  slice with an explicit split valve (Scope/Out) if the enumeration surfaces a
  mountain.
- **Recall (§18.3):**
  - `scripts/gates.sh mutation_g` answers the ticket's open question: Timeout
    counts as CAUGHT (Stryker/Infection convention, documented inline) and the
    #345 audit already surfaces caught-by-assert mislabels. So timeouts never
    redden MSI — the deliverable is classification honesty, not arithmetic
    (spec D4).
  - FULL mode runs cargo-mutants at `--jobs 1` (two desktop freezes at 2 jobs,
    2026-08-07). Hours-long run; no concurrent heavy cargo work (spec D5).
    `MUT_JOBS` overrides; not used — the enumeration measures gate defaults.
  - The killer-test home: #272's notes (F2 inspect ledger is the ORIGIN of
    `refresh_efind_matches` — the stale-offset ropey-panic fix) + the
    `headless_drive.rs` REQ-flow family (REQ-004 there covers replace-one
    advance). #339 extended the same family. app.rs is cov-excluded but NOT
    mutants-excluded (PR-claude-cov-excluded-is-not-mutants-excluded…001) —
    every fn there must be killable or skip-justified; this one is neither.
  - PR-#298 equivalent-mutant doctrine → spec D3 (delete redundancy, never
    suppress; re-run to confirm the hole didn't relocate).
  - PR (cargo deadlock): after many concurrent cargo runs, cap
    `CARGO_BUILD_JOBS=4` if gpui-crate builds stall at 0% CPU — recovery recipe
    on file; relevant across this pipeline's long runs.
- **Discovery:** the precise edit surface for Design:
  - `crates/marley_app/src/app.rs:15712` — `refresh_efind_matches`, the
    `partition_point(|&(ms, _)| ms < resume)` seam (READ ONLY — D2: no code
    change, a test kills the mutant).
  - `crates/marley_app/src/headless_drive.rs` — the REQ-flow harness (#272
    REQ-004 drove replace-one advance; the new case adds a needle-containing
    replacement).
  - `crates/marley_app/src/code_syntax.rs` — `is_ident_start` /
    `is_ident_continue` / `highlight_line`, the timeout cluster (≥6 of 15).
  - `scripts/gates.sh` mutation_g + (potentially new) `.cargo/mutants.toml` —
    the timeout remedy surface; NOTE: mutants.toml would become a gate-defining
    file → §15 receipt fingerprint set may need it (inspect angle).
  - No leftover `mutants.out/` from the #402-era partial run — the enumeration
    is genuinely required, nothing to salvage.
- **Decisions:** D1–D5 locked in the spec (enumerate-first at gate defaults;
  test-not-code for efind; PR-#298 doctrine for equivalents; timeout=caught
  convention unchanged, classify + remedy false kills only; no heavy cargo
  concurrent with the FULL run).
- **Enumeration run:** started in the background at the end of Phase 1
  (`scripts/gates.sh`, FULL mode, gate defaults, caffeinated; log →
  scratchpad `gates-full-407.log`). Expected END STATE: RED at gate:5 with a
  complete `mutants.out/outcomes.json` — that artifact IS the REQ-001
  measurement. Static gates run first (minutes) — an early abort there is loud
  and immediate.

## Phase 2 — Design

### Load-bearing design discovery: the headline mutant is ALREADY DEAD
TICKET-407's text (created 2026-08-06 at #402's stopped FULL run, exported
2026-08-09) is STALE on its headline item. The #404 pipeline's delivery-gate
sweep (commit `4024f29`, 2026-08-07) caught the same mutant and fixed it at
source: `efind_replace_resume_boundary_keeps_adjacent_match_headless`
(`headless_drive.rs:6942`) drives TWO Replace-Ones over `aaa` with replacement
`b` and asserts the TEXT (`bba` correct vs `bab` under the `<=` mutant — the
comment records why a 1-survivor fixture proves nothing: `efind_index` clamps
both arms to 0). The #404 notes' post-archive block records the scoped re-run:
app.rs 76 mutants — 72 caught, 4 unviable, **0 missed**. The fixture is exactly
the ticket's requested shape (adjacency at the resume offset; needle-containing
replacement is the `aaa`→`b…` degenerate where the NEXT match starts AT resume).
**REQ-002 therefore pivots to VERIFY-ONLY** (D2 holds — a test, not a code
change; the test simply already exists):
1. The running enumeration must show the `partition_point` `<` mutant CAUGHT.
2. A sabotage smoke at validate: hand-flip `<`→`<=`, run just
   `efind_replace_resume_boundary_keeps_adjacent_match_headless` → RED; revert
   → green. (Deferred to validate per D5 — no builds beside the live FULL run.)
If the enumeration contradicts #404's record (missed again ⇒ the test is flaky
or was weakened), that is a design deviation: investigate, strengthen, file the
flake as a failures-ledger entry.

### Architecture / approach
- **No production behavior change** is the expected end state. The work is:
  verification (REQ-002), enumeration-driven killer tests / D3 redundancy
  deletions in whatever files the addendum lists (REQ-003), and the timeout
  classification record (REQ-004).
- **Timeout classification procedure** (REQ-004), over the enumeration's
  `mutants.out/outcomes.json` + `mutants.out/log/*`:
  1. `jq` every outcome with `summary=="Timeout"` → its `log_path`.
  2. Log shows `FAILED|panicked|assertion` → **CAUGHT-BY-ASSERT** (the #345
     mislabel: genuine detection, wall-clock artifact). No remedy.
  3. Else, log tail shows a test started and never finished → **HANG** (genuine
     detection under the standing Stryker convention gate:5 already encodes).
     No remedy. Grounded expectation for the `code_syntax.rs` cluster: the
     quote/number/ident scan loops in `highlight_line` advance solely via
     `i += 1`; cargo-mutants' `i *= 1` replacement is a textbook no-progress
     infinite loop (read at design: `code_syntax.rs:325-375`).
  4. Else (every logged test passing, wall clock exceeded) → **SLOW-BUT-PASSING
     = FALSE KILL** → remedy required.
- **Remedy tree for false kills only** (in order):
  1. Add/strengthen a fast-failing behavior test that DISTINGUISHES the mutant
     (false kill → real kill; §0 source-fix; naturally also faster).
  2. If behaviorally equivalent-but-slower (cache/memo-condition mutants): pin
     the mechanism with a behavior-visible test where cheap; else apply D3
     (is the operator redundant?) or document under the standing convention —
     the classification table IS the honesty record.
  3. LAST RESORT, only on evidence of systemic auto-timeout tightness (false
     kills scattered with margins just past multiplier×baseline): a global
     `.cargo/mutants.toml` `timeout_multiplier` raise — cargo-mutants has NO
     per-file knob (prior-art finding). **Binding rider:** `gate_state_hash`
     (`.claude/hooks/lib-hook-helpers.sh:44`) fingerprints `crates scripts
     .claude/hooks clippy.toml deny.toml .gitleaks.toml Cargo.toml Cargo.lock`
     — `.cargo/` is NOT in the set, so introducing mutants.toml MUST add
     `.cargo` to that path list in the same change, or the receipt fails to
     bind a gate-defining file (§15 hole, found at design).
- **§20 confirm:** N/A holds — quality-gate infrastructure; no reference-app
  behavior to match. The one user-adjacent contract touched (F5 Replace-One
  resume) shipped reference-matched at #272/#339 and is only being PINNED.
- **§14:** no new production surfaces; typed-error/panic rules untouched.
  Explicitly REJECTED: fail-fast/progress asserts in the `code_syntax.rs` hot
  loops purely to convert hangs into fast panics — harness-serving production
  code for a case the convention already counts correctly (risk R4).

### File manifest (lean; items 1–3 conditional on the addendum)
1. `crates/marley_app/src/headless_drive.rs` — ONLY IF the enumeration
   contradicts #404's kill record: strengthen the boundary test (e.g. the
   ticket's literal `ab`→`xaby` variant). Expected: NO EDIT.
2. `<per-addendum files>` — killer tests in the owning crate's `#[cfg(test)]`
   mod (or the headless REQ-flow family for RootView seams); or D3 redundancy
   deletions. The addendum lists each missed mutant → file → remedy class.
3. `crates/marley_app/src/code_syntax.rs` — ONLY IF a false-kill timeout traces
   to a missing span assert: add the distinguishing test in its `#[cfg(test)]`
   mod (line 475+). Expected for hangs: NO EDIT.
4. `.cargo/mutants.toml` (NEW) + `.claude/hooks/lib-hook-helpers.sh` (add
   `.cargo` to the `gate_state_hash` path list, same change) — ONLY on the
   remedy-tree's last-resort branch.
5. `docs/planning/pipeline/active/407-*.md` — phase entries + the addendum
   (ungated docs).

### Regression Test Plan
| REQ | Test / verification | Kind |
|---|---|---|
| REQ-001 | Enumeration completeness: `gates-full-407.log` shows mutation_g completed (cargo-mutants exit ∈ {0,2,3}); `outcomes.json` caught/missed/timeout/unviable counts recorded in the addendum | run evidence |
| REQ-002 | Existing `efind_replace_resume_boundary_keeps_adjacent_match_headless` is the killer: enumeration shows the app.rs `partition_point` `<` mutant CAUGHT; validate sabotage smoke: flip `<`→`<=` → `cargo nextest run -E 'test(efind_replace_resume_boundary)'` RED → revert → green | existing gpui::test + negative smoke |
| REQ-003 | Per addendum item: new killer test (or D3 deletion) then targeted `cargo mutants -f <file> [--re <fn>]` → 0 missed | unit/headless + targeted mutants |
| REQ-004 | The classification table in the addendum: every Timeout → class + log-line evidence; false-kill count after remedies = 0 | log forensics record |
| REQ-005 | Final `scripts/gates.sh` (FULL) exit 0; `.git/ignibyte-gate-receipt` present; gate:5 line "0 missed" quoted in notes | gate exit code |
- Uncoverable paths: none new (no production code planned). Pre-existing
  coverage state observed green at the enumeration's gate:4 (lines 100.00%;
  the 157 misses in the log are REGION-level, not the gated metric).

### Risks / decisions
- **R1** — enumeration contradicts #404's efind kill ⇒ flaky/weakened test;
  investigate + strengthen (manifest item 1), failures-ledger entry.
- **R2** — mountain valve (spec Out): >~10 additional missed mutants ⇒ fix the
  tractable set, split follow-up tickets carrying the enumeration.
- **R3** — timeout borderline flakiness run-to-run (margin near
  multiplier×baseline): classification records evidence; the FINAL FULL run is
  the arbiter; borderline flips get margins documented, not gamed.
- **R4** — REJECTED: production fail-fast asserts for mutation-harness comfort
  (see §14 note). Revisit ONLY via remedy-tree step 1 (a behavior test that
  happens to fail fast is fine; an assert that exists for the harness is not).
- **R5** — `.cargo/mutants.toml` is outside today's receipt fingerprint; the
  binding rider in the remedy tree closes the §15 hole if the file is ever
  introduced.

### DESIGN ADDENDUM — the definitive fix ledger (enumeration COMPLETE)

**REQ-001 — the enumeration is COMPLETE.** Box run #4 finished 2026-08-13
20:14:32 box time, rc=3 (`marley-mutants-20260813-114903.scope`): 4,784
tested this run + 1,046 iterate credit (the restored 2026-08-12 seed's 892
caught / 154 unviable) = **5,830/5,830**. Merged record: **caught 4,758 /
missed 16 / timeout 54 / unviable 1,002**. Zero nextest FLAKY marks — the
retry net was never needed mid-run. Artifacts: box
`~/builds/marley/mutants.out.final-407-20260813/` (archived against the
rotation gotcha), local pull `./mutants-box/`, plus the 2026-08-12 local
partial at `target/mutants.out.partial-407-20260813/` (cross-checks the
seed's caught portion). NOTE the completion was masked ~70 min by a stale
`state: RUNNING` — see the zsh-orphan discovery below.

**The fix ledger — 16 missed → 6 clusters → 7 remedies (all landed
2026-08-13 evening, Chad's goal directive: fix + scoped isolation runs, NO
second full sweep — full re-run deferred):**
1. `terminal_blocks/mouse.rs` ×5 (`mouse_report` `+`→`-`/`*` on
   release/wheel/X10-release mod_bits) → killer test
   `modifier_bits_ride_release_and_wheel` (byte-exact vectors: SGR release
   0+shift=4 'm', wheel 64+ctrl=80 / 65+alt=73, X10 release 3+shift=39).
   The prior matrices carried mods only on press/drag.
2. `marley_lsp/signature_help.rs` ×3 (`param_range` len==2 guard→true,
   `start>end`→`>=`, `end>total`→`>=`) → killer test
   `parse_label_offset_boundaries_exact` (len-3 array → None; [2,2] →
   Some(2..2); [0,total] → Some(0..5)).
3. `editor/ime.rs` ×3 (`replace_text_ctx:150` no-op guard `||`→`&&`,
   `!=`→`==`, `!`-deletion) → killer test
   `t407_branch_two_noop_guard_edits_both_ways` (collapsed-range insert +
   real-range empty-text delete, text AND caret asserted).
4. `editor/ime.rs:235` ×1 (`selected_utf16` match guard `a != caret`→true)
   → **D3 REDUNDANCY DELETION** (PR-#298 doctrine): the guard is provably
   equivalent — `anchor` is Some only when `!is_caret()` and `is_caret` IS
   `anchor == head`, so the inequality is the binding's invariant. Guard
   removed, `_` arm tightened to `None` (total match); comment records the
   invariant. Production edit, behavior-preserving.
5. `editor/line_move.rs` ×1 (`move_block_edit` Up-arm `b0 - 1`→`/`) →
   killer test `t407_move_up_swaps_and_rides_by_gap_length` (UNEQUAL line
   lengths pin both `b0 - 1` terms: swap text + caret-delta -3 asserted).
6. `marley_command/lib.rs` ×2 (`default_opener` → `""`/`"xyzzy"`) → the
   existing C4 assert was macOS-gated; added the
   `#[cfg(not(target_os = "macos"))]` `"xdg-open"` arm. Box-blind-spot
   class: killed locally all along, unasserted on Linux.
7. `marley_app/browser_probe.rs` ×1 (`probe_web_origin:179` cap `<`→`<=`)
   → killer test `probe_status_line_cap_is_exclusive_at_1024` (peer sends
   EXACTLY 1024 newline-free bytes, socket parked open via channel — the
   cap, not EOF/timeout, ends the loop → `Io("malformed response")`; the
   mutant burns the read budget into a timeout classification).

**Validation (the goal's isolation protocol, NEXTEST_TEST_THREADS=2, each
site its own `-o target/scoped-407/<name>` so nothing rotates):** new tests
first ran green (244/244 incl. full marley_editor suite — also validates
the D3 edit). Scoped `cargo mutants -f <file> [--re <fn>]` per site — the
LOCAL full suite (all 2158 tests, no exclusion filter) makes each scoped
run double as the TICKET-423 blind-spot cross-check by construction:
- mouse (`--re mouse_report`): 26/26 caught, 0 missed ✓
- sighelp (`--re param_range`): 28 tested, 14 caught + 14 unviable, 0 missed ✓
- linemove (`--re move_block_edit`): 23/23 caught, 0 missed ✓
- ime (whole file, production edit): 64 tested, 39 caught + 25 unviable,
  0 missed ✓ (the deleted-guard site enumerates no mutant; nothing new
  missed after the D3 edit)
- opener (`--re default_opener`): 2/2 caught ✓
- probe (`--re probe_web_origin`): 8 tested, 6 caught + 2 unviable,
  0 missed ✓
**Isolation total: 151 mutants tested across the six sites — 110 caught,
41 unviable, 0 missed.** All 16 ex-survivors fall inside these runs and
are dead. Runs completed 21:33 local, logs under `target/scoped-407/`.

**REQ-005 AMENDED (Chad decision, 2026-08-13 /goal):** no second full sweep
now — the 16 fixes + per-site scoped 0-missed = **assumed green**; the next
full-gate run happens at a later date and re-measures everything (the
`--iterate` seed on the box makes that cheap: only the 16 ex-missed + 54
timeouts + any new mutants retest).

**REQ-004 (timeout classification, 54 mutants) — REMAINING pipeline work,
NOT tonight's goal scope.** The 54 include the known 17 (14 code_syntax +
complete/editor_complete/editor_references) + ~37 new (find.rs,
terminal_blocks/session.rs pump, marley_text_offsets floor_char_boundary,
…). Classification per the Phase-2 procedure over
`mutants-box/outcomes.json` + logs; cross-platform riders already on file
(OOM-culled bombs record CAUGHT; retried flakes would mark FLAKY — zero
observed).

**New discoveries for the ledgers:**
- **PTY zsh leak (TICKET-423 adjacent):** the sweep leaked TEN orphaned
  `/bin/zsh` (PTY-test children surviving mutant-broken teardowns),
  reparented to PID 1 at ~3 MB each. They inherited the runner's lock fd 9
  → `.mutants.lock` stayed flocked → `marley-mutants status` reported
  RUNNING for ~70 min after rc=3, and the scope stayed "active running"
  (non-empty cgroup). Cleared via `systemctl --user stop` (interactive zsh
  ignores SIGTERM — the stop waits systemd's 90 s then SIGKILLs).
- **Runner improvement candidates (Chad's call, NOT applied):** (a) set
  CLOEXEC on the lock fd (orphans could never wedge `status`); (b) a
  post-run orphan reap in the wrapper. Recorded here + memory; the
  no-modify rule stands.

- **2026-08-13 crash + resume note:** the 2026-08-12 20:32 enumeration was
  killed at 00:07:47 on 2026-08-13 — the overnight session and the run died in
  the same second (SIGTERM logged in `debug.log` at 12920s, `err=interrupted
  phase=Build`); the Mac was rebooted 07:01. Partial artifact survived and is
  BACKED UP at `target/mutants.out.partial-407-20260813/` (gates.sh mutation_g
  `rm -rf`s `mutants.out`, and `--iterate` writes a fresh outcomes.json —
  **merge backup + iterate outcomes for the REQ-001 complete record**).
  Partial state: **1065/5830 outcomes** (892 caught / 1 missed / 17 timeout /
  154 unviable / 1 baseline Success), alphabetical file order, died in
  `editor_references.rs`.
  - **REQ-002 answered early:** app.rs COMPLETE (94/94); ALL
    `refresh_efind_matches` mutants CAUGHT incl. the partition_point
    `<`→`<=` at app.rs:16727:67 (line drifted from 15712). #404's kill record
    CONFIRMED — R1 off the table; only the validate sabotage smoke remains.
  - **Early REQ-003 item:** `browser_probe.rs:179:46: replace < with <= in
    probe_web_origin` MISSED — new, not in the ticket's known set.
  - **Timeouts so far (17):** the predicted `code_syntax.rs` cluster (14) +
    `complete.rs:57` / `editor_complete.rs:149` (`-=`→`/=`) +
    `editor_references.rs:133` (`==`→`!=`) — REQ-004 classification pending
    on the complete run (iterate RETESTS timeouts + the missed one; fresh
    logs land in the new mutants.out).
  - **Resumed 2026-08-13 ~07:15** via direct `cargo mutants --iterate --jobs 1
    --no-times --test-tool=nextest` (exact gate margs; direct run because the
    gate wipes the partial), TMPDIR scratch per #335 w/ traps, caffeinated,
    log → `target/mutants-iterate-407.log` (reboot-proof, unlike the dead
    scratchpad `gates-full-407.log`). ~4765 mutants remain ≈ 16h at the
    measured 12.1s/mutant — D5 stands: no heavy concurrent cargo work.
  - **07:26 — the resume died too (exit 1), and the root cause of BOTH deaths
    is now pinned.** Morning run: baseline ok, browser_probe MISSED
    re-confirmed, then the code_syntax timeout cluster → swap 0→3GB in
    minutes → seven abnormal nextest exits (code=102) → FATAL: the
    /var/folders scratch tree lost `complete.rs` mid-run ("does not exist,
    refusing to create it") — most likely a macOS tmp/low-space sweep during
    the storm (no purge event logged; relocation moots the question either
    way). Iterate credit intact: previously_caught=1046 carried, +16 new
    outcomes in the fresh mutants.out (the 00:07 partial stays authoritative
    in the backup).
  - **Root cause (cross-confirmed with the mem-watchdog session's PID
    tracking):** jobs=1 was honored; the killer is per-TEST memory. nextest
    (profile.default, TICKET-348) runs ~10 test procs in parallel; a poisoned
    mutant (e.g. `-=`→`/=` in a scan loop) makes tests ALLOCATE unboundedly
    until the 180s terminate ceiling — a time cap, not a memory cap — so
    5–9 GiB per proc. Under thrash the reaping failed and orphaned bombs
    persisted ~3h (watchdog saw the same PIDs 21:18→00:05; the 00:05 jetsam
    report shows TEN `marley_app-<hash>` procs ≈52 GiB on a 16 GiB box) →
    jetsam storms (21:15 / 21:17 / 00:05) → WindowServer userspace-watchdog
    death 00:07:45 → GUI session teardown = the SIGTERM that killed run +
    session at 00:07:47. No kernel panic; box wedged till the 07:00 power
    cycle.
  - **Relaunch safeguards (env-only, gate defaults untouched):**
    `NEXTEST_TEST_THREADS=2` (worst case 2×~9 GiB fits), scratch relocated to
    `/Volumes/Offload/marley-mutants-scratch/` (827G free, off the internal
    SSD that hosts swap, no `.git` ancestor so the F1/#335 constraint holds),
    sidecar reaper killing `marley_app-<hash>` test orphans (RSS>10 GiB —
    ABOVE the bombs' observed 9.3 GiB max so in-window bombs still die by the
    180s ceiling and register as Timeout, keeping REQ-004's classification
    honest — or etime>7 min, past 180s+grace, i.e. escapees only),
    `--iterate`. With Chad's ~9 GiB standing workload, threads=2 still means
    ~18 GiB of new demand per bomb window (deep swap, 3–4 min hiccups);
    threads=1 (one bomb max) is the in-use-friendly local variant at
    ~+20–40% wall clock. DEVIATION for the addendum: enumeration
    measured with test-threads=2 — timing-only (per-test 180s ceiling
    unchanged; fail-fast still cancels on first termination); R3 covers
    borderline timeout flips.
  - **Durable gate fix → Phase 3 scope** (mutation_g is already in the edit
    surface): mode-aware `NEXTEST_TEST_THREADS` beside the MUT_JOBS block
    (FULL⇒2), so the REQ-005 final FULL run is survivable on this 16 GiB
    box. Gate-defining change; receipt fingerprint already covers `scripts/`.
  - **2026-08-13 POLICY SHIFT (Chad, via the dev-box handoff): full-workspace
    mutation is BANNED on this Mac — local = scoped `-f` only; full sweeps
    run on the dev box** (24 threads / 125 GB, memory-capped systemd scope;
    lane validated end to end 2026-08-13; box has no GitHub access). Drive:
    `git push dev main` → `ssh dev marley-mutants start main` / `status` /
    `results`; pull `rsync -a dev:builds/marley/mutants.out/ ./mutants-box/`.
    Runner bakes `--iterate --jobs 2 --test-tool=nextest` + a nextest `-E`
    filter excluding 4 Linux-env-sensitive tests (→ TICKET-423; 2154/2158
    pass otherwise). Rotation gotcha: every run moves mutants.out →
    mutants.out.old (a scoped box run resets iterate memory; restore =
    `ssh dev 'mv ~/builds/marley/mutants.out.old ~/builds/marley/mutants.out'`).
  - **CORRECTIONS per the handoff:** the 07:15 local resume was STOPPED
    DELIBERATELY (supersedes the "tmp sweep, unproven" hypothesis above).
    Box seed = the restored FULL 2026-08-12 outcomes (the morning iterate's
    16 outcomes are superseded; locally the 00:07 backup stays authoritative,
    and `target/mutants-iterate-407.log`'s rc=1 is expected history).
    browser_probe.rs:179:46 `<`→`<=` in probe_web_origin is CONFIRMED MISSED
    ON BOTH PLATFORMS (box validation run 08:14, rc=2) — REQ-003's first
    definite killer-test item.
  - **Addendum caveat:** box results measure the suite MINUS the 4 excluded
    tests — cross-check any box-MISSED mutant against those tests locally
    before writing a killer test (blind-spot class).
  - **Design amendments this forces (fold into Phase 2/3):** REQ-005's
    "final scripts/gates.sh (FULL) exit 0" can no longer mean a local full
    sweep — gate:5 FULL must be re-plumbed for the split (local = scoped +
    threads cap; full = dev-box lane / imported outcomes; §15 receipt
    implications to design). The NEXTEST_TEST_THREADS knob above narrows to
    the LOCAL scoped lane. REQ-001's enumeration now completes on the box:
    **full sweep STARTED 2026-08-13 09:05:44 box time, main@514e5b9, unit
    `marley-mutants-20260813-090544.scope`, iterate credit live (1046
    excluded); log
    `/home/cpeppers/builds/logs/mutants-20260813-090544-514e5b9.log`.**
  - **Box run #1 DIED 09:11:45 (6.5 min in) — memcg oom-kill; seed restored;
    root cause pinned; restart held for a runner decision.** Only 5 outcomes:
    browser_probe MISSED re-confirmed + the first 4 `code_syntax.rs` timeouts.
    Kernel journal: `code_syntax::te… invoked oom-killer`, CONSTRAINT_MEMCG on
    the scope's cgroup — SIX parallel `marley_app` test procs at 4–6 GiB anon
    each (2 jobs × NEXTEST_TEST_THREADS=8 over the timeout cluster) blew the
    scope's MemoryMax=64G (swap 0); systemd's default `OOMPolicy=stop` then
    stopped the WHOLE scope (`Failed with result 'oom-kill'`) → cargo-mutants
    `Error: interrupted`, rc=1. Containment held — box services never noticed,
    exactly what the scope is for — but stop-policy kills the sweep, not just
    the bomb. Since `--iterate` RETESTS all 17 timeouts, an unmodified restart
    marches back into the same gauntlet and dies the same way.
  - **Recovery done (box):** crashed run's outcomes were a STRICT SUBSET of
    the seed (same 1 missed; its 4 timeouts = seed timeouts 1–4), so the seed
    was restored losslessly; crashed `outcomes.json` preserved at
    `~/builds/logs/crashed-run-20260813-090544-outcomes.json`. Restore-recipe
    correction: `rm -rf mutants.out` FIRST, then `mv` — the documented plain
    `mv` would nest into the existing dir.
  - **Fix decision → Chad (runner is no-modify by his policy):** recommended
    one-liner: add `-p OOMPolicy=continue` to the runner's systemd-run. Memcg
    then kills only the fattest bomb proc; nextest records that test's SIGKILL
    as a failure; the sweep grinds on; the 64G cap still protects the box.
    REQ-004 rider: an OOM-killed bomb records CAUGHT where the Mac recorded
    Timeout — cross-platform classification drift to note in the table (both
    count as caught under the standing convention, so MSI arithmetic is
    unaffected). Alternative without modifying the runner — pass-through
    `--exclude` of the 4 bomb files — trades a REQ-001 coverage hole and stays
    exposed to unknown bombs in the untested 4,766; inferior.
  - **Runner amended + sweep RESTARTED (Chad-approved, 2026-08-13):**
    `-p OOMPolicy=continue` added to the runner's systemd-run line (backup at
    `~/builds/logs/marley-mutants.pre-oompolicy-20260813`; scope property
    smoke passed; verified LIVE on the new scope alongside MemoryMax=64G /
    swap 0). Restart 10:13:50 box time, main@514e5b9, unit
    `marley-mutants-20260813-101350.scope`, iterate credit live (1046
    excluded / 4784 to test); log
    `/home/cpeppers/builds/logs/mutants-20260813-101350-514e5b9.log`. The
    bomb gauntlet (17 timeout retests, code_syntax first) falls in the first
    ~20 min — surviving it validates the fix; 15-min monitor armed.
  - **Restart #2 died at BASELINE (rc=4, 10:15:52) — not the OOM fix: a
    flaky PTY test.** `marley::integration
    echo_command_produces_a_block_with_its_output` timed out its 4 s pump
    window (`integration.rs:46`) in the unmutated tree (2153/2154 passed
    otherwise). Measured immediately after: 10/10 solo + 3/3 full-suite
    passes at lane conditions — a rare (~1-in-7 suite runs at worst) timing
    flake; filed as TICKET-423's 5th member (flaky, NOT excluded);
    baseline.log preserved at
    `dev:~/builds/logs/baseline-flake-20260813-101350.log`.
    **Addendum caveat (REQ-004):** mid-sweep this flake can false-CAUGHT a
    would-be survivor (expected ≈1–2 total = observed rate × survivor
    prevalence) — a hidden-survivor class the box↔local cross-check can't
    see; carry in the classification table. Rotation rule confirmed the hard
    way: EVERY run — even a 2-minute rc=4 — rotates mutants.out; restore the
    seed before each restart (`rm -rf mutants.out` first).
  - **Sweep restart #3 (seed re-restored): 10:34:29 box time, main@514e5b9,
    unit `marley-mutants-20260813-103429.scope`, iterate credit live (1046
    excluded / 4784 to test), OOMPolicy=continue verified on the scope; log
    `/home/cpeppers/builds/logs/mutants-20260813-103429-514e5b9.log`;
    15-min monitor re-armed.**
  - **Restart #3 died at BASELINE too (rc=4, 10:36:52) — DIFFERENT test,
    same class:** `marley_terminal::integration
    teardown_is_instant_when_child_already_reaped` failed its FIRST assert
    in 34 ms (`crates/terminal_blocks/tests/integration.rs:142`) — the pump
    bailed before observing ChildExited: the Linux PTY exit race (child exit
    closes the master → EIO on read) that `write_after_disconnect_errors`
    hits deterministically, here probabilistic (load/CPU-quota modulated).
    Scope-baseline tally 2026-08-13: 2 pass / 2 fail; direct unscoped runs
    13/13 green. Verdict: a flaky PTY-timing CLASS, not one test — and the
    same per-suite exposure would false-CAUGHT surviving mutants sweep-wide.
    Baseline log preserved at
    `dev:~/builds/logs/baseline-flake-20260813-103429.log`; teardown race
    filed as TICKET-423 member 6 (possible REAL `marley_terminal` pump gap
    on Linux — EIO-before-reap-event; triage production-fix vs
    test-tolerance at 423 design).
  - **Runner amendment 2 (Chad-approved): `NEXTEST_RETRIES=1`** in the scope
    env (backup `~/builds/logs/marley-mutants.pre-retries-20260813`). Flaky
    tests retry once and pass (nextest FLAKY, exit 0) — no rc=4 baselines,
    no false-caught mutants; genuine kills fail the retry and stay caught.
    Cost: OOM-culled bomb tests re-bomb once (+~30–50 min worst case across
    the 17). **REQ-004 rider:** FLAKY marks in mutant/baseline logs are the
    fingerprint of forgiven flakes — count and list them in the addendum.
  - **Sweep restart #4 (seed re-restored, third rotation): 11:49:03 box
    time, main@514e5b9, unit `marley-mutants-20260813-114903.scope`, iterate
    credit live (1046 excluded / 4784 to test), OOMPolicy=continue +
    NEXTEST_RETRIES=1 in the scope; log
    `/home/cpeppers/builds/logs/mutants-20260813-114903-514e5b9.log`;
    15-min monitor re-armed.**

### RESUME DESIGN (2026-08-14, unparked — Chad /goal: 407 then 424, auto-approved)

**Parking condition met + context:** the box sweep finished (see the addendum);
#423 landed (the excluded tests are portable now — the box verified 2169/2169
with NO exclusions at 29be05c; the runner still carries its `-E` filter +
`NEXTEST_RETRIES=1` until Chad's rider lands, so the blind-spot cross-check
protocol stays in force for box-missed mutants); #365/#423/#424 moved main to
8630e2c; `rust-toolchain.toml` now pins 1.96.0 on both machines.

**REQ-004 — CLASSIFICATION COMPLETE (design-time — the logs were already
local at `mutants-box/`; zero remedies required):**
| Class | Count | Evidence pattern | Remedy |
|---|---|---|---|
| HANG (genuine detection — the standing Stryker convention) | **39** | log shows a test started, never finished, no failure line — the no-progress-loop class (`+=`→`*=` / `-=`→`/=` in scan/advance loops: code_syntax idents, dcs `find/split_unescaped`, session.rs pump/write budgets, editor find/movement, floor_char_boundary, …) | none |
| CAUGHT-BY-ASSERT (the #345 mislabel — real kill, wall-clock artifact) | **15** | log shows `FAILED\|panicked\|assertion` (find_matches `+`→`-`/`*`, project_count→0, fold `!`-delete, dcs/handshake/search `+=`→`-=`, editor find_all, active_buffer leak-default, is_ident_start→true, …) | none |
| SLOW-BUT-PASSING (FALSE KILL) | **0** | — | **none needed** |
- Full 54-row detail reproducible from `mutants-box/outcomes.json` +
  `mutants-box/log/*` via the Phase-2 jq/grep procedure (commands in the
  transcript); the class counts above are the audit record. Consequences:
  the remedy tree's last resort (`.cargo/mutants.toml`) is DEAD — no
  mutants.toml, no `.cargo` receipt-path addition (R5 closes unused); NO
  crate changes remain anywhere in this pipeline's scope.

**REQ-005 measurement (decision):** the box is idle and the iterate seed is
live → a box re-run at `main@8630e2c` was STARTED at design close
(2026-08-14 10:50:00 box time, log `mutants-20260814-105000-8630e2c.log`,
watcher armed): retests the 16 ex-missed (now fixed) + 54 timeouts + the new
#423 session.rs mutants, with iterate credit for the 4,758 caught. This
REPLACES "assumed green" with a measured verdict at the exact final .rs tree
(REQ-004 required no crate changes, so 8630e2c IS final). Honesty caveat
recorded: the iterate credit was measured under box rustc 1.94 (pre-pin);
the retested-set verdicts land under the pinned 1.96. The next natural FULL
re-measure is same-compiler end to end.

**The gate:5 lane split (the re-plumb design):**
1. **DIFF mode: unchanged** (2 jobs, `--in-diff`, local — minutes, the
   per-commit loop).
2. **FULL mode's local sweep is BANNED-BY-DEFAULT** (the 2026-08-13 policy —
   16 GB + poisoned-mutant memory bombs): without an explicit opt-in, the
   FULL path never runs `cargo mutants` over the whole workspace locally.
   `MUT_FULL_LOCAL=1` re-enables it deliberately (capable machines only) and
   THAT path gains the mode-aware `NEXTEST_TEST_THREADS` cap (FULL⇒2 unless
   the operator already set it — the crash-forensics knob from the addendum).
3. **FULL mode's default = IMPORT the box lane's outcomes:**
   `MUT_OUTCOMES=<dir>` (a pulled `mutants.out`, e.g. `./mutants-box`) +
   `MUT_OUTCOMES_SHA=<sha>` (the commit the box measured — from the runner's
   log filename). Provenance enforced before any counting: (a) `HEAD` ==
   `MUT_OUTCOMES_SHA`; (b) the MEASURED SURFACE is clean —
   `git status --porcelain -- crates Cargo.toml Cargo.lock rust-toolchain.toml`
   empty (scripts/docs dirt is fine: mutants never measure it). Then the
   EXISTING #345 timeout audit + MSI threshold run over the imported
   `outcomes.json` (factored into a helper shared with the local path — same
   arithmetic, same floor clamp, one source of truth). Missing import in
   FULL mode → fail closed with the box-lane instructions. §15: the receipt
   then binds the worktree exactly as a local run would — the provenance
   check is what makes the import sound. The SHA attestation is
   operator-supplied, consistent with the §15 posture (discipline scaffold;
   the hard boundary remains the worktree fingerprint).
4. **`gate_state_hash` gains `rust-toolchain.toml`** (found at this design:
   #423's pin SELECTS THE COMPILER — a gate-defining file outside the
   fingerprint is exactly the §15 hole class the R5 rider described for
   mutants.toml; fix it for the file that actually exists).
5. **Scoped local runs** (`cargo mutants -f <file> --re <fn>`) stay ad-hoc
   verification outside the gate — documented in the mutation_g comment.

**File manifest (resume — replaces the conditional items; NO crate edits):**
| File | Change |
|---|---|
| `scripts/gates.sh` | mutation_g: extract the count/threshold body into `mutation_verdict <outcomes-dir>`; FULL default = import path (provenance checks + verdict over `MUT_OUTCOMES`); `MUT_FULL_LOCAL=1` opt-in for the local sweep + `NEXTEST_TEST_THREADS` FULL cap; comment block records the policy + the scoped-lane pointer. |
| `.claude/hooks/lib-hook-helpers.sh` | `gate_state_hash` path list += `rust-toolchain.toml`. |
| `docs/planning/pipeline/active/407-*.md` | phase entries (ungated docs). |

**Test plan (resume rows — the re-plumb is a gate-is-test change, §7):**
| Item | Verification |
|---|---|
| Import path happy | `scripts/gates.sh` FULL with `MUT_OUTCOMES=./mutants-box MUT_OUTCOMES_SHA=<box sha>` at a clean HEAD==sha → gate:5 PASS from imported counts; receipt written (REQ-005's vehicle). |
| Import provenance negative smokes | (a) wrong SHA → gate:5 RED with the mismatch message; (b) dirty `crates/` (scratch touch) → RED; revert → green. Exit codes, not prose (§15). |
| Local-full ban | FULL without `MUT_OUTCOMES` and without `MUT_FULL_LOCAL=1` → gate:5 RED citing the policy + instructions (negative smoke). |
| DIFF regression | `scripts/gates.sh --diff` end-to-end green after the edit (the per-commit loop must be untouched). |
| Receipt fingerprint | `rust-toolchain.toml` content-change flips `gate_state_hash` (hash before ≠ after a scratch edit; revert restores) — the §15 binding smoke. |
| REQ-002 sabotage smoke (deferred from the original plan) | flip `<`→`<=` at the `partition_point` (app.rs:16727) → `cargo nextest run -E 'test(efind_replace_resume_boundary)'` RED → revert → green. |
| REQ-005 | Box re-run terminal state pulled (`rsync` → `./mutants-box/`) → the import-path FULL gate green = the measured final verdict. |

**Risks (resume):** R6 — the box re-run surfaces a NEW missed mutant from the
#423 session.rs code (the grace/latch units were written to kill exactly
those; if one survives, it's a REQ-003-class item: killer test + scoped
re-verify before the import gate can pass). R7 — box runner still excludes
the 4 tests: any box-missed gets the documented local cross-check before
being treated as real. R3 (borderline timeout flips run-to-run) stands.

## Phase 3 — Implement
- **RESUME slice (2026-08-14) — React-first: N/A** (gate infra; no UI delta).
  Built exactly to the resume manifest, no crate edits:
  - `scripts/gates.sh` — `mutation_verdict <dir>` extracted (the #345 audit +
    MSI threshold verbatim, parametrized by outcomes dir); `mutation_g` FULL
    default is now the IMPORT lane (`MUT_OUTCOMES` + `MUT_OUTCOMES_SHA`, with
    the provenance checks: SHA resolves → HEAD equality → measured-surface
    clean) and fails closed with the box-lane instructions when no import is
    given; `MUT_FULL_LOCAL=1` re-enables the local sweep, which now caps
    `NEXTEST_TEST_THREADS` to 2 in FULL (operator's value wins; DIFF keeps
    the profile default — applied via a per-invocation env prefix, not an
    export, so later gates are untouched); the header comment records the
    ban, the lanes, and the scoped-verification pointer. `need cargo-mutants`
    moved below the import branch (the import path needs only jq).
  - `.claude/hooks/lib-hook-helpers.sh` — `gate_state_hash` path lists gain
    `rust-toolchain.toml` (both tracked + untracked halves).
  - Checks: `bash -n` both files OK; `shellcheck` clean (gate:11's own bar).
- **Deviations from the resume design:** none.
- **2026-08-13 (goal-directed fast path, Chad /goal):** the REQ-003 slice
  landed the same evening the enumeration completed — 6 killer tests + 1 D3
  redundancy deletion across 6 files, validated by per-site scoped runs
  (0 missed) instead of a full sweep; see the Phase-2 DESIGN ADDENDUM for
  the complete ledger. Still open for this pipeline: REQ-004 (classify the
  54 timeouts), the REQ-002 sabotage smoke, the mutation_g gate split
  re-plumb (local scoped / box lane), and the deferred full-gate re-run
  (REQ-005 as amended).

## Phase 3.5 — Inspect
- **Critics:** 2 spawned (shell correctness; §15 circumvention). Table below
  as reports land.
- **Lead function-level smokes (the REAL sed-extracted functions driven in a
  harness; end-to-end gate runs stay at validate):**
  | # | Scenario | Result |
  |---|---|---|
  | S1 | FULL, no import, no opt-in | RC=1 + the ban/instruction block ✓ |
  | S2 | import without `MUT_OUTCOMES_SHA` | RC=1, pointed message ✓ |
  | S3 | SHA not a commit | RC=1 ✓ |
  | S4 | ancestor SHA (514e5b9) vs HEAD | RC=1, both SHAs named ✓ |
  | S5 | measured surface dirty (scratch .rs under crates/) | RC=1, refused ✓ (cleaned) |
  | S6 | provenance passes → verdict over the OLD box pull | audit line `54 timeout(s); 15 mislabeled` — **independently matches the REQ-004 classification** — then `3920 caught / 16 missed → MSI 99.6%` RED ✓ (the old data SHOULD fail; the fresh run is the green candidate) |
  | S7 | verdict zero-total arms | diff→pass, full→fail-closed ✓ |
- **Critic findings ledger (both reports in; fixes applied + re-smoked):**
  | Sev | Finding (critic) | Verdict | Fix |
  |---|---|---|---|
  | MED-HIGH | SHA attestation is operator memory: symbolic refs (`HEAD`/`main`) make the pin a tautology, and NO artifact in a cargo-mutants pull records tree identity (verified: lock.json = version/time/host only) — an honest stale pull imports green (circumvention) | REAL | Layered belts: hex-literal ≥7 required (S9/S10 RED); optional `MEASURED_SHA` sidecar enforced when present (pull recipe writes it; S13 RED on mismatch); freshness — import mtime must be ≥ HEAD's commit time (S11 RED on the real stale pull); foreign-dir probe (first enumerated file must exist in HEAD). |
  | MED | No completeness check — a partial/mid-sweep pull (or a sloppy iterate merge) gates as a FULL verdict (circumvention + lead's subset finding) | REAL | Set-equality: non-baseline outcome names must EXACTLY cover mutants.json (S12b: 1999 vs 4784 → RED with the merge-note pointer); the header gained the exact jq merge recipe, and the check makes it enforced, not advisory. |
  | MED | `MUT_FULL_LOCAL=0` (any non-empty value) selected the banned local sweep — flag conventions disagreed with GATE_FAST (shell) | REAL | Gate on `= "1"` exactly (S8: `=0` now lands in the ban path). |
  | MED | Moving-ref acceptance (shell critic's independent duplicate of the tautology) | REAL (same fix) | covered above. |
  | LOW | Corrupt/truncated outcomes.json → DIFF false green via the zero-total arm (pre-existing, inherited by the extraction) | REAL | `jq empty` validation at the top of `mutation_verdict` — invalid JSON fails closed in every mode. |
  | LOW | Relative `MUT_OUTCOMES` error names a path that exists in the CALLER's cwd (gates.sh cd's to repo root) | REAL (cosmetic) | error now includes `(cwd $PWD)`. |
  | LOW-MED | `.config/nextest.toml` (the terminate ceiling — gate-defining per its own header) in NEITHER fingerprint (pre-existing; same class as the rust-toolchain.toml find) | REAL | added to both `gate_state_hash` path lines. |
  | LOW | Box `-E`/retry forgiveness direction | ruled FAIL-SAFE by both analyses (excluded-killer ⇒ MISSED ⇒ RED; retries suppress flaky-false-CAUGHT) | none; `MEASURED_ENV` sidecar → the runner rider list (Chad). |
  | LOW | `MUT_FULL_LOCAL` not host-blocked on the mini | posture-consistent (§15: deliberate acts are out of scope; env-opt-in + comment is the amendment style) | none. |
  | note | extensionless legacy `rust-toolchain` filename would evade the pathspec | repo uses `.toml` (the case arm covers it) | none; noted. |
  - **Clean lenses:** SC2155 declaration/assignment split; verdict return
    propagation; bash-3.2 constructs; untracked-under-crates caught by
    porcelain; env-prefix subshell semantics; receipt-scope-with-dirty-scripts
    SOUND (mutants never measure scripts; the gates that do judged this same
    worktree); `mutation_verdict` extraction verdict-neutral; tool-check
    ordering (no rm-before-need data loss).
- **Post-fix verification:** `bash -n` + shellcheck clean on both files;
  smokes S8–S13 all RED where they must be; S12a proves the FULL belt chain
  end-to-end (all provenance belts pass on a complete fresh-mtime import →
  the verdict then honestly REDs the old data's own 16 missed).
- **Lead finding (real, MED): iterate imports are SUBSET verdicts.** An
  `--iterate` run's `outcomes.json` records ONLY run-local rows (seed-credited
  mutants are skipped, not re-recorded — the 08-13 evidence: 4,785 rows vs
  the 5,830 merged record). A fresh-pull import therefore measures the
  retested subset; the whole-workspace FULL claim needs the MERGED outcomes
  (seed archive + fresh run, `jq -s` concat; audit logs: fresh log/ tree +
  archive's — the audit's `[ -f ]` guard skips absent logs, an
  under-count of mislabels only, never a count change). Fix: pull-recipe
  merge documented in the gate comment + used at validate. |

## Phase 4 — Validate
- **Gate-is-test posture (§7):** the shipped change is scripts/hooks — verified
  by real exit codes + negative smokes; no unit tests to add. The .rs-touching
  work of this pipeline (REQ-003's killer tests) shipped + was scope-verified
  in 12b78c8.
- **REQ-002 sabotage smoke:** `<`→`<=` flipped at the `partition_point`
  (app.rs:16727) → `efind_replace_resume_boundary_keeps_adjacent_match_headless`
  **FAILED** (1/1 red); reverted (crates diff empty) → **PASS** (1/1 green).
  #404's killer is live and load-bearing.
- **Workspace suite (formal):** `cargo nextest run --workspace` →
  **2169/2169 passed, 7 skipped** (the headed ignores); doctests: 18 ok lines.
- **--diff gate regression:** `scripts/gates.sh --diff` after the re-plumb →
  **GATE GREEN [diff], 15/15** (gate:5 took the DIFF lane: no crate lines in
  the diff → the documented trivial pass; the per-commit loop is untouched).
- **Import-lane negative smokes:** S1–S13 at inspect (real extracted function
  text, exit-code verdicts): ban path, missing/short/symbolic/wrong SHA, HEAD
  mismatch, dirty surface, stale mtime, partial outcomes (1999/4784
  set-inequality), wrong sidecar — every one RED; S12a proved the full belt
  chain green-through-to-verdict on a complete import.
- **Live-app capture: N/A — no UI delta** (gate infrastructure only).
- **REQ-005 — DONE, measured: GATE GREEN [full] via the import lane.**
  - Box re-run finished `rc=3 2026-08-14 11:28:02` (38 min at jobs=2):
    **136 tested: 65 caught, 17 unviable, 54 timeouts, 0 MISSED** — all 16
    ex-survivors dead on retest; every new #423 session.rs mutant caught; the
    hang class re-hung at its post-423 line numbers (296→335, 359→402 —
    drift exactly as the merge design predicted).
  - Merge reality check: the fresh pull's `mutants.json` holds only the 136
    RETESTED mutants (an `--iterate` artifact) — the honest completeness
    manifest is the CURRENT tree's own `cargo mutants --list` (5,834; safe
    under the ban — enumeration only). And the missing rows after a 2-way
    merge were EXACTLY the 1,046 iterate-credit generation → the merge needs
    all THREE generations (gen0 = the 2026-08-12 partial backup, gen1 = the
    08-13 sweep, gen2 = today), newest wins, enum-filtered. Result:
    **5,834/5,834 covered, 0 missing, 0 stale.**
  - The REAL `scripts/gates.sh` (FULL) with `MUT_OUTCOMES=./mutants-box-import`
    + the full HEAD sha: every provenance belt passed; gate:5 verdict
    `mutation: 54 timeout(s); 15 mislabeled … 4832 caught / 0 missed → MSI
    100.0% (floor 100%)` — the 54/15 audit line matches the REQ-004
    classification exactly. **GATE GREEN [full], 15/15; receipt written
    (11:31).** The first FULL green of the audit-debt saga — "assumed green"
    is now MEASURED green, and the import lane itself is proven end-to-end
    by the same run.
  - Honesty caveats carried: gen0+gen1 credit was measured under box rustc
    1.94 (pre-pin); the 136-retest generation is pinned-1.96. The runner's
    `-E` exclusions still applied to the box generations (the four excluded
    tests' killer coverage was cross-checked locally at 423). The next
    natural FULL re-measure is same-compiler, no-exclusion end to end.

## Phase 5 — Complete
- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (TICKET-407 — the
  measured FULL green + the lane split + the REQ-004 record);
  `docs/specs/standards/quality-bar.spec.md` Modes section now documents the
  gate:5 lane split (import default + belts + the MUT_FULL_LOCAL override).
  React parity: N/A.
- **Ledger appends:**
  `AD-claude-407-full-mutation-is-an-imported-verdict-behind-artifact-belts-001`;
  `PR-claude-407-an-imported-verdict-needs-artifact-derived-belts-001` (at
  inspect). Earlier 407-era appends (the sweep/OOM forensics lessons) were
  captured at their own phases.
- **Ticket:** closed with the full deliverable story + the accumulated
  runner-rider list for Chad (exclusions drop; MEASURED_SHA authoritative;
  lock-fd CLOEXEC + orphan reap); moved to `tickets/closed/`. BACKLOG has no
  stale row (its Deliberate row was removed at the resume promotion — checked).
- **Archive:** pair → `pipeline/completed/`.
- **Import artifacts kept:** `./mutants-box-import/` (the merged 5,834-row
  record + MEASURED_SHA — the REQ-005 measurement of record, gitignored);
  gen pulls at `./mutants-box/`, `./mutants-box-fresh/`,
  `target/mutants.out.partial-407-20260813/`.
