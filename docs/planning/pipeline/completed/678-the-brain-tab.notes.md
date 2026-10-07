# The Brain tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-678-the-brain-tab.md
- **Pipeline spec:** 678-the-brain-tab.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-07 (quoted in the spec).
- **Classification / tier:** feature; `marley_workbench` only.
- **Pre-flight:** green; #677 committed and installed; cargo idle.
- **Recall (§18.3):** #644 (the Brain view, its keys in `MarleyBrain menu`, AD-453); #645
  (`PageView`, its history and `navigate`); #658 (the Tasks tab's two columns, `LISTS_WIDTH`);
  #675/#676 (`in_group`); PR-claude-defer-in-does-not-leave-the-entitys-own-update.
- **Discovery:** `BrainView::new`, `open`, `today`, `open_page`, the page menu; `toggle_in_window`
  (the window handler of `ToggleBrainView`); `PageView::new`, `navigate`; the followers that
  downcast the active item to `PageView` (`knowledge_panel.rs:344`, `graph_tab.rs:135` and `:580`,
  `page_picker.rs:94`); the rail's `BrainSide`, `RailView`, `show_view`, `toggle_brain_view`,
  `brain_refusal`, `shown_brain`, `render_view_switch`, `render_new_page`, the body swap, the
  focus hand-off.

### Design
- **`rusty/brain_tab.rs`** (new): `BrainTab { workspace, navigation: Entity<BrainView>, page:
  Option<Entity<PageView>>, focus_handle, _page_events }`; `open_later` (through `in_rusty_group`,
  refusing with a toast while Rusty is unavailable), `open_today_later`; `show_page(slug, focus)`
  makes the page view the first time and `navigate`s it after; `Item` (title Brain, icon BookCopy,
  `act_as_type` hands out the page, `UpdateTab` when the page changes); the two columns.
- **`rusty/brain.rs`**: `BrainView::new` takes its host; `open` defers to the host's `show_page`;
  the page menu gains Open in New Tab (`open_page`); `Screen::Brain` first, `open_screen` opens the
  tab and Today through it; `ToggleBrainView` opens the tab; `toggle_in_window` goes.
- **`rusty/page.rs`**: `PageView::new` and `navigate` reachable from the tab; `page_in(item, cx)`,
  the page an item shows (a Page tab or the Brain tab's).
- **Followers**: `knowledge_panel`, `graph_tab` (two places), `page_picker` read `page_in`.
- **`rail.rs`**: `BrainSide` keeps only `connected`; the Brain view, `RailView`, the switch, New
  Page, the body swap, the focus hand-off and `brain_refusal` go; `screens_that_fit` counts the
  screens alone.
- **File manifest:** those, the guide, the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001 | Rusty's stand-in on, a project | `678-01-header` |
| REQ-002 | clicks Brain | `678-02-tab` |
| REQ-003 | opens the folder, clicks the page | `678-03-page` |
| REQ-004 | right-clicks the page, Open in New Tab | `678-04-new-tab` |

### Risks
- Rail tests in the tree that name the Brain view must keep building.
- A key bound in the rail for the Brain view (`secondary-f` to its search) loses its target; it
  goes with the view, or moves to the tab.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall; discovery.
- [x] Mint the pair; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's decision.

## Phase 2 — Code (2026-10-07)
### Built
- **`rusty/brain_tab.rs`** (new): `BrainTab`, `BrainTabEvent`, `open_later`, `open_today_later`,
  `show_page`, `today`; the two columns; `Item` with `act_as_type` for the page.
- **`rusty/brain.rs`**: `BrainView::host`; `open` defers to the host; `open_in_new_tab` and the menu
  entry; New Page after the search field (it lost its header button with the rail's view);
  `Screen::Brain`; `open_screen` without the Brain view; `ToggleBrainView` opens the tab;
  `toggle_in_window`, `search_focus` and `preview_on_click` went with the rail's view.
- **`rusty/page.rs`**: `PageView::new` and `navigate` reachable from the tab; `page_in`.
- **Followers**: `knowledge_panel`, `graph_tab` (twice), `page_picker` read `page_in`.
- **`rail.rs`**: the Brain view, `RailView`, the switch, New Page, the body swap, the focus
  hand-off and `brain_refusal` gone; `render_screens`; `screens_that_fit` without the switch.
- **`groups.rs`**: a Rusty group keeps the name Rusty (below).
- **`marley_workbench.rs`**: `ToggleBrainView`'s doc.
- **The guide**: the Brain article is the Brain tab's; the screens' items name the header.

### Deviations
- **A bug of #675's, fixed here.** The exploratory run's Rusty group read "Rusty 2": the run's copy
  of the profile holds the user's own Rusty group's record, and `make` gave every new group a name
  free across every window's groups. One Rusty group per window should be Rusty in each; `make`
  now keeps it.
- **A cut in `rail.rs` took five rendering functions with the Brain view's**; they were put back
  from HEAD before the first build, unchanged, and the diff removes only the Brain view's.

### Review
- `BrainView::open` defers to the tab, since a tree click runs inside the view's update and
  `show_page` updates the page view.
- The Knowledge panel's following of the page relies on the Brain tab's `UpdateTab` reaching the
  pane as a title change, which becomes `ActiveItemChanged` for the active pane.

### Gate
`just gate-diff`: red once (gate:18, the spelling gate, on a verbatim quote of Chad's in `rusty-in-marley.md`, now paraphrased there), then GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/678-the-brain-tab.sh` (`compositor sway`), on the debug build, Rusty's stand-in over a
scratch vault with `notes/a-note`.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001 | `678-01-header` | the header: Brain (first), Today, Graph, Tasks, Decisions, Memory, `…`, the folder +; no Projects/Brain switch; the rail lists `repo` and its terminal |
| REQ-002 | `678-02-tab` | Brain clicked: a `Rusty` group with `Brain` its row, the Brain tab in front: "Search the brain…" with its +, `notes 1` on the left, "Pick a page on the left." on the right |
| REQ-003 | `678-03-page` | `notes` opened, `a-note` clicked and selected: the page on the right with back, forward, outline, `notes / a-note`, the star and Edit, "A note" and its body |
| REQ-004 | `678-04-new-tab` | `a-note`'s menu, Open in New Tab: an `a-note` tab beside Brain, its own row under Rusty |

The exploratory run showed the group as "Rusty 2" (F-claude-678-…); after the fix every shot reads
Rusty. Focus: headless sway; Hyprland's one Marley window (Chad's) before and after, no rule added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: the Brain tab); the in-app guide's Brain article and the
  screens' items (before the gate); `marley_workbench.md` (a new section); `rusty-in-marley.md`
  (R-D9's note and R4).
- **Knowledge:** `F-claude-678-a-second-windows-rusty-group-was-called-rusty-2-001`,
  `L-claude-678-a-scripted-cut-between-two-markers-takes-everything-between-001`,
  `AD-claude-678-the-vault-has-its-navigation-in-a-brain-tab-001`.
- **Brain:** `decisions/the-vaults-navigation-moves-into-a-brain-tab-and-out-of-the-rail`.
- **Ticket:** closed; the pair archived.
