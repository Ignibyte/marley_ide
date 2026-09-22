---
pipeline_id: 48b517dd-651a-40bd-bce2-ce89fa89560a
ticket: forge#406 (f72268be-1018-4e0b-8843-10d70cd718a2) · local docs/planning/tickets/open/TICKET-406-browser-nav-lifecycle.md
aar_id: (deferred — forge unreachable at 2026-08-08 promote; open on first reachable capture)
status: Phase 5 — Complete PASS
title: Browser nav chrome + lifecycle — reload, loading state, error page, teardown (the #389 train slice-4)
type: feature
milestone: M29
references:
  - docs/marley_architecture/embedded-browser-model.md
  - docs/marley_architecture/pane-composition-model.md
  - docs/marley_architecture/app_shell.md
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
  - crates/marley_app/src/content_registry.rs
  - crates/marley_app/src/content.rs
  - crates/marley_app/src/fleet_live.rs
  - crates/marley_app/src/fleet_rail.rs
  - crates/marley_app/src/palette.rs
  - crates/terminal_blocks/src/pty_os.rs
  - crates/marley_app/src/app.rs
---

## Title
The #389 train slice-4 — the closer (embedded-browser-model.md:152 names it verbatim: "reload, loading
state, error page, teardown on pane close (mirror the PTY reaper contract)"). After #403 (residency +
the `B` tag) and #405 (the wry child on the pane rect + the z-order hide shim), the Forge pane exists
and paints; this slice makes it LIVABLE and makes its death honest: it tells the truth while loading,
tells the truth when the origin is down (error + retry — Forge-down ≠ broken pane), reloads on demand,
and on the last close comes OFF the render path deterministically — detach + drop, never a dangling
native child painting over gpui. **Deps: #403 + #405 HARD** (transitively #402's GO + #404's
`forge_web_base`, consumed via #405); the hardening of #405's hide interplay lands here, closing the
M29 train. Verified against today's tree (2026-08-06): NO train code has shipped yet — `TabContent`
has no Browser variant (tabs.rs:25-36), `Content::Browser` is declared-uninhabited (content.rs:48-49,
`addable() == false` :83), no `wry` in any Cargo.toml — so every dep-machinery reference below binds
to the QUEUED sibling specs' contracts and Phase 2 re-verifies against what actually shipped.
**Promote re-verification (2026-08-08, /work):** the dep gate is OPEN — #402 (GO, 55cb5cf lineage),
#403 (`TabContent::Browser(ContentId)` tabs.rs:46; `Content::Browser` resident content.rs:76), #404
(`forge_web_base` in browser.rs / mcp_config.rs), #405 (wry 0.56 in marley_app; webview_shim.rs with
nav-handler/`load_url`/`set_bounds`/`set_visible` — NO page-load handler and NO reload yet;
browser.rs pure seams `OverlayStates`/`webview_visible`/`rect_key`/`rect_changed`/`mount_plan`) all
SHIPPED. `CommandId(31)` still free (`NAME_PANE_ID = CommandId(30)` now app.rs:692). marley-web:
#403's stage shipped `BrowserPlaceholder.tsx`; no `BrowserPane.tsx` — #406 creates it. Phase 2 still
owns the full file:line re-bind.

## Scope
### In
- **(a0) THE STATE MACHINE — one pure enum-driven decision seam** (a `browser_state.rs`-ish module;
  D1). Load state `Loading / Loaded / Error(reason)` crossed with the #405 visibility axis
  (shown / hidden-by-overlay / hidden-by-tab); inputs are the wry load events (`PageLoadEvent::
  {Started, Finished}` — the documented set, docs.rs-verified; the FAILURE signal is
  D-OPEN-ERROR-SIGNAL), the reload verb, the #405 hide/show edges, and close; outputs are a render
  verdict (webview visible vs a Marley-drawn loading/error body) plus webview commands
  (`set_visible` / `reload` / `load_url` / detach-and-drop). The render and the wry callbacks are thin
  masked adapters that forward events and execute verdicts — no state decisions in the masked layer.
