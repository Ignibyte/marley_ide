---
pipeline_id: 9bd8403a-5aae-4e90-b90b-930932fb0a05
ticket: forge#405 (425c8332-8949-41bc-b252-008c922d85a0) · local docs/planning/tickets/open/TICKET-405-forge-url-pane.md
aar_id: (deferred — forge MCP unreachable in the promoting session; open from a forge-wired session)
status: Phase 5 — Complete PASS (2026-08-08); shipped — gate GREEN [diff] 15/15, headed sweep pixel-verified
title: The Forge URL pane — the wry child wired to the pane rect + the z-order shim (the #389 train slice-3)
type: feature
milestone: M29
references:
  - docs/marley_architecture/embedded-browser-model.md
  - docs/marley_architecture/pane-composition-model.md
  - docs/marley_architecture/app_shell.md
  - docs/planning/pipeline/completed/389-spike-embedded-browser.spec.md
  - docs/planning/pipeline/completed/400-cockpit-residency.spec.md
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/layout.rs
  - crates/marley_app/src/context_menu.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/grid_layout.rs
---

## Title
The #389 train slice-3 — the payoff: **Forge opens in the browser.** A wry child WKWebView attaches
through the gpui window's `raw-window-handle` at the Browser pane's rect, tracks that rect through
every layout change, loads the #404-derived Forge web origin (re-derived on every mount/restore,
never persisted — the #403 codec contract), pins navigation to that origin, and — the load-bearing
piece — **hides itself whenever any gpui overlay is up**, because a native child composites ABOVE
gpui's single Metal `GPUIView` scene (gpui paints the whole frame into one drawable; the child is a
subview over it — embedded-browser-model.md Q1). The webview is a masked platform surface
(`mutants::skip` per §0 ACCEPTED-UNTESTABLE), driven-validated; every masked fn is a thin adapter
over pure decision seams.

**HARD GATE + promote rule.** This spec BINDS to #402's recorded verdict and to its siblings'
shipped shape. `/work` must NOT promote this pipeline to active while ANY of:
- **#402** (spike, a17801ff-b56e-4a7d-ace4-b3252c49147e) has no verdict recorded in
  embedded-browser-model.md + the updated `AD-claude-embedded-browser-substrate-001`. A recorded
  **GO** → D1 stands and slices 3–4 proceed. A recorded **NO-GO** → the substrate pivots to
  CEF-OSR/`paint_surface` (gpui `window.rs:3181`; `core-video` already in the lockfile) and THIS
  SPEC RE-DESIGNS at Phase 2 before any implement — the rect/hide/origin decision SEAMS survive the
  pivot (a composited texture needs no hide-shim; D2 collapses to a recorded win), the adapter does
  not.
- **#403** (f7bf9657-b4c1-4fd6-b346-584660daf256) is unshipped — this spec consumes its
  `TabContent::Browser`/`Content::Browser` residency, bare `B` shell tag, and placeholder pane.
- **#404** (e84add90-0452-4bfa-9772-4ca04a54d5ae) is unshipped — this spec consumes its
  `forge_web_base` origin seam (and D3 shares its origin type).
#406 (f72268be-1018-4e0b-8843-10d70cd718a2, slice-4: nav chrome/loading/error/teardown-hardening) is
downstream, not a gate. wry is ABSENT from Cargo.lock today (verified 2026-08-06: zero `wry`
entries; `raw-window-handle` 0.6.2 present at Cargo.lock:4639) — **#402 is the one place wry enters
the tree**; this spec inherits it, never adds it.

