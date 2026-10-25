---
pipeline_id: a7c319a2-1ec0-4266-86eb-f45755f8ad3a
ticket: docs/planning/tickets/open/TICKET-921-file-finder-as-the-board.md
status: Phase 4 — Complete PASS
title: The file finder as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, docs/planning/pipeline/completed/839-pickers.spec.md]
---

## Title
The file finder drawn as the Palette board's "Go to a file" panel, in the Marley layout: each row
has the file's extension in a small mono slot, the name over its path on two lines, and a git
badge at the right; the field ends with the finder's key ("Ctrl P"); the preview is headed by the
file's path and its line count ("crates/…/rail.rs · 3 412 lines").

## Scope
### In
- `crates/file_finder/src/file_finder.rs` (Zed crate, one hunk set): `MarleyFinderLook`, a global
  `render_match_impl` asks for each file row. It hands the hook the row's extension and its git
  status (`Project::project_path_git_status` for the row's `ProjectPath`). When the hook answers,
  the row puts the answer's tile in the start slot (after the multi-select checkbox, in place of
  the file icon), stacks the name label over the path label, and puts the answer's badge in the
  end slot (the History icon stays for a history row with no badge). Without the global, Zed's
  row.
- The same file: `searchbar_trailer` puts the hook's key hint before the ignored-files toggle while
  the global answers.
- `crates/picker_preview/src/picker_preview.rs` (Zed crate, one hunk set): `EditorPreview` keeps
  the shown buffer's line count beside `current_path`; `render_message_or_editor` asks a global,
  `MarleyPreviewHeader`, for a header line over the editor and draws it when it answers.
- `crates/marley_workbench/src/pickers.rs` (Marley crate): sets both globals; `finder_marks` (the
  extension tile, the badge letter and its color from the git status, the key hint, all only in
  the Marley layout); `preview_header` ("path · N lines").
- `docs/marley/zed-touchpoints.md`: a row for each of the two Zed files.

### Out (explicitly deferred)
- The preview under the list: the board stacks it; #839 D2 put it at the right, and that stays.
- The file glyph at the field's left: `Picker::render_editor` has no leading slot, and the review
  did not list it. A follow-up ticket if wanted.
- Zed's ignored-files toggle and the multi-select toggle in the field stay; the board draws
  neither, but each is a working control.
- The command palette's names, tiles, scope chip and footer: #920.

## Reference (§20)
Upstream Zed: `file_finder`'s `FileFinderDelegate` (its matches, `labels_for_match` with the match
highlights and the path's elision, multi-select, history rows, `searchbar_trailer`) and
`picker_preview`'s `EditorPreview` (the editor-backed preview #839 put at the right). Every
behaviour stays Zed's; Marley changes how a row, the field's end and the preview's top are drawn.

### Prior art
- `file_finder.rs:2174` `render_match_impl`: the start slot is a checkbox and `FileIcons`'s icon,
  the child a one-line `h_flex` of the name label and the path label, the end slot the History
  icon or a spacer. `file_finder.rs:1327` `labels_for_match`: the path label is already `Small`
  and `Muted`, so the second line needs no new label.
- `file_finder.rs:477` `Match` and `:491-518`: `History` carries its `ProjectPath`; a `Search`
  match's comes from `project_path_for_search_match` (`:867`). `Channel` and `CreateNew` rows have
  no file and keep Zed's look.
- `file_finder.rs:1849` `searchbar_trailer`: the ignored-files toggle and the scan indicator;
  `picker/src/render.rs:96` adds the multi-select toggle after it. These are the "two icon
  buttons" the review saw.
- Git status: `Project::project_path_git_status` (`project/src/project.rs:2550`) returns a
  `FileStatus` from the git store, the same source the project panel reads
  (`project_panel.rs:1179`). The project panel maps a status to a letter and a color in
  `git_status_indicator` (`project_panel.rs:8393`: `!`, `U`, `D`, `M`, `A`); Marley maps
  `FileStatus` the same way in its own crate.
