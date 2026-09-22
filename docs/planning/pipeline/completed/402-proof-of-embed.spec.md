---
pipeline_id: ea606aa8-be2a-47c2-bb59-ce20f294fff8
ticket: forge#402 (a17801ff-b56e-4a7d-ace4-b3252c49147e) · local docs/planning/tickets/open/TICKET-402-proof-of-embed.md
aar_id: 120433f1-f4b4-42d9-b0c9-58a00a255fe9
status: Phase 5 — Complete PASS
title: Scratch proof-of-embed — a wry child WKWebView over a gpui window (the #389 train slice-1)
type: spike
milestone: M29
references:
  - docs/marley_architecture/embedded-browser-model.md
  - docs/planning/pipeline/completed/389-spike-embedded-browser.spec.md
  - docs/zed_architecture/subsystems/01-gpui-ui-framework.md
  - scripts/gates.sh
  - deny.toml
  - Cargo.lock
  - crates/marley_visual_harness/Cargo.toml
  - ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2 (src/platform/mac/window.rs, src/window.rs)
---

## Title
The #389 train's slice 1 — an **env-gated scratch probe bin** that attaches a **wry
`WebViewBuilder::build_as_child` WKWebView** to a gpui window through the window's exposed
`RawWindowHandle::AppKit`, and walks the go/no-go checklist embedded-browser-model.md Q1 pinned before
`TabContent::Browser` is allowed to exist. The #389 spike locked wry-as-child (D: Q1-A) on paper with
**two load-bearing unknowns** no doc can settle: **(1) z-order** — the child NSView composites ABOVE
gpui's single self-composited Metal scene (`GPUIView`, gpui platform/mac/window.rs:111), so gpui's own
overlays (palette, menus) draw BEHIND it unless the hide-the-webview-while-overlay-up shim works in
practice; **(2) focus/IME** — gpui routes keys by making its one native view first responder
(`native_window.makeFirstResponder_(native_view)`, platform/mac/window.rs:776), so a native child
capturing first-responder status (and IME composition) must hand keyboard focus back cleanly, both
directions. The probe answers both with a headed, operator-driven run and records **GO** (wry-A stands)
or **NO-GO** (pivot to CEF-OSR via gpui `paint_surface(bounds, CVPixelBuffer)` — window.rs:3181, with
`core-video` 0.4.3 already in the lock) back into embedded-browser-model.md + the AD. This is **the ONE
place wry enters the tree**; the product binary stays wry-free. Findings are the deliverable — the probe
itself is §0 ACCEPTED-UNTESTABLE scratch behind an explicit, documented gate exclude.

**Tree verification (2026-08-06 — the design doc is M26-era; all its gpui claims re-verified against
gpui 0.2.2 as shipped today, zero drift):** `impl rwh::HasWindowHandle for MacWindow` returns
`RawWindowHandle::AppKit(AppKitWindowHandle::new(native_view.cast()))` (platform/mac/window.rs:1548,
:1552-1553); `impl HasWindowHandle for Window` delegates to `self.platform_window.window_handle()`
(window.rs:4845-4847); `paint_surface` at window.rs:3181 and the public `surface()` element at
elements/surface.rs:32 (the pivot door exists today). `wry` is ABSENT from Cargo.lock; `core-video`
0.4.3 and exactly ONE `raw-window-handle` (0.6.2) are present — wry 0.56.0 requires `^0.6`, so the
handle gpui hands out type-checks straight into `build_as_child` with no rwh dual-version hazard. One
drift found: wry has moved **0.55.1 → 0.56.0** (2026-07-30) since the #389 sweep — P2 re-pins.

## Scope
### In
- **The probe bin** (home = D-OPEN-PROBE-HOME): a scratch gpui app that opens one window, obtains its
  `HasWindowHandle` (window.rs:4845), attaches a wry child WKWebView via `build_as_child`, and provides
  the three checklist affordances — a designated pane rect the child tracks (`with_bounds`/`set_bounds`
  across resize/move), a palette-style gpui overlay toggle wired to the hide-shim
  (candidate `set_visible(false)` — D-OPEN-HIDE-SHIM), and a focus/IME exercise (a local `data:` page
  with a text input so composition can be typed inside the webview; no network dependency).
- **The env gate**: the bin refuses to run without an explicit env flag (candidate
  `MARLEY_WEBVIEW_PROBE=1`; exact name P2) — a headed run is always a deliberate operator act.
- **The gate exclude, explicit + documented (§0)**: a named coverage-exclude entry with its reason in
  gates.sh's ACCEPTED-UNTESTABLE comment block (:179-220) + the `--ignore-filename-regex` (:222), and
  per-fn `#[mutants::skip]` with justification (the `mutants = "0.0.3"` attribute-crate precedent,
  marley_visual_harness/Cargo.toml:23). Never a silent regex ride.
- **The recorded verdict**: GO/NO-GO with per-checklist-point evidence written into
  embedded-browser-model.md Q1 + the AD block (:158) updated; forge
  `architecture-decision-record`, doc-local fallback per the #388/#389 precedent (the tool erred both
  times — 389 notes :125).
