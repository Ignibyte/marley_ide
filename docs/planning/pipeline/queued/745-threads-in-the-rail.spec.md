---
pipeline_id: 1a30863b-ae8f-4d86-a091-4130d723bb23
ticket: docs/planning/tickets/open/TICKET-745-threads-in-the-rail.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Threads in the rail
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/queued/744-one-new-agent.spec.md
  - docs/planning/pipeline/completed/737-the-threads-page.spec.md
  - docs/planning/pipeline/completed/739-rustal-icons-in-the-status-bar.spec.md
  - docs/planning/pipeline/completed/644-brain-view-in-the-rail.spec.md
  - docs/planning/pipeline/completed/678-the-brain-tab.spec.md
---

## Title
A Projects | Threads switch at the top of the rail. Threads lists every conversation in the rail,
newest first; a click opens it in a tab or brings its tab forward. The Home page gets a Threads
entry, and the status bar's Threads button leaves. Chad, 2026-10-11: "we need to have a button that
shows all agent threads or something again … add a button in the side panel that toggles between
the two? with an option on the home page as well", then "moving the threads back over to the top
left". Zed's own sidebar flips between its thread list and its thread history the same way; the
rail replaced that sidebar and kept only the projects.

## Scope
### In
- **The switch.** The rail's header reads PROJECTS and THREADS where it reads PROJECTS today; the
  shown view's name is pressed. A click on the other one flips the view. `ctrl-g` in the rail flips
  it too, as Zed's sidebar binds `agents_sidebar::ToggleThreadHistory` in its own context.
