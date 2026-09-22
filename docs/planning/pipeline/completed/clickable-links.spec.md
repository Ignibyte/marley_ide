---
pipeline_id: 79d92172-1c52-45b0-a2b1-c399100c93dd
ticket: forge#196 (b76e42db-b8b2-4e44-8ecc-d2bc5e65c1ae) · local docs/planning/tickets/open/TICKET-196-clickable-links.md
aar_id: 189c5b14-1076-4161-b65f-5aa88cd632fe
status: Phase 5 — Complete PASS
title: Clickable links — URLs + file paths in terminal output open on click
type: feature
milestone: M12.2
references: []
---

## Title
Make URLs and file paths in terminal block output clickable: a URL opens in the
system browser; a file path opens in Marley's code view. This is also the **front
half of the M13 terminal↔editor wedge** (#212 extends the file case to open at a
specific `:line:col` in the editor).

## Scope
### In
- A PURE link scanner (new `marley_app` module) over one rendered output line →
  a list of link spans `(byte_range, LinkTarget::{Url(String) | File(PathBuf)})`
  — a URL matcher + a file-path heuristic. cov/MSI 100.
- Render: the link spans are underlined + hover-highlighted in the block output,
  composing with the existing #31 styled-run (`run_paint`) render; a click opens
  the target.
- Open: a URL via a small adapter in `marley_command` (process-spawn confined
  there, §14); a file via the existing `open_file_in_viewer` + `resolve_under_root`
  (#190).

### Out (explicitly deferred)
- **OSC 8 explicit hyperlink escapes** (`ESC]8;;URI ST … ST`). DISCOVERY: the Block
  model does NOT capture hyperlinks today (`terminal_blocks::StyledRun` = text +
  color, no hyperlink field), so honoring OSC 8 is a separate `terminal_blocks`
  grid→run plumbing change. Filed as a follow-up; this slice ships the text-scan
  (the common case + the #212 dep). (§3 one-slice.)
- The `:line:col` precision + editor placement — that's the M13 wedge ticket #212
  (this ships file→code-view open; #212 upgrades to open-at-line once the editor
  is editable).
- Clickable paths in the CODE view or elsewhere — block output only here.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — PURE `scan_links(line: &str) -> Vec<Link>` in a new `marley_app` module
  (`links.rs`), cov/MSI 100, textual heuristic only (NO filesystem stat in the pure
  fn — existence is checked in the shim, where `open_file_in_viewer` already guards).
- **D2** — OSC 8 DEFERRED (see Out); design verifies the grid genuinely doesn't
  carry it before finalizing (if it's cheap to plumb, design may reconsider — but the
  default is defer).
- **D3** — URL open via a NEW `marley_command` adapter (spawn confined per §14); file
  open reuses `open_file_in_viewer`/`resolve_under_root` (#190). No spawn in the
  render or the pure scanner.
- **D4** — link rendering OVERLAYS the existing `run_paint` styled runs; it must not
  break text selection hit-testing or copy on a block row.
- **D5** — autonomous through commit (chad away, prioritized the wedge work); hold the
  push (offer-first).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a block output line contains a URL, the scanner shall return a `Url` link span bounding exactly the URL (excluding trailing punctuation). | scan_links unit fixtures |
| REQ-002 | WHEN a line contains a file-path-like token, the scanner shall return a `File` link span for it. | scan_links unit fixtures |
| REQ-003 | WHEN a line contains multiple links, the scanner shall return all of them in order, non-overlapping. | scan_links unit (multi-link fixture) |
| REQ-004 | WHEN a rendered link is clicked, the system shall open a URL in the system browser (via the `marley_command` adapter) or a file in the code view (resolved against the project root). | driven capture (click URL → opens; click path → code view) |
| REQ-005 | Link rendering shall compose with the #31 styled-run render and shall not break text selection or copy on a block row. | review + driven (select a line with a link) |
| REQ-006 | Process-spawn for URL-open shall be confined to `marley_command` (absent from the render/pure layers). | review (grep) + the adapter's own test |

## Phase Plan
- **P2 Design** — the `Link`/`LinkTarget` types + `scan_links` signature + the URL/path
  heuristics (+ boundary rules); the `marley_command` open-url adapter; how the render
  overlays links onto `run_paint` runs without breaking selection; confirm OSC 8 is not
  cheaply available; the manifest + regression test table.
- **P3 Implement** — `links.rs` (pure) + the adapter + the render/click shim.
- **P3.5 Inspect** — critics: scanner boundary/security (no shell-inject via a crafted
  URL to the opener; sanitize what reaches spawn), selection composition, clean-room.
- **P4 Validate** — scan_links unit+mutation; the gate; driven capture (URL + path click).
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; file the OSC 8 follow-up; close.
