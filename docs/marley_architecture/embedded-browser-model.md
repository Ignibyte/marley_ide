# The embedded-browser model — the generic embedded browser (M26 #389 spike, opens Phase E)

**Status:** the train SHIPPED through #406, then the **2026-08-09 scrap-forge pivot (#410) ripped its
Forge identity**: §Q3's URL-derivation charter is **RETIRED** — Marley has NO production browser-origin
source (the mount gate's `mount_plan(None)` / the Retry arm's `retry_plan(None, …)` are the one seam a
future URL feature fills). Everything GENERIC survives, shipped and tested: the Q1 substrate
(wry-as-child WKWebView + the overlay hide-shim — 27 states as built, 26 since #411 retired the
sprint card), the Q2 two-substrate split, the #406
lifecycle machine (probe/deadline/renderer-gone/error cards; the probe re-homed in-app at #411 —
`marley_app::browser_probe`, its livewire test riding along), the #404-hardened pinned-origin nav
verdict (moved in-app: `browser.rs::same_web_origin`, #410), and the Q4 bare-`B` codec. The pre-#410
body below is the design record of the train as built — Q3's charter reads as history, not live law.
**Reference (§20):** N/A — Marley-specific (chad's "Forge opens in the
browser", 2026-07-22; roadmap Phase E + [orchestration-shell.md](orchestration-shell.md) §8–9 — the
Forge-facing halves of both retired by the pivot). No
Warp/Zed analog; research inputs are the published docs of **permissive** crates (wry, objc2-web-kit,
cef — no copyleft source). gpui (Apache-2.0) source is adoption.

## The target
chad's third rail section is **Browser** (the #385 `RailSection::Browser` already ships — today it is the
cockpit/Forge tabs' rail home, `tabs.rs:82`; only the *webview content* is absent); **"Forge will open in
the browser"** — the project's Forge web UI rendered as a live pane,
the roadmap's Phase-E "project Forge URL as a live pane." Starting point (2026-07-22 map): **zero webview
capability** in the tree today.

## Q1 — Substrate: **wry-as-child WKWebView for v1**, CEF off-screen-render as the named alternative

**The gpui constraint (Apache-2.0 study):** a gpui macOS window is **one self-composited Metal `NSView`**
(`GPUIView`, gpui `platform/mac/window.rs:111`) — gpui paints its WHOLE scene into one Metal drawable each
frame; there is no per-element native subview tree. Two escape hatches exist:
- **(a) The NSView is exposed via `raw-window-handle` 0.6.2** — `MacWindow: HasWindowHandle` returns
  `RawWindowHandle::AppKit(native_view)` (window.rs:1548), and `gpui::Window: HasWindowHandle` delegates
  to it. A child view attached through this handle becomes a subview **of the Metal `GPUIView`**, so it
  composites **above** gpui's painted scene. gpui offers no API to register/position it — you manage the
  child yourself through the raw handle.
- **(b) `Window::paint_surface(bounds, CVPixelBuffer)`** (gpui `window.rs:3181`, the public `surface()`
  element) composites an externally-rendered CoreVideo pixel buffer INTO gpui's own Metal scene (used
  upstream for video). `core-video` is **already in Marley's lock file** — a native off-screen-render
  compositing target exists today.

**The two embed shapes + the fork:**

| | **(A) wry-as-child WKWebView** | **(B) CEF off-screen-render** |
|---|---|---|
| how | wry `WebViewBuilder::build_as_child(HasWindowHandle)` + `with_bounds`/`set_bounds` to a pane rect — attaches a WKWebView subview to gpui's exposed handle | CEF renders the page into a pixel buffer (`RenderHandler::OnPaint`, windowless) → gpui `paint_surface(CVPixelBuffer)` composites it as a normal element |
| engine | **WKWebView** (WebKit, native macOS) | **Chromium** (real Blink + CDP) |
| **z-order** | ❌ **the hard problem** — the child composites ABOVE gpui's Metal scene, so gpui's OWN overlays (⌘⇧P palette, context menus, find bar) draw BEHIND the webview. Mitigation (primary): **hide the webview while a gpui overlay is up** — the reliable measure, since the palette / context menus are centered and can cover any pane rect, so "constrain overlays to never overlap the webview" is only a *partial* secondary measure. A real but bounded shim | ✅ **solved for free** — the page is just a texture gpui draws, so gpui overlays layer above it naturally |
| weight | **light** — wraps native WebKit, no bundle; wry consumes the same `raw-window-handle` 0.6.2 gpui exposes | **heavy** — bundles the CEF/Chromium binary framework (hundreds of MB); a render-pump + input-injection plumbing |
| CDP | ❌ none (WebKit ≠ Chromium) | ✅ CDP (the agent-browser lane, Q2) |
| license | Apache-2.0 OR MIT (wry 0.55.1) | Apache-2.0 OR MIT (cef 150.x; + the separate binary framework) |

