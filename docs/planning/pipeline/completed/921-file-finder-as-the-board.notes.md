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
- **Built:** `file_finder.rs` `MarleyFinderMarks`, `MarleyFinderLook` (row and hint), the ask in
  `render_match_impl` (the row's `ProjectPath` from a history or search match, its extension),
  the tile in the icon's place, the stacked labels, the badge in the end slot, the hint first in
  `searchbar_trailer`; `picker_preview.rs` `MarleyPreviewHeader`, `marley_lines`, the header over
  the editor; `pickers.rs` `finder_marks`, `finder_hint`, `preview_header`, `grouped`;
  `file_finder` and `picker_preview` added to `marley_workbench`'s dependencies.
- **Deviations:** the hook takes the project and the `ProjectPath` and Marley reads the git status
  itself, since `file_finder` does not depend on the `git` crate where `FileStatus` lives. The key
  is `workspace::ToggleFileFinder` (the action named `file_finder::Toggle`).
- **Review:** without the global the row and the trailer are Zed's; the history row keeps its
  icon when it has no badge; nothing reads a view in a render.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- **Scenario:** `script/e2e/921-file-finder-as-the-board.sh`, sway, 4 shots, each read.
  - `921-01-rows`: rail.rs (rs, M), rail.md (md), rail.sh (sh), rail_new.rs (rs, U),
    rail_order.rs (rs), each name over "repo/src/"; Ctrl-P at the field's end; the preview headed
    "src/rail.rs · 1 201 lines". REQ-001 to REQ-004 met.
  - `921-02-history`: the opened rail.rs as the history row, same layout, its M badge.
  - `921-03-day`: Marley Day, "docs/rail.md · 1 line".
  - `921-04-zed`: Zed's rows on one line with icons, no hint, Zed's layout. REQ-005 met.
- **Fix found in Test:** the count read 1 202 for a 1 201-line file: the empty line after the final
  newline was counted (F-claude-921). Gate and scenario rerun.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md`, the look plan's W3.3 row, ledger rows for `file_finder.rs`
  and `picker_preview.rs`.
- Knowledge appended: F-claude-921-a-buffers-line-count-took-the-line-after-its-newline-001.
- Brain: decision recorded.
