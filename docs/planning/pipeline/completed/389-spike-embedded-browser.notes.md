# 389-spike-embedded-browser — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** chad's refinement — the third section is Browser; Forge opens IN the browser.
  Roadmap Phase E's opening spike, now unblocked (fleet Layers 1–2 shipped M23–M25).
- **Sprint:** #37 M26; forge #389 `88f9c375-191d-4e52-892e-9b84309e9a4f`.
- **Ground truth (2026-07-22 Explore map):** ZERO substrate — no webview dep in any Cargo.toml,
  nothing in Cargo.lock (only `block2`, the objc FFI runtime), gpui 0.2.2 has no webview element;
  the sole grep hit is a doc comment (crates/editor/src/ime.rs:23). The native Forge surface to
  eventually succeed: `cockpit_body`'s Forge arm app.rs:5444-5462 over `self.forge_sprint`
  (`marley_forge_client` is serde-only — no HTTP). The fleet rail (fleet_rail.rs +
  `fleet_rail_body` app.rs:1001, right dock) is NOT part of any retirement.
- **The central tension for P2/P3:** gpui self-composites its whole window → a webview must be a
  native CHILD view overlaid on a pane rect (position/resize/input/z-order sync — palette and
  menu overlays must never fight the webview), OR CEF's off-screen rendering into a texture
  (sidesteps overlay, costs real complexity). WKWebView has no CDP → the pane lane and the
  agent-browser lane may be different substrates (question 2).
