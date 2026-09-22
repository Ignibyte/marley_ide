---
pipeline_id: a340c225-ab60-4302-ab84-d6ed42b76fd4
ticket: forge#403 (f7bf9657-b4c1-4fd6-b346-584660daf256) · local docs/planning/tickets/open/TICKET-403-browser-tab-residency.md
aar_id: f394490d-c4fe-4e29-9f30-cc8e7250d848
status: Phase 5 — Complete PASS
title: TabContent::Browser + Content::Browser residency + the bare `B` shell tag (the #389 slice-2 + the #400-gated Browser half)
type: feature
milestone: M29
references:
  - docs/marley_architecture/embedded-browser-model.md
  - docs/marley_architecture/pane-composition-model.md
  - docs/planning/pipeline/completed/400-cockpit-residency.spec.md
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
  - crates/marley_app/src/content.rs
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/grid_layout.rs
  - crates/marley_app/src/status_bar.rs
  - crates/marley_app/src/app.rs
---

## Title
The embedded-browser train's slice-2 (#389, embedded-browser-model.md Q4 + train step 2) fused with
the Browser half of the #388 slice-8 that #400 shipped cockpit-only — the pure, substrate-independent
model: a Browser TAB exists, registers, persists, and renders a placeholder, with **zero webview or
URL code**. Verified against today's tree (all line numbers 2026-08-06):
- **The section is already home, waiting for its namesake:** `RailSection::Browser` (tabs.rs:51) is
  the cockpit tabs' transitional rail home (`rail_section()`, tabs.rs:127-133 — Cockpit → Browser at
  :131); the Browser＋ menu's three rows are all cockpit residents (`SECTION_BROWSER_ITEMS`,
  context_menu.rs:158-171).
- **The variant is DECLARED, not shipped:** `Content::Browser` exists as a payload-less unit variant
  (content.rs:49, "DECLARED — browser residency is gated on the #389 substrate"), classified by
  `kind()` (content.rs:109), pinned un-addable (`addable()`, content.rs:83), tested at
  content.rs:331-335. NO `TabContent::Browser`, no `TabLayout::Browser`, no `FocusTab` Browser arm,
  no `PaneContent::Browser` anywhere (workspace.rs:341-355 carries Terminal/FileTree/CodeView/Git
  only) — this ticket cashes the declaration at the TAB layer and leaves the cell layer out.
- **The precedent is #400:** `TabContent::Cockpit(ContentId, RightSection)` (tabs.rs:32) — the id
  views the registry singleton, the tag keeps `cockpit_section`/`rail_section`/the `C=` codec writer
  registry-free; the resolve door `resolve_or_register_cockpit` + `CockpitIndex` + the pinned-anchor
  lifecycle + `release_cockpit_views` live in content.rs:228-302; open (app.rs:3965-3979), restore
  (app.rs:2241-2256), serialize (app.rs:5303-5304), close (app.rs:8497-8501, :8668-8670), render
  dispatch (app.rs:18455-18474). This ticket is its Browser sibling with the lifecycle fork inverted
  (D-OPEN-LIFECYCLE below).
- **The codec slot is reserved by design:** shell entries are `T=`/`C=`/`V=` (`serialize_shell`
  doc, grid_layout.rs:330; writer :334-399; reader :405-467) and an unknown tag is SKIPPED
  (grid_layout.rs:455, tested with `X=?` at :996-1009) — the bare `B` lands additively; old layouts
  restore unchanged, old builds skip `B`. The #389 Q4 two-layer analysis holds: at the SHELL layer
  even a URL would be framing-safe, but nothing needs one — the bare marker is the decision.
- **The status bar was built to fail INTO this:** the `FocusTab` wiring comment names "a future tab
  kind (e.g. an embedded browser — a roadmap pillar) fails the build INTO this label"
  (app.rs:20557-20558); the exhaustive matches at app.rs:20561-20593 and `focus_label`
  (status_bar.rs:81-105) each grow a Browser arm.

