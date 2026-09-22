# Browser nav chrome + lifecycle (the #389 train slice-4) — Notes

- **Forge ticket:** #406 `f72268be-1018-4e0b-8843-10d70cd718a2`
- **AAR:** deferred — opened when /work promotes this pipeline to active
- **Local ticket doc:** docs/planning/tickets/open/TICKET-406-browser-nav-lifecycle.md
- **Pipeline spec:** 406-browser-nav-lifecycle.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan (2026-08-06, /spec batch draft)
- **Request:** M29 batch-planning (/spec) — the #389 embedded-browser follow-up train, slice 4 of 4:
  nav chrome (reload / loading / error page) + lifecycle (deterministic webview teardown through the
  registry) + hardening the #405 hide interplay. Sprint: M29, forge sprint #40
  `6dff19c6-fb61-472b-89c3-98d3215a489b`.
- **Classification / tier:** feature, M (per the forge ticket), milestone M29. Queued behind #403 +
  #405 (HARD deps; transitively #402-GO + #404). Closes the M29 train.
- **Forge recall (§18.3):** `sprint-get 6dff19c6…` — the full train confirmed: #402 spike
  (proof-of-embed, the ONE place wry enters), #403 (residency + `B` tag; Browser likely DROPS on last
  close — its lifecycle fork), #404 (`forge_web_base` derivation), #405 (wry child on the pane rect +
  z-order hide shim + pinned-origin nav callback), #406 (this). #406's forge description recalled
  verbatim (nav chrome / lifecycle / hide-hardening, REACT-FIRST Zone B). `knowledge-search` ran twice
  ("webview browser teardown release_view must_use reaper", "skip detach pump mutants masked palette")
  — top hits are prevention_rule + architecture_decision nodes; the three load-bearing rules the spec
  cites by slug: `PR-claude-registry-release-returning-a-resource-must-be-must-use-001`,
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`,
  `BF-claude-skip-detach-pump-fleet-live-001` (5th strike — the masked-shim home turf), plus
  `PR-claude-boot-decisions-key-the-restored-active-root-001` (retry re-derivation keying) and
  `AD-claude-embedded-browser-substrate-001` (wry-A; CEF-OSR the named revisit).
- **Discovery (verified 2026-08-06, today's lines):**
  - **Tree baseline — no train code shipped yet:** `TabContent { Terminal, Cockpit, CodeView }`
    (tabs.rs:25-36, no Browser); `RailSection::Browser` is the cockpit tabs' rail home (tabs.rs:51);
    `Content::Browser` DECLARED-uninhabited (content.rs:48-49; `addable() == false` :83); no `wry` in
    any Cargo.toml; no browser/webview module under crates/. All dep-machinery cites in the spec bind
    to the QUEUED sibling specs; P2 re-verifies.
  - **The reaper contract (#348):** pty_os.rs `impl Drop for OsPtyChannel` :102-161 — SIGHUP → poll
    under deadline → SIGKILL → second deadline → give-up-and-leak (`mem::forget` :151, D5
    zombie-over-hang); escalation decisions in the pure `reap_step`, the Drop a `mutants::skip`
    executor (:108). The template IF webview drop goes off-thread.
  - **The #396 release→reap pattern:** `release_grid_terminals` app.rs:5799-5817 — the shared teardown
    tail of every terminal close path (⌘W, pane ×, close-tab, close-project); one
    `std::thread::spawn(move || drop(reap))` :5815; content-keyed maps scrubbed on last drop only.
    The pump auto-reap twin: app.rs:1758-1764.
  - **The #397 inline contrast:** `release_editor_views` content.rs:219-226 — last-view instance
    dropped INLINE ("a Buffer free is plain memory work"); answers the `#[must_use]` with an explicit
    drop. **The #400 pinned contrast:** `release_cockpit_views` content.rs:289-302 (debug-asserted
    never-reaped) — inapplicable to Browser (reapable native resource; #403 forks it to
    drop-on-last-close). Registry semantics: `release_view` content_registry.rs:87, `#[must_use]`
    :85-86; `acquire_view` :74-79; `insert` :65.
  - **The palette mechanism:** `CommandId(pub u32)` palette.rs:12; `Command` :16; `action_for_command`
    palette.rs:127 (static ids 0-29 today; exhaustive-map test :301); `cockpit_commands()`
    app.rs:10182; dynamic bases CONNECT 1000 / THEME 2000 / WORKFLOW 3000 (#204) / ADD_TO_PANE 4000
    (#398, rebuilt-at-open D5) / PANE_ARRANGEMENT 5000 (#399) at app.rs:659-674; special statics
    `SAVE_WORKFLOW_ID = CommandId(9)` :676, `NAME_PANE_ID = CommandId(30)` :682 with the :678 "30 is
    the first genuinely FREE static id" ledger note (+ the recorded id-20 collision trap). Reload →
    static `CommandId(31)`; re-audit the ledger at P2.
  - **The error-precedent find (no single precedent combines the three halves):** (1) #384 pattern —
    `classify_fleet_setup` fleet_live.rs:98-127 (pure, first-failure-wins, typed arms), clamp AT
    CONSTRUCTION `Misconfig::new` :53-58 via `clamp_card_text` fleet_rail.rs:151-159 (control/bidi
    strip + take(200); `NO_FORGE_CLIENT_REASON` :143), header `fleet_header_title` fleet_live.rs:138
    ("Fleet · misconfigured" outranks wire state), body caption app.rs:1091-1101 (wired
    :18292-18308). (2) #378 failure-line-as-affordance — `dispatch_retry_draft` fleet_rail.rs:212-217,
    chip :266-272 ("send failed: {reason}"), clickable line app.rs:1286-1313. (3) #395 pane-sized
    hint-panel shape — app.rs:18416-18455 (centered flex column; `hint()` closure :18422-18432;
    keyboard hints, not clickable). Anti-precedents (recorded gaps, not conventions): dead PTY pane
    has NO presentation (auto-close or frozen scrollback; policy comment app.rs:1587-1590; the remote
    `✗ {host}` badge :19603-19614 is the closest marker); spawn failure is a silent no-op
    (:7191-7192); settings-parse error is swallowed with no UI (:1535-1544). Loading-caption
    vocabulary: `forge_empty_hint` "loading sprint…" forge_view.rs:66.
  - **wry at the published-docs level (docs.rs, latest 0.56.0; the #389 doc pinned 0.55.1 — the
    LANDED version is #402's, P2 confirms):** `with_on_page_load_handler(Fn(PageLoadEvent, String))`,
    `PageLoadEvent::{Started, Finished}` — **no error variant**; `with_navigation_handler(Fn(String)
    -> bool)`; `WebView::{reload, load_url, set_visible, set_bounds, focus, focus_parent}`. **No
    documented did-fail-load/load-error callback** → D-OPEN-ERROR-SIGNAL (deadline / probe / the
    objc2-web-kit `WKNavigationDelegate` escape hatch embedded-browser-model.md:57-60 names).
  - **Behavior maps (§20 legs):** Warp — browser-free, confirmed across all 9 subsystem + 78 crate
    docs (every hit is the wasm target or the external system browser). Zed — no browser pane; yield:
    zed 07:150-165, the `Item` trait's `reload(…)` + `added_to_workspace/deactivated/on_removed`
    lifecycle hooks + `Item::navigate` (the future history home). Marley docs:
    embedded-browser-model.md:152 (the slice-4 line), :95-96 (pinned origin — no address bar), :37
    (child composites ABOVE the Metal scene — the orphan-webview worst case), :57-60 (objc2-web-kit
    escape hatch); pane-composition-model.md:73/:163 (close→teardown contract through the registry).
  - **Test-fixture lane for a controllable origin:** marley_forge_client/tests/livewire.rs:8/:127
    (loopback `TcpListener` bind 127.0.0.1:0 — "the workspace's first TcpListener-based test
    fixture") + marley_app/tests/fleet_livewire.rs:54 — the REQ-002 driven lane's precedent.
  - **marley-web (parity):** /Volumes/Offload/Projects/marley-web/docs/MARLEY-PARITY.md port map
    :550-590; row :585 `views/BrowserView.tsx` → right_dock.rs (EXISTING — the section surface; its
    URL bar/back-forward/simulated pages are POC theater beyond the pinned v1); row :587 FleetDock
    (EXISTING — the misconfigured vocabulary donor); `views/BrowserPane.tsx` TO-CREATE (or extend
    #403's shipped prototype — P2 binds), shaped on the `*Pane.tsx` siblings (ForgePane / AgentsPane /
    BlockDetailsPane); `ui/spinner.tsx` + `ui/skeleton.tsx` exist unused (loading candidates). NO
    retry affordance exists anywhere in the POC today. Dev command confirmed:
    `pnpm --filter @workspace/marley-ide run dev` → localhost:5173 (strictPort), typecheck gate
    MARLEY-PARITY.md:638.
- **Decisions:** D1 one pure state machine (adapters decision-free); D2 error+retry never
  blank/broken, reason clamped at construction, retry re-derives the origin (unreachable ≠
  unconfigured); D3 teardown deterministic + total through `release_view`, INV-NO-ORPHAN-WEBVIEW
  named; D4 the #405 hide matrix re-driven as regression; D5 reload = palette verb v1 (static
  `CommandId(31)`), header row a P2 D-OPEN recommended NO. D-OPENs: DROP-THREAD (probe WKWebView
  teardown cost in the #402 bin; detach is main-thread regardless; wry `WebView` likely not `Send` —
  verify, never assume), ERROR-SIGNAL (landed-wry callback confirm; deadline / probe / delegate
  candidates), ERROR-TAXONOMY (DNS/refused/timeout/5xx vs off-origin-policy — granularity follows the
  signal), HEADER-ROW (the React prototype answers).

### Phase 1 addendum — /work promote (2026-08-08)
- **Promoted** queued → active by /work ("on the next ticket"); no other pipeline active. Autonomous
  mode: /goal Stop-hook "/work on the next ticket" — phases proceed without per-phase human pause
  (autonomous-through-commit), decisions recorded here instead.
- **Dep gate OPEN, verified in-tree:** #403 `TabContent::Browser(ContentId)` tabs.rs:46 +
  `Content::Browser` resident content.rs:76/:120; #404 `forge_web_base` (browser.rs, mcp_config.rs,
  app.rs); #405 wry 0.56 dep-edge in marley_app/Cargo.toml:19 + webview_shim.rs (nav-handler,
  `load_url`, `set_bounds`, `set_visible`; `mutants::skip` on the FFI executor :64) + browser.rs pure
  seams (`OverlayStates` :43, `overlay_is_up` :112, `webview_visible` :144, `rect_key` :151,
  `rect_changed` :162, `MountPlan`/`mount_plan` :169/:178). NO page-load handler + NO reload in the
  shim yet — #406's surface is genuinely unshipped, no scope collision.
- **CommandId ledger spot-check:** `NAME_PANE_ID = CommandId(30)` moved to app.rs:692 (ledger comment
  :688 "30 is the first genuinely FREE static id"); no `CommandId(31)` anywhere — 31 free as planned
  (P2 formal re-audit stands).
- **marley-web spot-check:** #403's React stage shipped `views/BrowserPlaceholder.tsx` (+
  `BrowserView.tsx` pre-existing); **no `BrowserPane.tsx`** → #406 CREATES it per the spec's
  existing-or-to-create fork; `ui/spinner.tsx` + `ui/skeleton.tsx` confirmed present.
