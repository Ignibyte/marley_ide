# Fleet rail — read-only FleetSnapshot render (#369) — Notes

- **Forge ticket:** #369 (279d5673-899d-466a-be94-d6c3417c9251)
- **AAR:** ca93ce71-629a-42b1-a6d0-438b16727ebf
- **Local ticket doc:** docs/planning/tickets/open/TICKET-369-fleet-rail-readonly.md
- **Pipeline spec:** 369-fleet-rail-readonly.spec.md

## Phase 1 — Plan
- **Request:** #369 (feature, M23, sprint "M23 — Fleet Control Plane: Layer 1") — the shell's first fleet
  surface: a READ-ONLY rail rendering `marley_fleet::FleetSnapshot` (#367, a concurrent sibling that owns the
  pure model — `Session`/`FleetSnapshot`/attention/staleness). Per-seat cards: state chips (closed vocabulary,
  Error ≠ Idle), opaque label chips (generic — never interpreted), render-only question-cards for Waiting
  seats, staleness dimming + last_event relative time, attention ordering preserved from the snapshot. Zero
  writes; fixture-provable (Layer 0 is external/unshipped; the ② live feed may not exist yet).
- **Classification / tier:** feature; full pipeline. A render-surface ticket on the coverage-excluded app.rs →
  the #307 pattern governs the whole test story (pure seams cov/MSI 100; headless drives + gate-15 capture
  prove the excluded arms).
