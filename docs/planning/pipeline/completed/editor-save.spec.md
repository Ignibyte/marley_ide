---
pipeline_id: 99180a00-55d9-49ba-a84a-87873485b831
ticket: forge#252 (6fba8467-f1bf-4a9c-ad79-88938c0b6cba) · local docs/planning/tickets/open/TICKET-252-editor-save.md
aar_id: c3576454-eafb-4236-b84d-2cc8599209d1
status: Phase 5 — Complete PASS
title: Editor save (⌘S) + dirty ● indicator
type: feature
milestone: M15
references: [forge#249, forge#251, forge#237]
---

## Title
⌘S writes the active editor file's `buffer.text()` to its path on disk + marks it clean; the #237 file-tab strip
shows a dirty ● on any file with unsaved edits (`buffer.version() != saved_version`). The two halves of "save".

## Scope
### In
- **`file_dirty_flags()` on `EditorSurface`** (pure) — a per-file `bool` (each `OpenFile::is_dirty()`), so the
  tab strip can mark EVERY dirty file, not just the active one. `files()` keeps returning `&CodeViewState`
  (render/#243 unaffected).
- **`save_active(&mut self) -> std::io::Result<()>`** on the APP (app.rs shim — keeps `editor_surface` pure):
  `std::fs::write(active_file().path, active_buffer().text())?;` then `active_mark_saved()`. On `Err` → do NOT
  mark clean (the ● stays); typed `io::Result`, no `unwrap` (§14). A save error → a status flash (reuse the
  existing flash), not a crash.
- **⌘S → "save"** — a new `(⌘S, "save")` binding in `Keymap::default_bindings()` (pure) + a `"save"` arm in
  `dispatch_action` (shim): guarded to an active editor tab (else no-op).
- **The ● render** — the tab strip (app.rs ~4981) draws a ● beside a dirty file's name.

### Out (explicitly deferred)
- Save-as / a new path; save-all; a save-confirm-on-close dialog; external-change (on-disk) detection; atomic /
  temp-file write (v1 is a plain `fs::write` — noted); a save keybinding-settings entry.

## Reference (§20)
**Warp — its edited/unsaved indicators (the ticket cites them) + the universal editor ●/⌘S convention.** Warp
marks unsaved/edited state on its inputs/blocks with a dot-style affordance; the file editor mirrors that
behavior for a file tab (a ● when `version != saved_version`, cleared on ⌘S) and adopts the standard editor
⌘S-writes-to-disk convention. This is a behavior/convention reference (a dot when dirty, ⌘S saves + clears) — no
Warp/Zed source read; the fs write + the keymap reuse Rust std + the in-repo keymap. The #250 observed
monospace-grid capture stands for the editor render.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the write lives in the app.rs SHIM, not `editor_surface`** (`save_active` does the `fs::write` +
  `active_mark_saved`), so `editor_surface.rs` stays PURE (no IO). `mark_saved` runs ONLY after a successful
  write (REQ-004: an `Err` leaves the file dirty).
- **D2 — per-file dirty is a new pure `file_dirty_flags()`** (maps `OpenFile::is_dirty()`); the tab strip zips
  it with `files()`. Keeps `files()`'s signature (no #250/#243 caller churn).
- **D3 — ⌘S via the keymap action path** (not a direct on_key_down check): `(⌘S,"save")` in the pure
  `default_bindings` + a `dispatch_action` `"save"` arm, mirroring ⌘W/⌘P/⌘D. Guard the arm to an editor tab.
- **D4 — a save error surfaces a status flash** (reuse the existing flash), not a panic; the ● staying is the
  primary "not saved" signal.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Typing into an editor file shall show a ● on THAT file's tab (`buffer.version() != saved_version`), per-file. | pure unit + driven |
| REQ-002 | ⌘S with an editor tab active shall write the active file's `buffer.text()` to its path on disk. | driven |
| REQ-003 | After a successful save the file shall be clean (● cleared; `saved_version == buffer.version()`). | pure unit + driven |
| REQ-004 | A failed write shall NOT mark the file clean (the ● stays) and shall not crash (a status flash). | pure unit (mark-only-on-Ok) + review |
| REQ-005 | ⌘S with a terminal tab active shall be a no-op (terminal/prompt unaffected); each file's dirty state is independent. | pure unit (per-file) + review |

## Phase Plan
- **P2 Design** — confirm D1-D4; `file_dirty_flags` signature + the strip zip; `save_active`'s io::Result +
  mark-on-Ok ordering; the `(⌘S,"save")` binding + the `dispatch_action` arm + the editor guard; the flash reuse;
  the pure test matrix (file_dirty_flags per-file; the mark-on-Ok logic); `cargo mutants --list`. Confirm §20.
- **P3 Implement** — `file_dirty_flags` (editor_surface); `save_active` + the "save" dispatch arm + the ● render
  (app.rs); the `(⌘S,"save")` binding (keymap); `cargo check`.
- **P3.5 Inspect** — critics: mark-clean-ONLY-on-Ok (REQ-004); per-file dirty (not just active); the ⌘S guard
  (editor-only, terminal unaffected); no data-loss on the write (writes the buffer, to the right path); no panic.
- **P4 Validate** — pure units cov/MSI 100 (file_dirty_flags, the mark-on-Ok logic, the keymap binding) + DRIVEN
  (mac unlocked — a SCRATCH temp .txt: open, type → ● appears, ⌘S → ● clears + the file on disk has the text,
  then DELETE the temp; DATA-SAFETY: never a real repo file). Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #252; archive.
