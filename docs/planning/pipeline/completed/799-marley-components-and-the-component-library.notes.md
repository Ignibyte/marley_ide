# Marley components and the Component Library — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-799-marley-components-and-the-component-library.md
- **Pipeline spec:** 799-marley-components-and-the-component-library.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see the ticket). He confirmed the batch the same day.
- **Research:** `docs/planning/design-notes/agent-components-2026-10-17.md`.
- **Discovery (drafted by /spec; re-verified at promotion):**
  - Zed's registry: `crates/component/src/component.rs` (`Component`, `ComponentScope`,
    `register_component`), `crates/ui_macros/src/derive_register_component.rs`,
    `crates/component_preview/src/component_preview.rs` (the page layout, `PreviewPage`).
  - `ui` pieces: `crates/ui/src/components/{data_table,progress,callout,chip,divider,label}.rs`.
  - Home: `crates/marley_workbench/src/home_page.rs`, `render_configure` (a fixed array of six
    buttons dispatched by id).
  - Settings: `crates/settings_ui/src/marley_page.rs`, the "System One Calls" `ActionLink`
    (built by name with `cx.build_action`, sent to the original window, Settings closed).
  - Tab pattern: `crates/marley_workbench/src/guide_tab.rs` (`open`, reused by title).
  - Workspace deps: `schemars` 1.0, `serde_json` with `preserve_order`.
- **Recall (§18.3):** the ledgers hold nothing on component catalogs; the rules that bind here
  are PR-claude-798-read-no-pane-bounds-in-render-001 (nothing that locks layout state in render;
  the library and the renderer read only their own data) and the dylint rule for
  `SharedString::new_static` (L in the port traps). Completed pipelines: #733's guide tab (the tab
  pattern), #659's "System One Calls" action link (the Settings pattern).
- **Promoted 2026-10-17**, seams re-read on the merged tree.
- **Checklist (no TaskCreate):** pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓ (the
  spec's), spec ✓, design ✓.

### Design
- **`crates/marley_components` (Marley, new):** `parse.rs` (the tree: a walk over the JSON that
  checks `type` against the catalog, takes `children` for `row`, `column` and `card`, reads each
  component's own fields with `serde_path_to_error` so an error names its JSON path, and enforces
  the limits), `render.rs` (one element per type from `ui`: `Label`, `Headline`, `Table`, `Chip`,
  `ProgressBar`, `Callout`, `Divider`; the sparkline on a `canvas`; colors only through `Tone`),
  `catalog.rs` (each type's group, description in Strict STE since agents read it, schema from
  `schemars` with `type` and `children` added, and an example).
- **`marley_workbench/src/component_library.rs` (Marley, new):** `OpenComponentLibrary`; the tab
  with the scratch box (an auto-height `Editor`, parsed on every edit) and the groups.
- **Ways in:** Home's Configure card (a seventh button); `settings_ui/src/marley_page.rs` (Zed
  path, Marley's file; an `ActionLink` by name); the palette.
- **Manifests:** `Cargo.toml` (member, workspace dependency), `marley_workbench/Cargo.toml`.

### Visual check plan
- `script/e2e/799-marley-components-and-the-component-library.sh`, `compositor sway`: the palette
  opens the library (`799-01-palette`); scrolled shots of the groups (`799-02` to `799-05`); JSON
  pasted into the scratch box through the run's clipboard (`wl-copy` on the run's display) for the
  table, the error and the 600-row table (`799-06` to `799-08`); Home's button and the Settings
  link (`799-09`, `799-10`); a light theme (`799-11`, `theme` set in the profile).

## Phase 2 — Code
- **Built:** `crates/marley_components` (one file, `marley_components.rs`: the tree's parse with
  JSON paths and limits, the renderer, the catalog with schemas and examples);
  `marley_workbench/src/component_library.rs` (the tab, the scratch box, `OpenComponentLibrary`);
  Home's Components button; the Settings page's action link; the manifests and their rows.
- **Deviations from the plan:**
  - One file, not three modules: nursery's `redundant_pub_crate` refuses `pub(crate)` and
    `pub(super)` in a private module, rustc's `unreachable_pub` refuses a `pub` that is not
    re-exported, and the parse, render and catalog code share their private types. Zed's rules
    prefer fewer files, and the crate is one component.
  - `unused_results = "allow"` in the crate's lint table, with the comment the other gpui crates
    carry (`marley_workbench`, `marley_browser`).
  - The text size enum is `TextScale`: the `ui` prelude exports a `TextSize`.
  - `node()` dispatches to one `*_element` function per long type (`too_many_lines`).
