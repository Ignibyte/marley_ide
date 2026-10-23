# Search and the editor's pop-ups as the board words them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-942-search-and-editor-popups-as-the-board.md
- **Pipeline spec:** 942-search-and-editor-popups-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11: SearchMore, EditorPopups).
- **Classification / tier:** feature, S–M. Five small Zed hunks behind look globals, one default,
  and Marley's side in `search_looks.rs`.
- **Recall (§18.3):**
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the kind-word function and the
    note are called from the editor's render; they take a kind or nothing and read no view.
  - F-claude-899-a-popover-row-outside-its-list-let-the-click-through-001: a row added to one of
    Zed's pop-ups outside its list occludes. The prediction note is not a row and takes no click,
    so it must not occlude: a click past the line's end still moves the caret.
  - L-claude-897-look-for-zeds-own-view-before-building-one-001: Zed's `TabAccept` mode already
    draws a hint after the line's end; the note reuses its placement, not a new layout.
  - PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001: the Ollama stand-in listens
    on a loopback port the scenario picks; the user's own Ollama is never asked.
  - #891's and #856's scenarios hold the repository, branch, extension and crate fixtures; #900's
    holds the Ollama stand-in.
  - The brain: to be consulted when the spec is promoted.
- **Discovery:** `project_search.rs` (`ProjectSearchView::new`, the replacement editor's
  placeholder), `recent_projects.rs` (`placeholder_text`), `branch_picker.rs` (`placeholder_text`),
  `extension_card.rs` (the two "Upgrade" buttons), `code_context_menus.rs`
  (`render_completion_kind_letter` and its caller in `CompletionsMenu::render`),
  `edit_prediction.rs` (`render_edit_prediction_popover`'s inline arm), `default.json`
  (`diagnostics.inline`), `search_looks.rs`.
- **Decisions:** D1–D5 in the spec.

### Design
- **Replace row:** in `ProjectSearchView::new`, `replace_enabled: cx.has_global::<MarleySearchLook>()`
  with a `// Marley:` comment, and the replacement editor's placeholder "Replace with" under the
  same check. The toolbar's toggle reads `replace_enabled`, so it shows pressed.
- **Placeholders and "Update":** each `placeholder_text` and button label checks its existing
  global (`cx.has_global::<MarleyRecentLook>()`, `MarleyBranchLook`, `MarleyExtensionLook`).
- **Kind words:** `pub struct MarleyCompletionKindLook { pub word: fn(CompletionItemKind) ->
  Option<&'static str> }` in `code_context_menus.rs`, a `Global` and `Copy`. `CompletionsMenu::render`
  reads it once with `cx.try_global` and hands the function to `render_completion_kind_letter`,
  which draws the word when there is one and Zed's letter otherwise. `search_looks.rs` holds
  `fn completion_kind_word(kind: CompletionItemKind) -> Option<&'static str>` with D3's table.
- **Prediction note:** `pub struct MarleyPredictionNote;` in `edit_prediction.rs`. In the
  `EditDisplayMode::Inline` arm, when the global is set, build a muted XSmall label "<key> takes
  the prediction" (the key's text from `edit_prediction_keybind_display(Inline)`), lay it out and
  place it as `render_edit_prediction_end_of_line_popover` places its element, with no popover
  frame. Return `None` when there is no key or the line is too wide to fit it.
- **Inline diagnostics:** `"enabled": true` in `default.json`'s `diagnostics.inline`, with a
  `// Marley:` comment.
- **File manifest:** `crates/search/src/project_search.rs` (Zed crate),
  `crates/recent_projects/src/recent_projects.rs` (Zed crate), `crates/git_ui/src/branch_picker.rs`
  (Zed crate), `crates/extensions_ui/src/components/extension_card.rs` (Zed crate),
  `crates/editor/src/code_context_menus.rs` (Zed crate), `crates/editor/src/edit_prediction.rs`
  (Zed crate), `assets/settings/default.json` (Zed file), `crates/marley_workbench/src/search_looks.rs`
  (Marley crate), `script/e2e/942-search-and-editor-popups-as-the-board.sh` (Marley-owned).
- **Ledger rows (docs/marley/zed-touchpoints.md):** extend the rows of `project_search.rs` (the
  replace row's first state and placeholder), `recent_projects.rs` (the placeholder),
  `branch_picker.rs` (the placeholder), `extension_card.rs` ("Update"), `code_context_menus.rs`
  (`MarleyCompletionKindLook`) and `default.json` (`diagnostics.inline.enabled`); a new row for
  `crates/editor/src/edit_prediction.rs` (`MarleyPredictionNote`).

### Visual check plan
- REQ-001: open project search (`pane: deploy search`); `942-01-search` shows the Replace field and
  its placeholder.
- REQ-002: `projects: open recent`; `942-02-recent` shows the field's placeholder.
- REQ-003: `git: branch`; `942-03-branches` shows the placeholder over the branches.
- REQ-004: `zed: extensions`, the Installed filter; `942-04-extensions` shows the staged extension
  with "Update". Needs the registry: when the run cannot reach it, the notes say so and the diff
  review verifies the label.
- REQ-005: in `src/main.rs`, type `self.wid` and press Ctrl+Space; `942-05-completions` shows "fn"
  and "fld".
- REQ-006: type the start of a line the stand-in completes; wait past the debounce;
  `942-06-prediction` shows the grey text and the note.
- REQ-007: the unused variable's line with rust-analyzer's warning; `942-07-inline-error` shows
  the message at the line's end.
- REQ-005 and REQ-007 in Day: `profile_setting theme '"Marley Day"'`; `942-08-day`.
- REQ-008 has nothing to see: the review of the diff checks each hunk sits behind its global.

### Risks
- An inline prediction may not show while rust-analyzer's completions are up (eager mode shows it
  only with no completions); the scenario types where rust-analyzer offers none, or presses Escape
  first.
- The note after a long line may not fit the editor's width; it is left out then rather than
  drawn over the text.
- Inline diagnostics on by default change every editor; Zed's debounce keeps the redraws as they
  are in Zed with the setting on.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
