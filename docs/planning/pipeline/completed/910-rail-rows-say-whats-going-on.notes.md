# Rail rows say what is going on — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-910-rail-rows-say-whats-going-on.md
- **Pipeline spec:** 910-rail-rows-say-whats-going-on.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, line 3 (docs/marley/design-review-2026-10-23/README.md).
- **Classification / tier:** feature, M. Marley crates only (`marley_workbench`, `marley_rail`).
  Built after #905 (`item_kind.rs`), #906 (`GroupSummary`, the open header's summary), #907
  (`thread_activity.rs`) and #908 (the bottom row's failed chip on `send_block::send`).
- **Recall (§18.3):**
  - F-claude-551: a state word inside one truncating label gets cut; the state stays its own
    `flex_none` label (`RowLine::state`). The exit and Ask an agent follow that rule.
  - F-claude-603: a row line's state is for a few short words; anything as long as a name goes
    on a line of its own. A thread's activity can be long, so it is the line's text, cut at the
    end, not a state.
  - F-claude-750: an element beside a rail row's title takes its width; an element goes in as a
    `CardLine::Element`, a line of its own. The Ask an agent link sits inside the command's line
    element, after the cut command.
  - F-claude-442: work deferred from inside the rail's update re-enters the rail. The link's
    click goes through `send_block::send`, which already defers.
  - F-claude-903: stand-ins go first on the terminals' PATH through the scenario's `.bashrc`, or
    Ask an agent starts the box's real Claude Code.
  - L-claude-555 and L-claude-877: rail rows move when Needs you shows and when a header changes;
    the scenario brings terminals forward by tab and measures clicks from its probe run's shots.
  - L-claude-644: `Rail` sits at clippy's bool and line caps; nothing new goes on `Rail` itself.
- **Discovery:**
  - `crates/marley_workbench/src/rail.rs`: `thread_subtitle` (11286), `render_thread_row`
    (6445), `live_statuses` (9367), `ThreadEntry` (748), `terminal_snapshot` (9514),
    `render_terminal_row` (6609), `command_snapshot` (9655), `command_line` (9694),
    `project_name` (10764), `render_project_row` (5739), `header_icon` (674).
  - `crates/marley_rail/src/marley_rail.rs`: `CommandSnapshot` (214), `ThreadStatus::label`
    (425), `ProjectRow::summary` (695), `summary` (1344), its call (1809), the tests' fixtures
    (`summary: None`, 2206 on).
  - `crates/marley_workbench/src/send_block.rs`: `chip` (192), `send` (103), `Request::Fix`.
  - `crates/marley_workbench/src/groups.rs`: `DEFAULT` (33).
- **Decisions:** D1–D7 in the spec. Progress and a ticking elapsed time on a running shell row go
  Out to the follow-up for A-core's "Determinate build progress".

