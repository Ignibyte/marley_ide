---
pipeline_id: 53964479-3bfd-42be-b369-2a5d8135e81e
ticket: docs/planning/tickets/open/TICKET-746-marley-and-rusty-in-the-title-bar.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Marley and Rusty in the title bar
type: feature
slice: the Marley layout (docs/marley/workbench-shell.md), tabs everywhere (docs/planning/design-notes/tabs-everywhere-2026-10-11.md)
references:
  - docs/planning/pipeline/completed/739-rustal-icons-in-the-status-bar.spec.md
  - docs/planning/pipeline/completed/738-marley-and-rusty-in-tabs.spec.md
  - docs/planning/pipeline/queued/745-threads-in-the-rail.spec.md
---

## Title
The Rusty and Marley buttons move from the status bar's right (#739) to the title bar's right,
beside the guide's `?` and the user menu. Chad, 2026-10-11: "Rusty and Marley go to the top right
next to (?) and Avatar instead. That gives them more spotlight without mucking with the left panel
tabs that should only be for swapping."

## Scope
### In
- Two buttons in the title bar's right-hand group, after the `?` (#599) and before Sign In and the
  user menu, in both layouts:
  - **Rusty** (`RUSTY_ICON`, tooltip "Rusty"): Rusty's home page in a tab of the shown group, or
    that tab forward; shown while Rusty is on and connected.
  - **Marley** (`IconName::Sparkle`, tooltip "Talk to Marley" with its key): #738's `talk to
    marley`; shown while AI is on and the Marley entry is there.
  The rules and handlers are #739's, unchanged.
- The status bar's Rusty and Marley buttons leave; Home stays there.
- One small Zed touch in `crates/title_bar/src/title_bar.rs`: a hook through which a Marley crate
  hands the title bar a view, drawn after the `?`, with its row in `docs/marley/zed-touchpoints.md`.
- The guide (`docs/marley/guide.md`, the in-app page) and the walkthrough: where the buttons are.

### Out (explicitly deferred)
- New icons: Chad will redraw them.
- Home in the title bar: Chad kept it in the status bar.
- Keys for the buttons: their commands keep the keys they have.

## Reference (§20)
Upstream Zed, `title_bar::TitleBar`: the right-hand group (call controls, connection status, the
update notice, Sign In, the user menu), drawn the same in every layout, with Marley's `?` added
after the update notice (#599). Behavior kept: Zed's group and its order; Marley's buttons join it
beside the `?`.

### Prior art
- **The code we ship:**
  - `title_bar.rs`: `init` (104) makes a `TitleBar` per workspace (`observe_new`, 112-118) and
    sets it as the workspace's `titlebar_item`; `TitleBar::new` (466); the right-hand group in
    `render` (379-425), the `?` hunk (387-400) by action name, since `title_bar` depends on no
    Marley crate. Zed has no seam for another crate's title bar item.
  - `workspace::Workspace::titlebar_item` (`workspace.rs:3553`) and its render (9742): a workspace
    draws its own title bar, so a button holding that workspace opens in the shown group.
  - `pane.rs` `MarleyNewItemMenu` (4327, #699) and `workspace.rs` `MarleyCloseGuard` (744): the
    house shape for a hook, a `Global` holding a function a Marley crate sets.
  - `status_buttons.rs` (#739): `RustalButtons`, its three observers (settings, `rusty::Rusty`,
    `assistant::Assistant`) and its Rusty (82-96) and Marley (114-134) buttons, which move.
    `rusty::{is_on, is_connected, open_home_here, RUSTY_ICON}` (`rusty.rs` 697, 692, 86, 90),
    `assistant::{marley_present, talk_to_marley}` (`assistant.rs` 996, 1003).
  - `marley_workbench` already depends on `title_bar` (`Cargo.toml:82`).
- **Behavior maps:** `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`: the
  `title_bar` row of its crate table (the titlebar view and its window-control shell) and the
  workspace's `titlebar_item` slot.
- **Published material:** none.

## UI proof
`script/e2e/746-marley-and-rusty-in-the-title-bar.sh`, under `compositor sway`, on #739's set-up:
the scripted agent runs the Marley entry, Rusty is on through `marley_rusty`'s stand-in `rusty-mcp`,
the copy's thread list emptied, the scratch project shown.

Shots:
- `746-01-bar`: the title bar's right end: Rusty and Marley after the `?`, before the user menu;
  the status bar's right end with Home and none of the two.
- `746-02-rusty`: Rusty clicked: Rusty's home page in a repo tab.
- `746-03-marley`: Marley clicked: a Marley tab in the repo; one session started.
- `746-04-again`: Default shown, Marley clicked: the repo shown with that Marley tab in front; no
  second session.
- `746-05-rusty-off`: Rusty turned off in the run's settings: the title bar without Rusty, Marley
  still there.
- `746-06-zed`: `marley: use zed layout`: both buttons in the title bar of Zed's layout.

## Locked-In Decisions
- **D1 — A view hook in `title_bar.rs`.** `MarleyTitleBarButtons`, a `Global` holding a function
  that takes the title bar's workspace and returns a view; `TitleBar::new` asks it once and keeps
  the view; the right-hand group draws it after the `?`. The view is Marley's: it observes what
  each button rests on and redraws itself. Rejected: buttons by action name, as the `?` is (the
  title bar cannot know when Rusty is connected or the Marley entry is there); wrapping Zed's title
  bar item in a Marley view (the platform title bar lays out the window controls at the far right,
  and a wrapper would put the buttons outside them); keeping them as a status item (Chad moved
  them).
- **D2 — After the `?`, before Sign In and the user menu:** next to both, as Chad put it. Rusty,
  then Marley, #739's order. Rejected: before the `?` (the `?` would split them from the avatar
  Chad named).
- **D3 — The buttons and their rules move unchanged** into a second view in `status_buttons.rs`,
  `TitleButtons`; `RustalButtons` keeps Home. Rejected: one view in two places (a status item and
  a title bar view need different traits and widths).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Rusty is on and connected and the Marley entry is there, the title bar shall show Rusty and Marley buttons after the `?` and before the user menu. | Shot 746-01 |
| REQ-002 | The status bar shall not show Rusty or Marley buttons, and shall keep Home. | Shot 746-01 |
| REQ-003 | WHEN Rusty is clicked, the system shall open Rusty's home page in a tab of the shown group, or bring that tab forward. | Shot 746-02 |
| REQ-004 | WHEN Marley is clicked, the system shall run `talk to marley`: the latest Marley conversation forward where it is, else a new one in a tab of the shown group. | Shots 746-03, 746-04 and the session count |
| REQ-005 | WHILE Rusty is off or not connected, the title bar shall not show the Rusty button. | Shot 746-05 |
| REQ-006 | WHILE AI is off or the Marley entry is not there, the title bar shall not show the Marley button. | The review of the diff (#739's condition, moved) |
| REQ-007 | The buttons shall show in Zed's layout as in Marley's. | Shot 746-06 |
| REQ-008 | The Zed hunk shall carry a `// Marley:` comment, and `docs/marley/zed-touchpoints.md` shall have its row. | The review; `enforce-zed-ledger.sh` |
| REQ-009 | The change shall pass `script/gates.sh --diff`. | The gate's exit code |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes.
- **P2 Code** — the ledger row first, then `title_bar.rs` (the hook, the field, the child),
  `status_buttons.rs` (`TitleButtons`, the hook set from `init`, `RustalButtons` with Home only),
  the in-app guide page; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the guide, the walkthrough and the architecture note (§21), ledger
  capture (§19), close the ticket, archive, commit.
