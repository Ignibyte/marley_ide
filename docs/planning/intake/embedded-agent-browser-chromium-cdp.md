---
status: intake
created: 2026-07-09
ticket: unassigned
pipeline_spec: unassigned
---

# Embedded agent-aware browser (Chromium + CDP) — a Marley pillar (future)

## What
An **embedded web browser inside Marley that the agent (brain) understands and controls** —
"cursor-like": it can read what's on the page as structured data, see it, and drive it. Not a
dumb webview — a surface where an agent can observe + act on live web content.

chad (2026-07-09): "one aspect we want to have is a web browser that is cursor-like where it
understands what its looking at… i dont mind the 150mb part."

## The deciding constraint → Chromium, not a system webview
"Understands what it's looking at" is a requirement for the **Chrome DevTools Protocol (CDP)**,
which means a real **Chromium**, NOT the `wry`/WKWebView route:
- **WKWebView (mac system webview) has no CDP** — only inject-JS DOM scraping + a snapshot: a weak,
  DIY imitation, no accessibility-tree-as-data, clunky input, and it's WebKit ≠ Chrome. WebView2
  (Windows) does expose CDP, but mac has no equivalent → the good path would be Windows-only.
  Rejected for a consistent, cross-platform "understand + control" surface.
- **CDP over Chromium** gives the whole surface in one protocol:
  `Accessibility.getFullAXTree` + `DOMSnapshot` (the page as labeled structured data — the
  "understanding"), `Page.captureScreenshot` (pixels for a vision model), `Runtime.evaluate`
  (JS in the page), `Input.dispatchMouse/KeyEvent` + `Page.navigate` (control). This is exactly
  what Cursor's browser / Claude-in-Chrome / Playwright use — proven family.
- 150MB (bundled Chromium / CEF) is accepted (chad).

## Recommended architecture — screencast-into-a-gpui-texture (spike this first)
Marley is a gpui GPU-surface app; a native child web view fights the compositor (z-order vs our
docks/overlays, clipping, focus). The clean route SIDESTEPS that:
- **Run headless Chromium, drive it over CDP, paint `Page.startScreencast` JPEG frames into a gpui
  texture; forward gpui mouse/key events back via `Input.dispatch`.** The browser is then *pixels in
  the gpui scene*, not a native overlay — compositing problem gone. Build the back/forward/URL chrome
  natively in Marley (calling `Page.navigate`).
- **One connection does BOTH display and observe/control** — because we render via CDP, the agent's
  "understanding" is the same session. Unifies render + agent-control.
- Tradeoff: screencast is frame-streamed (great for browsing + agent work, not 120fps video). Fine.
- Alternative: **CEF in-process** as a real child view — better fidelity, but reintroduces the
  child-view compositing tax. Spike the screencast route first; fall back to CEF if fidelity demands.

## This unifies the forge/agent-data render question
If Chromium is in-process anyway, the earlier "render forge/agent DATA as web vs native gpui?"
question collapses: we'd have a first-class web runtime in the app, so the **cockpit dashboards can
ride the same surface**. One embedded Chromium = the agent-controllable browser AND the render target
for forge/agent panels. (The `marley_forge_client` data seam is already pure + tested and doesn't
change either way — this is a presentation decision on top of it.)

## The architecture it crystallizes into (chad 2026-07-09)
chad's framing: "Marley is also a web server that we could lock down to only the browser, and the
data — forge / brains / wikis — is built in HTML/CSS, much easier to manage than dynamic layouts in
Rust." Confirmed, with two sharpenings:

- **Serve the web content over an in-process CUSTOM SCHEME, not a network server.** Register a
  `marley://forge` / `marley://wiki/…` scheme the embedded Chromium resolves via a Rust callback —
  NO socket, NO port, nothing bindable from another local process. This is the strongest "locked to
  only the browser": there is no server to lock down. A literal loopback HTTP server is a real
  surface (localhost is reachable by any local process — cf. the `marley_forge_client` localhost +
  never-logged-bearer lesson); acceptable only with a per-launch nonce, and only if we want standard
  web tooling (fetch/websockets/an SPA). Prefer the custom scheme. Either way it's a THIN ADAPTER
  over the already-pure/gate-tested Rust data layer, exposing it as JSON/HTML to the page.
