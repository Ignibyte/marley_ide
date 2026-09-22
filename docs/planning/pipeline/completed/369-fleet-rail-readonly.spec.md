---
pipeline_id: 00e072a6-989a-4f07-a524-8c1233d6aa28
ticket: forge#369 (279d5673-899d-466a-be94-d6c3417c9251) · local docs/planning/tickets/open/TICKET-369-fleet-rail-readonly.md
aar_id: ca93ce71-629a-42b1-a6d0-438b16727ebf
status: Phase 5 — Complete PASS (shipped 2026-07-20; gate GREEN --diff, 15/15)
title: Fleet rail — read-only render of the FleetSnapshot (state chips + question-cards + staleness)
type: feature
milestone: M23
references:
  - docs/marley_architecture/orchestration-shell.md
  - docs/marley_architecture/fleet-control-plane.md
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/layout.rs
  - crates/marley_app/src/agent_view.rs
  - crates/ui_components/src/lib.rs
---

## Title
The shell's first fleet surface: a READ-ONLY rail rendering `marley_fleet::FleetSnapshot` (#367's pure model) —
per-seat cards with state chips from the closed vocabulary, generic opaque-label chips, render-only
question-cards for Waiting seats, staleness dimming, and attention ordering. *Insight before control*
(orchestration-shell.md §12 Layer 1, slice ③): zero writes, no dispatch, no session verbs. The rail must be
fully provable from an in-crate fixture feed — Layer 0 (`seat_events` emitters) is external and unshipped, and
the ② live client feed may not exist yet.