- **The Threads view**, drawn under the header in place of the filter row and the project rows:
  - a search field where the filter sits, matching the title or any folder, as #737's page does;
  - one list of the store's conversations, drafts and archived ones left out, newest first by
    `updated_at`; each row draws the agent's icon, the title, and "agent · folder" (home as `~`),
    in the rail's own row card;
  - a click opens the conversation in a tab of the shown group, or brings its tab forward where it
    is open (#737's `open_thread`, which #744 keeps out of the panel).
- The list follows the store: a new, renamed or archived conversation shows or leaves without a
  switch.
- While Threads shows, the header's Add Project button and its empty-space menu are hidden, the
  rail's row keys do nothing, and PROJECTS carries the rail's attention dot when a project needs
  you.
- The rail starts on Projects.
- **The Home page** (`home_page.rs`, #701): a Threads button in its NEW AGENT card, after New
  Agent…, opening the Threads page (#737).
- **The status bar** (`status_buttons.rs`, #739): the Threads button leaves. `marley: open threads`
  stays in the palette.
- The guide (`docs/marley/guide.md`, the in-app page) and the walkthrough: the switch, the Home
  entry, the status bar's buttons.

### Out (explicitly deferred)
- Old CLI sessions in the list: #754 adds them to the Threads page and this list together.
- Agent chips, the Marley and Rusty section and archived conversations in the rail: the page has
  them.
- Up, Down and Enter over the Threads list, live status marks on its rows, and the row of the
  conversation in front marked: the project rows keep those.
- Remembering the view across a restart.
- The Zed layout: Zed's sidebar already has its own history switch.

## Reference (§20)
Upstream Zed, the `sidebar` crate: the bottom bar's history button (`IconName::Clock`,
`ToggleThreadHistory`, `ctrl-g` in `ThreadsSidebar`) flips the sidebar's column between its thread
list and `ThreadsArchiveView`, the store's threads searched by title then folder
(`sidebar.rs` `render_sidebar_bottom_bar` 7521, `toggle_archive` 7697). Behavior kept: one switch,
the same column, Zed's action and key, Zed's thread store, newest first. Marley's differences: the
switch sits in the rail's header ("back over to the top left"), the list is drawn with the rail's
row card, and a click opens a tab (#734).

### Prior art
- **The code we ship:**
  - `ThreadsArchiveView` (`agent_ui/src/threads_archive_view.rs:163`) is public and emits
    `Activate { thread }`, so the rail could host Zed's history whole. Not taken: it draws its own
    header at the title bar's height with window controls (`render_header` 860), meant to replace a
    sidebar's whole column, buckets rows by day, lists archived threads with Zed's worktree
    restore, and offers Import and New Thread entries the rail would have to answer. #737's D1
    turned it down for the page for the same reasons.
  - `threads_page.rs` (#737): `Row` (83), `sections` (126), `matches` (283), `shown_folder`,
    `open_thread` (310). The rail reads the same rows through one shared function.
  - `rail.rs` (at 19dda499a2; #743 moves the lines): `render_header` (4390, the PROJECTS label and
    `render_add_project` 4743), `render_filter` (4695), `render_rows` (4663), `render_inbox`
    (2026), `impl Render for Rail` (9420), `row_card` (9176), the row keys `select_next` (3057) to
    `select_child` (3083), `confirm` (3148), `focus_filter` (3112), and the store subscription that
    already refreshes the rail (`thread_store_subscription`).
  - `marley_rail::has_attention` (`marley_rail/src/marley_rail.rs:1791`), the dot #644 drew on
    Projects while another view showed.
  - #644's D2 and D3 (`644-brain-view-in-the-rail.spec.md`): a second view is an entity the rail
    draws under its header in place of its filter, inbox and rows, not rows in `marley_rail`'s
    model; the header's end button follows the view.
  - The Marley keymap's `MarleyRail && !Picker` block (`crates/marley_workbench/keymap.json:263`)
    already binds Zed's `agents_sidebar` actions for the rail.
  - `home_page.rs` `render_agents` (258), `status_buttons.rs` (the Threads button, 97-113).
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` §1.7 (the
  `MultiWorkspace` sidebar the rail fills, Zed's `sidebar` crate with its thread switcher); no map
  covers the history view.
- **Published material:** none.

## UI proof
`script/e2e/745-threads-in-the-rail.sh`, under `compositor sway`. #737's set-up: the scripted agent
runs the Marley entry and a custom server, Scripted; three keys bound to `marley::NewAgentThread`
start Marley in the repo, Scripted on `other/` and Scripted on `second/`, each asked one line; the
copy's thread list emptied first.

Shots:
- `745-01-header`: the rail's header with PROJECTS pressed and THREADS; the status bar without the
  Threads button.
- `745-02-threads`: THREADS clicked: three rows, `second` first, each with its agent and folder; no
  project rows; THREADS pressed.
- `745-03-search`: `other` typed in the search field: only the `other/` conversation.
- `745-04-open`: the search cleared, the `second/` tab closed, its row clicked: a tab in the shown
  group with its turn; `session/load` logged.
- `745-05-forward`: the Marley row clicked while its tab is open: that tab in front, one Marley tab.
- `745-06-key`: `ctrl-g` with the focus in the rail: the project rows again.
- `745-07-home`: Home clicked in the status bar, then its Threads button: the Threads page in a tab.

## Locked-In Decisions
- **D1 — A switch in the header, not a third place.** Chad: "a button in the side panel that
  toggles between the two", "back over to the top left". Two label buttons, PROJECTS and THREADS,
  where the PROJECTS label is, the shown one pressed. Rejected: icon buttons (#644's; the label
  says which view shows without a tooltip), Zed's `ToggleButtonGroup` (too wide beside the window
  controls, #644 D3), Zed's history button at the bottom (Chad asked for the top).
- **D2 — Why this switch stands where #678 removed one.** AD-claude-678 took the Projects | Brain
  switch out because the Brain tab already held the vault's tree and navigated its pages inside
  itself: the rail's copy was a second home for one tree. Conversations have no such tab. Each
  opens as its own tab, as a project's rows open theirs, and the rail is where Zed's layout keeps
  them (its sidebar's history) and where the Marley layout already lists each project's threads.
  The Threads page stays the place to filter by agent and to find archived ones; the rail is the
  quick list. The switch also needs no connection, so the header keeps its shape whatever is on,
  where #644's appeared only while Rusty was reached. Chad asked for both ways in.
- **D3 — Marley's own list over `ThreadsArchiveView`** (see Prior art), drawn with the rail's row
  card so it reads like the project rows beside it.
- **D4 — One source.** The rail and the page read the same rows from `threads_page.rs`; #754 adds
  CLI sessions there once.
- **D5 — Zed's action and key.** The switch answers `agents_sidebar::ToggleThreadHistory`, bound to
  `ctrl-g` in the Marley keymap's rail block, as Zed binds it in its sidebar and as the rail
  already takes Zed's thread switcher (AD-claude-454).
- **D6 — The rail starts on Projects.** Rejected: remembering the view, as Zed's sidebar does
  (`SerializedSidebarView::History`): a restart that opens on Threads hides the projects the
  window restored.
- **D7 — The status bar's Threads button leaves** (Chad: threads go "back over to the top left");
  the Home page's entry and the palette open the page.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The rail's header shall show PROJECTS and THREADS, with the shown view's pressed. | Shots 745-01, 745-02 |
| REQ-002 | WHEN THREADS is chosen, the rail shall list the saved conversations, archived ones and drafts left out, newest first, each with its agent's icon, title, agent and folder, in place of the project rows. | Shot 745-02 |
| REQ-003 | WHEN text is typed in the Threads view's search field, the rail shall list only conversations whose title or folder holds it. | Shot 745-03 |
| REQ-004 | WHEN a conversation with no open tab is clicked, the system shall open it in a tab of the shown group with its earlier turns. | Shot 745-04 and `session/load` in the log |
| REQ-005 | WHEN a conversation whose tab is open is clicked, the system shall bring that tab forward and open no second one. | Shot 745-05 |
| REQ-006 | WHEN `agents_sidebar::ToggleThreadHistory` runs in the rail, or the other view's name is clicked, the rail shall flip between Projects and Threads. | Shot 745-06; shot 745-02 for the click |
| REQ-007 | WHILE Threads shows, the rail's row keys shall not move the project cursor or open a project row, and PROJECTS shall carry the attention dot when a project needs the user. | The review of the diff |
| REQ-008 | The Home page shall offer a Threads button that opens the Threads page. | Shot 745-07 |
| REQ-009 | The status bar shall not show a Threads button. | Shot 745-01 |
| REQ-010 | WHEN the thread store changes while Threads shows, the list shall follow it without a switch. | The review (the rail's store subscription refreshes it) |
| REQ-011 | The change shall pass `script/gates.sh --diff`. | The gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, the `rail.rs` lines checked
  again after #743 and #744.
- **P2 Code** — `rail.rs` (the header switch, the Threads view, the key guard), `threads_page.rs`
  (the shared rows), `home_page.rs`, `status_buttons.rs`, the Marley keymap, the in-app guide page;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run `script/e2e/745-threads-in-the-rail.sh`, read every
  shot.
- **P4 Complete** — CHANGELOG, the guide, the walkthrough and the architecture note (§21), an AD
  that records D2 beside AD-claude-678, ledger capture (§19), close the ticket, archive, commit.
