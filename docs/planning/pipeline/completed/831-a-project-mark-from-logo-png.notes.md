# A project's mark from its logo.png — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-831-a-project-mark-from-logo-png.md
- **Pipeline spec:** 831-a-project-mark-from-logo-png.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #830; Chad's overnight goal (2026-10-20).
- **Classification / tier:** feature, L. Mostly Marley crates; one additive hunk in `title_bar`
  and a field in `settings_content`.
- **Split:** the board's notifications and switcher marks went to TICKET-859 (gpui's
  `SystemNotification` has no image, so every platform changes); a remote project's logo went to
  TICKET-860 (#564 reads with `std::fs`). Both are queued before #794.
- **Recall (§18.3):**
  - #564's pipeline built the search, the 256 KiB limit, the 32 px shrink and the re-search on
    worktree events; this ticket keeps all of it and moves the state.
  - BF-claude-svg-asset-not-registered-renders-blank: a missing image is a silent blank, so the
    shots must show the image itself.
  - F-claude-438-a: a value another surface reads is kept by `refresh` (here: by the global), never
    written only in `render`.
  - The brain: nothing on this seam.
- **Discovery** (an Explore map, this session):
  - `project_icons.rs`: `NAMES` (:17-33, `logo` 10th), `EXTENSIONS` (:36), `PAGES` (:39), size
    limit (:42), shrink (:108-152), `icon_in` (:156, blocking `std::fs`), `is_candidate` (:188).
  - `rail.rs`: `project_icons: HashMap<PathBuf, ProjectIcon>` (:231), `ProjectIcon` (:696-705),
    `ICON_SEARCH_EVERY` (:707), `follow_icons` (:1639-1659, local projects only, after every
    refresh at :1443), `search_icon` (:1663-1701, `background_spawn`, throttle and `again`),
    `note_icon_files` (:1705-1737, from `WorktreeUpdatedEntries` at :2999), `header_icon`
    (:670-685), the header's icon at :4955-4961, `HeaderMenu` (:501, built at :5568).
  - `rail_strip.rs`: `initial`, `name_hash` and the tile face (:273-311).
  - Home: `MarleyHome::render_recent` (`home_page.rs:401-444`), `Recent { paths, name, folder }`
    (:153), local recent workspaces only (:195-201).
  - Title bar: `MarleyTitleBarButtons` (`title_bar.rs:213-217`), made in `TitleBar::new`
    (:559-562), drawn at :421; `render_project_name` called at :356, the #829 mark at :329-336;
    the hook is set in `status_buttons.rs:50`.
  - `launch.rs:107-145`: `LaunchConfigs`, the path-keyed `Global` pattern.
- **Decisions:** D1 to D5 in the spec.

### Design
- **`project_icons.rs`** (Marley):
  - `NAMES`: `"logo"` moved to the front.
  - `pub struct ProjectMarks { marks: HashMap<PathBuf, Entry> }`, `impl Global`, where `Entry` is
    #564's `ProjectIcon` moved here: `image`, `chosen`, `searched_at`, `search: Option<Task<()>>`,
    `again`, and `source: Source { Search, Picked(PathBuf), Letter }` from the override.
  - `pub enum ProjectMark { Image(Arc<RenderImage>), Letter(char) }`.
  - `pub fn mark(root: &Path, name: &str, cx: &App) -> ProjectMark`: the override's letter, else
    the cached image, else the letter. It does not search.
  - `pub fn want(root: &Path, cx: &mut App)`: starts a search for a root not yet cached (or whose
    override changed) with #564's throttle, in `cx.background_spawn`. On the result it updates
    the global, which wakes its observers. The search is `icon_in(root)` for `Source::Search`, or
    a one-file read through the same size check and decode for `Source::Picked`.
  - `pub fn files_changed(root: &Path, paths: &[PathBuf], cx: &mut App)`: #564's
    `note_icon_files` logic (candidate or the chosen file → search again).
  - `pub fn forget_missing(roots: &HashSet<PathBuf>, cx: &mut App)` for the rail's "drops roots
    that are gone" (only roots the rail asked for; Home's recent roots stay).
  - The override is read from `MarleySettings::project_logos`. A settings observer, set in
    `project_icons::init(cx)`, compares each entry's source with it and searches again where it
    changed.
  - `pub fn project_mark(root: &Path, name: &str, size: Pixels, cx: &App) -> AnyElement`: the
    image at `size`, rounded, or the letter (`Label`, semibold, `Color::Custom(tint)`) centered on
    a `size` box with `tint.opacity(0.18)`, the #830 tile face moved here. `initial` and
    `name_hash` move from `rail_strip.rs` too.
- **`rail.rs`** (Marley): `project_icons` field removed; `follow_icons` calls `want` for each
  local root and `forget_missing`; `search_icon` goes; `note_icon_files` calls `files_changed`;
  `Rail::new` observes `ProjectMarks` (notify). `header_icon` takes the root and name and returns
  `project_mark(root, name, px(16.), cx)` for a project, keeping the projectless group's
  `ListTree` icon. The header menu gets the three entries (D3): "Choose Logo…" via
  `cx.prompt_for_paths(PathPromptOptions { files: true, directories: false, multiple: false,
  .. })`, then a settings write of the chosen path; "Use a Letter" writes `"letter"`; "Use the
  Project's Logo" (only while an entry exists) removes it. Closed groups get the entries too.
- **`rail_strip.rs`** (Marley): the tile face becomes `project_mark(root, name, px(24.), cx)`; the
  tile keeps its 36 px frame and ring.
- **`home_page.rs`** (Marley): `render_recent` calls `want` for each recent's first path (once,
  when the list is read) and draws `project_mark(first, name, px(16.), cx)` in place of the
  folder icon; `MarleyHome` observes `ProjectMarks`.
