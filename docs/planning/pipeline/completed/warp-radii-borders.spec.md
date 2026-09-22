---
pipeline_id: 744c4f0a-3c59-45a4-82c3-2c29b20cf63f
ticket: forge#221 (58a6eb14-fef1-46b2-b678-446c7d17cbdd) · local docs/planning/tickets/open/TICKET-221-warp-radii-borders.md
aar_id: d6efcd44-6272-46fb-b167-6d02dcad6ae8
status: Phase 5 — Complete PASS (Plan · Design · Implement · Inspect · Validate all PASS; palette capture confirms REQ-001..003; closes #224)
title: Warp visual parity — round the floating overlay cards (corner radii/borders); closes #224
type: feature
milestone: M12.2
references: []
---

## Title
Match Warp's rounding on the floating overlay cards. Discovery (Explore-mapped, exact app.rs lines):
all 10 floating overlays render SQUARE — none call `.rounded`, and 6 of them are borderless. Warp's
overlays are subtly-framed ROUNDED cards. Round them (reusing the `corner_radius` token) + border the
6 borderless ones so the rounding reads. This CLOSES #224 (the corner_radius orphan). Borders,
separators, and the #130 divider already read Warp-like → no churn.

## Scope
### In
- **D-A** — add `.rounded(colors.corner_radius)` to all 10 overlay outer boxes:
  - the 6 BORDERLESS listing overlays — Palette (l.4899), Launcher (l.4940), Finder (l.4982),
    History (l.5018), Forge (l.5043), Fleet (l.5072) — ALSO get `.border_1().border_color(colors.border)`
    so the rounded edge is defined (a real Warp card, not a bg-luminance step).
  - the 4 already-framed overlays — Diff (l.5287), Find (l.5357), Completion (l.5707),
    Context-menu (l.5799) — just get `.rounded`.
- Resolves #224 (the corner_radius token now frames the floating cards, its intended use).

### Out (explicitly deferred)
- The #130 pane-divider grip (the 6px hover-brightening strip is functional + Warp-like; a distinct
  center grip is a subjective future refinement).
- Borders + separators (caption_header l.336, block `border_t` l.4403, dock edges l.368/4008) — already
  uniform 1px + muted `colors.border`, Warp-like → no churn.
- The git changes side-panel (docked, not a floating card) + the status-flash toast (a transient pill).
- Any radius/border VALUE change — the tokens (px(6), muted border) stay; only wiring more consumers.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — reuse the existing `corner_radius` (px 6) + `border` tokens; NO value change. The rounding is
  the same radius the #219 rail rows already use → visually consistent.
- **D2** — the 6 borderless overlays get a border because a rounded corner with no edge doesn't read as
  rounded (Warp's overlays are subtly framed). The 4 already-framed ones only add `.rounded`.
- **D3** — shim-only render wiring (the overlay boxes are inside the `mutants::skip` render), validated by
  driven captures (open the overlays, confirm rounded + framed) — like #217/#220. The `corner_radius`
  token's exact-value test (lib.rs:184) already guards the radius value; no new pure logic.
- **D4** — auto-approved (/work 195–222): document with captures; CLOSE #224 at completion.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a floating overlay (palette/launcher/finder/history/forge/fleet/diff/find/completion/context-menu) is shown, its outer card shall render with rounded corners (`corner_radius`). | Driven capture (open ≥2 overlays; rounded corners) + review (10 `.rounded` sites) |
| REQ-002 | The 6 borderless listing overlays shall render a 1px muted `colors.border` so the rounded edge is defined. | Driven capture (framed rounded card) + review |
| REQ-003 | The change shall not alter overlay CONTENTS (rows/text/position), nor the separators / dock borders / #130 divider. | Review + capture |
| REQ-004 | The `corner_radius` token shall have render consumers beyond the #219 rail rows (resolving the #224 orphan). | Review (the 10 card sites consume it) + close #224 |

## Phase Plan
- **P2 Design** — confirm the gpui `.rounded(px)` + `.border_1().border_color()` idiom on each overlay box;
  the exact per-overlay edit (the 6 borderless get border+rounded, the 4 framed get rounded); confirm no
  clipping of overlay contents / occlude behavior; file manifest + test plan.
- **P3 Implement** — the 10 render edits.
- **P3.5 Inspect** — critics: each overlay still occludes / positions correctly; the border doesn't shift
  contents; the 4 framed ones aren't double-bordered; clean-room (tokens only).
- **P4 Validate** — driven captures (palette + a couple others rounded+framed); gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close TICKET-221 + forge #221; **close #224**.
