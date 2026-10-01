---
pipeline_id: 71dad0e4-1cb3-4700-aeb6-4ebc01305798
ticket: docs/planning/tickets/open/TICKET-475-shell-tests-scratch-data-dir.md
status: Phase 4 — Complete PASS
title: "The shell tests install Marley's scripts in a scratch directory"
type: chore
slice: prong 1 T0 test hygiene; wave 5, first (before #634 runs the suites)
references: [docs/planning/pipeline/completed/474-block-hover-actions.notes.md]
---

## Title
Three test binaries reach Marley's install of its shell scripts, which writes
`paths::data_dir()/shell_integration`, the user's own `~/.local/share/marley`: the unit tests of
`terminal` (its bash and zsh PTY tests, the two that call `marley_shell_integration` directly, and
Zed's `Shell::System` tests), `terminal_view` (Zed's title and panel tests, which start the system
shell) and `marley_workbench` (the routing tests, which open center terminals; its other modules
read and write `paths::data_dir()` too). Before #634 runs the suites, each of those test binaries
sets a scratch data directory before its first test, so a run never writes the user's data.

## Scope
### In
- `terminal::marley_use_test_data_dir()`, behind `test-support`: `paths::set_custom_data_dir` on
  `marley-test-data` beside the test binary's `deps` folder (the build profile's folder, inside the
  user's own target directory).
- In each of the three test binaries, a `#[ctor::ctor(unsafe)]` function that calls it, so it runs
  before `main` and before any test reads the data directory; `ctor` among each crate's
  dev-dependencies.
- A run of the tests that reach the install with `XDG_DATA_HOME` on an empty folder: the folder
  stays empty, and the scripts are in `marley-test-data`. The user's scripts' times, before and
  after.

### Out (explicitly deferred)
- Running the suites to green (#634): a test that fails for another reason is #634's.
- Zed's other test binaries that may read the data directory without starting a shell
  (`agent_ui`, `workspace` …): none of them reaches the install.
- Making the install's write atomic for test processes that share the folder (Risks).

## Reference (§20)
N/A — Marley-specific test hygiene: Zed's own tests set no data directory because Zed installs no
shell scripts; the mechanism is Zed's own (below).

### Prior art
- **Code we already ship:** `paths::set_custom_data_dir` (`crates/paths/src/paths.rs`), which
  Zed's `--user-data-dir` calls (`crates/zed/src/main.rs`, `crates/cli/src/main.rs`): a
  `OnceLock` that must be set before `data_dir()` or `config_dir()` is first read, and moves
  `config_dir()` under it too. Zed's run-once-before-the-tests pattern,
  `#[cfg(test)] #[ctor::ctor(unsafe)] fn init_logger()` (`crates/editor/src/test.rs`,
  `action_log`, `agent`, `file_finder`), with `ctor = "1.0.12"` in the workspace; its expansion
  carries its own `#[allow(unsafe_code)]`, so `marley_workbench`'s `unsafe_code = "deny"` holds.
  `marley_terminal::shell_integration::install_in` (writes only a changed file). `terminal`'s
  `test-support` feature, which `terminal_view` and `marley_workbench` already enable for their
  tests.
- **Checked, not used:** a ctor in a library behind a feature (it would run in any binary that
  turned the feature on); `XDG_DATA_HOME` set by the test (`std::env::set_var` is `unsafe` in
  edition 2024, and `marley_workbench` denies unsafe code); a per-process temporary folder (no
  destructor runs at exit, so nextest's one process per test would leave one folder per test).
- **Behavior maps / published material:** none needed.

## UI proof
N/A — no UI delta: a test-only change. Its proof is the run's log: the tests that reach the
install run with `XDG_DATA_HOME` on an empty folder, which stays empty, and the user's scripts
keep their times; the visual check owed by §7 is `just shot`.

## Locked-In Decisions
- D1 — A `ctor` per test binary, Zed's own pattern: the data directory is a process-wide
  `OnceLock`, and only a function run before `main` sets it before every test, under cargo test's
  threads as under nextest's processes.
- D2 — One folder per build profile, shared by the three binaries and their processes, inside the
  user's target directory: nothing left behind per process, no fixed path under `/tmp`, and
  `cargo clean` removes it. The tests shared the user's folder the same way until now.
- D3 — The helper lives in `terminal` behind `test-support`, a function only: nothing runs unless a
  test binary's own `ctor` calls it.
- D4 — No test is added; the existing ones change in no line.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the tests of `terminal`, `terminal_view` and `marley_workbench` that start a shell run, they shall write nothing under the user's data directory. | A run with `XDG_DATA_HOME` on an empty folder: it stays empty (the run's log); the user's scripts' times before and after |
| REQ-002 | WHEN those tests start a shell, it shall load Marley's scripts from `marley-test-data`. | The scripts in `marley-test-data/shell_integration` after the run; the bash and zsh PTY tests' block checks, which pass only with the integration loaded (their log) |
| REQ-003 | The diff shall add no test function and change no test's body. | Review of the diff |

## Phase Plan
- **P1 Plan** — find every test that builds a PTY terminal (Explore).
- **P2 Code** — the scratch directory; the gate green.
- **P3 Test** — the run and the times; `just shot`.
- **P4 Complete** — knowledge; close, archive, commit.