- `picker_preview.rs:197` `update_from_buffer` already stores `current_path`; nothing draws it.
  `render_message_or_editor` (`:329`) is the one place both layouts draw the editor.
- Marley's hook pattern: `MarleySwitcherLook` (`tab_switcher.rs:736-746`, #883: a tile in the
  start slot and a line under the title, asked per row) and `MarleyBranchLook` (#891), set from
  `tab_tiles.rs` and `search_looks.rs`; `MarleyPreviewDefault` (`picker.rs:64`, #839) set from
  `pickers.rs`, which already knows the finder by its delegate name and gates on `marley_layout`.
- The key hint: `KeyBinding::for_action_in` (used at `rail.rs:5700`) finds the bound keystroke;
  Linux binds `ctrl-p` to `file_finder::Toggle` (`assets/keymaps/default-linux.json:702`).
- Ledger: L-claude-839 (a uniform list measures one row: a row that grows a second line on every
  row stays a uniform list), PR-claude-a-render-hook-reads-its-context-not-its-view-001.

## UI proof
Scenario `script/e2e/921-file-finder-as-the-board.sh` (run under sway, keys only): a scratch
repository with a committed `src/rail.rs`, `src/rail_order.rs`, `docs/rail.md` and
`script/rail.sh`; then `src/rail.rs` changed and `src/rail_new.rs` left untracked. No agent runs.
- `921-01-rows`: Ctrl P, "rail" typed: rows with the extension, the name over the path, "M" on
  `rail.rs` and "U" on `rail_new.rs`; the field's "Ctrl P"; the preview headed
  "src/rail.rs · N lines" (REQ-001, REQ-002, REQ-003, REQ-004).
- `921-02-history`: a file opened, Ctrl P with nothing typed: the history row in the same layout
  (REQ-001).
- `921-03-day`: the same as `921-01-rows` in Marley Day (REQ-001 to REQ-004).
- `921-04-zed`: `marley: use zed layout`, Ctrl P, "rail" typed: Zed's rows, field and preview
  (REQ-005).

## Locked-In Decisions
- D1 — The new look is the Marley layout's, as #839 D1 set for the palette; in Zed's layout the
  hook answers nothing and the finder is Zed's.
- D2 — The extension slot shows the file's extension in mono, in the accent's light orange, as
  the board draws it; a file with no extension (`Makefile`) keeps its file icon there.
- D3 — The badge follows the project panel's letters and order (conflict `!`, untracked `U`,
  deleted `D`, modified `M`, added `A`) in the matching theme status color; a clean file has none.
- D4 — The row's second line is Zed's path label as it is, with its match highlights and its
  elision; only its place changes.
- D5 — The key hint is the keystroke bound to `file_finder::Toggle`, read from the keymap, so a
  rebound key shows the new key; with no binding, no hint.
- D6 — The preview's header belongs to the shared editor preview, so every picker that previews a
  file through it (the file finder, project symbols) gets it in the Marley layout. The count is
  the buffer's lines, grouped by thousands with a space, as the board writes it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout is on, each file row of the file finder shall show the file's extension in a mono slot at its start and its name over its path on two lines, in Marley Night and Marley Day. | `921-01-rows`, `921-02-history`, `921-03-day` |
| REQ-002 | WHILE the Marley layout is on, a row whose file has a git status shall end with the status's letter in its status color. | `921-01-rows`, `921-03-day` |
| REQ-003 | WHILE the Marley layout is on, the file finder's field shall end with the key bound to open it. | `921-01-rows`, `921-03-day` |
| REQ-004 | WHILE the Marley layout is on and the preview shows a file, the preview shall be headed by the file's path and its line count. | `921-01-rows`, `921-03-day` |
| REQ-005 | WHILE Zed's layout is on, the file finder shall draw Zed's rows, field and preview. | `921-04-zed` |
| REQ-006 | Each Zed-crate hunk shall carry a `// Marley:` comment and a row in `docs/marley/zed-touchpoints.md`. | The review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the two Zed hooks with their ledger rows, `pickers.rs`'s marks and header; a review
  of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