- **(a1) The reload verb, via the current palette mechanism:** one FIXED static command —
  `CommandId(31)`, the next genuinely-free static id after `NAME_PANE_ID = CommandId(30)`
  (app.rs:682; the :678 comment records the #399 id-collision trap that made 30 the floor — P2
  re-audits the ledger before taking 31) — registered in `cockpit_commands()` (app.rs:10182) with an
  `action_for_command` arm (palette.rs:127; ids 0–29 mapped today, the exhaustive-map test :301
  extends). NOT a dynamic range: reload is one verb, not a rebuilt-at-open block (`ADD_TO_PANE_BASE`
  app.rs:670 and its D5 rebuild discipline are the contrast, not the template). No-op safely when no
  Browser pane is active.
- **(a2) The loading state:** Marley-drawn, in-pane, shown from load-start until `Finished` — the
  shipped loading-caption vocabulary is `forge_empty_hint`'s "loading sprint…" (forge_view.rs:66);
  presentation settled in the React prototype.
- **(a3) The error page:** an honest in-pane surface for an unreachable origin, composed from the
  three shipped precedents (the discovery finding: no single precedent combines them) — the **#395
  centered hint panel SHAPE** (app.rs:18416-18455 — centered flex column, muted title + affordance
  rows), the **#384 clamped-reason DISCIPLINE** (classify once into a typed enum,
  `classify_fleet_setup` fleet_live.rs:98-127; clamp AT CONSTRUCTION, `Misconfig::new` :53-58 via
  `clamp_card_text` fleet_rail.rs:151-159; the header names the fault — "Fleet · misconfigured",
  fleet_live.rs:138), and the **#378 failure-line-as-affordance CLICK seam** (`dispatch_retry_draft`
  fleet_rail.rs:212-217; the failed line itself is clickable, app.rs:1286-1313). Retry re-derives the
  origin (D2). Unreachable-origin error ≠ the no-config empty state #403 ships — two different truths.
