---
pipeline_id: 26955ac4-4003-4da4-89a3-0bff2205530a
ticket: docs/planning/tickets/open/TICKET-936-component-library-as-the-board.md
status: Phase 4 — Complete PASS
title: The Component Library as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/F-tools.md, docs/marley/look-plan.md]
---

## Title
The Component Library's templates as a strip of chips, Try it's hint and error on its header
line, and the chosen component highlighted in the left list, as the Components board draws them.

## Scope
### In
- **Templates as a chip strip** (Marley crate, `component_library.rs`, `templates_group`). The
  heading "Your templates" with "components/ in Marley's config folder" beside it as a link that
  opens the folder (it replaces the Open Folder button and the sentence under the heading). Under
  it, one chip per file in a wrapping row: the file name, then "loaded" in the success colour, or
  the file's error in the error colour, cut short with the whole error in a tooltip. The drawn
  template cards and the error callouts go. "No templates yet." stays for an empty folder.
- **A loaded chip draws its template** (`templates_group`, `ComponentLibrary`). A click on a
  loaded chip writes the template's example tree (`Template::example_tree`, pretty JSON) into the
  Try it box, which draws it. A click on a broken chip opens its file in the workspace.
- **Try it header** (`ComponentLibrary::render`). The header line reads "Try it" and then, muted,
  "Paste JSON; it draws as you type." While the box holds JSON that does not draw, the parser's
  error sits at the right of the same line in the error colour, and the callout under the field
  goes. A tree that draws is drawn under the field as now.
- **The chosen component** (`ComponentLibrary::render_navigation`). While the find field holds
  exactly one component's name, that name's row is filled with the selected-element colour and
  its text drawn in the default colour; the other rows stay as they are.

### Out (explicitly deferred)
- The bottom row for the Component Library ("Component Library", a red chip for a broken
  template, "17 components · 3 templates"): the review lists it as missing; #909 (the
  bottom row for other screens) builds it.
- A line number for every error ("line 3: …"): Marley's parser names a JSON path, not a line
  (`ParseError { path, message }`). Turning a path into a line needs a JSON parser that keeps
  positions; not this slice.
- The page's order (the board shows the chosen card, then Try it, then the templates): the page
  keeps Try it at the top, where L-claude-799 says a long page is checked best.

## Reference (§20)
N/A — Marley-specific: the Component Library is Marley's own tab (#799, #801, #849), drawn with
Zed's `ui` components (`Label`, `Chip`-like pills, `Tooltip`); no Zed or Warp view to match.

### Prior art
- `crates/marley_workbench/src/component_library.rs:346-427` `templates_group`: the Templates
  heading, Open Folder (`:358-362`), the folder sentence (`:365-372`), one drawn card per
  template (`:383-416`), one error `Callout` per broken file (`:417-426`). Gap holds at HEAD.
- `component_library.rs:288-311`: the Try it box: a muted "Try it" label (`:297-300`), the
  scratch editor with the placeholder "Type or paste a component's JSON here to draw it"
  (`:142-146`), and `tried` drawn under it (`:266-270`, `:310`), an error through
  `marley_components::render_error`, a `Callout` (`crates/marley_components/src/marley_components.rs:1123-1132`).
  Gap holds at HEAD.
- `component_library.rs:187-246` `render_navigation`: a name's click writes it into the find
  field (`:215-219`); every row has the same look and only a hover colour (`:214`). Gap holds.
- `marley_components.rs:65-77` `ParseError` is a JSON path and a message; `:545-548`
  `parse_str_with` turns a JSON syntax error into `$: The text is not JSON: … at line L column
  C.` (serde_json's own wording, so a syntax error already carries its line).
- `marley_components.rs:775-781` `Template` (name, description, path, `example_tree`) and
  `:826-831` `BrokenTemplate { path, error }`: everything a chip needs is already loaded into
  `MarleyTemplates`.
- Zed's `ui` crate: `Chip` (`crates/ui/src/components/chip.rs`), `Tooltip::text`, and
  `ButtonLike`/`h_flex().on_click` rows as Home's pills use them (#903 `home_page.rs` `pill`).
- The selected-row fill: `colors.element_selected` / `ghost_element_selected`, as the rail and
  the Threads page use for their chosen row.
- Specs: #799 (the tab), #801 (templates; its REQ-001 drew each template, which the strip keeps
  one click away), #849 (the left list; its D1 filter, kept).

## UI proof
Scenario `script/e2e/936-component-library-as-the-board.sh` (sway). Setup: a scratch repository,
and the run's config folder `components/` holding two good templates (#801's `deploy_status.json`
and a second) and one broken file, as #801's scenario writes them. No agent and no Rusty.
- `936-01-chosen`: the library open, `card` typed into the find field (the state a click on its
  name in the left list writes, #849): the name filled, the page
  narrowed to it, Try it's header with its hint, and the template strip (REQ-001, REQ-003,
  REQ-005).
- `936-02-template`: `deploy_status` chip clicked: its JSON in Try it and its card drawn under
  it (REQ-002).
- `936-03-error`: a component with a wrong tone pasted into Try it: the error on the header line,
  no callout (REQ-004).
- `936-04-day`: `936-01-chosen`'s view in Marley Day (REQ-001, REQ-003, REQ-005).

## Locked-In Decisions
- D1 — The strip replaces the drawn template cards, as the board shows; a loaded chip clicked
  draws its template in Try it, so #801's "see a template drawn" stays one click away.
- D2 — A broken chip shows the error Marley already words, cut short; the tooltip holds the
  whole of it. No line numbers are computed (Out).
- D3 — A name is chosen while the find field holds exactly that name. Typing anything else
  unmarks it. #849's filter is unchanged.
- D4 — The folder link opens the folder with the system, as Open Folder does now.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The Component Library shall show its templates under "Your templates" and the folder link as one chip per file, "loaded" in the success colour or the file's error in the error colour, with no drawn template cards. | `936-01-chosen`, `936-04-day` |
| REQ-002 | WHEN the user clicks a loaded template's chip, the Try it box shall hold that template's example and draw it. | `936-02-template` |
| REQ-003 | The Try it box's header line shall read "Try it" followed by "Paste JSON; it draws as you type." | `936-01-chosen` |
| REQ-004 | WHILE the Try it box holds JSON that does not draw, the system shall show the parser's error on the header line in the error colour and no callout under the field. | `936-03-error` |
| REQ-005 | WHILE the find field holds exactly one component's name, the left list shall draw that name's row filled as selected. | `936-01-chosen`, `936-04-day` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `component_library.rs` only; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
