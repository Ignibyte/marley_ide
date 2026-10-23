# Recordings with a timeline and Draft a Test, and scripts' last run — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-932-recordings-and-script-runs.md
- **Pipeline spec:** 932-recordings-and-script-runs.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, item 11 (BrowserTools).
- **Classification / tier:** feature, L. Marley crates only (`marley_workbench`; `marley_browser`
  is read, not changed). No Zed crate hunk.
- **Recall (§18.3):**
  - AD-claude-499 (the recorder keeps a drawn page's minute), AD-claude-506 (a recording becomes a
    test the project keeps; Marley writes no test into the project, D4), AD-claude-523 (scripts
    in the config folder, run in a terminal beside the tab, the user's act only).
  - #892 D2: the Recordings list is read when its tab is chosen, never on a timer; the expanded
    minute follows the same rule (read on expand, on Record This).
  - F-claude-867-opening-a-project-from-a-page-read-the-page-001: opening an item from a page
    happens from a spawned task, after the page's update ends. Draft a test opens its buffer from
    the task that drafted it.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: the scenario's terminals run Marley's
    runner, not an agent; their `.bashrc` sets the prompt and #523's npm registry variables.
  - The `*_in(dir)` rule (CONSTITUTION §14): `runs_in(data)` and `record_run_in(data, …)` take
    the data folder; the scenario's profile copy gives the run its own.
- **Discovery:** `crates/marley_workbench/src/browser.rs`: `record_this` (6622), `render_drawer`
  (7170), `drawer_rows` (7271), `read_recordings` (7315), `recording_row` (7339), `RecordingRow`
  (8426), `recording_line` (8488), `read_scripts` (6489), `render_script_row` (7866),
  `BrowserEvent` (198), `hub_event` (5479). `crates/marley_workbench/src/browser_tools.rs`:
  `draft_in` (743). `crates/marley_workbench/src/playwright_scripts.rs`: `Scope::label` (66),
  `Script` (77), `run_command` (306–380). Read only: `crates/marley_browser/src/recorder.rs`
  (`Entry`, `TimedEntry`, `read_in`), `crates/marley_browser/src/playwright.rs` (`draft`).
- **Decisions:** D1–D6 in the spec.

### Design
- `browser.rs`:
  - `struct ExpandedRecording { id: String, bars: Vec<ActivityBar>, steps: Vec<RecordedStep>, }`
    with `struct ActivityBar { entries: usize, has_step: bool }` and
    `struct RecordedStep { before_save: Duration, words: String }` (named fields, per `.rules`).
    `BrowserView` gains `expanded: Option<ExpandedRecording>` and `reading_minute: Option<Task<()>>`.
  - `minute_of(timeline: &Value) -> ExpandedRecording`, a free function: buckets by
    `at_ms * 20 / (seconds * 1000)`, clamped to 0..20; steps from `kind == "action"` and
    `kind == "navigation" && !within`.
  - `step_words(entry: &Value) -> Option<String>`: `Open <host and path>`, `Click <name>`,
    `Type in <name>`, `Press <key>`; the name from the first locator of kind `role`, `label`,
    `placeholder`, `text` or `test id`, cut to 40 characters.
  - `recording_row` takes whether it is expanded; a click toggles; the expanded part is a
    `v_flex` under the row: an `h_flex` of twenty `div`s (`items_end`, each `h(px(4. + 2. * n))`
    capped at 36 px, accent or `border` colour), the step rows (time in the buffer font, muted;
    words), and `prompts::render_button` Draft a test and Send to the agent.
  - `record_this`: on `Ok(id)`, sets `drawer = Some(Drawer::Recordings)`, reads the list and the
    minute of `id`; on `Err`, `drawer_error`.
  - `draft_test(id, window, cx)`: spawns `draft_in(&recordings_dir(), &id, redactor)` in the
    background, then in the workspace: `project.create_buffer(…)`, sets the text (a header
    comment `// Drafted from recording <id>. Save it as <path>; run it with <run>.`, the draft's
    `note` if any, then the test), the TypeScript language from the registry, opens
    `Editor::for_buffer` in the tab's pane beside the tab.
- `browser_tools.rs`: `draft_in` becomes `pub(crate)`; nothing else.
- `playwright_scripts.rs`:
  - `pub struct LastRun { pub ran_at: u64, pub result: RunResult }` and
    `pub enum RunResult { Passed, Failed { exit_code: i32 }, Unknown }`, serde-derived.
  - `runs_in(data: &Path) -> io::Result<HashMap<PathBuf, LastRun>>` (a missing file is empty) and
    `record_run_in(data: &Path, script: &Path, last_run: LastRun) -> anyhow::Result<()>` (read,
    insert, write to a temporary file and rename).
  - `run_command`: after `block_end`, `record_run_in` in the background with the result, then
    `BrowserHub` emits `BrowserEvent::ScriptRan { project }`.
  - `Scope::label`: "every project" for `Global`.
  - `ran_ago(ran_at: u64, now: u64) -> String`: "ran just now", "ran N min ago", "ran N h ago",
    "ran yesterday", "ran on Oct 21", as `clients.rs:754` writes its times.
- `read_scripts` reads `library_in` and `runs_in` together into `ScriptsTray { scripts, runs, … }`;
  `hub_event` on `ScriptRan` for its project calls `read_scripts`.
- Files: `crates/marley_workbench/src/browser.rs`, `browser_tools.rs`, `playwright_scripts.rs`
  (Marley crate). No `docs/marley/zed-touchpoints.md` row: no Zed path changes.

### Visual check plan
- REQ-001, REQ-002: setup serves #506's loopback page; the steps click a button, type into a
  field and follow a link, then `marley: record this`; `932-01-minute` shows the drawer on
  Recordings, the expanded minute's bars (accent where the steps fell), the steps with their
  times and words, Draft a test and Send to the agent.
- REQ-003: a click on Draft a test; `932-02-draft` shows the new editor tab, its header comment
  with the suggested `tests/…spec.ts` path, and the test's `page.goto`. The steps check that no
  file appeared under `$E2E_WORK/repo/tests`.
- REQ-004, REQ-005: setup keeps `passes.mjs` and `throws.mjs` for the project and `never.mjs` for
  every project, and #523's registry; the steps open the tray, Run `passes`, wait for its block,
  Run `throws`, wait, and leave the tray open; `932-03-scripts` shows "passed", "failed · exit 1"
  and "not run" with scopes and times. The steps print `runs.json` from the run's data folder.
- `932-04-day`: Marley Day, the minute expanded again and the tray (REQ-001, REQ-004).
- Every criterion has a shot; nothing here needs a live remote service. The first Run installs
  `playwright-core` from the loopback registry, which #523's scenario proved works offline.

### Risks
- `Project::create_local_buffer` panics on a remote project (`mention_crease.rs:313`);
  `create_buffer` is the call to use, and Code checks it on a local project only, since a
  recording names a local root.
- The tray's row grows a second line; the tray is `max_h(rems(14.))` and scrolls, so three rows
  fit, more scroll.
- A recording saved before #506 has no `action` entries: its steps are navigations only, and
  Draft a test shows the draft's own refusal ("holds no click, fill or key press").
- Two runs that end at the same moment both read, change and write `runs.json`; one result can
  be lost. Runs start from the user's clicks, seconds apart, so this is noted and not guarded.
- The 220 px drawer (#887) may be short for the minute and its steps; the rows scroll inside it.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
