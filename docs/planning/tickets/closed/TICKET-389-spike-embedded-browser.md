# TICKET-389 — SPIKE: embedded browser substrate + Forge-in-browser plan (opens Phase E)

- **Forge ticket:** #389 `88f9c375-191d-4e52-892e-9b84309e9a4f` (spike, M26)
- **Owner:** unclaimed (queued)
- **AAR:** — (opened at promotion)
- **Pipeline doc:** ../../pipeline/queued/389-spike-embedded-browser.spec.md
- **Source:** chad's 2026-07-22 refinement — third section is Browser; "Forge will open in the
  browser"; roadmap Phase E ("the project Forge URL as a live pane")
- **Status:** closed

## Summary
Marley has zero webview substrate (confirmed 2026-07-22: no dep, no gpui element). This spike
(docs only — nothing ships) settles Phase E's opening moves: the substrate choice (wry vs direct
WKWebView vs CEF) against gpui's self-composited NSView-backed window, the honestly-named CDP fork
(the Forge pane needs no CDP; the agent-browser pillar does — possibly two substrates), the
Forge-in-browser plan (URL derivation from the #381 .mcp.json resolution, auth posture under the
loopback wall, pinned-origin v1 nav, native-cockpit coexistence/retirement), the
`TabContent::Browser` + shell-codec migration plan, licensing, and a sliced follow-up train.

## Acceptance
A decision doc naming the substrate (with embed-feasibility evidence), the CDP fork recorded, the
Forge-in-browser + codec plans written, forge AD + ticket slices emitted, zero shipped app code.
Full EARS in the pipeline spec.
