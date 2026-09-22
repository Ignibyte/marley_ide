# The Forge URL pane — the wry child wired to the pane rect + the z-order shim — Notes

- **Forge ticket:** #405 425c8332-8949-41bc-b252-008c922d85a0
- **AAR:** deferred — opened when /work promotes this to active
- **Local ticket doc:** docs/planning/tickets/open/TICKET-405-forge-url-pane.md
- **Pipeline spec:** 405-forge-url-pane.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** /spec batch (M29, forge sprint #40 6dff19c6-fb61-472b-89c3-98d3215a489b) — draft the
  Phase-1 spec for the #389 train slice-3: the Forge URL pane (wry child at the Browser pane's rect,
  the z-order hide-shim, #404-derived pinned origin, never-persisted URL). HARD-GATED on #402's GO;
  consumes #403 (residency/placeholder/codec) + #404 (origin seam); #406 is downstream.
- **Classification / tier:** feature, **L**, headed-driven-heavy (the deliverable is a masked
  platform surface — most REQ verifies are driven/headed per §7 honesty; the unit+mutation surface
  is the four pure seams: `overlay_is_up`, origin equality, rect mapping/diff, mount decision).
- **Forge recall (§18.3):** `bulletin-list` → **empty**. `sprint-get` 6dff19c6… → M29 confirmed
  active with all five train tickets; sibling uuids recorded in the spec's gate (402
  a17801ff-b56e-4a7d-ace4-b3252c49147e spike / 403 f7bf9657-b4c1-4fd6-b346-584660daf256 / 404
  e84add90-0452-4bfa-9772-4ca04a54d5ae / 406 f72268be-1018-4e0b-8843-10d70cd718a2). `ticket-get`
  #405 → description matches the sprint plan verbatim (the five lettered halves; its
  `cockpit_body app.rs:5451` / `fleet_rail_body app.rs:1006` anchors are LEGACY lines — today
  :5854 / :1076, corrected in the spec). `knowledge-search`/`knowledge-explain` (webview / z-order /
  overlay-hide / masked-surface) → top hits: AD cf751352-9cbe-49dd-b2d8-508b373a5393 (the #389
  substrate AD — content recorded verbatim in embedded-browser-model.md as
  `AD-claude-embedded-browser-substrate-001`) + prevention-rule nodes 739601c4 / 31c07755 /
  d99b6b26 / 3df9fa6e / 78ade569 / f639a481 / cbdd64a6. Named rules bound in the spec:
  `BF-claude-skip-detach-pump-fleet-live-001`,
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`,
  `PR-claude-boot-decisions-key-the-restored-active-root-001`,
  `PR-claude-registry-release-returning-a-resource-must-be-must-use-001` (the #406 teardown edge).
- **Discovery** (all against app.rs @ 35fd2c7, 2026-08-06):
  - **THE OVERLAY INVENTORY — the spec's crown jewel.** The overlay-draw region is ONE contiguous
    render tail, app.rs:19781–21117 (~28 independent `if` branches appended above the pane scene;
    `impl Render for RootView` starts :16339). Full table with state-field lines + draw lines lives
    in the spec's Scope-In. Shape: **Class A** 14 floating overlays reachable while a Browser tab
    is active (palette :19781 … context_menu :20999, incl. the non-obvious `forge_open` #69 sprint
    card :20096, `fleet_open` #68 ⌘⇧E :20129, `diff` :20358, both find bars :20419/:20456, and
    `status_flash` :20531 — the D-OPEN-FLASH fork); **Class B** 12 editor-anchored cards/pickers
    (:21072–:21115 — hover/code-action/def/references/⌘T/⌘⇧O/⌘⇧F/⌘⇧M/completion/signature/rename/
    goto) — unreachable-in-practice while a whole-tab Browser is active, membership deliberately
    unconditional; **Class C** excluded by construction — the ⌘⇧C git panel (:20185, keyed on a Git
    pane in the ACTIVE grid's `rect_list`) + terminal inline completion (:20898, `rect_list` lookup
    self-clears :20899-20901) — a Browser tab has no grid (`active_is_terminal` :18601 → empty
    `rect_list`); **Class D** chrome-inline non-overlays (renaming_tab :17834 rail row,
    session/top-search/commit focus states, focused border :19767). **Predicate = A+B = 26 states.**
  - **The choke-point truth:** NO existing boolean means "some overlay is drawing". The nearest
    thing, `text_input_blocked()` (app.rs:10597-10626, M16 #267), IS a proven one-predicate choke
    point ("a NEW overlay only needs a line here") but for TEXT CAPTURE, and its set diverges from
    the visual set in BOTH directions (omits `completion_menu` by design per its :10598 NOTE, plus
    hover/signature/forge/fleet/flash; includes four non-overlay focus states). So `overlay_is_up`
    must be a NEW pure sibling fn enumerating the 26 states — the spec's D2 — with the same NOTE
    discipline + a P3.5 completeness adversary re-deriving the set from the tree.
  - **The geometry truth:** the Browser pane's rect IS `center_bounds` (app.rs:18407-18412), pure
    arithmetic over `region_widths` (:17622; layout.rs:108) + `files_w` + `content_band`
    (layout.rs:126); the cockpit tab already renders full-screen at exactly this rect
    (:18468-18471). Rect mutators: window resize (viewport read each render), dock toggles
    (`dock()` :10166; right-dock :18290), Files panel toggle/drag (:460/:476; :18377/:18395),
    tab/project switch (`try_active_tab` :10642; cockpit dispatch :18455). Geometry consumed
    off-render is CAPTURED per frame today — `editor_geom: Rc<Cell<EditorFrameGeom>>` :513 written
    from the paint (:6577/:6594; the #352 lesson — CHANGELOG "fold projection reaches the
    frame-geometry layer") — and side-effects are diffed-before-applied per frame (PTY R37 :18614
    "resize only when the cell grid actually changed"). Both precedents ground D-OPEN-RECT-SYNC's
    recommended capture-and-diff arm.
  - **Dependency truths:** Cargo.lock TODAY has **zero `wry`** entries (also no `objc2-web-kit`);
    `raw-window-handle` 0.6.2 present (:4639); `core-video` present (the NO-GO fallback substrate
    already in-tree). gpui is a crates.io dep 0.2.2 (marley_app/Cargo.toml:16) — source reading is
    adoption (§20 leg 3). Marley uses NO gpui `deferred()` (zero hits in app.rs) → no adoptable
    overlay signal; finding recorded in the spec's Prior art.
  - **wry API (published, docs.rs live 2026-08-06):** `build_as_child(HasWindowHandle)` (macOS
    NSView subview) / `with_bounds` (**default 200×200@0,0 — first-frame hazard**) /
    `set_bounds(Rect)->Result` (child-only) / `set_visible(bool)->Result` / `load_url(&str)` /
    `with_navigation_handler(Fn(String)->bool)` (false = reject) / `with_focused`, `focus()`,
    `focus_parent()` / `reparent` is gtk-bound (no portable remove-from-superview) / drop semantics
    undocumented (→ #406).
  - **Parity grounding:** MARLEY-PARITY.md Zone A (:21, frozen Marley-drawn chrome/overlays) vs
    Zone B (:44, "The surfaces are free. What lives *inside* a pane is where we design");
    no-counterpart precedent row (`utils/commandSimulator.tsx` "no counterpart — POC-only"); port
    map `views/BrowserView.tsx` ↔ `right_dock.rs` :585. The webview's pixels are the Forge web
    app's own → the N/A-expected CONDITIONAL arm, flip protocol in the spec.
  - **Reference sweep:** `docs/warp_architecture/` + `docs/zed_architecture/` recursive grep for
    webview/WKWebView/embedded-browser → **zero hits** (matches embedded-browser-model.md's "No
    Warp/Zed analog"). VS Code Simple Browser README (MIT, fetched): "a very basic browser preview
    using an iframe embedded in a webview… for showing simple web content" — the pinned-simple
    published convention. (The v1.44/48/53 release notes checked did not name it; the extension's
    own README is the citation.)
  - **`forge_web_base` consumers today:** `mcp_json_path` mcp_config.rs:28; the one-config site
    app.rs:2403 → `forge_endpoint_from` :2409 + the #376 `endpoint_for_brain` decision (:2390/
    :2398) — the browser URL becomes consumer three (#404's seam). Shell codec: `serialize_shell`
    grid_layout.rs:334 / `restore_shell` :405 (the ticket's :355/:410 were legacy lines).
    `RailSection::Browser` tabs.rs:51/:60/:69 (model doc's :82 legacy).
- **Decisions:** D1–D7 + five D-OPEN forks locked in the spec (substrate-gated-on-#402; the
  26-state single-seam hide predicate; pinned origin over #404's type; never-persist; no bearer;
  additive coexistence; masked-thin-adapter discipline). Notable Phase-1 calls: `status_flash`
  membership deliberately left as D-OPEN-FLASH (include-by-default recommended — a swallowed
  confirmation is the silent-failure shape; the ~2s blink is the cost); Class-B states included
  unconditionally so reachability reasoning can rot without consequence; Class-C exclusions are
  by-construction (grid-keyed), not reachability guesses.

### Promotion to active (2026-08-07, /work 405)
- Queued pair → `docs/planning/pipeline/active/` (git mv); no other active pipeline (checked).
- **HARD promote-gate VERIFIED against tonight's tree:** #402 verdict RECORDED —
  embedded-browser-model.md:167 "✅ DONE — GO (M29 #402, 2026-08-06)" +
  `AD-claude-embedded-browser-substrate-001` (:202, wry-as-child WKWebView) → D1 stands.
  #403 SHIPPED (8d879f3 — TabContent::Browser + bare `B` tag + placeholder). #404 SHIPPED
  (4024f29, tonight — `forge_web_base` + the parsed-host re-check). #404's Phase-5 carries bind
  HERE: no web UI serves at the MCP origin yet (401 unauthed / 404 authed) — the pane will
  honestly render the 401 JSON on mount; #406 owns graceful error states; the hatch stays out.
- **Forge claim + AAR: DEFERRED (§19 best-effort)** — the forge MCP tools are not reachable in
  this session (ToolSearch: none); `.mcp.json` wiring itself verified
  (http://127.0.0.1:8080/mcp/forge). Claim #405 + open the AAR from a forge-wired session;
  Phase-1's recall record (bulletins empty; the named prevention rules) stands as the recall.
- Environment pre-flight (from /work): cargo 1.96.0, gates.sh OK, cargo-mutants 27.1.0,
  cargo-llvm-cov 0.8.7, hooks wired, marley-web OK, no active pipeline. **wry NOW IN LOCK**
  (1 entry — entered with #402's `marley_webview_probe`, per the spec's inherit-never-add note;
  gate:8 already vetted the tree at #402's landing). NB: the session shell's
  cwd `~/Projects/ignibyte/Marley` is a SYMLINK to `/Volumes/Offload/Projects/Marley` (verified
  same HEAD) — one tree, no clone drift.
- **Same-night context:** the delivery gate went DIFF-first tonight (2b9eab9 — FULL demoted to
  the deliberate overnight audit after the 2026-08-07 double freeze; mutation jobs mode-aware).
  This pipeline's Phase-4 gate runs `--diff` per the updated commit.md.

## Phase 2 — Design (2026-08-07, the /goal night run)

### D-OPENs settled — each from #402's RECORDED evidence (verdict block re-read + probe source re-read tonight)
- **D-OPEN-HIDE-MECHANISM → `set_visible(false)` primary.** Verdict: "the hide-shim holds under
  BOTH mechanisms — first-party set_visible(false) (primary) and zero-bounds (alternate)".
  Zero-bounds stays the recorded alternate. TWO probe invariants adopted verbatim: hide
  INVALIDATES the applied-rect sync key (probe main.rs:129-131) so restore re-applies the real
  rect via capture+diff; and NEVER focus a hidden webview (main.rs:288-294's recorded refusal —
  the keyboard-wedge hazard; moot in v1, which calls no focus verbs, binding for #406).
- **D-OPEN-RECT-SYNC → per-frame capture+diff IN RENDER** — exactly the probe's proven shape
  (main.rs:227-246): rounded-i32 `[x,y,w,h]` applied-key in a `Cell`, `set_bounds` only on key
  change, unrounded f64 applied. wry `set_bounds` is Logical→Logical — **no scale-factor math**
  (the verdict's slice-3 input). ONE sync site right after the render's tab-body dispatch — it
  must run when the Browser tab is NOT active too (that IS the hide path), self-healing a frame
  later per the #352 `editor_geom` lesson. No pump coupling.
- **D-OPEN-FOCUS-HANDOFF → v1 is HANDS-OFF.** No `focus()`/`focus_parent()`/`with_focused` call
  anywhere. gpui keeps first responder unless the user clicks into the page (AppKit-native);
  typing + IME inside the page is probe-proven. **HONEST GAP RECORDED:** the probe's clean
  return trip used the page's IPC button — ⌘-chords while the webview holds first responder are
  NOT proven to reach gpui (the spec's palette-trigger worry stands open). v1 exposure is
  bounded: we never steal focus programmatically, overlays also open by mouse, and the shim keys
  on STATE not input path. Driven check at P4; a fix (global shortcut monitor / page-IPC) is
  #406's if the check fails.
- **D-OPEN-WEBVIEW-HOME → APP-SIDE.** RootView gains `browser_view:
  Option<webview_shim::BrowserWebview>` + `browser_attach_error: Option<String>`; the !Send
  main-thread WebView stays OUT of the #394 registry's data plane; identity remains #403's one
  app-wide resident (the registry keeps the ContentId; the webview is the render-side
  attachment). Teardown v1 rides the #403 release path: the close arm that reaches
  `content::release_browser_views` (content.rs:397) drops `browser_view` when `find_browser`
  (content.rs:348) goes None — drop-on-last-close mirrored; #406 owns
  detach-off-the-render-path hardening. The weak-capture rule
  (`PR-claude-callback-stored-inside-owned-resource-captures-weak-001`) is satisfied BY
  CONSTRUCTION: the only stored closure (the nav handler) captures a plain origin `String` +
  calls a pure fn — no owner handle ever enters the WebView. Recorded against the rule.
- **D-OPEN-FLASH → INCLUDE** (the spec's recommended default): a swallowed confirmation is the
  silent-failure shape; the ~2s blink is the accepted cost. Flip criteria recorded for #406:
  exclude only WITH a driven capture showing the blink harms more than the pill informs.

### The overlay inventory — FROZEN @ 2b9eab9 (tonight; two grep passes, `if` + `if let` forms)
**All 26 states verified, ZERO new overlays since the spec's 2026-08-06 sweep.** Fresh draw
anchors — Class A (14): palette :19888 · naming_workflow :19950 · naming_pane :19979 ·
fleet_dispatch_draft :20028 · agent_launcher :20054 · finder :20119 · history :20163 · forge
:20203 · fleet :20236 · diff :20465 · find :20526 · efind :20563 · status_flash :20638 ·
context_menu :21109. Class B (12), now drawn via `*_overlay()` helper calls: hover :21182 ·
code_action :21186 · def_picker :21189 · references :21193 · symbols :21197 · file_symbols
:21201 · search :21205 · problems :21209 · completion_popup :21213 · signature :21217 ·
rename_draft :21221 · goto :21225. Class C re-confirmed excluded-by-construction (terminal
`completion` :21008 rect_list-keyed self-clearing; git panel — commit_* :20406-:20416 live
inside it). Class D unchanged (top_search :20932-:20945 etc.). Snapshot inputs = the underlying
STATE fields (Class B via `.is_some()` on hover_card / code_action_menu / def_picker /
open_references / open_symbols / open_file_symbols / open_search / open_problems /
completion_menu / signature_card / renaming_symbol / goto_line).

### Architecture (§14 typed/total; §20 re-confirmed N/A — Marley-specific, VS Code Simple
Browser stays the behavior-level convention only; no Warp/Zed material exists or was touched)
- **Pure seams (browser.rs — already #403's cov/MSI-100 pure module):**
  ```rust
  pub struct OverlayStates { /* 26 named pub bools, one per frozen-inventory state */ }
  pub fn overlay_is_up(s: &OverlayStates) -> bool          // the 26-arm ||; D2's one seam
  pub fn webview_visible(browser_tab_active: bool, overlay_up: bool) -> bool  // && !
  pub fn rect_key(x: f32, y: f32, w: f32, h: f32) -> [i32; 4]                 // probe rounding
  pub fn rect_changed(applied: Option<[i32; 4]>, next: [i32; 4]) -> bool      // diff-before-apply
  pub enum MountPlan { Attach(String), Placeholder }
  pub fn mount_plan(web_base: Option<String>) -> MountPlan  // Some→Attach, None→Placeholder (REQ-006)
  ```
- **Pure seam (marley_forge_client, beside `forge_web_base`):**
  `pub fn same_web_origin(candidate_url: &str, mounted_origin: &str) -> bool` — rides the
  private `web_origin_of` (ONE parser, and the #404 parsed-host loopback re-check comes free),
  so `Some(candidate_origin) == Some(mounted)` is the whole body. Initial `load_url(origin)`
  self-passes (an origin is its own origin); same-origin paths/queries proceed; every hostile
  shape (userinfo, non-loopback, scheme/port mismatch, subdomain suffix, malformed) lands
  `false` via `None` or inequality.
- **The one-config THIRD consumer lands at the boot site** (app.rs:2417-2427, beside the
  forge client + brain-endpoint consumers): `let forge_web_base = mcp_json.as_deref()
  .and_then(marley_forge_client::forge_web_base);` stored as `RootView.forge_web_base:
  Option<String>`. D4 reading: re-derived per LAUNCH (boot/restore — the same granularity the
  MCP client + brain use; one-config-coherence, `PR-claude-boot-decisions-key-the-restored-
  active-root-001` inherited from the shipped resolution). Nothing persisted; a mid-session
  `.mcp.json` edit retargets nothing until relaunch — coherent with both sibling consumers.
- **The masked adapter (webview_shim.rs, whole-file §0 ACCEPTED-UNTESTABLE):**
  `BrowserWebview { webview: wry::WebView, origin: String, applied: Cell<Option<[i32;4]>>,
  shown: Cell<Option<bool>> }` — `attach(window, rect, origin) -> Result<Self, String>`
  (`WebViewBuilder::new().with_bounds(...).with_navigation_handler(move |url|
  same_web_origin(&url, &origin_clone)).build_as_child(window)` + `load_url(origin)`);
  `sync(rect, visible)` (rect_changed → set_bounds; shown-diff → set_visible; hide invalidates
  `applied` per the probe invariant); `Drop` = wry drop. Every fn `mutants::skip` with the
  documented shape; the ONLY logic inside is "call the pure decision, apply the verdict".
- **The sync flow (app.rs, masked wiring):** render computes `center_bounds` (:18481) → tab-body
  dispatch (Browser arm :18549 keeps the #403 placeholder for `mount_plan == Placeholder` or
  attach-error) → ONE `self.sync_browser_webview(center_bounds, window)` site after the
  dispatch: mount step (browser resident exists && `browser_view.is_none()` &&
  `browser_attach_error.is_none()` && `MountPlan::Attach` → attach, error → stored, no retry —
  probe precedent), then `sync(rect_key'd center_bounds, webview_visible(active_tab.is_browser(),
  overlay_is_up(&self.overlay_snapshot())))`. `overlay_snapshot()` sits beside
  `text_input_blocked` (:10671) with its NOTE discipline ("a NEW overlay only needs a line
  here") + a pointer comment at the render tail (:19888).

### File manifest
| # | File | Change |
|---|---|---|
| 1 | `crates/marley_app/Cargo.toml` | + `wry = "0.56"` (the probe's exact pin — a dep edge only, zero version churn) |
| 2 | `crates/marley_app/src/browser.rs` | + `OverlayStates` (26 named bools) + `overlay_is_up` + `webview_visible` + `rect_key`/`rect_changed` + `MountPlan`/`mount_plan`; `#[cfg(test)]` units T1–T4 (the module keeps its #403 cov/MSI-100 floor) |
| 3 | `crates/marley_forge_client/src/lib.rs` | + `pub same_web_origin` beside `forge_web_base`; units T5–T8 |
| 4 | `crates/marley_app/src/webview_shim.rs` | **NEW, whole-file masked** — `BrowserWebview` per the architecture block; per-fn `mutants::skip` + module-level §0 documentation |
| 5 | `crates/marley_app/src/app.rs` | masked wiring: 3 RootView fields; the boot-site consumer-three line (:2424 neighborhood); `overlay_snapshot()` beside :10671; the render-tail pointer comment; ONE `sync_browser_webview` site after the tab-body dispatch; the drop at the `release_browser_views` close arm(s) (:1758/:5855/:8510 neighborhoods — exact site at implement); Browser body arm branches placeholder-vs-webview on the mount state |
| 6 | `scripts/gates.sh` | rust_cov `--ignore-filename-regex` + `marley_app/src/webview_shim\.rs`; ACCEPTED-UNTESTABLE comment line (the #404 precedent) |

**React-first: N/A RE-CONFIRMED** (flip protocol armed, not tripped): v1 adds NO net-new
Marley-drawn chrome — the placeholder is #403's unchanged, and the pane's new pixels are the
page's own (today: the forge sidecar's honest 401 JSON — no web UI exists at the origin yet,
the #404 probe's recorded carry). Any #406 error card/loading bar trips the flip.

### Regression Test Plan (per REQ)
| # | Test (home) | Proves |
|---|---|---|
| T1 | `overlay_flip_table_each_of_26_states` — all-false → false; each single-true → true (browser.rs) | REQ-003 unit; kills any dropped `\|\|` arm |
| T2 | `webview_visible_truth_table` — 4 arms exact (browser.rs) | REQ-003/004 decision seam |
| T3 | `rect_key_rounds` + `rect_changed_none_applies` / `_same_skips` / `_differs_applies` (browser.rs) | REQ-002 diff-before-apply |
| T4 | `mount_plan_some_attach_none_placeholder` (browser.rs) | REQ-001/006 decision |
| T5 | `same_web_origin_same_origin_paths_proceed` — origin itself + paths/queries (fc) | REQ-005 proceed arm |
| T6 | `same_web_origin_hostile_rejects` — non-loopback, https, port mismatch, subdomain suffix, userinfo `:pass@`, malformed, empty (fc) | REQ-005/D3 reject arms |
| T7 | `same_web_origin_default_port_elision` — mounted portless vs nav `:80` (fc) | REQ-005 + the #404 D-DEFAULT-PORT carry |
| T8 | `nav_inputs_never_carry_bearer` — sentinel-bearer config composed through `forge_web_base` → equality inputs sentinel-free (fc) | REQ-007 |
| T9 | `browser_boot_derivation_feeds_pane_headless` — TestAppContext boot w/ fixture `.mcp.json` → `forge_web_base` field == expected; None-config → None + placeholder arm chosen (headless_drive.rs) | REQ-001/006/008 decision halves |
| T10 | **HEADED driven sweep** — attach at rect (page pixels ≠ placeholder), re-sync on resize/dock/files-drag, hide+reappear per overlay CLASS (palette, context menu, find bar, flash), tab-away/back, off-origin attempt stays, restart-restore re-derivation | REQ-001/002/003/004/005/008 headed halves; **env-blocked protocol** (the #204/#205 precedent, #400 REQ-003 shape) if the headed lane is blocked — named carries, never silent |
| T11 | `cargo mutants --list -f` over the touched files; neighboring `#[mutants::skip]` bindings re-verified (the 5th-strike skip-detach rule) | Floors |
| T12 | The #403 codec suite green unchanged (bare `B`, byte-identical wire) | REQ-008 codec half |

### Risks / decisions (reversible-but-load-bearing)
1. **⌘-chords while the webview holds first responder — unproven** (the probe proved the IPC
   return trip, not chord passthrough). v1 never steals focus; driven check at P4; #406 owns a
   fix. The worst case is bounded: click any gpui chrome, chords return.
2. **Teardown rides #403's release path** — the drop happens where release runs (render-adjacent
   is possible); #406's detach-off-the-render-path hardening is the spec'd successor. Recorded,
   not hidden.
3. **Attach failure = silent placeholder + stored error, no retry** (probe precedent); a tab
   close-and-reopen naturally retries (the holder drops with the resident). #406's error card
   consumes `browser_attach_error`.
4. **REQ-001's "the real Forge page" is today the sidecar's 401 JSON** (no web UI at the origin
   yet — the #404 probe carry). The headed capture asserts page-pixels-at-rect ≠ placeholder,
   not Forge chrome.
5. **Boot-granularity derivation** (consumer three at the boot site): a mid-session `.mcp.json`
   edit retargets the pane only at relaunch — deliberately coherent with the MCP client + brain
   consumers (one config, one read, three consumers).
6. **`status_flash` included** — the ~2s blink while a flash shows is accepted (correctness over
   polish); #406 may flip WITH driven evidence.

**Presented in-transcript on the /goal night run (the standing confirm); flagged for morning
review alongside the phase gates.** Status → Phase 2 PASS.

## Phase 3 — Implement (2026-08-08, the /goal night run)
- **React-first: N/A** (per the P2 re-confirm — no net-new Marley-drawn chrome; recorded, not
  skipped).
- Built exactly the Phase-2 manifest, all six files:
  1. `marley_app/Cargo.toml` — `wry = "0.56"` (the probe's pin). **Lock churn: ONE line**
     (`+ "wry"` in marley's dep list; wry stays 0.56.0, zero version bumps).
  2. `browser.rs` — `OverlayStates` (26 named pub bools, RootView field names VERBATIM, no
     `Default` so a new field breaks the builder loudly) + `overlay_is_up` (the 26-arm `||`) +
     `webview_visible` + `rect_key`/`rect_changed` + `MountPlan`/`mount_plan`; module doc
     extended. gpui-free holds.
  3. `marley_forge_client/lib.rs` — `pub same_web_origin` riding the private `web_origin_of`
     (one parser; the #404 parsed-host loopback re-check inherited free); `forge_web_base`'s
     "ships no caller" doc line trued up.
  4. `webview_shim.rs` (NEW, whole-file masked §0) — `BrowserWebview { webview, applied, shown }`;
     `attach` (build_as_child + with_bounds + nav-handler + load_url), `sync`
     (bounds-first-then-visibility; hide invalidates the applied key), `wry_rect`
     (Logical→Logical). Nav closure captures ONLY the plain origin String (the weak-capture
     rule holds by construction, recorded in the module doc).
  5. `app.rs` — 3 RootView fields + ctor; the boot-site consumer-three line (beside the
     forge-client + brain resolutions); `overlay_snapshot()` beside `text_input_blocked` with
     the NOTE discipline; `sync_browser_webview` (project_count guard → try_active_tab →
     is_browser; mount once per resident life, error stored not retried) called at the ONE
     site right after `center_bounds`; `reap_browser_holder` called after ALL THREE
     `release_browser_views` sites (:4014 switch-return, :8564 tab close, :8743 project
     close); the Browser body arm branches mounted-bg vs #403-placeholder; the render-tail
     OVERLAY REGION pointer comment.
  6. `scripts/gates.sh` — cov exclude `marley_app/src/webview_shim\.rs` + the
     ACCEPTED-UNTESTABLE comment sentence (the #404 precedent shape).
- **Deviations from design (recorded):** (a) `BrowserWebview` DROPPED the planned `origin`
  field — the origin lives inside the nav closure and was consumed by `load_url`; a stored
  copy would be dead code under `-D warnings`. (b) `mod webview_shim` sits in lib.rs's
  alphabetical order (house order), not beside `mod browser`. (c) `sync` orders bounds BEFORE
  visibility so a restore shows at the FRESH rect in one pass — a strict upgrade over the
  probe's restore-then-heal; recorded in the shim doc.
- **Checks:** `cargo check --workspace` green (3.06s warm; pre-existing `block v0.1.6` note
  only). Preemptive `cargo clippy -p marley -p marley_forge_client --all-targets` → 2
  warnings, both fixed at source (a doc lazy-continuation rephrase in browser.rs; the shim's
  bounds-apply nested `if` folded into the short-circuit chain — semantics identical).
  `cargo fmt` clean. No tests written (P4's job, per the phase contract).

## Phase 3.5 — Inspect (2026-08-08)
- **Lenses (4 independent critics + the inspector's own checks):** A = the BLIND
  overlay-completeness adversary (re-derived the floating-overlay set from the render tail with
  no access to docs/planning — its 26 matched the frozen inventory 26/26, then its two flags
  became findings 1–2); B = lifecycle + wry-0.56 semantics read from the LOCAL registry source
  (the focus/IME family folded here); C = secrets/bearer walk + §20 provenance + gates-regex +
  skip hygiene; D = state integrity / release paths / cells / reuse. Inspector's own: the
  flash-expiry re-render trace and a live ERE test of the amended coverage regex. Forge
  knowledge-search skipped — MCP unreachable this session (§19; recorded at promotion).
- **Findings ledger:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | 1 | **med** | **Top-search results dropdown missing from the hide inventory** — draws on `!top_search_query.is_empty()` alone (:21099), overflows the top bar 12 rows deep into the content area (content_top = 30), fully reachable beside a Browser tab; the P2 sweep filed it as Class-D chrome with the box it hangs from. Found INDEPENDENTLY by critics A and B. | REAL | **FIXED** — 27th state `top_search_results` mirroring the draw gate exactly (query-non-empty, focus deliberately NOT part of it); `overlay_is_up` arm + builder line + spec amended 26→27 (Scope-In + REQ-003). |
  | 2 | **med** | **The references card's SEARCHING arm escapes the shim** — the same card surface draws while `open_references` is None (`references_request` out + host reports pending, :14823-14841); the snapshot captured only `is_some()`. | REAL (the unconditional-membership doctrine exists for exactly this) | **FIXED** — the `open_references` builder line mirrors the FULL compound gate. |
  | 3 | **med** | **`renaming_symbol` PERSISTENT over-hide → blank Browser pane** — the rename card's draw gate needs a live editor (`active_editor()?`, :14644-5); rename has NO lost-editor pump heal (hover/completion/signature do), survives tab switches, so rename→click-Browser hides the webview indefinitely with nothing drawn over it. Critic D; the INVERSE direction of 1–2. | REAL | **FIXED** — mirror the full gate: `renaming_symbol.is_some() && active_editor().is_some()` (behavioral alternative — a pump lost-editor heal — rejected as out-of-scope rename-semantics change). |
  | 4 | low | Latent 0-project panic in the references compound (`active_project().root` unguarded); unreachable today via a transitive invariant (webview ⇒ resident ⇒ ≥1 view ⇒ ≥1 project), but one future release path missing its reap converts a leak into a render panic. Critics B + D converged. | REAL (latent) | **FIXED** — `project_count() > 0` belt inside the compound (mirror-safe: the tail never runs at 0 projects — launcher early-return). |
  | 5 | low | Subframe navigations are origin-pinned too — wry-0.56's macOS delegate has no `isMainFrame` filter, so cross-origin/`about:blank` IFRAMES get cancelled. Today's page (the 401 JSON) has none. | REAL but CONTINGENT; wry 0.56 exposes no frame info | No change; **carried to #406** (revisit iff the Forge web UI grows iframes). |
  | 6 | info | `load_url`'s `map_err` is dead on macOS (wry panics on invalid NSURL, never Errs — unreachable from our origin domain: WHATWG loopback origins are all NSURL-parseable). | Correct-as-written | Keep for cross-platform shape (critic C's own recommendation). |
  | 7 | info | Failed attach falls to the #403 placeholder copy ("arrives with…") though config resolved; `browser_attach_error` is write-only this slice. | BY-DESIGN (P2 risk 3) — a copy change would trip the React-parity flip | Carried to #406 (the error card consumes the stored String; ledger makes the debt explicit). |
  | 8 | info | `webview_shim.rs` untracked — a commit missing the add breaks the build. | REAL (mechanical) | **FIXED** — `git add -N` (intent-to-add; explicit staging can no longer miss it). |
  | 9 | nit | `mount_plan(Option<String>)` clones the origin — once per mount attempt only (the gate short-circuits after). | Accepted | No change. |
  | 10 | low | Pure seams have ZERO units yet; every skip justification cites P4 test IDs still unwritten. Critics C + D. | PHASE-CORRECT (the #404 finding-3 precedent) | P4 lands T1–T12 before `/commit`; flagged so the cited IDs don't rot. |
- **Checked-clean (verified, not assumed):** full bearer walk — bearer dropped at the
  `endpoint.url()` seam, wry's error variants carry no payload, `browser_attach_error` has no
  sink, RootView derives no Debug (C); loopback inheritance — a non-loopback candidate refuses
  regardless of the mounted string (C); §20 — zero Warp/Zed in the whole diff, all precedents
  in-house (C); gates ERE exact — matches only the shim file (C + inspector's live test); skip
  hygiene — 3 shim fns + 3 app.rs wiring fns all skip+justified, ZERO skips on the pure seams
  (C); release-path completeness — all three release sites reaped, no fourth route, NO
  mid-session shell-restore path exists, quit drops on the main thread (D); single RootView
  ctor; headless boots derive None → Placeholder by construction (D); cell invariants — no
  persistent desync, worst case one-frame stale rect self-healed; macOS `set_bounds`/
  `set_visible` are INFALLIBLE in wry 0.56, `Drop` does `removeFromSuperview` (B, D); flash
  expiry re-renders via the 16ms pump's dirty→notify (inspector + B); initial `load_url`
  self-passes the nav pin; `window.open`/`target=_blank` with no handler = DENY-by-default (B,
  C); mount gate — background-restore never mounts, same-frame mount visible to the body arm
  (B); reuse — `rect_key`/`rect_changed` have no in-tree twin, the OverlayStates ↔
  `text_input_blocked` divergence is deliberate on both sides (D); no unwrap/expect on any
  config/URL path (B, C).
- **Validate-phase carries:** T1's flip table is 27 rows; the three COMPOUND mirrors need
  named units in the T9 headless drive family (rename-arm false without editor; references
  in-flight arm; top-search query arm — drivable headlessly, no webview needed); everything
  else per the P2 plan.
- **Forge capture candidates (MCP unreachable — for the P5 AAR):**
  `BF-claude-snapshot-mirrored-state-not-draw-gate-001` (one class, BOTH directions: two
  under-hides + one persistent over-hide) and
  `PR-claude-mirror-the-draw-gate-not-the-state-001` ("a visibility predicate derived from
  render state must mirror each DRAW GATE, not the bare state field; verify per-field against
  the render, and re-verify when either side changes").

## Phase 4 — Validate (2026-08-08)
- **Units written (T1–T9 + the P3.5 carries), all landed:**
  - browser.rs (5 new): `overlay_flip_table_each_of_27_states` (all-false DOWN + each single-true
    flips + the count assert — kills every `||`→`&&` swap), `webview_visible_truth_table` (4 arms),
    `rect_key_rounds_each_component` (rounds-not-truncates, per component),
    `rect_changed_arms_exact` (None-applies / same-skips / moved-applies),
    `mount_plan_some_attaches_none_placeholder`.
  - marley_forge_client (4 new): `same_web_origin_same_origin_proceeds` (initial-load self-pass +
    paths/queries), `same_web_origin_hostile_rejects` (9 shapes incl. the userinfo divergence, the
    subdomain suffix, a DIFFERENT loopback host — origin equality not loopback-ness),
    `same_web_origin_default_port_elided` (the #404 carry), `nav_inputs_never_carry_bearer`
    (sentinel composed through the whole derivation).
  - headless_drive.rs (3 new drives + 3 `#[cfg(test)]` hooks in app.rs, the `*_for_test` idiom):
    `browser_boot_derivation_feeds_pane_headless` (configured root → the origin, sentinel-free;
    unconfigured → None), `overlay_snapshot_mirrors_compound_gates_headless` (top-search query
    alone registers; rename-without-editor must NOT — the over-hide catch; references latch alone
    must NOT), `rename_with_live_editor_registers_headless` (the `&&`'s other half).
- **Runs (real output in transcript):** targeted `cargo nextest run -p marley -p
  marley_forge_client` → **1010 passed / 2 skipped** first try; workspace →
  **2091 passed / 5 skipped** (+13 over pre-#405); doctests 0-in-scope green; the #403 codec
  suite green unchanged (T12).
- **T10 HEADED DRIVEN SWEEP — RUN LIVE, pixel-verified** (screen unlocked at 00:35; AX_TRUSTED;
  the selftest harness; PNGs in the session scratchpad, paths in transcript):
  - Boot note: the first `open`-launched instance raced a TCC removable-volume prompt and sat
    windowless; the grant was clicked, and the harness ran on a direct-exec relaunch (clean
    stderr, exit 0 both runs).
  - REQ-001 ✓ `t10-3-webview.png`: Browser opened via the rail plus-menu (Forge/Agents/Details/
    Browser — the #403 shape) → the NATIVE WKWebView composites at exactly `center_bounds`
    (flush to Files-panel edge / top bar / status bar; `focus: browser`). The page is the live
    forge origin; **evidence correction for #406:** WebKit renders the 401 JSON response as a
    BLANK WHITE page (not visible JSON text) — the error card is even more warranted.
  - REQ-003 ✓ `t10-4-palette.png` + `t10-5-reappear.png`: ⌘⇧P over the webview → the palette
    fully unclipped, the webview HIDDEN (dark scene behind); Esc → the webview back at the rect.
    The palette is the driven class representative; per-class STATE flips are the 27-row unit.
  - REQ-004 ✓ `t10-6-tabaway.png` + `t10-7-tabback.png`: terminal tab active → full pane layout,
    zero webview pixels; back → webview restored.
  - REQ-002 ✓ `t10-8-resized.png`: window 1306×660 → 1000×560 → the webview tracked flush to
    the smaller rect (dock/files-drag arms ride the identical capture+diff path — recorded).
  - REQ-008 ✓ `t10-9-restored.png`: quit → relaunch (NEW process) → the Browser tab restored
    from the bare `B`, origin RE-derived, webview REMOUNTED on the first active render.
  - Cleanup ✓ `t10-10-closed.png`: rail ✕ closed the tab (section empty, webview dropped via
    the reap), original active tab restored, app quit clean — the user's workspace back to its
    pre-drive shape.
  - Honest lane splits: REQ-005's off-origin attempt is not drivable (no in-page nav affordance
    on a blank page; WebKit has no CDP) — owned by the T5–T7 units + the wry-source verification
    (nav handler pins ALL frames; `window.open` deny-by-default). REQ-006's headed no-config arm
    would need scrubbing the real config — owned by the headless drive's unconfigured boot.
- **T11 mutants --list (the 5th-strike re-check):** browser.rs 43 (4 pre-existing placeholder +
  39 new, incl. the 26 `||`→`&&` swaps — each named-killed by the flip table);
  **webview_shim.rs 0** (the whole-file mask holds); fc lib.rs 11 across the origin fns. No skip
  drift.
- **Gate:** first `--diff` run → **3 static reds** (rustfmt wrap in the flip table; clippy
  `type_complexity` on the flip array — fixed at source with a test-local `type Flip` alias;
  rustdoc private-intra-doc-link `[web_origin_of]` from a pub doc — de-linked to plain code
  text). Heavies were ALREADY green on that run (coverage 100% floor, mutation 42/42). Re-run →
  **GATE GREEN [diff], 15/15** — coverage 100% (84 978 regions counted), mutation **42 caught /
  0 missed → MSI 100**, receipt written.
- **Pre-existing exclusions:** none touched; the only workspace warning remains the
  `block v0.1.6` future-incompat note (mac windowing stack).

## Phase 5 — Complete (2026-08-08)
- **§21 docs:** CHANGELOG `[Unreleased] Added` entry (the pane, the 27-state shim, the
  mirror-the-draw-gate rule, the driven proof, the WebKit-401-blank carry);
  embedded-browser-model.md **train slice-3 ticked SHIPPED** with the #406 learnings block
  (401-blank truth, iframe pin, chord gap, attach-error debt, flash-blink criteria);
  scripts/selftest/README.md gains the TCC-prompt-eats-the-boot gotcha (found live tonight).
  **Parity sync: N/A confirmed at close** — no Marley-drawn pixel changed (the placeholder is
  #403's byte-identical; the pane's new pixels are the page's own), MARLEY-PARITY.md untouched,
  the flip protocol stays armed for #406's error card.
- **Knowledge — captured LOCALLY (§19: forge MCP unreachable in this session; submit from a
  wired session with these texts):**
  - **AAR (effectiveness: high).** What worked: the #402 probe paid VERBATIM (weak-capture rule,
    applied-key capture+diff, hide-invalidates-key, no-scale-math — every mechanic transferred
    without rediscovery); the P2 frozen-inventory + P3.5 BLIND-adversary pairing is the
    completeness discipline (the adversary's independent re-derivation matched 26/26, then found
    what the freeze missed); the headed sweep produced pixel proof units cannot (the palette
    hide/reappear captures are the shim's existence proof). What bit: THREE snapshot-mirror bugs
    in one fn — the builder mirrored bare STATE fields while draw sites gate on compound
    conditions (two under-hides + one persistent over-hide); statics went red on a full gate
    cycle because fmt/clippy weren't re-run after the TEST-writing round (run them after EVERY
    edit round, tests included); the `open`-launched ad-hoc bundle stalled windowless behind a
    TCC removable-volume prompt (now a README gotcha — grant, then direct-exec).
  - **BF candidate:** `BF-claude-snapshot-mirrored-state-not-draw-gate-001` (high) — one class,
    both directions, three instances in one builder.
  - **PR candidate:** `PR-claude-mirror-the-draw-gate-not-the-state-001` — "a visibility
    predicate derived from render state must mirror each DRAW GATE (its full compound
    condition), never the bare state field; verify per-field against the render tail, and
    re-verify whenever either side changes."
  - **Evidence corrections recorded:** WebKit renders a 401 `application/json` response as a
    BLANK WHITE page (the design's "renders the 401 JSON" assumption was wrong in the
    interesting direction — #406's error card is MORE necessary, not less); wry 0.56 macOS has
    no `isMainFrame` filter (subframes inherit the pin — fine today, a real constraint if the
    Forge UI grows iframes).
- **Ticket:** local doc → `tickets/closed/` (status: closed). Forge `ticket-close` + AAR
  submission DEFERRED — MCP unreachable (recorded at promotion and here).
- **Follow-ups for #406 (recorded, not ticketed here):** the error card consumes
  `browser_attach_error` + owns the 401-blank honesty; detach-off-the-render-path teardown
  hardening; the chord-while-webview-focused gap; the iframe/isMainFrame constraint; the
  flash-blink flip criteria.
- **Archive:** pair moved to `docs/planning/pipeline/completed/`.