The work: `TabContent::Browser` id-bearing from birth, `Content::Browser` under the #394 registry
lifecycle (`ContentRegistry` — insert/acquire_view/release_view with the `#[must_use]` returns,
content_registry.rs:65/:74/:85), the bare `B` shell tag, a minimal open exposure (D-OPEN-OPEN-VERB),
and a placeholder pane render. Pure model + codec at cov/MSI 100; app.rs wiring masked. This is the
substrate #404 (URL seam) and #405 (webview) mount onto without touching the model again.

## Scope
### In
- **(a) `TabContent::Browser` id-bearing from birth** (D1): the variant joins the closed enum
  (tabs.rs:25-36); `Tab::browser` ctor beside `Tab::cockpit` (tabs.rs:166-173); `rail_section()` →
  `RailSection::Browser` (tabs.rs:127-133); the exhaustive accessor matches each grow their arm —
  `grid`/`grid_mut` (tabs.rs:194-207), `cockpit_section`/`cockpit_view`/`cockpit_content_id`
  (tabs.rs:216-236 — Browser answers `None`), `code_view`/`editor` (tabs.rs:240-269), `key_context`
  (tabs.rs:275-290 — empty, the cockpit posture: global bindings only), the `rail_rows`
  `(row_content, pane_marked)` match (tabs.rs:1103-1118 — the Cockpit shape `(None, false)`), plus
  new `is_browser`/`browser_content_id` accessors for the close-path collects and the codec writer.
  `FocusTab` gains a Browser variant + `focus_label` arm (status_bar.rs:64-105) wired at
  app.rs:20561-20593. The rail needs NO section work — tabs group by `rail_section()` generically
  (tabs.rs:1090-1091).
- **(b) `Content::Browser` residency**: the declared unit variant (content.rs:49) enters the
  registry lifecycle — a resolve door (open/re-open/second-project/restore converge on ONE live
  instance; census ≤1, the #396 construct-once→register→hand-back-an-id idiom) and a release seam
  whose lifecycle Phase 2 settles per D-OPEN-LIFECYCLE (recommended DROPPED-on-last-close; the v1
  unit payload drops INLINE like editors, content.rs:219-226 — no reaper analog, it owns nothing).
  `addable()` STAYS `false` (content.rs:77-85) — no cell arm this slice.
- **(c) the bare `B` shell tag**: `TabLayout::Browser` (grid_layout.rs:207-231), writer arm beside
  `C=` (grid_layout.rs:382-386 — a bare `B`, no payload, no `=` needed; Phase 2 confirms the exact
  byte against the `strip_prefix` reader symmetry), reader arm in the `restore_shell` chain
  (grid_layout.rs:427-455), serialize collect via `is_browser` beside app.rs:5303-5304, restore arm
  beside the cockpit's resolve+acquire at app.rs:2241-2256. Ids NEVER serialize; NO URL exists to
  serialize (the #389 Q4 bare-marker decision — re-derivation is #404's when a URL exists at all).
  The grid leaf `b` (split-cell layer, reserved bytes `\t\n\r,:=\x1f` — `breaks_grid_framing`,
  grid_layout.rs:314-316) is explicitly deferred.
- **(d) the placeholder pane**: a Browser tab renders a full-screen placeholder in the center region
  (the cockpit dispatch precedent, app.rs:18455-18474) — calm empty-state copy, themed, NO webview,
  NO URL read, total over the no-config case (Q3: no `.mcp.json` → the section still works; nothing
  here reads mcp_config at all this slice). React-first: prototyped in marley-web (below).
- **Minimal open exposure** per D-OPEN-OPEN-VERB's Phase-2 resolution (recommended: a 4th Browser＋
  row + `SectionAction::OpenBrowser` through `dispatch_section_verb`, app.rs:7269-7316) — without
  SOME verb the tab is unreachable and nothing above is drivable.
- **Regression pins:** `shell_codec_round_trips` (grid_layout.rs:680), `shell_codec_malformed_pieces_skipped`
  (:996), `restore_shell_parses_a_no_terminal_project` (:731), tabs.rs `cockpit_section_cases`/
  `open_or_switch_cockpit_cases`/`rail_section_maps_each_content_kind`/`rail_section_order_and_labels`/
  `rail_rows_groups_tabs_by_section_in_fixed_order` (:1591/:1613/:1867/:1881/:1902), status_bar
  `focus_label_non_terminal_tabs` (:158), content.rs `addable_kinds_are_terminal_and_editor`/
  `payloadless_kinds_classify` (:316/:331 — the latter CHANGES only if D5 resolves that the variant
  gains a payload, which D5 says it must not) — all green, byte-identical wires.

### Out (explicitly deferred)
- **The webview itself — #405** (wry-as-child WKWebView, the z-order/overlay shim, position/resize
  sync, pinned-origin nav; #389 Q1/train slice-3). Zero platform code this slice.
- **The URL seam — #404** (the `.mcp.json`-origin derivation, the endpoint→web-base mapping, Q3's
  keyed-on-active-root re-derive). The placeholder needs NO URL; nothing here reads mcp_config.
