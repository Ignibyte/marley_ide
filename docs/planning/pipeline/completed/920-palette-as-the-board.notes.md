# The command palette as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-920-palette-as-the-board.md
- **Pipeline spec:** 920-palette-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order, line 8 (the palette half; the file finder is #921).
- **Classification / tier:** feature, M. One Zed crate hunk (`command_palette`) and a Marley
  crate file (`marley_workbench/src/pickers.rs`).
- **Recall (§18.3):**
  - L-claude-839-a-uniform-list-cannot-take-a-heading-row-001: the palette is a `Picker::list` in
    the Marley layout and `render_match` returns an `AnyElement`. The tile goes inside the
    `ListItem`, the heading stays outside it.
  - L-claude-798-driving-menus-and-the-palette-in-a-scenario-001 and the scenario helper
    `palette`: scenarios type Zed's names. That is why D1 keeps matching on them.
  - L-claude-657-the-palette-runs-the-command-used-last-for-a-shared-prefix-001: the ranking by
    use stays as it is; the history is keyed by Zed's name.
  - L-claude-836-a-cold-start-types-the-first-palette-late-001: settle before the first palette
    in the scenario.
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the hook is asked in `new`,
    not in `render_match` (D7).
  - L-claude-661-hide-a-feature-from-the-palette-with-zeds-filter-001: hidden commands never
    reach the hook; nothing to do.
- **Discovery:**
  - `crates/command_palette/src/command_palette.rs`: `CommandPalette::new` (`:113-162`), `Command`
    (`:246-250`), `update_matches` (`:625-728`, the `Case::Smart` at `:666`), `matches_updated`
    (`:375-431`, #839's order at `:418-425`), `render_match` (`:809-881`), `render_footer` (`:883-934`); `searchbar_trailer`
    is new for this delegate.
  - `crates/marley_workbench/src/pickers.rs`: `init` sets #839's hooks.
  - `docs/marley/zed-touchpoints.md:141`: the `command_palette.rs` row.
- **Decisions:** D1–D7 in the spec.

### Design
- Approach. A second global beside `MarleyPaletteGroups`:
  `pub struct MarleyPaletteLook { pub row: fn(&'static str, &App) -> MarleyCommandRow, pub help: &'static [&'static str] }`,
  and `pub struct MarleyCommandRow { pub name: SharedString, pub glyph: SharedString, pub tile: MarleyTile }`
  with `pub enum MarleyTile { Marley, Zed }` (an enum, not a bool). Marley sets it only while the
  Marley layout is on: either `pickers::init` sets it and the palette also asks `marley_layout`
  through #839's `MarleyPaletteGroups`, or the look carries a `fn(&App) -> bool` of its own. The
  Code phase picks the one with the smaller hunk.
- `CommandPalette::new` asks `row` once per action (`action.name()`) and stores the answer as
  `Command::marley_row: Option<MarleyCommandRow>`. A second pass over the commands sets the row
  to `None` for any two that share a plain name (D2).
- `update_matches`: `Case::Ignore` while the look is set. `matches_updated`: when the query is
  "?", the matches are the commands whose action names are in `help`, in that order.
- `render_match`: when the command has a row, the tile and the plain name replace the bare label.
  The highlight positions shift by the dropped prefix's length when the plain name is the tail of
  Zed's name (case aside); otherwise the plain name shows without highlight.
- `searchbar_trailer`: a muted `Label` "Commands", `LabelSize::Small`. `render_footer`: three
  muted labels, the third pushed right; no buttons, Enter and Ctrl Enter keep their bindings.
- `pickers.rs`: `palette_row` with a `const` table of (action name, plain name, glyph) for the
  board's commands and the renames, the default rule (namespace dropped, Title Case), the glyph
  rule (D3) and the tile kind from the namespace (D4). The tile is drawn by the palette from
  `MarleyCommandRow`: 24 px square, `rounded_md`, mono XSmall glyph; accent ink on an accent tint
  for `Marley`, `Color::Muted` on `element_background` for `Zed`.
- File manifest:
  - `crates/command_palette/src/command_palette.rs`: Zed crate, `// Marley:` on each hunk.
  - `crates/marley_workbench/src/pickers.rs`: Marley crate.
  - `docs/marley/zed-touchpoints.md`: the `command_palette.rs` row adds `MarleyPaletteLook`
    (rows, case, chip, footer, "?"), with the same restore note.
  - `script/e2e/920-palette-as-the-board.sh`: the scenario.

### Visual check plan
- REQ-001, REQ-003, REQ-004: open the palette with `ctrl-shift-p` and an empty query. Shot
  `920-01-empty`: tiles, plain names under #839's headings, the chip, the footer. Then Marley Day
  (`profile_setting theme '"Marley Day"'`). Shot `920-05-day`.
- REQ-001, REQ-002: type "Open th". Shot `920-02-typed`: Open Threads in the list, letters
  highlighted, tiles kept.
- REQ-005: clear and type "?". Shot `920-03-help`: Open Guide and Open Keymap.
- REQ-002: Escape, then `palette "marley: open threads"`. Shot `920-04-ran`: the Threads page.
- REQ-006: `palette "marley: use zed layout"`, open the palette. Shot `920-06-zed`: Zed's
  rows, no chip, Zed's footer.
- REQ-007: the review of the diff (the name passed to history, hit counts and telemetry is
  unchanged).

### Risks
- Highlight positions: `StringMatch::positions` index Zed's name. If the shift is wrong, letters
  highlight in the wrong place. The rule is to shift only when the plain name is the tail, and to
  show no highlight otherwise.
- `Case::Ignore` changes the ranking a little for a query with capitals. That applies only while
  the look is set.
- #921 changes the file finder's look in the same `pickers.rs`. Whichever ticket lands second
  rebases its `init` lines.
- The default rule may read oddly for some Zed commands ("Toggle Focus" from several panels
  collides, so those keep Zed's names by D2). The scenario sees only a sample, so the Code phase
  reads the map's output over the whole command list once, as a quick check.

## Phase 2 — Code
- **Built:** in `command_palette.rs`, `MarleyPaletteRow`, `MarleyPaletteLook`, `Command::marley_row`
  (asked in `new`), `marley_drop_twins`, `marley_positions`, `marley_tile`, `marley_footer`, the
  delegate's `marley_look`/`marley_help`, `Case::Ignore` while the look is set, the "?" list in
  `matches_updated`, and a `searchbar_trailer` (the palette had none; the trait's default was
  `None`); in `pickers.rs`, `PaletteName`, `PALETTE_NAMES`, `palette_row`, `capitalized`.
- **Deviations:** the footer's keys are `KeyBinding`s for `menu::Confirm` and
  `menu::SecondaryConfirm` with the board's words beside them, so a rebound key reads as bound.
  The look is on when any command got a row, which `palette_row` gives only in the Marley layout.
- **Review:** matching, `command_usage`, history and `confirm` still use `Command::name` (REQ-007);
  the look runs once per palette, never in `render_match` (D7). The unit test's `Command` gained
  the field.
- **Gate:** `just gate-diff` 17/17 green.

## Phase 3 — Test
- **Scenario:** `script/e2e/920-palette-as-the-board.sh`, sway, 6 shots, each read.
  - `920-01-empty`: RECENT and MARLEY groups; rows "Open Browser" (www, accent), "Use Marley
    Layout" (Ma), "Accept Suggestion" (Su) with Right; the Commands chip; the footer "Enter to run
    · Ctrl-Enter to bind a key · ? for help". REQ-001, REQ-003, REQ-004.
  - `920-02-typed`: "Open th": Open Threads first, "Open" and "th" lit; Zed's own commands with
    neutral tiles; three "open preview to the side" commands keep Zed's names (twins, D2). REQ-002.
  - `920-03-help`: "?": Open Guide (?) and Open Keymap (Km, Ctrl-K Ctrl-S). REQ-005.
  - `920-04-ran`: "marley: open threads" ran: the Threads page. REQ-002.
  - `920-05-day`: the palette in Marley Day. `920-06-zed`: Zed's layout: "marley: use zed layout"
    rows, no tiles, Add Keybinding… and Run. REQ-006.
- The scenario gained its own HOME after the first run (the run's terminal had read the box's
  shell config); rerun green.

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md`, the look plan's W3.3 row, the `command_palette.rs` ledger row.
- Knowledge: nothing new.
- Brain: decision recorded.
