---
pipeline_id: ebbd2828-fb87-4d60-814d-2f9269af7c68
ticket: forge#389 (88f9c375-191d-4e52-892e-9b84309e9a4f) · local docs/planning/tickets/open/TICKET-389-spike-embedded-browser.md
aar_id: — (opened at promotion)
status: Phase 5 — Complete PASS
title: SPIKE — embedded browser substrate + Forge-in-browser plan (opens Phase E)
type: spike
milestone: M26
references:
  - docs/marley_architecture/roadmap.md (Phase E)
  - docs/marley_architecture/orchestration-shell.md (§8–9 — the browser + hosted-seat extension)
  - docs/planning/intake/ (embedded-agent-browser-chromium-cdp pillar)
  - docs/planning/pipeline/queued/385-sectioned-rail.spec.md (the Browser section this fills)
---

## Title
Settle how Marley gets a live web surface — the substrate, the embed mechanics against gpui, the
CDP question, and the Forge-in-browser plan — as a decision-complete spike. chad (2026-07-22):
the third rail section is **Browser**; "Forge will open in the browser" (the roadmap's Phase-E
"project Forge URL as a live pane", whose fleet-layer prerequisite M23–M25 just shipped). The
2026-07-22 map confirmed the starting point: **zero** webview capability anywhere in the
workspace.

## Scope
### In
Five questions, answered in a decision doc:
1. **Substrate.** `wry`/`tao` (Apache-2.0/MIT; wraps WKWebView on macOS) vs direct
   `objc2-web-kit` WKWebView (MIT; matches Marley's macOS-only reality; fewer layers) vs CEF
   (BSD; heavy; brings real Chromium + CDP). Evaluated against **gpui 0.2.2's self-composited
   scene**: gpui draws its whole window; a webview is a NATIVE child view — so the real questions
   are: can a child NSView overlay a gpui pane rect (position/resize sync), who wins input
   routing, z-order vs gpui overlays (palette, menus — which must draw ABOVE the webview or never
   overlap it), and scroll/occlusion behavior. A minimal scratch proof-of-embed (a WKWebView child
   over a gpui window) if the environment permits; otherwise the constraint is cited from the
   platform docs and the proof becomes slice 1 of the follow-up train.
2. **The CDP fork, named honestly.** The agent-browser pillar
   (embedded-agent-browser-chromium-cdp) wants CDP→MCP tools; **WKWebView (and therefore wry on
   macOS) exposes NO CDP.** The Forge pane needs none. Decide: WKWebView for the pane surface now
   + a separate headless-Chromium lane for the agent browser later, vs CEF once for both. Record
   the decision + its revisit trigger.
3. **Forge-in-browser.** URL source: derive the Forge web-UI base from the same `.mcp.json`
   active-root resolution #381 shipped (`mcp_config::mcp_json_path`) — ONE config, two consumers.
   Auth posture: the loopback wall stands; the MCP bearer is NEVER placed in a URL or exposed to
   page JS; the web UI's own auth (if any) is its own concern. v1 nav policy: pinned origin (the
   Forge base), no arbitrary browsing. Coexistence: the native forge cockpit tab
   (app.rs:5444-5462, `self.forge_sprint`) keeps working; the doc names the retirement criteria
   ("desktop for hands, web for intents" — the native fleet rail is NOT retired).
4. **The `TabContent::Browser` + codec migration plan.** The new variant; a `TabLayout` arm; a
   framing-safe grid leaf for a URL-bearing pane (`b=<url>`? — the reserved bytes `, : = \x1f`
   (grid) + `\t \n \r` (shell) constrain the encoding; a URL contains `:` and `=` → the leaf
   needs an escape or an indirection — decide it here so #385's Browser section has a landing
   path). Back-compat: old layouts restore unchanged.
5. **Licensing + the train.** All candidates permissive (wry/tao Apache-2.0/MIT; objc2 MIT; CEF
   BSD) — record versions; then the ordered follow-up slices (proof-of-embed → TabContent::Browser
   behind the section → Forge URL pane → nav chrome → the CDP lane).

### Out (explicitly deferred)
- ANY shipped app code (spike wall; scratch proof never staged).
- The actual webview integration (→ the follow-up train this spike produces).
- The CDP→MCP browser tool family (its own pillar; this spike only places it).
- Multi-tab browsing UX, favorites, devtools chrome.
- Windows/Linux webview lanes (macOS-only reality today; noted for the record).

