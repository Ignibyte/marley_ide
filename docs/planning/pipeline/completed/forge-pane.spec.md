---
pipeline_id: 6d4fba49-5522-4944-83b7-bcbf2692c766
ticket: forge#64 (63c1376d-52aa-4c20-9a05-e431215a6728) · local docs/planning/tickets/open/TICKET-064-forge-pane.md
aar_id: 602bf407-9035-4eab-9084-8df25478928e
status: Phase 5 — Complete PASS
title: Forge pane — the current sprint's tickets
type: feature
milestone: M2.B
references:
  - crates/marley_app/src/forge_view.rs (NEW PURE — status_glyph + sprint_rows + TicketRow)
  - crates/marley_app/src/keymap.rs (cmd-shift-f → toggle-forge)
  - crates/marley_app/src/app.rs (SHIM: startup fetch + the Forge overlay)
  - crates/marley_app/Cargo.toml (add marley_forge_client dep)
---

## Title
The visible cockpit surface (the M2.B headline): cmd-shift-f toggles a Forge overlay showing the current
sprint + its tickets, read live from `marley_forge_client` (#63). CLOSES M2.B — Marley shows the work
alongside the agents (#62) in one window.

## Scope
### In
- PURE (`forge_view.rs`, cov/MSI 100): `status_glyph(status) -> &'static str` (done→✓, in-progress→◐,
  open→○, else •); `TicketRow { number, glyph, status, title, kind }`; `sprint_rows(&SprintView) ->
  Vec<TicketRow>` (map each ticket; empty→empty).
- PURE (`keymap.rs`, cov/MSI 100): cmd-shift-f → `toggle-forge` + a keymap test.
- SHIM (`app.rs`, mutants::skip + cov-excluded): `RootView { forge_sprint: Option<SprintView>,
  forge_open: bool }`; `new()` reads `.mcp.json` from the cwd → `ForgeClient::current_sprint()`
  best-effort into `forge_sprint`; dispatch `toggle-forge`; a Forge overlay (mirrors the cmd-P finder)
  rendering the header + `sprint_rows`, or an unreachable placeholder.
- `marley_app/Cargo.toml` += `marley_forge_client`.

### Out
- Auto-refresh / an interval poll (a later ticket — first cut fetches once at startup). Writing to forge.
  Clicking a ticket. A dedicated dock region (an overlay is the first cut). The agents-on-tickets join.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a toggled OVERLAY (cmd-shift-f), mirroring the finder/history overlays — the fastest visible
  cockpit surface; a dedicated dock can come later.
- D2 — fetch ONCE at startup (best-effort, non-fatal): a localhost connect refuses fast if forge is down
  → `forge_sprint = None` → the overlay shows "forge unreachable". No UI hang.
- D3 — the pure `forge_view` (glyph + rows) is cov/MSI 100; the fetch + overlay render are the masked shim.
- D4 — cmd-shift-f is free (no `"f"` keymap binding; the find bar is triggered outside the keymap).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `status_glyph(s)` is called, it shall return ✓ for `done`, ◐ for `in-progress`, ○ for `open`, and • otherwise. | unit |
| REQ-002 | WHEN `sprint_rows(view)` runs, it shall yield one `TicketRow` per ticket (glyph from its status; number/status/title/kind copied); an empty ticket list → an empty Vec. | unit |
| REQ-003 | WHEN the keymap is queried, cmd-shift-f shall map to `toggle-forge`. | unit |
| REQ-004 (visual) | WHEN cmd-shift-f is pressed, an overlay shall show the current sprint + its tickets (with status glyphs). | self-test (launch → cmd-shift-f → capture) |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on forge_view + keymap; app shim excluded. | gate |

## Phase Plan
- **P2** — `status_glyph`/`sprint_rows`/`TicketRow` + the keymap + the startup-fetch/overlay shim,
  mutation targets, the unit + self-test plan.
- **P3** — forge_view.rs + keymap + the app.rs shim + the Cargo dep.
- **P3.5** — critic: the glyph arms, the row mapping, the keymap non-conflict, the best-effort startup
  fetch (no hang / non-fatal), the overlay seam, mutants.
- **P4** — status_glyph + sprint_rows + keymap unit tests (cov/MSI 100) + the SELF-TEST (cmd-shift-f →
  the live sprint) + gate GREEN.
- **P5** — docs, AAR, archive, close #64 — **M2.B COMPLETE**.
