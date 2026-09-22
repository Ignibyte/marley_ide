---
pipeline_id: 515e1b2d-c898-47f2-a95f-fabdf4b2a40e
ticket: forge#173 (48330513-d9a8-43d5-acc2-b6ca9faff3da) · local docs/planning/tickets/open/TICKET-173-pump-all.md
aar_id: d87d0f69-4d6c-4a8e-b263-195b6c803a83
status: Phase 5 — Complete PASS
title: M11 — pump EVERY grid (background tabs stay live)
type: feature
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/tabs.rs (PURE: grids/grids_mut/locate_pane)
  - crates/marley_app/src/app.rs (SHIM: the pump loop + dead-pane close + refresh over ALL grids)
---

## Title
Background tabs stop freezing: the 16ms pump drains EVERY grid's PTYs (the latent background-stall fix),
agent statuses/quiet-ticks track across tabs (the #167 glyph un-freeze), and live titles follow (#157).
Render stays active-only.

## Scope
### In
- PURE `tabs.rs`: `Workspace::grids()` / `grids_mut()` (every terminal tab's grid across every project) +
  `Workspace::locate_pane(PaneId) -> Option<(usize, usize)>` (built here; #174 reuses it).
- SHIM: the pump iterates `grids_mut()`; dead panes close in their OWNING grid via `locate_pane`
  (the per-grid last-pane stays visibly dead, as today); `refresh_agent_statuses` resolves agents across
  all grids (`grids().find_map(terminal)`).

### Out
- Closing whole tabs from the pump; render/scrollback for background grids; pump staggering (measure first —
  pump() idles as a cheap try-read).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `grids`/`grids_mut` shall yield exactly the terminal-tab grids across all projects (cockpit/code tabs skipped); `locate_pane` shall return the (project, tab) owning a pane and None for an unknown id — both mutation-killed. | unit |
| REQ-002 (visual) | WHEN an agent runs in tab A and tab B is active, tab A's rail glyph shall transition (●→◔) while backgrounded, and returning shall show the accumulated output. | driven capture |
| REQ-003 | A dead pane in a background grid shall close in ITS grid (agents/remotes entries dropped), not the active one. | code/critic |
| REQ-004 | gate GREEN; the pure fns cov/MSI 100; an idle-CPU sanity sample before/after (honest note). | gate + ps |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (borrow shape of the all-grids loop; dead-close exactness; the
remote-pane rule across grids; perf). P4 tests + driven + gate. P5 docs.