## Reference (§20)
**N/A — Marley-specific.** No reference app to match: Warp has no embedded browser
(docs/warp_architecture/subsystems/00-overview.md — research map) and Zed's web views are not a
studied subsystem in our maps. The behavior ("the project's Forge URL as a live pane") is
Marley's own roadmap Phase E + orchestration-shell.md §8–9, ratified by chad 2026-07-20 and
sharpened 2026-07-22 ("Forge will open in the browser"). Research inputs are published docs of
permissive projects (wry/tao, objc2-web-kit, CEF) — no copyleft source involved.

### Prior art
1. **Behavior maps** — none applicable (no Warp/Zed analog; confirmed in the maps' overviews).
   The in-house design record IS the behavior source: orchestration-shell.md §8–9 (the browser
   half of the fleet plane: Forge URL pane, CDP→MCP family, hosted seat) + the
   embedded-agent-browser intake pillar.
2. **Published material** — wry/tao docs (cross-platform webview embedding; the child-window
   `WebViewBuilder` positioning APIs), Apple's WKWebView documentation (child-view embedding,
   input/first-responder behavior), CEF's off-screen-rendering docs (the only candidate that can
   render INTO a texture — relevant because gpui self-composites; OSR would sidestep the
   NSView-overlay problem at real complexity cost), the CDP spec (what the agent lane needs).
3. **Our permissive deps** — the sweep's key NEGATIVE result (2026-07-22 map): **no crate we ship
   owns this seam** — no webview dep exists in any manifest, gpui 0.2.2 has no webview element,
   and the one grep hit is a doc comment. gpui (Apache-2.0) source IS the study target for the
   embed side: how its macOS window/NSView is structured and whether a native child view can be
   layered (reading it is adoption). `block2`/`objc2` already in the lock file = the FFI
   toolchain a direct-WKWebView route would use is partly present.
4. **In-house constraints** — the loopback wall + bearer-never-logged rules (#370/#375/#381
   lineage) bind question 3; `mcp_config::mcp_json_path` (#381) is the URL-derivation seam;
   PR-claude-boot-decisions-key-the-restored-active-root-001 applies to the URL source (active
   root, never launch cwd).

## Locked-In Decisions
- **D1 — Browser-only section; Forge is a page.** (chad 2026-07-22.) The native forge cockpit
  coexists until the doc's named retirement criteria are met; the native FLEET rail is out of
  scope of any retirement ("desktop for hands, web for intents").
- **D2 — Docs-only spike.** Scratch proof code never staged; empty `crates/` diff at complete.
- **D3 — Security posture is inherited, not invented:** loopback wall, bearer never in URL/JS,
  pinned-origin v1 nav. Any relaxation is a future ticket with its own review.
- **D4 — The CDP fork gets an explicit recorded decision** with a revisit trigger — never an
  implicit "we'll see".
- **D5 — Decision-complete output** (the #388 D4 standard): the follow-up train's first ticket
  can start P2 from the doc alone.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The spike shall produce a decision doc naming the chosen substrate, with embed-against-gpui feasibility evidence (a scratch proof-of-embed, or the cited platform constraint + proof scheduled as train slice 1). | Doc review. |
| REQ-002 | The doc shall record the CDP fork decision (pane substrate vs agent-browser lane) with its revisit trigger. | Doc review. |
| REQ-003 | The doc shall carry the Forge-in-browser plan: URL derivation via the #381 resolution seam, the D3 auth posture, pinned-origin v1 nav, and the native-cockpit coexistence/retirement criteria. | Doc review. |
| REQ-004 | The doc shall carry the `TabContent::Browser` + shell/grid codec migration plan, incl. the framing-safe URL-leaf encoding and old-layout back-compat. | Doc review. |
| REQ-005 | The spike shall record a forge AD, list candidate-crate licenses/versions, and emit the ordered follow-up train; zero app code ships (`crates/` diff empty). | Forge record; doc; diff review at the gate. |

## Phase Plan
- **P2 Design** — the evaluation matrix's axes + the proof-of-embed plan (env check: can this
  session build/run a scratch WKWebView child?). Human checkpoint on the substrate shortlist.
- **P3 Implement** — write the decision doc (+ the scratch proof if env permits).
- **P3.5 Inspect** — critics attack the confident sentences: the embed claim (input routing,
  z-order vs palette overlays), the URL-derivation edge cases (no .mcp.json → the #384
  misconfigured arm), the codec escape.
- **P4 Validate** — REQ checklist vs the doc; empty-crates-diff; gate `--fast` (docs-only).
- **P5 Complete** — AD + AAR, archive, close #389; hand the train to /spec.
