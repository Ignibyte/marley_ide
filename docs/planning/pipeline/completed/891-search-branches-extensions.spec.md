---
pipeline_id: 3644714f-261a-4f63-b828-5137ba4a1b87
ticket: docs/planning/tickets/closed/TICKET-891-search-branches-history-extensions.md
status: Phase 4 — Complete PASS
title: Project search's count, the branch picker's dots and notes, and the extensions' rows
type: feature
slice: The Marley look, W5 (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
As the SearchMore board draws them: project search counts its matches and the files that hold
them ("3 of 14 matches in 6 files"); the branch picker gives each branch a dot and a note
(current, N behind, worktree, remote); and each extension's card leads with a tile of its name's
first letters and says Installed in the eyes' green.

## Scope
### In
- `crates/search/src/project_search.rs` (Zed crate): the count's words while Marley's look is set.
- `crates/git_ui/src/branch_picker.rs` (Zed crate): `MarleyBranchLook`, a hook for a branch row's
  dot and note.
- `crates/extensions_ui/src/components/extension_card.rs` (Zed crate): `MarleyExtensionLook`, a
  hook for the card's tile, and Installed.
- `crates/marley_workbench/src/search_looks.rs` (new): sets the three looks.

### Out (explicitly deferred)
- History in the Changes inspector and the guide's Try it: TICKET-897, new behaviour.
- File and line over each search hit: Zed's results draw each file's header with its path, and
  the gutter gives the line.

## Reference (§20)
Upstream Zed for the three views: `search::project_search`, `git_ui::branch_picker`,
`extensions_ui`. The hooks follow #854's `MarleyRecentLook` and #883's `MarleySwitcherLook`.

### Prior art
- The search's count is `"{index}/{matches}"` in its toolbar; the results' multibuffer holds the
  files that matched.
- `git::repository::Branch` carries `upstream` with `UpstreamTracking::Tracked { ahead, behind }`;
  `Repository::linked_worktrees` names each worktree's branch.
- `ExtensionCard::render` heads the card with the name, the version and "(vX installed)".
- The brain (consultation b7caf5a1): nothing on this seam.

## UI proof
Scenario `script/e2e/891-search-branches-extensions.sh` (sway), from the probe
`891-branch-picker.sh`: a scratch repository on `feature/one` behind `main`, a worktree on
`agent/rail-strip`, a remote `origin`, and files that hold a word.
- `891-01-search`: project search for the word: "N matches in M files" (REQ-001).
- `891-02-branches`: `git: branch`: dots and notes (REQ-002).
- `891-03-extensions`: the extensions page: tiles, and Installed on an installed one (REQ-003).
- `891-04-day`: Marley Day, the branch picker (REQ-002).

## Locked-In Decisions
- D1 — Each look is a global Marley sets at start; without it Zed's own rows draw, so Zed's
  tests and the Zed layout see no change.
- D2 — A branch's note: current for the head; N behind for a local branch behind its upstream
  (N ahead when only ahead); worktree for a branch checked out in a linked worktree; remote for
  a remote branch. The dot is the coat's orange for the head, the lilac for a worktree, and
  muted otherwise.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project search has matches, its toolbar shall say how many matches in how many files, with the active match's place. | `891-01-search` |
| REQ-002 | The branch picker shall show each branch with a dot and a note: current, behind, worktree or remote, in Marley Night and Marley Day. | `891-02-branches`, `891-04-day` |
| REQ-003 | Each extension's card shall lead with a tile of its name's first letters, and an installed one shall say Installed in green. | `891-03-extensions` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
