# The Agent tab: an agent's full detail in the center — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-609-agent-tab.md
- **Pipeline spec:** 609-agent-tab.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - DecisionsView is the smallest read-only center item, found again rather than opened twice.
  - No sparkline exists in a Marley crate; git_graph.rs and circular_progress.rs draw with canvas and PathBuilder.
  - Ely-GPUI-Components (MIT OR Apache-2.0) charts/ may be read for scale and path code, rewritten against Zed's theme (evaluation of 2026-09-30).
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:**
  - #608 (AD-claude-608): the reads keep full detail only for what a `Wanted` entry names; the
    tab registers its own entry, so its detail is read while it lives.
  - F-claude-607 and PR-claude-607: the reads run while a Fleet surface shows, asked of the
    windows at each read. The tab is a surface: the check grows to "a Fleet panel an open right
    dock shows, or an Agent tab that is some pane's active item".
  - AD-claude-604: one click marks, two open; Enter opens the marked row.
  - The brain: not asked in this phase, an omission caught at Complete; asked then
    (consultation 79fe7caff20a443bbf32d9f0b27061c0), it returned nothing on this seam, only
    follow-ups due on other projects.
- **Seams re-verified:**
  - `decisions::open` (`decisions.rs:51-59`): `items_of_type::<DecisionsView>`, then
    `activate_item(&open, true, true, ...)` or `add_item_to_active_pane(Box::new(view), None,
    true, ...)`; `impl Item for DecisionsView { type Event = (); fn tab_content_text }`.
  - `Workspace::items_of_type` (`workspace.rs:4383`), `Workspace::panes` (`workspace.rs:6422`),
    `Pane::active_item` (`pane.rs:1385`).
  - `canvas(prepaint, paint)` with `PathBuilder::stroke(width)`, `move_to`, `build()` and
    `window.paint_path(path, color)` (`circular_progress.rs:89-125`).
  - The contract: `Gate { name, state, detail }` with `GateState { Pending, Pass, Fail,
    Skipped, Unknown }`; `Event { at_ms, kind, text }`; `TokenUse { input, output, cache_read:
    Option }`; `HostSnapshot.network: Option<Network { rx_bps, tx_bps }>`. The fixtures give
    each run's phases their `started_ms` and `ended_ms` and gates, and `details.json`'s
    `recent_events`.
  - `ClickEvent::click_count()` and the rail's double-click; #604's `double_click` helper.