**Hard dependency: #367** (pure `marley_fleet`: `Session { id, title, state, question iff Waiting, labels
opaque map, last_event, transport }`; `FleetSnapshot` with reducer-ordered seats, derived attention, and the
per-seat staleness flag). This ticket renders THOSE types and re-derives nothing — if #367 has not merged at
promotion time, Phase 2 blocks on its types landing first.

## Scope
### In
- **NEW pure module `crates/marley_app/src/fleet_rail.rs`** (cov/MSI 100) — every render decision extracted:
  state-chip mapping (label + fill color per state, all 6), opaque-label chip projection (verbatim
  `key: value`, deterministic order), question-card content decision (present iff Waiting-with-question),
  staleness dim decision (consumes ONLY the reducer's flag), relative-time formatting (u64 seconds in —
  `Instant`/`Duration` banned in pure fns, the notify.rs:22 precedent; reuse/mirror `fmt_duration`
  agent_view.rs:108), empty-state hint, and a row projection that PRESERVES snapshot order.
- **`crates/marley_app/src/app.rs`** (shim, `mutants::skip`): the rail render arm calling only the pure fns;
  an `Option<FleetSnapshot>` app-state field; a toggle verb for the rail; a `fleet-demo-feed` fixture-load
  verb (dispatchable via `dispatch_for_test` app.rs:2107 and the palette) that installs/refreshes the demo
  snapshot and sets dirty.
- **`crates/marley_app/src/layout.rs`**: `dock_title(DockSide::Right)` "Details" → "Fleet" (stale since #153
  moved the cockpit to full-screen tabs; pure + already unit-tested at layout.rs:373) — contingent on D1's
  landing zone surviving Phase 2.
- **The demo fixture**: an in-crate `FleetSnapshot` exercising all six states, a Waiting-with-question seat,
  a stale seat, and opaque labels that are deliberately NOT Forge vocabulary.
- Landing zone (D1, Phase-2 final call): revive the permanently-Closed right dock slot as the fleet rail.

### Out (explicitly deferred)
- **Answering questions** — a Layer-2 receipted verb; the card is render-only and must honestly read as
  not-yet-actionable.
- **Dispatch UI, delivery states, session verbs** (`session.send`, surface-to-human, takeover) — Layer 2+.
- **Live feed wiring** — the ② `marley_forge_client` subscription/adapter is its own Layer-1 slice; this rail
  consumes whatever snapshot the app holds and ships fixture-driven.
- **Any Forge vocabulary in the render path** — "ticket"/"phase"/"gate" as *interpreted* concepts. A
  Forge-specific string in `fleet_rail.rs` or the render arm is a defect (orchestration-shell.md §2's
  mechanical check); such strings may only ever arrive as opaque label DATA from a feed.
- **Unifying the existing LOCAL Agents cockpit** (`RightSection::Agents`, app.rs:4912 — launched agent CLI
  panes) into the fleet envelope — a different surface for a different object; explicitly not touched.
- Persisting fleet data across restarts (the snapshot is ephemeral render state), OS notifications/sounds
  (#226 territory), seat filtering/search, and any reducer/model logic (#367 owns it).

## Reference (§20)
N/A — Marley-specific. The fleet control plane is Marley-original design; the references are
docs/marley_architecture/fleet-control-plane.md (§7.2 "The fleet rail — seats as first-class objects") and
orchestration-shell.md (§3 the `Session` envelope this rail renders; §12 Layer 1 slice ③). Verified before
writing this: docs/warp_architecture/subsystems/04-agent-ai-mcp.md documents Warp's Agent Mode as a cloud
conversation-loop client (SSE multi-agent transport + input classifier) — no observed fleet-of-seats rail UI
exists in our behavior maps. Zed's collab panel is a *people/channels* rail (not an agent-seat rail), is not
covered by our docs/zed_architecture/ maps, and its GPL source is off-limits regardless. No copyleft source
consulted.

### Prior art
1. **Behavior maps** — swept docs/warp_architecture/subsystems/ (04-agent-ai-mcp.md: Agent Mode =
   conversation loop, not a fleet rail) and docs/zed_architecture/ (crate maps only; no collab-panel behavior
   doc): no analog to adopt. The rail's DESIGN source is our own
   docs/marley_architecture/fleet-control-plane.md evidence night (§2's incident table is the requirements
   list: Error ≠ Idle first-class, absence-of-heartbeat as signal, questions as data).
2. **Published** — none found to adopt; there is no published spec for an agent-fleet rail (the MCP/LSP specs
   govern the *feed*, ticket ②/④'s concern, not this render). Recorded as checked.
3. **OUR DEPS + OUR OWN CODE (highest-yield — the sweep that pays):**
   - gpui `uniform_list` is already shipped in the editor tab (app.rs:5317) — available IF virtualization is
     ever needed, but it wants uniform row heights; question-cards make seat cards variable-height, and fleet
     sizes are ~4–12 seats (the evidence night ran 4), so a plain flex column + the EXISTING rail wheel-scroll
     idiom (`rail_scroll` + `scroll_steps`/`scroll_code`, app.rs:15188–15203) is the lean fit (D-OPEN-1).
   - The dock substrate ALREADY EXISTS: `docks: [DockState; 2]` (app.rs:163) with the right slot permanently
     Closed since #153/#171 (app.rs:1932–1934); `region_widths` handles all four open/closed combinations
     PURE + tested (layout.rs:108, test layout.rs:406) and the render already consults
     `self.dock(DockSide::Right)` (app.rs:15116); `dock_panel` renders either side incl. the right's inner
     border (app.rs:945, 965). Reviving the slot is configuration, not construction — this settled D1.
   - Chip/badge idioms: the #222 keycap chip (ui_components/src/render/keyboard_shortcut.rs:9–11) for chip
     shape; the #203 tab-rail badge for state colors (`colors.success`/`colors.danger`,
     ui_components/src/lib.rs:56/58); `rail_highlight` (app.rs:976) for wash-not-surface highlights; the
     cockpit `placeholder(hint)` empty-state idiom (app.rs:4902, `agents_empty_hint` app.rs:4916);
     `fmt_duration` (agent_view.rs:108, pure + tested) for duration tiering.

## Locked-In Decisions
- **D1 — Landing zone: revive the RIGHT dock slot (`docks[1]`) as the Fleet rail** (final confirmation is
  Phase 2's). Evidence: the slot, the pure width math, the panel renderer, and the accessor all exist and are
  tested (see Prior art 3); `dock_title(Right)` is a stale "Details" (layout.rs:68) ready to become "Fleet".
  Rejected alternatives: (a) the LEFT "Workspace" rail — it is the Workspace→Project→Tab NAVIGATOR
  (app.rs:15119–15122); mixing observation into navigation conflates two models; (b) a full-screen cockpit
  tab — the fleet must be glanceable BESIDE the work (fleet-control-plane.md §3: the cockpit window and the
  radio, "side by side"), and `RightSection::Agents` already means LOCAL agent panes (naming collision).
  v1 reuses `DOCK_WIDTH`; widening is a Phase-2 option.
- **D2 — The #307 pattern, total**: app.rs is coverage-excluded and its render fns are `mutants::skip`, so
  EVERY decision (chip mapping, ordering preservation, relative time, staleness application, card/empty
  selection, label projection) is an extracted PURE fn in `fleet_rail.rs` with cov/MSI 100; the render arm
  only calls them; the headless drive + the gate-15 capture prove the excluded arms. Pure fns take plain data
  + u64 seconds (never `Instant` — notify.rs:22).
- **D3 — State chips are FILLS, never borders** (`PR-claude-gpui-no-box-sizing-border-inflates` — a border
  inflates an auto-sized cell); colors from `ThemeColors` (success/danger/accent/muted family); a unit
  asserts the 6 states map distinctly AND Error ≠ Idle in BOTH theme families (mirroring
  `rail_highlight_is_a_distinct_accent_wash`, app.rs:18202). Any hover/active wash on a card must differ from
  the panel surface (`PR-claude-selection-bg-distinct-from-container`).
- **D4 — The rail renders snapshot order VERBATIM and derives nothing.** Attention ordering
  (Waiting/Error first) and the staleness flag are #367's reducer outputs; a rail-side re-sort or a rail-side
  wall-clock staleness computation is a defect. Genericity is tested: the fixture's labels include
  non-Forge strings rendered verbatim.
- **D5 — Fixture-driven provability**: an in-crate demo `FleetSnapshot` + a `fleet-demo-feed` verb reachable
  from `dispatch_for_test` (headless) and the palette (the live gate-15 capture). No network, no ② client,
  no Layer-0 dependency. The verb replaces the snapshot OUTSIDE the render loop and MUST set dirty
  (`PR-claude-pump-state-change-must-set-dirty-to-repaint`).
- **D6 — Question-cards are honestly inert in Layer 1**: prompt + options render as NON-interactive chips
  (no mouse listeners on options) plus a muted "answers arrive in Layer 2"-class hint — the affordance must
  read as not-yet-actionable, not as a broken button.
- **D7 — Per-card affordances gate on the card's own data**, never on global state
  (`PR-claude-per-pane-render-affordance-must-gate-on-is-focused` — the per-item-loop form: dimming reads
  THIS seat's stale flag, the question-card reads THIS seat's question; no cross-seat leakage).

### D-OPEN (Phase 2 decides)
- **D-OPEN-1 — Scroll strategy**: plain flex column + the existing `rail_scroll`/`scroll_code` idiom
  (lean fit: ≤ ~12 variable-height cards) vs gpui `uniform_list` (needs uniform heights; would force
  question-card collapse). Lean stated; P2 confirms against the card layout it draws.
- **D-OPEN-2 — Toggle verb + persistence**: reclaim `"toggle-right-dock"` (currently re-routed to the
  Details cockpit tab since #153, app.rs:7906–7907) vs a NEW `toggle-fleet-rail` verb (keeps ⌘⇧B behavior
  untouched); and whether the rail's open state persists via the `persist_dock_*` settings idiom (the right
  slot currently boots Closed unconditionally, app.rs:1934).

## Acceptance Criteria (EARS)
One observable behavior per row. Verify: unit = `fleet_rail.rs` pure test (cov/MSI 100); headless = a
`*_headless` drive through the real dispatch path; capture = the gate-15 driven capture of the demo fixture.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a seat renders, its card shall show the seat title, its state chip, and its transport (tmux/bridge/local) as a subtle muted hint. | unit (card projection) + capture |
| REQ-002 | WHEN state chips are mapped, each of the 6 closed-vocabulary states shall get a distinct label + fill, and Error's fill shall differ from Idle's in BOTH theme families. | unit (all 6 + Error≠Idle across themes) |
| REQ-003 | WHEN a seat is Waiting AND carries a question, its card shall render the prompt + every option; WHEN a seat is any other state (or Waiting without a question), no question-card shall render. | unit (decision fn) + headless (fixture holds both cases) |
| REQ-004 | WHILE Layer 1 is in effect, clicking a question-card option shall change no app state, and the card shall carry a muted not-yet-actionable hint. | headless (click → state unchanged) + capture review |
| REQ-005 | WHEN a seat's labels map holds arbitrary key/value strings, the rail shall render each as a verbatim `key: value` chip in deterministic order, with no key-specific interpretation; no Forge-vocabulary constant shall exist in `fleet_rail.rs` or the render arm. | unit (non-Forge fixture strings verbatim) + review grep |
| REQ-006 | WHEN the snapshot flags a seat stale, its card shall render dimmed (muted over normal fg); an unflagged seat shall not dim; the rail shall apply ONLY the reducer's flag (no rail-side clock math). | unit (dim decision takes only the flag) + headless |
| REQ-007 | WHEN a seat's last_event is N seconds past, its card shall show a relative time formatted by the pure helper (`fmt_duration` tiering + "ago"). | unit (u64 secs in → string out) |
| REQ-008 | WHEN the snapshot orders seats by derived attention (Waiting/Error first), the rail shall render cards in exactly snapshot order, never re-sorting. | unit (projection preserves order) + headless (fixture attention order ≠ insertion order) |
| REQ-009 | WHEN the fleet rail is toggled open, the center shall inset by the dock width and the left Workspace rail / files panel / pane grid shall keep working; toggled closed, the width returns to the center. | headless (toggle → region assertions) + existing suite green |
| REQ-010 | WHEN the snapshot is empty or absent, the open rail shall render a muted empty-state hint — never a crash and never a blank panel. | unit (empty-hint decision) + headless (empty fixture) |
| REQ-011 | WHEN a new snapshot replaces the current one outside the render loop (the fixture-feed verb), the rail shall repaint on the next frame without user input. | headless (dispatch update → next frame shows the new state) |

## Testing boundary (honest)
`fleet_rail.rs` is pure → cov/MSI 100 carries REQ-002/003(decision)/005/006(decision)/007/008(projection)/010(decision).
The app.rs arm is a coverage-excluded `mutants::skip` shim → the `*_headless` drives through
`dispatch_for_test` (app.rs:2107) carry REQ-003/004/006/008/009/010/011 end-to-end, and the gate-15 driven
capture of the demo fixture carries the pixels (REQ-001/004). If the capture is env-blocked (locked machine),
the standing fallback applies (`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`):
units + headless + mechanism, capture re-verified when unlocked.

## Phase Plan
- **P2 Design** — confirm D1 (right-dock revival) against the drawn card layout; settle D-OPEN-1/2; the
  `fleet_rail.rs` fn signatures against #367's landed types (BLOCK here if #367 has not merged); the card +
  question-card + empty-state layout; the fixture's exact seat set; the test plan per REQ.
- **P3 Implement** — `fleet_rail.rs` (pure), the app.rs shim arm + state + verbs, `dock_title`, the fixture.
- **P3.5 Inspect** — independent critics vs the diff: genericity (Forge-string grep), the dirty-flag path,
  per-card gating (D7), chip fills vs borders, order preservation, the mutants::skip-detach trap
  (`cargo mutants --list` after any fn insertion near a skipped shim).
- **P4 Validate** — write + RUN the units (cov/MSI 100 on fleet_rail.rs) + the headless drives; the gate-15
  capture of the demo fixture; full gate green.
- **P5 Complete** — archive, AAR capture, close #369.
