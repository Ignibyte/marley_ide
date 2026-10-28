---
pipeline_id: a8f36c51-77c1-470f-a4d7-bd2ce9f6ee5c
ticket: docs/planning/tickets/open/TICKET-942-search-and-editor-popups-as-the-board.md
status: Phase 4 — Complete PASS
title: Search and the editor's pop-ups as the board words them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/D-panels-editor.md, docs/marley/look-plan.md]
---

## Title
Search and the editor's pop-ups use the SearchMore and EditorPopups boards' words and first states:
project search opens with its Replace field shown; Open Recent reads "Open a recent project"; the
branch picker reads "Switch to a branch, or name a new one"; an extension's update button reads
"Update"; completions label their kinds with short words ("fn", "fld"); a grey prediction has a
"Tab takes the prediction" note beside it; and errors show at the line's end by default.

## Scope
### In
- `crates/search/src/project_search.rs` (Zed crate): while `MarleySearchLook` is set,
  `ProjectSearchView::new` starts with `replace_enabled` true, so the Replace field shows under the
  query, and the replacement editor's placeholder reads "Replace with". The replace toggle still
  hides the field for that search.
- `crates/recent_projects/src/recent_projects.rs` (Zed crate): `placeholder_text` reads "Open a
  recent project" while `MarleyRecentLook` is set.
- `crates/git_ui/src/branch_picker.rs` (Zed crate): the list states' `placeholder_text` reads
  "Switch to a branch, or name a new one" while `MarleyBranchLook` is set. The remote-name state
  keeps Zed's words.
- `crates/extensions_ui/src/components/extension_card.rs` (Zed crate): both "Upgrade" buttons
  (while upgrading, and when a newer version is out) read "Update" while `MarleyExtensionLook` is
  set.
- `crates/editor/src/code_context_menus.rs` (Zed crate): `MarleyCompletionKindLook`, a new global
  with a function from a `CompletionItemKind` to its word. While it is set,
  `render_completion_kind_letter` draws the word in the same syntax colour, in a badge six
  characters wide; the tooltip keeps the kind's full name.
- `crates/editor/src/edit_prediction.rs` (Zed crate): `MarleyPredictionNote`, a new global. While
  it is set, the `EditDisplayMode::Inline` arm of `render_edit_prediction_popover` draws a muted
  "<key> takes the prediction" after the line's end, the key from
  `edit_prediction_keybind_display`. The other display modes keep Zed's hints.
- `assets/settings/default.json` (Zed file): `diagnostics.inline.enabled: true` with a `Marley:`
  comment, so a diagnostic's message shows at the line's end.
- `crates/marley_workbench/src/search_looks.rs` (Marley crate): sets the two new globals with the
  others, and holds the kind words (D3).

### Out (explicitly deferred)
- Search results as `file:line` rows with the hit's line under each: #891 Out (Zed's multibuffer
  of excerpts stays).
- History as a list in the Changes inspector: #897 Out (Zed's git panel History tab stays).
- Open Recent's "open" label for a project open in another window: #854 Out (the picker does not
  know other windows).
