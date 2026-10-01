---
pipeline_id: 71dad0e4-1cb3-4700-aeb6-4ebc01305798
ticket: docs/planning/tickets/open/TICKET-475-shell-tests-scratch-data-dir.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "The shell tests install Marley's scripts in a scratch directory"
type: chore
slice: prong 1 T0 test hygiene; wave 5, first (before #634 runs the suites)
references: [docs/planning/pipeline/completed/474-block-hover-actions.notes.md]
---

## Title
The PTY tests of #463, #465, #474 and #466 start shells through `TerminalBuilder::new`, which
installs Marley's integration scripts in `paths::data_dir()/shell_integration`: the user's own
`~/.local/share/marley`. Before #634 runs the suites, each such test process uses a scratch data
directory, so a run never rewrites the scripts the user's terminals read.

## Scope
### In
- Every test that reaches `install_in(paths::data_dir()…)` through `TerminalBuilder::new` (the
  `terminal` crate's Marley PTY tests, and any `terminal_view` or `marley_*` test that builds a real
  PTY terminal) sets a scratch data directory first (`paths::set_custom_data_dir`, once per test
  process, to a temporary folder kept for the process's life).
- A check that a run of those tests leaves `~/.local/share/marley/shell_integration` as it was
  (its files' modification times before and after).

### Out (explicitly deferred)
- Running the suites to green (#634).

## Reference (§20)
N/A — Marley-specific test hygiene: Zed's own tests set no data directory because Zed installs no
shell scripts.

### Prior art
- **Code we already ship:** `paths::set_custom_data_dir` (Zed's `paths` crate, a `OnceLock` set
  once per process, which `--user-data-dir` uses); `marley_terminal::shell_integration::install_in`
  (takes the directory it writes to); the PTY tests in `crates/terminal/src/terminal.rs`
  (`finished_block_of`); `tempfile` among `terminal`'s dev-dependencies (#463).
- **Behavior maps / published material:** none needed.

## UI proof
N/A — no UI delta: a test-only change. Its proof is the test run's log and the scripts' times
before and after it; the visual check owed by §7 is `just shot`.

## Locked-In Decisions
- D1 — One scratch directory per test process, set before any terminal is built: the data dir is
  a process-wide `OnceLock`.
- D2 — No test is added; the existing ones change only where they build a terminal.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the `terminal` crate's tests run, they shall leave `~/.local/share/marley/shell_integration` unchanged. | The scripts' modification times before and after the run, in the notes |
| REQ-002 | WHEN those tests start a shell, it shall still load the scripts from the tree. | The tests' own checks pass (their log) |

## Phase Plan
- **P1 Plan** — find every test that builds a PTY terminal (Explore).
- **P2 Code** — the scratch directory; the gate green.
- **P3 Test** — the run and the times; `just shot`.
- **P4 Complete** — knowledge; close, archive, commit.
