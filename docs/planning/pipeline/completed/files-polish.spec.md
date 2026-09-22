---
pipeline_id: 5008ac69-d923-481d-9802-ef8ab55926ce
ticket: forge#168 (c458c35e-ea56-472b-a6b7-cb311a51d560) · local docs/planning/tickets/open/TICKET-168-files-polish.md
aar_id: 8324936d-ab1d-45cf-8d61-f5fe5456674e
status: Phase 5 — Complete PASS
title: M10 — Files panel polish (refresh on open + drag-resize width)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/settings.rs (PURE: clamp_files_width + the files_width setting + tests)
  - crates/marley_app/src/app.rs (SHIM: the open-refresh, files_panel_w, the drag handle + root move/up)
---

## Title
The Files panel stops lying and starts flexing — the tree refreshes every time it opens (new files appear),
and its right edge drags to a persisted width.

## Scope
### In
- Refresh: both `files_open` toggle sites call `sync_active_project()` on false→true.
- Width: `files_panel_w` (default 240) replaces the const in the render; PURE `clamp_files_width` (160–480,
  NaN→240) + `AppliedSettings.files_width` + `persist_files_width` (the right_section pattern, tested);
  a 4px edge handle + a root move/up drag pair (the #130 divider pattern); persist on release.

### Out
- Live FS watching (refresh is on-open only). A double-click-to-reset width. Left-edge docking options.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `clamp_files_width` shall clamp below 160 → 160, above 480 → 480, pass interior values, and map NaN → 240. | unit |
| REQ-002 | `files_width` shall round-trip through the settings store and load into `AppliedSettings` (default 240). | unit |
| REQ-003 (visual) | WHEN a file is created while the panel is closed, toggling it open shall show the file. | driven capture |
| REQ-004 (visual) | WHEN the panel's right edge is dragged, the panel shall resize (the terminal shifts) within the clamp. | driven capture |
| REQ-005 | gate GREEN; the pure clamp + setting at cov/MSI 100. | gate |

## Phase Plan
P2 folded (mirror right_section + the #130 drag). P3 implement. P3.5 self-review (mirrors; the one novel bit
is the root-level second drag pair — check it can't fight the divider drag). P4 tests + driven + gate. P5 docs.
