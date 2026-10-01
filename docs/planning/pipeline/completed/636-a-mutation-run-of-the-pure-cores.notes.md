# A mutation run of the Marley crates' pure cores — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-636-a-mutation-run-of-the-pure-cores.md
- **Pipeline spec:** 636-a-mutation-run-of-the-pure-cores.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** wave 5 (Chad, 2026-09-30); the workflow keeps mutation for the end.
- **Recall (§18.3):** the mutation decisions and failures named in the spec's Prior art; the
  shared target dir fills `/mnt/fast` (the memory's rule: drop Marley's old incremental variants
  when it reaches 100%).

## Phase 1 — Plan (promoted 2026-10-01)
- **Classification / tier:** chore; a report, and code only where a survivor shows a bug.
- **Recall (§18.3):** AD-claude-443-mutation-topology-and-no-masks-001 (the full run: copy mode,
  two workers, a target per copy, `--no-config`, no masks); F-claude-443-a-full-mutation-workers-
  shared-one-target-001 (copies sharing a target report wrong verdicts and poison the main
  tree); BF-claude-mutation-survivors-loopbound-childbin-serial-001 (env races under threaded test
  runs, equivalent loop-bound mutants); the retired `script/mutation.sh` (bffb491dc9^), whose
  topology this run repeats. The brain (consultation `2715f214…`) listed follow-ups due, nothing
  on the question.
- **Discovery:** `cargo mutants --list` per Marley crate: the nine with no gpui hold 2,658
  mutants; `marley_browser` (1,197) and `marley_workbench` (5,442) depend on gpui and are out.
- **Disk:** `/mnt/fast` 37G free (97%); the run's copies and targets go to `/home` (1.3T free).

### Design
- **Approach:** one `cargo mutants` invocation over the nine crates, as the retired script ran
  it: `env -u CARGO_TARGET_DIR -u CARGO_MUTANTS_OUTPUT -u CARGO_MUTANTS_MINIMUM_TEST_TIMEOUT -u
  CARGO_BUILD_TARGET_DIR -u CARGO_BUILD_BUILD_DIR TMPDIR=~/.cache/marley-mutants cargo mutants
  --no-config --no-times --test-tool=nextest --jobs 2 -p … --output <scratchpad>`, in the
  background, its log kept. Then each missed mutant read at its place and marked **untested**,
  **equivalent** or **bug**; each bug fixed; the findings doc and the intake written.
- **File manifest:** `docs/planning/design-notes/mutation-run-2026-10.md` (new);
  `docs/planning/intake/kill-mutation-survivors.md` (new); Marley crate sources only for a bug.

### Visual check plan
| REQ | What runs | What shows it |
|---|---|---|
| REQ-001, REQ-002 | the run; `outcomes.json` | the findings doc's per-crate counts and its survivor table |
| REQ-003 | a bug's fix | its crate's suite green; the mutant caught on a rerun of that file |
| REQ-004 | `git diff` | no test function added |
| §7 | `just shot` | Marley starts as before |

### Risks
- Hours of run time: about 2,658 mutants at two workers; the box stays usable (separate targets),
  but no other cargo runs in the main tree while the copies build, per the box's rule of thumb.
- A baseline that fails in a copy (a test that needs the user's environment) stops the run: it
  is named and fixed at its cause, never skipped.

## Phase 2 — Code
- **The run** (07:00 to 08:28, 1 h 27 min): the baseline passed; 2,658 mutants, 961 caught, 12
  timeouts, 1,448 missed, 237 unviable: 40.2% of viable killed. `marley_fleet`, `marley_sdk` and
  `marley_system_one` 100%; `marley_agent` 3.2%. Exit 3 (timeouts), a complete run. The working
  tree was never touched (copy mode); `git diff -- crates` is empty.
- **The reading:** `marley_agent`'s own suite has 5 tests, and cargo-mutants runs only the
  mutated crate's tests, so modules added since #483 lose nearly every mutant. Each survivor was
  sorted by whether a test of its crate names its function: 1,358 do not (untested by
  construction, 35 of 68 files have no unit test); the 90 that do were read one by one: 2
  equivalent (`TerminalSession::shutdown` to `()`, the reap's log-only `!=`), 88 untested
  branches, **no bug**. So no code changed.
- **Written:** `design-notes/mutation-run-2026-10.md` (how it ran, the counts per crate and per
  file, the reading, the survivors worth a test first) and `mutation-run-2026-10-survivors.md`
  (all 1,448, each with its place, function, mutation and mark); `intake/kill-mutation-survivors.md`
  (Chad's decision: keep the rule, a survivors pass per sprint, or tests for the risky paths).
- **A finding about #634:** its dispatch test now compares `tools/list` with the registry's own
  list, so `is_served` replaced by `true` survives; noted in both docs.
- **Gate:** docs only, `script/gates.sh --fast`: `GATE GREEN [fast]`, 16 passed.

## Phase 3 — Test
- REQ-001 and REQ-002: the findings doc's per-crate and per-file tables come from the run's
  `outcomes.json`; the survivors file lists all 1,448 with place and mark. REQ-003: no survivor
  read as a bug, so no code changed and no suite needed rerunning. REQ-004: no test function in
  the diff (it holds no Rust).
- **§7 visual check:** `just shot 636-marley-starts` (exit 0): Marley starts as before, the trust
  prompt over the scratch repository, the rail with its terminal and the machine's containers,
  the terminal's prompt. Focus report: "the user's window and workspace are as they were". The
  shot stays in the scratchpad.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Changed: the mutation run, reported); the findings doc and the
  survivors file in `design-notes/`; the intake; wave 5 marked done in
  `design-notes/remaining-work-2026-09-30.md`. No code, no Zed path.
- **Ledger:** L-claude-636-a-mutation-run-reads-only-the-mutated-crates-tests-001. No bug, so no
  `F-` block.
- **Brain:** `brain decide` on the consultation (`2715f214…`).
- Ticket closed, pipeline archived, committed.
