---
pipeline_id: 721af705-bb28-4b7d-90f3-3ab737085409
ticket: forge#219 (6de7c89d-96b2-4f5d-9cc7-c875c1dad801) · local docs/planning/tickets/open/TICKET-219-warp-sidebar-styling.md
aar_id: 1d75b4a5-6514-4ab5-bce9-cc38ac424ddd
status: Phase 5 — Complete PASS (Plan · Design · Implement · Inspect · Validate all PASS; driven captures confirm REQ-001..005)
title: Warp visual parity — session sidebar active/hover row highlight (fix the invisible selection)
type: feature
milestone: M12.2
references: []
---

## Title
Match Warp's left session list. The genuine delta (confirmed at discovery — code at app.rs
~3721-3945 + tabs.rs `rail_rows`; Warp ref `scratchpad/219-warp-sidebar.png`; current Marley
`scratchpad/219-marley-sidebar.png`):
Marley's active Tab/Pane row highlights with `bg(colors.surface)` — but the rail's dock is ALSO
filled `colors.surface` (#194), so the highlight is **surface-on-surface = invisible**; the active
tab reads only via brighter text. Warp shows the selected row as a **distinct, slightly-lighter
rounded filled box** and highlights rows on **hover**. Make Marley's selection visible + Warp-like.

## Scope
### In
- **D-A** — the active Tab + Pane row highlight (app.rs 3871 / 3927): replace `bg(colors.surface)`
  with a DISTINCT elevated fill (accent-tinted wash, or a solid step above surface — design fixes the
  exact value/API) + `.rounded(...)` for the Warp rounded-box look. Keep the bright `foreground` text.
- **D-B** — a subtle HOVER highlight on Tab + Pane rows (Warp highlights on hover): `.hover(|s| s.bg(...))`
  at a lower intensity than the active fill.
- **Pure seam** — a small pure `rail_active_highlight(...) -> Hsla` (the chosen active-row fill) with an
  exact-value test AND a guard test asserting it is PERCEPTIBLY DISTINCT from `surface` (pins the exact
  invisibility bug so it can't regress — reuse #194's `contrast_ratio` or a lightness-delta ≥ threshold).

### Out (explicitly deferred)
- The per-row LEADING terminal icon — Marley's rail is a Workspace→Project→Tab→Pane TREE that uses
  INDENT for hierarchy + already glyphs agent tabs; a generic per-row icon competes with the indent and
  is a larger separate visual decision, not a bounded tweak.
- 2-line title+subtitle rows (needs per-tab git/cwd data plumbing).
- Row height / indent (already reasonable/compact — no churn) and the MARLEY/project header (already
  Warp-like).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — the active highlight must be perceptibly distinct from `surface` (the dock bg) — that
  distinctness is the fix and is guarded by a pure test, not eyeballed.
- **D2** — a pure `rail_active_highlight` helper is the single source of the highlight color, cov/MSI 100
  (exact-value + the distinct-from-surface guard). The render (bg/rounded/hover) is the shim
  (`#[cfg_attr(test, mutants::skip)]`), capture-validated like #216/#217.
- **D3** — recommend an accent-tinted fill (ties to #191's focus-accent identity + guaranteed distinct
  from the neutral grays); design confirms the exact gpui `Hsla` alpha/lightness API + value, or falls
  back to a solid elevated fill. Tokens only — no new hardcoded hsla beyond deriving from `accent`/`surface`.
- **D4** — auto-approved (/work 195–222): document with the Warp-vs-Marley captures.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Tab or Pane row is the active row, the sidebar shall render it with a fill that is perceptibly distinct from the dock `surface` background (a visible selection box). | Driven capture (active row visibly highlighted) + review |
| REQ-002 | The active-row highlight shall be rounded (a Warp-style box, not an edge-to-edge band). | Driven capture + review |
| REQ-003 | WHEN the pointer hovers a Tab or Pane row that is not active, the sidebar shall render a subtle hover highlight distinct from the rest. | Driven capture (hover shows a highlight) |
| REQ-004 | The pure highlight helper shall return a color perceptibly distinct from `surface` for all supported themes (light + dark). | Unit test (exact-value + distinct-from-surface guard, cov/MSI 100) |
| REQ-005 | The change shall not alter the active-row TEXT color (stays `foreground`), the row labels, the close ×, or the agent-status glyph. | Review + capture |

## Phase Plan
- **P2 Design** — confirm the gpui `Hsla` alpha/lightness API + the exact highlight value (accent-tint vs
  solid step); the helper signature + home (pure-testable); the render change (bg→helper, +rounded,
  +hover on Tab & Pane rows); the distinct-from-surface guard threshold; file manifest + test plan.
- **P3 Implement** — the pure helper + the row render change.
- **P3.5 Inspect** — critics: the highlight is genuinely distinct in BOTH themes; no regression to the
  #177 rename / #167 agent glyph / close-× / the active text color; clean-room.
- **P4 Validate** — helper unit tests (exact-value + distinct guard) + driven captures (active box visible,
  hover highlight, both themes if feasible); gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close.