- **The grid leaf `b`** (Browser as a split CELL) + any `PaneContent::Browser` arm (workspace.rs:341)
  + flipping `addable()` — the #389 Q4 deferred layer, gated on the exposure follow-up.
- **The CDP agent-browser lane** (#389 Q2 — a separate headless-Chromium pillar, not this section).
- **Any general user-typed-URL browser** (needs the percent-encoding/side-table codec #389 Q4
  defers; v1 content identity is the derived Forge origin, arriving at #404/#405).

## Reference (§20)
**N/A — Marley-specific composition model** (the #388 model A + the #389 embed train are chad's own;
no reference app to match for the residency/codec mechanics). The honest adjacent behaviors,
research-level only (re-verified today, not carried over):
- **Zed (behavior map):** docs/zed_architecture/subsystems/07-workspace-panes-palette.md — the
  citizenship frame this slice extends to a new kind: §1.3 (:126) the one `Pane` type serves center
  and dock contents alike; §1.4 (:143, :170-171) "*dozens* of unrelated views — are all first-class
  pane items"; §1.6 (:199) panels as peer `Panel` impls. The comparison table's two GAP rows
  (:266 pane-content, :269 dock-contents "no uniform panel citizenship") are the rows the
  registry train closes kind-by-kind — #400 closed cockpit, this closes Browser, by Marley's OWN
  route (closed enum + `ContentId` registry, `AD-claude-pane-content-id-registry-001`). Notably
  **Zed ships NO embedded-browser pane item** — the map's item roster (editor, terminal, search,
  diagnostics, image/markdown/svg previews) contains none, and the map itself cites Marley's
  "embedded-browser intake" as a Marley-side trigger (:19, :279, :554). Behavior observed from the
  MAP, never Zed source (§20 — a reworded translation is still a derivative work).
- **Warp:** checked — swept docs/warp_architecture/subsystems/00-07 + crates notes for
  browser/webview behavior: every hit is **Warp-on-Web** (Warp compiled to WASM and mounted IN a
  browser — warp_web_event_bus.md, serve-wasm.md, warp_logging.md), the inverse of a browser pane
  inside the app. 04-agent-ai-mcp.md: no browser-pane behavior. No owner; no Warp (AGPL) source
  consulted.
- **Published behavior:** VS Code's **Simple Browser** (a built-in webview-backed browser tab opened
  by command, no persistence of arbitrary state promised) is the convention analog for "a browser
  surface as an ordinary tab citizen" — adopted at the published-docs level only. Zed's ABSENCE of
  one (above) is itself a data point: no ecosystem pressure fixes the shape, so Marley's
  bare-marker/derived-URL design stands on its own config lineage.

### Prior art
1. **Behavior maps — checked; the Zed §1.3/§1.4/§1.6 + GAP-row citations above are the yield**
   (re-verified at today's line numbers). Warp: no owner (Warp-on-Web only — where I looked listed
   above). Research, not source.
2. **Published material.** VS Code Simple Browser (above); the #389 doc's substrate research (wry /
   objc2-web-kit / CEF published docs) belongs to #405, not this slice — nothing here embeds.