## Scope
### In
The five halves, each grounded in today's verified tree:
- **(a) Attach + rect sync.** Build the wry child against the gpui window handle
  (`WebViewBuilder::build_as_child<W: HasWindowHandle>` — on macOS "NSView subview of parent's
  content view", docs.rs; gpui `MacWindow: HasWindowHandle` returns `RawWindowHandle::AppKit`, gpui
  `window.rs:1548` per the #389 study) and position it at the Browser pane's rect. That rect IS
  `center_bounds` (app.rs:18407-18412) — pure arithmetic over `region_widths` (app.rs:17622 ←
  layout.rs:108) + `files_w` + `content_band` (layout.rs:126) — and the cockpit tab already renders
  its full-screen body at exactly this rect (app.rs:18468-18471): the Browser tab inherits the same
  placement. The rect changes on: window resize (the render reads `window.viewport_size()` each
  frame), left/right dock toggle (`dock()` app.rs:10166; right-dock width note :18290), Files-panel
  toggle + edge drag (`files_open` :460, `dragging_files_edge` :476, drag start :18377 / persist on
  release :18395), and tab/project switch. Sync mechanism is D-OPEN-RECT-SYNC; the in-house
  precedents are `editor_geom: Rc<Cell<EditorFrameGeom>>` (:513, written from the paint at
  :6577/:6594 — the #352 frame-geometry lesson: geometry consumed off-render is CAPTURED per frame
  and self-heals a frame later) and the PTY R37 diff-before-apply ("resize only when the cell grid
  actually changed", :18614).
- **(b) THE Z-ORDER SHIM — the primary mitigation, driven by ONE seam.** Hide the webview while any
  gpui overlay is up; reappear on dismiss. The overlay set is the **verified inventory below** (D2)
  — the design doc's "palette / context menus / find" is a 3-item sketch of a **26-state** reality;
  enumerating it honestly is a deliverable of this spec, because a missed overlay = invisible UI,
  this feature's worst failure mode. Also: a **backgrounded** Browser tab's webview never paints —
  hidden whenever the Browser tab is not the active tab of the active project (`try_active_tab`
  app.rs:10642; the cockpit-arm dispatch precedent :18455).
- **(c) Load on mount + restore.** On Browser-tab mount and on restart-restore, load exactly the
  #404 `forge_web_base` origin, re-derived from the active root (`mcp_json_path` mcp_config.rs:28;
  the one-config-three-consumers site app.rs:2403 → `forge_endpoint_from` :2409 + the #376
  `endpoint_for_brain` decision) — never read from persisted state: the #403 tab persists as the
  bare shell tag `B` (`serialize_shell` grid_layout.rs:334 / `restore_shell` :405), mirroring
  Cockpit's `C=<key>`.
- **(d) PINNED-ORIGIN nav.** wry's `with_navigation_handler(impl Fn(String) -> bool)` — `false`
  rejects (docs.rs) — backed by a pure target-origin equality fn sharing #404's origin type. No
  address bar; a general browser is a later, separately-reviewed capability.
- **(e) No-config → placeholder.** `forge_web_base` = `None` (no `.mcp.json` / unresolved /
  unparseable) → NO webview is constructed; the #403 placeholder stays (never a broken page — the
  #384-sibling misconfigured-state pattern).

Plus: **focus/IME per #402's recorded findings** (gpui routes keys through
`makeFirstResponder(native_view)`, gpui `window.rs:776`; wry offers `focus()`/`focus_parent()` —
policy is D-OPEN-FOCUS-HANDOFF). **Coexistence:** the native forge cockpit (`cockpit_body`,
app.rs:5854, Forge arm) keeps working — additive only; the native FLEET rail (`fleet_rail_body`,
app.rs:1076) is NEVER retired ("desktop for hands, web for intents").

**The overlay inventory (verified against app.rs @ 35fd2c7, 2026-08-06).** The overlay-draw region
is ONE contiguous render tail (app.rs:19781–21117) of independent `if` branches appended above the
pane scene. Every floating overlay, its state field, and its draw site:

*Class A — floating, reachable while a Browser tab is active (14 states):*