- **Forge (§19):** `.mcp.json` wired but server unreachable at promote (connection refused
  127.0.0.1:8080) — recall skipped (the /spec-time recall above stands); `aar-open` deferred; capture
  steps will note forge-unreachable unless it returns before P5.
- **Env pre-flight (from /work):** cargo 1.96.0, gates.sh OK, cargo-mutants 27.1.0, cargo-llvm-cov
  0.8.7, hooks wired, marley-web dev tree OK.

## Phase 2 — Design (2026-08-08, autonomous /goal)

### Dep re-bind (the entry gate) — every load-bearing cite re-verified against the shipped tree
- `browser.rs` (pure, #405): `OverlayStates` :43 / `overlay_is_up` :112 / `webview_visible(browser_tab_active, overlay_up)` :144 / `rect_key` :151 / `rect_changed` :162 / `MountPlan`+`mount_plan` :169-183; placeholder copy :22/:29. Tests pin all (27-flip table, truth table, rect arms, mount arms).
- `webview_shim.rs` (masked, #405): `BrowserWebview { webview, applied: Cell, shown: Cell }`; `attach(window, rect, key, origin) -> Result<Self, String>` :37 (build_as_child + pinned-origin nav handler + `load_url`); `sync(rect, key, visible)` :65 (bounds-first, hide invalidates key). **A failed attach is RETURNED — the #405 module doc names "#406's error card consumes it" verbatim.** No page-load handler, no reload — #406's surface confirmed unshipped.
- Holder + wiring (app.rs): `browser_view: Option<BrowserWebview>` :517, `browser_attach_error: Option<String>` (:518-ish, set :10851), `forge_web_base` :523 (boot-derived :2443-2445), `sync_browser_webview` :10830 (the ONE shim site; mount gate = `browser_view.is_none() && browser_attach_error.is_none()`), `reap_browser_holder` :10868, render arms :18727-18776 (mounted → plain backdrop under the composited child; unmounted → placeholder), render-site call :18664.
- Close paths (the #403/#405 audit, all routing `release_browser_views` + `reap_browser_holder`): switch-handback `open_browser_tab` :4072-4077; ⌘W/close-tab `close_tab_at` :8626-8628; `close_project_at` :8805-8808. `content.rs`: `find_browser` :348, `resolve_browser` :369, `release_browser_views` :397 (debug-assert browser-only; inline drop; its :391-393 note *predicting* an off-thread collector is OVERRIDDEN by evidence below).
- Palette: `CommandId(pub u32)` palette.rs:12; `action_for_command` :127 (statics 0-29 + gaps 3/9/30-special); exhaustive-map test :301; `cockpit_commands()` app.rs:10321; activation site :3520-3565 (special ids → dynamic ranges → `action_for_command` → `dispatch_action` :9611); ledger consts app.rs:668-692, `NAME_PANE_ID = CommandId(30)` :692. **31 free (re-audited: no `CommandId(31)`, no dynamic base below 1000).**
- Pump: `PUMP_INTERVAL_MS: u64 = 16` app.rs:701; the drain block landmark = the `forge_pending` arm :1811-1815; test-pump precedent `pump_fleet_live_for_test` :2831.
- Donors: `clamp_card_text` fleet_rail.rs:151-159 (pub; control+bidi strip, take(200)); #395 hint-panel shape app.rs:18674-18706; #384 classify/first-failure-wins fleet_live.rs:98-127; #378 failure-line-as-affordance fleet_rail.rs:212-217.
- Probe precedent: `marley_forge_client/src/adapter.rs` raw `TcpStream` HTTP (connect :134, :558-564) + `http_status_code` lib.rs:345; loopback `TcpListener` test lane tests/livewire.rs (:127 lineage) — **the forge client is deliberately TLS-free (raw HTTP only), which bounds the probe design below.**
- Driven lane: headless_drive.rs browser suite :11309-11537 (`browser_census`, door-drops-on-last-close, bare-`B` restore, boot-derivation) — the harness the D4 matrix extends. Headless drives view methods directly (render never runs → `sync_browser_webview` never attaches) so the driven rows assert MACHINE + registry truth; the live-webview composite stays the headed/operator lane.

### D-OPENs — settled, with evidence (the wry 0.56.0 source sweep; ADOPTION, in-registry)
- **D-OPEN-DROP-THREAD → INLINE, by type-system necessity (a #339-class WIN — the substrate forbids the alternative).** `wry::WebView` is `!Send`: macOS `InnerWebView` holds `mtm: MainThreadMarker` (objc2's `PhantomData<*mut ()>`, explicitly "!Send and !Sync") + `Retained<WryWebView>`/ObjC delegates (wkwebview/mod.rs:132-141); no `unsafe impl Send` exists for it. `thread::spawn(move || drop(webview))` cannot compile — the #402-bin cost probe is MOOT. Better: **wry's `Drop` already detaches** — `InnerWebView::drop` calls `self.webview.removeFromSuperview()` then deliberately over-retains webview+manager (anti-UAF; wkwebview/mod.rs:1417-1438). So INV-NO-ORPHAN-WEBVIEW is discharged by DROPPING THE HOLDER on the main thread — exactly what shipped `reap_browser_holder` does; #406 hardens totality (every state × every close path), adds the state-reset, and the content.rs:391-393 "route off-thread like #396" prediction is retired with this recorded reason. Teardown is plain ObjC release work (no child-process reap analog; WebKit's processes die asynchronously on their own).
- **D-OPEN-ERROR-SIGNAL → out-of-band ORIGIN PROBE (primary) + in-machine LOAD DEADLINE (backstop) + renderer-terminate hook (macOS).** The sweep's decisive fact: wry 0.56 delivers **NOTHING on a transport failure** — `PageLoadEvent::Started` fires on `didCommitNavigation` (NOT provisional start; navigation.rs:17-26), `Finished` on `didFinishNavigation` (:39-47), and `didFail*` is not implemented at all; `load_url`/`reload` return `Ok(())` unconditionally; `evaluate_script` silently QUEUES pre-commit (pending_scripts, mod.rs:569/:723-726) so JS can't probe liveness; no navigation-response/status hook exists (decidePolicyForNavigationResponse is hardcoded Allow). Candidate (c) — swizzling `WKNavigationDelegate` via `webview()` — would DISABLE wry's own Started/Finished + script injection (wry holds one delegate for everything): rejected. Candidate (a) alone is honest-but-slow (refused would wait the whole deadline). So: **(b) probe** — `marley_forge_client::adapter::probe_web_origin` (raw TcpStream GET, the adapter's own idiom, livewire-testable) races the load and delivers the typed fault FAST; **(a) deadline** — pure tick counting in the machine (`PUMP_INTERVAL_MS`-denominated, no clock reads) catches probe-Ok-but-render-hung; **plus** wry's one real failure hook, `with_on_web_content_process_terminate_handler` (Darwin ext, lib.rs:1633/1655-1660) → `RendererGone` (a live pane that silently went blank is the #384 anti-precedent; cfg(target_os="macos")). A subtlety the probe covers that events can't: **HTTP 4xx/5xx COMMITS a body** → Started+Finished fire success-shaped; the probe sees the status and outranks (first-fault-wins, sticky).
- **D-OPEN-ERROR-TAXONOMY →** `ProbeOutcome` (owned by marley_forge_client, wire-level): `Status(u16)` / `ConnectRefused` / `HostNotFound` / `TimedOut` / `TcpOnlyOk` (https: the adapter is TLS-free by house posture → TCP-connect-only truth, no status) / `Io(String)`. Machine-side `LoadFault` (marley_app): `Refused` / `Dns` / `Timeout` / `Http(u16)` (status ≥400) / `Io(ClampedReason)` / `RendererGone` / `LoadHung` (deadline) / `AttachFailed(ClampedReason)` (subsumes + retires the `browser_attach_error` field). Classify: `Status(200..=399) | TcpOnlyOk → probe-ok`. **Off-origin nav-rejection stays a POLICY verdict in the nav handler — it never enters the machine, never renders the error card** (the #405 pinned-origin seam untouched). Reasons render via `ClampedReason` (private-field newtype, clamped AT CONSTRUCTION through fleet_rail's `clamp_card_text` discipline — control/bidi strip + ≤200).
- **D-OPEN-HEADER-ROW → NO header row v1 (recommendation stands; the React prototype at P3 confirms or overturns, verdict recorded there).** Rationale re-grounded: Zed's chrome-less item-level `reload` convention (zed 07:150-165), the palette verb + the error card's own retry + #395-style hints cover every affordance, and the house spends pane pixels only on truth.

### Architecture (D1 made concrete)
**One pure machine, one signal channel, one executor.** New pure module `browser_state.rs` (browser.rs stays the #405 seams + pane copy):
- `LoadPhase { Loading { ticks: u32 }, Loaded, Error(LoadFault) }`; RootView holds `browser_load: Option<LoadPhase>` — `None` = unmounted/placeholder truth (subsumes the mount gate: attach attempts only when holder AND phase are both None).
- `BrowserSignal` (wire, crosses threads tagged `(gen, signal)`): `LoadStarted` / `LoadFinished` / `RendererGone` / `Probe(ProbeOutcome)`. One boot-created `mpsc` pair; senders clone into the wry handlers + probe threads carrying ONLY `(Sender, u64 gen)` — the #402 weak-capture rule holds by construction.
- `BrowserEvent` (machine input): the four signals mapped pure + `PumpTick` + `ReloadRequested` + `Closed`.
- `step(phase, event) -> (Option<LoadPhase>, Directives)` — exhaustive, no catch-all. Key rows: birth `on_attach_ok() -> (Loading{0}, [SpawnProbe])`, `on_attach_failed(msg) -> Error(AttachFailed)`; `Loading+Finished → Loaded`; `Loading+Started → Loading{0}` (commit resets the deadline — the server answered, render gets its own window); `Loading+Probe(fault) → Error(fault)`; `Loading+PumpTick{ticks==LOAD_DEADLINE_TICKS} → Error(LoadHung)`; `Loaded+Started → Loading{0}` (a real same-origin nav re-loads honestly, no re-probe — Finished/deadline guard it); `Loaded+Probe(fault) → Error(fault)` (the 5xx-commits case: probe truth outranks a success-shaped body); **Error is STICKY** (first-fault-wins, #384) — only `ReloadRequested`/`Closed` leave it; `*+RendererGone → Error(RendererGone)` (except sticky Error keeps its first fault); `*+Closed → (None, [Teardown])` — TOTAL from every state (close-while-loading/hidden route the same row).
- Reload vs retry (two directives, decided pure): `Loaded+ReloadRequested → (Loading{0}, [IssueReload])` (in-place `WKWebView.reload`); `Error+ReloadRequested` and `Loading+ReloadRequested → (Loading{0}, [Retry])`. `retry_plan(derived: Option<&str>, mounted: Option<&str>) -> RetryPlan { LoadSame, ReAttach(origin), Unconfigure }` — D2's re-derivation: the executor re-runs the BOOT chain (`mcp_json_path` keyed on the restored ACTIVE root, `PR-claude-boot-decisions-key-the-restored-active-root-001` → read → `forge_web_base`); same origin + live holder → `load_url`; changed origin or no holder → drop + fresh attach (the nav-handler's pinned origin is attach-time, so an origin CHANGE REQUIRES re-attach); derived None → holder+phase cleared → the #403 placeholder (unreachable ≠ unconfigured, D2's two truths).
- Render verdict (pure, composes the UNTOUCHED #405 axis): `phase_shows_webview(phase) = matches!(Some(Loaded))`; shim visibility = `webview_visible(active, overlay) && phase_shows_webview(phase)`; `body_plan(phase) -> { Placeholder (None), LoadingCard, ErrorCard(fault), PlainBackdrop (Loaded) }` — the webview composites ABOVE the scene, so a Marley-drawn card REQUIRES the child hidden; loading/error therefore hide it by construction, and visibility inputs never enter `step` (REQ-006's independence is structural).
- Executor (masked, app.rs): `SpawnProbe` → one short-lived `std::thread::spawn` per load attempt (bounded by the probe's own connect/read timeouts; a dead receiver's send error is ignored); `IssueReload`/`Retry` as above; `Teardown` → `reap_browser_holder` (now also clears `browser_load`, bumps `browser_gen`). Pump drain arm (the :1811 landmark): drain rx → drop stale gens (pure `signal_applies(tag, gen)`) → map → `step` → execute; plus one `PumpTick` step per frame while `Loading`. `LOAD_DEADLINE_TICKS = 15_000 / PUMP_INTERVAL_MS` (≈937 ticks / 15s — generous because the probe owns the fast-fail path).
- Traps designed around (sweep-sourced): `load_url` PANICS on an NSURL-rejected string (mod.rs:847 unwrap) — every URL we feed is #404's WHATWG-normalized origin, noted at the call sites; eval-queues-forever pre-commit — never used as liveness; `reload()` is infallible-by-construction (discard-Ok is honest).
- §14: typed faults end-to-end (`LoadFault`/`ProbeOutcome`), no unwrap on any load/derive path, spawns confined to the adapter/executor, `ClampedReason` owns its invariant with a private field, `ProbeOutcome` single-owner in marley_forge_client (browser_state maps, never re-declares).

### §20 confirm
**N/A — Marley-specific STANDS** (behavior maps re-verified browser-free at Phase 1; nothing here consults Warp/Zed source). Published-convention match: VS Code Simple Browser's preview-grade contract (explicit reload affordance + visible loading signal) — matched at the docs level by the palette verb + LoadingCard; Zed's chrome-less item-level reload convention supports NO-header-row. Substrate contract adopted from wry 0.56 source (permissive, in-registry — the Prior-art leg-3 sweep above). Clean-room wall intact.

### File manifest
**marley-web half (BUILT + VISUALLY APPROVED FIRST at P3):**
- CREATE `artifacts/marley-ide/src/components/views/BrowserPane.tsx` — the pane chrome-states surface: LoadingCard (spinner/skeleton vocabulary from `ui/spinner.tsx`/`ui/skeleton.tsx`), ErrorCard (#384 header-names-the-fault + clamped reason + Retry affordance in the #378 failure-line idiom), Loaded backdrop; the D-OPEN-HEADER-ROW A/B experiment (verdict recorded); a dev state-cycler so all three states inspect at localhost:5173.
- EDIT `artifacts/marley-ide/src/pages/Workspace.tsx` — route the browser tab body through `BrowserPane` states (placeholder stays `BrowserPlaceholder`).
- (at P5) `marley-web/docs/MARLEY-PARITY.md` — port-map row: `views/BrowserPane.tsx` ↔ `browser_state.rs` + the app.rs card arms.

**crates half (the 1:1 port target):**
- NEW `crates/marley_app/src/browser_state.rs` — everything under Architecture above + full unit suite (cov/MSI 100).
- `crates/marley_forge_client/src/adapter.rs` — `ProbeOutcome` + `probe_web_origin(origin, timeouts) -> ProbeOutcome` (raw GET `/` via the :134 idiom; https → TCP-connect-only → `TcpOnlyOk`; DNS resolve errors → `HostNotFound`) + pure status-line reuse (`http_status_code`); consts for connect/read timeouts.
- `crates/marley_forge_client/tests/livewire.rs` — probe rows on the loopback lane (200 / 500 / refused / accept-no-respond timeout / unresolvable host).
- `crates/marley_app/src/webview_shim.rs` — `attach(...)` grows `(gen: u64, tx: Sender<(u64, BrowserSignal)>)`: wires `with_on_page_load_handler` (Started/Finished → send) + `with_on_web_content_process_terminate_handler` (macOS cfg) with plain `(Sender, u64)` captures; stores `origin: String` (+ accessor) for `retry_plan`; adds masked `reload()` / `load_url(origin)` pass-throughs.
- `crates/marley_app/src/browser.rs` — the card copy fns only (loading headline; error header/hint vocabulary — final strings settled by the React stage), #405 seams untouched.
- `crates/marley_app/src/app.rs` — RootView fields `browser_load` / `browser_gen` / `browser_signals: (Sender, Receiver)`, retire `browser_attach_error` into `Error(AttachFailed)`; mount-gate update in `sync_browser_webview` + visibility composed with `phase_shows_webview`; pump drain+deadline arm at the :1811 landmark; the directive executor; render arm: LoadingCard/ErrorCard bodies in the #395 centered-panel shape with the retry click seam; `cockpit_commands()` += id-31 "Browser: Reload" row; activation flows through `action_for_command` → `dispatch_action` arm `"browser-reload"`; `reap_browser_holder` clears phase + bumps gen; test hooks (`browser_step_for_test`, `browser_phase_for_test`, `pump_browser_for_test`, signal-injection sender).
- `crates/marley_app/src/palette.rs` — `CommandId(31) => Some("browser-reload")` + the :301 map test row + the #399-lineage reverse-guard (31 unswallowed by dynamic ranges).
- `crates/marley_app/src/headless_drive.rs` — the driven matrix (below).

### Regression test plan (≥1 row per REQ)
| REQ | Tests |
|---|---|
| REQ-001 | Units: `on_attach_ok → (Loading{0}, [SpawnProbe])`; every entry edge → Loading (attach / Retry / IssueReload / Loaded+Started); `Loading+Finished → Loaded`; exhaustive-no-catch-all (a `match` over the full product; adding a variant breaks compile). Driven: inject Started/Finished via the signal sender → `pump_browser_for_test` → phase flips; `body_plan` LoadingCard→PlainBackdrop. |
| REQ-002 | Adapter livewire (REAL loopback): 200→`Status(200)`, 500→`Status(500)`, killed listener→`ConnectRefused`, accept-then-silence→`TimedOut`, bogus host→`HostNotFound`. Units: classify map total over `ProbeOutcome`; sticky Error (first-fault-wins matrix); `retry_plan` all four arms (LoadSame / ReAttach-on-changed-origin / ReAttach-on-no-holder / Unconfigure); `ClampedReason` strips controls+bidi, caps 200 (donor-parity with `clamp_card_text`). Driven: inject `Probe(ConnectRefused)` → Error; `ReloadRequested` → Loading + Retry directive; inject probe-ok + Finished → Loaded (the serve→kill→retry→recover cycle at machine level). Headed lane: live loopback serve→kill→retry (operator protocol if env-blocked, recorded). |
| REQ-003 | Units: `action_for_command(CommandId(31)) == Some("browser-reload")`; the :301 map test extended; reverse-guard: 31 hits NO special id and NO dynamic range (the #399 id-collision trap shape); `cockpit_commands()` carries the row (id+title). Driven: dispatch with Browser active + Loaded → IssueReload → Loading; dispatch with NO browser pane → no-op (phase untouched, None stays None). |
| REQ-004 | Existing census suite stands (door-drop/reopen :11325, restore :11419). New driven: every close path (switch-handback / close_tab_at / close_project_at) with an injected phase → phase None + census 0 + gen bumped; double-release stays safe. §18.1 site audit re-walks all `release_view` consumers. |
| REQ-005 | Driven: close-while-Loading, close-while-hidden (overlay up), overlay-then-close — each: phase None, holder None, attach-gate re-armed, stale-gen signals dropped after close (inject a pre-close-gen probe result post-close → no phase resurrection). INV-NO-ORPHAN: wry's Drop-detaches fact (adopted, cited) + holder-drop totality; headed hierarchy walk where the harness allows. |
| REQ-006 | Units: `step` never consumes visibility (structural — no visibility param exists; assert via the API shape + the product-space matrix `phase_shows_webview × webview_visible` all 8 arms). Driven: overlay over Loading → phase persists + PumpTick continues (deadline not paused by hides); tab away/back with Error → ErrorCard re-presents same fault; hidden Finished → Loaded while hidden → re-shown shows webview. |
| REQ-007 | `scripts/gates.sh --diff` exit codes (cov/MSI floors on browser_state.rs + browser.rs + the adapter probe classify); `cargo mutants --list -f` on ACTUAL touched files + neighbor-skip re-verify (the 5th-strike rule); React↔Marley parity pair captured at P4 (loading + error states, sampled per MARLEY-PARITY.md). |
| Type contract | `ClampedReason` private-field construction — `compile_fail` doctest (the house trybuild posture for a single-newtype privacy contract; full trybuild harness if one already exists in the crate). |

**Genuinely uncoverable (documented, §0):** the wry FFI verbs (existing masked pattern, unchanged), the app.rs glue (coverage-excluded crate posture; every decision it consults is a tested pure seam), the LIVE composite (webview actually painting/hiding over the Metal scene) — the headed/operator lane.

### Risks (reversible-but-load-bearing)
1. **Probe races the truth** — probe-ok then render-hang is caught by the deadline; probe-fault then a somehow-committed body keeps the STICKY error card (retry is one click; honest + predictable beats auto-flapping).
2. **https origin** → `TcpOnlyOk` (no status visibility; the adapter is TLS-free by posture) — Forge is http today; recorded, not solved.
3. **Reload of a Loaded page shows the LoadingCard** (webview must hide for the card to paint) — honest per D1; the React stage evaluates the feel; if it grates, the fallback (keep webview shown until Finished) is a one-row machine change, recorded as the alternative.
4. **SPA reality:** in-app route changes never commit → no spurious Loading (good); server-rendered multi-page origins re-enter Loading per click — honest.
5. **Pump-tick deadline pauses if gpui throttles the timer while occluded** — acceptable for a backstop.
6. **`Started`-resets-deadline** gives a slow post-commit render its own full window — deliberate.
7. Renderer-terminate hook is macOS-only (cfg-gated); other targets simply never see `RendererGone` — the arm stays total.
8. The retired `browser_attach_error` field must migrate everywhere (:10844 gate, :10851 set, :10871 reap) — grep-audited at implement.

## Phase 3 — Implement (2026-08-08, autonomous /goal)

### React-first stage (FIRST, per the parity contract)
- Dev server up (`pnpm --filter @workspace/marley-ide run dev` → localhost:5173, background).
- **CREATED `components/views/BrowserPane.tsx`** — the three chrome states in the house two-tier
  centered idiom + a POC-only bottom-right state cycler (placeholder/loading/error/loaded + header
  toggle; never ports). **EDITED `pages/Workspace.tsx`** — the `'web'` route now renders
  `BrowserPane` (placeholder reachable inside it; `BrowserView`'s simulated web half stays
  unreachable theater). Typecheck green.
- **Visually inspected (screenshots READ):** `.playwright-mcp/406-react-loading.png` (Loading… +
  origin caption, two-tier), `.playwright-mcp/406-react-error.png` ("Browser · unreachable" header +
  "connection refused — http://127.0.0.1:8080" + "↻ Retry" hover line),
  `.playwright-mcp/406-react-loaded-headerB.png` (the A/B's variant B).
- **D-OPEN-HEADER-ROW verdict: NO header row v1 (variant A ships).** The B pixels showed a 28px row
  spending pane height on the user's own config (a pinned origin that never changes at runtime) + a
  reload affordance the palette already owns, double-booking the top bar directly above — decorative
  redundancy against the flattest-surface posture; Zed's chrome-less item-reload convention concurs.
- **Settled copy (ported verbatim):** loading headline "Loading…" + origin caption; error header by
  fault class ("Browser · unreachable" / "Browser · web process exited" / "Browser · failed to
  embed" — the React demo showed the dominant class; the other two share the exact layout); reason
  line "{reason} — {origin}"; retry line "↻ Retry". Loading is deliberately TEXT (the
  `forge_empty_hint` textual vocabulary; no new gpui animation seam) — `ui/spinner.tsx`/
  `skeleton.tsx` considered and rejected for parity cost.

### Rust half (the 1:1 port, per the P2 manifest)
- **NEW `marley_forge_client/src/probe.rs`** (+ lib.rs export): `ProbeOutcome`
  {Status/TcpOnlyOk/ConnectRefused/HostNotFound/TimedOut/Io}, `probe_web_origin` (raw TcpStream GET
  `/`, connect 1500ms / read 2500ms, https → TCP-only), pure `split_origin` (WHATWG two-scheme,
  bracketed IPv6) + `classify_io` (one ErrorKind decision table). **Deviation from P2 manifest:**
  probe lives in a NEW `probe.rs`, NOT `adapter.rs` — adapter.rs is wholesale coverage-excluded, and
  a loopback-testable fn must not hide in an excluded file (§0 ACCEPTED-UNTESTABLE honesty).
- **NEW `marley_app/src/browser_state.rs`**: `ClampedReason` (private field, clamps via
  `fleet_rail::clamp_card_text` at construction), `LoadFault` (8 arms), `LoadPhase`
  {Loading{ticks}/Loaded/Error}, `BrowserSignal`/`BrowserEvent`, `step` (exhaustive, no catch-all;
  sticky Error; Started-resets-deadline; probe-truth-outranks on Loaded), `on_attach_ok`/`
  on_attach_failed`, `classify_probe`, `RetryPlan`/`retry_plan`, `phase_shows_webview`,
  `BodyPlan`/`body_plan`, `signal_applies`, `fault_header`/`fault_reason`/`error_reason_line`,
  `LOAD_DEADLINE_TICKS = 15_000/PUMP_INTERVAL_MS`. **Deviations:** (1) NO `Closed` event /
  `Teardown` directive — `reap_browser_holder` is the ONE teardown seam (unconditional over every
  phase; a machine row prod never routes would be decision-table theater; close-totality is
  driven-proven instead). (2) The planned `compile_fail` doctest is DROPPED — the module is
  crate-private so doctests never build against it (a false-passing compile_fail is a lie-shaped
  test); the privacy contract is compiler-enforced and the clamp is unit-pinned at P4.
- **`webview_shim.rs`**: `attach` grows `(generation, signals: Sender<(u64, BrowserSignal)>)` —
  wires `with_on_page_load_handler` (Started/Finished forward-only) + macOS-cfg'd
  `with_on_web_content_process_terminate_handler` (RendererGone); stores the pinned `origin` (+
  accessor); adds masked `reload()` + `load_pinned()` (Results discarded — wry returns `Ok(())`
  unconditionally on both, P2 sweep; the probe + deadline own failure truth). Weak-capture rule
  holds: handlers capture only `(Sender, u64, String)`.
- **`browser.rs`**: `loading_headline()` + `retry_affordance()` copy fns (React-settled); #405
  seams untouched.
- **`app.rs`**: fields `browser_load`/`browser_gen`/`browser_signal_tx`/`browser_signal_rx`
  (channel built at the boot-derivation site); `browser_attach_error` RETIRED into
  `Error(AttachFailed)` (all 5 sites migrated); mount gate now `holder.is_none() &&
  browser_load.is_none()`; visibility composes #405's axis `&& phase_shows_webview(..)`;
  `pump_browser` (gen-gated drain + one PumpTick while Loading) wired into the pump beside the
  `forge_pending` arm; `apply_browser_event` (take/step/execute); `execute_browser_directives`
  (SpawnProbe → one short-lived thread; IssueReload; Retry → `derive_forge_web_base_now` (the boot
  chain re-run, active-root-keyed) → `retry_plan` arms: LoadSame → `load_pinned`+probe, ReAttach →
  swap origin + clear holder/phase + gen-bump → next-frame re-mount, Unconfigure → full clear →
  placeholder); `reap_browser_holder` extended (phase cleared + gen bumped — the machine dies with
  the holder); render arm rebuilt over `body_plan` (shared `frame()`/`card()` closures; Loading
  card; Error card with the #378-idiom retry line wired to `browser_reload_requested`);
  `browser_tab_is_active` helper extracted (dedup of the #405 guard, now shared with the verb);
  `cockpit_commands()` += id-31 "Browser: Reload"; `dispatch_action` += `"browser-reload"`; six
  `#[cfg(test)]` driven-door hooks (phase get/force, pump, sender, gen, verb).
- **`palette.rs`**: `CommandId(31) => Some("browser-reload")` row.
- `cargo fmt` + `cargo check --workspace` GREEN; clippy clean except the expected
  unused-test-hooks warning (P4's tests consume all six hooks; the gate's `-D warnings` at
  validate forces that). The `block v0.1.6` future-incompat note is pre-existing upstream.

## Phase 3.5 — Inspect (2026-08-08, autonomous /goal)

Four independent critics over the working-tree diff (teardown-leak adversary; state-machine
correctness vs EARS; security/secrets/§20 provenance; state-integrity + simplification), lead-reviewed
skeptically. My own pre-pass first: the two command-registry invariant tests pass by construction
(id 31 in both tables, not special, unique), the two other `cockpit_commands` consumers are
additive-safe, and the reap-always-bump interleaving is clean (bump only fires with the resident
gone).

### Ledger — confirmed REAL and FIXED
| # | Sev | Finding (convergence) | Fix |
|---|---|---|---|
| 1 | MED | **Intra-generation stale-probe race**: retries spawn fresh probes under the SAME gen; a pre-retry probe's late fault lands on the post-retry attempt, and sticky Error swallows the true recovery (teardown critic + correctness critic, independently; both flagged that a gen-bump "fix" would orphan the live webview's Started/Finished and wedge reloads into LoadHung). | The staleness gate grew a SECOND axis: `probe_attempt: u64` on RootView, bumped by every `SpawnProbe`; `BrowserSignal::Probe { attempt, outcome }`; the pump drops mismatched attempts via the same pure `signal_applies` law (doc updated to name both axes). |
| 2 | HIGH | **Probe connected only to `addrs.first()`**: `localhost` resolves `::1`-first on macOS while a v4-only sidecar refuses it → permanent false "unreachable" card over a page WKWebView loads fine; retry re-probes the same addr — wedged (integrity HIGH + correctness MED + security's adjacent note — full convergence). | Try EVERY resolved addr, first successful connect wins, classify only when all fail (from the last error) — `TcpStream::connect`'s each-addr semantics, the adapter's own idiom. P4 adds the livewire row (v4-only listener + `localhost`-spelled origin). |
| 3 | MED | **`Unconfigure` conflated transient read-failure with genuine absence**: a retry click racing an editor's atomic save (`.mcp.json` momentarily unreadable/mid-write) tore the pane down for the session — the placeholder has no retry affordance and nothing re-derives until relaunch (teardown + correctness critics, convergent). | `derive_forge_web_base_now` now returns `(file_found, derived)`; new pure `effective_retry_base(file_found, derived, current)`: no file → Unconfigure; file present but underivable → KEEP the current origin (conservative — worst case is the card persisting against the old origin; nothing destroyed by an ambiguous read). |
| 4 | MED | **Teardown triple hand-written at 3 sites** (reap + ReAttach + Unconfigure) — the byte-equality-across-copies drift class fleet_rail already outlawed (integrity critic). | One `teardown_browser_life()` (holder drop + machine kill + gen bump); the `forge_web_base` writes stay at the two arms where they legitimately differ. |
| 5 | LOW | **Per-tick `cx.notify()` while Loading**: `Loading{t} != Loading{t+1}` → dirty every 16ms for up to 15s repainting a card with no tick-derived pixel (correctness + integrity, convergent; the feared String clone was verified a non-cost). | `changed` is now a DISCRIMINANT compare (render-visible transitions only; `Error(a)→Error(b)` unreachable under sticky — documented at the fn). |
| 6 | LOW | **Probe read loop had per-read timeouts but no total deadline**: a hostile loopback dribbler (1 byte/2.4s) could park the thread ~42 min inside the byte cap (security + correctness, convergent; blast radius small — the gates drop the eventual send). | Wall-clock budget: `Instant` deadline at entry; each iteration shrinks `set_read_timeout` to the remainder; exhausted → `TimedOut`. |
| 7 | LOW | **1xx classified as a sticky HTTP fault** (`103 Early Hints` before a final status → retry-proof wrong card) (correctness critic). | `classify_probe` reachable-Ok widened to `(100..400)` — any status line proves a live HTTP server; the PAGE verdict belongs to wry; a 1xx-then-hang origin still lands on the deadline's `LoadHung`. |

### Ledger — REJECTED (with reasons)
- **Off-origin 3xx → `LoadHung`'s imprecise reason** (correctness LOW): fixing needs `Location`
  parsing + a policy-vs-failure classification for an exotic misconfiguration (the forge origin
  redirecting off-origin); the card that shows is typed and truthful-but-unspecific ("page load
  timed out" — the page truly never loaded). Documented as a known limitation at `classify_probe`.
- **`IssueReload` holderless if-let as a debug_assert candidate** (integrity, self-rejected): the
  silent drop is REQUIRED — P4's close-matrix rows drive injected phases holderless by design.
- **`BrowserSignal`/`BrowserEvent` merge, `load_pinned` rename, pub-vs-pub(crate), Disconnected
  arm** (integrity, all self-rejected on verification): each earns its place / matches house idiom.

### Adversary walks that came back CLEAN
- **INV-NO-ORPHAN-WEBVIEW**: all three close paths release→reap in order; view accounting
  balanced; app-quit rides RootView drop → wry `removeFromSuperview`, inline by `!Send` necessity;
  no missing clear (AttachFailed's holder-None/phase-Some is the designed stored error).
- **Cross-generation gate**: teardown clears + bumps in straight-line main-thread code; the pump
  gates at drain; a passing signal still needs a live machine; two attaches can never share a gen.
- **Mount-gate re-entry**: blocked after reap (tab gone), re-armed after ReAttach, parked on
  `Error` (stored-not-retried). **LoadSame recursion**: depth-1, owned Vec, unobservable transient.
- **Security**: header injection REFUTED by fuzzing the REAL derivation with 20 hostile configs
  (the #404 wall constrains the authority charset); no bearer anywhere near probe/cards/logs;
  weak-capture holds for all three wry closures; every card string clamped at construction; the
  wry NSURL panic unreachable from the derivation's output shapes. Residual (pre-existing,
  #404-accepted): a hostile config can aim pane+probe at any loopback port.
- **§20 provenance**: house idioms + wry public-API adoption (MIT/Apache dual-licensed); nothing
  Warp/Zed-derived. PASS. **Skip-detach re-check**: every new masked fn carries a justified skip;
  the formal `cargo mutants --list -f` walk runs at P4 per the Floors note.
- Post-fix `cargo fmt` + `cargo check --workspace` GREEN. Forge unreachable — `failure-record` /
  `prevention-rule-record` capture deferred with the P5 batch (§19 best-effort).

## Phase 4 — Validate (2026-08-08, autonomous /goal)

### Tests written (per the P2 plan + the P3.5 inspect additions)
- **browser_state.rs units (17):** birth arms exact; Loading rows (Started-resets-ticks, Finished,
  probe ok/fault, deadline boundary ±1, RendererGone, mid-load reload→Retry); Loaded rows (nav
  re-entry+probe, idempotent Finished, PumpTick no-op, probe-outranks-5xx, reload→IssueReload+
  SpawnProbe); sticky-Error absorption loop over all five signals + retry exit; `classify_probe`
  all arms (1xx/2xx/3xx ok, 4xx/5xx fault, TcpOnlyOk, transport arms, Io clamps);
  `ClampedReason` donor-parity clamp (control+bidi strip, 200 cap); `retry_plan` 4 arms;
  `effective_retry_base` 4 arms (the P3.5 ambiguity collapse); `phase_shows_webview`;
  the FULL 16-arm visibility product space (#405 axis × phase axis); `body_plan` 4 arms;
  `signal_applies` (one law, both axes); `fault_header` 3 classes / all 8 faults;
  `fault_reason` + `error_reason_line` exact; `LOAD_DEADLINE_TICKS == 937` pin.
- **probe.rs units (5):** `split_origin` http/https/ports/IPv6-bracket shapes + 7 refusal arms;
  `classify_io` exact; `connect_failure` both arms (the P3.5 coverability restructure — the
  zero-addrs edge is now a pure pinned arm, not unreachable defense); unprobeable-origin → Io.
- **livewire rows (9, REAL loopback):** 200 / 500 / malformed-garbage→Io / refused /
  accept-then-silence→TimedOut (~2.5s) / **dribbler-hits-total-deadline** (the P3.5 fix's row) /
  **cap-exit-on-newline-free-flood** (kills the cap-widening mutant: Io vs mutant-TimedOut) /
  **localhost-origin-reaches-v4-only-listener** (the P3.5 HIGH's row) / https→TcpOnlyOk /
  `.invalid`→HostNotFound.
- **headless driven matrix (7):** close-while-Loading via ⌘W (phase None + census 0 + gen bump);
  close-while-Error via project close; the pump's TWO staleness gates through the REAL channel
  (matching Finished lands; stale-GEN RendererGone dropped; stale-ATTEMPT probe fault dropped;
  current-attempt fault lands); reload-verb guard (non-Browser tab no-op; Browser tab dispatches,
  holderless directives safe); deadline→LoadHung through real pumps; retry-with-absent-config →
  Unconfigure teardown + gen bump; RendererGone→card.
- **palette/app registry:** `action_for_command(31)` row; the statics-below-dynamic-bases pin
  (generalizes the #399 collision guard); browser.rs chrome-copy pins. The two pre-existing
  registry invariant tests pass with the new row by construction.
- **Type contract:** `ClampedReason` privacy is compiler-enforced (crate-private module —
  doctests can't build against it, so the planned compile_fail was replaced by the behavioral
  clamp pin; recorded at P3).

### Test runs (real output)
- `cargo nextest run --workspace` → **`Summary [15.116s] 2135 tests run: 2135 passed, 5 skipped`**
  (the 5 skips are the pre-existing env-gated headed lanes).
- `cargo test --workspace --doc` → ok (no failures across all crates).
- Final post-gate-chase count: **`2144 tests run: 2144 passed, 5 skipped`** (the 9 additions: the
  mutant-killer empty-workspace row, the EOF/RST/dribble/cap livewire rows, and the
  per-binary instantiation rows the coverage chase demanded).

### The LIVE drive (self-test harness) + parity pair
Launched per the README's TCC gotcha (direct-exec after the `open` stall — the documented
/Volumes/Offload dialog case). Captures in `.playwright-mcp/` (gitignored), all READ:
1. `406-live-boot.png` — restored workspace, Browser section present, collapsed.
2. `406-live-menu.png` — the Browser section ＋ menu (Forge/Agents/Details/**Browser** — the #403
   4th row).
3. **`406-live-error-card.png` — REQ-002 LIVE**: with the forge genuinely down, opening the
   Browser tab attached, wry stayed silent (as the sweep predicted), the probe classified
   `ConnectRefused`, and the pane rendered **"Browser · unreachable" / "connection refused —
   http://127.0.0.1:8080" / "↻ Retry"** — centered two-tier, webview hidden, no header row.
4. **`406-live-recovered.png` — REQ-002 retry-recovers LIVE**: started a loopback fixture origin
   on 8080, clicked ↻ Retry → re-derive → LoadSame → load+probe → Started/Finished → **the wry
   child painting the fixture page at exactly the pane rect** (rail/files intact around it).
5. **`406-live-after-verb.png` — REQ-003 LIVE**: ⌘⇧P → "browser reload" → Enter — the palette
   overlay hid the webview (the #405 shim), the verb dispatched IssueReload, the page reloaded
   and re-showed Loaded. No flicker artifacts, no stale frame.
6. **`406-live-closed.png` — REQ-005 LIVE**: ⌘W on the Browser tab — tab gone from the rail,
   focus back on the terminal, NO orphan webview painting anywhere (INV-NO-ORPHAN-WEBVIEW
   witnessed on pixels). Chad's workspace left as found.
**Parity pair (React `406-react-error.png` ↔ live `406-live-error-card.png`), pixel-sampled via
magick:** headline strip maxima 142,146,159 ↔ 134,138,150; caption 90,93,101 ↔ 81,84,92 —
caption/headline luminance ratio 0.634 ↔ 0.604, both ≈ the specified /60 tier
(`text-muted-foreground/60` ↔ `colors.muted.opacity(0.6)`); retry == headline muted on both
(unhovered); bg (11,12,15) ↔ (14,15,17). Copy identical, geometry identical (centered two-tier +
retry line), casing identical. Absolute Δ≈8/255 is capture-space gamma (Chrome sRGB vs
screencapture), not a design delta. **PARITY: PASS. The approved React prototype was the
reference; no Rust fix needed; no marley-web fix needed.** marley-web typecheck green at P3.

### The mutants walk (the 5th-strike rule) — and its catch
`cargo mutants --list -f` over the five touched files: browser_state 37 / probe 35 / browser 47
(mostly the pre-existing #405 seams) / palette 55 / **webview_shim 2 — the walk CAUGHT the
unskipped `origin()` accessor** (a bare field read constructible only over a live window; its
mutants would have survived to a red gate). Skipped with justification per the file's
ACCEPTED-UNTESTABLE posture — exactly the drift class
`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators` exists to catch, caught
BEFORE the gate this time.

### Gate
- **`scripts/gates.sh --diff` → GATE GREEN [diff], 15 passed 0 failed, exit 0** (receipt written).
- The road there (three reds, each fixed at source): **(1) rustfmt** — heredoc-appended tests
  unformatted → `cargo fmt`. **(2) mutation MSI** — ONE missed mutant, `>`→`>=` in
  `browser_tab_is_active` (the #395 empty-workspace arm no test drove) → killed by the new
  `browser_reload_verb_total_over_empty_workspace_headless` row (0 projects: the mutant panics on
  the empty Vec, the original no-ops). **(3) coverage** — first 3 REAL missed probe lines
  (write-err arm undeterministic on loopback → folded into the read's one failure surface;
  the deadline-bail branch raced the read-timeout → replaced with a ≥1ms-clamped remaining so
  enforcement rides one path; the EOF break → the short-close livewire row). Then a
  PHANTOM missed line no per-line view (lcov/text/html/missing-lines) could name — llvm's
  per-instantiation line tables: each test binary carries its own instantiation of the probe fns,
  and lines executed in no BINARY-local instantiation count missed even when another binary
  covers them. Resolved by executing the probe's arms in EVERY linking binary (unit-side +
  app-side port-1/ftp/loopback-200 rows) — and the two fixture `if let Ok(accept)` implicit-else
  arms that introduced fresh honest misses were rewritten as `expect` accepts (fixture code;
  panic-on-fail is the correct posture). End state: **probe.rs 172/172, headless_drive
  8587/8587, TOTAL 46925/46925 lines — 0 missed.**

### Pre-existing (not in scope)
- The `block v0.1.6` future-incompat warning (upstream transitive, pre-dates #406).
- 5 skipped tests: the env-gated headed lanes (pre-existing).

## Phase 5 — Complete (2026-08-08, autonomous /goal)

### §21 documentation
- **CHANGELOG.md**: the #406 Added entry (machine, out-of-band probe, inline-by-`!Send` teardown,
  two-axis staleness, CommandId(31), no-header-row verdict, the live proofs).
- **embedded-browser-model.md**: train item 4 → ✅ SHIPPED, **the #389 Phase-E train is
  COMPLETE**; the entry records the settled D-OPENs and retires the "mirror the PTY reaper"
  prediction with the `!Send` evidence.
- **content.rs** `release_browser_views` doc: the #403-era off-thread prediction REPLACED with
  the settled truth (app-side holder, inline drop by type necessity, `reap_browser_holder` as
  the one teardown seam) — a comment-only .rs change; /commit's own gate run re-receipts.
- **Parity sync**: MARLEY-PARITY.md gains the `BrowserPane.tsx` port-map row + the #406 route
  note ("web" renders BrowserPane; BrowserView's simulated half stays unrouted theater);
  marley-web committed (`3a0358b`, typecheck green). The 1:1 holds at close.

### Knowledge capture — forge unreachable at every phase of this pipeline (connection refused,
### 127.0.0.1:8080) — captured locally per §19; submit to forge when it returns
- **AAR (what worked):** (1) The **prior-art leg-3 sweep paid triple** — reading wry 0.56's
  source dissolved TWO locked D-OPENs before design ended (no-signal-on-transport-failure forced
  the probe design; `!Send` + Drop-detaches killed the off-thread teardown question and the #402
  measurement probe with it) and pre-empted a production panic (`load_url`'s NSURL unwrap,
  proven unreachable from the derivation). (2) **React-first earned its order**: the A/B header
  experiment took minutes in Vite and settled D-OPEN-HEADER-ROW on pixels before any Rust
  existed; the settled copy ported verbatim into pinnable pure fns. (3) The **forge-down
  machine became the test fixture**: the live drive proved the error card against the REAL
  outage, then a loopback fixture origin proved retry-recovery — REQ-002's full cycle on
  pixels with zero mocking. (4) The **selftest harness README's gotcha list** (TCC stall →
  direct-exec; activate-in-same-shell) was exactly right, twice.
- **Failures (each was real, each fixed at source):**
  - `F1` (inspect MED, two critics independently): the intra-generation stale-probe race —
    retries spawned probes under one generation; a pre-retry probe's late fault stuck a false
    sticky card over a recovered page. Fix: the attempt axis on the staleness gate.
  - `F2` (inspect HIGH, three critics convergent): first-resolved-addr-only probe connect —
    `localhost` → `::1`-first vs a v4-only sidecar = permanent false "unreachable". Fix:
    every-addr connect; the livewire localhost/v4 row pins it.
  - `F3` (inspect MED): retry's `Unconfigure` conflated a transiently unreadable `.mcp.json`
    with removed config — one mistimed retry click destroyed the pane for the session. Fix:
    `effective_retry_base` (absent → placeholder; underivable → keep current).
  - `F4` (validate, the mutants walk's catch): the unskipped `origin()` accessor on the FFI
    holder — its mutants were unkillable headless; caught by `cargo mutants --list -f` BEFORE
    the gate (the 5th-strike rule doing its job).
  - `F5` (validate, the gate chase): the llvm-cov **per-instantiation phantom** — a line
    executed in no BINARY-LOCAL instantiation counts missed even when another binary covers
    it, and NO per-line view (lcov/text/html/--show-missing-lines) can name it. Resolved by
    executing the probe's arms in every linking binary; the two fixture `if let Ok(accept)`
    implicit-elses that arose were rewritten as `expect` accepts.
- **Prevention-rule candidates (for `prevention-rule-record` when forge returns):**
  - `PR-candidate-per-binary-instantiation-coverage`: an UNMASKED fn in a crate linked by
    multiple test binaries must have its arms executed in EACH binary (or the file's lane
    documented) — the llvm line summary counts per-instantiation, and the phantom it produces
    is invisible to every per-line view. Symptom signature: "N missed lines" with an empty
    missing-lines listing.
  - `PR-candidate-fixture-if-let-accept`: in COUNTED source (src/ test modules,
    headless_drive), fixture accepts are `.expect(...)`, never `if let Ok` — the implicit
    else is an uncoverable zero region; panic-on-fail is the correct fixture posture anyway.
  - `PR-candidate-staleness-gates-per-supersession-axis`: a gen-tagged channel gate is
    life-scoped only; every RESTARTABLE async producer within one life (probes, fetches)
    needs its own attempt axis, or a superseded producer's late result corrupts state the
    sticky/first-wins consumer can't heal.
- **Architecture decisions settled (recorded in the spec/notes; `architecture-decision-record`
  when forge returns):** out-of-band probe + tick deadline + renderer hook as the error
  signal (wry's silence is structural); inline teardown by `!Send` necessity; sticky
  first-fault-wins cards; no header row v1.

### Ticket + archive
- TICKET-406 → `docs/planning/tickets/closed/`, `status: closed` (forge `ticket-close` deferred —
  unreachable; the frontmatter carries the forge id for reconciliation).
- Pipeline pair → `docs/planning/pipeline/completed/`.
