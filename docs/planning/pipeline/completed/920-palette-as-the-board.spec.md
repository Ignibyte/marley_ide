---
pipeline_id: 8141484f-f66a-492f-9980-3b0eff04d87a
ticket: docs/planning/tickets/open/TICKET-920-palette-as-the-board.md
status: Phase 4 — Complete PASS
title: The command palette as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/C-threads.md, /srv/stacks/marley/docs/design/marley-canvas-2026-10-20/project/Palette.dc.html]
---

## Title
In the Marley layout the command palette draws its rows, field and footer as the Palette board
does: a plain name for each command ("Open Browser", not "marley: open browser") with a glyph
tile before it, a "Commands" chip at the field's right, and the footer "Enter to run · Ctrl Enter
to bind a key · ? for help". A query of "?" lists the help commands.

## Scope
### In
- `command_palette.rs` (Zed crate hunk): `MarleyPaletteLook`, a global the palette asks once when
  it opens, in `CommandPalette::new`, for each command's row: a plain name, a glyph and a tile
  kind. Each `Command` keeps the answer beside Zed's name. Without the global, Zed's palette.
- `command_palette.rs` (Zed crate hunk): `render_match` draws the tile in front of the name and
  shows the plain name; the typed letters stay highlighted when the plain name is Zed's name with
  its namespace dropped.
- `command_palette.rs` (Zed crate hunk): while the look is set, the match ignores case, so a query
  typed as a row reads ("Open Th") still finds it.
- `command_palette.rs` (Zed crate hunk): `searchbar_trailer` returns the muted "Commands" chip
  while the look is set.
- `command_palette.rs` (Zed crate hunk): `render_footer` draws the board's footer while the look
  is set: "Enter to run", "Ctrl Enter to bind a key", and "? for help" at the right. Enter and
  Ctrl Enter keep their actions.
- `command_palette.rs` (Zed crate hunk): in `matches_updated`, a query of exactly "?" lists the
  commands the look names for help, in its order.
- `pickers.rs` (Marley crate): the name map and `palette_row`, which answers the hook: the
  board's names and glyphs for the commands it draws, and a rule for every other command.
- `docs/marley/zed-touchpoints.md`: the row for `command_palette.rs` grows to name the new hook.

