---
pipeline_id: 7db91883-d28a-4405-b189-967fdd3d0b05
ticket: forge#400 (95954769-f57f-4167-99ff-e8b01bfd5756) · local docs/planning/tickets/open/TICKET-400-cockpit-residency.md
aar_id: adb07e4f-5271-4895-a691-765631dbcf2c
status: Phase 5 — Complete PASS
title: Cockpit onto the registry — the residency half (the #388 slice-8, cockpit only)
type: feature
milestone: M28
references:
  - docs/marley_architecture/pane-composition-model.md
  - docs/marley_architecture/embedded-browser-model.md
  - docs/planning/tickets/open/TICKET-396-terminals-onto-registry.md
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/right_dock.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/grid_layout.rs
  - crates/marley_app/src/app.rs
---

## Title
The #388 train slice-8, **COCKPIT half only** — the three cockpit surfaces become `Content::Cockpit`
registry residents under the #394 one-instance-many-views lifecycle. The precise shipped seam (verified
against the tree; the ticket's "right_dock-resident" is legacy phrasing — the side dock was retired at
M9 #153 and Zed's map records Marley's right dock as "retired"):
- **The vocabulary** lives in `right_dock.rs` (PURE, 142 lines): `RightSection { Details, Agents,
  Forge }` (:11-18), the #95 persisted key `cockpit.right_section` (`right_section_key`/`from_key`,
  :22/:32), the top-bar strip (`top_tabs` :62, rendered app.rs:19071), `section_icon` (:75).