### Design
- **Approach.**
  - *A new file, `crates/marley_workbench/src/agent_tab.rs`*, since the tab is a component of its
    own: `AgentView`, its `open` function and the `Sparkline` drawing. `fleet.rs` makes what the
    tab reads `pub(crate)`: `Fleet`'s sources and samples, `Source` and its helpers,
    `Selected`, `Wanted`, `state_chip`, `runtime_icon`, `phase_color`, `how_long`, `compact`,
    `start_polling`.
  - *Opening.* `agent_tab::open(workspace, selected, window, cx)`: the `AgentView` for the same
    `Selected` among `items_of_type`, activated; else a new one added to the active pane. The
    Fleet panel gets its workspace's `WeakEntity` from `init`. It opens on a row's double-click
    (`click_count() == 2`, after the first click selected it), on `menu::Confirm` (Enter) for the
    selected row, and on an Open button at the snapshot's header. The tab's host section opens
    the others the same way.
  - *The tab's data.* `AgentView { selected, workspace, focus_handle, _fleet }` registers its
    entity id in `Wanted` at creation and leaves it at release, so the reads keep its detail. It
    observes the `Fleet` global and draws from it; an agent no longer listed reads "No longer
    listed by <source>."
  - *Reads while the tab shows.* `panel_shows` becomes `fleet_shows`: also true when an
    `AgentView` is the active item of a pane in a window's shown workspace. The tab's render
    starts the reads when none run, as the panel's does.
  - *Samples (D2).* `Fleet.samples: HashMap<HostKey, VecDeque<Sample>>`, `HostKey { source,
    host }` and `Sample { at_ms, cpu, memory, network }` (CPU and memory in percent as `f32`,
    network as `rx_bps + tx_bps`). Each read appends one per host snapshot and drops what is
    older than 30 minutes. The graphs draw from these.
  - *The tab's parts*, in a scrolling column:
    - the header: #608's, wider, with the work item's title beside the key;
    - PHASES: one row per phase, its name, a track spanning the run's start to now (or its
      end) with the phase's bar placed by `relative()` from per-mille integers (no float cast),
      coloured by `phase_color`, and under it each gate's state word and name, the failure's
      detail in red;
    - EVENTS: the run's events and the agent's `recent_events`, merged, newest first, each
      "12:04:31 · kind · text" in local time;
    - RESOURCES: CPU, memory and network, each a `Sparkline` (a `canvas` stroking a path through
      the samples, scaled to 100 for CPU and memory and to the window's largest for network)
      with the current value beside it and "N samples" under the three;
    - TOKENS: the run's and the day's input, output and cache reads;
    - ON THIS HOST: the other agents the list puts on the same host, each a row with its chip
      that opens its tab.
  - *Item.* `impl Item for AgentView`: `tab_content_text` the agent's name, `tab_icon`
    `IconName::Server`; `type Event = ()`; not serialized (D1).
- **File manifest** (Marley crates only):
  - `crates/marley_workbench/src/agent_tab.rs` (new): `AgentView`, `open`, `Sparkline`.
  - `crates/marley_workbench/src/fleet.rs`: the workspace handle, the opener calls (double-click,
    Confirm, the Open button), samples, `fleet_shows`, `pub(crate)` items.
  - `crates/marley_workbench/src/marley_workbench.rs`: `mod agent_tab;`.
- **Visual check plan** (`script/e2e/609-agent-tab.sh`, sway, on the pseudo provider):

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001..005 | Toggle the panel, wait about a minute, double-click build-1 | `tab.png`: build-1's tab in the center with the timeline (plan passed, code active), its gates, the events newest first, three graphs with about 30 samples, the tokens, review-1 under ON THIS HOST |
  | REQ-002 | Click docs-1 in the panel, press Enter | `failed.png`: docs-1's tab, code's bar red, the clippy gate failed with its detail |
  | REQ-001 | Double-click build-1 again | `again.png`: build-1's tab active, one tab for it in the tab bar |
  | REQ-006 | In build-1's tab, click review-1 under ON THIS HOST | `other.png`: review-1's tab |

- **Risks and decisions:**
  - A tab in the background keeps its `Wanted` entry but no reads run for it alone, so it shows
    what the last read gave when it comes forward; the samples have a gap while nothing shows.
  - Samples live in memory only (D2), so a restart empties the graphs.

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - `agent_tab.rs` (new): `open_later` (deferred through `Window::defer`), `open` (finds the tab
    by `Selected` among `items_of_type::<AgentView>` or adds one to the active pane),
    `AgentView` (its `Wanted` entry, a read at creation, the global observed), and its parts:
    `render_timeline` with `share` (per-mille integers, no float cast), `render_gates` and
    `render_gate`, `render_events` (run and recent events merged, sorted newest first,
    duplicates dropped, 50 shown), `render_history` with `history_row` and `sparkline` (a
    `canvas` stroking a `PathBuilder` line), `token_line`, and `render_neighbours`; `impl Item`
    with the agent's name and `IconName::Server`.
  - `fleet.rs`: `Fleet.samples` of `HostKey` to `VecDeque<Sample>`, filled by `sampled` each read
    and cut to 30 minutes; `memory_share`; `fleet_shows` (a Fleet panel an open right dock
    shows, or an `AgentView` that is a pane's active item); the panel's workspace handle from
    `init`, `open_selected` on a double-click, on `menu::Confirm` and on the snapshot's Open
    button (`render_snapshot_header` takes an `end` element); `pub(crate)` for what the tab reads.
- **Deviations:**
  - `render_agent` and `render_neighbours` are associated functions (clippy's `unused_self`):
    the neighbour rows reach the workspace through the listener's view.
  - Events are shown as "12 s ago · kind · text" rather than a clock time, so the log needs no
    time zone.
- **Review of the diff:**
  - REQ-001: the double-click, Enter and Open all go through `open_later`; the tab is found
    again by `Selected`.
  - REQ-002: the timeline places each started phase's bar on the run's span, colours it by
    state, and lists its gates with a failed gate's detail.
  - REQ-003: events newest first. REQ-004: three lines over the host's samples. REQ-005: the
    run's and the day's tokens with cache reads. REQ-006: the host's other agents open their
    tabs.
  - Re-entrancy: every open is deferred, since `open` reads each Agent tab and a neighbour's
    click comes from inside one; `fleet_shows` reads panes only from the read, at the top level.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/609-agent-tab.sh` under sway, on the pseudo provider: the panel
  shows for a minute; a double-click on build-1; a click on docs-1 and Enter; a double-click on
  build-1 again; a click on review-1 under build-1's ON THIS HOST.
- **Shots, each read** (run 2):
  - `tab.png` (REQ-001 to REQ-005): a build-1 tab with the server icon after "repo — bash".
    The header ("claude-code · claude-opus · for 38 m", "build-1 · /srv/work/pipeline",
    "RB-142 · Split the pipeline module · in progress"). PHASES: plan green over its 33 m, code
    green from its start to test's ("1 h") with "check pass" and "clippy pass", test a blue
    sliver at the end ("4 s"), complete pending. EVENTS newest first, from "0 s ago · gate ·
    clippy: no warnings" to "11 m ago · phase · code started". RESOURCES: CPU and network as
    waves and memory flat, with "24 %", "3.1 / 8.0 GB" and "in 55 kB/s · out 20 kB/s", over "35
    samples over 1 m". TOKENS: "This run in 838k · out 66k · cache 2.3M", "Today in 3.9M · out
    312k". ON THIS HOST: review-1 with its warning mark and `waiting`.
  - `failed.png` (REQ-002): docs-1's tab; code's bar red over 48 m, "check pass" and "clippy
    fail", and "clippy: 3 warnings in receipts.rs" in red; test and complete pending.
  - `again.png` (REQ-001): build-1's tab active again; the tab bar still holds one build-1 tab
    and the docs-1 tab.
  - `other.png` (REQ-006): review-1's tab opened after build-1's, with its test phase active
    and its tests gate passed, and build-1 under its ON THIS HOST.
- **Reds found and fixed** (run 1):
  - build-1's code phase drew as a dot: the pseudo provider marks the phase passed when the run
    moves on without giving it an `ended_ms`, and the timeline ended such a phase at its start.
    A phase with a start and no end now runs to now while active, else to the next phase's
    start. A store may do the same, so the fix is the tab's.
  - The snapshot's Open read as plain text beside the chip; it is an outlined button now.
  - `just gate-diff` after the fixes: 17 PASS, 0 FAIL. Run 2 added the `other` step and was
    green.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Added: the Agent tab); `three-prong-plan.md` (#609 shipped);
  `marley_workbench.md` (samples and the tabs in the Fleet section, and a new section for
  `agent_tab.rs`); the guide, the guide page (article `fleet-agent-tab` and its nav entry) and the
  walkthrough (Part 12's new checks). No path outside the Marley-owned set changed.
- **Knowledge:** F-claude-609-a-passed-phase-without-an-end-drew-as-a-dot-001,
  AD-claude-609-the-agent-tab-is-read-only-and-its-samples-live-in-memory-001.
- **Brain:** consultation 79fe7caff20a443bbf32d9f0b27061c0 closed with `brain decide`
  (`decisions/marleys-agent-tab-is-read-only-and-its-resource-history-lives-in-memory`,
  follow-up by 2026-10-14).

