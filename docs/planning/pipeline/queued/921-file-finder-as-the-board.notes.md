# The file finder as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-921-file-finder-as-the-board.md
- **Pipeline spec:** 921-file-finder-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, line 8 (the palette and the file finder); the file finder half.
- **Classification / tier:** feature, M. Two small Zed-crate hunks (`file_finder`,
  `picker_preview`) and Marley's `pickers.rs`.
- **Recall (§18.3):**
  - L-claude-839-a-uniform-list-cannot-take-a-heading-row-001: the finder is a
    `Picker::uniform_list_with_preview`; a second line on every row keeps rows one height, so it
    stays a uniform list. `CreateNew` and `Channel` rows must be the same height as file rows,
    or they clip.
  - PR-claude-a-render-hook-reads-its-context-not-its-view-001 and F-claude-865: the row hook runs
    inside the picker's render. It takes the extension and the status as arguments and never
    reads the picker or the workspace.
  - L-claude-658: a file opened from the finder in the Marley layout may not come to the front;
    the history shot needs only the finder's own row, so it does not matter here.
  - F-claude-903: no agent runs in this scenario, so no stand-ins are needed.
  - #839's spec: `MarleyPreviewDefault` and the delegate name "file finder"; D1 (Marley layout
    only) carries over.
- **Discovery:**
  - `crates/file_finder/src/file_finder.rs`: `render_match_impl` (`:2174`), `searchbar_trailer`
    (`:1849`), `Match` (`:477`), `project_path_for_search_match` (`:867`).
  - `crates/picker_preview/src/picker_preview.rs`: `EditorPreview` fields, `update_from_buffer`
    (`:197`), `render_message_or_editor` (`:329`).
  - `crates/marley_workbench/src/pickers.rs`: `init`, `preview_default`, `FILE_FINDER`.
  - `crates/project/src/project.rs:2550` `project_path_git_status`; `git::status::FileStatus`.
- **Decisions:** D1–D6 in the spec. The board's sample paths and counts are placeholders.

### Design
- `file_finder.rs` (Zed crate): `pub struct MarleyFinderFile<'a> { extension: Option<&'a str>,
  status: Option<FileStatus> }`, `pub struct MarleyFinderMarks { tile: AnyElement, badge:
  Option<AnyElement> }`, and `pub struct MarleyFinderLook { row: fn(&MarleyFinderFile, &App) ->
  Option<MarleyFinderMarks>, hint: fn(&FocusHandle, &mut Window, &mut App) -> Option<AnyElement> }`
  as a `Global`. In `render_match_impl`, for `History` and `Search` rows only, build the
  `ProjectPath`, read its status, ask `row`; on `Some`, the tile replaces the icon, the child
  becomes a `v_flex` of the two labels, and the badge fills the end slot. `CreateNew` and
  `Channel` rows get a matching two-line height so the uniform list keeps one row height.
  `searchbar_trailer` asks `hint` and puts its answer first. Each hunk carries
  `// Marley: the Palette board's finder rows (#921)`.
- `picker_preview.rs` (Zed crate): `line_count: Option<u32>` set in `update_from_buffer` from the
  buffer's `max_point().row + 1`, cleared with the message; `pub struct MarleyPreviewHeader(pub
  fn(&str, u32, &App) -> Option<AnyElement>)` as a `Global`; `render_message_or_editor` asks it
  with `current_path`'s display and the count while the editor shows, and draws the answer over
  the editor.
- `pickers.rs` (Marley crate): `init` sets both globals; `finder_marks` returns `None` outside
  the Marley layout, else the extension in mono `text_accent` in an 18 px slot (the file icon
  when there is no extension) and the badge from `badge_for(status)`; `finder_hint` draws the
  bound keystroke as muted text; `preview_header` draws "path · N lines" in mono, muted, with a
  bottom border. Badge letters are an enum (`Badge::Conflict`, `Untracked`, `Deleted`,
  `Modified`, `Added`) with its letter and color, not a tuple.
- `docs/marley/zed-touchpoints.md`: two rows, `crates/file_finder/src/file_finder.rs`
  (`MarleyFinderLook`: asked per file row and for the field's end) and
  `crates/picker_preview/src/picker_preview.rs` (`MarleyPreviewHeader` and the kept line count).
  Each says how to put it back: re-add the asks; take the rest of upstream's code.

### Visual check plan
- REQ-001: `921-01-rows` ("rail" typed: `rs`, `md`, `sh` slots, names over paths);
  `921-02-history` (a file opened, the finder empty: the history row in the same layout);
  `921-03-day`.
- REQ-002: `921-01-rows` ("M" on `src/rail.rs`, "U" on `src/rail_new.rs`, none on clean files);
  `921-03-day` (the colors read on the light ground).
- REQ-003: `921-01-rows`, `921-03-day` ("Ctrl P" before the two toggles).
- REQ-004: `921-01-rows`, `921-03-day` ("src/rail.rs · N lines" over the preview, N the file's
  real count from the fixture).
- REQ-005: `921-04-zed` (after `marley: use zed layout`: icon, one-line row, no badge, no hint,
  no header).
- REQ-006: the review of the diff.
- Setup: the repository is committed with `git -c user.name=… -c user.email=… commit`, then
  `src/rail.rs` is edited and `src/rail_new.rs` written, so the git store reports both before
  the finder opens (settle long enough for the scan).

### Risks
- `labels_for_match` sizes the path's elision for a one-line row (`full_path_budget` takes the
  name's width off). On two lines the path has more room than the budget gives, so it may elide
  sooner than it must. Acceptable; pass a wider budget only if the shots show paths cut short.
- The git store fills after the worktree scan; a row drawn before it has no badge and gains one
  on the next render. The scenario settles first.
- The header changes the project symbols picker's preview too (D6). It is not shot here; the
  review of the diff checks it stays Zed's outside the Marley layout.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
