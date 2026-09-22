---
pipeline_id: 88aca305-fc53-4343-be01-d047378b0d94
ticket: forge#194 (1469c76c-b4ac-4072-9d92-57ca7fe96a07) · local docs/planning/tickets/open/TICKET-194-terminal-black-panel-gray.md
aar_id: 209075eb-b1d7-4713-80b6-94078239581f
status: Phase 5 — Complete PASS
title: Terminal-black pane background + gray panel/dock surfaces (Warp-look dark theme)
type: feature
milestone: M12.2
references: []
---

## Title
Recalibrate the DARK theme's `ThemeColors` surface tokens so the terminal pane
reads near-black and the panels/docks/tab-bar/footer read a distinct, slightly
lighter gray — chad's Warp-look direction ("I like the terminal area being black
with the panel being the slight gray color"). Keep the foreground text and the
shipped #31 ANSI palette legible on the darker background.

## Scope
### In
- The DARK `ThemeColors` (ui_components/src/lib.rs `default_for(Dark)`): terminal
  pane background → near-black; the panel/dock/tab-bar/footer/right-dock surface →
  a distinct lighter gray; border / focus-accent / selection / cursor tuned for
  the new contrast.
- Where the render assigns those tokens (marley_app themes.rs / color.rs / the
  pane + dock render) — so the terminal pane and the chrome pull from the right
  token (they may share one today; the separation is the point).
- A PURE contrast-ratio guard (WCAG relative-luminance) as the testable seam that
  proves text + ANSI stay legible on the new terminal bg.

### Out (explicitly deferred)
- The LIGHT theme (unchanged).
- Font sizes (#195, separate).
- A user-facing theme picker (#199) — this ticket recalibrates the built-in dark
  default, not the switching UI.
- Any Warp source/theme-file/hex/asset copying (§20 clean-room).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — DARK theme only. Prefer recalibrating EXISTING tokens (e.g. `background`
  → terminal near-black, panels → `surface`/gray) over adding a new token, unless
  design finds the render can't separate them cleanly (decided in Phase 2).
- **D2** — The testable PURE seam is a WCAG contrast-ratio helper (relative
  luminance → ratio) asserting text-vs-terminal-bg ≥ target + each ANSI color ≥ a
  min ratio; cov/MSI 100. `hsla` literals are NOT mutated by cargo-mutants (M1.E
  #35), so the token values are guarded by EXACT-value regression asserts + the
  contrast test, not mutation of the literals.
- **D3** — The accent-vs-`on_accent` pairing must still clear WCAG AA (the M1.E #35
  inspect lesson — don't regress button-label contrast while darkening).
- **D4 — TASTE GATE (chad AWAY).** This pipeline runs **PLAN + DESIGN only**. Phase 2
  proposes the exact shades (current values + proposed targets + rationale + the
  contrast check) and then **PAUSES at the design→implement gate** for chad's
  approval of the shades. Do NOT implement or commit unapproved values.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the dark theme is active, the terminal pane background shall render near-black, distinctly darker than the panel surfaces. | Driven capture + exact token-value assert |
| REQ-002 | WHEN the dark theme is active, the dock/panel/tab-bar/footer surfaces shall render a gray distinctly lighter than the terminal pane (a visible separation). | Driven capture + token-value assert |
| REQ-003 | The foreground text and each #31 ANSI color shall remain legible on the terminal background (body text contrast ≥ 7:1; each ANSI color ≥ a documented minimum). | Pure WCAG contrast-ratio unit test over the token set |
| REQ-004 | The change shall not alter the LIGHT theme. | Light-theme value regression test unchanged |
| REQ-005 | The chosen values shall be Marley-original (not copied from Warp's theme files/assets). | Review (§20 clean-room) |

## Phase Plan
- **P2 Design** — read the current dark tokens + every render site that assigns
  them; propose the exact near-black + gray (+ border/selection/cursor) with a
  contrast table; name the pure guard + the manifest + the test plan. **PAUSE for
  chad's shade approval (D4).**
- **P3 Implement** — (after approval) set the approved token values; wire the
  render so the terminal pane and chrome pull the right token.
- **P3.5 Inspect** — critics: contrast regressions, ANSI-on-black legibility, no
  light-theme bleed, clean-room.
- **P4 Validate** — the contrast unit test + value regressions; driven capture
  (terminal black, panels gray, text + ANSI legible).
- **P5 Complete** — CHANGELOG + app_shell/theme doc; AAR; close.
