# The Component Library as the board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-936-component-library-as-the-board.md
- **Pipeline spec:** 936-component-library-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Components).
- **Classification / tier:** feature, S. Marley crate only (`marley_workbench`).
- **Recall (§18.3):**
  - L-claude-799 (checking a long scrolling page): put the thing to check at the top, or narrow
    the page; the scenario clicks `card` so the page is short and the strip is in view.
  - #801's scenario writes templates into `$E2E_PROFILE/config/components/`; reuse its files.
  - #849's scenario clicks the find field at a measured place (`FIND_X`, `FIND_Y`); the left
    list's rows sit under it.
  - No failure or prevention rule names `component_library.rs`.
- **Discovery:** one file, `crates/marley_workbench/src/component_library.rs`:
  `templates_group` (the strip), `ComponentLibrary::render` (Try it's header and where the error
  goes), `render_navigation` (the chosen row), and a new handler on `ComponentLibrary` that
  writes a template's example into the scratch editor.
- **Decisions:** D1–D4 in the spec.

### Design
- `templates_group` becomes a method on `ComponentLibrary` (it needs `cx.listener` for the chip
  clicks), or takes the listener it needs. The heading row: `Label` "Your templates", a muted
  "·", and a link-styled `Button` "components/ in Marley's config folder" that runs
  `cx.open_with_system(&templates_dir())`.
- A chip: a rounded, bordered `h_flex` with an id per file name, the name in the buffer font, and
  a state `Label` (`Color::Success` "loaded" or `Color::Error` with the error). The error is cut
  with `util::truncate_and_trailoff` to about 40 characters; `.tooltip(Tooltip::text(error))`
  holds the whole. Chips sit in an `h_flex().flex_wrap().gap_2()`.
- Loaded chip click: `serde_json::to_string_pretty(&template.example_tree())` set into
  `self.scratch` with `Editor::set_text`; the existing `BufferEdited` subscription runs
  `try_scratch`, so nothing else changes. Broken chip click: the workspace opens
  `broken.path` (the library keeps a `WeakEntity<Workspace>` taken in `open`, passed to `new`).
- Try it header: an `h_flex` of "Try it", a muted hint, and, when `self.tried` is
  `Some(Err(error))`, a flex-filling `Label` with `error.to_string()` in `Color::Error`,
  single-line and truncated. The body under the field draws only `Some(Ok(tree))`.
- Chosen row: compare `self.query(cx)` (already lower-cased) with the entry's name; when equal,
  `.bg(colors.element_selected)` and the label's colour `Color::Default` (the rest keep
  `Color::Default` too, so only the fill marks it; the board's ink change is the fill's
  contrast).
- File manifest: `crates/marley_workbench/src/component_library.rs` (Marley crate). No Zed path,
  so no `docs/marley/zed-touchpoints.md` row.

### Visual check plan
- REQ-001: setup writes two good templates and a broken one; open the library from the palette,
  click `card`; `936-01-chosen` shows two "loaded" chips and one red error chip, no drawn cards.
- REQ-002: click the `deploy_status` chip (place from the first run's shot); `936-02-template`
  shows its JSON in the box and the card drawn under it.
- REQ-003: `936-01-chosen` shows the header line with the hint.
- REQ-004: clear the box, type `{"type": "badge", "text": "beta", "tone": "orange"}`;
  `936-03-error` shows the error on the header line and no callout.
- REQ-005: `936-01-chosen` shows `card`'s row filled; `936-04-day` the same in Marley Day.

### Risks
- A long error on the header line could push the hint off; the error label must shrink and
  truncate (`min_w_0`, `truncate`), never the hint.
- Writing into the scratch editor from a click while the editor has focus elsewhere: `set_text`
  needs the window; the listener has it.
- Template file names can collide with element ids of the catalog cards; prefix chip ids with
  `marley-template-chip-`.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
