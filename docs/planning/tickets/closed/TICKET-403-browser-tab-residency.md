# TICKET-403 — TabContent::Browser + Content::Browser residency + the bare `B` shell tag (the #389 slice-2 + the #400-gated Browser half)

- **Forge ticket:** #403 `f7bf9657-b4c1-4fd6-b346-584660daf256` (feature, M29)
- **Owner:** unclaimed
- **AAR:** `f394490d-c4fe-4e29-9f30-cc8e7250d848` (submitted — 5 novel findings)
- **Pipeline doc:** ../../pipeline/completed/403-browser-tab-residency.spec.md
- **Source ticket:** sprint M29 (forge sprint #40 `6dff19c6-fb61-472b-89c3-98d3215a489b`)
- **Status:** closed — shipped 2026-08-07, gate GREEN [diff] 15/15, mutation 33/33 MSI 100%

## Summary
The pure, substrate-independent model half of the embedded-browser train — #389's slice-2
(`embedded-browser-model.md` train step 2) fused with the Browser half of the #388 slice-8 that #400
deliberately left gated. Four lettered pieces: **(a)** `TabContent::Browser` born id-bearing per the
#400 cockpit precedent (`TabContent::Cockpit(ContentId, RightSection)`, tabs.rs:32 — Browser needs no
section tag: one kind, so the variant discriminant IS the tag), `rail_section()` →
`RailSection::Browser` (the #385 section finally hosts its namesake next to the cockpit tenants), and
`focus_label` gains a Browser arm (the status_bar.rs:64 `FocusTab` match was built to fail INTO this —
app.rs:20557 names "an embedded browser" as the anticipated kind); **(b)** cash #396's DECLARED
`Content::Browser` variant (content.rs:49 — verified declared, unit, payload-less) under the #394
one-instance-many-views registry lifecycle; the pinned-vs-dropped fork is Phase 2's (cockpit is pinned
by a standing anchor; Browser likely DROPS on last close since the later webview is a reapable
resource — the v1 placeholder owns nothing, so either is safe now and the fork is settled on what the
future payload needs); **(c)** a bare `TabLayout::Browser` shell tag `B` mirroring Cockpit's
`C=<key>` (grid_layout.rs:382/:449) — NO URL ever persists (the #389 Q4 bare-marker decision: v1
content IS the derived Forge URL, re-derived from the active root when #404 lands; this slice has no
URL at all), and back-compat is the shipped unknown-tag skip (restore_shell, grid_layout.rs:455 —
old layouts restore unchanged, old builds skip `B`); **(d)** a placeholder pane renders full-screen
behind the Browser section (no webview, no URL dependency — including the Q3 no-config case). Pure
model + codec at cov/MSI 100; app.rs wiring masked.

## Out (other tickets / later slices)
The webview itself (#405 — wry child, z-order shim, pinned-origin nav); the URL derivation seam
(#404 — the `.mcp.json`-origin re-derive; the placeholder needs NO URL); the split-cell grid leaf
`b` + any `PaneContent::Browser` cell arm (the deferred #389 Q4 grid layer; Browser stays
`addable() == false`, content.rs:83); the CDP agent-browser lane (a separate pillar); any general
user-typed-URL browser (needs the percent-encoding/side-table codec #389 Q4 defers).

## Acceptance (headline)
A Browser tab opens from the Browser section, resolves through the registry (one live instance —
open/re-open/second-project/restore converge on it while it lives), lists under the Browser rail
section, labels the footer, and renders a placeholder pane with zero webview/URL code; the shell
codec round-trips a bare `B` (ids and URLs never serialize) while every shell WITHOUT a Browser tab
serializes byte-identical to today and every old layout restores unchanged; all close paths route
`release_view` with the `#[must_use]` return consumed. React-first: the rail rows + placeholder pane
are prototyped in marley-web first and ported 1:1 (parity pair captured at Validate). Full EARS
criteria in the pipeline spec.
