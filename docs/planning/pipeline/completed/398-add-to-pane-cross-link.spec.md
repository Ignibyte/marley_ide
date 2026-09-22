---
pipeline_id: 7144a1b0-d110-4233-a386-d4721d1aec22
ticket: forge#398 (7aa54235-760c-4191-a5e6-12a5b65dfa7e) · local docs/planning/tickets/open/TICKET-398-add-to-pane-cross-link.md
aar_id: bad06f88-7368-44bf-bd67-58947689e178
status: Phase 5 — Complete PASS
title: Add-anything-to-a-pane + the ContentId cross-link — one instance, many views goes live (the #388 slice-5)
type: feature
milestone: M28
references:
  - docs/marley_architecture/pane-composition-model.md
  - docs/planning/tickets/open/TICKET-396-terminals-onto-registry.md
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/context_menu.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/palette.rs
  - crates/marley_app/src/app.rs
  - docs/planning/pipeline/completed/warp-workflows.notes.md
  - marley-web/docs/MARLEY-PARITY.md
  - marley-web/artifacts/marley-ide/src/components/LeftRail.tsx
  - marley-web/artifacts/marley-ide/src/components/ContextMenu.tsx
  - marley-web/artifacts/marley-ide/src/components/views/SplitTerminalView.tsx
---

## Title
The #388 train's slice-5 — the registry payoff. #394 shipped the refcounted lifecycle
(`content_registry.rs`: `insert`/`acquire_view`/`release_view`-returns-owned-on-last-drop, cov/MSI 100);
#396 moves terminal ownership onto it; #397 moves editors. Through all three, **every entry sits at
view_count 1** — the refcount machinery is proven but unexercised in anger. This slice makes it real:

- **The add gesture:** a verb/command set to drop an OPEN `ContentId` into a split cell — a palette flow
  plus Section/Pane context-menu rows — so any section's content becomes a Pane view of it. Adding an open
  terminal to a split yields TWO cells rendering ONE session (shared PTY/scrollback); adding an open editor
  yields one Buffer, two views (the #397 per-view caret/scroll split doing its job). The dispatch is
  `acquire_view` + the existing split mechanics (`split_focused_pane` app.rs:6827 / the #246 split-to-file
  shape), never a second instance.
- **The cross-link half #390 explicitly deferred** ("the `ContentId`-per-cell label + home-section
  cross-link is still deferred to slice 5 (needs terminals+editors registered)" — pane-composition-model.md
  Q5.4): Panes cell rows carry a ContentId-derived label — today they read the positional
  `"pane {k+1}"` (tabs.rs:868) under a `"PANE n"` arrangement header (tabs.rs:857) — and the
  Editor/Terminal home sections cross-list content that also lives in a Pane (chad's model A: "the original
  stays open"; two navigator entries, one instance). This includes pane-ONLY content (e.g. a #246
  split-opened file) which today has NO home-section row at all.

View-count rules per the #394 registry: acquire on add, release on close, last-view drop returns ownership
to the caller — terminals keep the ~600ms off-thread reap (#396's contract, byte-identical; slice-5 only
makes the N>1 arm reachable).

## Scope
### In
- **The add-to-pane verb set (surface = D-OPEN-GESTURE-SURFACE):** new `MenuAction`/`SectionAction`-style
  arms + rows in the existing const tables (the context_menu.rs:111-117 documented extension point: "one
  row in the relevant table + one arm — nothing else") and a palette entry in a new dynamic id range above
  `WORKFLOW_BASE` (app.rs:620), resolved via the shared `dynamic_command_index` (palette.rs:183-187). The
  target rides the menu KIND (the #175 pattern, context_menu.rs:63-71) so a mid-menu focus/pump shift never
  retargets the add. Dispatch = activate-first #174 idiom + #387-F1 persist-the-switch, exactly as
  `dispatch_section_verb` (app.rs:6772-6820) does today.
- **Acquire-on-add / release-on-close symmetry:** the add path calls `acquire_view` (content_registry.rs:66)
  exactly once and mounts the id in the new cell; every existing close path keeps routing through the #396
  `release_view` contract unchanged — closing one view of shared content now genuinely decrements and the
  instance survives to the last close.
- **The label seam:** a pure ContentId→(kind, title)→row-label derivation fn (cov/MSI 100); `rail_rows`
  (tabs.rs:786) stays pure over the model — the render shim supplies registry-derived inputs (exact
  plumbing Phase 2, under D-OPEN-LABEL-SHAPE). Labels derive fresh each render (the #390 discipline: no
  stored identity, renumber/relabel on change).
- **The cross-list seam:** a pure computation of home-section extra rows (content mounted in cells, keyed
  by home `RailSection`, minus what a tab row already lists), presentation-only; activation reuses the
  existing coordinate focus path (`jump_to_pane`, app.rs:7024). Marker shape = D-OPEN-CROSSLIST-MARKER.
- **Verb enablement:** a pure fn deciding when add-to-pane verbs are present/inert — no open addable
  content, no splittable focused grid (cockpit tab focused), or the #395 empty workspace → absent or
  safely inert over the #392 `try_workspace` totality twins.
- **Parity:** the React-first build in marley-web (see below) + the Validate parity pair.
- **Regression pins:** the #390 rail suite (57 tabs tests), #393 section-menu suite, #396 reaper-timing +
  persistence round-trips, #246 split-to-file, #395 empty-workspace suite — all green unchanged.

### Out (explicitly deferred)
- **Drag-and-drop add** — verb-driven v1 stands; gpui 0.2.2's typed `on_drag`/`on_drag_move<T>`/`on_drop<T>`
  (div.rs:279-499) is the recorded v2 substrate, not adopted here.
- **Nameable arrangements** — the `[[panes]]` settings table, "make a Pane", key-based restore rebinding:
  slice-6 **#399**.
- **Cockpit/browser kinds** — slice-8 **#400** (browser waits on #389); the add verbs cover
  registry-resident kinds only (terminals, editors, git per the shipped `Content` enum arms).
- **Global cross-workspace Panes** — gated slice-7 (needs the multi-workspace model).
- **Restore-time de-dup of same-key cells** — a shared-view grid persists as today's per-cell leaves (ids
  NEVER serialize, #394) and restores as per-cell instances exactly as #396 ships; sharing is
  session-local until #399's key rebinding. Codec byte-identical; zero persistence change.
- **Move/replace semantics** — the add never closes or relocates the origin ("the original stays open");
  a replace-in-place or move-to-pane verb is a different, destructive gesture, not this ticket.

## Reference (§20)
**Warp (terminal / cockpit / UX reference) — checked, THIN COVERAGE, stated honestly:** the behavior maps
carry Marley's split surface only at the overview level — docs/warp_architecture/subsystems/00-overview.md:45
records "own docks + `PaneGroup` split/close … MATCHED" and the center-PaneGroup diagram (:133-136); **no
move-pane / add-to-split gesture is mapped anywhere in docs/warp_architecture/subsystems/**, and
docs/warp_architecture/observed/ holds only the monospace-grid caret capture (250-warp-monospace-grid-caret.png)
— nothing on split composition. No new observation was required: the gesture here is Marley's own #388
design (chad's model A), not a Warp mimic. **Zed (same-gpui editor reference) at the BEHAVIOR level:**
docs/zed_architecture/subsystems/07-workspace-panes-palette.md maps the behavior Marley matches — items are
pane-hostable peers that can be **placed into another pane without closing their origin**: drag-between-panes
(`can_drop_predicate` :136, `handle_drop` :163), and splitting **clones the item view onto the same
underlying entity** (`can_split`/`clone_on_split` :157) — the one-instance-many-views end state, which
Marley reaches verb-first (Zed's drag is the v2 analog). Research, not source; clean-room §20 untouched: no
Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked.** The Warp/Zed citations above, plus the in-house design doc
   docs/marley_architecture/pane-composition-model.md (slice-5's own paragraph, the Q2 instance/view table,
   the recorded `AD-claude-pane-content-id-registry-001` + `AD-claude-one-instance-many-views-001`).
2. **Published material — none found, honestly.** No protocol spec governs pane composition (no LSP-like
   leg here); no external doc consulted beyond the maps.
3. **Our permissive deps + in-house seams — the paying leg.** gpui (Apache-2.0) SHIPS a typed drag-drop
   primitive (`on_drag`/`on_drag_move<T>`/`on_drop<T>` + `DragMoveEvent<T>`,
   ~/.cargo/registry/src/…/gpui-0.2.2/src/elements/div.rs:62-499) — recorded as the v2 drag substrate;
   adopting it now would violate the verb-first v1 (D1), so noted-not-adopted. ropey/regex/
   alacritty_terminal/tree-sitter own nothing near this seam. The real adoption is in-house: the ONE menu
   machinery + const row tables (#393, context_menu.rs:111-194), the dynamic palette ranges + shared id↔index
   math (#87/#199/#204, palette.rs:183-187, app.rs:1883-1931), `split_focused_pane`/#246 split-to-file
   (app.rs:6827/6795-6803), `jump_to_pane` (app.rs:7024), and `ContentRegistry::acquire_view`/`release_view`
   (content_registry.rs:66/78) — every mechanism the gesture needs already exists; this ticket composes them.

## React-first (parity)
**UI-AFFECTING — Zone A (rail + context menus + palette; port-map rows: `components/LeftRail.tsx` ↔ app.rs
rail/tabs.rs/context_menu.rs, `components/ContextMenu.tsx` ↔ context_menu.rs,
`views/SplitTerminalView.tsx` ↔ grid_layout.rs/content_registry.rs, `overlays/CommandPalette.tsx` ↔
palette.rs — marley-web/docs/MARLEY-PARITY.md).** Implement builds the WHOLE flow in marley-web FIRST: the
add-to-pane verb rows (menu + palette), the ContentId-derived Panes cell labels, and the home-section
cross-listing with its marker, iterated at localhost:5173 (`pnpm --filter @workspace/marley-ide run dev`)
until the flow is confirmed in the browser — THEN ported 1:1 into the pure seams + app.rs shims. The POC's
`PaneItem { type, name }` is the ContentId stand-in (MARLEY-PARITY.md § Shared vocabulary — kept aligned for
exactly this port); the POC's `panes[].items` already filter in the rail search (LeftRail.tsx:96-100), the
`PANE ${id}` rows exist (:281), and the #393 Section menus are mirrored row-for-row (ContextMenu.tsx:92-118)
— this ticket extends those, it does not rebuild them. POC discipline: the shared instance is mocked state
(`commandSimulator` scrollback keyed once, rendered in two cells) — the POC never ports its data. Validate
captures the React↔Marley parity pair.

## Locked-In Decisions
- **D1 — the gesture is VERB-driven through the ONE menu machinery (the #393 discipline).** New rows in the
  existing const tables + new action arms, dispatched via the one render/keyboard/esc/one-modal machinery
  (`MenuKind`/`items_for`, context_menu.rs) and the palette — **no drag-and-drop** (v2; gpui primitive
  recorded above), **no parallel popover/overlay kind**. The add targets ride the KIND (the #175
  target-rides-the-kind pattern) so a #173-pump shift while a menu is open can never retarget.
  Which exact menus/verbs = D-OPEN-GESTURE-SURFACE.
- **D2 — acquire/release symmetry; the #396 close contract reused unchanged.** Every add acquires exactly
  one view (`acquire_view`); every cell/tab close releases exactly one (the #396 `release_view` routing,
  no new close semantics); the LAST release returns the owned content to the caller, which keeps the
  existing ~600ms off-thread reap (`thread::spawn(drop)`, the TICKET-348 bounded contract). Slice-5's only
  lifecycle delta is making view_count > 1 REACHABLE — the 1-view paths stay byte-identical.
- **D3 — the label source is a pure derivation; `rail_rows` stays pure.** ContentId → (kind, title) → row
  label is a pure fn (cov/MSI 100, exhaustive over the registered kinds — no defensive catch-all); the
  gpui-free model layer never reads the registry globally — the render shim supplies the derived inputs
  (the #390 derived-fresh discipline: labels recompute each render, no stored identity). Exact per-kind
  text = D-OPEN-LABEL-SHAPE.
- **D4 — cross-listing is PRESENTATION-ONLY.** Computed fresh each render from the model (like the #390
  Panes section: no persisted state, no model mutation, nothing new in the codec); a cross-listed row
  navigates via the existing coordinate path (`jump_to_pane` / the #174 activate-first idiom). A row
  appears the moment content is paned and disappears when the last paned view closes — by construction.
- **D5 — the dynamic palette rows REBUILD, never append (the #204/R1 constraint).** The workflows range
  could append (`id = base + index`) ONLY because v1 workflows never delete — "no id reuse/collision; a
  future delete needs a rebuild" (warp-workflows.notes.md:148-149/:193-194). Open content closes
  constantly, so an append-only add-to-pane range would misdispatch onto reused/shifted indexes: the block
  rebuilds from the CURRENT open-content set (or resolves against a dispatch-time snapshot — Phase 2 picks
  the mechanism), in a fresh id range above `WORKFLOW_BASE`, using the shared `dynamic_command_index`.
- **D6 — persistence is untouched.** Ids never serialize (#394); the leaf codec stays coordinate/shape-based
  and byte-identical; a shared-view grid restores as per-cell fresh instances (#396's rebuild) — sharing is
  session-local until #399. The #163/#205/#177 round-trips are regression-pinned.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-GESTURE-SURFACE** — exactly which menus/verbs. Candidates: (a) a context menu on the rail's
  CONTENT rows (Editor/Terminal Tab rows + cross-listed rows) with "Open in Split Right/Down" targeting
  THAT row's content — a new target-bearing `MenuKind`; (b) dynamic palette rows ("Add to Split: <title>",
  one per open content, the #199/#204 range idiom under D5); (c) rows in the Panes＋ or per-section ＋
  menus. **Recommendation: (a)+(b), not (c)** — the #393 ＋ tables are CREATE-verbs ("various things" that
  make new content); add-to-pane VIEWS existing content, and folding it into the create menus would blur
  the table's documented meaning. Phase 2 settles the exact rows against keymap collisions (any chord must
  clear keymap.rs — the MARLEY-PARITY keybindings table shows how crowded ⌘⇧-space already is) and the
  one-modal discipline (#393's `close_transient_overlays`).
- **D-OPEN-LABEL-SHAPE** — what a Panes cell row reads per kind. Candidates: terminal → the #177 title
  lineage (custom title beats live command title); editor → file basename; git → "git diff". Single
  clamped line (the ~150px Nav row). Sub-question: does the "PANE n" arrangement HEADER also gain a derived
  summary ("PANE 1 — cargo · main.rs")? **Recommendation: cell rows get the content label; the header stays
  "PANE n"** until slice-6 naming makes it user-owned.
- **D-OPEN-CROSSLIST-MARKER** — how a home-section row indicates "also in a Pane". Candidates: a muted
  trailing glyph (the split/grid glyph from the icons.rs set), a "· pane" text suffix, or unmarked listing.
  **Recommendation: a muted trailing glyph** — visible at a glance, zero row-height cost, no label
  truncation pressure; confirmed visually in the POC first (the React-first stage exists exactly for this).
- **D-OPEN-EMPTY-CELL-TARGET** — does add-to-pane CREATE the split or FILL an existing cell?
  **Recommendation: create-the-split** (the #246/#155 idiom: split the focused pane along the verb's axis;
  the new cell holds the acquired view) — Marley's grid has NO empty cells to fill (every `PaneState` holds
  content), so "fill" could only mean REPLACE, a destructive different verb (deferred with move/replace).
  The no-grid/empty-workspace arm is enablement's job (REQ-007), not a special target rule.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user invokes an add-to-pane verb targeting an OPEN terminal (via the settled D-OPEN-GESTURE-SURFACE surfaces), the focused pane shall split along the chosen axis and the new cell shall present the SAME terminal instance — one registry entry at view_count 2, shared session/scrollback, the origin's Terminal-section row still present. | driven scenario (two cells render one session; input through either view lands in the one PTY) + registry unit (add acquires exactly once → view_count 2) + §18.1 inspect: no second spawn on the add path |
| REQ-002 | WHEN the user invokes an add-to-pane verb targeting an OPEN editor, the new cell shall present the same Buffer (view_count 2) and an edit made through either view shall be visible in the other (per-view caret/scroll stay independent — the #397 split). | driven scenario + units over the shared-instance/per-view seams; #246 split-to-file suite green unchanged (the fresh-open path is NOT this verb) |
| REQ-003 | WHEN the rail renders a Panes cell row, its label shall derive from the cell's ContentId (kind + title per D-OPEN-LABEL-SHAPE), never the positional "pane n"; a title-source change (rename/live command title) shall be reflected on the next render. | pure label-derivation units per registered kind incl. the clamp; rail_rows row-label unit; the #390 tabs suite green (arrangement numbering unchanged) |
| REQ-004 | WHILE content whose kind homes under Editor or Terminal is mounted in any Pane cell, its home section shall list it (marked per D-OPEN-CROSSLIST-MARKER) — including pane-ONLY content that today has no home row — and activating the cross-listed row shall focus that cell via the existing coordinate path. | pure cross-list computation units (tab-mounted, pane-only, both, none); §18.1 inspect of the click path (`jump_to_pane` + #174 sync + persist); #390/#393 rail suites green |
| REQ-005 | WHEN one view of multi-view content closes — by ANY close path, including closing the home TAB of content that also lives in a Pane cell — exactly ONE view-count shall release and the instance shall survive (no PTY reap, no editor teardown); WHEN the LAST view closes, ownership shall return to the caller and the existing off-thread reap shall run (terminals: the TICKET-348 bounded contract, unchanged). | registry units over every close path incl. tab-close-with-paned-view and close-project; the #396 reaper-timing suite green; §18.1 inspect: acquire/release symmetry on every add/close/replace/error path (the leak lens) |
| REQ-006 | WHEN the open-content set changes (open, close, reopen), the palette's dynamic add-to-pane rows shall REBUILD from the current set — never append onto stale `base + index` — so a dispatch always resolves the CURRENT target (the #204/R1 constraint: append held there only because workflows never delete). | unit over the row builder (open→close→reopen sequence dispatches the right target, no off-by-one); the exhaustive `action_for_command`-map test extended for any new static arm (`PR-claude-new-palette-arm-extend-the-exhaustive-map-test`) |
| REQ-007 | WHEN no addable target exists — no open content of an addable kind, no splittable focused grid (cockpit tab focused), or the #395 empty workspace — the add-to-pane verbs shall be absent or safely inert (no panic, no phantom split, the #392 `try_workspace` totality). | pure enablement units over the closed input space; #395 empty-workspace + #392 totality suites green |
| REQ-008 | WHEN Validate runs, the React POC (gesture + labels + cross-listing confirmed at localhost:5173) and the Marley build shall present the same rail/menu/pane behavior — the parity pair captured. | React capture + Marley driven/headless capture receipt; if the machine state blocks driving, the recorded env-blocked protocol applies (documented explicitly; units + mechanism carry — the #204/#205 precedent) |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the label derivation (exhaustive over the registered kinds), the cross-list
computation, the verb enablement, the new menu row tables (the `section_items`/`items_for` full-slice-equality
test idiom, context_menu.rs:397-458), and the palette range math (reusing `dynamic_command_index` + the
rebuild builder). MASKED: the app.rs dispatch/acquire/split shims and render arms (app.rs is
coverage-excluded), per the shipped #390/#393 pattern. Typed inputs; no `unwrap` on registry lookups (an
unknown id is `None`, never a panic — the `#[must_use]` contracts on `acquire_view`/`release_view` are
load-bearing here: a dropped `release_view` return reaps on the render thread). The app.rs edits land near
the masked close-path/pump shims — the skip-detach trap's home turf (5th strike was `pump_fleet_live`,
`BF-claude-skip-detach-pump-fleet-live-001`): re-run `cargo mutants --list -f` on the ACTUAL touched files
after placement, and re-verify neighboring `#[mutants::skip]` bindings.

## Phase Plan
- **P2 Design** — verify the SHIPPED #396/#397 shapes and bind to them (the accessor/tag/close-path reality,
  not this spec's predictions — the #384-D7 pattern); settle D-OPEN-GESTURE-SURFACE (against keymap.rs +
  one-modal), LABEL-SHAPE (exact per-kind templates + clamp), CROSSLIST-MARKER (glyph choice), and
  EMPTY-CELL-TARGET (confirm create-the-split); exact signatures for the label/cross-list/enablement seams
  + the `rail_rows` input plumbing; the palette rebuild mechanism (rebuild-on-change vs dispatch-time
  snapshot) under D5; the BOTH-HALVES file manifest — **marley-web files first** (LeftRail/ContextMenu/
  SplitTerminalView + the PaneItem state), then the Rust manifest; per-REQ test plan.
- **P3 Implement** — **React-first: build the gesture + labels + cross-listing in marley-web and confirm
  the flow at localhost:5173 (per `## React-first (parity)`)**; then Rust: pure seams first (label
  derivation, cross-list computation, enablement, menu tables, palette range), then the masked shims
  (menu/palette dispatch → `acquire_view` + split-mount; the render label/marker arms).
- **P3.5 Inspect** — adversarial lenses: **refcount leak on EVERY path** (walk every add/close/replace/
  error path incl. tab close with a paned view, close-project, pump-dead-pane, boot cleanup — does every
  acquire have exactly one release?); **label staleness** (rename/custom-title/live-title changes propagate;
  no cached label survives a content change); **menu reachability** (every new row reachable; the #393
  `close_transient_overlays` one-modal discipline holds; the search-filter guard app.rs:16522 treated
  correctly for cross-listed rows); **restore of a shared grid** (per-cell rebuild, no double-free, codec
  byte-identity); dispatch-vs-stale-index on the palette range (D5); skip-detach re-check; provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; the two driven scenarios (terminal add, editor add) +
  captures; the React↔Marley parity pair (REQ-008); `cargo mutants --list -f` on the actual touched files;
  gate green (`--diff`), cov/MSI 100 on the pure seams; #390/#393/#396/#246/#395 regression suites green.
- **P5 Complete** — CHANGELOG; tick slice-5 in pane-composition-model.md Q5 (the train ledger); AAR capture
  (the rebuild-vs-append lesson closes the #204/R1 follow-up); archive; close #398.