- **Title bar:** `title_bar.rs` (Zed) gets `MarleyProjectMark(pub Arc<dyn Fn(WeakEntity<Workspace>,
  &mut Window, &mut App) -> AnyView>)` beside `MarleyTitleBarButtons`, a `marley_project_mark:
  Option<AnyView>` field made in `new`, and `.children(self.marley_project_mark.clone())` just
  before `render_project_name`. `status_buttons.rs` (Marley) sets the global to a
  `TitleProjectMark` view, which observes `ProjectMarks` and the workspace's project (worktrees
  added or removed), reads the first visible worktree's root and name, calls `want`, and renders
  `project_mark(.., px(16.), cx)` only for `ProjectMark::Image` (D4), else an empty element.
- **Settings:** `settings_content/src/marley.rs` (Zed crate, Marley file) gains
  `project_logos: Option<BTreeMap<String, String>>`. `MarleySettings.project_logos:
  BTreeMap<PathBuf, ProjectLogo>`, where `ProjectLogo { Letter, File(PathBuf) }` is parsed from
  the content. `default.json` gains `"project_logos": {}` with a comment.
- **File manifest:**
  | File | Owner | Change |
  |---|---|---|
  | `crates/marley_workbench/src/project_icons.rs` | Marley | the global, `mark`, `want`, renderer |
  | `crates/marley_workbench/src/rail.rs` | Marley | use the global; header mark; menu entries |
  | `crates/marley_workbench/src/rail_strip.rs` | Marley | shared renderer |
  | `crates/marley_workbench/src/home_page.rs` | Marley | recent marks |
  | `crates/marley_workbench/src/status_buttons.rs` | Marley | `TitleProjectMark`, the hook |
  | `crates/marley_workbench/src/marley_workbench.rs` | Marley | `project_logos`, `init` call |
  | `crates/title_bar/src/title_bar.rs` | Zed | `MarleyProjectMark` hook |
  | `crates/settings_content/src/marley.rs` | Zed crate (Marley file) | `project_logos` |
  | `assets/settings/default.json` | Zed asset | `"project_logos": {}` |
  | `docs/marley/zed-touchpoints.md` | docs | clauses on three rows |

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002 | `logo/logo.png` (magick, 64 px orange with "L") and `plain/` opened; rail open | `831-01-rail-night` |
| REQ-005 | `logo` shown | `831-02-title` (crop of the title bar) |
| REQ-003 | `marley.rail_shrunk` written true in the profile | `831-03-strip` |
| REQ-004 | strip opened again; Home from the status bar's Home button (`marley: open home` in the palette) | `831-04-home` |
| REQ-006 | `logo.png` overwritten with a green square while running; wait 3 s | `831-05-replaced` |
| REQ-007 | right-click `logo`'s header, click "Use a Letter"; print `marley.project_logos` | `831-06-letter` |
| REQ-002 | theme set to Marley Day | `831-07-day` |

Not reachable: "Choose Logo…" opens the desktop's file chooser (portal), which the headless sway
cannot drive. Its write is the same settings path as "Use a Letter", and a hand-written
`project_logos` entry pointing at a file is shown instead if time allows. Remote projects are
TICKET-860.

### Risks
- Moving the cache can lose #564's throttle or `again` semantics; keep them byte for byte in the
  moved code.
- Home searching many recent roots at once: the throttle is per root, and each search is one
  small directory walk of fixed names; Home lists at most its recent count.
- The title bar view must follow worktree changes, since the hook runs once per title bar.