- **Review of the diff:** the renderer and the library read only their own data in render (no
  lock or entity that layout holds, PR-claude-798-read-no-pane-bounds-in-render-001); a table row
  never has more cells than columns (`Table::row` asserts it), since the parse refuses longer rows
  and the renderer pads shorter ones; the catalog's descriptions are Strict STE; no Warp source.
- **Gate run 1 (`799-gate-1.log`):** **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/799-marley-components-and-the-component-library.sh` (`compositor
  sway`). Trees go into the scratch box through the run's clipboard (`wl-copy` on the run's display,
  Ctrl+A, Ctrl+V). Scroll distances differ between runs, so the 600-row table's end is found in a
  series of shots (`799-08-limit-01` to `-14`, the one holding row 499 picked by OCR), and the
  callout, which sits at the page's end, is shown through the scratch box at the top.
- **Fixes the check found (Code, then gate run 3):**
  - A callout had no icon: Zed's `Callout` draws one only when given one, while the catalog says
    the tone sets the color and the icon. `Tone::icon` gives Info, Check, Warning or XCircle.
  - A progress bar, a bar chart and a sparkline in a `column` shrank to their labels' width (the
    column's default `align: start` stops stretching); each now takes the full width.
  - The error fixture's unknown type `tabel` failed gate:18 (typos); it is `gauge`.
- **Shots, each read (`scratchpad/shots-799/`, the error re-shot in `shots-799-final/`):**
  - `799-01-palette`: the palette opens the Component Library tab: title, the line about
    `component_show`, the Try it box, the Layout group with `row` (two stats, the change in the
    success color) beside its JSON (REQ-001).
  - `799-02-layout`: `column`, `card` (titled "staging", its border the success green,
    `srgb(161,193,129)` sampled), `divider`; `799-03-text-data`: `divider`, `text`, `heading`,
    `code` (rust, the editor font); `799-04-charts` (first run): `table` with a right-aligned
    Latency column, striped rows, success, warning and error badges and its caption; `key_value`
    with "failing" in the error color; `stat` with "+2 today" in the warning color; `bar_chart` with
    bars in proportion and the warning bar gold; `799-08-limit` (first full run): `sparkline`, a line
    through the eight values; `799-05-status-end`: `badge`, `progress` at 64%, `callout`'s header
    (REQ-002).
  - `799-06-scratch`: the pasted card with its table drawn under the box (REQ-003).
  - `799-07-error` (`shots-799-final`): "Marley could not draw this component" and
    "$.children[1]: The type `gauge` is not in the catalog. The types are: text, heading, …";
    nothing drawn (REQ-004).
  - `799-08-limit-11`: rows down to "row 499", then "100 more rows" (REQ-005).
  - `799-06b-callout`: the warning callout with its triangle icon, two badges, a full-width
    progress bar at 64%.
  - `799-09a-home`, `799-09-home`: Home's Configure card with Components; its click opens the
    library tab (REQ-001).
  - `799-10a-settings`, `799-10-settings`: the library closed and the Settings window searched to
    Marley › Agents › Component Library; its button closes Settings and opens the library (REQ-001).
  - `799-11-theme`: One Light: the pasted card and table in the light palette, the badges in their
    tones (REQ-006).
- **Gate run 2 (`799-gate-2.log`):** red on gate:18 (`tabel`). **Gate run 3 (`799-gate-3.log`):
  GATE GREEN [diff].**

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("Marley components and the Component Library");
  `docs/marley/guide.md` ("The Component Library (#799)"). Touchpoints rows: `Cargo.toml`
  (thirteen members, `marley_components`), `settings_ui/src/marley_page.rs` (the action link);
  gate:16 green.
- **Knowledge appended:** L-claude-799-checking-a-long-scrolling-page-in-a-scenario-001. The two
  look-fixes (the callout's icon, the shrinking bars) were caught before they shipped and carry no
  rule beyond the lesson.
- **Brain:** no decision recorded; the format choice (a nested tree, not A2UI) is in the design
  note and the spec's D1.
