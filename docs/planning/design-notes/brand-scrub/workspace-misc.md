# Brand-scrub audit — workspace/misc cluster

**Scope:** `crates/marley_app/src/{workspace,grid_layout,nav,right_dock,file_tree_view,workflows,text_selection,editor_surface}.rs` (+ `marley_settings/value.rs` and test fixtures per the flag).
**Method:** `grep -niwE 'warp|zed'` (whole-word, case-insensitive) each file.
**Result:** 8 whole-word mentions across 6 files. **All are `Warp`; zero `Zed`.** **All are comments / doc-comments** — no strings, identifiers, theme-names, or test-data. None affect runtime, so **none are hard-KEEP**. Every mention uses Warp purely as a design/UX reference pointer; the meaning survives in the reference docs (which keep their brand names).

`text_selection.rs` and `editor_surface.rs` are **clean** (0 matches). `marley_settings/value.rs` **does not exist**; no matches anywhere under `crates/marley_settings/`; no matches in any `crates/**/*.toml` fixture.

| File:Line | Category | CURRENT (phrase) | PROPOSED reword | Notes |
|---|---|---|---|---|
| `crates/marley_app/src/workspace.rs:183` | doc-comment | `The kind of a pane (M5 #107) — the Warp typed-pane grid.` | `The kind of a pane (M5 #107) — the typed-pane grid.` | Drop the brand; "typed-pane grid" already names the concept. Keeps #107. |
| `crates/marley_app/src/workspace.rs:287` | doc-comment | `The Warp-layout enabler: not every pane owns a terminal…` | `The panel-grid enabler: not every pane owns a terminal…` | Meaning = panes can host non-terminal panels. Keeps #120 (same block). |
| `crates/marley_app/src/grid_layout.rs:3` | doc-comment (module) | `boot straight into a `\``[terminal \| files \| code \| git]`\`` Warp layout for the captures)` | `…a `\``[terminal \| files \| code \| git]`\`` reference layout for the captures)` | "reference layout" preserves the capture intent. Keeps #122 (same block). |
| `crates/marley_app/src/grid_layout.rs:4` | doc-comment (module) | `the Warp arrangement is a sessions sidebar plus a row of panes` | `the reference arrangement is a sessions sidebar plus a row of panes` | Same module blurb; describes the target layout, not the brand. |
| `crates/marley_app/src/nav.rs:5` | doc-comment (module) | `[`\`FoldState`\`] + […] add Warp-style folding: a folded block's output rows collapse` | `…add command-block folding: a folded block's output rows collapse` | "command-block folding" names the actual behavior. Keeps M12 #184. |
| `crates/marley_app/src/right_dock.rs:74` | doc-comment (verbatim chad quote) | `the compact top-bar icon shown in place of the text tab (chad's "icons like Warp")` | `…in place of the text tab (chad's "use icons, not text tabs")` | Quotes chad's original ask; reword keeps his intent, drops the brand. Keeps M7 #134. Not load-bearing, but review the paraphrase. |
| `crates/marley_app/src/file_tree_view.rs:1` | doc-comment (module) | `the file-explorer view helpers (M5 #113, the Warp explorer look)` | `…the file-explorer view helpers (M5 #113, the IDE file-explorer look)` | "IDE file-explorer look" keeps the visual reference generically. Keeps #113. |
| `crates/marley_app/src/workflows.rs:1` | doc-comment (module) | `command-palette workflows (#204, Warp Workflows analog): the saved-command model` | `command-palette workflows (#204 — reusable saved commands): the saved-command model` | Marley's own feature is already "workflows" (lowercase, in-code); the reword drops the branded-feature analogy while keeping #204. Not load-bearing. |

## Summary
- **8 mentions, 6 files, all `Warp`, all comments.** No `Zed` in scope. No brand in `text_selection.rs` / `editor_surface.rs`.
- **Zero load-bearing** — no runtime strings, config keys, theme-names, or test-data depend on the word. Safe to reword all 8.
- Every ticket number (#107, #120, #122, #113, #204, #184, #134) is preserved in the proposed rewords.
- Two to eyeball before applying: `right_dock.rs:74` (verbatim chad quote) and `workflows.rs:1` (references a branded feature name) — reworded, not KEEP, but the paraphrase is a judgment call.
- Flagged out-of-scope hits confirmed absent: `marley_settings/value.rs` does not exist; no brand in `marley_settings/**` or any `*.toml`.
