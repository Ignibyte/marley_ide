# TICKET-402 — Scratch proof-of-embed — a wry child WKWebView over a gpui window (the #389 train slice-1)

- **Forge ticket:** #402 a17801ff-b56e-4a7d-ace4-b3252c49147e (spike, M29)
- **Owner:** 12c8eaf8-5fc5-4421-844d-14626ff86417 (agent session, 2026-08-06)
- **AAR:** 120433f1-f4b4-42d9-b0c9-58a00a255fe9
- **Pipeline doc:** ../../pipeline/completed/402-proof-of-embed.spec.md
- **Source ticket:** sprint M29 (forge #40)
- **Status:** closed (2026-08-06 — GO verdict recorded; forge #402 done)

## Summary
An env-gated scratch probe bin that attaches a wry `WebViewBuilder::build_as_child` WKWebView to a
gpui window through the window's exposed `RawWindowHandle::AppKit` (gpui window.rs:4845 →
platform/mac/window.rs:1548 — re-verified against gpui 0.2.2 as shipped, zero drift from the M26
design doc) and walks the go/no-go checklist embedded-browser-model.md Q1 pinned: (1) position/resize
the child to a pane rect (`with_bounds`/`set_bounds`), (2) a gpui palette-style overlay still shows
via the hide-the-webview-while-overlay-up shim, (3) keyboard + IME first-responder handoff both
directions. The #389 spike locked wry-as-child on paper; this probe settles its two load-bearing
unknowns (z-order vs gpui overlays, focus/IME handoff) in a live headed run and records **GO** (wry-A
stands) or **NO-GO** (pivot to CEF-OSR via gpui `paint_surface` + the already-in-lock `core-video`)
back into embedded-browser-model.md + the AD. This is the ONE place `wry` enters the tree (absent
today — confirmed); the `marley` product binary and app shell stay untouched and wry-free. The probe
is §0 ACCEPTED-UNTESTABLE headed scratch behind an explicit, documented gate exclude — the findings,
not the code, are the deliverable.

## Acceptance
All three Q1 checklist points exercised in an operator-driven headed run with observations recorded
(not unit-tested — the honest §7 arm for a headed path); the GO/NO-GO verdict + evidence written into
embedded-browser-model.md and the `AD-claude-embedded-browser-substrate-001` record updated;
`scripts/gates.sh` green with the probe's exclude explicit + documented (named coverage entry +
reason, per-fn `#[mutants::skip]` justifications); product binary/app shell untouched (`cargo tree -i
wry` shows no product path). Full EARS criteria live in the pipeline spec.