- **The residence** is a TAB: `TabContent::Cockpit(RightSection)` (tabs.rs:24/28) built by
  `Tab::cockpit` (:153), switched by `Project::open_or_switch_cockpit` (:411-424 — existing→switch,
  absent→append, per project), listed under `RailSection::Browser` (:127 — the #385/#389 "native
  cockpit meanwhile" home), persisted as the bare shell-codec tag `C=<section key>`
  (`TabLayout::Cockpit`, grid_layout.rs:217/:355/:411), restored at app.rs:2038-2044.
- **The state** is app-side and NOT per-instance: `right_section: RightSection` (app.rs:433, restored
  :2387) is the persistent SELECTION (the parity doc pins this: "the cockpit highlight is a persistent
  selection, `right_section`, not 'am I looking'" — MARLEY-PARITY.md:185); the surface DATA is global
  service state (`agents` :190, `remotes` :382, `forge_sprint` :389, `forge_pending` :392); the three
  surface modules are pure projections over it (pane_details.rs / agent_view.rs / forge_view.rs — no
  owned instance state anywhere). The render is the masked `cockpit_body(section, …)` shim
  (app.rs:5396-5550), total over no-terminal via the #391 `try_workspace` contract (its Details arm
  renders empty — the #391 CHANGELOG names this site) behind the #392 zero-tab render guard.

The work: cash #396's net-new `Content::Cockpit` declaration (the #385-comment discipline) — one
registry instance per surface (3 app-wide singletons), `TabContent::Cockpit` becomes id-bearing (the
#388 open-Q1 `(ContentId, tag)` shape; the section tag keeps `cockpit_section`/`rail_section`/codec
pure with no registry read), open/restore paths **resolve-or-register** by section key (two projects'
Forge tabs = two VIEWS of ONE instance — the first content in the app whose view-count legitimately
exceeds 1, ahead of slice-5), and every cockpit close path routes `release_view` under a **pinned
lifecycle** (a cockpit is never handed to teardown — D-OPEN-SINGLETON-LIFECYCLE). Byte-identical
renders, selection semantics, and codec; the visible delta is whatever the slice-5 machinery exposes
for the kind, per D-OPEN-KIND-GATING — nothing else. This is the precondition for cockpit-in-a-pane
and for Browser residency (#389's Q4 already mirrors the cockpit `C=` marker for its future `B=` tag —
keeping the cockpit codec unchanged keeps that mirror true).

**Ordering:** hard-ordered AFTER #396 (`Content` enum + the migration pattern + `RootView.content:
ContentRegistry<Content>` all land there; /work must not promote #400 while #396 is unshipped). #398 is
SOFT — only the exposure half reads it; residency needs only #396 and may land first (#398 also needs
#397, so residency-first is the likely sprint path).

## Scope
### In
- **Cash `Content::Cockpit`** (declared in #396's enum): the variant carries the surface identity —
  candidate `Content::Cockpit(RightSection)`; per D-OPEN-STATE-MIGRATION it owns identity ONLY this
  slice (no data migration — the surfaces stay pure projections over app-side service state).
- **The singleton index + resolve-or-register:** an app-side `RightSection → ContentId` resolution
  (register-at-boot vs lazy is a Phase 2 sub-fork under D-OPEN-SINGLETON-LIFECYCLE); opening,
  switching, a second project's tab, and restore all resolve the SAME id per section; ≤3 cockpit
  entries ever exist.
- **`TabContent::Cockpit` becomes id-bearing** (+ the pure section tag): `Tab::cockpit`,
  `is_cockpit`/`cockpit_section` (tabs.rs:195/:199), `open_or_switch_cockpit` (id supplied by the
  app-side caller — the #396 construct-once→register→hand-back-an-id pattern from the Q3 ripple
  table), `rail_section` unchanged (tag-based, no registry read).
- **Close-path routing:** cockpit-tab closes (close-tab / close-project / any tab-drop path the #396
  audit enumerates) route `release_view`; the `#[must_use]` return is consumed at every site
  (`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`); under the pinned lifecycle
  the return is `None` by construction on every user-reachable path (the anchor view holds) — asserted,
  not assumed.
- **Persistence:** codec byte-identical — `C=<section key>` stays the whole persisted form; ids NEVER
  serialize (the #396 discipline); restore resolve-or-registers per restored cockpit tab (cross-project
  dedupe to one instance).
- **Render through the registry:** `cockpit_body`'s dispatch resolves the tab's `ContentId` →
  `Content::Cockpit(section)` and renders the same three arms — byte-identical output; the top-bar
  strip + `right_section` selection + #95 persistence untouched.
- **Regression pins:** tabs.rs `open_or_switch_cockpit_cases`/`cockpit_section_cases` (:1303/:1296),
  right_dock.rs suite (:89-:140), the #95 settings round-trip (settings.rs:697 area), grid_layout
  cockpit codec tests (:653/:696), status_bar `FocusTab::Cockpit` (:160) — all green unchanged.

### Out (explicitly deferred)
- **Browser residency** — #389/Phase E (the wry/WKWebView substrate, `TabContent::Browser`, the `B=`
  tag); this slice only stays layout-compatible with it.
- **The add-to-pane gesture + cross-listing themselves** — #398 (slice-5); this slice makes the kind
  *eligible*, D-OPEN-KIND-GATING decides whether anything is *enabled*.
- **Any cockpit surface redesign** — rows/glyphs/labels/empty-hints byte-stable; no new pixels.
- **Global cross-workspace Panes** — slice-7, gated on multi-workspace.
- **Data migration into instances** (`forge_sprint`/`forge_pending`/`agents`/`remotes` stay app-side
  service state; a cockpit CELL render arm (`PaneContent::Cockpit`) is NOT added unless
  D-OPEN-KIND-GATING's exposure arm fires — and then only per its Phase 2 resolution).

## Reference (§20)
**N/A — Marley-specific composition model** (the #388 model A is chad's own; no reference app to match
for the residency mechanics). The honest adjacent behaviors, research-level only:
- **Zed (behavior map — the strongest analog):**
  docs/zed_architecture/subsystems/07-workspace-panes-palette.md maps exactly the citizenship this
  slice buys — the same `Pane` serves the center area AND a dock's contents (§1.3), "dozens of
  unrelated views are all first-class pane items" (§1.4), and the agent/outline/git panels are peer
  `Panel` impls docks host interchangeably (§1.6). Its Marley-comparison table names today's gap
  verbatim: "cockpit sections = `enum RightSection` rendered as top tabs (`TabContent::Cockpit`), not
  dock panels — **GAP** — no uniform panel citizenship". This ticket closes that row for the cockpit
  kind by Marley's OWN route (closed enum + `ContentId` registry per
  `AD-claude-pane-content-id-registry-001`, not Zed's open traits). Behavior observed from the MAP,
  never Zed source — clean-room §20 (a reworded translation is still a derivative work).
- **Published behavior:** VS Code's documented movable views/panels — panel-area views can be dragged
  into the editor area and live as editor-tab citizens (published docs) — is the convention the later
  slice-5 exposure matches; adopted at the published-behavior level only.
- **Warp:** checked — the cockpit-adjacent map chapter
  (docs/warp_architecture/subsystems/04-agent-ai-mcp.md §7, :269) carries agent event-stream
  observability (the "live workflow panel" opportunity note), no drawer/panel-residency behavior to
  cite. No Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked; the Zed citations above are the yield.** zed 07 §1.3/§1.4/§1.6 + the
   comparison table's Docks-PARTIAL and Dock-contents-GAP rows (the row this ticket closes). Warp
   04-agent-ai-mcp.md §7: no owner. Research, not source.
2. **Published material.** VS Code movable-panels convention (above). No protocol surface in this
   ticket.
3. **Our permissive deps — checked, no owner; the real adoption leg is IN-HOUSE.** gpui (Apache-2.0)
   provides render primitives only — no registry/dock seam to adopt; ropey/regex/alacritty_terminal/
   tree-sitter own nothing near a content-lifecycle seam. The seams that DO own this are already
   shipped Marley code, and this ticket reuses rather than reinvents them: `content_registry.rs`
   (#394) owns the refcounted lifecycle including exactly the drop-on-last-close catch
   (`PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001`); `right_dock.rs` owns the
   section vocabulary + persistence keys; `tabs.rs` owns tab citizenship; the #396 migration pattern
   (id+tag, resolve-through-registry, rebuild-on-restore) is the template every step follows.

## React-first (parity)
**N/A expected — pure residency, no UI delta (ownership/identity migration only); CONDITIONAL — Phase 2
re-confirms the arm against #398's shipped state before implement starts.** What the code reading found
(2026-08-04, the basis for the expected arm): **(1)** slice-5's add-to-pane machinery does not exist in
the tree (no add-to-pane verb/command/menu row anywhere in crates/ — #398 is a sibling M28 ticket,
unshipped); **(2)** #398's drafted gesture operates on "an OPEN ContentId" (kind-agnostic at the
registry layer) but its flows name terminals/editors and it hard-depends on #397, so it likely lands
AFTER this slice; **(3)** a cockpit surface has NO pane-cell arm today — `PaneContent { Terminal,
FileTree, CodeView, Git }` (workspace.rs:317) carries no Cockpit, and `cockpit_body` renders only
full-screen for a cockpit TAB (app.rs:5396) — so registering the kind does NOT by itself make
cockpit-in-a-pane reachable; the exposure gates per-kind on a net-new cell arm (D-OPEN-KIND-GATING).
**The flip protocol:** if Phase 2 finds #398 shipped first with a registry-generic verb + cell dispatch
that reaches the Cockpit kind, this section flips to UI-AFFECTING (Zone B surface): prototype the
cockpit-in-pane reachability in marley-web FIRST — the POC's cockpit views already exist
(`views/AgentsPane.tsx`, `views/ForgePane.tsx`, `views/BlockDetailsPane.tsx` ↔ `agent_view.rs` /
`forge_view.rs` / `pane_details.rs`, MARLEY-PARITY.md port map :581-583) — iterate at localhost:5173
(`pnpm --filter @workspace/marley-ide run dev`), then port 1:1, and Validate captures the React↔Marley
parity pair. If it lands as pure residency (the expected arm), the implement-phase notes record
"React-first: N/A — no UI delta (pure ownership migration)" with the vocabulary tie: **ContentId stays
aligned with the POC's PaneItem stand-in** (MARLEY-PARITY.md § Shared vocabulary) so the slice-5/-6
exposure ports cleanly.

## Locked-In Decisions
- **D1 — `Content::Cockpit` is DECLARED in #396's enum and CASHED here** (the #385-comment discipline:
  a new kind picks its pre-declared variant; no parallel content type, no second registry). #400 adds
  no enum; it wires the variant + the residency.
- **D2 — one instance per cockpit surface; Details/Agents/Forge are app-wide SINGLETONS.** The
  one-instance-many-views contract (`AD-claude-one-instance-many-views-001`) applies unchanged: N
  tabs/cells (including tabs in different projects) hold the same `ContentId`; the view-count guards
  the drop. Resolve-or-register is keyed by `RightSection` — the same stable key the #388 Q4
  persistence model already assigns cockpit content.
- **D3 — `right_section` selection semantics unchanged.** The persistent-selection behavior the parity
  doc documents (MARLEY-PARITY.md:185-189) and the #95 persisted key `cockpit.right_section`
  (settings.rs:130) stay byte-identical; the selection remains dock-side state, never per-instance.
- **D4 — the dock keeps rendering through the registry view with NO visual delta.** `cockpit_body`'s
  three arms, the top-bar strip, icons, labels, and empty-hints render byte-identically; only the
  dispatch's ownership path changes (tab id → registry resolve → the same section arms).
- **D5 — Browser stays OUT** (#389/Phase E). Compatibility only: the cockpit `C=<key>` codec stays
  unchanged, preserving the marker shape #389's Q4 mirrors for its future `B=` tag; nothing here
  pre-builds webview substrate.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-SINGLETON-LIFECYCLE** — are cockpit residents ever DROPPED on last view close, or
  pinned-for-life? **Recommend PINNED**: the top-bar strip + persisted `right_section` are standing
  references to every section independent of any tab existing, and a cockpit owns no reapable resource
  — so drop-on-last-close must not reap a cockpit. Candidate mechanism: a standing **dock-anchor
  view** acquired at registration (view-count starts at 1 and user close paths can never return it to
  0), keeping the generic `ContentRegistry` untouched (no special-case branch; the #394 lifecycle
  stays pure) and making `release_view == None` on every cockpit close path a provable invariant.
  Sub-fork: register all 3 at boot (recommended — 3 tiny entries, restore never races the index) vs
  lazily on first open.
- **D-OPEN-STATE-MIGRATION** — which state moves into `Content::Cockpit` vs stays dock-side.
  **Recommend: identity ONLY this slice.** `right_section` (selection, D3) stays app-side;
  `forge_sprint`/`forge_pending` (pump-owned fetch state), `agents`/`remotes` (terminal-content
  trackers — #396 re-keys them to terminal `ContentId`s) stay app-side service state; the surfaces
  remain pure projections, which is precisely what makes the migration structural. Record the later
  fork: per-instance surface state (scroll, filters) moves in only when a surface actually grows owned
  state.
- **D-OPEN-KIND-GATING** — does slice-5's add-to-pane verb enable for Cockpit now or behind a
  follow-up? **Recommend: gated behind the follow-up unless it is free.** Evidence today: the verb
  does not exist (#398 unshipped) and a cockpit CELL needs a net-new `PaneContent` arm + cell render +
  grid-leaf codec that no ticket declares — not "for free" at the render layer even though the
  REGISTRY layer is free. If #398 has shipped by implement time: enable Cockpit only if its dispatch
  is registry-generic and zero new cockpit render code is needed; otherwise ship residency structural
  and record the exposure as the follow-up. Phase 2 confirms against the shipped code, and the
  React-first arm follows this resolution.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a cockpit tab renders or is switched to, the system shall resolve its surface through the registry — the tab's `ContentId` → `Content::Cockpit(section)` — for all three sections, with the pure tag accessors (`cockpit_section`/`rail_section`/`is_cockpit`) reading the tag, never the registry. | units over the migrated seam (id-bearing `TabContent::Cockpit` + resolve dispatch); exhaustive per-section match (no catch-all — `PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001`) |
| REQ-002 | WHEN a section is opened, switched to, opened in a second project, or restored from a persisted shell, the system shall resolve the SAME `ContentId` per section (≤3 cockpit entries ever; cross-project tabs are views of one instance) and shall never serialize an id (the `C=<key>` codec byte-identical). | resolve-or-register units (same-id across open/re-open/second-project/restore); registry `len` ≤3 pin; grid_layout cockpit codec round-trips (:653/:696) green byte-identical |
| REQ-003 | WHILE the app runs with cockpit tabs open, the cockpit presentation — top-bar strip, section icons, tab labels, and all three `cockpit_body` arms including empty states — shall render byte-identical to pre-migration. | driven/headless capture pair (before/after) of a cockpit tab + the strip, or the visual-harness byte assert; if the machine state blocks driving, the recorded env-blocked protocol applies (units + full-suite byte-identity carry — the #204/#205 precedent) |
| REQ-004 | WHEN the user switches cockpit sections or restarts the app, tab switching and the persistent `right_section` selection (#95) shall behave exactly as shipped. | existing suites green unchanged: tabs.rs `open_or_switch_cockpit_cases`/`cockpit_section_cases`, right_dock.rs full suite, the #95 settings round-trip, status_bar `FocusTab::Cockpit` |
| REQ-005 | WHEN the last tab viewing a cockpit section closes (including the cross-project last), the system shall NOT reap the cockpit content — the instance survives with the pinned anchor view and `release_view` returns `None` on every user-reachable cockpit close path. | pinned-lifecycle units (register → acquire tab views → release all → entry alive, `view_count ≥ 1`, `get(id)` Some); §18.1 inspect: reap-safety walk over every close path |
| REQ-006 | WHEN any cockpit-tab close path runs, the system shall route it through `release_view` with the `#[must_use]` return consumed at the site, and every acquire shall have a matching release (no orphaned views, no double-release). | leak-pairing units (acquire/release counts across close-tab/close-project sequences); double-release safety unit; §18.1 inspect: site-by-site audit per `PR-claude-registry-release-returning-a-resource-must-be-must-use-001` |
| REQ-007 | IF D-OPEN-KIND-GATING enables cockpit-in-a-pane this slice, THEN the slice-5 gesture shall place a cockpit `ContentId` into a split cell and render it (driven proof), with the React prototype preceding the Rust per the flipped React-first arm; OTHERWISE the implement-phase notes shall record "React-first: N/A — no UI delta (pure ownership migration)" with the ContentId↔PaneItem vocabulary tie. | conditional: driven gesture capture + parity pair; or the recorded N/A line + vocabulary-tie note in the Phase 3 notes (the enforce-react-parity.sh contract) |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the resolve-or-register decision fn + singleton index, the pinned-
lifecycle guard, the id-bearing `TabContent::Cockpit` accessors + tag purity, `right_dock.rs` (stays
100), the codec arms. MASKED: the app.rs shims (`cockpit_body`, `open_cockpit_section`, the top-bar
strip render, restore/serialize wiring, close-path plumbing — app.rs is coverage-excluded). Typed
inputs; no `unwrap` on restore-derived paths. The app.rs edits land near the masked pump/boot shims —
the skip-detach trap's home turf (5th strike `BF-claude-skip-detach-pump-fleet-live-001`): re-run
`cargo mutants --list -f` on the ACTUAL touched files after placement and re-verify neighboring
`#[mutants::skip]` bindings (`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`).
Every `release_view` call site consumes the `#[must_use]` return
(`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`).

## Phase Plan
- **P2 Design** — verify #396's SHIPPED `Content` enum + migration shape and bind to it (the hard-dep
  gate); confirm the React-first arm against #398's shipped state (the flip protocol); settle
  D-OPEN-SINGLETON-LIFECYCLE (anchor mechanism + boot-vs-lazy), D-OPEN-STATE-MIGRATION (confirm
  identity-only), D-OPEN-KIND-GATING (gated vs free, with the cell-arm evidence); exact
  `Content::Cockpit` + id-bearing `TabContent::Cockpit` signatures; the file manifest + per-REQ test
  plan; enumerate every cockpit-tab close path from the #396 audit.
- **P3 Implement** — React-first stage ONLY if the arm flipped applicable (marley-web
  AgentsPane/ForgePane/BlockDetailsPane reachability, confirmed at localhost:5173, then port 1:1);
  then the pure seams (variant wiring, singleton index, resolve-or-register, tag accessors), then the
  masked app.rs wiring (open/restore/serialize/close/render dispatch).
- **P3.5 Inspect** — adversarial: reap-safety on cockpit content (walk EVERY close path — can any
  sequence hand a `Content::Cockpit` to teardown?); dock render byte-identity; state-migration
  completeness (no cockpit state duplicated or left dangling between app-side and instance);
  acquire/release leak pairing incl. restore + project-close; codec byte-identity; skip-detach
  re-check; §20 provenance.
- **P4 Validate** — write + RUN the units per REQ; `cargo mutants --list -f` on the actual touched
  files; gate green (`--diff`), cov/MSI 100 on the pure seams; the REQ-003 capture (or the documented
  env-blocked fallback); all named regression suites green; parity handling per the resolved arm.
- **P5 Complete** — CHANGELOG; mark pane-composition-model.md slice-8 shipped (cockpit half; Browser
  half stays gated on #389); AAR capture; archive; close #400.
