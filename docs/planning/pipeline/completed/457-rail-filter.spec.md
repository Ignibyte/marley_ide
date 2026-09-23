---
pipeline_id: 89d006e6-aff2-4bcc-b690-3fcf40ab44e8
ticket: docs/planning/tickets/closed/TICKET-457-rail-filter.md
status: Phase 4 — Complete PASS
title: A filter for the rail
type: feature
slice: workbench shell W6h
references: [docs/marley/workbench-shell.md, docs/planning/pipeline/completed/453-rail-keyboard-and-reorder.spec.md, docs/warp_architecture/observed/beautifului-2026-08-12-notes.md]
---

## Title
A filter field under the rail's header narrows the rail to the projects, terminals and threads
whose name or title contains what is typed, with the matched characters highlighted, as Zed's
Threads Sidebar filter does. `ctrl-f` (`cmd-f` on macOS) reaches it from the rail; up, down and
Enter work from inside it, and Escape clears it.

## Scope
### In
- **The rows** (`marley_rail`, pure). While the filter holds text:
  - a project shows when its name matched or one of its terminals or threads did;
  - a project whose name matched shows every row under it, and any other shown project only
    the rows that matched;
  - the fold is ignored;
  - each row carries the characters to highlight;
  - a shown header's attention dot covers the rows the rail is not showing.

  `first_match` is the first shown row that matched. One visibility function serves every row
  type, and `rail_rows`, `selection` and #453's cursor functions all read it.
- **Matching** (`marley_workbench::rail`): Zed's
  `agent_ui::threads_archive_view::fuzzy_match_positions` over each project's name and each
  terminal's and thread's title, which are the strings the rows draw. It runs synchronously
  while the snapshot is built.
- **The field.** A row under the header holds:
  - a search icon and a single-line editor with the placeholder "Filter…";
  - at its right, the filter's key while the field is empty, or a clear button while it has
    text.

  "No matches" shows when the filter hides every row. While filtering, the fold chevrons are
  hidden and left and right do not fold.
- **Keys.** The Marley keymap binds `secondary-f` to Zed's `agents_sidebar::FocusSidebarFilter`
  in `MarleyRail && !Picker`. In the field:
  - up and down move the keyboard's row, and Enter opens it: Zed's defaults reach the rail's
    handlers past the single-line editor;
  - each edit moves the keyboard's row to the first match;
  - Escape works as in Zed's sidebar. In the field it clears the text, or with none moves focus
    to the rows. On the rows it clears the text. Otherwise it does what it did before.

### Out (explicitly deferred)
- Fuzzy matching, and case folding beyond ASCII: Zed's matcher is a substring match that
  ignores ASCII case.
- Matching a terminal's second line or a thread's worktrees.
- Keeping the filter across restarts; Zed's sidebar does not.
- Vim's `/` to reach the filter.
- Typing on the rows to start filtering.

## Reference (§20)
- **Upstream Zed:** the Threads Sidebar's filter (`crates/sidebar/src/sidebar.rs`):
  - a single-line editor in the header (`:852-856`, `:7328-7402`);
  - `fuzzy_match_positions` (`crates/agent_ui/src/threads_archive_view.rs:104-128`) on group
    names and thread and terminal titles, run synchronously in the rebuild (`:1862-1949`);
  - a group shows when it or a child matches, and a name match shows every child
    (`:1886-1921`);
  - collapsed groups show their matching rows, and headers lose the chevron (`:1591`,
    `:2448-2458`);
  - "No threads match your search." (`:7282-7301`);
  - `ctrl-f` and `cmd-f` to `agents_sidebar::FocusSidebarFilter` in `ThreadsSidebar`
    (`assets/keymaps/default-linux.json:752`, `default-macos.json:807`);
  - `menu::Cancel` clears the text, then hands focus to the list (`:3325-3355`);
  - each edit selects the first match (`:880-889`, `:2196-2209`).

  Marley keeps the matcher, the rules and the keys. It differs in three places:
  - the rail's focus key lands on the rows (#453), not in the filter;
  - the keyboard's row goes to the first row that matched, header or not;
  - the highlighted row is drawn while typing, since #453's cursor shows while the rail holds
    focus.
- **Warp:** N/A. `docs/warp_architecture/` describes no session filter; the sweep covered
  `crates/fuzzy_match.md`, `subsystems/07-app-entry-build-tooling.md` and `observed/`. The
  rail's visual reference puts a quick-search field directly under the header, with a key hint
  at its right edge (`docs/warp_architecture/observed/beautifului-2026-08-12-notes.md:13`).

### Prior art
- **Behavior maps:** `docs/zed_architecture/subsystems/09-vim-keymap-contexts.md` (depth and
  load order, and `!` over the whole stack, `:99-120`); the beautifului capture above; nothing
  on a Warp session filter.
