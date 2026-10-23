---
pipeline_id: 5dfb1b3e-e8f2-45c9-8b29-463d29d95a5d
ticket: docs/planning/tickets/open/TICKET-932-recordings-and-script-runs.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Recordings with a timeline and Draft a Test, and scripts' last run
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/design-review-2026-10-23/README.md]
---

## Title
A recording shows its minute as the BrowserTools board draws it: activity bars over the minute,
the steps at their times, and Draft a test beside Send to the agent. Record This opens it in the
drawer. Each Playwright script in the tray shows when it last ran and its result.

## Scope
### In
- **The minute in the drawer** (`crates/marley_workbench/src/browser.rs`, Marley crate). A click
  on a row of the drawer's Recordings tab expands it, and a second click folds it:
  `ExpandedRecording`, new, read off the main thread with `recorder::read_in`. Under the row:
  twenty bars, one per three seconds of the minute, each as tall as the entries in it (frames
  and snapshots not counted), accent when a step falls in it; then the steps, each at its time
  before the save (`−0:41`) and in words (`step_words`, new); then Draft a test and Send to the
  agent. `recording_row`, `drawer_rows`, `render_drawer`.
- **Record This opens it** (`browser.rs` `record_this`): once saved, the drawer opens on
  Recordings, the list is read again and the new recording is expanded. A failure shows in the
  drawer's error line, as Send's does (#892); the success toast goes.
- **Draft a test** (`browser.rs`, `crates/marley_workbench/src/browser_tools.rs`, Marley crate):
  the button runs `draft_in` (made `pub(crate)`) off the main thread with the agent redactor, and
  opens the draft's text in a new unsaved TypeScript buffer in the tab's workspace, headed by a
  comment naming the suggested path and the run command. A refusal (no action to replay, a start
  page a test cannot open) shows in the drawer's error line.
- **Send to the agent** under the minute: the row's existing Send line (`recording_line`, #892).
- **Last run kept** (`crates/marley_workbench/src/playwright_scripts.rs`, Marley crate):
  `LastRun { ran_at, result }` and `RunResult { Passed, Failed { exit_code }, Unknown }`, new;
  `runs_in(data)` and `record_run_in(data, script, last_run)` keep them in
  `<data>/playwright/runs.json`, by the script's path. `run_command` records the result when the
  run's block ends, and the hub emits `BrowserEvent::ScriptRan { project }` (new variant) so an
  open tray reads again.
- **The script row** (`browser.rs` `render_script_row`, `read_scripts`): the name over
  "<scope> · ran <when>", the result coloured (passed in the success colour, "failed · exit N"
  in the error colour, "ended" and "not run" muted), Run, and Edit. `Scope::label` reads "every project" for
  the global scope, as the board does.

### Out (explicitly deferred)
- A draft kept as a script in the tray, the board's "drafted from a recording" state: a draft is
  a `@playwright/test` spec and the tray runs Marley's runner modules (AD-claude-523), and Marley
  writes no draft into the project (AD-claude-506 D4). A follow-up ticket decides whether the tray
  keeps specs.
- A count of failed tests ("1 failed"): Marley's runner reports an exit code, not a test count.
  The row says "failed · exit N".
- A run history beyond the last run, and a running state in the row.
- The bottom row's "recording the last minute": #909 (the bottom row for the other screens).
- The new-script field's wording and scope: #931.

## Reference (§20)
N/A — Marley-specific: Marley's flight recorder (#499), its draft (#506) and its Playwright
scripts (#523), drawn as the BrowserTools board draws them. Zed's `Project::create_buffer` and
`Editor::for_buffer` open the draft as an unsaved buffer, as Zed's agent panel opens text that
has no file.

### Prior art
- `browser.rs:6622` `record_this`: saves, then a toast. `browser.rs:7339` `recording_row`: when,
  title, URL, length and Send; `RecordingRow::of` (`:8436`) reads `recorder::list_in`'s summary,
  which drops the entries (`marley_browser/src/recorder.rs:781`).
- `recorder.rs:820` `read_in` gives a recording's timeline: `entries` with `kind` and `at_ms`
  (`TimedEntry`, `:428`), `seconds`, `project`. `Entry::Action` keeps `action`, `locators`,
  `key`, `field`; `Entry::Navigation` keeps `url` and `within` (`recorder.rs:240-345`).
- `browser_tools.rs:718` `draft_test` and `:743` `draft_in`: the draft, its suggested `path`, `run`,
  `note`, from `marley_browser::playwright::draft` (`playwright.rs:56`). AD-claude-506: the draft
  writes nothing and the project keeps the test.
- `agent_ui/src/conversation_view.rs:7188` opens text with `Project::create_local_buffer`;
  `agent_ui/src/ui/mention_crease.rs:313` notes that call panics on a remote project.
- `playwright_scripts.rs:77` `Script` holds name, scope and path, no runs; `:256` `run` and
  `run_command` already know the exit code from `block_end` and write it into the recorder
  (`BrowserHub::record_script`, `browser.rs:2304`).
- `browser.rs:7866` `render_script_row`: name, "for <scope>", Run and Edit; `:6489`
  `read_scripts`.
- Relative times: `clients.rs:754` writes "last call N min ago" with plain arithmetic; Zed's
  `time_format::TimestampFormat::Relative` does the same through the `time` crate, which
  `marley_workbench` does not depend on.
- docs/zed_architecture/ and docs/warp_architecture/: nothing on this seam.

## UI proof
Scenario `script/e2e/932-recordings-and-script-runs.sh` (sway, `browser-fixture.sh`): an offline
Chromium; #506's loopback page, where the user clicks, types into a field and follows a link;
#523's loopback npm registry for Marley's `playwright-core`; two scripts kept for the project,
one that passes and one that throws, and a third never run. The terminals' `.bashrc` sets the
prompt only; no agent CLI is started.
- `932-01-minute`: clicks and typing in the page, then `marley: record this`: the drawer on
  Recordings with the new recording expanded, its bars, its steps and the two buttons (REQ-001,
  REQ-002).
- `932-02-draft`: Draft a test: the editor tab with the drafted test and its path comment
  (REQ-003).
- `932-03-scripts`: both scripts run, the tray open: "passed", "failed · exit 1" and "not run",
  each with its scope and when (REQ-004).
- `932-04-day`: Marley Day, the minute expanded and the tray (REQ-001, REQ-004).

## Locked-In Decisions
- D1 — The minute shows in the drawer's Recordings tab, under its row, not in a card of its own:
  the drawer already holds the recordings and their Send (#892).
- D2 — A bar is three seconds; twenty bars span the minute. Its height counts clicks, scrolls,
  keys, typing, actions, navigations, console entries, requests and agent actions; frames and
  snapshots are not activity. A bar is accent when a step falls in it.
- D3 — A step is a user action (click, fill, press) or a navigation to a new document. Its words
  name the element by its first locator that reads as words (role and name, label, placeholder,
  text, test id), never a fill's text: "Open 127.0.0.1:4180", "Click Save changes", "Type in
  Email", "Press Enter".
- D4 — Draft a test opens the draft an agent would get (the same `draft_in`, through the agent
  redactor) in an unsaved buffer. Marley writes nothing into the project (AD-claude-506 D4); the
  user saves it where they choose.
- D5 — The last run is kept in Marley's data folder, not beside the scripts in the config folder:
  it is state, not something the user edits. It is keyed by the script's path, so a script with
  the same name in both scopes keeps its own.
- D6 — A run whose shell does not say how it ended is "ended" in the muted colour; a run whose
  terminal closed first is not recorded.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs Record This, the drawer shall open on Recordings with the new recording expanded, showing twenty activity bars over the minute, in Marley Night and Marley Day. | `932-01-minute`, `932-04-day` |
| REQ-002 | An expanded recording shall list its steps at their times before the save, in words, with Draft a test and Send to the agent under them. | `932-01-minute` |
| REQ-003 | WHEN the user clicks Draft a test, Marley shall open the drafted test in a new unsaved editor tab headed by its suggested path, and write no file. | `932-02-draft` |
| REQ-004 | Each script in the Playwright tray shall show its scope, when it last ran and its result: passed, failed with its exit code, or not run. | `932-03-scripts`, `932-04-day` |
| REQ-005 | WHEN a script's run ends, Marley shall keep its time and result in its data folder, and an open tray shall show them. | `932-03-scripts`, the diff review (`runs_in`, `record_run_in`) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the minute in the drawer, Draft a test, the last run kept and shown; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
