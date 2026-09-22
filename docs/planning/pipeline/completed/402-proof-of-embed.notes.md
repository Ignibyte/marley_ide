# Scratch proof-of-embed — a wry child WKWebView over a gpui window — Notes

- **Forge ticket:** #402 a17801ff-b56e-4a7d-ace4-b3252c49147e
- **AAR:** 120433f1-f4b4-42d9-b0c9-58a00a255fe9 (opened 2026-08-06 at /work promote)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-402-proof-of-embed.md
- **Pipeline spec:** 402-proof-of-embed.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** M29 sprint selection (forge sprint #40, id 6dff19c6-fb61-472b-89c3-98d3215a489b) —
  the #389 train's slice 1, handed to /spec by embedded-browser-model.md Q5 ("Scratch proof-of-embed
  … the go/no-go for (A) vs a pivot to (B); env-gated; the one place that adds wry"). Executes the
  M26 #389 decision doc; opens nothing else — findings are the deliverable.
- **Classification / tier:** spike, S/M. Env-gated HEADED probe (operator-driven, live compositor +
  WebKit + first-responder state) — §0 ACCEPTED-UNTESTABLE with an explicit documented gate exclude;
  §7's uncoverable-path arm applies to verification (recorded observations, not unit tests). The
  probe IS `.rs` under `crates/`, so the §15 receipt + §21 CHANGELOG commit hooks both fire at
  /commit.
- **Forge recall (§18.3):** `knowledge-search` "embedded browser webview wry WKWebView pane
  substrate" → 2 architecture_decisions + 6 prevention_rules (ids only from the search API; top AD
  cf751352-9cbe-49dd-b2d8-508b373a5393). The governing record is
  `AD-claude-embedded-browser-substrate-001`, doc-local at embedded-browser-model.md:158 (the
  #388/#389 AD-tool-errs fallback — 389 notes :125). Nothing browser-specific beyond the #389
  record; the adjacent prevention-rule hits are registry/pane-lifecycle rules from the #394-#400
  train, not webview-specific. Wry published docs re-fetched live (docs.rs, 2026-08-06) — recorded
  in the spec's Prior art leg 2 (version drift 0.55.1 → 0.56.0 found).
- **Discovery (verified against the tree 2026-08-06):**
  - **gpui 0.2.2** (crates.io registry dep — `Cargo.lock:2170`; source at
    `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2`; reading it is §20
    ADOPTION). ALL the M26 design-doc claims re-verified, zero drift:
    `impl rwh::HasWindowHandle for MacWindow` → `RawWindowHandle::AppKit(AppKitWindowHandle::
    new(native_view.cast()))` (platform/mac/window.rs:1548, :1552-1553); `impl HasWindowHandle for
    Window` delegates to `self.platform_window.window_handle()` (window.rs:4845-4847); the single
    self-composited `GPUIView` NSView (platform/mac/window.rs:111); key routing
    `native_window.makeFirstResponder_(native_view)` (platform/mac/window.rs:776); the pivot door
    `paint_surface(bounds, CVPixelBuffer)` (window.rs:3181) + `surface()` element
    (elements/surface.rs:32).
  - **Lockfile:** `wry` ABSENT (grep exit 1 — "the one place wry enters the tree" holds);
    `core-video` 0.4.3 present (:1163); `raw-window-handle` 0.6.2 the SOLE rwh version (:4639, count
    1) — wry 0.56.0 wants `^0.6`, so gpui's handle type-checks straight into `build_as_child`;
    `objc2` 0.6.4 + `block2` 0.6.2 already in-tree (wry's macOS FFI base unifies);
    `objc2-web-kit` would be net-new (^0.3.2, Zlib/Apache/MIT).
  - **wry 0.56.0** (docs.rs live fetch; released 2026-07-30; Apache-2.0/MIT): `build_as_child` +
    `with_bounds`; `WebView::set_bounds` ("only effective if … created as a child"), `set_visible`,
    `focus()`, `focus_parent()` — the checklist's rect + hide-shim + focus-handoff surfaces are all
    first-party API. Design doc pinned 0.55.1 — drifted; P2 re-pins.
  - **Gates (scripts/gates.sh):** coverage exclude mechanism = `--ignore-filename-regex` (:222)
    fed by the documented ACCEPTED-UNTESTABLE reason block (:179-220); precedent entries already
    cover `marley_app/src/bin/`, `ui_components/src/bin/`, `marley_lsp/src/bin/`,
    `marley_visual_harness/src/bin/`. Mutation = whole-workspace ZERO file exclusions (:226-227);
    the in-source mechanism is `#[mutants::skip]` + justification (`mutants = "0.0.3"` attribute
    crate — marley_visual_harness/Cargo.toml:23). gate:2 clippy `--workspace --all-targets`
    (:368) lints every workspace bin each run; gate:3 `--no-tests=warn` (:83) makes a test-less
    probe crate a visible warning; gate:8 deny (:371) vets the whole new lock tree against
    deny.toml's permissive-only allowlist (:39-51); gate:14 brand-scrub (:168) bans warp/zed words
    in probe source.
  - **Probe-home evidence:** existing bins are fixture bins inside the crate they exercise
    (marley_app/src/bin/marley.rs, marley_lsp/src/bin/fake_ls.rs,
    ui_components/src/bin/marley_widgets_gallery.rs); the headed OS-bound precedent is the DEDICATED
    crate marley_visual_harness (own bin, documented excludes, the "gpui `Application::run()` owns
    the main thread" subprocess note). A probe bin inside marley_app would put wry into marley_app
    `[dependencies]` → linked into the `marley` product binary unless optional-dep +
    `required-features`, which in turn makes `--all-targets` skip (uncompiled by every static gate —
    invisible rot). → recommendation: dedicated bin-only crate.
  - **Prior-art leg 1 (maps):** no embedded-webview behavior for either reference.
    docs/warp_architecture/ browser/web hits are Warp's cloud/web crates (mcp.md, websocket.md,
    serve-wasm.md, warp_web_event_bus.md). docs/zed_architecture/ mentions are Marley's own forward
    notes only (01-gpui-ui-framework.md:807 PaintSurface "zero-copy video door";
    07-workspace-panes-palette.md:19/:279/:554 intake triggers).
  - **Verdict/AD landing zone:** embedded-browser-model.md Q1 + the AD block (:158); the
    #388/#389 precedent is doc-local when the forge AD tool errs. Probe evidence home = THESE notes
    (Phase 4) — docs/warp_architecture/observed/ is scoped by its README to reference-app captures,
    so it is NOT the probe-screenshot home.
  - **Line-drift note (honest record):** design doc cites `tabs.rs:82` for `RailSection::Browser`;
    after M27/M28 the sites are tabs.rs:60/:69/:80/:131 — content intact, lines drifted. All gpui
    citations: no drift.
- **Decisions:** D1 wry-as-child is the substrate under test (objc2-web-kit the in-family fallback;
  CEF-OSR the named pivot, no CEF work here) · D2 env-gated scratch, never in the product
  binary/render path (linker-level, `cargo tree -i wry` provable) · D3 the Q1 checklist IS the
  acceptance — all THREE points (rect, overlay+hide-shim, focus/IME both directions) · D4 output =
  recorded GO/NO-GO into embedded-browser-model.md + AD update (doc-local fallback per #388/#389) ·
  D5 the gate exclude is explicit + documented per §0 (named coverage entry + reason; per-fn
  mutants::skip justifications; no silent regex ride). **Open forks for P2:** D-OPEN-PROBE-HOME
  (recommend dedicated bin-only crate, grounding above) · D-OPEN-OVERLAY-SIM (probe-local
  palette-style gpui overlay — must actually overlap the webview rect) · D-OPEN-HIDE-SHIM
  (recommend `set_visible(false)` first; bounds-offscreen and remove-from-superview the
  alternatives; record WebKit side effects as slice-3 input).

## Phase 2 — Design

### Architecture / approach (D-OPENs settled, each with evidence)
- **D-OPEN-PROBE-HOME → SETTLED: dedicated bin-only crate `crates/marley_webview_probe/`.** The
  workspace is `members = ["crates/*"]` (root Cargo.toml:3) — the crate auto-joins with NO root
  manifest edit. Grounding re-confirmed: a marley_app-resident bin would link wry into the product
  binary unless optional-dep + `required-features`, and required-features makes gate:2
  `--all-targets` (gates.sh:368) skip it — invisible rot. The dedicated crate is the
  marley_visual_harness precedent (headed, OS-bound, documented excludes) and makes D2's proof
  `cargo tree -i wry` → exactly one reverse path.
- **Probe shape.** `main()`: env-gate check FIRST (`MARLEY_WEBVIEW_PROBE=1`; refusal =
  message + exit code 2 BEFORE any gpui call — a headless-safe negative smoke), then
  `gpui::Application::new().run(...)` (owns the main thread — the harness lesson,
  marley_visual_harness/Cargo.toml:26-27) opening one ~1100×700 window with a `ProbeView` root.
- **The ONE pure fn (Floors: purity found, not invented):** `probe_enabled(var: Option<&str>) ->
  bool` (`Some("1") → true`, else false). Real `#[cfg(test)]` unit tests; **NO mutants::skip** —
  cargo-mutants mutates it and the tests kill honestly. Everything else carries
  `#[cfg_attr(test, mutants::skip)] // justification: …` (the exact os_shim.rs:15 shape; `mutants =
  "0.0.3"` as dev-dependency per marley_visual_harness/Cargo.toml:23).
- **Attach.** First render: `window.window_handle()` (gpui `Window: HasWindowHandle`,
  window.rs:4845 → `RawWindowHandle::AppKit`, platform/mac/window.rs:1548) →
  `WebViewBuilder::new().with_url(DATA_PAGE).with_bounds(rect).with_ipc_handler(…)
  .build_as_child(&window)`; the `WebView` lives in `ProbeView` (dropped at quit — teardown
  observed and recorded as a #406 input). All wry calls happen in render/key handlers = gpui main
  thread (WKWebView is main-thread-only — satisfied structurally).
- **Pane rect + tracking (REQ-001).** The designated rect = right ~55% of the content area,
  inset 12px, computed from window bounds each render; gpui draws a 2px accent FRAME + caption
  around it (the frame must stay visible around the webview's edges — the live alignment cue).
  Sync = capture+diff: render recomputes → differs from last-applied → `set_bounds`. Coordinate
  mapping recorded as the slice-3 input: gpui `Pixels` are LOGICAL points; wry `Rect` takes
  `dpi::LogicalPosition/LogicalSize` — direct numeric copy, NO scale_factor math on macOS.
- **D-OPEN-OVERLAY-SIM → SETTLED: probe-local palette-style gpui overlay.** An absolute-positioned
  centered card (~46% w × ~55% h, opaque bg, border, `occlude`) — centered over the window while
  the webview holds the right 55%, so the rects OVERLAP BY CONSTRUCTION (the collision is real).
  Plain gpui divs; no ui_components dep (the test is compositing order, not pixel fidelity).
- **D-OPEN-HIDE-SHIM → SETTLED: `set_visible(false)` primary; offscreen-bounds the in-session
  alternate.** Overlay up → hide via the active mechanism; ⌘M cycles mechanism
  (SetVisible ↔ ZeroBounds) so BOTH candidates are observed in one run; restore on dismiss.
  Remove-from-superview is DROPPED (wry `reparent` is gtk-only — the #406 sweep; superview surgery
  via objc2 bypasses wry's API and is escalation-only if both mechanisms fail, recorded not built).
  WebKit side effects (flicker on restore, process suspension) are observations → slice-3 inputs.
- **Focus/IME exercise (REQ-003).** `DATA_PAGE` is a `data:` URL (no network): large `<input
  autofocus>` + a "Return focus to host" button posting `window.ipc.postMessage('focus_host')` →
  `with_ipc_handler` → `focus_parent()` + a `last_ipc` display. The gpui side renders a "last key:"
  line from its key listener. Both directions: click/type + one IME composition (macOS input
  source, e.g. Hiragana) inside the webview → IPC button → type gpui-side (routing provably
  returned) → ⌘F (`webview.focus()`) → type inside again.
- **Keybindings (gpui-side, pressed while gpui is focused):** ⌘O toggle overlay (drives the shim) ·
  ⌘M cycle hide mechanism · ⌘F hand focus INTO the webview (`focus()`); return trips are the IPC
  button / clicks (chords pressed while the webview is focused go to the page — the protocol
  accounts for it).
- **§20 confirm:** Reference stays **N/A — Marley-specific** (re-confirmed: no reference-app
  analog; design consulted wry's published docs + gpui 0.2.2 registry source — ADOPTION, outside
  the wall). No Warp/Zed source consulted at design. gate:14's brand-scrub (gates.sh:168) binds the
  probe's source text; keep it clean of the brand words.

### File manifest
| File | Change |
|---|---|
| `crates/marley_webview_probe/Cargo.toml` | NEW — `publish = false`, `edition/license.workspace`; deps: `gpui = "0.2.2"`, `wry = "0.56"` (exact pin at add; feature audit below); dev-dep `mutants = "0.0.3"` |
| `crates/marley_webview_probe/src/main.rs` | NEW (~250–320 lines) — env gate + `probe_enabled` (+its `#[cfg(test)]` tests), `ProbeView` (webview handle, overlay flag, hide mechanism, last-key/last-ipc, last-applied rect), attach-on-first-render, capture+diff `set_bounds` sync, overlay card, hide shim, IPC handler, `DATA_PAGE` const, keybindings |
| `scripts/gates.sh` | MODIFY — ACCEPTED-UNTESTABLE block (:179-220) gains the probe entry (wording below); `--ignore-filename-regex` (:222) gains `\|marley_webview_probe/src/` |
| `Cargo.lock` | regenerated by the dep add — verify: rwh stays SOLE 0.6.2; objc2 0.6.4/block2 0.6.2 unify; `objc2-web-kit` 0.3.x net-new; record the wry transitive set |
| *(P5 only)* `docs/marley_architecture/embedded-browser-model.md` + `CHANGELOG.md` | verdict + AD update; the changelog entry (the probe IS `.rs` — both commit hooks fire, §15/§21) |

NO `marley_app/src/**` edit; NO root Cargo.toml edit (members glob). React-first: N/A confirmed —
single-half manifest.

**Exclude wording (draft for the gates.sh block):** "marley_webview_probe (M29 #402) adds the SAME
shim exclude: src/ (the env-gated HEADED wry/WKWebView child-embed probe — live compositor + WebKit
child process + first-responder state; operator-driven per the pipeline-notes protocol; every fn
mutants::skip-justified EXCEPT the pure env-gate `probe_enabled`, which keeps real unit tests and
stays in the mutation bar). Findings, not code, are the deliverable (embedded-browser-model.md Q1)."

### Regression Test Plan (one row per REQ + smokes)
| REQ | Proof | Lane |
|---|---|---|
| REQ-001 | Headed protocol step 1: webview fills the drawn frame at rest (frame visible on all edges); live window resize tracks; ≥2 screenshots → notes Phase 4 | operator headed (§7 uncoverable: live compositor) |
| REQ-002 | Step 2: ⌘O → webview hides (set_visible), overlay fully visible + unclipped; dismiss → restore; REPEAT under ⌘M ZeroBounds mode; flicker/latency observations; ≥2 screenshots | operator headed |
| REQ-003 | Step 3: ASCII + one IME composition inside the webview input; IPC "return focus" → gpui last-key updates on next keypress; ⌘F back in; both directions twice; observations recorded | operator headed |
| REQ-004 | P5 diff review: Q1 verdict text + AD block updated; forge `architecture-decision-record` attempted, outcome noted (doc-local fallback per #388/#389) | doc review |
| REQ-005 | `scripts/gates.sh --diff` exit 0 at P4; the exclude entry names probe + reason; `probe_enabled` tests RUN under nextest (counts recorded); `cargo mutants --list -f crates/marley_webview_probe/src/main.rs` shows only skip-justified fns + the honestly-mutated pure fn | gate run |
| REQ-006 | `git diff --stat` paths ⊆ {probe crate, gates.sh, Cargo.lock, docs}; `cargo tree -i wry` output recorded — single reverse path (probe only) | review + command output |
| smoke-a | Ungated run (`cargo run -p marley_webview_probe`, no env) refuses: message + exit ≠ 0, no window — runnable headlessly | negative smoke |
| smoke-b | `probe_enabled`: `None`/`Some("0")`/`Some("")` → false, `Some("1")` → true | unit (in the mutation bar) |
| smoke-c | trybuild: N/A — the probe exposes no cross-crate type contract (recorded, not skipped silently) | — |

Genuinely uncoverable (named per §7): all of main.rs EXCEPT `probe_enabled` — live compositor +
WebKit child + first-responder state; verified by the operator-driven protocol above, evidence in
notes Phase 4.

### Risks / decisions (recorded)
- **R1 — gate:8 over the full multi-target graph.** deny.toml has NO `[graph] targets` scoping, so
  wry's Linux (gtk-family wrappers) + Windows (webview2-com) transitives are license-vetted too —
  MIT-class expected; ANY red is fixed visibly (feature-prune or a justified deny.toml change),
  never ignored.
- **R2 — feature set unverifiable offline** (crates.io unreachable from this session; docs.rs
  API-surface facts stand from the plan-phase fetch). Baseline = default features; at `cargo add`,
  run `cargo tree -e features -p wry` and RECORD the resolved set; prune anything heavy/non-mac.
- **R3 — first-frame bounds:** `with_bounds` at build avoids wry's 200×200 default (the #405 sweep
  hazard) by construction.
- **R4 — IME × hide/show interaction:** a composition interrupted by ⌘O may misbehave — that is a
  FINDING to record (slice-3 input), not a probe defect.
- **R5 — event-loop premise:** `build_as_child` requires no separate wry/tao event loop on macOS
  (gpui runs NSApplication). If wry demands one anyway, that is an API-blocker finding → the
  `objc2-web-kit` in-family fallback per D1 — recorded, not improvised.
- **R6 — teardown:** WebView drop at quit observed + noted (feeds #406's teardown fork).
- **R7 — chord capture:** while the webview is focused, gpui chords go to the page; the protocol
  presses gpui-side chords only while gpui is focused (the IPC button is the webview-side exit).

## Phase 3 — Implement
- **React-first: N/A** — recorded per the spec (scratch probe, no product UI; nothing for
  marley-web to prototype).
- **Built (to the manifest, 2026-08-06):**
  - `crates/marley_webview_probe/Cargo.toml` — bin-only crate, `publish = false`, deps gpui 0.2.2 +
    wry 0.56 (resolved 0.56.0), dev-dep `mutants 0.0.3`; the D2 header comment names the one-entry
    rule + the `cargo tree -i wry` proof.
  - `crates/marley_webview_probe/src/main.rs` — the probe per design: `probe_enabled` pure env gate
    (+5 unit tests, NO mutants::skip — stays in the mutation bar); `main` refuses ungated with exit
    code 2 BEFORE any gpui call; `ProbeView` with attach-on-first-render
    (`build_as_child(window)` — the `&Window: HasWindowHandle` impl, gpui window.rs:4845),
    capture+diff `set_bounds` sync keyed on rounded logical px and guarded by `!overlay_up`;
    ⌘O overlay toggle driving the hide shim, ⌘M live mechanism cycle
    (set_visible ↔ zero-bounds), ⌘F `focus()` into the webview, Esc dismiss; the IPC
    `focus_parent()` return trip via `with_ipc_handler` (main-thread `Rc<Shared>`); one-shot
    `window.focus(&handle)` seeding (never re-stolen — ⌘F must keep its handoff); every wry
    `Result` displayed in the status line, zero unwrap/expect (§14); all headed fns carry
    `#[cfg_attr(test, mutants::skip)] // justification: …` (the os_shim.rs:15 shape).
  - `scripts/gates.sh` — the marley_webview_probe ACCEPTED-UNTESTABLE entry appended to the
    documented block (:219-225 post-edit) + `|marley_webview_probe/src/` added to the coverage
    `--ignore-filename-regex`; shellcheck clean under the gate's own invocation
    (`-S info -e SC1091`).
  - `Cargo.lock` — regenerated: **rwh stays SOLE 0.6.2; `cargo tree -i wry` = exactly one reverse
    path (the probe — D2 proven at the resolver level)**; wry 0.56.0 default features; net-new
    tree is wry's full multi-target graph (gtk-family/webview2-com/jni-android wrappers —
    lock-entries only on macOS; gate:8 vets their licenses at validate per R1).
- **Deviations from design (with reason):**
  1. `with_html(EXERCISE_PAGE)` instead of a `data:` URL — same no-network intent, kills the
     data:-URL percent-encoding hazard entirely (recorded at design as the likely shape).
  2. `use gpui::prelude::*` added inside `run_probe` — `cx.new` lives on the `AppContext` trait
     (compile-time find; the selftest fixture's exact idiom).
  3. Geometry stayed inline in `render` (no named helper fns) — per the don't-invent-purity
     stance; the only named pure fn is `probe_enabled`.
  4. *(recorded at P3.5, ledger F2)* The design's "the WebView lives in ProbeView" became a
     `Rc<Shared>` split (webview/status/last_ipc) because the IPC closure needs the WebView it is
     stored inside — and the closure captures the Rc as **Weak** (`Rc::downgrade`), since wry
     stores the handler inside the WebView and a strong capture would cycle (WebView Drop
     unreachable — the F2 finding).
- **Compile status:** `cargo check --workspace` green; `cargo clippy -p marley_webview_probe
  --all-targets -- -D warnings` green; `cargo fmt` applied. (The `block v0.1.6` future-incompat
  note is pre-existing gpui-tree noise, unrelated.)

## Phase 3.5 — Inspect
**Critics run (3, parallel, each instructed to verify concretely):** correctness (wry/gpui registry-
source evidence + live command runs) · security/provenance/supply-chain (live cargo deny/audit/
machete/mutants-list runs, brand-scrub, §20) · design-fidelity/state-integrity/simplification
(bullet-by-bullet Phase-2 walk + geometry math + notes-truthfulness).

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| F1 | HIGH | gate:8 `cargo deny` RED — wry's Linux-only gtk3 tree brings 10 error-level `unmaintained` advisories (RUSTSEC-2024-0412..0420 gtk-rs family + RUSTSEC-2024-0370 proc-macro-error via gtk3-macros); blocks FULL/--diff and the §15 receipt | REAL (exit 1 observed) | Per-id justified ignores in deny.toml per the TICKET-007/022 precedent, naming the Linux-only/compiled-out/no-fix facts + the glib RUSTSEC-2024-0429 unsound audit-warning for §0 visibility. The `[graph] targets` alternative was REJECTED — it would unscope license vetting for all future non-mac transitives. Re-run: `advisories ok, bans ok, licenses ok, sources ok` |
| F2 | MED | Rc CYCLE: the IPC closure captured a strong `Rc<Shared>`; wry stores the closure INSIDE the WebView (wry-0.56.0 wkwebview/mod.rs:144/:553/:1423) and `Shared` owns the WebView → Drop unreachable; the R6 teardown observation would have been process-exit reclamation, a lie fed to #406 | REAL | `Rc::downgrade` + `upgrade()` guard at the handler top; recorded as Phase-3 deviation 4 below |
| F3 | MED | ⌘F with the overlay up focuses a hidden/zero-bounds webview — first responder leaves GPUIView, plain keys (incl. Esc dismiss) stop routing, status says "focus() → webview" with nothing visible → possible FALSE focus-handoff failure in the verdict | REAL (wry `focus()` = unconditional makeFirstResponder, mod.rs:1057; gpui re-asserts only at window open, :776) | The ⌘F arm refuses while `overlay_up` with an explanatory status line |
| F4 | MED | REQ-003 spec text demands "one IME composition … in the gpui surface afterward", but the probe's gpui side is display-only (no input handler — gpui NSTextInputClient resolves None, window.rs:1697); the design protocol had narrowed to "type gpui-side" silently | REAL (spec/protocol divergence) | RECORDED NARROWING, not silent: the return-half evidence is RAW key routing (`last_key`) — a gpui-side IME composition is structurally out of a display-only probe, and product-grade IME-after-return is slice-3's assertion in the real shell (which owns the IME machinery). Annotated in the spec's REQ-003 verify cell pointing here |
| F5 | LOW | cargo-audit allowed-warning count 13→24 incl. one new `unsound` (glib 0.18.5, RUSTSEC-2024-0429); gate:7 stays green | REAL, no gate impact | Named inside the F1 deny.toml comment so it is §0-visible |
| F6 | LOW | Dead `PartialEq` derive on `HideMode` (no `==` anywhere) | REAL | Derive reduced to `Clone, Copy` |
| F7 | LOW | "rounded logical px" doc vs `as i32` truncation | REAL | Code now `.round()`s (both the attach seed and the sync key) — code matches the record |
| F8 | LOW | `let _ = cx.open_window(…)` — a failed open ran headless-and-silent, against the probe's own errors-displayed bar | REAL | `if let Err` + eprintln |
| F9 | LOW | `status.borrow_mut()` held across wry FFI in `apply_restore` + the ⌘F arm (hygiene; not reachable today — WKScriptMessage delivery is async) | REAL (hygiene) | Outcome-first everywhere, matching `apply_hide` |
| F10 | LOW | Chord overmatch: ⌘⇧O/⌘⌥O/⌘⌃O also trigger (gpui keeps base key + surviving modifiers) | ACCEPTED — matches the marley_app looseness precedent (app.rs:2944), harmless under the operator protocol | None (noted) |
| F11 | LOW | The pane frame overflows the window right edge at vw ≲ 167px (frame_x unclamped while widths floor) | ACCEPTED — cosmetic at absurd sizes for a scratch probe; casts saturate, nothing negative reaches wry | None (noted) |
| F12 | LOW | Spec REQ-005/Floors text "the probe crate has no tests / no-tests warning visible" is unsatisfiable — the design added `probe_enabled` tests after that P1 text | REAL (stale P1 text) | Supersession recorded: the operative row is the notes test-plan REQ-005 (tests RUN + counted; no no-tests warning expected for THIS crate). Spec annotated |

**Clean sweeps recorded (verified, not assumed):** RefCell discipline end-to-end (no synchronous
re-entry: WKScriptMessage delivery is cross-process async; gpui registers no responder overrides) ·
attach race impossible (`build_as_child` fully synchronous, never pumps the runloop) ·
`window.focus` during render legal (post-paint Focus pass, window.rs:1939) · keystroke literals
("escape", platform ⌘) correct · the sync-key state machine airtight across all
(overlay, mechanism, resize) interleavings incl. ⌘M-while-up · wry `set_bounds` Logical→Logical
identity (no scale math needed — mod.rs:1031) · mutation surface = exactly 3 mutants, all
`probe_enabled`, all killable (skip census: 7/7 headed fns justified, 0 leaks) · lock additive
(zero removed/moved pre-existing deps; the one `-version` line is a diff artifact — redox_users
0.4.6 kept AND 0.5.2 added) · licenses/bans/sources all ok over the ~102 new crates · brand-scrub
clean · machete clean (the mutants dev-dep pattern matches the harness precedent) · §20 provenance
clean (published permissive API surfaces only) · D2 proven (`cargo tree -i wry` → one path) ·
gates.sh exclude wording verbatim-accurate against the code.

**Post-fix verification:** `cargo clippy -p marley_webview_probe --all-targets -- -D warnings`
green · nextest 5/5 · `cargo deny check` → advisories ok, bans ok, licenses ok, sources ok ·
`cargo mutants --list -f …/main.rs` → the same 3 killable mutants (the fixes added no mutation
surface) · `cargo fmt` applied.

## Phase 4 — Validate
- **Tests RUN (2026-08-06):** `cargo nextest run --workspace --no-tests=warn` → **2050 tests run:
  2050 passed, 5 skipped** (the skips are the pre-existing `#[ignore]` headed suites). The probe's
  5 `probe_enabled` units are in the run (verified individually at inspect: 5/5). Doctests:
  `cargo test --workspace --doc` → 0 failures.
- **Negative smoke (REQ-005-adjacent / smoke-a):** `cargo run -q -p marley_webview_probe` with the
  env UNSET → stderr `marley_webview_probe: headed scratch probe (M29 #402) — refusing to run
  without MARLEY_WEBVIEW_PROBE=1`, **exit=2**, no window (run twice — inspect + validate).
- **Assisted headed drive: attempted, then deliberately ABORTED (16:41).** The env-gated probe
  launched headed and its titled window appeared ("Marley WebView Probe (M29 #402)" — verified in
  a screen capture before abort), partially evidencing REQ-001's attach path (window creation
  only; the window was occluded). The capture showed the desktop in ACTIVE OPERATOR USE (a live
  terminal session frontmost, mid-keystroke) — sending synthetic ⌘-chords would have hijacked the
  keyboard mid-work, so the drive was stopped, the probe killed, and the capture DELETED (it
  contained unrelated private screen content; nothing committed). This is the recorded
  env-blocked/deferred arm (the #204/#227 precedent): **the 3-point checklist protocol remains the
  operator run** (below).
- **The operator protocol (REQ-001/002/003 — run when the desktop is free):**
  1. `MARLEY_WEBVIEW_PROBE=1 cargo run -p marley_webview_probe`
  2. REQ-001: confirm the webview fills the blue frame (border visible on all edges); resize the
     window and watch it track. Screenshot.
  3. REQ-002: ⌘O — overlay card must be fully readable, webview hidden; ⌘O again — webview
     restores into the frame. ⌘M then repeat under the zero-bounds mechanism. Note any
     flicker/latency on restore. Screenshot the overlay-up state.
  4. REQ-003: click into the page input; type ASCII + one IME composition (e.g. Hiragana);
     click "Return focus to host (IPC)"; type — the "gpui last key" line must update
     (raw-key routing = the return evidence per the F4 narrowing); ⌘F to hand focus back in;
     type inside again. Note both directions.
  5. Drop screenshots anywhere convenient; observations land here (Phase 4) and feed the P5
     verdict into embedded-browser-model.md Q1.
- **Gate (2026-08-06):** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15 passed / 0 failed,
  exit 0** — gates 1/2/3/7/8/9/10/11/12/13/14 static + gate:4 coverage ≥100% (the probe's
  documented exclude in force), gate:5 mutation MSI ≥100% (the diff's 3 `probe_enabled` mutants
  all killed), gate:6 miri, gate:15 visual/AX. The §15 receipt is written
  (`.git/ignibyte-gate-receipt`).
- **Pre-existing notes:** the `block v0.1.6` future-incompat warning (gpui tree) predates this
  ticket; the 5 nextest skips are the standing `#[ignore]` headed suites. Neither is in scope.
- **Operator run (chad, 2026-08-06, confirmed at /pipeline:complete):** ALL THREE checklist
  points hold — (1) the webview fills and tracks the pane frame through resize; (2) ⌘O shows the
  palette-style overlay fully visible with the webview hidden and restores it cleanly on dismiss,
  under BOTH mechanisms (⌘M: `set_visible` and zero-bounds); (3) keyboard + IME hand off cleanly
  in both directions (composition inside the webview; raw-key routing returns to gpui per the F4
  narrowing). **Checklist verdict input: GO — wry-as-child stands.**
- **PHASE 4: PASS** — tests 2050/2050 + smokes + GATE GREEN [diff] + the operator checklist
  above.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` entry (the probe + GO verdict + the deny/Rc-cycle inspect
  story); embedded-browser-model.md — the **Q1 → VERDICT: GO** block (with the two slice-3
  inputs: Logical→Logical no-scale-math + the weak-capture callback rule), train slice-1 ticked
  ✅ DONE, the AD block's de-risk-complete note. **Parity sync: N/A** — no UI ticket; marley-web
  untouched (recorded per the close checklist).
- **Knowledge (§19):** `aar-submit` → completed, effectiveness 4, 3 novel findings queued
  (distillation/confidence-drift/pattern-emergence). Materialized:
  `BF-claude-wry-ipc-closure-strong-rc-cycle-001`,
  `PR-claude-callback-stored-inside-owned-resource-captures-weak-001`,
  `AD-claude-embedded-browser-substrate-001` — **the AD tool SUCCEEDED this run** (it erred at
  #388/#389; the doc-local record now has a forge twin, id 2de8379f).
- **Lessons:** (1) the plan-phase prior-art RE-verify caught wry 0.55.1→0.56.0 drift — and the
  drifted version shipped the first-party `focus()`/`focus_parent()` the checklist needed; verify
  published-material claims at plan time, not implement time. (2) Critics running LIVE commands
  (cargo deny/mutants --list) caught the gate:8 red and the mutation surface truth BEFORE the
  gate run; the Rc-cycle find (tracing wry's retain site in its source) prevented corrupted
  teardown evidence from ever being recorded. (3) The assisted headed drive was ABORTED on an
  in-use desktop (synthetic input would hijack the operator's keyboard) and its capture deleted
  (private screen content) — the operator ran the protocol instead; respect-the-desktop is part
  of the headed lane's discipline. (4) crates.io API unreachable in-session — the feature audit
  deferred cleanly to `cargo add` time (cargo's own fetch worked).
- **Ticket:** forge #402 → done; local doc → tickets/closed/ (status closed).
- **Archive:** spec status → Phase 5 — Complete PASS; the pair moved to
  docs/planning/pipeline/completed/402-proof-of-embed.{spec,notes}.md.
