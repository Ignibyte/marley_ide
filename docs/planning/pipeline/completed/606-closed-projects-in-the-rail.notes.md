# List a window's closed projects in the rail — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-606-closed-projects-in-the-rail.md
- **Pipeline spec:** 606-closed-projects-in-the-rail.spec.md

## Phase 1 — Plan (Opus, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets go ahead and make that ticket and build it as well", on
  the follow-up proposed at the end of the #602 to #605 batch (from #601's Test finding): list the
  projects a restart leaves closed, dimmed, and open one on a click. It runs autonomously.
- **Classification:** feature, medium; `marley_rail` (a flag) and the workbench's `rail.rs`, and
  one new dependency of the workbench on the tree's `remote_connection` crate. No Zed file
  changes.
- **Pre-flight:** no active pipeline; README marker present. Cargo was busy with Chad's
  `just install` (a release build); Phase 2 waits for it (one cargo at a time).
- **Recall (§18.3):**
  - #601's notes: Zed reopens only a window's shown workspace and keeps its other project groups
    as keys; the rail has left groups with no workspace out since before #600.
  - L-claude-455: opening a folder again through `new_local` finds its saved row and restores
    its items, so an opened closed project brings its terminals back. AD-claude-575 keeps their
    ids.
  - L-claude-601: a scenario that checks a restart starts Marley with no path (`open_path ""`).
  - Brain (consultation 3fe8d384): nothing on this seam.
- **Discovery:** `rail_groups` filters `!group.workspaces.is_empty()`, and `build_snapshot`
  skips a group whose `listed_workspace` is `None`. Zed's Threads Sidebar opens such a group with
  `find_or_create_workspace(path_list, host, Some(key), connect_remote, None,
  OpenMode::Activate, None, …)` and dismisses the remote modal afterwards
  (`sidebar.rs:1295-1330`). `remove_project_group` handles a key with no workspace. `WeakEntity::new_invalid` exists.

### Design
- **Approach.**
  - *`marley_rail`:*
    - `ProjectSnapshot::closed` (in Code, `ProjectRow` did not take it; see Phase 2).
    - A closed project has no rows.
    - `cycle_project` passes over closed headers, so Next and Previous Project do not open every
      closed project on the way.
  - *`rail.rs`, the snapshot:*
    - `rail_groups` keeps groups with no workspace.
    - `build_snapshot` gives a group with no listed workspace a header-only `ProjectSnapshot`
      (`closed: true`, name matched by the filter) and a `GroupEntry` with `closed: true`,
      `workspace: WeakEntity::new_invalid()`, and no git or group ids.
    - `note_focus` never picks a closed group, since it holds no workspace, and the indices stay
      aligned because every group gets its entry.
  - *`rail.rs`, the header:*
    - `render_project_row` for a closed row: no disclosure, no `+`; the name in
      `Color::Disabled`; the icon at half opacity; a tooltip ("Not open. Click to open it.").
      A click runs `open_closed_project(key)` rather than `activate_workspace`.
    - `follow_icons` also searches a closed local group's first folder, so its dimmed header
      keeps its icon.
  - *`open_closed_project`:* mirrors the sidebar, with `find_or_create_workspace` and
    `remote_connection::connect_with_modal` for a remote group. The task is awaited in a
    `spawn_in`, which dismisses the connection modal and shows a failure as a toast.
  - *Enter:* `open_row`'s `Selection::Project` arm opens a closed group the same way.
  - *The menu:* `HeaderMenu` carries `closed`. `project_context_menu` disables Clear Browser
    Data… for a closed header; Move Up and Down and Remove Project work on the key as for any
    project.
- **File manifest:**
  - `crates/marley_rail/src/marley_rail.rs` (Marley);
  - `crates/marley_workbench/src/rail.rs` (Marley);
  - `crates/marley_workbench/Cargo.toml` (Marley: `remote_connection.workspace = true`);
  - `script/e2e/606-closed-projects-in-the-rail.sh` (Test).
  - No Zed path.
