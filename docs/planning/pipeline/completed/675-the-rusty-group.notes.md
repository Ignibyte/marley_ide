# The Rusty group — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-675-the-rusty-group.md
- **Pipeline spec:** 675-the-rusty-group.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-06 and 2026-10-07 (quoted in the spec).
- **Classification / tier:** feature; `marley_workbench` only.
- **Pre-flight:** green; #674 committed; cargo idle.
- **Recall (§18.3):**
  - #600/#601: `groups::make` makes a folderless workspace and runs `then` through the window;
    the Home group is found by its flag (`in_home`); records are kept in Zed's key-value store.
  - #661: Rusty off leaves no trace; the rail's group list must leave the Rusty group out.
  - PR-claude-defer-in-does-not-leave-the-entitys-own-update: the openers already defer.
- **Discovery:** `groups.rs` (`Group`, `SavedGroup`, `save`, `adopt`, `make`, `rename`);
  `rail.rs` (`in_home`, `header_icon`, `HeaderMenu::build`, `group_context_menu`, `rail_groups`);
  every Rusty opener (`graph_tab`, `tasks_tab`, `decisions_tab`, `memory_tab`, `skills_tab`,
  `secrets_tab` `open_later` and their palette actions; `page::open_later`; `capture::open_today`).

### Design
- **`groups.rs`**: `GroupKind { Named, Home, Rusty }` for `make`; `Group::rusty` and
  `SavedGroup::rusty` (serde default); `rusty_group(multi_workspace, cx)`; `is_rusty(id, cx)`;
  `RUSTY` names it.
- **`rusty.rs`**: `in_rusty_group(window, cx, open)`: deferred; in the Marley layout it finds or
  makes the window's Rusty group, shows it (`MultiWorkspace::activate`) and runs `open` in its
  workspace; in the Zed layout it runs `open` in the shown workspace.
- **The openers**: each tab's `open_later` and `page::open_later` go through `in_rusty_group`
  (they keep their workspace argument for the Zed layout's fallback); each palette action calls
  its `open_later`.
- **`rail.rs`**: `in_home` passes `GroupKind::Home`; the New Group prompt `Named`; `header_icon`
  takes the group's kind and draws `RUSTY_GROUP_ICON` (a placeholder) for Rusty; the Rusty group's
  menu has one disabled line in place of Rename and Remove; `rail_groups` leaves it out while
  Rusty is off.
- **File manifest:** `groups.rs`, `rusty.rs`, `rusty/{graph,tasks,decisions,memory,skills,secrets}_tab.rs`,
  `rusty/page.rs`, `rusty/capture.rs`, `rail.rs`, the guide, the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| — | a project with its terminal, Rusty on | `675-01-before` |
| REQ-001, 002 | clicks the header's Graph | `675-02-graph` |
| REQ-003 | clicks the project's terminal row | `675-03-project` |
| REQ-001 | opens a page from the Brain view's tree | `675-04-page` |
| REQ-004 | right-clicks the Rusty group's header | `675-05-menu` |
| REQ-005 | turns Rusty off | `675-06-off` |

### Risks
- A Rusty tab already open in a project from before stays there; new opens go to the group.
- `make` is asynchronous: two opens before the group exists could make two groups. The second
  `in_rusty_group` finds none yet; a pending flag on the window's groups guards it.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall; discovery.
- [x] Mint the pair; the ticket in progress; the backlog row removed.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's decisions.

## Phase 2 — Code (2026-10-07)
### Built
- **`groups.rs`**: `GroupKind`, `RUSTY`, `Group::rusty`, `SavedGroup::rusty`, `rusty_group`,
  `is_rusty`, `with_rusty_group` with `Groups::waiting_for_rusty`; `make` takes a kind.
- **`rusty.rs`**: `in_rusty_group`.
- **The openers**: `graph_tab` (both actions), `tasks_tab`, `decisions_tab`, `memory_tab`,
  `skills_tab`, `secrets_tab` and `page` (`open_later`, `open_at_heading_later`) route through it.
- **`rail.rs`**: `in_home` makes `GroupKind::Home`; `header_icon` takes `rusty`;
  `RUSTY_GROUP_ICON`; the Rusty group's menu; `rail_groups` leaves it out while Rusty is off;
  `leave_rusty_group` from `rusty_changed`.
- **The guide**: the Brain view article.

### Deviations
- **A window showing the group when Rusty turns off goes back to a project.** The exploratory
  run turned Rusty off with the group in front: the rail dropped it, and the window kept showing
  its tabs with nothing in the rail highlighted. `leave_rusty_group` fixes it; REQ-005 says so.
- **The page in the check comes from `rusty: open page`**, which shares `page::open_later` with the
  tree, so the scenario needs no tree coordinates.

### Review
- `in_rusty_group` and `leave_rusty_group` defer before they update the `MultiWorkspace`; `make`'s
  `then` runs through the window alone (#600).
- The palette actions now open through `open_later` from the workspace they were dispatched in,
  which is the Zed layout's fallback.
- A Rusty tab already in a project from before stays there; only new opens move.

### Gate
`just gate-diff` on the tree with the scenario: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/675-the-rusty-group.sh` (`compositor sway`), on the debug build, Rusty's stand-in over a
scratch vault with one page.

| Criterion | Shot | What it shows |
|---|---|---|
| — | `675-01-before` | the project `repo` and its terminal; no Rusty group |
| REQ-001, 002 | `675-02-graph` | after the header's Graph: a `Rusty` group with the placeholder icon after `repo`, `Graph` its row, highlighted, the Graph tab in front; the stand-in logged `brain_graph` |
| REQ-003 | `675-03-project` | the terminal row clicked: the terminal in front, Graph still under Rusty |
| REQ-001 | `675-04-page` | `rusty: open page` for `a note` with the project in front: `a-note` under Rusty beside Graph, the window showing the group with the page |
| REQ-004 | `675-05-menu` | the Rusty header right-clicked: one line, "Rusty's screens and pages open here", no Rename, no Remove |
| REQ-005 | `675-06-off` | Rusty turned off: no Rusty group, the window back on `repo` with its terminal highlighted |

`675-06-off` also shows the toast of the shortcut note that the earlier `Ctrl-Alt-U` in the
palette left: pre-existing, not in scope. Focus: headless sway; Hyprland's one Marley window
(Chad's) before and after, no rule added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: Rusty's screens open in a Rusty group); the in-app
  guide's Brain view article (before the gate); `marley_workbench.md` (a new section);
  `rusty-in-marley.md` (R4). No Zed path touched.
- **Knowledge:** `L-claude-675-route-a-feature-s-tabs-in-its-openers-not-its-callers-001`,
  `AD-claude-675-rustys-screens-open-in-a-per-window-rusty-group-001`.
- **Brain:** covered by the morning's decision (Rusty group); no new decision.
- **Ticket:** closed; the pair archived.