- **Supply-chain vetting of the ONE new dep tree**: wry (+ its new transitives, e.g. objc2-web-kit
  0.3.x) pass gate:8 `cargo deny` against deny.toml's permissive-only allowlist and gate:7
  `cargo audit`.

### Out (explicitly deferred)
- **Any product-shell wiring** — no `marley_app/src` edit; the `marley` product binary and render path
  are untouched and stay wry-free at the linker level.
- **`TabContent::Browser`**, the `B=` shell tag, `rail_section`/`focus_label` arms — train slice 2.
- **The Forge URL pane itself** (pane-rect sync in the real shell, #381 URL derivation, pinned-origin
  nav, the production hide-shim) — train slice 3; nav chrome/lifecycle — slice 4.
- **CEF-OSR work beyond naming the pivot** — the NO-GO arm records the pivot decision; it does not
  start it. The CDP agent-browser lane stays its own pillar (#389 Q2).
- Windows/Linux lanes; multi-webview; any persistence.

## Reference (§20)
**N/A — Marley-specific.** No reference app to match: the maps carry **no embedded-webview behavior**
for either reference (re-checked this plan — see Prior art leg 1; #389's spec recorded the same
negative at its Reference section). The behavior source is Marley's own design record:
embedded-browser-model.md (the M26 #389 decision doc this spike executes) + roadmap Phase E. Research
inputs are the published docs of permissive crates (wry Apache-2.0/MIT, objc2-web-kit
Zlib/Apache/MIT); **gpui (Apache-2.0) source reading is ADOPTION, explicitly outside the wall** — it
is this spec's primary evidence leg. No copyleft source consulted.

### Prior art
1. **Behavior maps — checked, no embedded-webview analog.** docs/warp_architecture/: the browser/web
   grep hits are Warp's *cloud/web* crates (mcp.md, websocket.md, serve-wasm.md, warp_web_event_bus.md
   — server/client plumbing, not an in-app webview surface). docs/zed_architecture/: the only
   "embedded-browser" mentions are **Marley's own forward notes** — 01-gpui-ui-framework.md:807
   ("`PaintSurface` is the zero-copy video door" for the embedded-browser/CDP lane — the pivot route's
   map entry) and 07-workspace-panes-palette.md:19/:279/:554 (pane-item intake triggers). Research,
   not source.
2. **Published material — wry docs.rs re-fetched live this plan (2026-08-06), refreshing the #389
   sweep.** Current wry = **0.56.0** (2026-07-30, Apache-2.0/MIT; the design doc's 0.55.1 has
   drifted). Confirmed API surface, by name: `WebViewBuilder::build_as_child` ("supported on macOS,
   Windows and Linux (X11 only)"), `with_bounds`; on `WebView`: `bounds()`, `set_bounds(Rect)` ("only
   effective if the webview was created as a child" — exactly our shape), **`set_visible(bool)`** (the
   hide-shim candidate is first-party API, not a hack), and **`focus()` / `focus_parent()`** ("try
   moving focus away from the webview back to the parent window") — a published two-way focus-handoff
   surface that maps 1:1 onto checklist point 3. wry's manifest wants `raw-window-handle ^0.6`,
   `objc2 ^0.6.4`, `block2 ^0.6`, `objc2-web-kit ^0.3.2`. Apple's WKWebView first-responder/IME
   behavior stays the platform-doc record behind the focus unknown (#389 sweep). `objc2-web-kit`
   direct is the named in-family fallback if wry's abstraction gets in the way (design doc Q1 note).
3. **Our permissive deps — the adoption leg, verified in gpui 0.2.2 source as shipped today**
   (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2`): `MacWindow: HasWindowHandle`
   → `RawWindowHandle::AppKit` (platform/mac/window.rs:1548-1556); `gpui::Window: HasWindowHandle`
   delegating (window.rs:4845-4849); the `GPUIView` single-NSView scene (platform/mac/window.rs:111);
   key routing via `makeFirstResponder_` (platform/mac/window.rs:776); the pivot door `paint_surface`
   (window.rs:3181) + `surface()` (elements/surface.rs:32). In-lock today: `raw-window-handle` 0.6.2
   (sole version), `core-video` 0.4.3, `objc2` 0.6.4, `block2` 0.6.2 — wry's macOS FFI base unifies
   with what gpui already ships; `wry` itself is absent (grep exit 1), so this spike really is its one
   entry point. **No crate we ship owns the child-embed seam itself** — wry is the new owner under
   test.

## React-first (parity)
**N/A — no UI delta:** the probe is a scratch, env-gated, operator-run headed bin outside the product
shell — no product surface, chrome, overlay, or pane changes (TabContent::Browser and the pane are
explicitly Out), so there is nothing for marley-web to prototype or keep parity with. The verdict doc
feeds the later train slices; their product UI (the Browser pane, nav chrome) goes React-first when
those slices are planned.

## Locked-In Decisions
- **D1 — wry-as-child WKWebView is the substrate under test.** The probe exercises exactly the Q1-A
  mechanics: `WebViewBuilder::build_as_child` against the gpui window's `RawWindowHandle::AppKit`
  (window.rs:4845 → platform/mac/window.rs:1548). `objc2-web-kit` direct is the named in-family
  fallback only if wry's abstraction itself blocks a checklist point; **CEF-OSR via
  `paint_surface` + core-video is the named pivot**, triggered only by a recorded NO-GO — no CEF work
  in this spike.
- **D2 — the probe is env-gated scratch and NEVER ships in the product binary or render path.** It is
  committed, workspace-member code (so the gates see it honestly) behind an explicit env flag; the
  `marley` product binary stays wry-free at the **linker level**, provable by `cargo tree -i wry`
  showing no path from the product bin (mechanism per D-OPEN-PROBE-HOME's resolution).
- **D3 — the Q1 go/no-go checklist IS the acceptance, all three points.** (1) position/resize the
  child to a pane rect (`with_bounds`/`set_bounds`), (2) a gpui palette-style overlay still shows via
  the hide-the-webview-while-overlay-up shim, (3) keyboard + IME first-responder handoff both
  directions. A run that skips a point yields no verdict — z-order alone is not the checklist.
- **D4 — the output is the recorded verdict**: GO (wry-A stands) or NO-GO (pivot to CEF-OSR) with
  per-point evidence, written into embedded-browser-model.md Q1 + the
  `AD-claude-embedded-browser-substrate-001` record updated (forge `architecture-decision-record`;
  doc-local fallback per #388/#389 — the tool erred both times).
- **D5 — the gate exclude is explicit + documented per §0.** Coverage: a named entry + reason in
  gates.sh's ACCEPTED-UNTESTABLE block (:179-220) feeding the `--ignore-filename-regex` (:222).
  Mutation: whole-workspace with ZERO file exclusions stands (:226-227) — the probe's fns carry
  `#[mutants::skip]` + a written justification each. gate:3's `--no-tests=warn` (:83) keeps the
  test-less probe a **visible** warning. If the probe home already sits under an existing excluded
  path, the documented comment is still extended to NAME the probe + its reason — no riding an
  incidental regex.

**D-OPEN (Phase 2 decides, with evidence):**
- **D-OPEN-PROBE-HOME** — `marley_app/src/bin/` vs an example vs a dedicated probe crate.
  **Recommend: a dedicated bin-only crate (candidate `crates/marley_webview_probe/`).** Grounding:
  a bin inside marley_app puts `wry` into marley_app's `[dependencies]`, which links wry into the
  `marley` product binary (breaking D2) unless it goes optional-dep + `[[bin]] required-features` —
  and required-features makes gate:2's `--all-targets` (gates.sh:368) SKIP the probe, leaving it
  uncompiled/unlinted by every static gate (invisible rot). A dedicated crate keeps wry confined to
  one manifest (`cargo tree -i wry` → one reverse path), gets compiled + linted by every gate run,
  costs exactly one new documented coverage-exclude entry (the §0 shape), and matches the
  marley_visual_harness precedent (a dedicated crate for a headed, OS-bound surface with documented
  excludes; its Cargo.toml also records the "gpui `Application::run()` owns the main thread"
  subprocess lesson the probe inherits).
- **D-OPEN-OVERLAY-SIM** — how the probe shows a palette-STYLE overlay (the probe is not marley_app,
  so the real ⌘⇧P palette is out of reach by design). Candidates: a probe-local centered gpui
  overlay layer (anchored/deferred paint, palette-like geometry) toggled by a keybinding; or minimal
  reuse of `ui_components` widgets. P2 picks the cheapest thing that makes the z-order collision
  REAL (the overlay rect must overlap the webview rect).
- **D-OPEN-HIDE-SHIM** — the hide mechanism under test: `set_visible(false)` (first-party wry API —
  confirmed present in 0.56.0) vs `set_bounds` to a zero/offscreen rect vs remove-from-superview via
  objc2. **Recommend: `set_visible` first**; observe and record WebKit side effects (flicker on
  restore, process suspension, painting stalls) — the observation is itself a slice-3 input.

## Acceptance Criteria (EARS)
Headed observations are verified by the **operator-driven probe run with observations (+ screenshots)
recorded into the pipeline notes Phase 4 section** — NOT unit tests: the probe is a live-compositor,
first-responder-dependent headed path, §7's genuinely-uncoverable class. (docs/warp_architecture/
observed/ is scoped by its README to *reference-app* captures, so probe evidence lands in the notes,
which are committed under docs/planning/.)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the operator runs the env-gated probe headed, the system shall attach a wry child WKWebView to the gpui window via `build_as_child` on the window's `RawWindowHandle::AppKit` and shall position/resize it to the designated pane rect (`with_bounds` at build, `set_bounds` on change), tracking the rect through window resize/move. | operator-driven headed run; checklist point 1 observations (rect alignment at rest + during live resize) + screenshots recorded in the notes Phase 4 |
| REQ-002 | WHEN the palette-style gpui overlay is toggled up WHILE the webview child is attached and visible, the system shall hide the webview via the shim (candidate `set_visible(false)`) so the overlay renders fully visible and interactive, and shall restore the webview on overlay dismiss. | operator-driven headed run; checklist point 2 observations (overlay unobscured; restore behavior; flicker/latency notes) + screenshots in the notes Phase 4 |
| REQ-003 | WHEN the operator moves focus between the webview and the gpui surface (click each direction; `focus()`/`focus_parent()` where applicable), keyboard input INCLUDING an IME composition shall land in the focused side, and focus shall hand back cleanly both directions (gpui keys route again after leaving the webview — the makeFirstResponder contract, platform/mac/window.rs:776). | operator-driven headed run; checklist point 3 observations both directions, incl. one IME composition inside the webview; the gpui-side return evidence is RAW key routing via the last-key display *(P3.5 ledger F4 narrowing: the probe's gpui surface is display-only — no input handler — so a gpui-side IME composition is structurally out; product-grade IME-after-return is slice-3's assertion in the real shell)*; recorded in the notes Phase 4 |
| REQ-004 | WHEN the probe session concludes, the pipeline shall record the verdict — GO (wry-A stands) or NO-GO (pivot to CEF-OSR via `paint_surface` + core-video) — with per-checklist-point evidence into embedded-browser-model.md Q1, and shall update `AD-claude-embedded-browser-substrate-001` (forge AD tool; doc-local per the #388/#389 fallback). | doc review at P5: the diff shows the Q1 verdict text + the updated AD record; forge AD call attempted and its outcome noted |
| REQ-005 | WHEN `scripts/gates.sh` runs (FULL or `--diff`) with the probe in-tree, the gate shall exit green with the probe's ACCEPTED-UNTESTABLE exclusion EXPLICIT and documented — a named coverage-exclude entry + reason in the gates.sh block (:179-220/:222) and per-fn `#[mutants::skip]` justifications — never a silent regex ride. | gates.sh exit code 0; review: the exclude entry names the probe + reason, the skip attributes carry written justifications; the `probe_enabled` unit tests RUN under nextest *(P3.5 ledger F12: the crate HAS tests — the earlier no-tests-warning expectation is superseded)* |
| REQ-006 | WHILE this spike is in flight and at close, the product binary and app shell shall be untouched — no `marley_app/src/**` edit, no `TabContent::Browser`, and no path from the `marley` product binary to `wry`. | diff review (crates diff confined to the probe home + gate files); `cargo tree -i wry` shows wry reachable only from the probe target per the resolved D2 mechanism |

## Floors (constitution)
**Pure surface: none expected — do not invent purity.** The probe is one headed shim end to end. IF
Phase 2 finds a genuinely pure helper worth extracting (e.g. a pane-rect → wry `Rect`
coordinate/DPI mapping fn), that helper gets unit tests at **cov/MSI 100** like any pure seam;
otherwise the pure floor is vacuously satisfied and says so in the notes.

**ACCEPTED-UNTESTABLE: the probe bin itself** (live compositor + WebKit process + first-responder
state — unreachable by tests). The exclusion mechanism, by name, per §0:
- **gate:4 coverage** — one new entry in the `--ignore-filename-regex` (gates.sh:222) with its reason
  added to the documented ACCEPTED-UNTESTABLE comment block (gates.sh:179-220), following the
  marley_visual_harness / `marley_app/src/bin/` precedent wording.
- **gate:5 mutation** — no file-level exclusion exists or is added (whole-workspace, zero exclusions —
  gates.sh:226-227); every probe fn carries `#[mutants::skip]` + a written justification (the
  `mutants = "0.0.3"` no-op attribute crate, per marley_visual_harness/Cargo.toml:23).
- **gate:3** — the probe ships the `probe_enabled` unit tests, which RUN under nextest (P3.5
  ledger F12 superseded the earlier no-tests expectation; `--no-tests=warn` (gates.sh:83) remains
  the net for any future test-less crate).
- **gate:2/1/9/14** still bind fully: the probe compiles clippy-clean at `-D warnings --all-targets`
  (gates.sh:368), is fmt-clean, its deps are machete-used, and it contains no Warp/Zed brand words
  (the gate:14 brand-scrub, gates.sh:168).
- **gate:6 miri** — the probe should need no `unsafe` of its own (wry owns the FFI); if P2 finds any,
  it carries `// SAFETY:` (gate:13) and the per-crate miri posture is declared per §0 gate:6.
- **gate:7/8** — cargo audit + cargo deny run over the NEW lock tree: wry's whole dependency tree must
  clear deny.toml's permissive-only allowlist (deny.toml:39-51); any copyleft transitive is a red, not
  an ignore.

## Phase Plan
- **P2 Design** — settle D-OPEN-PROBE-HOME (recommendation above), D-OPEN-OVERLAY-SIM,
  D-OPEN-HIDE-SHIM; pin the wry version (0.56.x current; re-check at design time) + the exact feature
  set (audit wry's default features for what a macOS child embed actually needs) + confirm rwh stays
  single-version after `cargo update -p`-free add; name the env flag; the probe file manifest (crate/
  bin layout, the `data:` exercise page, keybindings); draft the gates.sh exclude wording + the
  operator run-protocol (the exact checklist script + what to record per point).
- **P3 Implement** — the probe: crate + env-gated bin + the three checklist affordances (pane-rect
  tracking, overlay toggle wired to the hide-shim, focus/IME exercise page). No product-shell edits.
- **P3.5 Inspect** — adversarial review incl. **license/supply-chain**: gate:8 `cargo deny` over the
  new lock tree (wry's transitives permissive-only), gate:7 audit; §20 provenance (no copyleft source
  consulted; gpui reading recorded as adoption); D2 check (`cargo tree -i wry` — no product path); the
  exclude wording matches §0's explicit-and-documented bar; env-gate actually refuses an ungated run.
- **P4 Validate** — the operator runs the probe HEADED and walks ALL THREE checklist points; record
  observations + screenshots into the notes Phase 4 (the honest §7 verification for a headed path —
  no unit-test theater); then `scripts/gates.sh` green (FULL or `--diff`) WITH the documented exclude;
  confirm the no-tests warning is visible.
- **P5 Complete** — write the verdict into embedded-browser-model.md Q1 (+ the AD block :158);
  `architecture-decision-record` update (doc-local per the #388/#389 fallback if the tool errs);
  **CHANGELOG: this spike DOES ship `.rs` under `crates/`** (the probe bin), so BOTH commit hooks fire
  — `enforce-commit-gate.sh` demands the FULL/`--diff` receipt and `enforce-changelog.sh` demands the
  CHANGELOG entry in the same changeset (§15/§21) — plan the commit accordingly; archive the pipeline
  docs; close #402.
