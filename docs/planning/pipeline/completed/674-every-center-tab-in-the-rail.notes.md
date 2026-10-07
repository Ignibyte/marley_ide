# Every center tab has a row in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-674-every-center-tab-in-the-rail.md
- **Pipeline spec:** 674-every-center-tab-in-the-rail.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-06 and 2026-10-07 (quoted in the spec); the decisions are in
  `docs/planning/intake/rail-and-center-tabs.md`.
- **Classification / tier:** feature; `marley_rail` (the model) and `marley_workbench` (the rail).
  No Zed crate.
- **Pre-flight:** green; #673 committed; a release install running (no cargo of this ticket's
  until it ends).
- **Recall (§18.3):**
  - #504: Browser rows are the template for a row kind (`BrowserSnapshot`, `Row::Browser`,
    `Selection::Browser`, `activate_browser`, `close_browser`).
  - PR-claude-single-selection-is-a-derived-selector: one highlighted row, derived in
    `selection()`; the new kinds join its list.
  - #509: the turns fold, in memory per terminal, the model for the Files fold.
  - The brain: `decisions/marley-keeps-its-top-tabs-lists-every-tab-in-the-rail-and-opens-rustys-screens-in-a-rusty-group`.
- **Discovery** (an Explore pass): `marley_rail.rs` `ProjectSnapshot` (:32), `Focus` (:405),
  `Selection` (:551), the row structs and `Row` (:570-730), `selection()` (:762), `Shown` (:794),
  `held_order` (:994), `run` (:1039), `walk` (:1151), `cycle_row` (:1334), `parent` (:1366),
  `rail_rows` (:1417), the tests' `project()` and `selected_rows()`; `rail.rs` `build_snapshot`,
  `push_closed`, `member_browsers`, `Snapshot`'s entry maps, `Watched`, `sync_subscriptions`,
  `active_rows`, `note_focus`, `open_row`, `render_row`, `activate_browser`, `close_browser`;
  `rail_tests.rs`'s three exhaustive `Row` matches; Zed's `handle_pane_event` (a
  `ChangeItemTitle` outside the active pane emits no workspace event).

### Design
- **`marley_rail.rs`**: `TabKind { File, Other }`; `TabSnapshot { id, title, folder, kind, dirty,
  matched }`; `ProjectSnapshot::{tabs, files_open}`; `Focus::tab`; `Selection::{Tab(u64),
  Files(usize)}`; `TabRow`, `FilesRow { project, count, open, selected }`; `Row::{Tab, Files}`;
  `Shown::{Tab, Files}`. `walk` puts Other tabs after Browser tabs, and the Files row (when the
  project has files that show) and its files (when open, or under a filter) after threads.
  `selection()` tries the focused tab, then its project's Files row. `parent` of a file is its
  Files row, of the Files row its project. `cycle_row` reaches tabs; `run` gives them none.
- **`rail.rs`**: `member_tabs` beside `member_browsers`, filling `TabSnapshot`s and a `tabs` entry
  map (`TabEntry { workspace, pane, item }`); `Watched` and `sync_subscriptions` gain the panes,
  each subscribed to all its events; `active_rows` returns the tab; `files_folded:
  HashSet<String>` by header place, read into each snapshot; `open_row`, `render_row`,
  `render_tab_row`, `render_files_row`, `activate_tab`, `close_tab`, `toggle_files`.
- **File manifest:** `crates/marley_rail/src/marley_rail.rs`, `crates/marley_workbench/src/rail.rs`,
  `rail_tests.rs`, the guide, the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001, 002 | opens two files with the file finder | `674-01-files` |
| REQ-001, 005 | opens a project search | `674-02-other` |
| REQ-003 | types into the second file | `674-03-dirty` |
| REQ-004, 005 | clicks the Files row with a file in front | `674-04-folded` |
| REQ-006 | unfolds, clicks the first file row's close | `674-05-closed` |