- **Promoted 2026-10-25:** seams re-read after #905 to #909: `rail.rs` `render_thread_row`
  :6447 (still `thread_subtitle`), `live_statuses` :9371 (one caller, :9442), the thread
  snapshot build, `terminal_snapshot` :9519 (a shell's subtitle from
  `working_directory_label`), `command_snapshot` :9659, `command_line` :9698,
  `render_terminal_row` :6615, `header_icon` :677, `project_name` :10768; `marley_rail`'s
  `GroupSummary { words, needs_you }` (#906) and `CommandSnapshot`; `send_block::chip`'s failed
  test; `groups::DEFAULT` (private); `thread_activity` (#907) and `item_kind` (#905).

### Design
- Thread row: `live_statuses` returns, per live thread, its status and `thread_activity(…)`
  (#907, `crate::thread_activity`). The state colors come from #905's `item_kind.rs`. `ThreadEntry` gains the activity (or a `RowWords` struct holding the text); the line is
  built by a `thread_line(activity, agent_name, state)` that returns words matching the state.
  `thread_subtitle` goes. An unloaded thread falls back to "<agent> · idle".
- Shell row: `terminal_snapshot` asks a `shell_folder` helper for the folder: the path under the
  root when inside it, else the root itself home-relative (`~/srv/stacks/repo`). In
  `render_terminal_row`, a shell's command line comes before its folder; the first-line coloring
  and the two-line padding stay.
- Ask an agent: `CommandSnapshot` gains `index: usize`. A `shell_asks(terminal)` predicate holds
  `send_block::chip`'s test (newest live block failed, at the prompt, no agent in front), moved
  into `send_block` and called from both, so they cannot drift. #908's failed chip in the bottom
  row calls the same `send(…, Request::Fix, …)`; `send_selection::ask_to_fix` only asks about an
  editor's line, so it is not used. The failed line becomes a
  `CardLine::Element`: the cut command, the exit as a state, and a link-styled `Button` whose
  click stops propagation and calls `send_block::send(&view, index, Request::Fix, window, cx)`.
- Summary: #906's `GroupSummary { words, needs_you }` in `marley_rail` gains
  `counts: SummaryCounts { needs_you: usize, working: usize, done: usize }`. `summary` fills the
  counts by the same matches it has now, folding failed into needs you and not reporting into
  working (D5); `words` says "needs you" in place of "waiting" and becomes the pills' tooltip.
  `ProjectRow` gains `rows: usize` (what the open group lists) for "N rows". `project_name` takes
  the summary, the row count and a `Fold` enum (Open, Folded) and draws pills (`px_1p5`,
  `rounded_full`, the state's background and ink), or "N rows" open and "quiet" folded when the
  counts are empty.
- Default tile: `header_icon` wraps a projectless group's icon in a 20 px `rounded_md` tile on
  `raised(cx)`; the Default group (its name is `groups::DEFAULT`) gets `IconName::MarleyHome`.
- File manifest: `crates/marley_workbench/src/rail.rs` (Marley crate),
  `crates/marley_workbench/src/send_block.rs` (Marley crate),
  `crates/marley_rail/src/marley_rail.rs` (Marley crate). No Zed path; no
  `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- REQ-001: the scripted agent holds a tool call in flight; shot `910-01-working` of the rail.
- REQ-002: the scripted agent asks a permission; shot `910-02-asks`.
- REQ-003: the agent writes `notes.md` and ends the turn while another tab is in front; shot
  `910-03-done`.
- REQ-004: the thread's tab brought forward and left; shot `910-04-idle`.
- REQ-005, REQ-006: `false` run in one shell of `a`, the other idle in `a`'s root; shot
  `910-05-shells`.
- REQ-007: the link clicked; the stand-in's log and its terminal hold the fix request; the
  rail's selected row unchanged; shot `910-06-asked`.
- REQ-008 to REQ-010: `b` folded with its stand-ins waiting and working, `a` open, a third
  quiet project `c` open, Default folded and empty; shot `910-07-headers`.
- All: Marley Day selected through the theme selector; shot `910-08-day`.
- Not reachable: a thread restored from the metadata store with no live conversation is seen
  only if the run restarts; its fallback line is checked in the diff review.

### Risks
- #907's `ThreadActivity` may not carry the done turn's file count or the model; D2 adds them
  there, which grows this ticket a little.
- The pills take width from the project's name on a 260 px rail; the name truncates, the pills
  stay whole (F-claude-603's rule). Keep pills to three at most.
- Moving the folder under the command shifts a shell row's first line once its first command
  runs; the two-line height keeps the rows below in place (#796).
- Older scenarios that read the rail's summary text ("1 waiting") change wording; check
  `script/e2e/827-*.sh` and `542`'s expectations before the regression (L-claude-877).

## Phase 2 — Code
- **Built:** as designed in `rail.rs`, `send_block.rs` (`asks_fix`), `marley_rail.rs`
  (`SummaryCounts`, `ProjectRow.rows`, `CommandSnapshot.index`), `groups::DEFAULT` made
  `pub(crate)`; `rail_tests.rs`'s second-line test rewritten against `thread_line`.
- **Deviations:** `note_focus` counts a thread tab in front as shown
  (F-claude-910-a-thread-tab-in-front-was-never-shown-001). The pink pill also shows on a header
  whose hidden rows need the user without an agent count (`attention`). Shell rows' needs-you
  state does not join the header's counts, which count agents (D5).
- **Review:** the Ask an agent link stops the click reaching the row; `send` defers. No Zed path.
  Google style: `ThreadWords`, `LiveThreads`, `SummaryCounts` named-field structs; `Listed`
  enum, not a bool.
- **Gate:** `just gate-diff` 17/17 green after `too_many_lines` (three helpers out),
  `needless_pass_by_value`, `missing_docs` and dylint's `SharedString` literal rule in the test.

## Phase 3 — Test
- **Scenario:** `script/e2e/910-rail-rows-say-whats-going-on.sh` (sway): #827's stand-ins and bus,
  #907's scripted agent holding each "start" on its own release file.
- `910-01-working`: "Edit notes.md" in amber; the failed shell "false · exit 1 · 0 s Ask an agent"
  in red over its folder; the idle shell's folder; `a`'s amber "1" pill; Default's home tile
  (REQ-001, REQ-005, REQ-006, REQ-008, REQ-010).
- `910-04-idle`: the turn ended in front: "Opus 5.5 · idle", muted; `a` reads "3 rows" (REQ-004,
  REQ-009).
- `910-03-done`: a second turn ended unseen: "1 file changed" in the done colour, green dot, "1
  done" pill (REQ-003, REQ-008).
- `910-02-asks`: "Allow Edit README.md?" in pink, "1 needs you" (REQ-002, REQ-008).
- `910-06-asked`: Ask an agent clicked: a stand-in Claude Code started with "This command failed.
  Fix it:" and the block; the click did not select the failed row (REQ-007).
- `910-07-headers`: Default folded "quiet", `c` "1 row", `b` "1 needs you" (REQ-008 to REQ-010).
- `910-08-day`: Marley Day.
- Earlier runs: the idle shot kept the attention dot (fixed, above).

## Phase 4 — Complete
- CHANGELOG (Added and Fixed), `docs/marley_architecture/marley_workbench.md`. No Zed path.
- Knowledge: F-claude-910-a-thread-tab-in-front-was-never-shown-001.
