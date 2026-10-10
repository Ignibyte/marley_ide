---
pipeline_id: d371752c-f773-456a-889c-ff03f56bdf4b
ticket: docs/planning/tickets/open/TICKET-739-rustal-icons-in-the-status-bar.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Rustal icons in the status bar
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), agents anywhere (docs/planning/design-notes/agents-anywhere-2026-10-10.md)
references:
  - docs/planning/pipeline/queued/737-the-threads-page.spec.md
  - docs/planning/pipeline/queued/738-marley-and-rusty-in-tabs.spec.md
---

## Title
Home, Rusty, Threads and Marley as buttons at the right of the status bar, each opening its tab in
the group you are looking at. Chad, 2026-10-10: "remove the rustal icons to be placed down in the
bottom right and they just open in a new tab."

## Scope
### In
- A status item, added to every workspace's status bar from Marley's `init` (no edit to
  `crates/zed`), with four icon buttons and tooltips:
  - **Home**: Home's page (#701) in a tab of the shown group;
  - **Rusty**: Rusty's home page (#679) in a tab of the shown group; shown while Rusty is on and
    connected;
  - **Threads**: #737's page in the shown group; hidden while AI is disabled;
  - **Marley**: #738's `talk to marley`; shown while the Marley entry is there.
  An open tab of that kind in the shown group comes forward instead of a second one.
- The rail header's Rusty button goes; `rusty::OpenHome` and the Rusty group stay.
- Both layouts: the status bar is the same in Zed's.
- `docs/marley/guide.md`: the buttons; the rail header no longer has Rusty's.

### Out (explicitly deferred)
- Keys for the buttons: their commands keep the keys they have.
- Moving Home's page out of Home: Home keeps its page as its first tab (#701).

## Reference (§20)
Upstream Zed, `workspace::StatusBar`: right-side `StatusItemView`s (the dock buttons, the cursor
position) drawn the same in every layout. Behavior kept: Zed's status bar and its item order; the
buttons sit beside Zed's dock buttons.

### Prior art
- **The code we ship:**
  - `workspace::StatusItemView` and `StatusBar::add_right_item` (`status_bar.rs`); right items
    render reversed, so an item added before Zed's own sits beside the dock buttons;
    `insert_item_after` / `position_of_item` for a chosen place.
  - Marley adds no status item yet; its panels (Fleet, Containers, Knowledge) reach the status bar
    as dock buttons (`Panel::icon`).
  - `home_page::ensure` (`home_page.rs:40`) and `rusty/home_tab.rs` `open_later` (183): the pages
    and how they are added to a pane today.
  - `rail.rs` `render_rusty_button` (4426), the button that leaves.
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md` (the status bar
  cell, `StatusItemView`).
- **Published material:** none.

## UI proof
`script/e2e/739-rustal-icons-in-the-status-bar.sh`, under `compositor sway`, with the scratch
project shown, Rusty on through its stand-in, and the Marley entry on the scripted agent.

Shots:
- `739-01-bar`: the status bar's right end with the four buttons, and the rail header without the
  Rusty button.
- `739-02-home`, `739-03-rusty`, `739-04-threads`, `739-05-marley`: each button clicked in the
  project: its tab in the project's center.
- `739-06-again`: Threads clicked again: the same tab comes forward, no second one.

## Locked-In Decisions
- **D1:** one status item holding four buttons, added from Marley's `init`, so `crates/zed` is not
  touched.
- **D2:** a button opens its tab in the shown group (Chad: "they just open in a new tab"), except
  Marley, which follows #738 (an open conversation comes forward where it is).
- **D3:** the rail keeps Projects and its filter; only the Rusty button leaves.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The status bar shall show Home, Rusty, Threads and Marley buttons at its right while each one's feature is there. | Shot 739-01 |
| REQ-002 | WHEN Home, Rusty or Threads is clicked, the system shall open that page in a tab of the shown group. | Shots 739-02 to 739-04 |
| REQ-003 | WHEN Marley is clicked, the system shall run `talk to marley`. | Shot 739-05 |
| REQ-004 | WHEN a button is clicked while its tab is open in the shown group, the system shall bring that tab forward and open no second one. | Shot 739-06 |
| REQ-005 | The rail's header shall not show the Rusty button. | Shot 739-01 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — a new `status_buttons.rs` in `marley_workbench`, `rail.rs` (the button removed),
  `home_page.rs` and `rusty/home_tab.rs` (open in a given workspace), the guide; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