### Out (explicitly deferred)
- Groups while typing: the query "open" keeps Zed's fuzzy order without headings (#839 D1).
- The agent grid in the New Agent picker (#839 Out, D3).
- The file finder's rows, hint and preview header: #921.
- The selected row's accent fill (#3a2210 on the board): Zed's `ListItem` selection stays.
- The magnifier icon at the field's left: not in the review's gaps; a follow-up ticket if wanted.
- Plain names anywhere but the palette (the keymap editor, Marley's MCP tools' `palette_name`,
  the docs): each keeps Zed's names.

## Reference (§20)
Upstream Zed: `command_palette` keeps its behaviour. Commands are matched against
`humanize_action_name`'s names, ranked by use, run on Enter, and Ctrl Enter opens the keymap
editor on the selected action. The query history, `command_aliases` and the hit counts stay keyed
by Zed's name. Marley changes only what a row, the field's trailing slot and the footer show, and
what "?" lists.

### Prior art
- `crates/command_palette/src/command_palette.rs:122-136`: `CommandPalette::new` builds each
  `Command` from `humanize_action_name(action.name())`; the name is the fuzzy candidate
  (`:658-661`, `fuzzy_nucleo::Case::Smart` at `:666`), the hit-count key (`:653`) and the
  telemetry and history key (`confirm`, `:759-807`).
- The same file, #839's hunks: `MarleyPaletteGroups` (`:225-228`), asked in `new` (`:145-149`);
  `MARLEY_GROUPS` and `Command::marley_group` (`:231-243`), which reads the `marley:` prefix of
  Zed's name; the empty query's order in `matches_updated` (`:418-425`); the heading in
  `render_match` (`:818-829`, `:867-879`).
- The same file: `render_match` (`:809-881`) draws a `HighlightedLabel` of the name with the match
  positions and the `KeyBinding`; `render_footer` (`:883-934`) draws Add or Change Keybinding… and
  Run as buttons.
- `crates/picker/src/picker.rs:354`: `PickerDelegate::searchbar_trailer`, a control at the
  field's right; `file_finder.rs:1849` and `text_finder/delegate.rs:733` already use it.
- `crates/marley_workbench/src/pickers.rs`: #839 sets `MarleyPaletteGroups(crate::marley_layout)`
  there; the new look goes beside it.
- `crates/tab_switcher/src/tab_switcher.rs:734-746` and `crates/marley_workbench/src/tab_tiles.rs:99`
  (#883): the same shape of hook, a row's tile and the board's footer asked of Marley, and the
  tile Marley draws.
- `crates/marley_workbench/src/settings_tools.rs:74` `palette_name`: Zed's name for an action,
  used by Marley's MCP tools; it stays as it is.
- Actions the board names: `marley::OpenBrowser`, `marley::OpenThreads`, `marley::OpenMcpServers`,
  `marley::OpenComponentLibrary`, `workspace::Open`, `zed::OpenSettings`, `zed::OpenKeymap`
  (`crates/zed_actions/src/lib.rs:67`), `zed::OpenLog`; help: `marley::OpenGuide`
  (`marley_workbench.rs:443`).
- The board, `Palette.dc.html`: a 24 px tile with a mono glyph, accent ink on an accent tint for
  Marley's commands, muted ink on a neutral fill for the rest; the "Commands" chip in 11.5 px muted
  text; the footer in three spans, "? for help" pushed right.
- `docs/zed_architecture/`: nothing on the palette's rows. Published material: none needed.

## UI proof
Scenario `script/e2e/920-palette-as-the-board.sh` (run under sway, as #839's is, keys only): a scratch repository opened in the Marley layout, no terminals or agents needed. The
palette opens with `ctrl-shift-p`.
- `920-01-empty`: the palette with an empty query: #839's groups with a tile and a plain name on
  each row, the Commands chip, the footer (REQ-001, REQ-003, REQ-004).
- `920-02-typed`: "Open th" typed: plain names with the typed letters highlighted, Open Threads
  among them (REQ-001, REQ-002).
- `920-03-help`: "?" typed: Open Guide and Open Keymap (REQ-005).
- `920-04-ran`: Escape, then `palette "marley: open threads"`: the Threads page opens, so Zed's
  names still match (REQ-002).
- `920-05-day`: the empty palette in Marley Day (REQ-001, REQ-003, REQ-004).
- `920-06-zed`: `marley: use zed layout`, the palette opened: Zed's rows, field and footer
  (REQ-006).

## Locked-In Decisions
- D1 — Matching stays on Zed's name. The plain name is what a row shows. 222 scenarios type Zed's
  names into the palette, and the history, `command_aliases` and hit counts are keyed by them.
- D2 — The plain name: the namespace dropped and each word capitalized ("zed: open settings" →
  "Open Settings"). A map in `pickers.rs` overrides it where the rule reads wrong: "MCP", "Open
  Folder…" for `workspace::Open`, and the like. When two commands would read the same, both keep
  Zed's name.
- D3 — The glyph: the map's glyph for the commands the board draws (www, Th, Mc, Cl, .., St, Km,
  Lg); otherwise the first two letters of the plain name's second word, or of its only word.
- D4 — The tile's colour says whose the command is: accent for Marley's (`marley:`, `rusty:` and
  Marley's other namespaces), neutral for Zed's. A Recent row keeps its command's colour.
- D5 — The look follows the Marley layout, as #839's groups do; Zed's layout keeps Zed's palette.
- D6 — "? for help": a query of exactly "?" lists Open Guide (`marley::OpenGuide`) and Open
  Keymap (`zed::OpenKeymap`). The look names them, so the Zed hunk holds no Marley action.
- D7 — The hook is asked once per command when the palette opens, not in `render_match`, so no
  Marley code runs during the palette's render (F-claude-865).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the Marley layout is on, each row of the command palette shall show a glyph tile, accent for Marley's commands and neutral for the rest, and the command's plain name, in Marley Night and Marley Day. | `920-01-empty`, `920-02-typed`, `920-05-day` |
| REQ-002 | WHEN a query is typed, the palette shall match it against Zed's names without regard to case, and highlight the typed letters in the plain name. | `920-02-typed`, `920-04-ran` |
| REQ-003 | WHILE the Marley layout is on, the palette's field shall show a "Commands" chip at its right. | `920-01-empty`, `920-05-day` |
| REQ-004 | WHILE the Marley layout is on, the palette's footer shall read "Enter to run", "Ctrl Enter to bind a key" and "? for help". | `920-01-empty`, `920-05-day` |
| REQ-005 | WHEN the query is "?", the palette shall list Open Guide and Open Keymap. | `920-03-help` |
| REQ-006 | WHILE Zed's layout is on, the palette shall show Zed's rows, field and footer. | `920-06-zed` |
| REQ-007 | The palette's query history, aliases and hit counts shall stay keyed by Zed's names. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the hunks in `command_palette.rs`, the map and `palette_row` in `pickers.rs`, the
  touchpoints row; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/920-palette-as-the-board.sh`, read
  every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