### Risks
- A pane subscription per pane refreshes the rail on every pane event (scrolls emit none; titles,
  activations and adds do): the refresh already runs on every workspace event.
- Closing a dirty file from its row asks to save, as Zed's tab close does (`SaveIntent::Close`).

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the ticket in progress; the backlog row removed.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's decisions.

## Phase 2 — Code (2026-10-07)
### Built
- **`marley_rail.rs`**: `TabKind`, `TabSnapshot`, `TabRow`, `FilesRow`; `ProjectSnapshot::{tabs,
  files_open}`, `Focus::tab`, `Selection::{Tab, Files}`, `Row::{Tab, Files}`, `Shown::{Tab,
  Files}`; `walk`, `selection`, `held_order`, `run`, `cycle_row`, `parent`; `tab_row` and
  `files_row` (helpers, which keep `rail_rows` under the 100-line lint); the tests' helpers and
  four exhaustive matches.
- **`rail.rs`**: `TabEntry`, `Snapshot::tabs`; `Rail::{pane_subscriptions, files_folded}`;
  `Watched::center_panes` and `follow_panes`; `member_tabs`; `active_rows` with the tab;
  `note_files_open`, `toggle_files`, `activate_tab`, `close_tab`; `open_row` (now `&mut self`) and
  `render_row` arms; `render_tab_row`, `render_files_row`; `open_group_entry` (pulled out of
  `build_snapshot`, which the new lines took past 100).
- **`rail_tests.rs`**: three exhaustive `Row` matches take the new kinds.
- **The guide**: the rail's overview names the new rows, the Files fold and the close.

### Deviations
- **No Right and Left on the Files row**: Enter and a click fold it; the spec says so.
- **Names the lints asked for**: `center_panes` (beside `panels`) and `center_tabs` (beside
  `tags`), `clippy::similar_names`.

### Review
- `close_tab` and `activate_tab` read the rail's snapshot and update the pane or the workspace,
  never the rail inside its own update; `toggle_files` refreshes the rail itself.
- `member_tabs` skips terminals and Browser tabs by downcast, so no item has two rows.
- A pane's events refresh the rail; the refresh redraws only when the snapshot changed.

### Gate
`just gate-diff` on the tree with the scenario: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/674-every-center-tab-in-the-rail.sh` (`compositor sway`), on the debug build: a scratch
project with `two.txt` and `notes/one.md`.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001, 002 | `674-01-files` | after the terminal, `Files 2` with `two.txt` and `one.md` (`notes` under it), one.md highlighted as in front; the tab bar's same three |
| REQ-001, 005 | `674-02-other` | `Project S…` with the search icon after the terminal, highlighted; Files 2 under it |
| REQ-003 | `674-03-dirty` | after a click on one.md's row and an `x`: one.md in front with a blue dot, the tab's dot too |
| REQ-004, 005 | `674-04-folded` | Files clicked: the chevron right, no file rows, the Files row highlighted for one.md in front |
| REQ-006 | `674-05-closed` | unfolded, two.txt's × clicked: its row and its tab gone, `Files 1` |

The exploratory run before the gate had two y guesses off by a row (the click landed on two.txt,
then the close met its save prompt); the coordinates come from that run's shots. No code changed.
Focus: headless sway; Hyprland's one Marley window (Chad's) before and after, no rule added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: every tab in the rail); the in-app guide's rail
  overview (before the gate); `marley_workbench.md` (a new section). No Zed path touched.
- **Knowledge:** `L-claude-674-a-tab-change-outside-the-active-pane-reaches-no-workspace-event-001`,
  `AD-claude-674-every-center-tab-has-a-rail-row-and-files-fold-under-files-001`.
- **Brain:** the decision of the morning's talk
  (`decisions/marley-keeps-its-top-tabs-lists-every-tab-in-the-rail-and-opens-rustys-screens-in-a-rusty-group`)
  covers it; no new decision.
- **Ticket:** closed; the pair archived.
