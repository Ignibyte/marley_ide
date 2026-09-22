---
pipeline_id: 6c599308-cd22-49d4-8a10-d1f38397d922
ticket: forge#249 (bcec6818-fec6-40b3-adf1-1fb8eacc13f8) · local docs/planning/tickets/open/TICKET-249-editor-doc-model.md
aar_id: 64751716-e58b-4c44-aaf0-6c2756d55f34
status: Phase 5 — Complete PASS
title: Editor doc model — Buffer + caret + saved_version behind the active file
type: feature
milestone: M15
references: [forge#237, forge#243, marley_editor]
---

## Title
Back each open editor file with a real editable `marley_editor::Buffer` + a caret + a `saved_version` — the
FOUNDATION of the editable editor (#250 renders from it, #251 types into it, #252 saves it). Today the model is
the read-only, lossy `CodeViewState`.

## Scope
### In
- **A per-file `OpenFile { view: CodeViewState, buffer: Buffer, caret: CharOffset, saved_version:
  BufferVersion }`** held by `EditorSurface` (`files: Vec<OpenFile>`). The path is `view.path`.
- **`Buffer::from_text(&text)` seeded** at both `CodeViewState::new` construction sites (the raw text is in
  scope); `saved_version = buffer.version()` (a freshly-loaded file is clean).
- **`EditorSurface` accessors:** `active_file() -> &CodeViewState` (UNCHANGED — render/persist/tab-strip);
  ADD `active_buffer_mut()`, `active_caret()/_mut()`, `active_saved_version()`, `active_is_dirty()`.
- **Resolve the derive cascade** — `Buffer` derives nothing; `EditorSurface`/`OpenFile` adapt (see D2).

### Out (explicitly deferred)
- Rendering from the Buffer (#250 — until then `code_view_body` reads the kept `view`/`lines`).
- Typing / input-intercept (#251), save (#252), undo (#253), mouse/selection (#254/#255) — all build ON this.
- Multi-line movement + Enter⇒`\n` (the crate is single-line today; the input layer is #251's concern).
- Persisting the buffer/caret — transient by design (#243 persists PATHS; the file is re-read on restore).

## Reference (§20)
**Warp — the command-input editing model.** Warp's command line is a live, editable text **buffer with a
caret** (not a pre-rendered string) — the editing FEEL Marley mirrors here: a `Buffer` + `caret` behind the
active file, the same `(buffer, caret)` shape the prompt already uses. The file-editor **container** (a file
open as a tab/pane) is **Marley-specific** (the editor-as-peer intake pillar — Warp has no file editor). #249 is
the internal MODEL only (no pixels), so the behavior reference suffices; the **observed capture** of Warp's
input caret/typing is deferred to **#250** (the first visible render) and to `docs/warp_architecture/observed/`
(chad's Mac is currently locked — capture when available). Clean-room: observe behavior, reuse `ropey`/`gpui`
freely, never read Warp source.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — OPTION B: the edit model lives in `EditorSurface` per file (`OpenFile`), NOT in `CodeViewState`.**
  `CodeViewState` stays `Debug+Clone+PartialEq+Eq` (it's also the read-only split pane's model,
  `PaneContent::CodeView(CodeViewState)` `.cloned()` at app.rs:5639). `active_file()` keeps returning
  `&CodeViewState` so render + the #243 path-persistence + the file-tab strip are untouched.
- **D2 — the derive cascade is CONTAINED at `EditorSurface`.** `Buffer` derives nothing (buffer.rs:17,
  deliberate); `TabContent<S>`/`Tab<S>`/`Project<S>`/`Workspace<S>` are generic over `S` and carry NO
  Clone/Eq/Debug derives, so embedding `Buffer` does NOT cascade past `EditorSurface`. Design picks the minimal
  fix: (A) add `#[derive(Clone, Debug)]` (± PartialEq/Eq) to `marley_editor::Buffer` (Rope/SelectionSet/
  BufferVersion all support them; the crate author just didn't) IF anything needs `EditorSurface: Clone/Eq/
  Debug`; else (B) DROP those derives from `EditorSurface` + `OpenFile` (a manual `Debug` if a test needs it).
  Design greps the actual requirement (callers + tests) and picks the smaller.
- **D3 — dirty = `buffer.version() != saved_version`.** Save (#252) sets `saved_version = buffer.version()`.
- **D4 — no codec change.** #243 persists paths; the buffer is rebuilt from disk on restore (transient model).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `EditorSurface` shall back each open file with a `Buffer` seeded from the file text + a caret (CharOffset) + a `saved_version` (= the buffer's version at load). | pure unit |
| REQ-002 | `active_file()` shall still return the file's `CodeViewState` (render + #243 persistence + the file-tab strip unchanged); new accessors shall expose the active file's buffer / caret / dirty state. | pure unit + `cargo check` |
| REQ-003 | A file shall report dirty iff `buffer.version() != saved_version` — clean at load, dirty after an edit, clean again after `saved_version` is re-synced. | pure unit |
| REQ-004 | Opening an already-open path shall reuse its `OpenFile` (dedupe by path — the #237 behavior), not create a second. | pure unit |
| REQ-005 | The read path (`code_view_body`), the read-only split pane, and the #243 path-persistence shall be UNAFFECTED (compile + behavior). | `cargo check --workspace` + review |

## Phase Plan
- **P2 Design** — confirm D1-D4 + the `OpenFile` shape + the EditorSurface accessor set; RESOLVE the derive
  fork (grep the requirement, pick A or B); the manifest (editor_surface.rs + the 2 seed sites + maybe
  marley_editor Buffer derives); the pure test matrix; `cargo mutants --list`.
- **P3 Implement** — `OpenFile` + the surface refactor (Vec<OpenFile>) + the accessors + the seeds; the derive
  fix; `cargo check --workspace` clean.
- **P3.5 Inspect** — critic: the seed is correct (buffer text == file text, saved_version clean); dedupe
  preserved; the derive fix is minimal + nothing lost; the read/split/persist paths unaffected; clean-room.
- **P4 Validate** — pure units (cov/MSI 100 on the OpenFile/surface management) + `cargo check --workspace`;
  gate green. DRIVEN is minimal (no visible change until #250) — note it; the driven proof lands at #250.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #249; archive.
