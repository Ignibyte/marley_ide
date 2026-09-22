---
pipeline_id: b22246aa-1ad9-4cab-93b4-91c322039305
ticket: forge#253 (83b62a5a-f30e-48cf-b135-0c2430474eca) · local docs/planning/tickets/open/TICKET-253-editor-undo.md
aar_id: df83094b-ffd1-42e1-88ef-1c85c04ac691
title: Editor undo/redo — edit-history + ⌘Z / ⌘⇧Z
type: feature
milestone: M15
references: [forge#249, forge#251, forge#252, marley_editor]
status: Phase 5 — Complete PASS
---

## Title
Give `marley_editor::Buffer` an in-memory edit-history so ⌘Z undoes the last edit and ⌘⇧Z redoes it, with
consecutive typed characters COALESCED into one undo step (⌘Z removes a run/word, not one char), restoring the
caret to the edit site.

## Scope
### In
- **A pure `UndoHistory` in marley_editor** (a new `undo.rs`): `undone: Vec<EditRecord>`, `redone:
  Vec<EditRecord>`, where `EditRecord { at: CharOffset, removed: String, inserted: String }` (the edit replaced
  `removed` with `inserted` at `at`). `Buffer` owns one.
- **`edit()` records** an invertible step (capturing `removed` BEFORE the rope mutation) + clears the redo
  stack — UNLESS the edit is an internal undo/redo re-apply (a non-recording path, so the inverse never
  re-records).
- **COALESCING** — a single-char insert (`removed` empty, `inserted` one char) contiguous with the previous
  record's end (`prev.at + prev.inserted.chars == new.at`), same `origin`, and the previous record itself an
  insert-run → EXTEND `prev.inserted` instead of pushing. Backspace / delete / multi-char / non-contiguous →
  a new record.
- **`undo() -> Option<CharOffset>`** (the caret target = edit site; `None` when empty) applies the inverse +
  moves the record to `redone`. **`redo() -> Option<CharOffset>`** re-applies from `redone`.
- **⌘Z/⌘⇧Z wiring** — `(⌘,z)→"undo"` + `(⌘,⇧,z)→"redo"` keymap bindings + guarded `dispatch_action` arms
  routing to the active editor buffer + syncing `active_caret`.

### Out (explicitly deferred)
- Time-window coalescing (v1 = contiguity + origin, no timer); persistent history (transient); undo across
  file-close; selection-aware undo; grouping backspace runs; a redo ⌘Y alias.

## Reference (§20)
**Warp — its command-input undo/redo (chords + coalescing).** Warp's input supports undo/redo where a run of
typed characters undoes as one step (not char-by-char) — the editing FEEL Marley mirrors for the file buffer:
⌘Z undoes the last coalesced edit, ⌘⇧Z redoes, and a fresh edit after an undo clears the redo stack. The
history model itself is Marley's own pure `UndoHistory` (an invertible `EditRecord` stack). Clean-room: observe
the undo BEHAVIOR (chord + typing-run coalescing); reimplement over `ropey`/in-repo; read NO Warp/Zed source.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — `UndoHistory` is a pure marley_editor type** (`undo.rs`), owned by `Buffer`; `edit()` pushes/coalesces,
  `undo()/redo()` apply the inverse. cov/MSI 100 (record / coalesce / undo / redo / redo-clear / no-re-record).
- **D2 — the inverse-apply must NOT re-record** (else undo pushes → the stack never drains). Design picks a
  private non-recording rope edit (`apply_raw`) that `undo()/redo()` call, vs `edit()` which records.
- **D3 — the caret target is the edit site.** `undo()/redo()` return `Option<CharOffset>` (the offset after the
  inverse); the app syncs `active_caret` to it. (Buffer's own selection may also be set; the app's caret is the
  render source, #250.)
- **D4 — coalescing is contiguity + origin, NOT a timer** (deterministic, testable). A single-char Human insert
  contiguous with an insert-run extends it; anything else breaks the run.
- **D5 — an undo does NOT un-dirty** (version only increments; undo is a new version). A file undone to its
  saved content still shows the ● (#252) — acceptable v1; noted.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `edit()` shall record an invertible step on the history (unless it is an internal undo/redo re-apply). | pure unit |
| REQ-002 | `undo()` shall revert the buffer to before the last (coalesced) step + return the caret site; `None` (no change) when the history is empty. | pure unit |
| REQ-003 | Consecutive single-char inserts (contiguous, same origin) shall coalesce into ONE undo step. | pure unit |
| REQ-004 | `redo()` shall re-apply the last undone step; a NEW `edit()` after an undo shall clear the redo stack. | pure unit |
| REQ-005 | ⌘Z / ⌘⇧Z with an editor tab active shall undo/redo the active file's buffer + sync the caret; a terminal tab shall be unaffected. | driven + review |

## Phase Plan
- **P2 Design** — the `UndoHistory`/`EditRecord` shape + the coalesce predicate + the non-recording `apply_raw`
  (D2) + `undo/redo -> Option<CharOffset>` + the ⌘Z/⌘⇧Z bindings (roster count 39→41) + the dispatch arms +
  caret sync; the pure test matrix; `cargo mutants --list`. Confirm §20.
- **P3 Implement** — `undo.rs` + `Buffer::{undo,redo}` + `edit()` records/coalesces; the keymap bindings + the
  dispatch arms + the route (app.rs shim, KEEP dispatch_action's `mutants::skip` — the #252 detach-trap
  lesson); `cargo check`.
- **P3.5 Inspect** — critics: undo round-trips the buffer exactly (text + caret); coalescing groups a run; the
  inverse doesn't re-record (no infinite/leak); redo-clear on a new edit; the ⌘Z guard (editor-only); re-run
  `cargo mutants --list` for the skip.
- **P4 Validate** — pure units cov/MSI 100 (the history) + DRIVEN (type a word into the restored editor tab →
  ⌘Z removes it → ⌘⇧Z re-adds it). Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #253; archive.