## Phase 2 — Code
- **Built:** `project_icons.rs` became the shared cache: `logo` first in `NAMES`; `read_icon`
  split out of `icon_in`; `ProjectLogos` (its own `RegisterSetting` over
  `marley.project_logos`), `Source`, `Entry` (#564's `ProjectIcon` moved), the `ProjectMarks`
  global, `ProjectMark { Image, Letter }`, `init` (re-search where the pick changed), `mark`,
  `want`, `files_changed`, `search` (#564's throttle and `again` kept, plus a re-search when the
  pick changed mid-search), `picked`, `set_logo`, `choose_logo`, `initial`, `tint`,
  `project_mark`. The rail lost `project_icons`, `ProjectIcon`, `ICON_SEARCH_EVERY` and
  `search_icon`; `follow_icons` calls `want`; `note_icon_files` calls `files_changed`; the rail
  observes `ProjectMarks`; `header_icon` draws `project_mark` (the letter included), built in
  `render_blocks` and handed to `render_project_row`; the header menu gets Choose Logo…, Use a
  Letter and, while picked, Use the Project's Logo (local projects only). The strip's tile reads
  `mark`. Home asks for each recent project's mark and draws it in place of the folder. The title
  bar: `MarleyProjectMark` hook and field in `title_bar.rs`, `TitleProjectMark` in
  `status_buttons.rs`. `project_logos` in the settings content and `default.json`.
- **Deviations:** `project_logos` is read by its own settings struct in `project_icons.rs`, not a
  `MarleySettings` field, since `MarleySettings::from_settings` sits at clippy's 100-line limit.
  The rail no longer forgets the icons of groups that went: Home's recent projects share the
  cache, and a 32 px image per folder is small.
- **Review of the diff, and a probe run** (the scenario's coordinates needed a real run before the
  gate): a real bug, `TitleProjectMark::new` read the workspace while `TitleBar::new` runs inside
  the workspace's update. That is a double-lease panic at start, so no window opened. Fixed: the
  project subscription is made in a `cx.defer` through a weak handle (F block at Complete). The
  probe's shots then showed every surface.
- **Gate:** red three times at source: clippy (eight findings, among them `needless_pass_by_value`
  and a 102-line `render_project_row`, fixed by building the header icon in `render_blocks`),
  dylint (a heap-allocating `SharedString` literal → `new_static`), then clippy's
  `unused_qualifications`. Fourth run: `script/gates.sh --diff` 17/17.

## Phase 3 — Test
- **Scenario:** `script/e2e/831-a-project-mark-from-logo-png.sh` (sway, 1600×1000, debug build of
  the gated tree). The first final run's right-click hit `plain`'s terminal, because the box's own
  tmux sessions (#808) add a varying number of rows under Default. The scenario now sets
  `marley.tmux_hosts` to `[]` and its points were read again from the shots. The second run
  exited 0 with eight shots.
- `831-01-rail-night`: `plain`'s header with a "P" on a dark green tint, and `logo`'s with the
  64 px orange "L" logo at 16 px (REQ-001, REQ-002 met). The same shot is `831-02-title`: the
  title bar reads cat · menu · orange logo · `logo` · branch (REQ-005 met).
- `831-03-strip`: `marley.rail_shrunk` true. Tiles D (Default), P (tint) and the logo image,
  ringed as shown (REQ-003 met).
- `831-04-home`: Home from the status bar. RECENT PROJECTS lists `logo` with its image and
  `plain` with its "P"; other recent projects from the profile show their letters or icons
  (REQ-004 met).
- `831-05-replaced`: `logo.png` rewritten green while Marley ran. The rail header, the title bar
  and Home's row all show the green logo within the 4 s settle (REQ-006 met).
- `831-06-menu`: `logo`'s header menu with Choose Logo… and Use a Letter after Move Project
  Up/Down. `831-06-letter`: after Use a Letter, `logo`'s header and Home's row show "L" on a
  purple tint, and the title bar's mark is gone (D4: images only). The printout:
  `project_logos: {'<work>/logo': 'letter'}` (REQ-007 met).
- `831-07-day`: Marley Day. The letters on their tints read on the light ground (REQ-002 met). The
  probe run's Day shot, before the letter was picked, showed the image too.
- Not reachable: "Choose Logo…" opens the desktop's file chooser, which a headless sway cannot
  drive; it writes the same setting through `set_logo`, which Use a Letter showed working.
- Pre-existing — not in scope: the box's tmux sessions leak into scenarios that do not set
  `marley.tmux_hosts` (seen in #829 and #830 too).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #831); the guide (a project's mark, before the strip's paragraph);
  the architecture note's #564 section (the global, the readers, the hook); look-plan W1.2 done
  with #859 and #860 named; touchpoint rows for `title_bar.rs`, `settings_content/src/marley.rs`
  and `default.json` checked against what shipped.
- **Knowledge:** F-claude-831-a-title-bar-view-read-the-workspace-inside-its-update-001,
  PR-claude-831-a-view-a-workspace-hook-makes-reads-the-workspace-only-after-new-001,
  L-claude-831-a-clicking-scenario-sets-tmux-hosts-to-nothing-001.
- **Brain:** decisions recorded on consultation 1c086cb9 (follow-up 2026-11-04).
- **Ticket** closed; #859 and #860 filed and queued; pipeline archived; committed on
  `marley/workbench-shell` after a fresh gate (the scenario changed in Test).
