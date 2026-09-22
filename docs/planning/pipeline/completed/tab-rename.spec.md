---
pipeline_id: 6f4ae08d-5afd-432e-9974-4b7e6494c9df
ticket: forge#177 (46ffa2bc-4643-4b56-abac-b9087cbd904b) · local docs/planning/tickets/open/TICKET-177-tab-rename.md
aar_id: 1f1cfcd2-ca26-42a4-ae8f-2ff5a6f99e61
status: Phase 5 — Complete PASS
title: M11 — rename a tab (double-click) + persist custom titles
type: feature
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/tabs.rs (PURE: Tab.custom_title + display_title + Project::tab_mut)
  - crates/marley_app/src/grid_layout.rs (PURE: the T= title extension + sanitize_title)
  - crates/marley_app/src/app.rs (SHIM: the rename edit state/keys/render + codec wiring)
---

## Title
Double-click a rail tab row → an inline rename; the custom title beats the live command title (#157 keeps
winning when unset), and it SURVIVES a relaunch — the #163 "titles regenerate" limit closes.

## Scope
### In
- PURE `tabs.rs`: `Tab.custom_title: Option<String>`; `display_title(custom, command, fallback)` (the
  precedence); `Project::tab_mut(idx)`.
- PURE `grid_layout.rs`: `TabLayout::Terminal { blob, title }` — the wire grows `T=<title>\x1f<blob>` when
  a title exists (`\x1f` can't appear in the blob's alphabet); no `\x1f` → title-less (old blobs restore
  BYTE-identically); `sanitize_title` (strip \t \n \r \x1f, trim, cap 60) applied at WRITE (D2).
- SHIM: `renaming_tab: Option<(usize, usize, String)>`; double-click (click_count ≥ 2) on a tab row opens
  it; the row renders the draft + caret while renaming; an EARLY key branch (esc cancel / enter commit →
  custom_title + persist / backspace / printable append); build_shell_layout + the boot restore carry the
  title; `live_tab_title` consults custom first.

### Out
- Renaming projects/panes; empty-title semantics beyond "empty clears the custom title".

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | The codec shall round-trip a titled terminal tab, restore an UNTITLED old blob byte-identically, and sanitize framing chars at write (mutants killed). | unit |
| REQ-002 | `display_title` shall prefer custom > command > fallback (each pair pinned). | unit |
| REQ-003 (visual) | Double-click shall open the inline editor; typing + Enter shall rename the row; a relaunch shall SHOW the custom name. | driven (typed — #172) |
| REQ-004 | gate GREEN; the pure fns cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (codec back-compat + the rename-state lifecycle vs switches/closes +
modal-branch ordering vs #96/#166). P4 tests + driven + gate. P5 docs.
