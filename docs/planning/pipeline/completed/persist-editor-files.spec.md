---
pipeline_id: f3e34465-4adf-45dd-80ea-dadc7a77ad39
ticket: forge#243 (3c1e984e-e2f9-49c6-b890-c96f237f18d5) · local docs/planning/tickets/open/TICKET-243-persist-editor-files.md
aar_id: a456ff66-1cb3-424e-a711-1759826ad737
status: Phase 5 — Complete PASS
title: Persist all open editor files across restart, not just the active one
type: feature
milestone: M14
references: [forge#237, forge#205, forge#163, forge#240]
---

## Title
Persist ALL open editor-surface files across a restart — today only the ACTIVE file survives (#237's known
limitation), so the other file-tabs in the strip vanish on relaunch.

## Scope
### In
- **Extend `TabLayout::Code`** (grid_layout.rs:208) to carry the full open-file path LIST + the active index
  (not a single `String`).
- **Pure codec** for the code-tab payload: serialize `V=<active>\x1f<path1>\x1f<path2>…`; parse it back;
  BACK-COMPAT — an old `V=<path>` payload (no `\x1f`) parses as one file, active 0. A path that breaks framing
  (the existing `breaks_framing` — `\t`/`\n`/`\r` — plus the new `\x1f` separator) is DROPPED from the set, and
  the active index re-clamped into the survivors (mirror #205's lossy-but-safe D2).
- **Save** (app.rs:2156): map `tab.editor()` → all `surface.files()` paths + `active_index()`.
- **Restore** (app.rs:916-937): read each path (keep the #154 size/binary guards + the resolve-under-root),
  skipping any that fail (re-clamp active), and rebuild the ONE editor surface with all readable files, then
  activate the saved index; if NONE read, drop the tab (as today).

### Out
- Persisting UNSAVED edits — v1 persists PATHS only + re-reads from disk (unsaved-edit persistence belongs to
  the #242 editable editor). #243 is independent of editability.
- Per-file scroll/caret position — just the open set + which is active.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — `TabLayout::Code` carries `{ paths: Vec<String>, active: usize }`** (shape — struct vs tuple — a Phase-2
  nicety; the payload is a path list + active index).
- **D2 — a framing-breaking path is DROPPED, active re-clamped** (lossy-but-safe; a persisted layout must never
  forge the codec's `\t`/`\n`/`\x1f` framing — mirror #205 `breaks_grid_framing`).
- **D3 — back-compat: old `V=<path>` (no `\x1f`) → `{ [path], 0 }`** so a pre-#243 saved layout restores its
  one file.
- **D4 — restore rebuilds the surface from the readable files** (Phase-2 decides: loop `open_or_switch_code` +
  `activate`, or a new `EditorSurface::from_files` constructor). Unreadable files skipped (#154 guards), active
  re-clamped; none-read → drop the tab.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The code-tab payload codec shall round-trip N paths + the active index (serialize → parse → equal), and parse an old single-path `V=<path>` (no `\x1f`) as one file with active 0. | pure unit round-trip + back-compat case |
| REQ-002 | A path containing a framing char (`\x1f`/`\t`/`\n`/`\r`) shall be dropped from the persisted set and the active index re-clamped into the survivors (never emitting a framing-breaking wire). | pure unit (drop + re-clamp; empty set) |
| REQ-003 | Saving the editor tab shall persist all open files' paths + the active index; restoring shall re-open all readable files into ONE editor surface (skipping unreadable per #154, re-clamping active), or drop the tab if none read. | shim review + driven capture |
| REQ-004 | Opening N files → quit → relaunch shall restore all N in the file-tab strip with the saved active file shown. | driven capture (3 files → relaunch → 3 tabs, saved active) |

## Phase Plan
- **P2 Design** — the `TabLayout::Code` shape; the pure serialize/parse fn (grid_layout.rs, `serialize_leaf`/
  `parse_leaf` precedent) incl. the `\x1f` framing extension + back-compat; the save/restore shim wiring; the
  restore rebuild path (loop vs constructor); `cargo mutants --list -f grid_layout.rs`; the test matrix.
- **P3 Implement** — the pure codec + the `TabLayout::Code` shape → the app.rs save + restore wiring.
- **P3.5 Inspect** — critic: codec round-trip + back-compat + framing drops + re-clamp; restore skips unreadable
  + none-read drop; nothing else reads `TabLayout::Code(_)` as a single path; clean-room. **AWAIT the critic
  before Inspect-PASS (the #240/#241 lesson).**
- **P4 Validate** — RUN the codec unit matrix + a grid round-trip; gate green; DRIVEN capture (3 files restore).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #243; archive.