- **Security frame (inherited, D3):** loopback wall + bearer-never-logged (#370/#375/#381);
  URL derivation reuses `mcp_config::mcp_json_path` (#381) keyed on the restored active root
  (PR-claude-boot-decisions-key-the-restored-active-root-001); bearer never in URL or page JS;
  pinned origin v1.
- **Prior-art sweep:** recorded in spec — the key result is the NEGATIVE (no owner in-tree);
  published docs of wry/WKWebView/CEF + gpui source (adoption) are the study set.
- **Open items:** whether the scratch proof-of-embed is feasible in-session (P2 env check);
  the codec escape shape for a URL leaf.
- **AAR:** `e3a9752a-94b7-4c90-81e2-4607a21d6c45`.

## Phase 2 — Design (2026-07-22)

### Approach (SPIKE — deliverable is a decision DOC, no code)
**Doc:** `docs/marley_architecture/embedded-browser-model.md` (new; cross-linked from app_shell.md +
roadmap Phase E). Structure = the 5 spec questions. Study: the webview research agent (a0944bb4, running)
— gpui's macOS window structure (self-composited Metal NSView? does it expose a window handle /
`raw-window-handle`?), wry's `build_as_child` capability, objc2-web-kit, CEF off-screen-render, the CDP
confirm, and which FFI deps are already in the lock file.

### The crux (question 1) — my LEAN (to be confirmed by the agent facts + argued in the doc)
gpui **self-composites** its whole scene into one Metal-backed `NSView`. A webview is a native surface,
so two embed shapes:
- **(A) Native child `NSView` (wry `build_as_child` → WKWebView on macOS).** Light, macOS-native,
  matches "Forge is a page." **The hard constraint = z-order:** a native child view composites ABOVE
  gpui's Metal layer, so gpui's OWN overlays (the ⌘⇧P palette, context menus, the find bar) drawn by
  gpui would render BEHIND the webview — a palette over a Browser pane would be hidden. Mitigations:
  constrain the webview to its pane rect + gate gpui overlays to never overlap it (or hide the webview
  while an overlay is up); a real but bounded shim problem.
- **(B) CEF off-screen-render (paint the page into a pixel buffer → composite as a normal gpui
  element).** Sidesteps z-order entirely (the page is just a texture gpui draws, so overlays layer
  above naturally) AND brings real Chromium + CDP. Cost: heavy (bundles Chromium ~100 MB+; a
  render-pump; input-injection plumbing).

**Lean: (A) wry-as-child WKWebView for the Forge pane v1** (light, native, no Chromium bundle; the
Forge web UI is a trusted first-party page, so the z-order overlay-collision is a manageable shim gate,
not a blocker). **The CDP fork (question 2): TWO substrates** — WKWebView for the Forge PANE (needs no
CDP), a SEPARATE headless-Chromium lane (CEF or a driven external Chrome) for the AGENT browser (which
needs CDP→MCP). Revisit trigger: if the agent-browser lane wants to SHARE the pane surface, re-evaluate
CEF-OSR for both (its z-order win + CDP would then justify the weight).

### Forge-in-browser (question 3)
URL = derive the Forge web-UI base from the SAME `mcp_config::mcp_json_path` (#381) active-root
resolution — one config, two consumers (the MCP client + the browser URL);
PR-claude-boot-decisions-key-the-restored-active-root-001 applies (active root, never launch cwd). Auth:
the loopback wall + bearer-never-logged (#370/#375/#381) stand; **the bearer is NEVER in the URL or
exposed to page JS** — the web UI does its own auth. v1 nav: **pinned origin** (the Forge base only, no
arbitrary browsing). Coexistence: the native forge cockpit (`cockpit_body` Forge arm, app.rs:5444) keeps
working; retirement criteria named (feature-parity of the web UI + chad's call); the native FLEET rail
(fleet_rail_body, app.rs:1001) is NEVER retired ("desktop for hands, web for intents").

### TabContent::Browser + codec (question 4)
A `TabContent::Browser(BrowserState{ url, … })` variant + a `TabLayout::Browser` arm + a grid leaf. The
leaf must survive the framing bytes: grid reserves `, : = \x1f`, shell reserves `\t \n \r` — and a URL
contains `:` and `/` and often `=`. So a raw `b=<url>` leaf BREAKS the grid framing. Decision: **the
leaf stores an INDIRECTION, not the URL** — since the Browser section's v1 content is the project's
Forge URL (derived from `.mcp.json`, not user-typed), the leaf is a bare marker `b` (like `f`/`g`) and
the URL is RE-DERIVED on restore from the active root (the #381 seam). A general user-URL browser (later)
gets percent-encoding or a side-table id — deferred. Back-compat: old layouts (no `b`) restore unchanged.
This maps `Browser`→the `RailSection::Browser` #385 already ships (the section's real content lands here).

### File manifest
Docs only (ships ZERO `crates/*` code): `docs/marley_architecture/embedded-browser-model.md` (new) +
a cross-ref line in `app_shell.md` + a roadmap Phase-E cross-ref. Plus a forge AD (the substrate +
2-lane decision) + the emitted follow-up train (to /spec).

### Regression Test Plan (a SPIKE — verify = doc review + gate)
| # | Check | Proves |
|---|---|---|
| V1 | The doc names the chosen substrate (wry-as-child WKWebView) with the z-order constraint + mitigation, and the embed-feasibility evidence (agent facts / platform docs; a scratch proof scheduled as train slice 1 if in-session build isn't feasible) | REQ-001 |
| V2 | The doc records the CDP fork decision (WKWebView pane + separate headless-Chromium agent lane) with its revisit trigger | REQ-002 |
| V3 | The doc carries the Forge-in-browser plan (URL via #381, bearer-never-in-URL auth, pinned-origin nav, native-cockpit coexistence/retirement) | REQ-003 |
| V4 | The doc carries the `TabContent::Browser` + codec plan (the framing-safe `b`-marker + re-derive-on-restore, back-compat) | REQ-004 |
| V5 | A forge AD recorded (or doc-local if the tool errs, per #388), candidate licenses/versions listed, the follow-up train emitted; `crates/` diff EMPTY | REQ-005 |
| G | `scripts/gates.sh --fast` green | §0 |

Uncoverable by unit test: N/A — spike ships no code; verify = doc review + empty-crates-diff + `--fast`.
The scratch proof-of-embed is env-gated (a wry-in-gpui build needs adding wry to a scratch branch — NOT
staged per D2); if not feasible in-session, the doc cites the platform constraint + schedules the proof
as train slice 1 (REQ-001's escape hatch).

### Risks / decisions
- **D-TWO-SUBSTRATES:** WKWebView (pane) + headless-Chromium (agent CDP) — the honest answer to the CDP
  fork; a single CEF-OSR substrate is the revisit if the lanes must share a surface.
- **D-ZORDER-IS-THE-RISK:** the native-child-view z-order-vs-gpui-overlays collision is the load-bearing
  unknown; the doc must state the mitigation (constrain/hide overlays over the webview rect) + schedule
  the proof-of-embed as slice 1 to de-risk it before committing the section.
- **D-CODEC-INDIRECTION:** the grid leaf is a bare `b` marker (URL re-derived from `.mcp.json`), not a
  raw URL — avoids the framing-byte collision; a user-URL browser's encoding is deferred.
- **D-SECURITY-INHERITED:** loopback wall, bearer-never-in-URL/JS, pinned-origin — inherited, not
  invented (D3).

## Phase 3 — Implement (2026-07-22)
The spike's "implementation" is the decision DOC. Written:
- **`docs/marley_architecture/embedded-browser-model.md`** (new) — decision-complete, all 5 questions.
  Substrate DECIDED (webview agent a0944bb4 confirmed the facts): **wry-as-child WKWebView for v1** —
  FEASIBLE because gpui exposes its NSView via `raw-window-handle` 0.6.2 (the exact `HasWindowHandle`
  wry's `build_as_child` needs) + `set_bounds` positions the child; the load-bearing risk is **z-order**
  (the child composites above gpui's Metal scene → gpui overlays hidden), mitigated by constrain/hide +
  de-risked by the proof-of-embed slice 1. **CEF off-screen-render** is the named revisit (composites
  cleanly via the ALREADY-PRESENT gpui `paint_surface(CVPixelBuffer)` + brings CDP). **CDP fork = two
  substrates** (WKWebView pane + separate headless-Chromium agent lane; CEF unification the revisit).
  Forge-in-browser (URL via #381, bearer-never-in-URL, pinned-origin, native-cockpit coexistence),
  `TabContent::Browser` + the framing-safe `b`-marker codec (URL re-derived, never in the leaf), licenses
  (all permissive) + the 5-step Phase-E train — all specified.
- **`app_shell.md`** + **`roadmap.md`** (Phase E) — cross-ref notes added.
- **AD:** `AD-claude-embedded-browser-substrate-001` captured in the doc's "Architecture decision"
  section. **The forge `architecture-decision-record` tool erred again** (invalid args: missing field
  `decision`, well-formed payload — the same persistent tool bug that hit #388) → doc-local (§19).
- **Zero `crates/*` code** — docs only (D2 / REQ-005 by construction). Notable NEW fact from the study:
  gpui already ships `paint_surface` + `core-video` in-tree, so the CEF-OSR alternative has a native
  compositing target today — strengthening it as the z-order revisit.

## Inspect (Phase 3.5 — 2026-07-22)
**Method:** 2 independent adversarial critics (general-purpose) over the doc's confident sentences, in
parallel — (1) gpui/webview feasibility, (2) requirements + security + codec — PLUS my own foreground
verification of every load-bearing fact against the real sources (gpui 0.2.2 in the cargo registry;
`app.rs`/`tabs.rs`/`grid_layout.rs`/`mcp_config.rs`).

**Cross-confirmed TRUE (no correctness finding — the decisions all stand):** gpui 0.2.2 exposes
`impl HasWindowHandle for MacWindow` → `RawWindowHandle::AppKit` at `window.rs:1548` (exact) + `Window`
delegates at `window.rs:4845`; `raw-window-handle 0.6.2` in-lock (matches wry); `pub fn paint_surface(…,
CVPixelBuffer)` at `window.rs:3181` + `core-video 0.4.3` in Cargo.lock (the CEF-OSR fallback is real);
`GPUIView` = an NSView subclass (`window.rs:111`); `mcp_config::mcp_json_path` active-root resolution
(#381) real, keyed on active root; `RailSection::Browser` + `Cockpit→Browser` (tabs.rs:82); `forge_sprint`
cockpit arm (~app.rs:5451) + `fleet_rail_body` (app.rs:1006); the `f`/`g` bare pane markers + the
`breaks_grid_framing`/`breaks_framing` reserved-byte sets. Both critics returned **no HIGH, no
MED-correctness** finding.

**Findings — 8 confirmed-real, all FIXED in `embedded-browser-model.md`:**
| # | sev | finding | fix |
|---|-----|---------|-----|
| F1 | MED | **Codec-layer conflation** (critic 2): v1 Browser is a whole TAB → the SHELL codec (a `TabLayout::Browser` tag like Cockpit's `C=`, where only `\t\n\r\x1f` separate entries → a URL is framing-SAFE), but the doc reasoned via the GRID-leaf codec (`: =` breaking, `f`/`g`-leaf analogy) which belongs to the *deferred split-cell*. Decision (bare marker + re-derive) is safe either way — the *exposition* was wrong. | Q4 rewritten into a two-layer note (shell-tab `B` for v1; grid-leaf `b` for the deferred cell); Q5 slice-2 updated to match. |
| F2 | LOW | z-order mitigation oversold "constrain overlays to never overlap" (palette/menus are centered → uncontrollable) (critic 1). | Made **hide-webview-on-overlay** the primary mitigation; "constrain" demoted to a partial secondary measure (Q1 table + Decision). |
| F3 | LOW | Un-listed sub-risk: **keyboard/first-responder + IME** hand-off between the WKWebView child and GPUIView (`makeFirstResponder(native_view)`, window.rs:776) (critic 1). | Added as go/no-go check **(3)** in the slice-1 proof-of-embed (Q1 Decision). |
| F4 | LOW-MED | "`RailSection::Browser` ships as an **empty skeleton**" is wrong — it ships populated (the cockpit/Forge tabs' rail home, tabs.rs:82); only the *webview content* is absent (both critics). | Reworded "The target". |
| F5 | LOW-MED | The **endpoint→web-UI-base** step is unspecified — `.mcp.json` yields the MCP JSON-RPC endpoint (`…/mcp`), not the web base (critic 2). Also "two consumers" undercounts (MCP client + fleet brain already share it). | Q3: web base = the endpoint's **origin** (or an explicit mapping), a slice-3 input; corrected to **three consumers** (app.rs:2207/2220). |
| F6 | LOW | "#384 misconfigured arm" **mis-attributed** — #384 is a QUEUED spec about the **fleet** rail header, not the Browser section (critic 2). | Q3 reframed the no-config behavior as *this doc's own* decision (a sibling of #384's pattern), grounded in mcp_config.rs:23. |
| F7 | LOW | `/` listed as a framing byte — it is **not** reserved; only `:` and `=` break grid framing (critic 2). | Dropped `/` (Q4). |
| F8 | LOW | "Forge web UI does its own auth" is an unverified **assumption** — if it shares the bearer, a pinned no-bearer pane is unauthenticated (functional gap, not a leak) (critic 2). | Q3 flags it as a slice-3 validation (never via bearer-in-URL). |
| F9 | LOW | Line-number drift (pre-disclaimed) — 3180, 5444, 1001 (both critics). | Nudged to exact: 3181 / 5451 / 1006. |

**Rejected (1):** critic 2's LOW "empty-crates-diff scope" (working tree carries unrelated M23 crates
changes) — **REJECTED**: `git status --porcelain -- crates/` is EMPTY right now (verified twice this
phase); those M23 (#368/#369) changes were already committed earlier this session. REQ-005's
empty-crates-diff (already scoped to `crates/`) holds — no fix needed.

**Provenance:** all research from gpui (Apache-2.0, adoption) + published permissive-crate docs; §20 wall
intact (no Warp/Zed source read). No `failure-record` warranted (no code bug — a spike doc); the
codec-conflation lesson is a doc-precision class, captured here.

## Phase 4 — Validate (2026-07-22)
A docs-only spike → **the gate is the test** (no `.rs`, no unit tests to add); the deliverable is a
markdown design doc (not a render/input path) → **no driven capture** (N/A — not a UI-affecting change).
- **REQ checklist V1–V5 — ALL PASS** (doc review against the post-inspect `embedded-browser-model.md`):
  V1/REQ-001 substrate (A) + CEF-OSR alt + feasibility evidence (verified gpui facts) + slice-1 proof w/
  go/no-go ✓ · V2/REQ-002 two-substrate CDP fork + concrete revisit trigger ✓ · V3/REQ-003 #381 URL
  (3 consumers) + bearer-never-in-URL/JS + pinned-origin + native-cockpit coexistence/retirement ✓ ·
  V4/REQ-004 `TabContent::Browser` + the two-layer framing-safe codec (shell `B` / deferred grid `b`,
  bare-marker + re-derive) + back-compat ✓ · V5/REQ-005 AD doc-local (forge tool errs) + licenses/versions
  + 5-slice train + **empty crates diff** ✓.
- **REQ-005 empty-crates-diff:** `git status --porcelain -- crates/` → **EMPTY** (verified; the spike wall
  holds — the whole change set is 7 docs files).
- **Gate:** `scripts/gates.sh --fast` → **GATE GREEN [fast], 11/11, 0 failed**. Notable greens: gate:3
  tests (nextest + doctests ran — the whole suite is green), gate:14 docs (rustdoc -D warnings + doc-todos
  + **brand-scrub** — the new doc carries no Warp/Zed brand leak), gate:10 gitleaks (no secrets),
  gate:12 no-suppressions, gate:13 source-bans. gate:4/5/6/15 (coverage+mutation+miri+visual) SKIP under
  `--fast` — N/A for a zero-`.rs` change; the receipt comes from the next `.rs`-bearing commit, and this
  docs-only commit is not gated by the commit-gate hook.
- **Pre-existing failures:** none in scope.
