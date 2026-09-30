---
pipeline_id: fe052688-6560-4d77-b0fe-b55aec8ea2c5
ticket: docs/planning/tickets/open/TICKET-609-agent-tab.md
status: Phase 4 — Complete PASS
title: "The Agent tab: an agent's full detail in the center"
type: feature
slice: prong 2, D20, wave 1; after #608
references: [docs/marley/fleet-contract.md, docs/planning/pipeline/queued/608-agent-snapshot-in-the-fleet-panel.spec.md]
---

## Title
Opening an agent gives it a tab in the center: its run's phases and gates in time, its event
log, its host's resources over time, its token use, and the other agents on its host.

## Scope
### In
- **Opening:** a double-click on an agent's row, Enter on the selected row, or an Open button in
  its snapshot (#608). An agent's tab is found again by its id, not opened twice. The tab's title
  is the agent's name, with the fleet's icon.
- **The tab**, a read-only center item (`workspace::Item`), each part from the contract:
  - the header of #608's snapshot, wider;
  - the phase timeline: each phase as a bar from its start to its end (the active one to now),
    in the run's order, its gates under it with their state and their failure's detail line;
  - the event log: the run's events, newest first, time and kind and text, scrolling;
  - resource history: CPU, memory and network as line graphs over the samples Marley kept for the
    host, the last 30 minutes at most, with the current value beside each;
  - tokens: input, output and cache reads, for the run and for the day;
  - the host's other agents, each a row that opens its own tab.
- **Samples:** Marley keeps each host's snapshots in a ring buffer while any fleet surface shows
  it, one sample per poll, 30 minutes at most; the graphs draw from it, so a newly opened tab
  shows what was kept since the fleet began showing.
- **Live:** the tab follows the provider's changes while it is open.

### Out (explicitly deferred)
- Keeping the tab across a restart (a `SerializableItem`), and samples kept across a restart.
- The live terminal of a harness session (plan D10, wave 4).
- Actions.

## Reference (§20)
Upstream Zed: a center item is `workspace::Item` (`crates/workspace/src/item.rs:170-214`);
Marley's `DecisionsView` is the smallest read-only one, found again rather than opened twice
(`crates/marley_workbench/src/decisions.rs:52-59, 405-420`). The graphs draw with gpui's `canvas`
and `PathBuilder`, as Zed's git graph draws its lines (`crates/git_ui/src/git_graph.rs:3219-3431`).
Orca opens a card's detail from its dashboard (`docs/orca_architecture/05-terminal-and-workspace.md:466-494`).

### Prior art
- **Behavior maps:** the Orca dashboard note; #604's open-on-two rule.
- **Published material:** none needed.
- **Code we already ship:**
  - `DecisionsView` (the item and its opener).
  - `BrowserView`'s `open_url_tab` and `show_tab_where` for finding a tab again
    (`crates/marley_workbench/src/browser.rs:8238-8262`).
  - `canvas` with `PathBuilder::stroke` and `fill` (`git_graph.rs`, `circular_progress.rs:89-175`,
    `crates/gpui/examples/painting.rs`).
  - No sparkline exists in any Marley crate; Ely-GPUI-Components' `charts/` (MIT OR Apache-2.0)
    may be read for its scale and path code, rewritten against Zed's theme, with its notice
    kept if any code is taken. The Plan phase chose not to read it: a line through evenly spaced
    samples is a few lines of `PathBuilder::stroke`, `move_to` and `line_to`, as
    `circular_progress.rs:89-140` builds its arcs, so nothing is taken.
  - The rail's double-click (`rail.rs:4408-4410`, `ClickEvent::click_count() == 2`, #604) and
    #604's scenario helper `double_click`.
  - `Workspace::items_of_type`, `activate_item`, `add_item_to_active_pane`, `panes()` and
    `Pane::active_item()` for finding a tab again and for whether one shows.

## UI proof
The scenario `script/e2e/609-agent-tab.sh` (sway), on the pseudo provider:
- lets the fleet show for a minute, then double-clicks the working agent (`tab.png`: the phase
  timeline, the gates, the event log, the three graphs with a minute of samples, tokens, the
  host's other agent);
- (a click on the failed agent selects it first, then Enter opens it);
- opens the failed agent with Enter (`failed.png`: the failed gate with its detail);
- double-clicks the first agent again and shoots one tab, not two (`again.png`);
- opens the other agent from the host section (`other.png`).

## Locked-In Decisions
- D1 — The tab is read-only and not restored after a restart, for now.
- D2 — Marley keeps the samples, 30 minutes per host, while the fleet shows; the contract carries
  one snapshot, not a history.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user double-clicks an agent's row, presses Enter on it, or chooses Open in its snapshot, Marley shall open the agent's tab in the center, or show the one already open. | Shots `tab.png`, `again.png` |
| REQ-002 | WHILE the tab shows, it shall draw the run's phases in order as a timeline with each phase's gates and their states. | Shots `tab.png`, `failed.png` |
| REQ-003 | WHILE the tab shows, it shall list the run's events, newest first. | Shot `tab.png` |
| REQ-004 | WHILE the tab shows, it shall draw CPU, memory and network over the samples kept for the host. | Shot `tab.png` |
| REQ-005 | WHILE the tab shows, it shall show the run's and the day's tokens. | Shot `tab.png` |
| REQ-006 | WHEN the user opens another agent from the host section, Marley shall open that agent's tab. | Shot `other.png` |

## Phase Plan
- **P1 Plan** — promote, recall, the design (the item, the ring buffer's home, the graph element).
- **P2 Code** — the opener, the item, the graphs; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
