# Project search's count, the branch picker's dots and notes, and the extensions' rows — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-891-search-branches-history-extensions.md
- **Pipeline spec:** 891-search-branches-extensions.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #893 (#890 waits on Chad).
- **Classification / tier:** feature, M. Three small hooks in Zed crates, one Marley file.
- **Recall (§18.3):** a survey of the five seams: the search counts matches only; the branch
  picker has the upstream's ahead and behind and the repository its worktrees, but draws neither;
  the extensions card has no Marley code; there is no history list, and the guide has no Try it
  and the Browser runs no command from a page. The last two are new behaviour: split to #897. The
  brain had nothing.
- **Decisions:** D1, D2 in the spec.

### Design
- `project_search.rs`: `pub struct MarleySearchLook` (a `Global`); with it set, the count reads
  "I of N matches in F files" (F from the results' multibuffer), else Zed's "I/N".
- `branch_picker.rs`: `MarleyBranchRow { is_head, is_remote, ahead, behind, in_worktree }`,
  `MarleyBranchMark { dot, note }`, `MarleyBranchLook(fn(&MarleyBranchRow, &App) ->
  MarleyBranchMark)`; `render_match` puts the dot where the icon was and the note after the name.
- `extension_card.rs`: `MarleyExtensionLook(fn(&str, &App) -> AnyElement)`; with it set, the tile
  leads the card's name and "(vX installed)" reads "Installed vX" in `Color::Success`.
- `search_looks.rs`: `init` sets the three; `branch_mark` and `extension_tile`.
- **File manifest:** the three Zed files (ledger rows), `search_looks.rs`, `marley_workbench.rs`,
  `Cargo.toml` of `marley_workbench` if a dependency is missing, the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001 | files holding `rail_order`, Ctrl+Shift+F and the word | `891-01-search` |
| REQ-002 | the probe's repository, `git: branch` | `891-02-branches`, `891-04-day` |
| REQ-003 | `zed: extensions`; the run reaches no registry, so the installed tab or a dev extension | `891-03-extensions` |

## Phase 2 — Code
- Built as designed: `MarleySearchLook` in `project_search.rs`; `MarleyBranchRow`,
  `MarleyBranchMark` and `MarleyBranchLook` in `branch_picker.rs`; `MarleyExtensionLook` in
  `extension_card.rs`, re-exported through `components.rs` and `extensions_ui.rs`;
  `search_looks.rs` sets the three; `search` and `extensions_ui` joined the workbench's
  dependencies.
- Changed after the first shots:
  - Zed's card shows "(vX installed)" only for an older installed version, so the installed
    HTML said nothing. The card gained `is_installed`, and with the look it says Installed (with
    the version when older) for any installed extension.
  - The scenario's first search step pressed Ctrl+Shift+F with the terminal focused, which
    opened the terminal's own find bar. It runs `pane: deploy search` now. The fixture moved
    `main` one behind `origin/main` and pushed a remote-only `release`, since the current branch
    shows "current" first.
- Review: each hook is read only while the global is set (D1); the worktree check reads the
  repository's `linked_worktrees` through the picker's own repository handle.
- Gate: the first `just gate-diff` failed shellcheck (backticks in single quotes in the scenario), the ledger (two paths in one row) and cargo-sort (the new dependencies out of order); fixed at the source, the second run 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/891-search-branches-extensions.sh` (sway).
- `891-01-search`: `rail_order` in project search: "1 of 4 matches in 3 files" at the toolbar's
  right, the three files' results under it (REQ-001).
- `891-02-branches`: feature/one with the orange dot and "current", agent/rail-strip lilac and
  "worktree", main "1 behind", and under Remote Branches origin/release "remote" (REQ-002).
- `891-03-extensions`: every card led by its tile (Ht, Gi, To, Ja, Ca, Do); HTML, installed in
  the profile, says Installed in green (REQ-003).
- `891-04-day`: the branch picker in Marley Day, the head's row in the coat's tint (REQ-002).

## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, the four ledger rows, look-plan W5.
- Brain decision recorded on consultation b7caf5a1.
