---
pipeline_id: ab731829-a5ea-4d6b-bd41-5a60acd27a7d
ticket: forge#137 (7726f8db-4a2b-439d-9b8b-d61b73416d4a) · local docs/planning/tickets/open/TICKET-137-icons.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: real icons (SVG, themeable) [M8]
type: feature
milestone: M8 — Warp Chrome & Fidelity
references:
  - crates/marley_app/assets/icons/*.svg (NEW vendored, clean-room)
  - crates/marley_app/src/icons.rs (PURE: Icon enum + icon_path)
  - crates/marley_app/src/right_dock.rs (section_icon → Icon)
  - crates/marley_app/src/app.rs (SHIM: Assets AssetSource + with_assets + svg() render)
---

## Title
Real monochrome icons that respect the theme — the emoji in the top bar + cockpit become tinted SVG icons.

## Scope
### In
- Vendored clean-room monochrome SVG icons (filled silhouettes; gpui tints them).
- PURE `Icon` enum + `icon_path(Icon)`; `section_icon(RightSection) -> Icon`.
- SHIM: `Assets` AssetSource + `Application::with_assets`; render `svg()` tinted at the top-bar + cockpit
  emoji sites; drop the #134 active bg-pill (svg respects color → active = accent tint).

### Out
- Pane title-bar `pane_icon` glyphs (follow-up unless trivial). Custom/branded art (swap later).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — gpui `svg().path(icon_path(i)).size(px(16)).text_color(c)` renders a vendored SVG tinted by `c`
  (gpui rasterizes the SVG as an alpha mask + fills with the color; the SVGs are monochrome silhouettes).
- D2 — icons vendored under `crates/marley_app/assets/icons/`, served by an `Assets` AssetSource
  (`load` matches `icons/*.svg` → `include_bytes!`), registered via `Application::new().with_assets(Assets)`.
- D3 — since svg respects color, the #134 active-pill is removed; the active cockpit icon is accent-tinted.
- D4 — icons are clean-room authored (§20 OK); a `LICENSE`/NOTES file records provenance.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `icon_path(i)` is called, it shall return that icon's distinct asset path for each variant. | unit |
| REQ-002 | WHEN `section_icon(section)` is called, it shall map each section to its `Icon`. | unit |
| REQ-003 (visual) | WHEN the app renders, the top-bar + cockpit shall show themed monochrome SVG icons (no emoji); the active cockpit icon accent-tinted. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on icons.rs + section_icon; the Assets/render shim masked. | gate |

## Phase Plan
- **P2** — the SVG assets; icons.rs; the Assets AssetSource + with_assets; the render swap; test plan.
- **P3** — implement (assets + icons.rs + right_dock.rs + app.rs).
- **P3.5** — 1 self-review: the mapping distinct; the AssetSource load; the render tint; no emoji left.
- **P4** — icon_path/section_icon tests (cov/MSI 100) + a LIVE capture (SVG icons rendered) + gate GREEN.
- **P5** — docs, AAR, archive, close #137.
