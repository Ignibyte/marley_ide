# A System tab with This build and Where data lives — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-855-system-pages.md
- **Pipeline spec:** 855-system-pages.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #854; split while planning: the tab and two pages here,
  the other seven pages TICKET-885.
- **Classification / tier:** feature, M. Marley crate only, one new file (a new component).
- **Recall (§18.3):** the Component Library's tab pattern (#799).
- **Discovery:** no page shows the build or the folders today; `release_channel` and `paths` hold
  every value This build and Where data lives need.
- **Decisions:** D1, D2 in the spec.

### Design
- `system_pages.rs`: `OpenSystemPages`; `SystemPages { page, folders: Option<Vec<Folder>>,
  counting: Task<()> }`; `Page::{Build, Data}` with a title, covers and lead; `render_page` draws
  the board's card; `render_row(dot, name, detail, value)`; `folder_size(path)` walks a folder
  (symlinks not followed) on the background executor; Copy build info writes the rows to the
  clipboard; Open the folder reveals `data_dir`.
- **File manifest:** `system_pages.rs` (new), `marley_workbench.rs` (the module and its init),
  the scenario.

## Phase 2 — Code
- **Built:** `system_pages.rs` (new): `OpenSystemPages`; `SystemPages` with `Page::{Build,
  Data}` (title, covers, lead); `render_list`; `render_page` (the card), `render_buttons`,
  `render_row` (dot, name over detail, value); `build_rows` from `ReleaseChannel`, `AppVersion`,
  `AppCommitSha`, `paths::data_dir` and the platform; `folders` and `folder_size` (symlinks not
  followed) counted on the background executor, `size_text`; Copy build info to the clipboard,
  Open the folder reveals `data_dir`, Count again recounts (a count asked for while one runs
  waits). `marley_workbench.rs` declares and inits it.
- **Gate:** the first run failed an `async` block with no `.await` and six `SharedString`
  literals; `futures::future::lazy` and `new_static`; the second run green, 17 of 17, and the
  scenario re-run on it.
- **Review:** nothing writes or deletes; sizes never count on a timer (D2).


## Phase 3 — Test
- **Scenario:** `script/e2e/855-system-pages.sh` (sway).
- `855-01-build`: the System tab: System, This build (chosen), Where data lives at the left; the
  card "This build · Which build · Its data folder", the lead, Channel (Marley, the version),
  Commit, Data folder, Platform (linux x86_64, the desktop), Copy build info. REQ-001 met.
- `855-02-data`: Where data lives: Settings, Database, Logs, Terminal history, Browser profiles
  and Handoffs ("none yet"), Extensions, Language servers ("empty"), each with its path and size;
  Open the folder and Count again. REQ-002 met.
- `855-03-day`: Marley Day, Where data lives. REQ-001 and REQ-002 met.


## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new (the gate's `new_static` lint is lessons.md's already). Brain: `brain
  decide` on consultation 901fe435.
- Ticket closed; pipeline archived; TICKET-885 holds the other pages.