- An edit prediction provider: the user's choice. Zed's own needs a sign-in; Marley sets none.
- Hover, code actions, the rename note and inline assist: they match the board already (#856,
  #899, #900).

## Reference (§20)
Upstream Zed for the editor, search and pickers. Kept: `search::project_search`'s replace row and
its toggle; `recent_projects`' and `git_ui::branch_picker`'s pickers; `extensions_ui`'s cards and
their install and update operations; `editor::code_context_menus`' completion rows with
`completion_menu_item_kind: "symbol"` (#856); `editor::edit_prediction`'s inline prediction and
its accept key; Zed's inline diagnostics (`diagnostics.inline`), drawn at the line's end. Marley
changes words, the replace row's first state and one default; every behaviour is Zed's.

### Prior art
- `crates/search/src/project_search.rs:256` `MarleySearchLook` (#891); `:1456`
  `ProjectSearchView::new`, `:1662` `replace_enabled: false`; `:3237` the replace line, drawn only
  while `replace_enabled`; `:1555` the only use of `REPLACE_PLACEHOLDER`
  (`crates/search/src/search.rs:84`, "Replace in project…").
- `crates/recent_projects/src/recent_projects.rs:75` `MarleyRecentLook` (#854); `:976`
  `placeholder_text`, "Search projects…".
- `crates/git_ui/src/branch_picker.rs:679` `MarleyBranchLook` (#891); `:1282` `placeholder_text`
  with "Switch or type to create a branch…" for the list states.
- `crates/extensions_ui/src/components/extension_card.rs:18` `MarleyExtensionLook` (#891);
  "Upgrade" at `:361` (while upgrading) and `:382` (a newer version is out).
- `crates/editor/src/code_context_menus.rs:1136` the kind slot, from
  `completion_menu_item_kind`; `:1660` `render_completion_kind_letter`; `:1727`
  `completion_kind_letter`, single letters ("f" for both a function and a field); `:1969`
  `MarleyCodeActionsLook`, the pattern for a global this menu asks.
- `crates/editor/src/edit_prediction.rs:1083` `render_edit_prediction_popover`: its
  `EditDisplayMode::Inline` arm returns `None`, so an inline prediction has no hint; the
  `TabAccept` arm calls `render_edit_prediction_end_of_line_popover` (`:1897`), which places an
  element after a line's end; `render_edit_prediction_keybind` (`:2220`) reads the accept key.
- `assets/settings/default.json:2113` `diagnostics.inline.enabled: false`.
- `crates/marley_workbench/src/search_looks.rs` sets the looks of #856, #891, #899 and #900.
- Scenarios to reuse: `script/e2e/856-editor-pop-ups.sh` (a scratch crate rust-analyzer reads),
  `script/e2e/891-search-branches-extensions.sh` (branches and an installed extension in the
  profile), `script/e2e/900-inline-assist-offers-the-agent.sh` (an Ollama stand-in). Zed's Ollama
  prediction provider posts to `/api/generate` (`crates/edit_prediction/src/ollama.rs:78`).
- No other crate owns these words or states; the behaviour maps have nothing on them.

## UI proof
Scenario `script/e2e/942-search-and-editor-popups-as-the-board.sh` (sway). Setup: #856's scratch
crate in a git repository with a second branch, a line holding an unused variable, and #891's
profile extension staged with its manifest's version lowered so its card offers an update; the
Ollama stand-in of #900 answering `/api/generate` with a fixed completion, and
`edit_predictions.provider: "ollama"` with `edit_predictions.ollama.api_url` pointing at it.
- `942-01-search`: project search opened: the Replace field under the query, "Replace with"
  (REQ-001).
- `942-02-recent`: Open Recent: "Open a recent project" (REQ-002).
- `942-03-branches`: the branch picker: "Switch to a branch, or name a new one" (REQ-003).
- `942-04-extensions`: the staged extension's card: "Update" (REQ-004).
- `942-05-completions`: `self.wid` and Ctrl+Space: kind words "fn" and "fld" (REQ-005).
- `942-06-prediction`: a line typed, the stand-in's grey text and "Tab takes the prediction" after
  it (REQ-006).
- `942-07-inline-error`: the unused variable's warning at the line's end with no pointer over it
  (REQ-007).
- `942-08-day`: Marley Day: completions open over a line with its inline warning (REQ-005,
  REQ-007).

## Locked-In Decisions
- D1 — Each change hangs off the look global its view already asks for (#854, #891) or a new one
  beside them. With the globals unset, every view keeps Zed's words and first state.
- D2 — Every new project search opens with the Replace field shown; the toggle hides it for that
  search only.
- D3 — The kind words: `fn` (function, method, constructor), `fld` (field, property), `var`
  (variable), `const` (constant), `mod` (module), `struct` (struct), `class` (class), `trait`
  (interface), `enum` (enum), `vrnt` (enum member), `type` (type parameter), `kw` (keyword),
  `snip` (snippet), `val` (value), `unit`, `file`, `dir` (folder), `ref` (reference), `event`,
  `op` (operator), `color`, `text`. The badge is six characters wide, so the names line up.
- D4 — The note names the key Zed binds to accept (Tab by default) and shows only for a prediction
  drawn inline. It takes no click.
- D5 — Inline diagnostics are on in Marley's defaults with Zed's padding, debounce and severity;
  a user turns them off in Settings.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens project search, Marley shall show the Replace field under the query, with the placeholder "Replace with". | `942-01-search` |
| REQ-002 | WHEN the user opens Open Recent, its field shall read "Open a recent project". | `942-02-recent` |
| REQ-003 | WHEN the user opens the branch picker, its field shall read "Switch to a branch, or name a new one". | `942-03-branches` |
| REQ-004 | WHILE an installed extension has a newer version, its card shall offer "Update". | `942-04-extensions` |
| REQ-005 | The completions menu shall label each item's kind with its short word in the kind's syntax colour, in Marley Night and Marley Day. | `942-05-completions`, `942-08-day` |
| REQ-006 | WHILE an edit prediction shows as grey text on the line, the editor shall show "Tab takes the prediction" after the line's end. | `942-06-prediction` |
| REQ-007 | WHILE a line has a diagnostic, the editor shall show its message at the line's end with Marley's default settings, in Marley Night and Marley Day. | `942-07-inline-error`, `942-08-day` |
| REQ-008 | WHILE the Marley look globals are unset, project search, Open Recent, the branch picker, the extension cards, the completions menu and the inline prediction shall keep Zed's words and first state. | the review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the five Zed hunks and the defaults line, with their ledger rows; the two globals
  and the word table in `search_looks.rs`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