| RootView state (line) | overlay | draw |
|---|---|---|
| `palette_open` :163 | ⌘⇧P command palette (centered, `.occlude()`) | :19781 |
| `naming_workflow` :181 | workflow-naming card | :19843 |
| `naming_pane` :185 | #399 arrangement-naming card | :19872 |
| `fleet_dispatch_draft` :448 | #378 fleet dispatch draft card | :19921 |
| `agent_launcher` :166 | agent launcher | :19947 |
| `finder_open` :192 | file finder | :20012 |
| `history_open` :200 | history search | :20056 |
| `forge_open` :414 | #69 forge sprint card | :20096 |
| `fleet_open` :416 | #68 ⌘⇧E local-agents overlay | :20129 |
| `diff` :497 | diff viewer | :20358 |
| `find_open` :621 | terminal find bar (absolute, top 8px) | :20419 |
| `efind_open` :547 | ⌘F editor find bar (self-closes w/o an active editor, :15647 — included anyway) | :20456 |
| `status_flash` :615 | #77 transient bottom pill (click-through, ~2s) — membership is D-OPEN-FLASH | :20531 |
| `context_menu` :462 | EVERY menu kind — one state covers all `MenuKind`s (#166/#175/#393 plus-menu; context_menu.rs:225/:231) | :20999 |

*Class B — editor-anchored cards + pickers (12 states; no editor surface is visible while a
whole-tab Browser is active, so these are unreachable-in-practice in v1 — membership is
deliberately UNCONDITIONAL so that reachability reasoning can rot without consequence, and the
deferred split-cell arm inherits a correct predicate):*

| RootView state (line) | overlay | draw |
|---|---|---|
| `hover_card` :224 | #311 LSP hover card | :21072 |
| `code_action_menu` :256 | #323 code-action picker | :21076 |
| `def_picker` :240 | #312 definition picker | :21079 |
| `open_references` :244 | #317 ⇧F12 references picker | :21083 |
| `open_symbols` :272 | #325 ⌘T workspace-symbol picker | :21087 |
| `open_file_symbols` :274 | #304 ⌘⇧O file-symbol picker | :21091 |
| `open_search` :301 | #326 ⌘⇧F project-search picker | :21095 |
| `open_problems` :319 | #327 ⌘⇧M problems panel | :21099 |
| `completion_menu` :385 | #313 completion popup | :21103 |
| `signature_card` :264 | #324 signature-help card | :21107 |
| `renaming_symbol` :251 | #322 inline-rename draft | :21111 |
| `goto_line` :254 | #302 ⌃G go-to-line chip | :21115 |

*Class C — pane-rect-keyed, EXCLUDED by construction (cannot draw unless the active tab owns a
grid, which a Browser tab never does — `active_is_terminal` :18601 empties `rect_list`):* the ⌘⇧C
git panel (#115/#125 — keyed on a Git pane in the ACTIVE grid's `rect_list`, :20185-20191) and the
terminal inline completion (`completion` :468 — `rect_list` lookup; pane absent → self-clears,
:20899-20901).

*Class D — chrome-anchored inline states, NOT floating overlays (excluded, recorded):*
`renaming_tab` :486 (rail-row inline edit, :17834), `session_search_focused` :609 (rail search box,
:17642), `commit_focused` :500 (inside the Class-C git panel), `top_search_focused` :503 (top bar),
the #228 focused-pane border (:19767), sticky headers / scrollbar thumbs (pane-internal). All draw
outside `center_bounds` or inside a Class-C surface.

**The predicate = Class A + Class B = 26 states.** *(P3.5 AMENDMENT: the inspect adversary's
blind re-derivation added a 27th — the #141 top-search RESULTS DROPDOWN, drawn on
query-non-empty alone, overflowing the top bar over the content area, fully reachable beside a
Browser tab; the P2 sweep had misfiled it with the Class-D chrome box it hangs from. It also
found the references card's SEARCHING arm draws while `open_references` is None — captured by
widening that field's builder gate, not a new state. **The predicate = 27 states.**)*

### Out (explicitly deferred)
- **Nav chrome + lifecycle** — reload verb, loading state, error page, teardown-hardening
  (detach-off-the-render-path, the PTY-reaper analog) = **#406** (slice-4). v1 teardown is the
  registry release path #403 ships; #406 hardens it.
- **The CDP / agent-browser lane** — a separate headless-Chromium substrate (#389 Q2); not this
  pane, not this milestone.
- **General browsing / address bar / user-typed URLs** (and the URL-persisting codec that would
  need) — later, separately reviewed (#389 Q3/Q4).
- **Cockpit retirement** — named-criteria-only (#389 Q3); nothing here retires anything.
- **The grid split-cell arm** — a Browser as a split CELL (`PaneContent`/grid `b` leaf) stays with
  the #388 registry kind's deferred exposure; v1 is a whole TAB.
- **Constraining overlays away from the webview rect** — the partial secondary mitigation stays a
  documented idea, not a built system (centered overlays can cover any pane rect — #389 Q1).

## Reference (§20)
**N/A — Marley-specific** (chad's "Forge opens in the browser", 2026-07-22; roadmap Phase E;
embedded-browser-model.md is the governing design). No reference app to match: **no Warp analog and
no Zed analog** — verified 2026-08-06 by recursive sweep of `docs/warp_architecture/` and
`docs/zed_architecture/` for webview/WKWebView/embedded-browser: zero hits (and
embedded-browser-model.md's own header records "No Warp/Zed analog"). The honest published-behavior
adjacent: **VS Code's Simple Browser** (built-in, MIT; its README: "Provides a very basic browser
preview using an iframe embedded in a webview… primarily meant to be used by other extensions for
showing simple web content") — the published convention this pane matches at the BEHAVIOR level: a
deliberately minimal, pinned embedded browser surface for first-party content, not a general
browser. No Warp (AGPL) / Zed (GPL) source consulted.

### Prior art
1. **Behavior maps — checked, no owner.** `docs/warp_architecture/` + `docs/zed_architecture/`
   swept for embedded-browser behavior (see above): zero hits. Research legs end at the maps; no
   captures to cite.
2. **Published material — the wry API surface, verified live (docs.rs/wry, 2026-08-06):**
   `build_as_child<W: HasWindowHandle>(&W) -> Result<WebView>` (macOS: NSView subview);
   `with_bounds(Rect)` — **defaults to 200×200 at 0,0** (a first-frame hazard: the child must be
   built with the real rect or hidden until the first sync); `WebView::set_bounds(Rect) ->
   Result<()>` ("only effective if the webview was created as a child"); `set_visible(bool) ->
   Result<()>`; `load_url(&str) -> Result<()>`; `with_navigation_handler(impl Fn(String) -> bool)`
   — `false` rejects the navigation (D3's hook); `with_focused(bool)`, `focus() -> Result<()>`,
   `focus_parent() -> Result<()>` (D-OPEN-FOCUS-HANDOFF's verbs). **`reparent` is gtk-bound** —
   remove-from-superview is NOT in wry's portable API (feeds D-OPEN-HIDE-MECHANISM). Webview
   drop/teardown semantics are UNDOCUMENTED — exactly why hardening is #406. Plus the VS Code
   Simple Browser convention above. (#389's sweep in embedded-browser-model.md Q1/Q5 remains the
   deeper published-material record: engine/license/weight table, wry 0.55.1 Apache-2.0/MIT.)
3. **Our permissive deps — gpui (Apache-2.0, adoption; crates.io 0.2.2) + the in-house seams.**
   The #389 study's gpui findings stand: one self-composited Metal `GPUIView` per window (gpui
   `platform/mac/window.rs:111`), `HasWindowHandle` → `RawWindowHandle::AppKit` (:1548), keys via
   `makeFirstResponder` (:776), `paint_surface` (:3181) + in-lockfile `core-video` = the NO-GO
   fallback already in-tree. **The overlay-signal adoption finding: nothing to adopt.** Marley does
   not use gpui's `deferred()`/anchored overlay machinery at all — zero `deferred(` calls in app.rs
   (verified); its overlays are its OWN absolute-positioned render tail (:19781–21117). gpui
   therefore has no ready "overlay is up" signal; we adopt the SHAPE (one batched above-the-scene
   layer) and derive the SIGNAL from Marley's own state. The in-house pattern to mirror is
   **`text_input_blocked()` (app.rs:10597-10626)** — M16 #267's one-predicate choke point ("One
   predicate = one choke point; a NEW overlay only needs a line here") — but its SET is the wrong
   set (see D2): `overlay_is_up` is a NEW sibling fn, not a reuse. The other in-house seams this
   spec consumes rather than reinvents: #402's probe findings (the direct input), #404's origin
   type + derivation, #403's residency/placeholder/codec, the #394 `ContentRegistry` lifecycle,
   `editor_geom` + PTY-R37 rect-sync precedents (Scope a).

## React-first (parity)
N/A — no UI delta: the pane content IS an external web page — a native WKWebView compositing the
Forge web app's own pixels — not Marley-drawn chrome; there is nothing for React to prototype at
localhost:5173 that Marley draws (an iframe stand-in would rehearse none of the native z-order/rect
mechanics this ticket is about). The parity contract's own boundary carries the arm:
MARLEY-PARITY.md's Zone definitions ("The surfaces are free. What lives *inside* a pane is where we
design" — Zone B, :44; Zone A freezes Marley-drawn chrome/overlays, :21) plus its explicit
no-counterpart precedent (`utils/commandSimulator.tsx` — "no counterpart — POC-only", port-map
tail). The Marley-drawn pixels this feature relies on — the Browser rail row and the
placeholder/empty state — shipped with **#403**, whose spec carries its own UI-AFFECTING parity arm.
**CONDITIONAL — Phase 2 re-confirms (the #400 pattern). The flip protocol:** if Phase 2 (or any
later phase) adds ANY net-new Marley-drawn chrome — a pane header row, a loading bar, an error card,
a visible hide-state veil while an overlay is up — this section flips to **UI-AFFECTING (Zone B
surface)**: prototype it in marley-web FIRST (a `views/BrowserPane.tsx` beside the port map's
`views/BrowserView.tsx` ↔ `right_dock.rs` row, :585), visually verify at localhost:5173
(`pnpm --filter @workspace/marley-ide run dev`), then port 1:1; Validate captures the React↔Marley
parity pair per the enforce-react-parity.sh contract.

## Locked-In Decisions
- **D1 — substrate = wry-as-child WKWebView, per #402's recorded GO.** The gate (see Title): this
  spec binds to `AD-claude-embedded-browser-substrate-001` as updated by #402. A recorded NO-GO →
  the named pivot (CEF-OSR / gpui `paint_surface` + in-lockfile `core-video`) and a Phase 2
  re-design of this spec before any implement — the pure seams (rect mapping, origin equality,
  mount decision) carry over; the adapter and D2's shim do not (a composited texture layers under
  gpui overlays for free — that death would be a recorded win, not a deviation).
- **D2 — the hide-shim is the PRIMARY z-order mitigation, driven by ONE decision seam.** A pure
  `overlay_is_up(…) -> bool` enumerating the 26 Class-A+B states of the inventory above; the webview
  sync consults exactly this one fn — **a per-overlay ad-hoc patchwork is banned by construction**
  (no hide call anywhere except the single sync site). The truth this rests on, verified: NO single
  boolean means "some overlay is drawing" today — the render tail is ~28 independent branches — but
  the tail is one contiguous region (enumerable from one place) and `text_input_blocked`
  (app.rs:10597) proves the one-predicate discipline works. The two sets differ in BOTH directions
  (why this is a NEW fn): `text_input_blocked` deliberately omits `completion_menu` (its :10598
  NOTE — printables fall through by design) and never lists `hover_card`/`signature_card`/
  `forge_open`/`fleet_open`/`status_flash`, while including non-overlay focus states
  (`session_search_focused`/`commit_focused`/`top_search_focused`/`renaming_tab`) that the shim must
  NOT include. `overlay_is_up` carries the same "a NEW overlay only needs a line here" NOTE
  discipline, and the render tail gets a pointer comment; the P3.5 overlay-completeness adversary
  re-derives the set from the tree. Constraining overlays away from the rect stays the partial
  secondary, documented not built.
- **D3 — pinned-origin nav.** The wry navigation handler rejects off-origin: `false` from
  `with_navigation_handler` on any URL whose origin ≠ the mounted origin. The equality is a pure fn
  over **#404's origin type** (scheme + host + port; the handler's `String` URL parsed against it —
  hostile inputs included: userinfo tricks, scheme downgrade, port mismatch, subdomains). No address
  bar; nothing user-typed.
- **D4 — the URL is NEVER persisted.** The tab persists as #403's bare `B` tag; mount and restore
  RE-DERIVE via #404, keyed on the restored ACTIVE project root
  (`PR-claude-boot-decisions-key-the-restored-active-root-001`), never the launch cwd. A Forge URL
  that changes between sessions is picked up automatically; the codec stays byte-identical.
- **D5 — the bearer never enters the URL or page JS** (#389 D3; the #370/#375 bearer-never-logged
  lineage). The webview gets an origin, nothing more; the Forge web UI does its own auth. If it
  turns out to expect the MCP bearer, the pane renders unauthenticated and that gap closes later —
  never via a URL bearer.
- **D6 — coexistence, additive only.** `cockpit_body` (app.rs:5854) Forge arm untouched;
  `fleet_rail_body` (app.rs:1076) NEVER retired; cockpit retirement is named-criteria-only and not
  this ticket's business.
- **D7 — the webview is a MASKED surface, thin adapters over pure decisions.** Every wry call
  (build/attach, `set_bounds`, `set_visible`, `load_url`, focus verbs, the nav-handler closure)
  lives in `mutants::skip`-masked shims with the documented §0 ACCEPTED-UNTESTABLE shape,
  driven-validated; each shim's only logic is "call the pure decision, apply the verdict".
  Skip-placement is re-verified per `BF-claude-skip-detach-pump-fleet-live-001` +
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` (Floors).

**D-OPEN (Phase 2 decides, with #402's evidence):**
- **D-OPEN-HIDE-MECHANISM** — how "hide" is implemented: `set_visible(false)` (first-class,
  documented) vs offscreen/zero `set_bounds` vs remove-from-superview (NOT in wry's portable API —
  `reparent` is gtk-bound; would mean hand-rolled objc2). Phase 2 picks whichever #402 RECORDED as
  actually stopping WKWebView compositing over the Metal layer, and notes whether the mechanism
  drops first-responder cleanly.
- **D-OPEN-RECT-SYNC** — per-frame capture-and-diff (recommended: the render already computes
  `center_bounds` :18407 every frame; capture into a `Cell` à la `editor_geom` :513, then a
  post-paint sync applies `set_bounds` only on change, the PTY-R37 :18614 discipline — catches
  every layout change by construction, including drags) vs event-driven recompute at each mutating
  site (dock toggle, files drag, resize, tab switch — N call sites, rot-prone). Ground truth: ALL
  geometry consumed off-render today is render-captured (`editor_geom`, `rect_list`).
- **D-OPEN-FOCUS-HANDOFF** — click-in/click-out policy, whether ⌘-chords pass through a focused
  webview (if the child swallows ⌘⇧P, the palette — the shim's own trigger — can never open: the
  spike's checklist item 3 owns the answer), Esc semantics, IME composition. Resolved FROM #402's
  recorded findings; wry's `focus()`/`focus_parent()`/`with_focused` are the available verbs.
- **D-OPEN-WEBVIEW-HOME** — where the `wry::WebView` handle lives: inside `Content::Browser`
  (registry-owned; teardown rides #394 `release_view` → drop) vs an app-side
  `Option<WebView>`/`ContentId`-keyed map (keeps the main-thread-only native handle out of the
  registry's data plane). Phase 2 decides AGAINST #403's SHIPPED residency shape (v1 is a single
  app-wide Browser instance per #403); #406 owns teardown hardening either way.
- **D-OPEN-FLASH** (D2 sub-fork) — `status_flash` membership: include (the page blinks for the
  pill's ~2s life) vs exclude (a flash fired while the Browser tab is active is invisible behind
  the webview). Recommend INCLUDE by default — correctness over polish; a swallowed confirmation is
  the silent-failure shape — but Phase 2 may flip with evidence (the pill is passive,
  click-through, bottom-anchored).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Browser tab becomes active and `forge_web_base` resolves, the system shall attach the wry child at the Browser pane's rect (= `center_bounds`) and load exactly the derived origin — the real Forge page, not a placeholder. | **driven/headed** capture (page pixels at the rect); unit on the pure mount decision (`Some(origin)` → attach+load, `None` → placeholder) |
| REQ-002 | WHEN the pane rect changes — window resize, left/right dock toggle, Files-panel toggle or edge-drag, or switching back to the Browser tab — the system shall re-sync the webview to the new rect, applying `set_bounds` only when the rect actually changed. | **driven** sequence over each mutation; units on the pure rect-mapping + changed-rect diff fns (cov/MSI 100) |
| REQ-003 | WHILE the webview is showing, WHEN any overlay in the D2 inventory (all 27 Class-A+B states — the P3.5 amendment) opens, the system shall hide the webview before the overlay paints, and shall show it again when every overlay is dismissed. | unit on `overlay_is_up` — **each of the 27 states independently flips it** (mutation kills any dropped `\|\|` arm); **driven/headed** sweep per overlay CLASS (palette, context menu, a picker, a find bar, an editor card) capturing hide + reappear |
| REQ-004 | WHEN the Browser tab stops being the active tab of the active project (tab switch, project switch, restore-into-background), the system shall not paint the webview until the tab is active again. | **driven** tab-away/tab-back + project-switch capture |
| REQ-005 | WHEN a navigation targets a URL whose origin differs from the mounted origin, the nav callback shall reject it; same-origin navigations shall proceed. | unit on the pure origin-equality fn (hostile set: userinfo, scheme downgrade, port mismatch, subdomain, malformed URL); **driven** off-origin attempt — the pane stays on the Forge page |
| REQ-006 | WHEN `forge_web_base` yields `None` (no `.mcp.json` / unresolved / unparseable), the system shall construct NO webview and the #403 placeholder shall stay. | unit on the mount decision's `None` arm; **driven** no-config launch capture |
| REQ-007 | The system shall never place the MCP bearer in any loaded or navigated URL. | unit (the derivation path never sees the bearer — #404's fn signature can't receive it); **driven** assert over the nav-handler's observed URL log |
| REQ-008 | WHEN the app restarts with a persisted Browser tab, the system shall restore it from the bare `B` tag and re-derive the origin from the restored active root — no URL read from disk, codec byte-identical. | **driven** restart capture; #403's codec suite green unchanged |

**§7 honesty:** REQ-001/002/003(sweep)/004/005(attempt)/006(launch)/007(assert)/008 are
**headed/driven** — §0 gate:15 / ACCEPTED-UNTESTABLE territory, validated by the visual harness +
driven captures, not unit coverage; the pure seams under them (mount decision, rect mapping + diff,
`overlay_is_up`, origin equality) are the unit+mutation surface. If the machine blocks headed
driving, the recorded env-blocked protocol applies (units + documented carry — the #204/#205
precedent #400's REQ-003 used), recorded in the notes, never silently skipped.

## Floors (constitution)
Pure seams at **cov/MSI 100**: `overlay_is_up` (all 26 arms mutation-killable), the origin-equality
/ nav-decision fn (shared #404 origin type), the rect-mapping + changed-rect diff fns, the mount
decision fn. **MASKED** (`mutants::skip`, documented §0 shape, driven-validated): the wry adapter
(build/attach, bounds/visibility/load/focus verbs, the nav-handler closure's wiring — the pure
decision inside it stays tested) + the app.rs wiring (render-tail rect capture, post-paint sync,
mount/restore/close plumbing). The app.rs edits land in the masked pump/boot/render shim
neighborhood — the skip-detach trap's home turf (5th strike
`BF-claude-skip-detach-pump-fleet-live-001`): after placement, re-run `cargo mutants --list -f` on
the ACTUAL touched files and re-verify neighboring `#[mutants::skip]` bindings
(`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`). Typed inputs; no `unwrap` on
config/URL-derived paths (a hostile `.mcp.json` or URL string must land in the total error arms).
**Supply chain:** wry's whole dep tree passes gate:8 permissive-only (Apache-2.0/MIT per #389 Q5),
inherited from #402's landing — this ticket adds NO new dependency.

## Phase Plan
- **P2 Design** — re-check the promote gate (the #402 verdict recorded in
  embedded-browser-model.md + the AD; #403/#404 shipped) and CONSUME the findings: settle
  D-OPEN-HIDE-MECHANISM, D-OPEN-RECT-SYNC, D-OPEN-FOCUS-HANDOFF, D-OPEN-WEBVIEW-HOME (against
  #403's shipped residency), D-OPEN-FLASH; **freeze the overlay inventory against the then-current
  tree** (re-sweep the render tail — a sibling ticket may have added an overlay since 2026-08-06);
  exact signatures (`overlay_is_up` inputs, the adapter trait), the file manifest, the per-REQ test
  plan; re-confirm the React-first arm (the flip protocol). A recorded NO-GO → the D1 re-design
  happens HERE, before any implement.
- **P3 Implement** — pure seams first: `overlay_is_up`, origin equality (over #404's type), rect
  mapping + diff, mount decision; then the masked adapter + app.rs wiring (attach, capture, sync,
  hide, load, restore).
- **P3.5 Inspect** — adversarial critics: **the overlay-completeness adversary** (re-derive the
  overlay set from the render tail independently and diff it against `overlay_is_up`'s member list
  — find the overlay the inventory missed); the focus/IME edge critic (⌘-chords while the webview
  is focused, IME composition, Esc, click-out); secrets (bearer in any URL/log); reap/teardown
  sanity vs #403's release path (full hardening is #406); §20 provenance; skip-placement re-check.
- **P4 Validate** — write + RUN the units per REQ (`overlay_is_up` per-state flips, origin
  hostile set, rect diff, mount decision); the driven/headed sweep (REQ-003 per class + REQ-001/002/
  004/005/006/007/008); `cargo mutants --list -f` on the actual touched files; gate green
  (`--diff`), cov/MSI 100 on the pure seams; env-blocked protocol recorded if driving is blocked.
- **P5 Complete** — CHANGELOG; tick embedded-browser-model.md's train **slice-3 shipped** (+ any
  Q1/Q3 deltas the implementation taught); arch docs per §21; AAR capture; archive; close #405.