- **Published material:** Zed's key-binding docs on contexts (`docs/src/key-bindings.md`).
- **Code we already ship.**
  - `agent_ui::threads_archive_view::fuzzy_match_positions` is the matcher, in a crate
    `marley_workbench` already builds. `fuzzy` and `fuzzy_nucleo` both depend on gpui, so
    neither can go in `marley_rail`, and neither is what Zed's sidebar uses.
  - `editor::Editor::single_line`; `ui::HighlightedLabel`, which takes UTF-8 byte positions and
    debug-panics on one off a char boundary (`highlighted_label.rs:17-32`);
    `ui::ThreadItem::highlight_positions`; `ui::KeyBinding::for_action_in`.
  - `zed_actions::agents_sidebar::FocusSidebarFilter` ("Moves focus to the sidebar's
    search/filter editor.").
  - gpui tries each matching binding in turn until one is handled
    (`crates/gpui/src/window.rs:5944-5957`). A single-line editor's `editor::MoveUp`,
    `editor::MoveDown` and `editor::Cancel` propagate, so `menu::SelectPrevious`,
    `menu::SelectNext` and `menu::Cancel` reach the rail, as Zed's pickers get them.
  - The gpui-era rail needed a filter-skip guard in each row type, and one was missed
    (`PR-claude-new-render-arm-mirror-sibling-guards-001`). Here one visibility function serves
    every row type.
- **The shadow sweep** covered every `ctrl-f` and `cmd-f` binding in `assets/keymaps/`:
  default-linux, -macos and -windows, vim, and the linux and macos base keymaps.
  - The rail's path is `Workspace > MarleyRail menu`, since the `MultiWorkspace` root carries
    the active workspace's context (`multi_workspace.rs:2072-2076`). No binding of either key
    matches there: no context-free, `menu` or `Workspace` block binds them.
  - Inside the field, `Editor && mode == full` keeps the single-line editor out of
    `buffer_search::Deploy`. On macOS, `ctrl-f` in an `Editor` is `MoveRight`, but the binding
    there is `cmd-f`. The Emacs base keymap's `ctrl-f` wins inside the field, which is already
    focused by then.
  - Deferred popovers keep the rail as their dispatch parent (`window.rs:4338`), so the binding
    reaches the rail's context menus and the add-project picker. `!Picker` keeps `ctrl-f` out
    of the picker.

## UI proof
UI-AFFECTING.
- **Driven tests.**
  - Typing into the filter narrows the rail: a name match shows every row, a child's match
    shows its header, a folded project's match shows, and a project with no match is hidden.
  - The matched characters reach the rows; a title with a multibyte character before the match
    draws without a panic; "No matches" draws; the chevrons hide.
  - Each edit moves the keyboard's row to the first match, and down and Enter from the field
    walk and open.
  - Escape clears the field, then moves focus to the rows; on the rows it clears the field.
    The clear button empties it.
  - With Zed's default keymap and the Marley keymap bound, `ctrl-f` from the rows focuses the
    field, and `ctrl-f` in the add-project picker leaves focus where it is.
  - Unit tests cover the pure rows.
- **Live drive:** focus the rail, press `ctrl-f`, type, walk, press Enter and clear, with a
  screenshot of each. It needs keys, so it runs only while Chad is away from the desk;
  otherwise the Test phase records why.

## Locked-In Decisions
- D1 — Zed's matcher and rules: `fuzzy_match_positions`, a substring match that ignores ASCII
  case, over project names and terminal and thread titles, during the rebuild. The rows keep
  the rail's order. The ticket's "fuzzy match" is corrected: Zed's filter is not fuzzy.
- D2 — The pure crate decides what shows, from match positions the gpui side puts into the
  snapshot, through one visibility function for every row type.
- D3 — The key is Zed's own action: the Marley keymap binds `secondary-f` to
  `agents_sidebar::FocusSidebarFilter` in `MarleyRail && !Picker`. In the field, Zed's defaults
  reach the rail's handlers, and nothing else is bound.
- D4 — The rail's focus key keeps landing on the rows (#453); the filter is one `ctrl-f` away.
- D5 — Each edit moves the keyboard's row to the first row that matched. The text stays until
  it is cleared and is not saved.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the filter holds text, the rail shall show each project whose name contains it, ignoring the case of ASCII letters, with every row under it; each other project with only its terminals and threads whose title contains it; and no other project | unit tests + driven tests |
| REQ-002 | WHILE the filter holds text, the rail shall show a folded project's matching rows, hide the fold chevrons, not fold on left or right, and highlight the matched characters of each name and title | unit tests + driven tests |
| REQ-003 | WHEN the filter hides every row, the rail shall show "No matches" | driven test |
| REQ-004 | WHEN the filter's text changes to a non-empty text, the rail shall move the keyboard's row to the first row that matched; WHILE the filter has focus, up and down shall move that row over the shown rows and Enter shall open it | unit tests + driven tests |
| REQ-005 | WHEN `ctrl-f` (`cmd-f` on macOS) is pressed while the rail holds focus, the filter shall take focus, except in the add-project picker | driven tests with Zed's default keymap and the Marley keymap |
| REQ-006 | WHEN Escape is pressed in the filter, the rail shall clear its text, or with no text move focus to the rows; WHEN Escape is pressed on the rows while the filter holds text, the rail shall clear it; otherwise Escape shall reach what it reached before | driven tests |
| REQ-007 | WHEN the filter's clear button is clicked, the rail shall empty the filter and show every row again | driven test |
| REQ-008 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the rows in `marley_rail`; the field, the matching, the keys and the handlers in
  `rail.rs`; the keymap block; fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run the tests; the live drive; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the
  ticket, archive, commit.