- **HTML/CSS is the right tool for the DOCUMENT/DATA surfaces** — wikis, knowledge bases, ticket
  boards, a brain's state view, dashboards, markdown, tables, reflow. These are painful in gpui
  (hand-laid elements, no CSS/mature flexbox/markdown engine) and native in HTML. Decisively easier
  to manage — correct.

**The guardrail: the TERMINAL stays native gpui.** It's the moat (GPU-rendered, monospace-cell-accurate,
low-latency — the reason Marley beats an Electron terminal). The clean split: **terminal + core shell
chrome = native; document/data surfaces = HTML.** A hybrid, not all-web.

**Free bridge:** export `marley_ui_components::ThemeColors` tokens as CSS variables
(`:root { --background: …; --surface: … }`) so the web content inherits the dark theme (#194) from
ONE source of truth — native + web in lockstep.

**Testing boundary (honest):** the gate stops at the web edge. The Rust data layer (forge client,
scheme adapter, serializers) stays pure + MSI-100; the HTML/CSS/JS presentation gets web-style /
driven-capture testing (like the Artifacts). Not worse — different; don't claim MSI 100 covers a page.

**The whole shape:** a native GPU shell (terminal + chrome) + an in-process web layer for
documents/data + an agent-aware Chromium, unified by CDP (one connection RENDERS the web content AND
gives the agent its eyes). Local-first, no cloud dependency; the web/native line falls exactly on
"document vs terminal."

## Why / how it composes
- **The fleet plane claimed this pillar's first two concrete tenants (2026-07-20 —
  `docs/marley_architecture/orchestration-shell.md` §8–9):** ① the **project's Forge web URL as a live
  pane** (per-project config; the mission-control hypermedia surface renders there — "one HTML dialect,
  two transports": remote HTTPS/SSE in this pane, `marley://` for local surfaces); ② the **CDP→MCP
  browser tool family** (`browser.navigate/read_page/screenshot/click/type/…`) granted to a **hosted
  agent seat** — Marley's own Claude/Codex session, driving Marley over loopback MCP with CDP eyes.
  Discipline recorded there: *don't scrape what you can query* (structured Forge data via forge MCP;
  the browser is the human's rich view + the any-web escape hatch). Depends on `marley_mcp` existing.
- It's the **browser twin of [[brain-agent-session-supervision]]**: that intake is launch/control/
  **observe** an agent's TERMINAL (the Block stream); this is launch/control/observe an agent's
  BROWSER (CDP). **CDP is to the browser what the terminal Block model is to the terminal.** Same
  three-capability seam, new surface.
- Composes with **[[remote-connection-seam]]** (M5) — a remote agent's browser driven the same way.
- Licensing is clean-room-safe: Chromium / CEF / CDP are all BSD — no AGPL/Warp concern (§20).

## Constraints / risks (design in from day one)
- **Security surface** — an agent-driven browser with `Runtime.evaluate` + arbitrary `Page.navigate`
  is real attack surface. Needs a navigation/permission policy (allowlist? confirm-before-act?) and
  careful credential/cookie handling — NOT an open eval-to-anywhere. Highest-priority design item.
- **Scale** — this is a PILLAR (its own milestone, M3/M5-era), not an M12.2 polish ticket. Likely a
  new `marley_browser` crate (the CDP client + screencast decode + input-forward, pure-testable
  protocol seam) + an app.rs render shim (the texture + native chrome).
- Bundle: +Chromium/CEF (~150MB) — accepted; keep it a build/feature seam so a no-browser build stays lean.

## Promotion
Candidate, not a pipeline doc yet. Promote via `/work` when the agent-browser surface is scheduled
(M3 agent orchestration / M5 remote). At that point: **spike the headless-Chromium + CDP-screencast
→ gpui-texture route** (throwaway, like the M0 gpui+alacritty spike) to validate feel + focus +
latency BEFORE committing; design the security policy alongside. Decision to record as a forge AD
(`AD-…-embedded-agent-browser-chromium-cdp-…`).

---

# AMENDMENT (2026-08-11) — the two-lane split dies; three new pillars

*(Chad, this session. The pre-amendment body above stands as the 2026-07-09 design record. Note in
passing: its `marley_forge_client` references are pre-pivot history — that crate was deleted at #411
and the surviving seams moved in-app (`browser.rs::same_web_origin`); the data-layer ARGUMENT it
makes is unaffected. Read those as tombstones, per the #410 pivot.)*

## The stated goal, in Chad's framing

Two-fold:

1. **The agent drives the browser** to verify its own work, debug, and test.
2. **The agent sees what the user is looking at and describes it** — with a layer on top to *draw*
   on the page (circle things, annotate), plus a "record this" that captures the actions performed
   so the user can show the agent the issue they are hitting.

Added in the same conversation: **an element picker** — the user clicks the thing that is broken
(the example was a dead accordion), and the agent receives the exact element, durably enough to
re-find and re-drive it later.

## DECISION — one Chromium. The two-substrate split is retired.

`embedded-browser-model.md` §Q2 split the world into a **WKWebView display pane** (no CDP needed —
it just shows a page) and a **separate headless-Chromium agent lane** (CDP), and named its own
collapse trigger: *"if the agent browser must be VISIBLE in a pane AND share the surface, collapse
both onto one Chromium."*

**Goal 2 fires that trigger, definitionally.** An agent that describes what the user is looking at
must be attached to the SAME browser the user is looking at — same session, same cookies, same
scroll offset, same DOM. Two engines cannot satisfy that; the failure mode is an agent confidently
describing a page the user is not on. So §Q2's two lanes collapse to one, and this intake's own
2026-07-09 recommendation (screencast-into-a-gpui-texture, "one connection does BOTH display and
observe/control") is confirmed as the target rather than sitting in tension with the Q2 decision.

**Consequence for the shipped pane.** The wry/WKWebView browser (#402→#406) keeps its place until a
Chromium replacement is proven — it costs ~nothing today, since #410 left it with no page source and
the placeholder as its standing production state. On replacement it retires, and the **real payoff
is the hide-shim**: `browser::overlay_is_up`'s frozen 26-state inventory must mirror the draw gate of
every overlay in the app, which couples all future overlay work to the browser feature (#405's
inspect already caught two under-hides and a permanent over-hide). The texture route deletes that
coupling by construction — the page becomes something gpui draws, so gpui's overlays layer above it
for free. **Do not rip the WKWebView pane preemptively**; rip it when the replacement lands.

**Cost re-priced.** Forwarding input through `Input.dispatch*` is real work, and **IME/composition is
the hardest unproven piece** — #402 proved WKWebView hands IME off cleanly in both directions, and
that capability is rebuilt from scratch here. But being in the input path is also what makes the
recording pillar nearly free (below). Price it as a spike gate, not a footnote.

## Pillar A — the element picker (build this FIRST; highest value per unit of work)

The dominant failure mode in agent-driven web debugging is **referent ambiguity**: "the accordion
doesn't work" forces the agent to guess which of N clickable things is meant, and a wrong guess
yields a confident wrong diagnosis. The picker collapses the search problem to zero.

**Mechanism: CDP-native, no injected JS.** `Overlay.setInspectMode` + the `Overlay.inspectNodeRequested`
event IS DevTools' element picker, hover highlight included; the Overlay domain draws into compositor
output, so it appears in screencast frames for free. (`DOM.getNodeForLocation` is the manual route if
a Marley-branded hover highlight is wanted instead of DevTools' blue/orange box model.)

**Capture a DURABLE BUNDLE, not a node handle.** `backendNodeId` dies on navigation — and "find it
automatically later" is the whole requirement. Per pick, capture:

- **Ranked resilient locators** — `data-testid` → ARIA role + accessible name → stable `id` → text
  content → scoped CSS path only as last resort. Store several, ranked; brittle paths are the
  known killer.
- **The AX node** — role, accessible name, states. For the accordion case `aria-expanded` IS the bug
  surface.
- **Attached listeners** (`DOMDebugger.getEventListeners`) — "is a click handler even bound?"
  resolves a large fraction of *it doesn't work* reports immediately.
- **The listener's source location** — script URL + line; through source maps, a real file.
- **Interaction-blocking computed styles** — `pointer-events`, `z-index`, `overflow`, `visibility`.
  A dead accordion is very often an invisible overlay eating the click, or a `max-height`
  transition that never runs.
- The element's frame/screenshot crop.

**The Marley-specific payoff — the chain no generic browser tool has.** picked element → listener
source location → source map → file under the active project root → **open it in an editor pane at
the line**. Marley already owns the editor, the project root, and LSP. This is the feature's
signature move.

**Both goals are the same machinery.** The durable locator is also the agent's re-run handle:
after a fix, re-find → click → assert `aria-expanded` flips. Goal 1 (verify my work) and goal 2
(understand what the user means) are one mechanism pointed in opposite directions.

**Two details that decide whether it feels good or janky:**
- **Walk up to the nearest interactive ancestor.** People click the `<span>` label inside the
  `<button>`; resolve to the interactive node (role/listener-bearing) and keep the raw node as a
  fallback.
- **Shadow DOM.** Most component libraries put the real control in a shadow root
  (`includeUserAgentShadowDOM`, shadow-piercing locators). A naive CSS selector finds nothing.

## Pillar B — the annotation layer (render native, anchor via CDP)

**Recommendation: do NOT build the drawing as an injected JS overlay library** (the shape originally
imagined). Render annotations as gpui elements above the browser texture. Why, for these use cases
specifically:

- The **DOM and AX tree stay pristine** — injected annotation nodes would pollute every
  `getFullAXTree`/`DOMSnapshot` the agent reads, forcing you to filter your own markup back out.
- No CSP fight, no z-index war with page CSS, no `all: initial` defensiveness.
- Works identically on a single-`<canvas>` app or a PDF, where a DOM overlay has nothing to attach to.
- **Clean frame AND annotated composite for free** — the agent reads the clean texture, the human
  artifact gets the circles baked in. With injection that separation is manual.

**Scroll-tracking without injection (the mechanism that makes it work).** Store annotations in
**page coordinates**; every screencast frame carries `scrollOffsetTop`, `pageScaleFactor`, and
`deviceWidth/Height` metadata, so viewport placement is pure arithmetic per frame — no CDP round
trip, no lag. That coordinate mapping is a **pure, 100%-coverable seam**; only the CDP adapter is
masked. Re-resolve boxes only on real reflow, not continuously.

**Three layers:** *anchoring* (`DOM.getNodeForLocation` + `DOM.getBoxModel` → node ref + page rect) ·
*placement* (per-frame arithmetic from screencast metadata) · *rendering* (gpui, in Marley's design
system).

**The one legitimate injection** is anchoring TELEMETRY, never drawing: a tiny script in an
**isolated world** (`Page.createIsolatedWorld`, so page JS can neither see nor interfere with it)
running `ResizeObserver`/`MutationObserver` to push "element moved" events, so reflow re-resolution
is event-driven instead of polled.

**Unification:** a picked element (Pillar A) *is* an anchored annotation. Same anchoring layer, same
page-coordinate storage, same placement math — one mechanism, two features.

## Pillar C — "record this" (a structured trace, not a video)

A screen recording is opaque to the agent, which defeats the purpose. The primitive is a
**timestamped structured trace**; per step: the input event, the frame, console output, network
activity, any thrown exception, and an AX snapshot at key moments. An artifact a human can scrub
AND an agent can read to say *"the request 404'd two actions before the blank screen."*

**The action log is nearly free.** In the screencast architecture Marley forwards every mouse and
key event through `Input.dispatch*` — so Marley is already in the path of every interaction. Log
what you are already forwarding; no `rrweb`-style injection needed for the action stream.

**Make it RETROACTIVE — a flight recorder.** Keep a rolling ring buffer (~30–60s) of frames +
input + console + network at all times; "record this" *saves what already happened*. Bugs are
noticed after they occur, and the variant where the user must predict the bug and hit record first
is the variant nobody uses.

**Composed with the picker:** click the broken accordion in record mode → the trace holds the
dispatched click, whether a listener fired, the console error, and `aria-expanded` unchanged
before/after. A complete bug report from one gesture.

## Prior art (§20 — all permissive; reading these is ADOPTION, not derivation)

- **Playwright** (Apache-2.0) — its **trace viewer** is the closest existing artifact to Pillar C;
  read its trace format. Its **locator resilience** strategy is the model for Pillar A's ranking,
  and it already solves shadow-piercing.
- **rrweb** (MIT) — the injected session-recording library originally imagined for Pillars B/C.
  Worth reading, but argued AGAINST here: injection contaminates the very snapshots the agent reads.
- **CDP itself** is a published protocol spec (BSD Chromium) — the `Overlay`, `Accessibility`,
  `DOMDebugger`, and `DOMSnapshot` domains are the working surface.
- No Warp/Zed analog; no copyleft source involved.

## What the spike must now answer (additions to the Promotion gate above)

The original gate was feel + focus + latency. Add:

1. **IME / composition over `Input.dispatch*`** — dead keys, CJK, the composition return trip. The
   one capability the WKWebView route already proved and this route must rebuild. Highest risk.
2. **Screencast latency at interactive speed** — acceptable for a human-driven pane, not merely for
   agent work (the 2026-07-09 body concedes it is frame-streamed).
3. **`Overlay` domain visibility in screencast output** — assumed above; verify inspect-mode
   highlight actually reaches the frames.
4. **Cross-origin iframes (OOPIF)** — separate processes, so `getFullAXTree` on the main frame
   silently omits them and `Target`-domain handling is required. A smaller form of this already bit
   #405 (wry's macOS delegate had no `isMainFrame` filter).
5. **Coordinate-space composition** — device pixel ratio × page zoom × screencast scaling. Take
   these from frame metadata; do NOT hand-roll (the #402 Logical→Logical convenience does not
   transfer).

## Testing posture (fits the §0 floors)

The pure seams are substantial and should carry cov/MSI 100: the coordinate/placement math, the
locator ranking + resilience ordering, the trace/ring-buffer model, and the annotation state model
(mirroring how `browser_state.rs` is a pure machine with decision-free adapters). The CDP transport,
screencast decode, and gpui render stay masked adapters. The 2026-07-09 "gate stops at the web edge"
note still holds for any HTML surface — but note these pillars deliberately keep the interesting
logic on the RUST side, which is a reason to prefer them over in-page JS.

## Open questions

- **Security policy is still the highest-priority design item** (unchanged from the original body,
  and now larger): an element picker + `Runtime.evaluate` + arbitrary navigation + a trace capturing
  network activity is a real surface. Traces will capture **credentials and tokens** in headers and
  page content — redaction policy must be designed in, not retrofitted (the #370/#375
  bearer-never-logged lineage applies).
- Does the annotation set persist across sessions, and if so at what layer (a side table keyed by
  origin + locator)? The #403 codec lesson applies: persist a marker and re-derive, do not put URLs
  or arbitrary payloads on the wire.
- Should the picker feed the agent automatically, or stage picks for the user to confirm/caption
  before sending? (Leaning staged — a pick is often the START of a sentence, not the whole message.)
