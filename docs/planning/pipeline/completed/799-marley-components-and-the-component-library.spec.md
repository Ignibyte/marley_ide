---
pipeline_id: ad5f1731-b85e-4ed6-916f-9e50fdaad500
ticket: docs/planning/tickets/closed/TICKET-799-marley-components-and-the-component-library.md
status: Phase 4 — Complete PASS
title: Marley components and the Component Library
type: feature
slice: Control plane (agent components)
references:
  - docs/planning/design-notes/agent-components-2026-10-17.md
---

## Title
A catalog of components an agent can ask for, described as a nested JSON tree and drawn natively
in the user's theme, and a Component Library tab that shows each one beside the JSON that draws
it.

## Scope
### In
- **`crates/marley_components`** (new, `MIT OR Apache-2.0`, `[lib] path =
  "src/marley_components.rs"`):
  - The tree: `{"type": "<name>", ...fields, "children": [...]}`, parsed with serde into a
    closed enum of the catalog's types; a parse error names the JSON path of the bad node.
  - The catalog: `text` (with `style`: body, muted, strong, code), `heading` (levels 1 to 3),
    `row` and `column` (gap, alignment), `card` (title, children), `divider`, `table` (columns
    with labels and alignment, rows of cells; a cell is text, a number or a `badge`), `key_value`
    (label and value pairs), `stat` (label, value, change with a tone), `badge`, `progress`
    (value of a maximum, label), `bar_chart` (labelled values, horizontal bars), `sparkline` (a
    series of numbers), `code` (text with a language name), `callout` (tone, title, body).
  - Colors by tone only: `default`, `muted`, `accent`, `success`, `warning`, `error`, mapped to
    the theme's colors (`ui::Color`). No component takes a color value.
  - Limits that keep a bad tree from stalling the UI: nesting at most 12 deep, at most 2,000
    nodes, a table drawn to 500 rows with a line saying how many more there are.
  - The renderer: one `RenderOnce` element per type, built from Zed's `ui` pieces (`data_table`,
    `progress`, `callout`, `chip`, `divider`, `label`); the sparkline through gpui's `canvas`
    and `PathBuilder`.
  - The catalog's description of itself: per type, a one-line description, the JSON Schema of
    its fields (`schemars`) and a worked example. #800's `component_list` answers with it.
- **The Component Library tab** (`crates/marley_workbench`, beside `guide_tab.rs`): a center tab
  listing the catalog by group (layout, text, data, charts, status), each with its description,
  its example drawn, and the example's JSON. At the top, a scratch box: JSON typed or pasted
  there is drawn live under it, or its error is shown.
- **Three ways in:** a `marley: open component library` action; a seventh button on Home's
  Configure card (`home_page.rs`, `render_configure`); an action link on Marley's Settings page
  (`settings_ui/src/marley_page.rs`, built by name as "System One Calls" is, #659).

### Out (explicitly deferred)
- Agents calling the components: #800.
- Templates from files: #801.
- Interaction: sorting a table, buttons that answer the agent, forms. The first catalog is for
  showing.
- Registering Marley's components in Zed's Component Preview: its registry is static
  (`inventory`), so templates (#801) could never join it, and the library has to be Marley's.

## Reference (§20)
Upstream Zed (`component`, `component_preview`, `ui`): Zed's Component Preview lists its `ui`
components by scope with a preview each; the Component Library does the same for the agent
catalog, in a Marley tab. Published material: A2UI's basic catalog
(<https://a2ui.org/specification/v0.9-a2ui/>) for the type names the two share.

### Prior art
- The code we ship: `ui`'s `data_table`, `progress`, `callout`, `chip`, `divider` and `label`
  draw every component but the bar chart and the sparkline; gpui's `canvas` and `PathBuilder`
  draw the sparkline; `schemars` (workspace, 1.0) writes the schemas; `component_preview`'s page
  layout (a list beside a preview) is the library's model; `marley_workbench/src/guide_tab.rs` is
  the tab pattern (`add_item_to_center`, reused by title).
- Behavior maps: none in `docs/warp_architecture/` or `docs/zed_architecture/`.
- Published material: A2UI, MCP Apps, json-render, compared in
  `docs/planning/design-notes/agent-components-2026-10-17.md`. json-render's catalog-plus-schema
  model is the closest to this one; its renderers are JavaScript.

## UI proof
Scenario `script/e2e/799-marley-components-and-the-component-library.sh`, under `compositor
sway` (it clicks Home's button and the Settings link):
- `799-01-palette`: the palette action opens the Component Library tab (REQ-001).
- `799-02-layout` to `799-05-status`: each group scrolled into view, every component drawn from
  its example with its JSON beside it (REQ-002).
- `799-06-scratch`: a table typed into the scratch box is drawn under it (REQ-003).
- `799-07-error`: a tree with an unknown type shows the error with its JSON path, and nothing is
  drawn (REQ-004).
- `799-08-limit`: a table of 600 rows pasted into the box draws 500 and the "100 more rows" line
  (REQ-005).
- `799-09-home` and `799-10-settings`: Home's button and the Settings link each open the tab
  (REQ-001).
- `799-11-theme`: the library under a light theme: tones follow the theme (REQ-006).

## Locked-In Decisions
- **D1 — A nested tree, not A2UI's flat list.** An agent writes a nested tree in one call more
  reliably, the tool's schema teaches it, and A2UI is a preview with no GPUI renderer. Type
  names follow A2UI's where the two overlap.
- **D2 — Tones, not colors.** Every color comes from the theme.
- **D3 — Data in the tree.** No data model or bindings in this ticket; #801's placeholders are
  the first indirection.
- **D4 — The renderer lives in `marley_components`, the tab in `marley_workbench`**, so #800's
  hook in `agent_ui` can call the renderer through a `Global` function pointer without
  `agent_ui` depending on Marley crates.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the palette action runs, Home's Components button is clicked or the Settings link is clicked, Marley shall open the Component Library tab, or bring forward the one open | shots `799-01-palette`, `799-09-home`, `799-10-settings` |
| REQ-002 | The library shall show every catalog type with its description, its example drawn and the example's JSON | shots `799-02` to `799-05` |
| REQ-003 | WHEN valid JSON is in the scratch box, the library shall draw it under the box | shot `799-06-scratch` |
| REQ-004 | WHEN the scratch box's JSON is not a valid tree, the library shall show the parse error with the bad node's JSON path and draw nothing | shot `799-07-error` |
| REQ-005 | WHEN a table has more than 500 rows, the system shall draw 500 and a line naming how many more there are | shot `799-08-limit` |
| REQ-006 | The components shall take every color from the active theme | shot `799-11-theme` |

## Phase Plan
- **P1 Plan:** this spec; the design in the notes at promotion (each type's fields, the error
  type, the tab's layout).
- **P2 Code:** the crate, the tab, the three ways in, the touchpoints rows (`settings_ui`'s
  Marley page, `Cargo.toml`, `Cargo.lock`, `zed`'s manifest if it needs the crate); a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test:** the visual check above.
- **P4 Complete:** CHANGELOG, `docs/marley/guide.md` (the catalog), ledger, close, archive,
  commit.
