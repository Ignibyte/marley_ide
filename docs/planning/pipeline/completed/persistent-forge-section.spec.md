---
pipeline_id: 0748258b-8be3-4c04-9bd1-43ffb459be8f
ticket: forge#92 (2e809554-6495-4b7c-830b-c1c9ce2ea827) · local docs/planning/tickets/open/TICKET-092-persistent-forge-section.md
aar_id: dd6db9cb-14bf-4aac-8a4c-1667175489f7
status: Phase 5 — Complete PASS
title: persistent Forge section in the dock
type: feature
milestone: M2.F — The Persistent Cockpit
references:
  - crates/marley_app/src/forge_view.rs (PURE: forge_empty_hint)
  - crates/marley_app/src/app.rs (SHIM: forge_ticket_row + the Forge-section render)
---

## Title
Make the Forge sprint view always-visible in the #90 right-dock Forge tab — the current sprint's tickets +
status glyphs, auto-refreshing, with the M2.D modifier trio (copy/claim/comment) inline.

## Scope
### In
- PURE `forge_empty_hint(loading) -> &'static str` ("loading sprint…" / "no active sprint").
- SHIM: extract `forge_ticket_row(&self, row, cx)` (the trio-click clickable row) shared by the ⌘⇧F overlay
  + the Forge section; the #90 Forge-section renders the sprint's rows or the hint.

### Out
- Any change to sprint_rows/ticket_ref/status_glyph (#64/#70) or claim/comment (#75/#76). Removing ⌘⇧F.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `forge_ticket_row` is the single clickable-row builder (overlay + section), `mutants::skip` (UI-event).
- D2 — the section shows `forge_status_line` header + rows when `forge_sprint` is Some, else `forge_empty_hint`.
- D3 — the trio is unchanged: plain=copy (#70), ⌘=claim (#75), ⇧⌘=comment (#76) + the #77 flash.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `forge_empty_hint(loading)` runs, it shall be "loading sprint…" if loading else "no active sprint". | unit |
| REQ-002 (visual) | WHEN the Forge tab is active, the dock shall list the sprint's tickets (or the hint). | self-test (static live + engine) |
| REQ-003 | WHEN a ticket row is ⌘-clicked, it shall claim (the #75 path, unchanged). | self-test (forge read) / engine |
| REQ-004 | gate GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — forge_empty_hint; forge_ticket_row extraction + the overlay rewire + the Forge-section render; tests.
- **P3** — implement.
- **P3.5** — 1 critic: forge_empty_hint MSI; forge_ticket_row output-identical; the render + trio reuse.
- **P4** — forge_empty_hint test (cov/MSI 100) + gate GREEN + static live capture (synthetic click env-blocked).
- **P5** — docs, AAR, archive, close #92.