**Decision: (A) wry-as-child WKWebView for the Forge pane v1.** The Forge web UI is a **trusted,
first-party** page shown in a single pane — the light, native, no-Chromium-bundle path is right, and the
z-order overlay-collision is a **managed shim gate** (primary mitigation: hide the webview while an
overlay is up; constraining overlays to the non-webview area is only a partial secondary measure), not a
blocker. **The z-order is the load-bearing UNKNOWN** → the follow-up train's slice 1 is a **scratch
proof-of-embed** (a wry child over a gpui window) whose go/no-go checklist is: **(1)** position/resize the
child to a pane rect, **(2)** a gpui palette overlay can still show over/around it (z-order), and **(3)**
**keyboard / first-responder + IME focus hands off cleanly** between the WKWebView child and the GPUIView
— gpui routes keys through `makeFirstResponder(native_view)` (window.rs:776), so a native child
intercepting focus/IME is a *second* real unknown alongside z-order — before the section is committed.
*(An env-in-session proof was NOT attempted — D2 bars adding `wry` to a staged branch; the proof is train
slice 1.)* **(B) CEF-OSR is the named revisit:** if the z-order proves
too painful OR the agent-browser (Q2) must share the pane surface, CEF-OSR's clean composite (via the
already-present `paint_surface`) + unified CDP would justify the weight.