- **(b) Teardown through the registry lifecycle:** EVERY Browser close path routes
  `ContentRegistry::release_view` (content_registry.rs:87) with the `#[must_use]` return consumed at
  the site (:85-86 — "dropping it here tears down on the calling thread";
  `PR-claude-registry-release-returning-a-resource-must-be-must-use-001`); the last-view
  `Some(content)` hands the webview to teardown: detach from the NSView hierarchy + drop, with the
  off-thread-vs-inline fork explicit (D-OPEN-DROP-THREAD). The house patterns to choose between,
  verified: **#396 off-thread** — `release_grid_terminals` app.rs:5799-5817, one
  `std::thread::spawn(move || drop(reap))` :5815 under the #348 bounded-reap contract (pty_os.rs
  `Drop for OsPtyChannel` :102-161: SIGHUP → deadline → SIGKILL → deadline → give-up-and-leak,
  decisions in the pure `reap_step`, the Drop a `mutants::skip` executor); **#397 inline** —
  `release_editor_views` content.rs:219-226 ("a Buffer free is plain memory work, so editors
  deliberately skip the terminal reaper thread"). The **#400 pinned** third lifecycle
  (`release_cockpit_views` content.rs:289-302) does NOT apply — a Browser owns a reapable native
  resource, per #403's lifecycle fork (drops on last close).
- **(c) The #405 hide-interplay HARDENING:** the state machine owns loading × error × visibility as a
  total product space (a hidden pane's load continues; an overlay over an error page re-shows the same
  error; close is total from every state), and the driven matrix covers: overlay over the pane, tab
  away/back, close-while-loading, close-while-hidden, overlay-then-close.

### Out (explicitly deferred)
- **Address bar / arbitrary navigation / user-typed URLs** — the pinned-origin policy stands
  (embedded-browser-model.md:95-96; #405's nav callback rejects off-origin); a general browser is a
  later, separately-reviewed capability.
- **Back/forward history** — pinned origin makes it marginal; deferred-not-forgotten. When a later
  ticket un-defers it, the research-level home already exists: Zed's `Item::navigate` back/forward
  convention (zed 07-workspace-panes-palette.md:150-165, behavior map).
- **The CDP agent-browser lane** — a separate substrate by decision (embedded-browser-model.md Q2).
- **Any cockpit changes** — the native cockpit coexists (#389 Q3; retirement is named-criteria-only).
- **The split-cell grid `b` leaf** — deferred with the #388 registry Browser cell arm (#403's codec
  note).
- **Redesigning the #405 hide shim** — this slice hardens the interplay; the shim mechanics are #405's.

## Reference (§20)
**N/A — Marley-specific** (chad's Forge-in-the-browser, Phase E; no Warp/Zed analog — both behavior
maps verified browser-free, files enumerated under Prior art). Published conventions cited instead:
**VS Code's built-in Simple Browser** — "a very basic browser preview using an iframe embedded in a
webview" (extension README) — the convention that even a minimal, preview-grade embedded page keeps an
explicit reload affordance and a visible loading signal, matched at the published-docs level only; and
**wry's documented API** as the substrate contract (Prior art leg 2). Clean-room: no Warp (AGPL) / Zed
(GPL) source consulted.

### Prior art
1. **Behavior maps — checked; NO embedded-browser owner (the absence is the finding).**
   `docs/warp_architecture/`: zero hits for webview/embedded-browser terms across all 9 subsystem docs
   + 78 crate docs — every "browser" is Warp-on-Web's wasm target (warpui.md:18) or the EXTERNAL
   system browser (http_server.md:16 OAuth loopback; 03-terminal-session-core.md:419). Warp hosts no
   browser. `docs/zed_architecture/`: no browser pane either; the one adjacent yield is
   07-workspace-panes-palette.md:150-165 — Zed's `Item` trait carries `reload(…)` and the
   `added_to_workspace / deactivated / on_removed` lifecycle hooks on the ITEM, not bespoke per-pane
   chrome (supports D5's no-header-row lean), and `Item::navigate` is where back/forward lives when
   history un-defers. Research, not source.
2. **Published.** docs.rs/wry (fetched 2026-08-06; latest 0.56.0 — the LANDED version is #402's pin,
   P2 re-confirms): `WebViewBuilder::with_on_page_load_handler(impl Fn(PageLoadEvent, String))` with
   `PageLoadEvent::{Started, Finished}` and **no error variant**;
   `with_navigation_handler(impl Fn(String) -> bool)`; `WebView::{reload, load_url, set_visible,
   set_bounds, focus, focus_parent}` (`reload(&self) -> Result<()>` exists — the verb's substrate).
   **The load-bearing gap: wry documents NO did-fail-load/load-error callback**, so the error-page
   trigger cannot be a plain wry event — D-OPEN-ERROR-SIGNAL. VS Code Simple Browser as above.
3. **Our deps + in-house — the in-house dominates.** gpui (Apache-2.0) owns render primitives only —
   no load-lifecycle seam to adopt; ropey/regex/alacritty_terminal own nothing near this. The seams
   are already ours, verified at today's lines: the **#348 bounded-reap contract** (pty_os.rs:102-161,
   pure `reap_step` + a masked executor — the template IF teardown goes off-thread); the **#396
   release→reap pattern** (app.rs:5799-5817, the one reaper spawn :5815; the pump auto-reap twin
   :1758-1764); the **#397 inline-drop contrast** (content.rs:219-226); the **#400 pinned contrast**
   (content.rs:289-302 — inapplicable here, cited to close the lifecycle triple); the **#384
   misconfigured-presentation pattern** (fleet_live.rs:98-127, :53-58; fleet_rail.rs:151-159, :143);
   the **#378 failure-line-as-affordance** (fleet_rail.rs:212-217, :266-272; app.rs:1286-1313); the
   **#395 hint-panel shape** (app.rs:18416-18455); the **palette verb mechanism** (palette.rs:12,
   :127; app.rs:659-682, :10182); `Content::Browser` already DECLARED awaiting its cash
   (content.rs:48-49); and the **loopback TcpListener test-fixture lane** for a controllable origin
   (marley_forge_client/tests/livewire.rs:127 — "the workspace's first TcpListener-based test
   fixture"; marley_app/tests/fleet_livewire.rs:54). Recorded anti-precedents (gaps, not conventions):
   the settings-parse swallow (app.rs:1535-1544, no UI) and the silent spawn no-op (app.rs:7191-7192)
   — D2 exists so the Browser pane never joins them.

## React-first (parity)
**UI-AFFECTING — Zone B pane chrome:** the loading and error+reason+retry states, plus the D5
header-row question (answered BY the prototype). The marley-web files (port map:
/Volumes/Offload/Projects/marley-web/docs/MARLEY-PARITY.md:550-590; sources under
`artifacts/marley-ide/src/`):
- **EXISTING** `components/views/BrowserView.tsx` (port-map row :585 — the section-level Browser
  surface; its URL bar / back-forward / simulated pages are POC theater BEYOND the pinned v1 — the
  Rust port takes only the states, never an address bar).
- **EXISTING-OR-TO-CREATE** `components/views/BrowserPane.tsx` — the pane-level chrome-states surface.
  #403's React-first stage (this same batch) prototypes the Browser tab row + placeholder; if it ships
  this file, #406 extends it; if not, create it here matching the `*Pane.tsx` siblings
  (`ForgePane.tsx` / `AgentsPane.tsx` / `BlockDetailsPane.tsx` — the shape to copy). P2 binds to
  what #403 actually shipped.
- **EXISTING vocabulary donors:** `components/FleetDock.tsx` (port-map row :587 — the #384
  misconfigured caption + clamped-reason vocabulary, `Fleet · misconfigured`); `components/ui/
  spinner.tsx` + `ui/skeleton.tsx` (shadcn kit, currently unused — loading-state candidates).
Plan: **build & visually verify in marley-web first (`pnpm --filter @workspace/marley-ide run dev` →
localhost:5173), then port 1:1** — the three states (loading / error+retry / loaded) and the
header-row experiment iterate in seconds there; the look is settled BEFORE the cargo build, and the
header-row verdict (D5) is recorded from what the prototype proves. Validate captures the
React↔Marley parity pair; typecheck stays green (`pnpm --filter @workspace/marley-ide run typecheck`).

## Locked-In Decisions
- **D1 — the loading/error/visibility logic is ONE pure state machine.** A `browser_state.rs`-ish
  seam: `(state, event) -> (state, verdicts)`, exhaustive over the load × visibility × close product
  space; the wry callbacks and the render are thin adapters (forward events in, execute verdicts out)
  with NO state decisions in the masked layer. This is the #384 classify-once discipline applied to a
  live surface, and it is what makes cov/MSI 100 real rather than aspirational.
- **D2 — error + retry, never a blank or broken pane.** The misconfigured-not-silent house pattern:
  a typed reason, clamped at construction (reuse `clamp_card_text` or a sibling with the same
  control/bidi strip + cap), a header that names the fault, and the retry affordance on the page
  itself. **Retry re-derives the origin** through the #404 seam (keyed on the restored ACTIVE root —
  `PR-claude-boot-decisions-key-the-restored-active-root-001`), so a `.mcp.json` fixed while the error
  page is up is picked up for free. Unreachable ≠ unconfigured: no-config keeps #403's empty state;
  the error page is only ever a live-origin truth.
- **D3 — teardown is deterministic and total.** EVERY close path routes `release_view` with the
  `#[must_use]` return consumed; the last-view drop hands the webview to teardown, which detaches it
  from the view hierarchy and drops it. The named invariant — **INV-NO-ORPHAN-WEBVIEW: after the last
  Browser view closes, no WKWebView child remains attached under the GPUIView** — is the whole point:
  a wry child composites ABOVE gpui's entire Metal scene (embedded-browser-model.md:37), so an orphan
  is not a leak, it is a permanent opaque rectangle painting over the app — the worst failure mode.
  Total means close-while-loading and close-while-hidden route the exact same seam (no "wait for
  Finished", no visibility precondition).
- **D4 — the #405 hide interactions are re-driven here as regression.** The driven matrix: overlay
  over the pane, tab away/back, close-while-loading, close-while-hidden, overlay-then-close — each
  asserting both the visibility outcome AND the state machine's verdict (hidden load continues;
  re-shown error persists; teardown wins from any state).
- **D5 — reload is a palette verb v1.** Static `CommandId(31)` in `cockpit_commands()` +
  `action_for_command` (the #204-lineage static mechanism, NOT a dynamic base). The slim pane header
  row is a **Phase-2 D-OPEN with a recommendation: NO header row v1** unless the React prototype
  proves it earns its pixels — the palette verb, the error page's own retry, and #395-style hint
  affordances likely suffice; Zed's item-level `reload` convention (chrome-less) points the same way,
  and the house spends pane pixels only on truth (the #384 header names a fault; nothing ships a
  decorative row).

**D-OPEN (Phase 2 decides, with evidence):** — **ALL FOUR SETTLED at P2 (2026-08-08); verdicts + the
wry-0.56.0-source evidence in the notes' Phase 2 entry.** In one line each: DROP-THREAD → **inline**
(`wry::WebView` is `!Send` — off-thread drop cannot compile; wry's own `Drop` already
`removeFromSuperview`s, so holder-drop IS detach+drop); ERROR-SIGNAL → **origin probe (primary) +
pure tick deadline (backstop) + macOS renderer-terminate hook** (wry delivers NOTHING on transport
failure — `Started` fires on COMMIT; 4xx/5xx commits success-shaped, the probe sees the status);
ERROR-TAXONOMY → `ProbeOutcome` (forge-client-owned wire truth) → `LoadFault`
{Refused/Dns/Timeout/Http/Io/RendererGone/LoadHung/AttachFailed}, off-origin stays policy;
HEADER-ROW → **NO header row v1** (React confirms at P3). Original candidate analyses preserved
below for the record:
- **D-OPEN-DROP-THREAD** — off-thread (the #348/#396 template) vs inline (#397) webview drop.
  **Recommend: decide from what WKWebView teardown actually costs — a measurement/probe question**
  (the #402 scratch bin is the instrument: attach, load, detach+drop, time it). Two facts already
  bound the space: (1) AppKit view detachment is main-thread work — the DETACH half cannot leave the
  main thread regardless; (2) wry's macOS `WebView` wraps ObjC pointers and is unlikely `Send`, so a
  `thread::spawn(move || drop(content))` may not even compile — establish Send-ness against the landed
  wry, never assume it. If the measured main-thread cost is small (likely — no child process, no
  unbounded-`waitpid` analog), **inline detach+drop wins** and #348 stays a template; if off-thread
  proves necessary, #396's one-spawn-per-gesture shape is the pattern.
- **D-OPEN-ERROR-SIGNAL** — the wry callback set + the load-failure signal, confirmed against the
  LANDED wry version (the docs.rs sweep found `Started/Finished` only — no error event). Candidates:
  (a) a bounded Started-without-Finished load deadline — a pure two-stage escalation reusing the
  #348 `reap_step` SHAPE for load-not-kill; (b) an origin reachability probe alongside the load;
  (c) drop to `objc2-web-kit`'s `WKNavigationDelegate` (`didFailProvisionalNavigation`) if wry's
  abstraction lacks the event — the named escape hatch (embedded-browser-model.md:57-60).
- **D-OPEN-ERROR-TAXONOMY** — which failures map to which user-facing reasons: DNS / connection
  refused / timeout / HTTP-5xx, vs **off-origin-blocked which is a POLICY event, not a failure**
  (#405's nav-callback rejection never renders the error page). Granularity depends on
  D-OPEN-ERROR-SIGNAL's resolution; the mapping itself is a pure table at cov/MSI 100 whatever it maps.
- **D-OPEN-HEADER-ROW** — under D5; answered by the React prototype, verdict recorded at P2/P3.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the Browser pane begins loading its origin (first load, restore, reload, retry), the system shall present the in-pane loading state, and shall clear it when the load finishes. | state-machine units (every entry edge → Loading; Finished → Loaded; exhaustive, no catch-all); driven capture of a pane through a real load |
| REQ-002 | IF the origin is unreachable or the load fails, THEN the system shall render the in-pane error page — a clamped, typed reason + a retry affordance, never a blank pane — and WHEN retry is invoked after the origin returns, the system shall re-derive the origin and recover to Loaded. | taxonomy-mapping + clamp units; driven against a controllable loopback origin (the TcpListener fixture lane — livewire.rs:127 / fleet_livewire.rs:54: serve → kill → error → restart → retry → recovered); if the headed lane is env-blocked, the recorded operator-driven protocol applies |
| REQ-003 | WHEN the "Browser: Reload" palette command is dispatched with a Browser pane active, the system shall reload the current page; with no Browser pane active it shall no-op safely. | verb-registration units (`CommandId(31)` in `cockpit_commands` + `action_for_command`; the exhaustive-map test palette.rs:301 extended); driven dispatch re-runs the REQ-001 cycle |
| REQ-004 | WHEN any close path closes a Browser view (close-tab, close-project, any tab-drop path the #403 audit enumerates), the system shall route it through `release_view` with the `#[must_use]` return consumed at the site, and a last-view return shall be handed to teardown, never silently dropped. | units over the registry (acquire/release pairing across close sequences; double-release safety); §18.1 site-by-site audit per `PR-claude-registry-release-returning-a-resource-must-be-must-use-001` |
| REQ-005 | WHEN the last Browser view closes — including close-while-loading, close-while-hidden, and overlay-then-close — the system shall detach the webview from the view hierarchy and drop it deterministically (INV-NO-ORPHAN-WEBVIEW: no native child remains painting over gpui). | teardown decision-table units (close total from every state × visibility); driven close sequences + the §18.1 teardown-leak walk over every close path; headed hierarchy check where the harness allows |
| REQ-006 | WHILE the #405 hide interplay runs (overlay up over the pane, tab away/back) crossed with load states, the system shall keep visibility and load state independent and total — a hidden pane's load continues to completion, a re-shown pane presents its current state (including a persisted error page), and no hide/show sequence resets or fabricates load state. | state-machine product-space units (visibility × load, exhaustive); the D4 driven matrix |
| REQ-007 | The pure seams — the state machine, the reload/teardown decision table, the error-taxonomy mapping — shall stand at cov/MSI 100, and the React↔Marley parity pair for the chrome states shall be captured at Validate. | `scripts/gates.sh --diff` exit codes (gate:4/5 floors); the parity capture per the enforce-react-parity.sh contract |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the browser state machine (the `(state, event)` transition fn + verdict
emission), the reload/teardown decision table, the error-taxonomy mapping + reason clamp. MASKED (the
§0 ACCEPTED-UNTESTABLE discipline; app.rs is coverage-excluded): the wry callback adapters, the
palette dispatch arm, the detach/drop executor, the render bodies. These edits land on the masked
pump/close shims — the skip-detach trap's home turf (5th strike,
`BF-claude-skip-detach-pump-fleet-live-001`): after placement, re-run `cargo mutants --list -f` on the
ACTUAL touched files and re-verify neighboring `#[mutants::skip]` bindings
(`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`). Every `release_view` site
consumes the `#[must_use]` return
(`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`). Typed reasons end-to-end; no
`unwrap` on load-result or derive paths.

## Phase Plan
- **P2 Design** — verify #403 + #405 SHIPPED shapes and bind to them (the hard-dep gate; re-verify
  every file:line cite against the moved tree); settle the D-OPENs: D-OPEN-DROP-THREAD (the #402-bin
  teardown-cost probe + wry `Send`-ness), D-OPEN-ERROR-SIGNAL (confirm the landed wry's callback set;
  pick the failure mechanism), D-OPEN-ERROR-TAXONOMY, D-OPEN-HEADER-ROW (via the React prototype);
  exact state-machine types + adapter signatures; the file manifest + per-REQ test plan; enumerate
  every Browser close path from the #396/#400/#403 audits; re-audit the static-CommandId ledger
  before taking 31.
- **P3 Implement** — **React-first stage FIRST**: the chrome states (loading / error+retry) + the
  header-row experiment in marley-web, visually verified at localhost:5173, verdict recorded; then the
  pure seams (state machine, decision table, taxonomy + clamp); then the masked adapters (wry
  callbacks, palette arm, teardown executor, render bodies) wired through `release_view`.
- **P3.5 Inspect** — independent critics vs the diff: the **teardown-leak adversary** (hunt a close
  path or event ordering that strands the webview — the INV-NO-ORPHAN-WEBVIEW kill attempt);
  **state-machine completeness** (visibility × load × close product space — no unreachable state, no
  absorbing trap, no adapter-side decision smuggled in); the `release_view` site audit; §20
  provenance; the skip-detach re-check.
- **P4 Validate** — write + RUN the units per REQ; the driven sequences (load; error → retry against
  the loopback origin; reload; the D4 hide/close matrix); `cargo mutants --list -f` on the actual
  touched files; gate green (`--diff`), cov/MSI 100 on the pure seams; the parity captures
  (React↔Marley).
- **P5 Complete** — CHANGELOG; embedded-browser-model.md train-complete tick (slice-4 shipped — the
  Q5 follow-up train closes); pane-composition-model.md Browser-residency note if touched; AAR
  capture; archive; close #406.
