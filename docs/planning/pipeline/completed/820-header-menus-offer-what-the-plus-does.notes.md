# A header's right-click offers what its + does — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-820-header-menus-offer-what-the-plus-does.md
- **Pipeline spec:** 820-header-menus-offer-what-the-plus-does.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19 (item 2 of two): "Right clicking on the left panel some of the
  features the + has is missing so lets rectify that". The plain sentence given him: every
  header's right-click starts with the same entries as its `+`, and the empty space's menu offers
  New Agent… and Reopen Thread… in place of the old agent list.
- **Classification / tier:** bug; one Marley crate (`marley_workbench`, `rail.rs`).
- **Recall (§18.3):** #744 made New Agent… the one way to start an agent from the `+`; the empty
  space's menu (#600) kept the agent CLIs list from before it. The brain (consultation
  bef397c42ef24edc87712a776cdba077): nothing on this seam.
- **Discovery:** `render_project_menu` builds the `+` inline for every open header (projects,
  groups, Default); `HeaderMenu::build` picks `group_context_menu` (Default: a label) or
  `project_context_menu`; `deploy_empty_menu` runs its entries `in_home`.
- **Decisions:** D1, D2 in the spec.

### Design
- `fn new_entries(menu, rail, workspace, search_path, cx) -> ContextMenu`: the `+` body moved out
  of `render_project_menu`, which calls it.
- `project_context_menu`: when not `closed`, `new_entries(..).separator()` first;
  `group_context_menu`: the same first, then Rename/Remove, or for Default its label; both read
  `agents::launcher(cx).search_path` when the menu builds.
- `deploy_empty_menu`: New Agent… and Reopen Thread… run `in_home`, in place of the Agent CLIs list.
- File manifest: `crates/marley_workbench/src/rail.rs` (Marley);
  `script/e2e/820-header-menus-offer-what-the-plus-does.sh` (new).

### Visual check plan
- As the spec's UI proof, under sway: `repo` open (a git repository, so New Agent in Worktree is
  offered); right-clicks on `repo`'s header, Default's header and the empty space, each shot with
  the menu open and then Escape; New Terminal from `repo`'s right-click, the rail's new row.

### Risks
- None beyond the menus' length: the project menu grows by six entries.

## Phase 2 — Code
- Built to the manifest in `rail.rs`: `new_entries` holds the `+`'s body, moved out of
  `render_project_menu`, which calls it; `project_context_menu` (unless `closed`) and
  `group_context_menu` (Default included, which keeps its line after a separator) start with it,
  reading `agents::launcher(cx).search_path` at build; `deploy_empty_menu` offers New Agent… and
  Reopen Thread… in Default (`in_home`) in place of its Agent CLIs list.
- Deviation: `Rail::new_agent` (start a CLI in a terminal), whose last caller was that list, is
  removed; New Agent… starts CLIs.
- The first gate run failed clippy (a clone and a closure clippy asked to drop in the empty
  space's Reopen Thread…); fixed, and the second run is green, 17 of 17.
- A measuring run before the gate confirmed the points: `repo`'s header at (60, 136), Default's
  at (60, 88), the empty space at (60, 700), New Terminal in `repo`'s menu at (110, 158).

## Phase 3 — Test
- `just e2e script/e2e/820-header-menus-offer-what-the-plus-does.sh` on the debug build after the
  green gate, under sway; focus report "1 Marley windows before the run, 1 after". Shots read:
  - `820-01-project-menu`: `repo`'s right-click: New Terminal, New Browser Tab, New Agent…,
    Reopen Thread…, New Agent in Worktree ›, then Move Project Up, Move Project Down (disabled,
    the last project), No Archived Threads, Clear Browser Data…, Remove Project (REQ-001).
  - `820-02-default-menu`: Default's right-click: New Terminal, New Browser Tab, New Agent…,
    Reopen Thread…, then "What belongs to no project opens here" (REQ-002; Default has no git
    repository, so no worktree entry, as its `+` has none).
  - `820-03-empty-menu`: the empty space's: New Group…, New Terminal, New Browser Tab, New
    Agent…, Reopen Thread…; no Agent CLIs header (REQ-003).
  - `820-04-terminal-from-menu`: a second `repo — bash` row under `repo` and its tab in front,
    from New Terminal in `repo`'s right-click (REQ-004).

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #820); `docs/marley_architecture/marley_workbench.md`, the header
  menu's bullet. No Zed path; no slice status moves.
- Ledger: no append; the drift (#744 changed the `+` and not the menus) is the reason for one
  builder, D1, and nothing in it is likely to recur once both use it.
- Brain: consultation `bef397c42ef24edc87712a776cdba077` closed with `brain no-decision`.
- Ticket closed; it had no `BACKLOG.md` row (filed at Plan).
