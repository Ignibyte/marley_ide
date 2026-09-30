# The selected agent's snapshot under the Fleet panel's list — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-608-agent-snapshot-in-the-fleet-panel.md
- **Pipeline spec:** 608-agent-snapshot-in-the-fleet-panel.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-604: a row that is a place to look marks on one click and opens on two.
  - ui::ProgressBar has a fixed h_2 height and an over-colour past its max.
  - Sections a provider cannot fill are left out, never drawn empty (the contract's capabilities).
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.

### Promotion (Opus, 2026-09-30)
- The pair moved to `active/`; the BACKLOG row removed; the ticket in-progress.
- **Recall, again:**
  - AD-claude-604: a row that is a place to look marks on one click; opening is #609's.
  - F-claude-600 (a hand-built menu lost the keyboard to the rail's own focus) and F-claude-571
    (a child's click bubbling into a parent's focus handler): the row's click focuses the panel
    itself, and nothing inside a row takes the focus.
  - F-claude-607 and PR-claude-607: the panel's reads run while a dock shows it; a selection
    adds no reads of its own beyond one at the moment of the click.
  - The brain (consultation 4ef2294a2d224b27b3592b709f4d4395): nothing on this seam beyond
    #607's decision.
- **Seams re-verified:**
  - `ui::ProgressBar::new(id, value: f32, max_value: f32, cx)` (`progress_bar.rs:21`), over
    colour `status().error` past its max.
  - `row_frame` (`rail.rs:6679`): border and fill when selected, hover otherwise.
  - `AgentInfo { id, name, runtime, state, state_since_ms, last_seen_ms, host_id, cwd, model }`,
    `AgentDetail { agent, work_item, run, recent_events, usage, question, host }`,
    `Memory { used_bytes, total_bytes }`, `Cpu { percent: f64 }`.
  - The fixtures carry `cwd`, `model`, `state_since_ms`, `usage.today`, review-1's question q-19
    with options Yes and No, and both hosts' `cpu` and `memory`.
  - The split pattern in `git_graph.rs:97, 548-563, 3619-3650, 4023-4031`.
  - The rail's `key_context("MarleyRail menu")` (`rail.rs:7108`).

### Design
- **Approach.**
  - *Which details are read (D1).* A second global, `Wanted`, maps each Fleet panel's entity
    id to its selection, `Selected { source, agent }` (the source's name and the agent's id).
    `read_providers` keeps in each `Source` the `AgentDetail`s of wanted agents only; for the
    pseudo provider these come from its reading's `details`. The panels do not observe
    `Wanted`, so a selection redraws nothing by itself. A click registers the selection and
    runs one read at once (`cx.defer`), so the snapshot fills without waiting for the next
    poll; a panel's release removes its entry.
  - *Capabilities (D2).* `Source` keeps its handshake's `capabilities`. The snapshot draws the
    work item for `work_items`, the phase strip for `runs`, the resources for `hosts`, tokens for
    `usage` and the question for `questions`; the header always.
  - *Selection.* `FleetPanel.selected: Option<Selected>`. A row's click selects it and focuses
    the panel (`window.focus`), so the keys follow. The panel's key context is
    `FleetPanel menu`, so Zed's `menu::SelectNext` and `menu::SelectPrevious` (Down, Up) reach
    its `on_action` handlers, which walk the agents in the order they are drawn (one function,
    `drawn_order`, shared by the render and the keys). When a reading no longer lists the
    selected agent, the global's observer drops the selection and its `Wanted` entry.
  - *The split.* With an agent selected, the panel is a column: the list, a 1 px handle, the
    snapshot. The snapshot takes `snapshot_ratio` of the height (a third by default); the
    handle's `on_drag(DraggedFleetSplit, ...)` and the column's `on_drag_move` set the ratio
    from the pointer's place in the column's bounds, clamped between 0.15 and 0.85. With no
    selection the list fills the panel.
  - *The snapshot's sections*, each a small labelled block:
    - header: name, `runtime · model`, the state chip and "for 12 m" from `state_since_ms`
      against the reading's time, then the host's name and the folder (`cwd`);
    - work item: key and title, and the store's status word;
    - phase strip: one segment per phase, `flex_1`, 6 px high, coloured by `PhaseState`
      (passed: success, active: accent, failed: error, skipped and pending: muted
      element colours), with "phase n/m: name" under it, or the failed phase's name;
    - resources: CPU and memory as `ProgressBar`s with their numbers ("42 %",
      "3.1 / 8.0 GB"), or "no resources yet" when the host has no snapshot;
    - tokens today: "in 3.9M · out 310k";
    - question: its prompt and its options as read-only chips.
  - *No float casts* (the Marley lint table): `Cpu.percent` and `AgentProcess.cpu_percent` become
    `f32` in `marley_sdk` (the wire's numbers are unchanged), the pseudo's wobble table becomes
    `i16` so `f32::from` takes it, and memory's share is computed in whole percent and passed
    through `f32::from(u16)`. Sizes and token counts format from integers.
- **File manifest** (all Marley crates; no Zed crate is touched):
  - `crates/marley_sdk/src/host.rs`: `Cpu.percent` and `AgentProcess.cpu_percent` as `f32`.
  - `crates/marley_sdk/src/pseudo.rs`: the wobble as `i16`, `f32` arithmetic.
  - `crates/marley_workbench/src/fleet.rs`: `Wanted`, `Selected`, `Source.capabilities` and
    `Source.details`, the selection, the keys, the split and the snapshot.
- **Visual check plan** (`script/e2e/608-agent-snapshot-in-the-fleet-panel.sh`, sway, on the
  pseudo provider; rows at about y 141, 186 and 258 in #607's shots):

  | REQ | Set up and do | Shot |
  |---|---|---|
  | REQ-001, REQ-002 | Toggle the panel, click build-1 | `working.png`: build-1 drawn selected; the snapshot below with the header, "for …", RB-142's title, the strip with code active, CPU and memory bars, tokens today |
  | REQ-003 | Click review-1 | `question.png`: the question and its Yes and No |
  | REQ-004 | Click docs-1 | `failed.png`: its failed phase segment in red, named under the strip |
  | REQ-005 | Click review-1, press Down | `keys.png`: docs-1 selected, its snapshot shown |
  | REQ-006 | (review) | The capability checks in the diff; the pseudo offers every capability, so no scenario shows a section left out |

  Also seen in passing: the selection held across the 2 s reads between shots.
- **Risks and decisions:**
  - `Wanted` keyed by panel entity id keeps one selection per workspace; the reads cover the
    union, which stays small.
  - Changing `Cpu.percent` to `f32` is a type change in a crate only Marley uses so far.

## Phase 2 — Code (2026-09-30)
- **Built,** as designed:
  - `marley_sdk`: `Cpu.percent` and `AgentProcess.cpu_percent` are `f32`, and the pseudo's
    `WOBBLE` table is `i16`, so the panel's bars take the numbers with no cast.
  - `fleet.rs`:
    - `Wanted` (a global of each panel's `Selected { source, agent }`), and `Source` with
      `capabilities` and the `details` of wanted agents; `read_pseudo` keeps those from its
      reading; `read_now` reads at once for a new selection.
    - `Source::host_groups`, which the list and `drawn_order` (the keys' order) share, and
      `Source::host` and `host_name`.
    - `FleetPanel.selected` and `snapshot_ratio`: a row's click focuses the panel and selects;
      `menu::SelectNext` and `SelectPrevious` under the key context `FleetPanel menu` step
      through `drawn_order`; the global's observer drops a selection whose agent left the
      list; the panel's release removes its `Wanted` entry.
    - The split: the list (`flex_1`), a 1 px handle with a 7 px grip that drags a
      `DraggedFleetSplit`, and the snapshot at `relative(snapshot_ratio)`, set by the body's
      `on_drag_move` and clamped to 0.15 to 0.85.
    - `render_snapshot` and `render_snapshot_header`, `render_phase_strip` with `phase_color`,
      `render_resources` with `meter` (`ui::ProgressBar`), and the formatters `how_long`,
      `compact` and `gigabytes`; "Reading…" until the first detail arrives.
- **Deviations:** none from the design. The draft was written while `just install` compiled
  (L-claude-606) and copied in after it finished.
- **Review of the diff:**
  - REQ-001: the row's click selects, and the split shows the snapshot.
  - REQ-002: the header (state chip, "for …"), the work item, the phase strip, the CPU and
    memory bars and tokens today.
  - REQ-003: the question's prompt and its options as chips.
  - REQ-004: a failed phase is red in the strip and named under it.
  - REQ-005: Down and Up through Zed's list actions in the panel's key context.
  - REQ-006: each section checks `Source::offers` for its capability.
  - Re-entrancy: a selection writes only the `Wanted` global and defers its read; the observer
    reads globals only; `panel_shows` runs only from the read, at the top level.
  - Nothing changed as a result of the review.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/608-agent-snapshot-in-the-fleet-panel.sh` under sway, on the pseudo
  provider: toggle the panel, click build-1, review-1 and docs-1 in turn, click review-1 and
  press Down, then drag the split's handle from y 516 to y 400.
- **Shots, each read** (the last run; the panel cropped from each and read side by side):
  - `working.png` (REQ-001, REQ-002): build-1's row drawn selected, and below the split its
    snapshot: build-1 with a `working` chip, "claude-code · claude-opus · for 38 m",
    "build-1 · /srv/work/pipeline"; WORK ITEM "RB-142 · Split the pipeline module", "in
    progress"; RUN, a strip of four with the first green and the second blue, "phase 2/4:
    code"; RESOURCES, CPU 63 % and Memory 3.1 / 8.0 GB as bars; TOKENS TODAY "in 3.9M · out
    310k".
  - `question.png` (REQ-003): review-1 selected; its snapshot ends with QUESTION "Delete the
    old dispatch module?" and the chips Yes and No.
  - `failed.png` (REQ-004): docs-1 selected; its strip's second segment is red and
    "failed at code (2/4)" is under it in red.
  - `keys.png` (REQ-005): after a click on review-1 and Down, docs-1 is the selected row and
    its snapshot shows (its chip `stale` by then, as the pseudo provider intends).
  - `split.png` (the resizable split): the handle moved up to about y 400 and the snapshot
    took the room.
  - REQ-006 is the review's: the pseudo offers every capability, so no shot leaves a section
    out.
  - The selection held across the 2 s reads between shots.
- **Red found and fixed:** run 1's snapshot, at a third of the panel's height, cut the tokens
  line and left review-1's question below the fold on the 1000 px output. The default is now
  half (`SNAPSHOT_RATIO`), with the sections' gap `gap_2`; the spec's scope line says so.
  `just gate-diff` after the fix: 17 PASS, 0 FAIL. Run 2 was green; run 3 added the split's
  drag step and was green.
- **Seen, not a fault:** docs-1's CPU reads 1 % in one shot: the pseudo provider's wobble
  takes vps-2's 18 % below zero and the reading is held at its floor of 1.
- **Focus:** sway stopped with the run's Marley each time; Hyprland had no Marley windows
  before or after.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Added: an agent's snapshot in the Fleet panel);
  `three-prong-plan.md` (D20's wave, #608 shipped); `marley_workbench.md` (selection and the
  snapshot) and `marley_sdk.md` (`f32` CPU); the guide (`guide.md`), the guide page (a new
  article, `fleet-snapshot`, and its nav entry) and the walkthrough (Part 12's new checks). No
  path outside the Marley-owned set changed, so no touchpoint row.
- **Knowledge:** F-claude-608-a-third-of-the-panel-hid-the-snapshots-question-001,
  AD-claude-608-the-fleet-reads-full-detail-only-for-selected-agents-001.
- **Brain:** consultation 4ef2294a2d224b27b3592b709f4d4395 closed with `brain decide`
  (`decisions/marleys-fleet-panel-reads-full-agent-detail-only-for-selected-agents`, follow-up
  by 2026-10-14).