3. **Our permissive deps — checked, no owner; the adoption leg is IN-HOUSE and dominant.** gpui
   (Apache-2.0) renders divs — no tab/registry/codec seam to adopt; ropey/regex/alacritty_terminal/
   tree-sitter own nothing near content residency. The seams that DO own this are shipped Marley
   code this ticket reuses rather than reinvents: `content_registry.rs` (#394) owns the refcounted
   lifecycle (insert/acquire_view/release_view + the `#[must_use]` contracts,
   content_registry.rs:65-95, per `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001`);
   `content.rs` (#396/#397/#400) owns the birth/close idioms this slice's resolve/release seams
   mirror (`resolve_open` :194, `release_editor_views` :219, `resolve_or_register_cockpit` :267,
   `release_cockpit_views` :289); `grid_layout.rs` owns the codec discipline (#163 shell framing +
   unknown-tag skip, #205 write-time guards, #396 ids-never-serialize); `tabs.rs` owns tab
   citizenship + the section rail (#385/#390). The #400 spec is the direct template ticket.

## React-first (parity)
**UI-AFFECTING — Zone A (rail Browser-section tab rows) + Zone B (the placeholder pane).** Contract:
marley-web/docs/MARLEY-PARITY.md (zones :21/:44; port map :585; shared vocabulary :592-632).
- **Zone A — `artifacts/marley-ide/src/components/LeftRail.tsx` (EXISTS):** the POC's Browser
  section (:374-407) already renders an ALWAYS-present "Web" row (:388-393) — a POC-only state the
  parity doc flags as "a tab Marley has no equivalent for" (:187-188). The prototype turns it into a
  REAL tab row with Marley's tab semantics: born on open (absent until then), closable ×,
  active-highlight, participating in the rail tab filter — alongside the existing cockpit rows
  (`filteredBrowserTabs`, :134). State rides `src/App.tsx` (EXISTS — `activeBrowserTab`/
  `openBrowserTabs`/`browserUrl`, :117-119); the open verb rides the Browser＋ menu (the POC's
  ContextMenu section rows, mirroring `SECTION_BROWSER_ITEMS`). The row LABEL ("Browser" vs the
  POC's current "Web") is settled IN the prototype — the row is being born here, so the React stage
  is the design surface; the settled label ports 1:1.
- **Zone B — the placeholder pane:** `artifacts/marley-ide/src/components/views/BrowserView.tsx`
  (EXISTS) today renders the FULL simulated browser for its 'web' tab — URL bar, mock marley.dev,
  Manager chat (:242-396) — all FUTURE-slice content (#404/#405/CDP-adjacent, the POC designing
  ahead). The #403 prototype adds the PRE-WEBVIEW placeholder state: recommended as a small separate
  `views/BrowserPlaceholder.tsx` (**TO-CREATE**) so the POC's forward-looking web simulation
  survives untouched; alternatively a stripped state inside BrowserView.tsx — Phase 2 confirms
  which. The placeholder is what Marley ships this slice: calm empty-state copy, no URL bar, no
  page.
- **Plan line:** build & visually verify in marley-web first (`pnpm --filter @workspace/marley-ide
  run dev` → localhost:5173), then port 1:1. React iterates in seconds; the cargo build does not —
  the look is settled BEFORE the port. **Validate captures the React↔Marley parity pair** (rail rows
  + placeholder pane, both sides).
- **Vocabulary tie:** ContentId stays aligned with the POC's `PaneItem` stand-in
  (MARLEY-PARITY.md § Shared vocabulary, :592-599) — a Browser tab is a view row holding an id, so
  the later cell-exposure slice rides the same AddToPane/PaneItem vocabulary with no new concepts.

## Locked-In Decisions
- **D1 — id-bearing from birth, and the variant discriminant IS the tag: `TabContent::Browser(ContentId)`.**
  The #400 `(id, tag)` shape carried `RightSection` because THREE sections share one variant and the
  codec writes the per-instance key (`C=<key>`) — the tag bought registry-free codec/rail/accessor
  purity (tabs.rs:32/:216 comments). Browser v1 has ONE kind and a bare payload-less codec tag, so a
  separate tag field would duplicate what the enum discriminant already discriminates: `matches!`
  gives `is_browser`, the match arm gives `rail_section`/`focus_label`/the `B` writer — all pure, no
  registry read, zero redundancy. If a future multi-browser identity arrives (per-root, user-typed),
  the tag slot grows THEN, with the codec payload it would then require.
- **D2 — the bare `B` tag; ids never serialize; nothing URL-shaped persists.** The #389 Q4 decision
  binds verbatim: v1 Browser content IS the project's derived Forge URL, so no URL enters persisted
  state at either codec layer — the shell entry is a bare `B` mirroring Cockpit's `C=` (writer
  grid_layout.rs:382-386, reader :449-450), trivially framing-safe; restore rebuilds the TAB and
  resolves the registry instance (the #163/#205/#396 rebuild-from-shapes discipline —
  content_registry.rs:20-22 "never serialized"). This slice has no URL to re-derive; when #404
  lands, restore re-derives from the active root and a changed Forge URL is picked up automatically
  — the bare marker is what makes that free. The grid leaf `b` stays deferred.
- **D3 — back-compat is additive, pinned at today's skip.** `restore_shell` skips unknown entry tags
  (grid_layout.rs:455; proven with `X=?` at :996-1009), so an old layout restores unchanged under
  the new build and a `B`-bearing layout degrades to a skipped entry under an old build — no
  migration, no version bump. Every shell WITHOUT a Browser tab serializes byte-identical to today
  (the existing wire fixtures, e.g. grid_layout.rs:716, stay exact).
- **D4 — placeholder pane only; ZERO webview/platform code this slice.** No `wry`, no
  `objc2-web-kit`, no raw-window-handle use, no new dependency, no mcp_config read. The render is a
  masked app.rs arm beside the cockpit dispatch (app.rs:18455-18474) drawing themed empty-state
  copy; it must be total over no-config (it reads nothing that could be unconfigured). The #389
  slice-1 proof-of-embed and slice-3 pane are other tickets.
- **D5 — `Content::Browser` is DECLARED and CASHED, not re-declared.** Verified truth: the variant
  ALREADY EXISTS as a payload-less unit (content.rs:49, with `ContentKind::Browser` :67, `kind()`
  :109, `addable() == false` :83, tests :316-335) — #396's "all 6 kinds declared" DID ship. #403
  adds NO enum variant and NO payload (the v1 placeholder owns nothing — identity is the whole
  content, the cockpit D-OPEN-STATE-MIGRATION posture taken further: not even an identity payload is
  needed at one instance); it wires the declared variant into the resolve/release lifecycle. The
  payload slot grows at #405 when something ownable (the webview session) exists — and THAT is when
  the drop-on-last-close lifecycle pays (D-OPEN-LIFECYCLE).

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-IDENTITY** — one Browser instance app-wide, or one per active root? **Recommend ONE
  app-wide.** Evidence: the future URL derives from the ONE boot-time `.mcp.json` resolution keyed
  on the restored ACTIVE root (`mcp_json_path`, mcp_config.rs:28-40; consumed once at
  app.rs:2400-2410 feeding BOTH the forge client and the brain endpoint — the "one config, N
  consumers" lineage the browser URL joins as consumer three). One resolution ⇒ one origin ⇒ one
  instance; two projects' Browser tabs are two views of it (the cockpit cross-project shape,
  content.rs:496-535 tests). A per-root identity would imply per-root URL derivation that the
  shipped lineage does not do. Revisit trigger recorded for #404: if the URL seam re-derives on
  active-root SWITCH, identity follows the evidence then.
- **D-OPEN-LIFECYCLE** — pinned (the cockpit anchor) or DROPPED on last close? **Recommend DROPPED.**
  The cockpit pin exists because standing references outlive tabs (the top-bar strip + persisted
  `right_section`) and a cockpit owns nothing reapable (content.rs:229-231) — neither holds for
  Browser: no standing reference exists (the rail home is derived from open tabs), and the FUTURE
  payload is precisely a reapable resource (the #389 train's slice-4 names webview teardown
  "mirror the PTY reaper contract"), so drop-on-last-close is the lifecycle that stays correct when
  the webview arrives; pinning now would bake in a leak-shaped contract needing reversal. The v1
  unit payload makes either safe TODAY (a dropped unit is a no-op drop, inline like editors —
  content.rs:219-226); choose the one the future needs. Consequence: no index/anchor — the resolve
  door is find-live-or-insert (the `resolve_open` shape :194-213, scanning `kind()`), and
  close-then-reopen legitimately mints a fresh id (identity is the kind, not the id — ids are
  session-local anyway, content_registry.rs:20).
- **D-OPEN-OPEN-VERB** — how does a user OPEN the Browser tab? Verified today: the ONLY section open
  gestures are the ＋ menus (`section_items`, context_menu.rs:195-202; Browser＋ = 3 cockpit rows,
  :158-171, dispatched via `dispatch_section_verb` → `open_cockpit_tab`, app.rs:7301-7304) — no
  palette verb opens cockpit sections. **Recommend: a 4th Browser＋ row** ("Browser", a new
  `SectionAction::OpenBrowser` beside `OpenCockpit`, tabs.rs:90-103) through the same dispatch —
  the section finally offering its namesake; keep item 0 = Forge (the #387/#393 default,
  context_menu.rs:537-538 pins it — changing the default is not this ticket's call). A palette verb
  is optional garnish Phase 2 may add or defer. The React prototype exercises the chosen verb.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Browser tab exists, the system shall classify it purely from the variant — `rail_section()` → `RailSection::Browser`, `focus_label` → the Browser word, `key_context` → the cockpit posture, `is_browser`/`browser_content_id` answering and every other accessor (`grid`/`cockpit_*`/`code_view`/`editor`) answering `None` — with NO registry read in any of them, and the tab's `ContentId` shall resolve `Content::Browser` through the registry at the render dispatch. | units over every exhaustive match that grows an arm (tabs.rs accessors + rail_rows row-content, status_bar `focus_label`, content.rs classify); exhaustive per-variant, no catch-all (`PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001`) |
| REQ-002 | WHEN a Browser tab is opened, re-opened after a switch, opened in a second project, or restored from a persisted shell — the live-instance cases per the settled D-OPEN-IDENTITY/LIFECYCLE — the system shall resolve the SAME live `ContentId` (registry census ≤1 Browser entry ever) and shall never serialize an id. | resolve-door units: open/re-open/second-project/restore same-id; census ≤1 pin; post-drop reopen mints a fresh id (the dropped-lifecycle case); wire asserts contain no id bytes |
| REQ-003 | WHEN a shell WITHOUT a Browser tab serializes, the wire shall be byte-identical to today's (regression); WHEN a shell WITH a Browser tab round-trips, the bare `B` entry shall restore a Browser tab with no URL and no payload bytes, mixed with `T=`/`C=`/`V=` entries in order. | grid_layout units: existing exact-wire fixtures unchanged (:716 et al.); new mixed round-trip incl. `B` with exact-bytes assert; `restore_shell(serialize_shell(x)) == x` with a Browser tab |
| REQ-004 | WHEN a layout persisted by a pre-#403 build restores, the system shall rebuild it exactly as today (the additive pin — no arm reorder observable), and WHEN any unknown tag is encountered it shall still be skipped. | `shell_codec_round_trips` (:680) + `shell_codec_malformed_pieces_skipped` (:996) green unchanged; a legacy-fixture restore-equality unit; reader-chain order pinned by the mixed-entry test |
| REQ-005 | WHILE the active tab is a Browser tab, the system shall render the placeholder pane in the center region — no webview, no URL read, total over the no-`.mcp.json` case — and the Browser rail section shall list the tab row (born on open, gone on close). | driven/headless capture of the rail row + placeholder (or the recorded env-blocked protocol: units + the #204/#205 carry); a no-config boot case in the drive; pure placeholder-copy fn (if any) unit-covered |
| REQ-006 | WHEN any Browser-tab close path runs (close-tab, close-project, any tab-drop path the #400 audit enumerated), the system shall route it through `release_view` with the `#[must_use]` return consumed at the site, acquire/release leak-paired, and — per the settled lifecycle — the last view's drop shall dispose the v1 unit payload inline (no reaper, nothing owned). | leak-pairing units (acquire/release counts across close-tab/close-project/close-then-reopen); double-release safety; §18.1 site audit per `PR-claude-registry-release-returning-a-resource-must-be-must-use-001` |
| REQ-007 | WHEN the change lands, the React-first parity pair shall exist: the marley-web prototype (rail rows + placeholder) built and visually verified FIRST, the Rust port matching it 1:1, both captured at Validate. | the enforce-react-parity.sh contract: prototype commit/screens in marley-web + the Marley capture pair recorded in the Phase 4 notes |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the new tabs.rs arms + `Tab::browser` + `is_browser`/
`browser_content_id`, the status_bar Browser arm, the grid_layout `B` writer/reader arms, the
content.rs resolve/release browser seams over the #394 pure registry, and any open/label decision
fn. MASKED: the app.rs shims (open verb dispatch, restore arm, serialize collect, close-path
plumbing, the placeholder render — app.rs is coverage-excluded). Typed inputs; no `unwrap` on
restore-derived paths. The app.rs edits land near the masked pump/boot/render shims — the
skip-detach trap's home turf (5th strike `BF-claude-skip-detach-pump-fleet-live-001`): re-run
`cargo mutants --list -f` on the ACTUAL touched files after placement and re-verify neighboring
`#[mutants::skip]` bindings (`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`).
Every `release_view`/`acquire_view` call site consumes the `#[must_use]` return
(`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`).

## Phase Plan
- **P2 Design** — settle the three D-OPENs against the shipped tree (IDENTITY: confirm the one-boot
  `.mcp.json` resolution still holds at app.rs:2400; LIFECYCLE: dropped, with the resolve-door shape
  — find-live-or-insert vs an index; OPEN-VERB: the 4th ＋ row + `SectionAction` variant, palette
  yes/no); exact signatures (`TabContent::Browser(ContentId)`, `Tab::browser`, the resolve/release
  seams, the `B` byte); the file manifest + per-REQ test plan; enumerate every tab-drop path from
  the #400 close-path audit and map each to its release; confirm the React-first zone/files
  (BrowserPlaceholder.tsx to-create vs a BrowserView state) against the POC's current tree.
- **P3 Implement** — the React-first stage FIRST (LeftRail row lifecycle + ＋ row + placeholder at
  localhost:5173, visually verified); then the pure seams (tabs.rs variant + arms, status_bar arm,
  grid_layout codec arms, content.rs resolve/release); then the masked app.rs wiring (open verb,
  restore, serialize, close routing, placeholder render dispatch) — the 1:1 port of the settled
  prototype.
- **P3.5 Inspect** — adversarial critics: acquire/release leak pairing over EVERY close path incl.
  restore + project-close + close-then-reopen; restore-rebuild correctness (a `B` restore resolves
  exactly one instance across N projects); codec byte-identity (no-Browser shells + legacy
  fixtures); placeholder totality (no-config, no-terminal, zero-tab neighborhoods per #391/#392/#395);
  skip-detach re-check on the touched app.rs regions; §20 provenance.
- **P4 Validate** — write + RUN the units per REQ (suites named in the Verify column); `cargo
  mutants --list -f` on the actual touched files; gate green (`--diff`), cov/MSI 100 on the pure
  seams; the REQ-005 drive (or the documented env-blocked fallback); all named regression suites
  green byte-identical; the REQ-007 parity captures recorded.
- **P5 Complete** — CHANGELOG; pane-composition-model.md slice-8 gains its Browser-half shipped note;
  embedded-browser-model.md train step 2 ticked (shipped M29 #403); AAR capture; archive; close #403.
