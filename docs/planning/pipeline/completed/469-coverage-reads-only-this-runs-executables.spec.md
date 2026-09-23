---
pipeline_id: 86822222-9108-4b28-b954-705170a22516
ticket: docs/planning/tickets/closed/TICKET-469-coverage-reads-stale-executables.md
status: Phase 4 — Complete PASS
title: gate:4 reads only this run's executables
type: chore
slice: the gate
references: [docs/planning/knowledge/failures.md, script/gates.sh]
---

## Title
gate:4 reported 45 missed lines in #465 that the tree's tests covered, because cargo-llvm-cov
read a test executable an earlier run had left, built from older source
(`F-claude-465-gate4-counted-lines-from-a-stale-executable-001`). The coverage step now removes
the test executables in the coverage target before it runs, so the report comes only from what
this run builds.

## Scope
### In
- **`rust_cov`** (`script/gates.sh`): before `cargo llvm-cov nextest`, delete every test
  executable in `<target dir>/llvm-cov-target/debug/deps`: the regular, executable files with no
  extension. Libraries, proc-macro `.so` files, `.d` files and the incremental caches stay, so
  the run relinks only the tests it runs, as it already rebuilds the packages it covers.
- **CONSTITUTION**: gate:4's row says the step starts from no test executable.

### Out (explicitly deferred)
- A clean of the whole coverage target (`cargo llvm-cov clean --workspace`): every Zed crate is
  a workspace member, so it rebuilds the world each run.
- Build scripts' coverage (`--include-build-script`): not used.

## Reference (§20)
- N/A — Marley-specific: the gate is Marley's own, and neither Warp nor Zed has a counterpart.

### Prior art
- **Behavior maps:** none.
- **Published material:** cargo-llvm-cov's README asks for `cargo llvm-cov clean` when old
  artifacts skew a report.
- **Code we already ship.** cargo-llvm-cov 0.9.0, read in the registry: `report.rs`'s
  `object_files` walks the target directory and passes every executable whose stem matches
  `pkg_hash_re`, which lists every workspace member's name and targets ("Do not refer
  cx.workspace_members.include"), so executables of packages outside the run are read. By
  default it cleans only the packages it runs (`--no-clean` turns that off).

## UI proof
N/A — no UI delta: a change to the gate script.

## Locked-In Decisions
- D1 — Remove the test executables, not the target: the run builds the ones it needs, and
  everything else it would rebuild stays cached.
- D2 — Every test executable goes, the touched packages' included: cargo-llvm-cov cleans those
  anyway, and a single rule has no list to keep in step with the Marley packages.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN gate:4 runs, it shall read coverage only from test executables built in that run | negative smoke: a stale executable planted from older source reports missed lines without the change and none with it |
| REQ-002 | WHEN the coverage target holds no test executable yet, gate:4 shall run as before | the gate after `llvm-cov-target` holds none |
| REQ-003 | The diff gate shall be green (it runs no coverage when no Marley crate is touched, so the smoke is REQ-001's test) | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the removal in `rust_cov`; the CONSTITUTION row; a review of the diff.
- **P3 Test** — the negative smoke, with and without the change; `script/gates.sh --diff`
  green.
- **P4 Complete** — CHANGELOG and docs (§21), ledger capture (§19), close the ticket, archive,
  commit.
