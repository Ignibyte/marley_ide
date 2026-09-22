---
pipeline_id: d7696fe8-fcb4-4ea5-943e-b049d2dd49fa
ticket: forge#113 (f35494de-7d89-479b-a3e2-2f8736a76fb7) · local docs/planning/tickets/open/TICKET-113-file-explorer.md
aar_id: 7f60d2e5-5276-4607-a70b-c7d7b3a4a487
status: Phase 5 — Complete PASS
title: the file explorer (icons + dimming)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/file_tree_view.rs (NEW PURE: file_icon, entry_is_dimmed)
  - crates/marley_app/src/lib.rs (mod file_tree_view)
  - crates/marley_app/src/app.rs (SHIM: the Files tree render uses them)
---

## Title
Give the Files tree the Warp explorer look: a file-type icon per entry + dimmed dotfiles.

## Scope
### In
- PURE `file_icon(name)` (by extension) + `entry_is_dimmed(name)` (dotfiles).
- SHIM: the left-dock Files tree renders file rows with the type icon + dims dotfiles; dirs keep disclosure.

### Out
- Gitignored dimming (no `.gitignore` signal on TreeRow — dotfile-dimming instead). Making Files a
  separately splittable pane (folds into the typed-pane-content dispatch, seq-8).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `file_icon` maps the lowercased extension to a glyph (rs 🦀 / toml ⚙ / json {} / md 📝 / else 📄).
- D2 — `entry_is_dimmed(name)` = `name.starts_with('.')` (the only "hidden" signal derivable from the name).
- D3 — Files stays in the left dock (below the sessions); the icons + dimming land now.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `file_icon(name)` runs, it shall map the extension to its glyph (else the generic glyph). | unit |
| REQ-002 | WHEN `entry_is_dimmed(name)` runs, it shall be true for a dotfile, false otherwise. | unit |
| REQ-003 (visual) | WHEN the Files tree renders, file rows shall show type icons; a dotfile is dimmed. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on file_icon/entry_is_dimmed; the shim masked. | gate |

## Phase Plan
- **P2** — file_icon + entry_is_dimmed; the Files-loop render; test plan.
- **P3** — implement (file_tree_view.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: file_icon/dim MSI (arms/else/dotfile); the render applies them; dirs keep disclosure.
- **P4** — the icon/dim tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #113.
