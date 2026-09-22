---
pipeline_id: 6c599308-cd22-49d4-8a10-d1f38397d922
ticket: forge#249 (bcec6818-fec6-40b3-adf1-1fb8eacc13f8)
aar_id: 64751716-e58b-4c44-aaf0-6c2756d55f34
---

# Notes — Editor doc model (forge#249)

## Plan (Phase 1)

**Classification:** work pipeline, feature, medium. THE FOUNDATION of the M15 editable-editor train (#250-258
build on it). AUTONOMOUS (chad's `/goal /work 249 to 258`). The FIRST spec under the #248 `## Reference (§20)`
discipline — filled (Warp-behavior-referenced).

**Intent:** back the active editor file with a real editable `marley_editor::Buffer` + caret + saved_version, so
#250 can render from it, #251 type into it, #252 save it.

**Discovery (Explore — anchors):**
- `marley_editor::Buffer` (crates/editor/src/buffer.rs:17) — `from_text/text/edit(range,replacement,origin:
  EditOrigin)/version()->BufferVersion/len_*/point_at/set_selection`. NO public per-line getter (#250's concern).
  Already the prompt's engine. **Derives NOTHING** (deliberate; buffer.rs:206 test notes "not PartialEq").
- The prompt reuse: `TerminalPane { buffer, caret }` (workspace.rs:264) + `apply_key(&mut buffer, &mut caret,
  key)` (input.rs:58; Enter⇒Submit — the editor wants Enter⇒`\n`, #251's concern).
- `CodeViewState` (code_view.rs:143, derives Debug/Clone/PartialEq/Eq) = (path, lossy pre-rendered lines,
  scroll); raw text consumed+dropped; rendered by code_view_body (app.rs:2536). ALSO the read-only split pane's
  model (`.cloned()` app.rs:5639) → keep it derivable.
- `EditorSurface` (#237, editor_surface.rs:11) = files: Vec<CodeViewState>, active; active_file/_mut/open/close.
  Held by `TabContent::CodeView(EditorSurface)` (tabs.rs:22).
- Both CodeViewState::new sites (app.rs:2284 loader, app.rs:939 restore) have the raw `text` in scope → seed a
  Buffer beside it. Persistence transient (#243 paths only; re-read on restore).

**THE DERIVE FORK — resolved (contained):** the plan-phase grep confirmed `TabContent<S>`/`Tab<S>`/`Project<S>`/
`Workspace<S>` are generic over S and carry NO Clone/Eq/Debug derives (the derives at tabs.rs:439/471/484 are on
small enums like `TabError`, not these). So embedding a non-Clone `Buffer` in `EditorSurface` does NOT cascade
past it. The fork is CONTAINED to `EditorSurface`'s own derives + its tests → design picks (A) add Clone+Debug to
Buffer IF anything needs `EditorSurface: Clone`, else (B) drop `EditorSurface`/`OpenFile` derives (a manual Debug
if a test wants it). LEAN: whichever the compiler + tests demand — likely (B) drop, since the Explore found no
non-test clone/eq on EditorSurface; a marley_editor Buffer-derive (A) is fine too if cleaner.

**Decisions:** D1 OpenFile-in-EditorSurface (not CodeViewState) · D2 derive cascade contained at EditorSurface ·
D3 dirty = version != saved_version · D4 no codec change.

**Risks / load-bearing:**
- The derive fix (D2) is the one real design decision — small, but must not silently drop a needed
  Debug/Clone/Eq. Design greps + compiles.
- `from_utf8_lossy` seed text — an invalid-UTF-8 file's Buffer holds replacement chars → #252 save would rewrite
  lossily. Out of #249 scope (a #252 note); flag it.
- No VISIBLE change in #249 (the model is invisible until #250 renders from it) → driven validation is minimal;
  the visible proof + the Warp observed-capture land at #250.

**Test plan (finalized at design):** REQ-001/003/004 pure units on the EditorSurface OpenFile management (seed a
Buffer from text; dirty toggles on edit; open dedupes by path) with a `()`/test session type; cov/MSI 100. REQ-
002/005 `cargo check --workspace` (the derive cascade compiles; read/split/persist untouched) + review.

**Reference (§20):** Warp command-input buffer+caret model (behavior); the file-editor container is
Marley-specific; observed capture deferred to #250 (Mac locked). Filled in the spec.

**AAR:** 64751716-e58b-4c44-aaf0-6c2756d55f34 (opened).

**Phase 1 status: Plan PASS — autonomous (M15 /goal). Ready for Phase 2 — Design.**

## Design (Phase 2)

**## Reference (§20) confirmed:** Warp's command-input buffer+caret model — this design MATCHES it by reusing
the SAME `(Buffer, caret)` shape the prompt already runs (marley_editor + ropey, Apache/permissive), reimplemented
for a file; the file-editor container is Marley-specific. No Warp/Zed source read. Observed capture → #250.

**Architecture.** A gpui-free model refactor in `editor_surface.rs` (the #237 PURE surface) + 2 app.rs seed
sites. No render/input change (that's #250/#251).

### THE DERIVE FORK — RESOLVED: **Fork B (drop EditorSurface's derives).** Evidence (grepped):
- `EditorSurface` (editor_surface.rs:10) derives `Debug, Clone, PartialEq, Eq`. Its ONLY non-test uses are
  `EditorSurface::new` (tabs.rs:64) + `from_files` (app.rs:959) [ctors] + `TabContent::CodeView(EditorSurface)`
  [held]. **`TabContent<S>` (tabs.rs:22) has NO derive** (generic over S), so nothing upstream needs the surface
  Clone/Eq/Debug. The tests `assert_eq!` on `active_index()`/`.path` (usize/PathBuf), NEVER on the surface. →
  **Nothing requires EditorSurface: Clone/PartialEq/Eq/Debug.** Drop all four; `OpenFile` (holding a non-Clone
  `Buffer`) carries no derives either. NO `marley_editor` change (the :206 test confirms Buffer is intentionally
  not PartialEq — a version-sensitive `==` is a footgun; we don't need it). The implementer confirms by
  `cargo check --workspace`.

### The model
```
struct OpenFile { view: CodeViewState, buffer: Buffer, caret: CharOffset, saved_version: BufferVersion }
impl OpenFile {
    fn new(view: CodeViewState, text: &str) -> Self {                 // seed from the RAW text (NOT lossy lines)
        let buffer = Buffer::from_text(text); let saved_version = buffer.version();
        OpenFile { view, buffer, caret: CharOffset::zero(), saved_version }
    }
    fn is_dirty(&self) -> bool { self.buffer.version() != self.saved_version }
    fn mark_saved(&mut self) { self.saved_version = self.buffer.version(); }
}
pub struct EditorSurface { files: Vec<OpenFile>, active: usize }   // NO derives
```
- **Ctors take the raw text** (lossy `lines` can't reconstruct the buffer): `new(first: CodeViewState, text:
  &str)`; `from_files(files: Vec<OpenFile>, active)` (the caller builds OpenFiles — it has each file's text);
  `open(state: CodeViewState, text: &str)`.
- **D-open (deliberate change):** `open` on an ALREADY-OPEN path now just **activates** it (switch), preserving
  its buffer — it no longer REPLACES the CodeViewState (the old #237 disk-refresh-on-reopen would clobber unsaved
  edits). New path → append `OpenFile::new(state, text)`. (Disk-vs-buffer refresh/conflict is a later concern.)
- **Accessors:** `active_file() -> &CodeViewState` = `&files[active].view` (UNCHANGED sig → render/persist/tab-
  strip untouched); `active_file_mut() -> &mut CodeViewState` = `&mut files[active].view` (UNCHANGED sig → the
  scroll handler tabs.rs:118 → app.rs:4934 still works). NEW: `active_buffer_mut() -> &mut Buffer`,
  `active_caret() -> CharOffset`, `active_caret_mut() -> &mut CharOffset`, `active_saved_version()`,
  `active_mark_saved()`, `active_is_dirty() -> bool` (feed #251/#252).
- **`files()`** returned `&[CodeViewState]`; now the store is `Vec<OpenFile>`. Change to
  `files() -> impl ExactSizeIterator<Item = &CodeViewState> + '_` (map `|f| &f.view`) so the tab-strip
  (app.rs:4858) + persist (app.rs:2213) callers (which ITERATE the views for paths) keep working + retain
  `.len()`/`.enumerate()`. If a caller INDEXES `files()[i]`, add `file_at(i) -> Option<&CodeViewState>` (impl
  traces the 2-3 callers).

### File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/editor_surface.rs | ADD `OpenFile` (+ new/is_dirty/mark_saved); `EditorSurface { files: Vec<OpenFile>, active }` — DROP the derives; ctors take `text`/`Vec<OpenFile>`; `open` dedupe→activate (preserve buffer); `active_file()/_mut()` via `.view`; `files()` → view iterator; NEW active_buffer_mut/caret/saved/dirty accessors. Update the tests (they build via `new(cv, "x")`). |
| crates/marley_app/src/app.rs | loader seed (~2284): `EditorSurface::new(cv, &text)` / `.open(cv, &text)` — the raw `text` is in scope. Restore (~939-959): build `Vec<OpenFile>` from the re-read (cv, text) pairs → `from_files(open_files, active)`. Adapt any `files()` indexer. |
| crates/marley_app/src/code_view.rs | UNCHANGED (keeps its derives; still the split-pane model). |
| crates/marley_app/src/tabs.rs | `code_view()/code_view_mut()` unchanged (active_file/_mut sigs preserved); `code_surface`/`Tab::code` pass text if they construct a surface (trace tabs.rs:64). |

### Regression Test Plan
| # | Test (editor_surface.rs `#[cfg(test)]`) | Proves |
|---|---|---|
| T1 | `open`/`new` seed: `EditorSurface::new(cv("/a"), "hello\nworld")` → `active_buffer_mut().text() == "hello\nworld"`, `active_saved_version()==buffer.version()`, `active_caret()==0`, `!active_is_dirty()` | REQ-001 |
| T2 | dirty toggles: edit via `active_buffer_mut().edit(0..0, "x", Human)` → `active_is_dirty()`; `active_mark_saved()` → `!active_is_dirty()` | REQ-003 |
| T3 | dedupe: `open(cv("/a"), …)` on an already-open `/a` → `files().len()` unchanged + active switches; the buffer preserved (edit, re-open, buffer still has the edit) | REQ-004 |
| T4 | the existing `open_appends_then_dedupes` / `close_clamps` / `activate` / `from_files` tests — ported to the new ctors (assert on paths/index, unchanged) | REQ-002 (surface mgmt) |
| — | `cargo mutants --list -f editor_surface.rs` (post-impl) — the new accessor/seed mutants; kill with T1-T3. | MSI 100 |
| REQ-002/005 | `cargo check --workspace` — the derive-drop compiles; render/split/persist paths untouched. | compile + review |

**Uncoverable / deferred:** no visible change in #249 (the model is invisible until #250 renders from the
buffer) → driven validation is minimal (a note); the visible proof + the Warp observed-capture land at #250.

**Risks:** (1) the `files()` iterator change — trace every caller (impl). (2) the `from_utf8_lossy` seed →
invalid-UTF-8 files hold replacement chars → #252 save-back lossy (a #252 note, out of scope). (3) D-open no
longer disk-refreshes on reopen — deliberate (preserve edits); noted.

**Phase 2 status: Design PASS — Fork B resolved (drop derives, no marley_editor change), OpenFile model +
accessors + seed sites + test plan locked. Ready for Phase 3 — Implement.**

## Implement (Phase 3)

**Built (to the manifest):**
- **editor_surface.rs** — the core refactor, exactly as designed. `struct OpenFile { view, buffer, caret,
  saved_version }` (private) + `OpenFile::new(view, text)` / `is_dirty()` / `mark_saved()`. `EditorSurface {
  files: Vec<OpenFile>, active }` with the **derives DROPPED** (Fork B). Ctors `new(first, text: &str)` /
  `from_files(files: Vec<(CodeViewState, String)>, active)` / `open(state, text: &str)` (dedupe→activate,
  preserving the buffer — the deliberate D-open change). `active_file()/_mut()` sigs UNCHANGED (via `.view`).
  `files() -> impl ExactSizeIterator<Item = &CodeViewState>` (was `&[CodeViewState]`). NEW accessors:
  `active_buffer_mut`, `active_caret`, `active_caret_mut`, `active_saved_version`, `active_mark_saved`,
  `active_is_dirty`. Existing tests ported to the new ctors; added the ONE doc-model test T1/T3
  `open_file_seeds_buffer_and_tracks_dirty` (design's manifest called it out for the implement port).
- **app.rs** — `load_code_view_state` now returns `Option<(CodeViewState, String)>` (the raw text threaded
  out, since it's consumed inside). `open_file_in_viewer` passes `&text` to `open_or_switch_code`.
  `split_file_pane` ignores the text (`if let Some((cv, _text)) = …` — the read-only split pane is unchanged).
  The persist site + tab-strip use `surface.files()` as an iterator (dropped `.iter()`). The restore arm
  builds `readable: Vec<(usize, CodeViewState, String)>` → `files: Vec<(CodeViewState, String)>` →
  `from_files(files, new_active)`.
- **tabs.rs** — `Tab::code(title, state, text)` + `open_or_switch_code(state, text)` thread the text to
  `surface.open` / `EditorSurface::new`. `code_surface` unchanged.
- **code_view.rs** — UNCHANGED (keeps its derives; still the read-only split-pane model).

**Fork confirmed:** the design's Fork B (drop `EditorSurface`'s derives) is CORRECT — `cargo check --workspace
--all-targets` is clean with NO derive on `EditorSurface`/`OpenFile` and NO change to `marley_editor::Buffer`.
Nothing in non-test code needs `EditorSurface: Clone/PartialEq/Eq/Debug` (confirmed by the compiler, not just
the grep).

**Deviation from design (minor):** `from_files` takes `Vec<(CodeViewState, String)>` PAIRS (not
`Vec<OpenFile>`) so `OpenFile` stays private to the module — the design flagged this as the leaning; adopted.

**Two borrow-conflict fixes in ported tests (E0502, not design bugs — a mechanical consequence of `files()`
becoming an iterator whose temporary holds the borrow across the statement):** (1) `active_saved_version()` +
`active_buffer_mut().version()` in one `assert_eq!` → bound the version to a local first. (2)
`s.activate(s.files().len())` → `let n = s.files().len(); s.activate(n);`. Both are test-only; the production
accessors are unaffected.

**Compile/test:** `cargo check --workspace --all-targets` clean; `cargo nextest run -p marley
editor_surface:: tabs::` → 37/37 pass incl the new `open_file_seeds_buffer_and_tracks_dirty`. (Full suite +
mutation is Phase 4.)

**Phase 3 status: Implement PASS — workspace compiles, tests green, Fork B + from_files-pairs confirmed. Ready
for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

2 parallel general-purpose critics over the diff (Critic 1 = correctness + data/state integrity; Critic 2 =
simplification/reuse + clean-room), + self-review. **0 HIGH, 0 MEDIUM. 2 LOW folded. The model refactor is
correct** — both critics independently confirmed the load-bearing checks. `cargo check -p marley --all-targets`
exits 0 after the fixes.

**The load-bearing checks — CONFIRMED CLEAN (Critic 1):**
- **(a) seed from RAW text, not lossy lines** — both ctor sites (`load_code_view_state` app.rs:~2285 + the
  restore arm app.rs:~938) build `CodeViewState::new(full, &text, …)` AND thread that SAME `text` to
  `OpenFile::new` → `Buffer::from_text` (`Rope::from_str`, char-perfect). `code_lines` (code_view.rs:60) is
  provably lossy (tab-expand + truncate-at-CODE_MAX_COLS + drop `\n`) — never fed to the buffer. A buffer from
  `.lines` would corrupt #252 save-back; it isn't. **REJECTED as a defect.**
- **(b) clean-at-load** in all 3 ctors (`saved_version = buffer.version()` in `OpenFile::new`, routed by
  new/from_files/open). `from_files` re-derives per file (transient buffer). REJECTED.
- **(c) D-open** — the read-only split pane (`split_file_pane` → `PaneContent::CodeView(cv.cloned())`) is a
  SEPARATE path, untouched by `EditorSurface::open`. Grepped every `open_or_switch_code`←`open_file_in_viewer`
  caller (⌘↵ finder, output hit, file-link, file-tree, listener) — all user-initiated opens; NO caller relied
  on reopen-refreshes-from-disk (no file-watcher exists). Behavior change is deliberate + no caller breaks.
  REJECTED.
- **(d) close/activate clamp math** byte-identical, index-based, still correct over `Vec<OpenFile>`. REJECTED.
- **(e) files() iterator** — every caller (persist paths, tab-strip enumerate, tests) uses `.len()/.map()/
  .enumerate()`, none indexes; iterate in tab order. REJECTED.
- **(f) no new panic path** — `from_utf8_lossy` is the known out-of-scope #252 lossy-save note, not a crash.
  REJECTED.
- **(g) active_* OOB** — `active` provably in range (ctors clamp, close clamps, activate guards); the only
  empty-vec route to `from_files` is guarded by `if !readable.is_empty()` (app.rs:958, 1:1 with `files`).
  REJECTED as a live bug → see F2 (defense-in-depth lock added anyway).

**Clean-room + simplification — CONFIRMED CLEAN (Critic 2):** single seed point (`Buffer::from_text` appears
once, in `OpenFile::new`); `OpenFile` stays PRIVATE (tuple seam, not `pub OpenFile`); no needless clones (text
moved into the tuple, borrowed into the buffer, dropped — the one `from_utf8_lossy().into_owned()` is
necessary); `marley_editor` is the in-repo Apache/permissive crate (already the prompt's engine), NO Warp/Zed
source — behavior-shape match only; dropped `EditorSurface` derives are safe (whole-crate grep: no `{:?}`/
`.clone()`/`==` on a surface; `TabContent<S>`/`Tab<S>`/`Project<S>`/`Workspace<S>` carry no derives) and
`cargo check --all-targets` exits 0 = definitive; `split_file_pane`'s `_text` is correct (read-only pane, #258
follow-up), not a dropped wire-up.

**Findings folded (2 LOW):**
- **F1 [LOW] stale doc drift** — `open_or_switch_code` (tabs.rs:275) still read "refresh + activate an existing
  path"; #249 changed `open` to SWITCH-preserving-buffer (its own doc was updated; the caller doc lagged).
  **VERDICT real** (doc/behavior mismatch, no runtime effect). **FIX:** reworded to "M15 #249: SWITCH to —
  preserving its buffer/caret — an already-open path, else a new file-tab". Prevention: when a method's
  behavior changes, re-grep its callers' docs (PR-claude-update-caller-docs-when-method-behavior-changes).
- **F2 [LOW] unguarded ≥1-file invariant in `from_files`** — every `active_*` accessor + `active_file()`
  indexes `self.files[self.active]` with no bounds check; the ≥1-file contract lived only in a doc comment.
  Currently unreachable (the sole caller guards emptiness) → NOT a live bug, but a real latent OOB panic vector
  a future `from_files` caller could trip far from the cause. **VERDICT real-latent.** **FIX:** added
  `debug_assert!(!files.is_empty(), …)` at the top of `from_files` — locks the invariant at the source, makes a
  future violation fail immediately + self-documented. Verified via `cargo mutants --list -f
  editor_surface.rs`: the debug_assert generates NO mutant (macro is opaque to cargo-mutants); the only listed
  `from_files` mutant is `-> Default::default()`, UNVIABLE (no Default derive — the #204 no-Default rule) →
  MSI unaffected. Validate adds a `#[should_panic]` `from_files(vec![], 0)` test to cover the panic arm (region
  cov 100) + document the invariant.

**INFO (no action):** re-open on an already-open path discards the freshly-read `(state, text)` (wasted
`fs::read`+decode on a user-initiated, size-capped path) — inherent to dedupe-by-path + the deliberate
preserve-unsaved-edits tradeoff; not a defect. `BufferVersion` derives `Copy/PartialEq/Eq` so dirty-tracking
(`!=`) + return-by-value are sound; `edit()` bumps `version` so dirty actually fires.

**No forge failure-record** — neither finding is a shipped runtime bug (F1 doc-only, F2 unreachable-latent
hardening); the F1 prevention note is captured locally (forge not wired for this session). Lenses covered:
correctness, data/state integrity, panic/OOB, simplification/reuse, clean-room/provenance, dead-code.

**Phase 3.5 status: Inspect PASS — 0 HIGH/MED, 2 LOW folded (F1 doc, F2 invariant lock), compile clean, MSI
unaffected. Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added (editor_surface.rs `#[cfg(test)]` — 8 total, 5 ported + 3 new):**
- `open_file_seeds_buffer_and_tracks_dirty` (T1/T2, REQ-001/003) — seed from raw text, caret 0, clean at load;
  edit → dirty; mark_saved → clean; caret independently mutable. (added at implement)
- **`dedupe_preserves_buffer_on_reopen`** (T3, REQ-004) — NEW: edit /a's buffer, re-open /a with DIFFERENT
  text → no new tab, switches back, **buffer keeps the EDIT** (proves the #249 D-open preserve-unsaved-edits
  guarantee, the load-bearing behavior change).
- **`active_file_mut_exposes_active_view`** (REQ-002) — NEW: a write through `active_file_mut().scroll` reads
  back via `active_file()` (covers the accessor for region-cov 100).
- **`from_files_empty_panics`** (F2, `#[should_panic]`) — NEW: the debug_assert fires on an empty surface
  (covers the panic arm + documents the ≥1-file invariant).
- ported: `open_appends_then_dedupes`, `close_clamps_and_signals_empty`, `activate_switches_in_range_only`
  (incl the `<`→`<=` boundary kill), `from_files_clamps`.

**Runs (actual):** `cargo nextest run -p marley` → **344 passed, 2 skipped** (no regression). `editor_surface::`
→ **8/8 pass**.

**Mutation (`cargo mutants --list -f editor_surface.rs` → 36 listed):** all VIABLE mutants killed by the tests
(is_dirty true/false/`!=`; mark_saved/active_mark_saved `()`; open `()`/`==`→`!=`/`-1` arith; the full close
clamp set; activate `()`/`<`→`==`/`>`/`<=`; active_caret/caret_mut/is_dirty/index bodies). UNVIABLE (skipped,
don't count — the #204 no-Default rule, verified by grep): `from_files`/`active_file`/`active_file_mut`/
`active_buffer_mut`/`active_saved_version` `-> Default::default()` — `EditorSurface`/`OpenFile` derive no
Default, `CodeViewState` derives Debug/Clone/PartialEq/Eq (no Default), `Buffer` derives none, `BufferVersion`
derives Debug/Clone/Copy/PartialEq/Eq/PartialOrd/Ord/Hash (no Default). Gate ran the real mutants → **MSI ≥
100%**.

**DRIVEN — REAL CAPTURE (the mac UNLOCKED this session; NOT the env-blocked fallback):** bundled + `open
target/Marley.app` (pkill stale first). Two captures Read + asserted:
1. **Boot/restore** (`marley-249-boot.png`) — Marley restored a **3-file editor surface** (`buffer.rs`,
   `lib.rs`, `movement.rs` tabs, `movement.rs` active) via the session-restore **`from_files`→`OpenFile`
   seeding path I changed — NO crash; the active file renders correctly (syntax-highlighted `movement.rs` body
   + line numbers via the unchanged `active_file()`). Proves REQ-002 (render unchanged) + REQ-005 (restore/
   persist path intact) + the `files()` iterator (tab strip in tab order).
2. **Live open** (`marley-249-open.png`) — clicking `selection.rs` in the file tree **appended a 4th tab**
   (now active) and rendered its real source (`pub struct Selection { anchor, head }`). Proves the live open
   path end-to-end: `open_file_in_viewer`→`load_code_view_state` (now returns `(CodeViewState, String)`)→
   `open_or_switch_code(state, &text)`→`surface.open` appends an `OpenFile` seeded from the file's RAW text
   and renders it.
   The #249 doc model (buffer/caret/saved_version) is invisible by design (foundation for #250+), but the
   render + open + restore paths it must NOT break are proven intact on the live app.
   (Side note: the mac being unlocked now unblocks the deferred #247 launcher-pixels + #246 split-to-file
   re-verifies — OUT of #249 scope; left for a follow-up.)

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 (incl gate:4 coverage ≥ 100% lines, gate:5
mutation MSI ≥ 100%, gate:6 miri, gate:15 visual/AX). The receipt for `/commit` is written.

**Pre-existing (not in scope):** the `block v0.1.6` future-incompat warning (a transitive dep) — unrelated.

**Phase 4 status: Validate PASS — 8 unit tests (cov/MSI 100 on the pure seam), 344 marley tests green, 2 real
driven captures (boot-restore + live-open), GATE GREEN [diff]. Ready for Phase 5 — Complete.**