- **Forge recall (§18.3)** — the standing rules baked into the spec (D2/D3/D5/D6/D7 + Inspect checklist):
  - `PR-claude-pump-state-change-must-set-dirty-to-repaint` — the fixture-feed verb replaces the snapshot
    outside the render loop → MUST set dirty or the rail sits stale on an idle frame (REQ-011).
  - `PR-claude-per-pane-render-affordance-must-gate-on-is-focused` — the per-item-loop form: each card's
    dim/question-card gates on THAT seat's data, never global state (D7).
  - `PR-claude-selection-bg-distinct-from-container` — any card wash must differ from the panel surface
    (the #219 invisible surface-on-surface lesson; `rail_highlight` app.rs:976 is the shipped fix).
  - `PR-claude-gpui-no-box-sizing-border-inflates` — state chips use FILLS, not borders (the #220 lesson).
  - `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism` — the capture fallback if the
    machine is locked at Validate.
  - The mutants::skip-detach trap — inserting a fn above a skipped shim rebinds the attribute; re-run
    `cargo mutants --list` after edits near shims (P3.5 checklist).
  - notify.rs:22 precedent — pure fns take u64 SECONDS, never `Instant`/`Duration` (the rel-time helper).
- **Discovery (the landing zone + idioms; file:line evidence):**
  - **Proposed landing zone — revive the RIGHT dock slot.** `docks: [DockState; 2]` (app.rs:163); the right
    slot is permanently Closed since #153/#171 (app.rs:1932–1934, comment app.rs:8033) yet the whole substrate
    survives: `region_widths(window, left, right, dock)` is pure + tested for all four combinations
    (layout.rs:108; test `region_widths_all_four_combinations` layout.rs:406), the render still consults
    `self.dock(DockSide::Right)` (app.rs:15113–15118), and `dock_panel` renders either side incl. the right's
    inner `border_l_1` (app.rs:945, 965). `dock_title(Right)` = a stale "Details" (layout.rs:68 — the cockpit
    left the dock in #153) → becomes "Fleet". Revival is configuration, not construction.
  - **Alternatives rejected (final call in P2):** the LEFT dock is the Workspace→Project→Tab NAVIGATOR
    (app.rs:15119–15122) — observation crammed into navigation; a full-screen cockpit tab (the #153 pattern,
    `cockpit_body` app.rs:4844) breaks glanceability-beside-work (fleet-control-plane.md §3: cockpit window +
    radio, side by side) — and `RightSection::Agents` (app.rs:4912) already means LOCAL launched agent CLI
    panes (`self.agents`), a different object; the spec Scope-Outs unifying them.
  - **Verb + toggle wiring:** `"toggle-right-dock"` exists but re-routes to the Details cockpit tab
    (app.rs:7905–7907); `toggle_left_dock` (app.rs:8035) is the persist-on-toggle model → D-OPEN-2.
  - **Chip/badge/empty idioms to reuse:** #222 keycap chip shape (ui_components/src/render/keyboard_shortcut.rs:9–11);
    #203 badge state colors `success`/`danger` (ui_components/src/lib.rs:56/58; render app.rs:15361);
    `rail_highlight` wash + its both-themes distinctness test (app.rs:976, app.rs:18202);
    cockpit `placeholder(hint)` + `agents_empty_hint` (app.rs:4902, 4916); `fmt_duration` (agent_view.rs:108,
    pure + tested) for the relative-time tiering; the rail wheel-scroll idiom `rail_scroll` +
    `scroll_steps`/`scroll_code` (app.rs:15188–15203).
  - **Headless reachability:** `dispatch_for_test` (app.rs:2107) drives real verbs; `cockpit_commands`
    (app.rs:8061) + `dispatch_action` (app.rs:7364) register/dispatch palette verbs — the `fleet-demo-feed`
    fixture verb rides both (headless + live capture).
  - **uniform_list:** shipped in the editor tab (app.rs:5317) — available but wants uniform row heights;
    question-cards are variable-height and fleets are ~4–12 seats → plain column leans (D-OPEN-1).
  - **No `marley_fleet` crate exists yet** (crates/ listing) — confirms #367 runs concurrently; P2 blocks on
    its types if unmerged.
- **Prior-art (§20):** Reference = N/A, Marley-specific (fleet-control-plane.md §7.2 + orchestration-shell.md
  §3/§12 are the design source). Verified: docs/warp_architecture/subsystems/04-agent-ai-mcp.md maps Warp's
  Agent Mode as a cloud conversation-loop client (SSE multi-agent transport, input classifier) — no
  fleet-of-seats rail in our behavior maps; docs/zed_architecture/ holds crate maps only (no collab-panel
  behavior doc; Zed's collab panel is a people rail and GPL-source-off-limits anyway). Published: nothing to
  adopt. Deps + own code: the dock substrate + chip/badge/empty/scroll idioms above — the sweep settled D1.
- **Decisions:** D1 right-dock revival (P2 confirms) · D2 #307-pattern totality (every decision a pure fn in
  fleet_rail.rs, cov/MSI 100; shim only calls) · D3 chips-as-fills + Error≠Idle both themes · D4 snapshot
  order verbatim, zero rail-side derivation, genericity tested with non-Forge labels · D5 fixture verb +
  dirty · D6 inert question-cards with an honest hint · D7 per-card gating. Open: D-OPEN-1 (scroll strategy),
  D-OPEN-2 (verb reclaim vs new + dock persistence).

## Phase 2 — Design

### Architecture / approach
The shell's first fleet surface. A NEW pure module `crates/marley_app/src/fleet_rail.rs` holds every
render DECISION (cov/MSI 100); the `app.rs` render arm is a `mutants::skip` shim that only calls the
pure fns and lays out gpui `Div`s (app.rs is coverage-excluded — the #307 pattern). The rail lives in
the revived RIGHT dock slot (D1). It renders `marley_fleet::FleetSnapshot` (#367, SHIPPED) + the
`attention`/`is_stale` derivations. Add `marley_fleet = { path = "../marley_fleet" }` to
crates/marley_app/Cargo.toml (marley_app currently deps marley_forge_client but not marley_fleet
directly). §20 N/A-Marley-specific confirmed (no Warp/Zed fleet-rail analog; the design source is
fleet-control-plane §7.2 + orchestration-shell §3/§12; only our own gpui/dock/chip idioms adopted).

### Three refinements the SHIPPED #367 model forces (flag for inspect — the spec was drafted before
### #367 finalized its API)
- **R1 (refines D4/REQ-008):** #367's `FleetSnapshot::seats()` is **first-seen order** (its D9), and
  `attention()` is a SEPARATE read-time derivation returning only the attention-worthy seats — the
  snapshot is NOT itself attention-ordered as D4's wording assumed. Resolution: the rail renders
  `seats()` in **first-seen order VERBATIM** (honoring REQ-008's "never re-sort" — a stable rail with
  no jumps); attention prominence is carried by per-seat TREATMENT (an Error seat's red chip, a Waiting
  seat's question card, a stale seat's dim), not by reordering. An attention-SORTED rail (worthy seats
  floated to the top via `attention()`) is a deliberate v1 deferral (a follow-up), because D4/REQ-008
  locked first-seen-verbatim.
- **R2 (refines D6/REQ-006 staleness):** #367 stores **no** staleness flag on the seat — staleness is a
  read-time pure fn `is_stale(session, now_ms, stale_after_ms)` (its D3: no clock in the crate). So the
  masked app arm (which owns the clock read) computes `is_stale(seat, now_ms, STALE_AFTER_MS)` per seat
  and passes the resulting **bool** to the pure card fn; `fleet_rail.rs` never reads a clock. "The rail
  applies only the flag" holds — the flag is #367's `is_stale`, evaluated in the shim, consumed as data.
- **R3 (staleness threshold):** #367 deliberately ships NO default `stale_after_ms` (its
  D-OPEN-STALE-DEFAULT). The rail owns its display threshold as a documented `fleet_rail` const
  `STALE_AFTER_MS` (a Schelling value, e.g. 30_000 = 30s — revisit when L0 heartbeat cadence is known);
  it is injected into `is_stale`/`attention`, never hard-read.

### D-OPEN forks — SETTLED
- **D-OPEN-1 (scroll) → plain flex column + the existing `rail_scroll` wheel idiom.** Fleets are ~4–12
  variable-height cards (question-cards vary); `uniform_list` wants uniform heights and would force
  question-card collapse. Evidence: the spec's own Prior-art 3 + the shipped `rail_scroll`.
- **D-OPEN-2 (toggle verb + persistence) → a NEW `toggle-fleet-rail` verb; persist via the shipped
  dock-persistence idiom.** Rationale: `"toggle-right-dock"` (⌘⇧B) is re-routed to the Details cockpit
  tab since #153 — reclaiming it would change a shipped keybinding's behavior; a new verb keeps ⌘⇧B
  intact and gives the rail its own affordance. The rail's open state persists like `toggle_left_dock`'s
  `persist_dock_*` model (final wiring confirmed against the Explore map).

### Pure `fleet_rail.rs` fn inventory (all cov/MSI 100 — decisions only, no gpui)
- `pub const STALE_AFTER_MS: u64` (R3) + a `state_label(State) -> &'static str` (the 6 chip labels).
- `enum ChipTone { Working, Idle, Waiting, Error, Done, Starting }`-style OR a `chip_tone(State) ->
  ChipTone` mapping to a semantic tone the shim resolves to a ThemeColors FILL (D3 fills-not-borders);
  the PURE part is the State→tone map (all 6 distinct; Error's tone ≠ Idle's). The shim resolves tone →
  color for the active theme; a unit asserts the tone map is injective + Error≠Idle (theme-distinctness
  is the shim's #219-style test, but the pure tone map guarantees they differ).
- `transport_hint(Option<Transport>) -> Option<&'static str>` ("tmux"/"bridge"/"local"/None).
- `label_chips(&BTreeMap<String,String>) -> Vec<(String,String)>` — verbatim `key: value` in the map's
  deterministic (sorted) order; ZERO key interpretation (D4/REQ-005 genericity).
- `question_card(&Session) -> Option<&Question>` — Some iff `state == Waiting && question.is_some()`
  (D6 render-only; the shim draws prompt + non-interactive option chips + the inert hint) (REQ-003).
- `is_dimmed(is_stale: bool) -> bool` = the R2 bool passthrough (the pure decision consumes ONLY the
  flag; REQ-006) — trivial but it's the seam the shim gates on (D7 per-card).
- `rel_time(age_secs: u64) -> String` = `format!("{} ago", agent_view::fmt_duration(age_secs))` (R-time
  reuse; REQ-007). The shim computes `age_secs = now_ms.saturating_sub(seat.last_event_ms) / 1000`.
- `empty_hint(seat_count: usize) -> Option<&'static str>` — Some(muted hint) when 0 seats (REQ-010).
- `seat_order<'a>(snapshot: &'a FleetSnapshot) -> &'a [Session]` = `snapshot.seats()` verbatim (R1;
  REQ-008) — a thin pure accessor that documents the "never re-sort" contract as a named seam a test
  pins.

### The demo fixture (D5 — the in-crate `FleetSnapshot`, built via `marley_fleet::reduce`)
A `fleet_rail::demo_snapshot() -> FleetSnapshot` (pure, in fleet_rail.rs so it's reusable by the unit
tests + the fixture-feed verb) folding a `SessionEvent` stream that yields ~6 seats exercising every
render path: one per state (Starting/Working/Idle/Waiting/Error/Done), the Waiting one carrying a
`Question { prompt, options, context_refs }`, one seat made STALE (a very old `last_event_ms` so
`is_stale` fires against `now`), opaque labels that are **deliberately non-Forge** ("job: build-42",
"lane: alpha") to prove genericity (D4/REQ-005), and a mix of transports (tmux/bridge/local/None). The
fixture is authoritative for the headless drive + the gate-15 capture.

### Regression Test Plan (every REQ → pure unit + headless drive + capture)
| REQ | Test | Asserts |
|---|---|---|
| REQ-001 | `fleet_rail::t369_card_projection` + capture | title + `state_label` + `transport_hint` present per seat |
| REQ-002 | `fleet_rail::t369_chip_tone_all_states_distinct` | `chip_tone` injective over the 6 states; Error tone ≠ Idle tone (the shim's #219-style both-themes color test rides this) |
| REQ-003 | `fleet_rail::t369_question_card_iff_waiting_with_q` + headless | `question_card` Some for Waiting+question, None for every other state and Waiting-without-question |
| REQ-004 | headless `t369_question_option_click_is_inert` + capture review | clicking an option changes no app state (render-only, D6) |
| REQ-005 | `fleet_rail::t369_label_chips_verbatim_generic` + validate grep | non-Forge `key: value` rendered verbatim, sorted; grep: no Forge constant in fleet_rail.rs/render arm |
| REQ-006 | `fleet_rail::t369_is_dimmed_takes_only_the_flag` + headless | `is_dimmed(true/false)` == the flag; the shim's per-seat gate reads THIS seat's `is_stale` (D7) |
| REQ-007 | `fleet_rail::t369_rel_time` | `rel_time(45)`="45s ago", `rel_time(134)`="2m14s ago" (fmt_duration reuse) |
| REQ-008 | `fleet_rail::t369_seat_order_verbatim` + headless | `seat_order` == `snapshot.seats()` (first-seen; a fixture whose attention order ≠ insertion order proves no re-sort) |
| REQ-009 | headless `t369_toggle_insets_center` | toggle-fleet-rail open → `region_widths` insets center by DOCK_WIDTH, left rail/files/grid still work; closed → restored |
| REQ-010 | `fleet_rail::t369_empty_hint` + headless | `empty_hint(0)`=Some, `empty_hint(n>0)`=None; empty fixture → the hint renders, no crash/blank |
| REQ-011 | headless `t369_demo_feed_sets_dirty_repaints` | dispatch fleet-demo-feed → the snapshot field is set AND dirty/repaint requested → next frame shows it |

**Testing boundary:** fleet_rail.rs pure → cov/MSI 100 (chip_tone, question_card, is_dimmed, rel_time,
empty_hint, label_chips, transport_hint, seat_order, state_label, demo_snapshot). The app.rs arm +
toggle/feed verbs are coverage-excluded mutants::skip shims → the `*_headless` drives via
`dispatch_for_test` carry REQ-003/004/006/008/009/010/011; the gate-15 driven capture of `demo_snapshot`
carries the pixels (REQ-001/004). **Driven capture MANDATORY at validate** (bundle-app → drive
toggle-fleet-rail + fleet-demo-feed → screencapture → READ the png: assert cards stack with distinct
state chips, the question-card renders, a stale card dims). If env-blocked (chad at the machine / locked
screen — the memory warns he may be back), the standing fallback: units + headless + mechanism, capture
re-verified when free (`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`). No
trybuild (no type-contract); the invariants are runtime render-decision properties.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/Cargo.toml` | `+ marley_fleet = { path = "../marley_fleet" }` (the rail renders FleetSnapshot + attention/is_stale) |
| `crates/marley_app/src/fleet_rail.rs` | NEW, PURE — the fn inventory above + `demo_snapshot()` + `#[cfg(test)]` (cov/MSI 100) |
| `crates/marley_app/src/lib.rs` | `mod fleet_rail;` |
| `crates/marley_app/src/app.rs` | (shim, `mutants::skip`) an `Option<FleetSnapshot>` field on the app (+ a `fleet_rail_open` bool if the dock's Open/Closed state isn't reused); the right-dock body renders the fleet rail via the pure fns (plain flex column + `rail_scroll`, D-OPEN-1); a `toggle-fleet-rail` verb (new, D-OPEN-2) opening the right dock slot + persisting; a `fleet-demo-feed` verb (registered in `cockpit_commands`, dispatchable via `dispatch_for_test`) that installs `demo_snapshot()` + sets dirty (REQ-011). WATCH the mutants-skip detach trap when inserting fns near the shims (re-run `cargo mutants --list -f app.rs` after) |
| `crates/marley_app/src/layout.rs` | `dock_title(DockSide::Right)`: "Details" → "Fleet" (pure, already unit-tested at layout.rs:373 — update the test's expected string) |
| `crates/marley_app/src/settings.rs` | a `DockRight` setting + `persist_dock_right` (mirror `DockLeft`/`persist_dock_left` settings.rs:484) so the fleet dock's open state persists; boot restores it (mirror app.rs:1934) — pure round-trip, cov/MSI 100 |

### Render-arm wiring (CONFIRMED by the Explore map — exact current lines)
**The critical question is settled: `docks[1]` and `RightSection` are FULLY INDEPENDENT — no
collision.** The right dock slot is permanently `Closed` at boot with NO `dock_panel(Right)` render call
anywhere (`region_widths` yields 0 for it today) — genuinely dormant free real estate. `RightSection`
(Details/Agents/Forge) is a separate full-screen CENTER cockpit-TAB concept rendered by `cockpit_body`;
the `"toggle-right-dock"` verb is a misnomer that opens the Details TAB, not the dock. So reviving the
slot is 3 clean edits:
1. **The render arm** — at the app.rs gap after the left-dock block (~app.rs:15489, where a comment
   notes the right dock retired): `if regions.right > 0.0 { root = root.child(dock_panel(DockSide::Right,
   regions.left + regions.center, content_top, regions.right, content_h, &colors, fleet_body)); }`.
   `dock_panel(side, x, y, w, h, colors, content: gpui::Div)` (app.rs:944-971) places `content` (the
   fleet rail body) as its last child under a `caption_header(dock_title(Right))`; the right side already
   gets `border_l_1`. `region_widths` (layout.rs:108) + `DOCK_WIDTH` (app.rs:551 = 220.0) already model
   the right region (yields 0 when Closed, DOCK_WIDTH when Open). The `fleet_body` Div is built by the
   shim from the pure `fleet_rail.rs` fns (plain flex column + a fleet-own scroll field pair mirroring
   `rail_scroll`/`scroll_steps`/`scroll_code`; the `placeholder` empty-state is a local closure like
   cockpit_body's app.rs:4902).
2. **Open the slot + toggle** — a NEW `"toggle-fleet-dock"` verb (dispatch_action arm ~app.rs:7905+;
   a `cockpit_commands` entry app.rs:8061 with the next free `CommandId` for palette reach) that flips
   `docks[1]` (mirror `toggle_left_dock` app.rs:8035 exactly) + persists via a NEW `persist_dock_right` /
   `DockRight` setting (parallel to `persist_dock_left` settings.rs:484 / `DockLeft`) — boot restores it
   (mirror app.rs:1934, which currently hard-Closes the right slot). Toggle listener → `cx.notify()`.
3. **`dock_title`** — layout.rs:68 Right arm "Details" → "Fleet"; update the `dock_title_by_side` test
   (layout.rs:373).
- **The `fleet-demo-feed` verb** — a dispatch_action arm + a cockpit_commands entry that sets
  `self.fleet = Some(fleet_rail::demo_snapshot())` and calls `cx.notify()` (the synchronous-dispatch
  dirty idiom the map confirmed — REQ-011). Reachable headlessly via `dispatch_for_test` (app.rs:2107).
- **Manifest addition:** the `DockRight` setting + `persist_dock_right` in settings.rs (mirror
  `DockLeft`), and the two fleet scroll fields on the app struct.

### Risks / decisions
- **R1's first-seen order** (not attention-sorted) is the spec-faithful choice (REQ-008 "never
  re-sort"); prominence is via per-seat treatment. An attention-sorted rail is a named follow-up.
- **R2/R3 staleness** = the shim computes `is_stale(seat, now_ms, STALE_AFTER_MS)` (a #367 pure fn) and
  passes the bool in; no clock in fleet_rail.rs; the threshold is a rail const (revisit at L0 cadence).
- **The demo-feed dirty path** is load-bearing (REQ-011) — the verb must set dirty/request a repaint
  (the #203 `PR-…-pump-state-change-must-set-dirty-to-repaint` lesson).
- **Driven capture** may be env-blocked (chad back at the machine per the memory) → the documented
  fallback stands; validate states it honestly.

## Phase 3 — Implement
- **Built:**
  - `Cargo.toml` + `marley_fleet` dep; `lib.rs` `mod fleet_rail`.
  - `fleet_rail.rs` (NEW, PURE, 26 mutants): `STALE_AFTER_MS`, `state_label`, `ChipTone` +
    `chip_tone` (injective, Error≠Idle), `transport_hint`, `label_chips` (verbatim sorted),
    `question_card` (Some iff Waiting+question), `is_dimmed` (flag passthrough), `rel_time` (reuses
    `fmt_duration`), `empty_hint`, `seat_order` (verbatim), `demo_snapshot(now_ms)` (6 seats, all
    states, a Waiting+question, a stale seat, non-Forge labels, mixed transports).
  - `app.rs` (all shims `mutants::skip`, confirmed NOT in the mutant list — detach trap held): the
    `fleet_snapshot: Option<FleetSnapshot>` field + init; `now_epoch_ms` (the clock read, in the shim
    only); `fleet_rail_body` + `fleet_seat_card` (the rail render — state chip FILL via `chip_tone`→
    color, verbatim label chips, the inert question-card standing off `background` with non-interactive
    option chips + a muted "answered in Layer 2" hint, stale dim on THIS seat's `is_stale`, rel-time,
    transport hint, empty-state); the right-dock render arm at the ~15489 gap; `toggle_fleet_dock`;
    the `toggle-fleet-dock` + `fleet-demo-feed` dispatch_action arms; two `cockpit_commands` entries
    (CommandId 28/29).
  - `palette.rs`: `action_for_command` CommandId(28)→`toggle-fleet-dock`, (29)→`fleet-demo-feed`.
  - `layout.rs`: `dock_title(Right)` "Details"→"Fleet" + the `dock_title_by_side` test.
- **Checks:** `cargo fmt`; `cargo clippy -p marley --all-targets` exit 0; `cargo check --workspace` green.
- **Deviations / findings for inspect:**
  - **⚠️ Naming collision (FLAG):** app.rs ALREADY has a `fleet_open` ⌘⇧E overlay (#68) that lists LOCAL
    launched agents (`self.agents`) — a DIFFERENT object from the M23 control-plane fleet. I used
    distinct names (`fleet_snapshot`/`toggle-fleet-dock`/`fleet-demo-feed`) so there is no code clash,
    but there are now TWO "Fleet" surfaces (the legacy local-agents overlay + this control-plane dock
    rail). Per the spec's Scope-Out, unifying them is deliberately deferred — but the twin-"Fleet"
    naming is a UX smell worth surfacing to chad + a candidate follow-up.
  - **Deviation (session-only toggle):** dropped the `DockRight`/`persist_dock_right` persistence to keep
    v1 focused (the run's scope) — the rail boots Closed, the toggle is session-only. Persisting the open
    state is a bounded follow-up. (Refines D-OPEN-2's "persist" → v1 session-only.)
  - **Deviation (no wheel-scroll):** v1 is a plain flex column (the ~6–12-seat fleets fit); overflow
    wheel-scroll via the `rail_scroll` idiom is a follow-up. (Refines D-OPEN-1 → column now, scroll later.)
  - No tests here (Phase-4); production + doc only.

## Phase 3.5 — Inspect
**2 adversarial critics spawned** (fleet_rail correctness/mutation-readiness; genericity/masked-render/
naming-collision) — both RAN but their reports were not delivered in-session (a harness hiccup: after a
SendMessage poll they resumed and their completion notifications did not surface; I did not read the
transcripts per the overflow rule). **My OWN independent verification covered both lenses thoroughly and
is the inspect basis** (§18.1 satisfied by the spawned critics + this cross-check).

### Findings ledger
| # | Lens | Sev | Finding | Verdict | Resolution |
|---|---|---|---|---|---|
| F1 | genericity | Low | the demo Error seat was named "gate agent"/"dev-5/gate" — a benign demo VALUE, but it cluttered the D4/REQ-005 forge-vocab grep | REAL (grep-noise) | **Fixed** — renamed to "api agent"/"dev-5/api"; the grep now shows only doc-comment "gate" (English "the question-card gate" + "gate-15 capture") |
| F2 | naming | **Flag** | app.rs already has a `fleet_open` ⌘⇧E overlay (#68) listing LOCAL agents — now ALSO a control-plane "Fleet" dock rail. TWO "Fleet" surfaces | REAL, but no code clash (distinct field/verb names) | **Flagged for chad** + `PR-claude-grep-existing-name-before-adding-a-ui-surface-001`; unifying them is the spec's scope-out → a candidate follow-up. Not a defect |
| — | deviations | INFO | v1 = session-only toggle (no `DockRight` persistence) + no wheel-scroll (plain column) | Deliberate scope choices | Recorded; persistence + overflow-scroll are bounded follow-ups |

### Independent verification (both lenses, concretely)
- **Genericity (D4/REQ-005):** `rg` on fleet_rail.rs → no Forge vocab as a code identifier or matched
  label key (only 2 doc-comment "gate" + the demo VALUES, now non-Forge); `label_chips` renders keys
  VERBATIM (never matches a key); the render arm interprets no label key.
- **Prevention rules (by inspection of the app.rs render arm):** state chip is a FILL (`.bg(tone)`, app.rs
  ~1013 — not a border, #220); the question-card `.bg(colors.background)` (~1076) is DISTINCT from the
  dock panel `.bg(colors.surface)` (~1116) and from its option chips `.bg(colors.surface)` (~1062) —
  `background != surface` confirmed in both theme families (#219); the option chips carry NO
  `on_mouse_down` (the only on_mouse_down hits nearby are the #68 agent overlay + the #275 banner, not the
  fleet cards) → inert (D6); the stale dim reads THIS seat's `is_stale(seat, now, STALE_AFTER_MS)`
  per-seat in `fleet_seat_card` (D7); the demo-feed repaint rides the palette/keybinding dispatch call
  site's notify (REQ-011 — dispatch_action has no cx by design).
- **Correctness + mutation-readiness:** the 26-mutant kill-map is written into the Phase-4 section
  (~22 viable exact-value-killable, 4 auto-unviable Default-body on ChipTone/Question/Session/
  SessionEvent). `chip_tone` is injective + Error≠Idle; `question_card` gates on (Waiting, Some), not
  bare `question.is_some()`; `seat_order` returns `seats()` verbatim; `demo_snapshot(now)` yields the 6
  states + a stale seat + non-Forge labels. No unwrap/expect/panic on the render path.
- **Detach trap:** the new app.rs fleet fns are all `mutants::skip` and confirmed absent from the app.rs
  mutant list (the skip set did not shift).

### Result
Zero code defects; F1 fixed; F2 flagged + a prevention rule (`e89e35a3`); the deviations recorded.
`cargo check`/`clippy` green after the fix.

## Phase 4 — Validate — **PASS** (resumed after the emergency reboot; the pure units were already written+green pre-reboot)

### Tests added / present (every REQ → a real test)
**Pure units in `fleet_rail.rs` (10, all green — cov/MSI 100):** `t369_req002_state_label_all_six_distinct`,
`t369_req002_chip_tone_each_arm_error_ne_idle` (each arm explicit + Error≠Idle — MSI is blind to fieldless-enum
arm swaps), `t369_req001_transport_hint_all_arms`, `t369_req005_label_chips_verbatim_sorted`,
`t369_req003_question_card_gate` (Waiting+q→Some; Waiting-no-q→None; Working+q→None — the gate is state AND
question), `t369_req006_is_dimmed_both_directions`, `t369_req007_rel_time`, `t369_req010_empty_hint`,
`t369_req008_seat_order_verbatim_first_seen` (ids in insertion order, NOT attention order — proves no re-sort),
`t369_demo_snapshot_all_states_question_and_stale`.
**Headless drives added this phase in `headless_drive.rs` (2, gpui test-support harness — REAL RootView, no capture):**
- `fleet_dock_toggle_insets_center_and_empty_renders_headless` — REQ-009/010: boots Closed; `toggle-fleet-dock`
  → `dock(Right)==Open`, `region_widths` insets center by `DOCK_WIDTH` (left nav dock untouched); a real
  `run_until_parked` render pass draws the EMPTY `fleet_rail_body(None,…)` with no panic; second toggle → Closed.
- `fleet_demo_feed_installs_orders_and_renders_headless` — REQ-011/008: `fleet-demo-feed` installs the snapshot
  (`None`→`Some`, 6 seats); `seat_order` == first-seen ids verbatim; dock opened + `run_until_parked` renders the
  POPULATED body (question-card + stale seat) with no panic.
**Palette coverage fix:** `action_for_command_maps_every_row` extended to assert `CommandId(28)→"toggle-fleet-dock"`
and `(29)→"fleet-demo-feed"` (the two new arms were otherwise uncovered → would fail cov 100 + diff-mutation).
**Test-support seams added:** `RootView::fleet_snapshot_for_test()` (`#[cfg(test)]`, mirrors `font_size_for_test`);
`DOCK_WIDTH` → `pub(crate)` so the drive asserts the inset faithfully.

### Runs (actual)
- `cargo nextest run -p marley fleet` → **14 passed** (10 units + 2 drives + 2 pre-existing agent_view fleet tests), 0 failed.
- `cargo nextest run --workspace` → **1825 passed, 0 failed, 5 skipped** — no regression from the app.rs/palette edits.
- **Mutation** `cargo mutants -f fleet_rail.rs --package marley` → **26 mutants: 22 caught, 4 unviable, 0 missed = MSI 100.**
  The 4 unviable are exactly the `Default`-body mutants on the no-`Default` types (verified derives): `chip_tone`
  (ChipTone), `question_card` (Question), `seat_order` (Session), `upsert` (SessionEvent). `demo_snapshot`'s
  `FleetSnapshot::default()` mutant IS viable (FleetSnapshot derives Default) and is caught by the `len()==6` asserts.
- **Gate** `scripts/gates.sh --diff` (commit-valid receipt; cov floor 100 / MSI floor 100) → **green after
  three self-caught gaps** (the gate runs ALL 15 gates and reports at the end — not fail-fast):
  1. **Palette-arm coverage gap** — the new `CommandId(28)/(29)` arms were uncovered → extended
     `action_for_command_maps_every_row`. → `PR-claude-new-palette-arm-extend-the-exhaustive-map-test`.
  2. **fmt drift** — the interrupted Phase-3 (emergency reboot) left `fleet_rail.rs` unformatted (compiled +
     tested fine, so it hid until the gate's `cargo fmt --check`); `cargo fmt --all` source-fixed it. →
     `PR-claude-resume-after-interrupt-reverify-fmt-and-gate`.
  3. **Coverage-100 closure gap** — the drive's boot-state check `fleet_snapshot_for_test().map(|s| …)` left
     the mapping closure UNEXECUTED (Option::map short-circuits on the boot-time `None`) → 1 missed line + 1
     missed function in `headless_drive.rs`. Fixed to a closure-free `.is_some()`. Targeted `llvm-cov`
     re-confirmed `headless_drive.rs`/`fleet_rail.rs`/`palette.rs`/`layout.rs` all 100% lines, TOTAL 100%. →
     `PR-claude-none-state-assert-via-is_some-not-map-else-closure-uncovered`.
  Final gate: **gate:5 mutation 29 caught / 0 missed = MSI 100; gate:4 coverage 100% lines; gate:6 miri +
  gate:15 visual/AX PASS; all 15 green** (receipt written for the current tree).

### Driven capture — ENV-BLOCKED (documented fallback, `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`)
The machine is unlocked but **chad is actively at the terminal (frontmost = iTerm2)**; the memory's standing
instruction is explicit — do NOT drive synthetic input, it hits HIS frontmost window (keyboard CGEvents only
reach the frontmost app, so driving the rail would require focus-stealing his session). This is **not** the old
"masked, chad-verifies" punt: the two headless drives now **execute** `fleet_rail_body` for both the empty and the
populated snapshot (a real `run_until_parked` render pass) — only the rasterized pixels are deferred. The pixel
properties are otherwise pinned by the pure `chip_tone` injectivity + Error≠Idle test and the Inspect trace
(chip = FILL #220; card `.bg(background)` ≠ dock `.bg(surface)` #219; options carry no `on_mouse_down` → inert D6;
stale dim gates per-seat D7). **Re-verify the capture when the machine is free** (bundle → toggle-fleet-dock +
fleet-demo-feed → screencapture → READ; ~1 min, no ticket).

**Status: Phase 4 — Validate PASS** — `scripts/gates.sh --diff` GATE GREEN (15/15, 0 failed; coverage 100%
lines, mutation 29 caught / 0 missed = MSI 100, miri + visual/AX pass); commit receipt written. Ready for
Phase 5 — Complete.

## Phase 5 — Complete