**→ VERDICT (M29 #402, 2026-08-06): GO — wry-as-child stands.** The slice-1 probe
(`crates/marley_webview_probe`, env-gated, the tree's ONE wry entry point) ran all THREE checklist
points headed, operator-confirmed: **(1)** the child positions/resizes to the pane rect via a
render capture+diff `set_bounds` sync — and wry's `set_bounds` is Logical→Logical, so **no
scale-factor math is needed** (slice-3 input); **(2)** the hide-shim holds under BOTH mechanisms —
first-party `set_visible(false)` (primary) and zero-bounds (alternate) — with the overlay fully
visible and the webview restoring cleanly; **(3)** keyboard + IME hand off cleanly both directions
(composition inside the webview; the page-IPC `focus_parent()` return trip — wry 0.56 ships
first-party `focus()`/`focus_parent()`, a drift from the 0.55.1 this doc pinned). One landmine for
the production slices, found at #402's inspect and now a standing rule: **wry retains every handler
closure INSIDE the WebView it builds — a callback capturing state that owns the WebView must
capture WEAKLY** or the WebView's `Drop` is unreachable
(`PR-claude-callback-stored-inside-owned-resource-captures-weak-001`; #405/#406 wiring must honor
it). Slices 3–4 proceed on substrate (A); CEF-OSR remains the named revisit, untriggered.

*(A direct `objc2-web-kit` WKWebView route also exists — the `objc2` family is already in-tree, only
`objc2-web-kit` 0.3.2 (Zlib/Apache/MIT) is new — but wry's `build_as_child` + `set_bounds` already own
the child-positioning glue, so wry is the lower-effort v1; drop to raw `objc2-web-kit` only if wry's
abstraction gets in the way.)*

## Q2 — The CDP fork: **two substrates** (WKWebView pane + a separate headless-Chromium agent lane)

The agent-browser pillar ([[embedded-agent-browser-chromium-cdp]], orchestration-shell.md §8–9) wants a
**CDP→MCP** tool family (drive a browser as data). **WKWebView exposes NO CDP** (it is WebKit; CDP is a
Chromium protocol — confirmed). The Forge PANE needs no CDP (it just displays a page).

**Decision: two substrates.** The **Forge pane** = WKWebView (Q1-A, no CDP needed). The **agent browser**
= a separate **headless-Chromium** lane (CEF, or a driven external Chrome over `--remote-debugging-port`)
that speaks CDP — its own pillar, out of this section's scope. **Revisit trigger:** if the agent browser
must be VISIBLE in a pane AND share the Forge surface, collapse both onto **CEF-OSR** (one Chromium, CDP
+ clean composite) — accepting the bundle weight. Until then, two lanes keep the common case (a trusted
Forge page) light.

> **⚠ THE TRIGGER HAS FIRED (2026-08-11, owner decision — recorded in
> [`../planning/intake/embedded-agent-browser-chromium-cdp.md`](../planning/intake/embedded-agent-browser-chromium-cdp.md),
> the 2026-08-11 amendment).** The agent-browser pillar's stated requirement is now "the agent SEES
> what the user is looking at and describes it" — which forces the agent onto the SAME browser the
> human is viewing (same session, cookies, scroll, DOM), definitionally the shared-surface condition
> above. **Two lanes are retired as the target; ONE Chromium serves display + agent**, via the
> intake's screencast-into-a-gpui-texture route. Nothing has shipped against this, so the Q1-A
> WKWebView pane remains the LIVE substrate and is NOT to be ripped preemptively — it retires when a
> proven Chromium replacement lands, and the #405 hide-shim retires with it (the texture route makes
> gpui overlays layer above the page by construction, deleting the 26-state inventory's coupling to
> all future overlay work). The three pillars the amendment adds — a CDP element picker, a natively
> rendered annotation layer, and a flight-recorder trace — do not exist in this section's design.
> Read Q2 as superseded intent; Q1's shipped substrate as current fact.

## Q3 — Forge-in-browser *(RETIRED at #410 — the scrap-forge pivot; kept as the design record)*

> **#410:** everything in this section describing a LIVE derivation is history — `forge_web_base`,
> its boot/Retry call sites, and the `.mcp.json`→origin chain are deleted from `marley_app`; the
> pane's standing production state is the #403 placeholder ("The embedded browser awaits a page
> source."). The bearer-hygiene and parsed-host-loopback lessons live on in
> `browser.rs::same_web_origin`/`web_origin_of` (moved in-app) and the knowledge ledger.

- **URL source:** derive the Forge web-UI base from the SAME `.mcp.json` active-root resolution #381
  shipped (`mcp_config::mcp_json_path`) — **one config, three consumers** (the MCP client, the fleet
  brain endpoint, and now the browser URL; the first two already share it — app.rs:2207/2220).
  `PR-claude-boot-decisions-key-the-restored-active-root-001` applies: keyed on the restored ACTIVE
  project root, never the launch cwd (the #381 fix). **The endpoint→web-base step (PINNED — #404
  shipped it):** `marley_forge_client::forge_web_base(mcp_json) -> Option<String>` derives the
  **WHATWG origin, ASCII serialization** of the wall-admitted MCP endpoint (`mcpServers.forge.url` =
  `…/mcp`): scheme + host + port, path dropped, **scheme-default port elided** (`:80` ≡ portless —
  so #405's pinned-origin nav compares ORIGINS, never raw strings), IPv6 canonicalized
  (`[0:0:…:1]` → `[::1]`). It rides `forge_endpoint_from` (one parse, one loopback wall; a
  bearer-less config derives nothing — one-config-coherence) and **re-asserts loopback on the
  PARSED host** (`PR-claude-recheck-the-parsed-value-the-consumer-uses-001`: the wall's hand-parse
  and WHATWG disagree on `…:pass@evil.com` shapes — the derivation checks the host the origin is
  BUILT from, so a hostile config can never widen the pane beyond loopback). The explicit-mapping
  escape hatch stayed OUT: the 2026-08-07 real-instance probe found **no web UI at the origin at
  all yet** (unauthed `/` → 401 JSON; authed `/`+candidates → 404) — the seam pins the contract
  for when the forge ships one, and #405's pane renders the honest error state until then. No
  `.mcp.json` / unresolved / refused → `None` → **no Browser content** (the section
  shows its empty rail home, not a broken page). *(This is **this doc's own** decision — a sibling of
  #384's misconfigured-state pattern, NOT #384 itself: #384 is a QUEUED spec about the **fleet** rail
  header, not the Browser section. It is grounded in `mcp_config`'s shipped "unresolved active-root →
  surfaces as misconfigured downstream" semantics, mcp_config.rs:23.)*
- **Auth (inherited, D3 — not invented):** the loopback wall stands; the MCP **bearer is NEVER placed
  in the URL or exposed to page JS** (#370/#375 bearer-never-logged lineage). The Forge web UI does its
  own auth (a session cookie / its own login) — Marley just points the pane at the origin. *(Assumption
  to validate at slice 3: that the web UI authenticates itself. If it instead expects the MCP bearer, a
  pinned no-bearer pane renders **unauthenticated** — a functional gap to close then, never by putting
  the bearer in the URL.)*
- **v1 nav policy:** **pinned origin** — the pane loads only the Forge base; no address bar, no arbitrary
  browsing (a `WKNavigationDelegate`/wry nav callback rejects off-origin navigations). A general browser
  is a later, separately-reviewed capability.
- **Coexistence / retirement:** the native forge cockpit (`cockpit_body` Forge arm, app.rs:5451, over
  `self.forge_sprint`) **keeps working**; the web pane is additive. Retirement criteria (named, not
  automatic): the web UI reaches feature parity for the cockpit's job AND chad calls it. The native
  **FLEET rail** (`fleet_rail_body`, app.rs:1006) is **NEVER** retired — "desktop for hands, web for
  intents" ([[marley-hypermedia-ui-conclusion]]): the live seat mechanics stay native; the web pane is
  the "intents" surface.

## Q4 — `TabContent::Browser` + the codec

- **The variant:** `TabContent::Browser(BrowserState { url, … })`; `rail_section()` →
  `RailSection::Browser`; `focus_label` (status_bar.rs:81) → a `Browser` arm; a `TabLayout::Browser`
  codec arm. *(A `PaneContent::Browser` / grid `PaneKind` — a browser as a split CELL — is deferred to
  the #388 registry's Browser kind; see the two-layer note below, its codec differs.)*
- **Two codec layers — pick the right artifact.** A Browser TAB and a Browser split-CELL serialize at
  **different** layers with **different** reserved bytes (verified in grid_layout.rs):
  - **v1 = a whole TAB → the SHELL codec.** A Browser tab is a `TabLayout::Browser` entry, exactly like
    the Cockpit tab's `C=<section key>` (`serialize_shell`, grid_layout.rs:355; entries are `T=<grid
    blob>` / `C=<key>` / `V=<path>`, line 325). The shell codec only reserves the entry/field/project
    separators `\t \n` (plus `\x1f`) — a URL contains **none** of those, so even a literal `B=<url>`
    would be **framing-safe** here.
  - **deferred = a split-CELL → the GRID codec** (inside a `T=` blob). There the leaf codec reserves
    `\t \n \r , : = \x1f` (`breaks_grid_framing`, grid_layout.rs:310) — a URL contains `:` and often `=`
    (but **not** `/`, which is *not* reserved), so a raw `b=<url>` **grid leaf** would break framing.
- **Decision — persist a bare MARKER, re-derive the URL (holds at BOTH layers).** *(#410: the decision
  OUTLIVES the derivation it was made for — with no origin source at all, "nothing persisted" is
  trivially still true; a future URL feature inherits the marker + the user-typed-URL deferral below.)*
  v1 Browser content WAS
  the project's Forge URL (DERIVED from `.mcp.json`, not user-typed), so nothing needed to persist a URL at
  all: the tab is a **bare `TabLayout::Browser` tag** (a `B` entry mirroring Cockpit's `C`) and the URL is
  **RE-DERIVED on restore** from the active root (the #381 seam). The single source of truth stays the
  config, no URL ever enters persisted state (trivially framing-safe at either layer), and a Forge URL
  that later changes is picked up automatically. The deferred split-cell reuses the same indirection as a
  **bare grid leaf `b`** (like the `f`/`g` FileTree/Git leaves that carry no detail), side-stepping the
  grid `: =` hazard. A future general **user-typed-URL** browser — which MUST persist an arbitrary URL —
  gets percent-encoding or a side-table id, **deferred.** Back-compat: an old layout (no `B` tag / no `b`
  leaf) restores unchanged; both arms are additive (an unknown tag is skipped — `restore_shell`,
  grid_layout.rs:410).

## Q5 — Licensing + the follow-up train

**Licences (all permissive — no copyleft, safe for Marley's GPL editor layer):** wry 0.55.1
(Apache-2.0/MIT), objc2-web-kit 0.3.2 (Zlib/Apache/MIT), cef 150.x (Apache-2.0/MIT + the separate binary
framework), raw-window-handle 0.6.2 (already in-tree). Already present transitively: objc2/block2/
objc2-app-kit/cocoa/core-foundation/core-graphics/**core-video**/metal/raw-window-handle. New for the
wry route: `wry` (+ its deps).

**The follow-up train (Phase E — handed to /spec):**
1. **✅ DONE — GO (M29 #402, 2026-08-06).** **Scratch proof-of-embed** — a wry child WKWebView over
   a gpui window; verified position/resize to a pane rect, the gpui overlay showing via the
   hide-shim (both mechanisms), AND the focus/IME handoff (the Q1 verdict block above has the
   detail + the weak-capture rule for production callbacks). *(env-gated; `wry` 0.56.0 entered in
   exactly one place — `crates/marley_webview_probe`.)*
2. **✅ SHIPPED (M29 #403, 2026-08-07) — `TabContent::Browser` + the bare-marker codec.** The variant
   is `TabContent::Browser(ContentId)` — id-bearing from birth, **no tag beside it**: cockpit needs
   one because three sections share a variant and its codec writes a per-instance key, whereas
   Browser v1 has one kind and a payload-less marker, so the variant discriminant already IS the tag.
   `rail_section`/`focus_label`/`key_context` arms landed pure; `TabLayout::Browser` writes a bare
   `B` (no `=`, no payload — ids and URLs never reach the wire, and restore rebuilds the tab and
   resolves the resident through one door); a 4th Browser＋ row (`SectionAction::OpenBrowser`) opens
   it, appended so item 0 stays Forge; the pane renders a placeholder (`browser.rs::placeholder_*`)
   that reads NO config, so it is total over a machine with no `.mcp.json`.
   **The lifecycle is the inverse of #400's cockpit, and that inversion is the slice's real content.**
   Browser DROPS on last close, so there is no anchor and `resolve_browser` hands the caller exactly
   one acquired view in *both* branches (the `resolve_open` contract, not
   `resolve_or_register_cockpit`'s) — a switch that consumed none must hand it back. Copying the
   cockpit's acquire-only-on-append shape would have left a view no close path could ever release;
   dropped was chosen now because the slice-3 webview is precisely the reapable payload, and pinning
   would have baked in a contract needing reversal here. Identity: ONE app-wide resident, following
   the single boot-time `.mcp.json` resolution (app.rs) that already feeds the forge client and the
   brain endpoint — one config, one origin, one instance; cross-project tabs are views of it.
   Verified on the live app end to end: open → the `B` on disk → quit → relaunch → the tab restored
   → close → the wire back to its pre-#403 shape. *(pure model + codec at cov/MSI 100; app.rs
   wiring masked. The split-cell grid `b` leaf and `PaneContent::Browser` stay deferred.)*
3. **✅ SHIPPED (M29 #405, 2026-08-08) — The Forge URL pane.** The wry child rides `center_bounds`
   per-frame (the #402 probe's capture+diff shape verbatim: rounded-i32 applied-key,
   hide-invalidates-key, Logical→Logical no scale math), loads the #404 origin RE-derived each
   boot (the #403 bare-`B` codec byte-unchanged), and pins nav via
   `marley_forge_client::same_web_origin` (one parser + the parsed-host re-check; wry denies
   handler-less `window.open` — verified in wry 0.56's source). **The z-order hide-shim is
   live:** ONE pure predicate (`browser::overlay_is_up`) over a FROZEN **27-state** inventory
   *(26 since #411 retired the forge sprint card's `forge_open` row)* —
   the P3.5 blind adversary added the 27th (the top-search results dropdown the design sweep had
   misfiled as chrome) — built by ONE snapshot fn whose rule is now named: **mirror the DRAW
   GATE, not the bare state** (two under-hide catches + one persistent OVER-hide: a rename draft
   with no live editor would have blanked the pane forever). Driven-verified live end to end:
   mount at rect / palette hide+reappear / tab-away+back / resize tracking / restart-remount
   (pixel captures in the pipeline notes). **Learnings carried to slice 4 (#406):** WebKit
   renders the sidecar's 401 JSON as a BLANK WHITE page — the error card owns honesty (and
   `browser_attach_error` is stored, unread, waiting for it); SUBFRAME navs are pinned too
   (wry 0.56's macOS delegate has no `isMainFrame` filter — cross-origin iframes would need
   one); ⌘-chords while the webview holds first responder remain unproven (v1 never steals
   focus programmatically; the return trip is click-out); the `status_flash` ~2s hide-blink is
   the accepted D-FLASH cost — flip only with driven evidence.
4. **✅ SHIPPED (M29 #406, 2026-08-08) — Nav chrome + lifecycle. THE TRAIN IS COMPLETE.** One pure
   machine (`browser_state.rs`) owns loading × error × reload: `(phase, event) → (phase,
   directives)`, exhaustive, sticky-first-fault-wins; the wry callbacks, the probe thread, the
   pump, the palette verb, and the render are decision-free adapters. **The error signal is
   OUT-OF-BAND by necessity** (the P2 wry-0.56-source sweep: `Started` fires on COMMIT, `didFail*`
   is unimplemented, `load_url`/`reload` return `Ok(())` unconditionally, eval queues pre-commit —
   a transport failure delivers NOTHING): a raw-TcpStream origin probe
   (born `marley_forge_client::probe_web_origin`; re-homed at #411 to `marley_app::browser_probe` +
   `tests/browser_probe_livewire.rs` when the client crate was deleted — livewire-tested,
   every-resolved-addr connect) races
   each load and delivers the typed fault fast; a pure 15s tick deadline (937 × 16ms pump frames)
   backstops probe-ok-but-hung; macOS's one real failure hook
   (`with_on_web_content_process_terminate_handler`) rides the same channel. An HTTP 4xx/5xx
   COMMITS a success-shaped body — the probe's status is the only truth channel that sees it, and
   it outranks. **Teardown settled INLINE by type necessity, retiring the "mirror the PTY reaper"
   prediction:** `wry::WebView` is `!Send` (off-thread drop cannot compile) and wry's own `Drop`
   calls `removeFromSuperview` — so holder-drop IS detach+drop, `reap_browser_holder` is the ONE
   teardown seam (`teardown_browser_life`: holder + machine + a generation bump), and
   INV-NO-ORPHAN-WEBVIEW is its totality, driven from every phase × close path. **Staleness is one
   law, two axes** (`signal_applies`): the webview GENERATION (bumped per teardown/re-attach)
   drops a dead life's callbacks; the probe ATTEMPT (bumped per spawn) drops a superseded probe's
   late verdict — the P3.5 race where a pre-retry probe faulted the post-retry attempt and
   stickiness swallowed the recovery. Retry re-derives the origin through the #404 seam
   (active-root-keyed); `effective_retry_base` keeps a transiently unreadable `.mcp.json` from
   masquerading as removed config (absent file → placeholder; underivable file → keep the current
   origin). "Browser: Reload" is static `CommandId(31)` through `cockpit_commands` →
   `action_for_command` → `dispatch_action`, guarded by `browser_tab_is_active` (total over the
   #395 empty workspace). The chrome states are the React-settled two-tier cards ("Loading…" +
   origin; `fault_header` names the fault class — "Browser · unreachable" / "web process exited" /
   "failed to embed" — over a `ClampedReason` line + the "↻ Retry" affordance); **NO header row**
   (D-OPEN-HEADER-ROW judged on the A/B pixels: a 28px row of user-config + a palette-owned verb
   under the top bar is decorative redundancy; Zed's chrome-less item-reload concurs). The #405
   learnings landed: `browser_attach_error` retired into `Error(AttachFailed)` (the card gained
   retry); blank-white dishonesty is structurally gone (the webview paints ONLY while Loaded —
   `phase_shows_webview` composes with the untouched #405 visibility axis, so the load axis can
   never read or reset the hide axis, REQ-006 by construction). Live-proven end to end: forge
   down → the error card ("connection refused — http://127.0.0.1:8080"); fixture origin up →
   ↻ Retry → the wry child painting at the pane rect; ⌘⇧P "Browser: Reload" through the palette
   overlay's hide/show; ⌘W → tab gone, no orphan child (captures in the pipeline notes). *(pure
   seams at cov/MSI 100 — the probe crate-side too, unmasked in a NEW `probe.rs` beside the
   excluded adapter; app.rs wiring masked per posture. Back/forward history stays
   deferred-not-forgotten; the split-cell `b` leaf and the CDP lane unchanged.)*
5. **✅ SHIPPED (M30 #410, 2026-08-09) — The forge identity RIPPED (the scrap-forge pivot).** The
   train's Forge-facing half un-ships: `RootView.forge_web_base` + the `.mcp.json` boot/Retry
   derivation deleted; the mount gate is `mount_plan(None)` and the Retry arm `retry_plan(None, …)`
   — the ONE seam a future URL feature fills (a `pub(crate)` pure planner referenced only by tests
   would trip dead_code, so the seams stay CALLED with the constant input —
   `L-claude-rip-keeps-pure-seams-called-constant-input-001`). The pinned-origin verdict moved
   in-app (`browser.rs::same_web_origin` + `web_origin_of`, byte-identical, tests moved, the #404
   parsed-host loopback re-check intact) so the coverage/mutation floor keeps enforcing it after
   TICKET-411 deletes `marley_forge_client`. `orchestration.web_url` (dead since #371) deleted —
   old settings files still parse. The placeholder is the standing production state ("The embedded
   browser awaits a page source." — React-settled, parity-paired). Machine, shim, probe edge, and
   codec byte-unchanged; the probe re-homes in #411.
6. **(separate pillar) the CDP agent-browser lane** — headless-Chromium + CDP→MCP tools; NOT this
   section. Placed here only to mark the fork.

## Architecture decision (recorded to forge; doc-local if the AD tool errs — see #388)
- **`AD-claude-embedded-browser-substrate-001`** — the Forge Browser pane uses **wry-as-child WKWebView**
  (light, native, feasible via gpui's exposed `raw-window-handle`), with the native-child **z-order vs
  gpui-overlays** collision as the load-bearing risk (mitigated by constraining/hiding overlays over the
  webview rect; de-risked by the proof-of-embed slice 1). **CEF off-screen-render** (composites cleanly
  via gpui's `paint_surface`, + CDP) is the named revisit if z-order fails or the agent-browser must
  share the surface. The **agent-browser CDP** lane is a SEPARATE headless-Chromium substrate (two lanes,
  not one) until a shared-surface need forces CEF unification. **(2026-08-11: that shared-surface need
  is now stated — see the Q2 trigger-fired note. The two-lane half of this AD is superseded intent; its
  substrate half, wry-as-child for the SHIPPED pane, stands until a Chromium replacement is proven.)** **De-risk complete (M29 #402): the
  slice-1 probe ran GO** — the z-order hide-shim holds under both mechanisms and focus/IME hands off
  cleanly; the decision stands confirmed, with the weak-capture callback rule recorded for the
  production slices.

## Related
[app_shell.md](app_shell.md) (the Browser section #385) · [roadmap.md](roadmap.md) (Phase E) ·
[orchestration-shell.md](orchestration-shell.md) (§8–9 the browser + hosted-seat) ·
[pane-composition-model.md](pane-composition-model.md) (#388 — Browser as a registry content kind).
