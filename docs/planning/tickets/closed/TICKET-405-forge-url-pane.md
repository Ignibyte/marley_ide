# TICKET-405 — The Forge URL pane — the wry child wired to the pane rect + the z-order shim (the #389 train slice-3)

- **Forge ticket:** #405 425c8332-8949-41bc-b252-008c922d85a0 (feature, M29)
- **Owner:** unclaimed
- **AAR:** deferred — opened when /work promotes this to active
- **Pipeline doc:** ../../pipeline/completed/405-forge-url-pane.spec.md
- **Source ticket:** sprint M29 "The Embedded Browser" (forge sprint #40, 6dff19c6-fb61-472b-89c3-98d3215a489b)
- **Status:** closed (2026-08-08 — shipped through the full pipeline; gate GREEN [diff] 15/15; forge `ticket-close` deferred, MCP unreachable in the delivering session)

## Summary
The payoff of the #389 train: **Forge opens in the browser.** The wry child WKWebView attaches to the
gpui window's `raw-window-handle` at the Browser pane's rect (`center_bounds`, the cockpit-tab
precedent), stays position/resize-synced with gpui layout, loads the #404 `forge_web_base`-derived
origin on mount and on restore (re-derived, never persisted — the #403 bare-`B`-tag codec contract),
and rejects off-origin navigation in the wry nav callback (pinned origin; no address bar). The
load-bearing piece is **the z-order shim**: a native child composites ABOVE gpui's single Metal scene,
so the webview must hide while ANY gpui overlay is up — driven by ONE pure decision seam
(`overlay_is_up` over the spec's verified 26-state overlay inventory), never a per-overlay patchwork.
A backgrounded Browser tab's webview never paints. **HARD-GATED on #402's recorded GO** (NO-GO pivots
the substrate to CEF-OSR/`paint_surface` and the spec re-designs before implement); also depends on
#403 (tab/residency/placeholder) and #404 (origin derivation). Coexistence per #389 Q3: the native
forge cockpit keeps working (additive); the native FLEET rail is never retired. The webview is a
masked platform surface (`mutants::skip` per §0 ACCEPTED-UNTESTABLE), driven-validated; every masked
fn stays a thin adapter over pure decisions.

## Acceptance
Headline: with a resolvable `.mcp.json`, opening the Browser tab shows the **real Forge web page** at
the derived origin, at the pane's exact rect, tracking every rect change (resize, dock/files-panel
toggle + drag, tab switch); **every overlay in the spec's inventory** hides the webview and brings it
back on dismiss; a backgrounded Browser tab never paints; off-origin navigation is rejected; no-config
keeps the #403 placeholder; the bearer never appears in any loaded URL; restart restores the Browser
tab and re-derives the origin. Full EARS criteria live in the pipeline spec.