- **Visual check plan** (sway):
  - Three projects: `repo` (opened), then `repo-b` and `repo-c` handed over (#513). `repo-b`'s
    terminal goes to `sub`. Show `repo`, quit, start with no path.
  - `restart.png`: `repo` open; `repo-b` and `repo-c` dimmed, header only (REQ-001).
  - `tooltip.png`: a dimmed header's tooltip (REQ-002).
  - Click `repo-b`: `opened.png`, `repo-b` open with its terminal in `sub` (REQ-003).
  - Right-click `repo-c`: `menu.png`, then Remove Project: `removed.png` (REQ-005).
  - REQ-004 (Enter) is the same `open_row` path: a review.
- **Risks:**
  - A closed group whose folders are gone: `find_or_create_local_workspace` opens an empty
    project there, as Zed's sidebar does; any error shows as a toast.
  - `render_project_row` is near `too_many_lines`: a helper for the closed header's pieces if
    clippy asks.

## Phase 2 — Code (2026-09-30)
- **Built:**
  - `marley_rail`: `ProjectSnapshot::closed`, and `cycle_project` passes over closed headers.
  - `rail.rs`, the snapshot:
    - `rail_groups` keeps groups with no workspace;
    - `build_snapshot` gives such a group `push_closed`, a header-only snapshot and a
      `GroupEntry { closed: true, workspace: WeakEntity::new_invalid() }`.
  - `rail.rs`, the header:
    - `render_project_row` for a closed group: a `Color::Disabled` name, the icon at half
      opacity, no chevron or `+`, the tooltip "Not open. Click to open it.";
    - element ids from `closed_id` (a hash of the key), since every invalid handle shares one
      entity id and two closed headers would share their menus' ids;
    - a click runs `open_closed_project`.
    - The header's right-hand end moved into `render_header_end`.
  - `rail.rs`, opening and the menu:
    - `open_closed_project` calls `find_or_create_workspace(…, OpenMode::Activate, …)`, with
      `remote_connection::connect_with_modal` for a remote group; afterwards it dismisses the
      modal and shows a failure as a toast.
    - `open_row` opens a closed header on Enter.
    - `HeaderMenu` and `project_context_menu` carry `closed`, which disables Clear Browser Data….
    - `follow_icons` searches a closed local group's folder too.
  - `Cargo.toml`: `remote_connection.workspace = true`.
- **Deviations:**
  - The flag lives on `ProjectSnapshot` and `GroupEntry` only. On `ProjectRow` it was a fourth
    bool, which `struct_excessive_bools` refused, and the header already has its `GroupEntry`.
  - `render_header_end` was extracted, and `build_snapshot`'s `listed` iterator folded into its
    `for`, for `too_many_lines`.
- **Review of the diff:**
  - REQ-001: header only, dimmed, no chevron or `+`.
  - REQ-002: the tooltip.
  - REQ-003: Zed's own `find_or_create_workspace`, which restores the saved workspace for the
    folders (L-claude-455).
  - REQ-004: `open_row`'s closed arm.
  - REQ-005: Remove Project works on the key.
  - Indices stay aligned: every group now gets exactly one snapshot and one entry, where before
    `listed` could skip one.
  - Re-entrancy: `find_or_create_workspace` runs inside the `MultiWorkspace`'s update from the
    rail's listener, as `activate_workspace` already does, and the task is awaited outside it.
  - **Found and fixed in review:** closed headers would have shared element ids (see Built).
- **Process note:** Chad's `just install` release build reached `marley_workbench` after my first
  edits, so the installed build may hold part of this change. `just install` runs again after the
  commit.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/606-closed-projects-in-the-rail.sh` (sway).
  - `repo`, then `repo-b` and `repo-c` handed over (#513), each trusted; `repo-b`'s terminal
    goes to `sub`. `repo` is shown, then a quit and a start with no path.
  - Run 1 found a red and missed one click (both below). Run 2 followed the gate's rerun.
- **Run 1:**
  - `restart.png` showed the closed headers without a chevron, so their names sat 26 px left of
    an open project's. Fix at the source: a closed header keeps the chevron's room with an
    invisible, disabled `Disclosure`.
  - `menu.png` placed Remove Project at y 195; the scenario's guess (180) missed it.
  - The gate ran again after the fix: 17 PASS, GATE GREEN.
- **Run 2, every shot read:**
  - `three.png`: `repo-c`, `repo-b` (its terminal "sub — bash" in `sub`), `repo`.
  - `before.png`: `repo` shown before the quit.
  - `restart.png` (REQ-001): `repo-c` and `repo-b` dimmed, header only, no chevron or `+`, their
    names lined up with `repo`'s; `repo` open with its terminal.
  - `tooltip.png` (REQ-002): the pointer on `repo-b` shows "Not open. Click to open it."
  - `opened.png` (REQ-003): after the click, `repo-b` is open (chevron, `+`), its terminal back as
    "sub — bash" in `sub`, the window showing `repo-b`; `repo-c` is still dimmed.
  - `menu.png` (REQ-005): `repo-c`'s menu: Move Project Up (disabled at the top), Move Project
    Down, Clear Browser Data… (disabled: closed), Remove Project.
  - `removed.png` (REQ-005): `repo-c` gone; `repo-b` and `repo` stay.
- **Not reachable by a shot:** REQ-004 (Enter) runs `open_row`'s closed arm, the same call as the
  click; covered by the review.
- **Focus:** its own headless sway; Hyprland had 0 Marley windows before and after.
- **Noted, minor:** a header tooltip showing when the header is right-clicked stays over the
  menu's first entry until the pointer moves; moving onto the menu clears it (`menu.png`).

## Phase 4 — Complete (2026-09-30)
- **Documented:**
  - `CHANGELOG.md`.
  - `docs/marley_architecture/marley_workbench.md` (the rail lists every group; Closed projects)
    and `marley_rail.md` (Closed projects).
  - `docs/marley/workbench-shell.md` (the slice line), `docs/marley/guide.md` (after a restart)
    and `docs/marley/walkthrough.md` (2.2).
  - The guide page's `restore` article.
  - `build_snapshot`'s doc comment, which still said only open groups were listed.
  - No Zed path touched.
- **Knowledge:**
  - F-claude-606-closed-headers-would-have-shared-their-element-ids-001;
  - F-claude-606-a-closed-headers-name-sat-left-of-the-others-001;
  - L-claude-606-just-install-reads-sources-as-it-reaches-each-crate-001;
  - AD-claude-606-closed-projects-are-listed-and-opened-on-a-click-001.
  - Brain: consultation 3fe8d384 closed as
    `decisions/marleys-rail-lists-a-windows-closed-projects-dimmed-and-opens-one-on-a-click`.
