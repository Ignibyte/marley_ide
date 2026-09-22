---
pipeline_id: 27848d21-b809-44bf-a61c-cbba6ec523a9
ticket: forge#34 (2d48faf5-6191-4a84-8066-0e5af1b70b67) · local docs/planning/tickets/open/TICKET-034-terminal-font.md
aar_id: 55039a8f-140d-4ba1-b0f3-ffa6bdfe86cc
status: Phase 5 — Complete PASS
title: monospace terminal font + accurate cell metric
type: feature
milestone: M1.E
intake: docs/planning/intake/terminal-monospace-font-and-cell-metric.md
references:
  - crates/marley_app/src/app.rs (the cell-metric read :635-646 + the terminal spans)
  - crates/marley_app/src/workspace.rs (CellSize — the metric type; plan_resize #30 consumes it)
  - docs/specs/SPEC-app-shell.spec.md (gains the terminal-font clause)
---

## Title
Terminal text renders in the window's AMBIENT (proportional) font — no font is set on the Block
command/output spans, the prompt, or the alt-screen grid — so columns don't align, and the #30 cell
metric (`em_advance × line_height`) is read from that proportional font, so `plan_resize` (#30) +
the viewport capacity (#32) + the alt-screen grid (#33) size against a cell width that doesn't match
the drawn glyphs. Set an explicit MONOSPACE font for terminal text and read the cell metric from it.
Closes the intake.

## Scope
### In
- `crates/marley_app/src/app.rs` (shim) — a `TERMINAL_FONT` family constant (`"Menlo"` — the macOS
  system monospace; clean-room, not a proprietary/Warp font). Apply `.font_family(TERMINAL_FONT)`
  to the terminal-content spans: the Block command + output runs (#31), the prompt row, and the
  alt-screen grid (#33). The cockpit chrome (docks/palette/titlebar) keeps the UI font.
- `crates/marley_app/src/app.rs` — the cell-metric read (:635-646) resolves `TERMINAL_FONT` (via
  `resolve_font(&gpui::font(TERMINAL_FONT))`) for `em_advance`, and uses that font's line height,
  instead of `window.text_style().font()`.
- NEW pure `fallback_cell(font_size: f32) -> CellSize` (workspace.rs or a small metrics module) —
  the metric fallback when `em_advance` fails: advance ≈ `0.6 · font_size`, height ≈ `1.2 · font_size`
  (a monospace cell's typical ratios), clamped > 0. Replaces the hardcoded `px(8.0)` fallback with
  a font-size-relative one — the (thin) PURE gate surface.
- SPEC-app-shell: the terminal-font clause. CHANGELOG + arch doc. Promote the intake → done.

### Out (explicitly deferred)
- Bundling a custom open font as an asset (Menlo-by-name needs no bundling on macOS — the CI target;
  a bundled cross-platform mono is a later cut). User-configurable font family/size (a settings
  clause — M2). Ligatures, font fallback chains, per-pane fonts. The palette/chrome (#35+).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `TERMINAL_FONT = "Menlo"` — the macOS system monospace (the CI/dev target is macOS-only per
  the cross-platform-mutation note). Clean-room: a system font, not Warp's. gpui resolves it by name
  (`gpui::font(..)` ships). A bundled open mono is deferred (no asset pipeline yet).
- D2 — The metric reads from `TERMINAL_FONT`, so `plan_resize`/viewport/grid finally size against the
  ACTUAL drawn cell — closing the #30 inspect #3 fidelity gap.
- D3 — `fallback_cell(font_size)` is PURE + tested (the 0.6/1.2 ratios + the >0 clamp) — the only
  new pure surface; the font application + metric read are shim (app.rs, cov-excluded, masked visual).
- D4 — Mono is applied to TERMINAL content only (not the cockpit chrome) — the UI font stays for
  docks/palette/titlebar.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `fallback_cell(font_size)` is called, it shall return a `CellSize` with `w ≈ 0.6·font_size` and `h ≈ 1.2·font_size`, each clamped to at least a small positive value (never 0/negative for a non-positive font size). | unit tests (14.0 → (8.4, 16.8); 0.0 / negative → the positive floor) |
| REQ-002 | WHEN the terminal content is rendered, the system shall paint it in the monospace `TERMINAL_FONT` and read the pane cell metric from that font (not the ambient proportional font). | shim + the masked visual baseline (aligned columns) — chad-verified |
| REQ-003 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `fallback_cell`; the aligned-columns visual baseline rides the desktop-session deferral (masked). | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — the `TERMINAL_FONT` constant + where `.font_family` is applied (the pane content
  container vs per-span), the metric-read change, `fallback_cell`'s exact shape + clamp, the SPEC
  clause + mutation targets.
- **P3 Implement** — app.rs font + metric + `fallback_cell` + spec + CHANGELOG; promote the intake.
- **P3.5 Inspect** — critics: the fallback ratios/clamp, the metric-read correctness, the font
  applied to terminal-only, no chrome regression, clean-room (system font).
- **P4 Validate** — the `fallback_cell` unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #34.
