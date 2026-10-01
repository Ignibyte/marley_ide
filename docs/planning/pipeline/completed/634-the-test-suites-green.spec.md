---
pipeline_id: f1626f53-313f-4c6f-97d6-a245dfaf4a30
ticket: docs/planning/tickets/open/TICKET-634-the-test-suites-green.md
status: Phase 4 — Complete PASS
title: "The test suites run and green"
type: chore
slice: wave 5, the test pass (after #475)
references: [docs/planning/design-notes/remaining-work-2026-09-30.md]
---

## Title
The tests in the tree have not run since #483 (2026-09-23). This ticket runs the Marley crates'
suites and the Marley tests inside the Zed crates Marley changed, and makes them pass without
adding a test: a stale test follows the behavior its ticket shipped; a real bug is fixed in code.

## Scope
### In
- **Tier 1, must be green:** every test target of the eleven Marley crates (`marley_agent`,
  `marley_browser`, `marley_dcs`, `marley_fleet`, `marley_mcp`, `marley_rail`, `marley_remote`,
  `marley_sdk`, `marley_system_one`, `marley_terminal`, `marley_workbench`), and the tests Marley
  added inside Zed crates (since the 2026-09-18 fork point `78648aaf7d`: `paths` 2, `terminal` 10,
  `terminal_view` 13, `zed` 1).
- **Tier 2, triaged:** the full suites of the 21 Zed crates whose Rust Marley changed (`agent_ui`,
  `editor`, `feature_flags`, `git`, `git_ui`, `git_ui_core`, `gpui`, `gpui_linux`, `markdown`,
  `markdown_preview`, `paths`, `project`, `settings`, `settings_content`, `settings_ui`, `task`,
  `terminal`, `terminal_view`, `title_bar`, `workspace`, `zed`), since a Marley hunk can break a
  Zed test beside it. A failure there that a Marley hunk causes is fixed like a Tier 1 failure;
  one that involves no Marley hunk is named with why (upstream, or this box) and left.
- `cargo nextest run` under Zed's own profile (`.config/nextest.toml`: a test ends after 60 s),
  one cargo command at a time, its log kept.
- Triage of every failure: **stale** (a ticket since 2026-09-23 changed the behavior on purpose:
  the test's expectation follows the shipped behavior, and the notes name the ticket),
  **non-deterministic** (gpui's "Your test is not deterministic": the test lets real IO park, as
  Zed's own tests that run real shells do, or awaits it), **harness** (the test's own setup is
  stale: a global it reads that nothing sets), **bug** (the code is fixed, an `F-…` block at
  Complete), or **cannot run here** (a missing program, the display).
- The run again, green.

### Out (explicitly deferred)
- New tests (CONSTITUTION §0); coverage; the golden e2e set (#635); mutation (#636).
- Zed crates Marley never changed; doc tests.

## Reference (§20)
N/A — Marley-specific: the test debt of Marley's own workflow; Zed's tests in untouched crates
stay Zed's.

### Prior art
- **Code we already ship:** the suites in `crates/marley_*/src/*_tests.rs`, `#[cfg(test)]`
  modules and `crates/marley_*/tests/`; `cargo-nextest` 0.9.143 with Zed's profile
  (`.config/nextest.toml`: a 60 s end per test, the `db` tests serial); gate:2, which builds every
  target, so the tests compile today; gpui's `cx.executor().allow_parking()`, which Zed's own
  real-shell tests call (`terminal_panel.rs`) and Marley's routing, blocks and agent-bar tests
  already use; #475's scratch data folder for the three binaries that start shells.
- **Knowledge:** AD-claude-483-e2e-visualization-tests-replace-unit-tests-001 (the tests kept,
  run by no gate); L-claude-465-zeds-clippy-bans-std-process-in-tests-too-001;
  F-claude-443-e-a-hup-ignoring-test-child-outlived-its-test-001 (a PTY child outlives its test
  process); L-claude-475-tell-a-stale-test-from-your-change-by-running-it-without-the-change-001.
- **Behavior maps / published material:** none needed.

## UI proof
The suites' logs are the proof of REQ-001 to REQ-003. One fix shows in the app: a terminal whose
prompt editor has the keys counts as focused. `script/e2e/634-the-prompt-editor-keeps-its-terminal-focused.sh`
drives ctrl-` from the prompt editor back to the code (`634-04-code`), and the same scenario on
the build before the fix shows the terminal kept.

## Locked-In Decisions
- D1 — No new test; a test changes only to follow behavior a ticket shipped, with the ticket named.
- D2 — The shipped behavior is the reference: a test is never made to pass by changing code that
  works as its ticket meant.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Tier 1 runs, every test shall pass, or be named in the notes with why it cannot run on this box. | The green run's log |
| REQ-002 | WHEN Tier 2 runs, every failure shall be fixed if a Marley hunk causes it, and otherwise named with why. | The Tier 2 log and the notes' table |
| REQ-003 | WHEN a test failed in the first run, the notes shall give its class (stale with its ticket, non-deterministic, harness, bug with its fix, cannot run here). | The notes' triage table |
| REQ-004 | The diff shall add no test function. | Review of the diff |

## Phase Plan
- **P1 Plan** — the crate list; the run's command.
- **P2 Code** — the first run, the triage, the fixes; the gate green.
- **P3 Test** — the green run; `just shot`.
- **P4 Complete** — knowledge (each bug an `F-…`); close, archive, commit.
