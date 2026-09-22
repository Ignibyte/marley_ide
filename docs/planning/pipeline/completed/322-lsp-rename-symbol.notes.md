# LSP rename symbol (F2) — prepareRename + the multi-file WorkspaceEdit engine — Notes

- **Forge ticket:** #322 25a0742a-b2a2-4543-b261-c3b85772f1f9
- **AAR:** eadbc5a6-f9a6-4bc4-b879-22982b95d827
- **Local ticket doc:** docs/planning/tickets/open/TICKET-322-lsp-rename-symbol.md
- **Pipeline spec:** 322-lsp-rename-symbol.spec.md

## Phase 1 — Plan

### Request
F2 renames a symbol everywhere — and builds the ONE WorkspaceEdit apply path that #323 code-actions
and (later) import-organizing formatting reuse. The M21 foundation ticket; the first M21 pipeline.

### Classification / tier
Work pipeline, feature, M21. ONE shippable slice: F2 → prepareRename → inline draft → rename →
WorkspaceEdit parse → multi-file apply (open buffers + closed files) → result flash. The applier is a
foundation #323 depends on.

### Forge recall (§18.3)
- `bulletin-list` → none.
- `knowledge-search` (WorkspaceEdit apply / undo / multi-file rename) → the top hits are the guard/
  apply/wire rules I authored across #310-313, ALL already woven into the ticket + decisions:
  - `PR-claude-external-uri-key-normalize-both-sides-001` (#310) → D5: key by canonical path both sides.
  - `PR-claude-second-consumer-must-inherit-the-first-consumers-guards-001` (#312) → D8: the rename
    apply inherits the live-identity + version guards.
  - `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001` (#313) → P4 asserts the real
    `textDocument/rename` PAYLOAD via the tee, not the method name.
  - `PR-claude-two-gated-calls-must-read-the-phase-once-001` (#312/#309) → the apply reads editor
    identity ONCE.
  - the `mutants::skip` detach trap + `trace-the-real-cargo-mutants-list` → run `--list -f` for the
    real mutant set on the new pure fns.
- ADs in scope: the two new pillar ADs are unrelated to this ticket; the §20 clean-room AD posture
  applies (published LSP 3.17 spec, Zed behavior-only).

### Discovery (Explore, §18.2) — the precise surface (from the M21 code-surface map)
**REUSE AS-IS:**
- `Buffer::edit(range: Range<CharOffset>, replacement, origin) -> EditResult` (buffer.rs:247);
  `begin_undo_group(sel_before)` / `end_undo_group(sel_after)` (buffer.rs:457/463) — subsequent
  `edit`s collect into ONE undo unit. ⚠️ `edit_at_selections` SELF-BRACKETS (buffer.rs:284) → NEVER
  nest it inside an open group; the rename applier brackets its own group around RAW `edit()` calls
  (the #314/Tab-indent precedent).
- `EditOrigin::Agent` (types.rs:13) — the origin for a programmatic (non-keystroke) write.
- `file_uri(path)` / `path_from_file_uri(uri)` (handshake.rs:41/64) — the #310 normalize pair.
- `EditorSurface::open_docs() -> Iterator<(&Path, &Buffer)>` (editor_surface.rs:404); per-project at
  app.rs:1118 — enumerate ALL projects' surfaces for D3.
- The #311 request recipe: `RequestPurpose` (lsp_host.rs:34), `request()` (:452), `take_responses()`
  (:472), the ONE drain `consume_lsp_responses` (app.rs:6116). Raw caps on `Negotiated.server_caps`
  (handshake.rs:31) → gate on `["renameProvider"]`.
- The inline-draft idiom: `renaming_tab`/`naming_workflow` (#177/#204) — type→Enter→Esc.
- `word_query` (editor_complete.rs:144, #313) — the prepareRename fallback range.
- `VIEWER_MAX_BYTES` (app.rs:596) + `is_probably_binary` (code_view.rs:390) — the closed-file skip
  rules (one truth about what Marley considers text).

**NET-NEW (ordered by risk):**
1. **The multi-file applier** — the coordination heart: open-vs-closed routing, per-buffer undo
   groups, closed-file read→apply→write, version + limit skips. Highest risk (the write path).
2. **`apply_text_edits(text, edits) -> Result`** (pure) — sort + last-to-first + overlap-reject. The
   #314-shared primitive (D1). Small, obvious truth table; the mutation surface (reorder/overlap
   boundaries).
3. **WorkspaceEdit parse** (pure) — both shapes → `Vec<FileEdits>`; resource-op reject; per-element
   skip; path normalize.
4. **The closed-file writer** — read→apply→`fs::write`; NO machinery exists (only `save_active`'s
   whole-text write). New, and it touches disk → the careful path.
5. `RequestPurpose::{Rename, PrepareRename}` + keys + the two `apply_*` arms in the ONE drain.
6. The inline rename draft state + the F2 key arm + the result flash.

### Decisions
D1 build `apply_text_edits` here (the #314 core) · D2 resource-ops reject-whole · D3 closed-file
read→apply→write, open-in-any-surface via Buffer · D4 per-file undo, disk-only not undoable · D5 key
by canonical path both sides · D6 hand-parse · D7 prepareRename optional→word_query fallback · D8
inherit the request guards whole. Full text in the spec.

### Risks
- **R1 (highest) — the closed-file write path.** New disk-touching machinery; a bug corrupts a file
  the user didn't have open. Mitigate: the pure `apply_text_edits` is proven at MSI 100; the writer is
  a thin shim over it; size/binary limits gate it; overlap rejects; live-drive byte-checks a real
  closed-file rename.
- **R2 — undo across N files.** Per-file groups (D4) — a rename touching 3 open files = 3 undo units,
  not 1. Honest v1; the flash says N files. A single cross-file transaction is the multibuffer-era
  upgrade.
- **R3 — the open-vs-disk split.** A file open in ANOTHER project's surface must be edited via its
  Buffer, not disk, or the two diverge. D3; enumerate ALL surfaces.
- **R4 — stale apply.** The rename fires a request, the user keeps typing; applying a stale
  WorkspaceEdit edits moved text. D8 (inherit the guards) + the version skip (REQ-009).

### Ticket corrections filed
None — the ticket's dep note ("sequence after #314") is inverted by D1 (build the primitive here);
recorded as a decision, not a ticket error.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture / approach
Two pure layers + a shim, the #311/#312 shape. The whole ticket hangs on **one shared pure primitive**
that resolves LSP edits into char-offset ranges — both the open-buffer apply AND #314's future
formatting reuse it, which is the "one apply path" convergence the batch is designed around.

**The shared primitive (why it is shaped this way):** the server sends edits as LSP `(line, character)`
ranges. The OPEN-buffer path must apply through `Buffer::edit()` (to get the undo group + anchors +
`EditOrigin::Agent`), while the CLOSED-file path applies to a plain `String`. Both need the SAME thing
first: convert each LSP range to a char-offset range (through the #309 encoding bridge), sort
last-to-first, and reject overlap. So the primitive returns the resolved ranges; the two callers differ
only in how they apply them.

**Layer 1 — PURE `marley_lsp/src/workspace_edit.rs` (NEW):**
- `LspTextEdit { range: (Position, Position), new_text: String }`, `Position{line,character}` —
  Marley-local (D6, no lsp-types).
- `FileEdits { path: PathBuf, version: Option<i64>, edits: Vec<LspTextEdit> }`.
- `parse_workspace_edit(&Value) -> Result<Vec<FileEdits>, WorkspaceEditReject>` — `documentChanges`
  (each a `TextDocumentEdit`; a `CreateFile`/`RenameFile`/`DeleteFile` `{kind}` element →
  `Err(ResourceOpsUnsupported)`, the whole-edit reject, D2) OR legacy `changes` (uri→[TextEdit],
  `version: None`); path via `path_from_file_uri` (D5), a non-`file:` uri skips that entry;
  per-EDIT `filter_map` so one malformed edit is dropped not fatal (REQ-003); `null`/absent → `Ok(vec![])`.
- `resolve_text_edits(text: &str, edits: &[LspTextEdit], enc: PositionEncoding) -> Result<Vec<ResolvedEdit>, EditApplyError>`
  — **the heart.** Convert each range to `Range<usize>` (char offsets) via the #309 `position.rs`
  bridge; sort start-descending (last-to-first); reject on overlap (`start < prev_applied_start`
  after sort, i.e. any range whose end exceeds the next range's start) → `Err(Overlap)`; a position
  past EOF → `Err(OutOfRange)`. `ResolvedEdit { range: Range<usize>, new_text: String }`.
- `apply_resolved(text: &str, resolved: &[ResolvedEdit]) -> String` — pure splice, last-to-first
  (already ordered), byte-safe over char offsets.
- `apply_text_edits(text, edits, enc) -> Result<String, EditApplyError>` = `apply_resolved(text,
  resolve_text_edits(...)?)` — the CLOSED-file wrapper + the #314-reusable String primitive (REQ-008).
- `parse_prepare_rename(&Value) -> Option<(Position, Position)>` — `{range}` / `{range,placeholder}`
  → the range; `{defaultBehavior:true}` / `null` → `None` (D7 word-range fallback).
- `rename_params(uri, line, char, new_name)` + `prepare_rename_params(...)` — delegate to
  `text_document_position_params`, rename adds `newName`.

**Layer 2 — PURE `marley_app/src/editor_rename.rs` (NEW):**
- `RenameKey { uri, line, character }` — the stale-guard identity (mirror `DefinitionKey`; no version
  at request time — the per-file version check lives in the applier, REQ-009).
- `rename_summary(applied: usize, skipped: usize) -> String` — `"renamed in N files"` +
  `" (M skipped)"` only when `skipped > 0` (the conditional suffix is a mutation target).
- `version_conflict(edit_version: Option<i64>, synced_version: Option<i64>) -> bool` — `true` only
  when both are `Some` and differ (a `None` on either side = no claim, apply) — REQ-009.

**Layer 3 — SHIM `marley_app/src/app.rs`:** fields `renaming_symbol: Option<RenameDraft>` (the inline
draft — input string + `RenameKey` + the seed range, mirroring `renaming_tab`), `rename_request:
Option<RenameKey>`. Fns (all mirror the #312 definition path):
- `begin_rename()` (F2) → `send_prepare_rename_request()` (or word-range fallback) — Ready+provider
  gated, else flash (REQ-007).
- `apply_prepare_rename_response(key, result)` — the #312 supersede + live-identity guards (D8), then
  open the draft seeded with the parsed range (or the word range).
- draft key handling (type/backspace/Enter/Esc — the `renaming_tab` idiom); Enter → `confirm_rename`.
- `confirm_rename(new_name)` → `send_rename_request(new_name)`.
- `apply_rename_response(key, result, root)` — the guards, then `parse_workspace_edit`; a
  `ResourceOpsUnsupported`/parse-empty → flash; else `apply_workspace_edit(files)` → flash
  `rename_summary`.
- `apply_workspace_edit(files) -> (applied, skipped)` — **the multi-file applier**: per `FileEdits`,
  find an open `Buffer` for the path across ALL surfaces (D3); OPEN → `version_conflict`? skip :
  `resolve_text_edits(buf.text())` → `begin_undo_group` → `buf.edit(range, new_text, Agent)`
  last-to-first → `end_undo_group` (REQ-004); CLOSED → over `VIEWER_MAX_BYTES`/`is_probably_binary`?
  skip : read → `apply_text_edits` → `fs::write` (REQ-005); any resolve/write error → skip + count.
- `RequestPurpose::{PrepareRename(RenameKey), Rename(RenameKey)}` (lsp_host.rs) + the two arms in the
  ONE drain (app.rs:6116 — never a second drain).
- F2 keymap row (KeyContext::Editor); the draft render (reuse the `renaming_tab` inline overlay).

### §20 confirmation
**Zed (the editor).** The design matches the observed behavior — F2 renames every occurrence across the
workspace as one action per file — via the published LSP 3.17 wire (`prepareRename`/`rename`/
`WorkspaceEdit`/`TextDocumentEdit`) plus Marley's OWN `Buffer`, undo groups, and `EditOrigin::Agent`.
The replace-everywhere semantic is the spec's (a `WorkspaceEdit` names explicit per-file ranges), not
read from anyone's source. No Zed GPL source read — clean-room confirmed.

### File manifest
| File | Change |
|---|---|
| `crates/marley_lsp/src/workspace_edit.rs` | NEW pure: `LspTextEdit`/`Position`/`FileEdits`/`ResolvedEdit`/`EditApplyError`/`WorkspaceEditReject`; `parse_workspace_edit`, `resolve_text_edits`, `apply_resolved`, `apply_text_edits`, `parse_prepare_rename`, `rename_params`/`prepare_rename_params`. |
| `crates/marley_lsp/src/lib.rs` | `mod workspace_edit;` + re-exports. |
| `crates/marley_app/src/editor_rename.rs` | NEW pure: `RenameKey`, `rename_summary`, `version_conflict`. |
| `crates/marley_app/src/lib.rs` | `mod editor_rename;`. |
| `crates/marley_app/src/lsp_host.rs` | `RequestPurpose::{PrepareRename, Rename}(RenameKey)`. |
| `crates/marley_app/src/app.rs` | 2 fields; `begin_rename`/`send_prepare_rename_request`/`apply_prepare_rename_response`/the draft key+render/`confirm_rename`/`send_rename_request`/`apply_rename_response`/`apply_workspace_edit`; the 2 drain arms. |
| `crates/marley_app/src/keymap.rs` | F2 → begin-rename (KeyContext::Editor). |
| `crates/marley_app/src/headless_drive.rs` | the drives below. |

### Regression Test Plan (≥1 per REQ)
| Test (loc) | REQ | Asserts |
|---|---|---|
| `workspace_edit::tests::parse_both_shapes` | 003 | `changes` AND `documentChanges` → the same `FileEdits`; version extracted from `documentChanges`; a malformed EDIT skipped while siblings survive; non-`file:` uri skipped; `null`/`{}`→empty. Positions OFF-GRID to kill `Some((0\|1,0\|1))`. |
| `workspace_edit::tests::parse_resource_ops_reject` | 003/D2 | a `documentChanges` containing a `CreateFile`/`RenameFile`/`DeleteFile` → `Err(ResourceOpsUnsupported)` (the WHOLE edit, not a skip). |
| `workspace_edit::tests::resolve_orders_and_rejects_overlap` | 006/008 | last-to-first ordering; ADJACENT edits (end==next.start) SURVIVE; OVERLAP (end>next.start) → `Err(Overlap)`; a position past EOF → `Err(OutOfRange)`. The boundary is pinned exactly (the popup_origin discipline — the `<`/`<=` mutants live here). |
| `workspace_edit::tests::apply_text_edits_last_to_first` | 008 | `apply_text_edits` over a String replaces every range correctly, an earlier edit's offsets unshifted by a later one (proof of last-to-first); an emoji-bearing line pins the encoding bridge (a UTF-16 range maps to the right char offset). |
| `workspace_edit::tests::parse_prepare_rename_shapes` | 001 | `{range}` / `{range,placeholder}` → the range; `{defaultBehavior:true}` / `null` → `None`. |
| `editor_rename::tests::rename_summary` | 004/005 | `(3,0)`→"renamed in 3 files"; `(2,1)`→"…(1 skipped)"; the 0-skipped suffix omitted (the conditional mutant). |
| `editor_rename::tests::version_conflict` | 009 | both Some & differ→true; equal→false; either None→false. |
| `editor_rename::tests::rename_key_identity` | 001 | uri/line/character equality (a differing field ⇒ a different key). |
| `headless::rename_open_buffer_undo` | 004 | feed a `rename` response touching an open buffer → both occurrences replaced; ONE undo reverts the whole file's rename; a superseded/moved response DROPPED. |
| `headless::rename_closed_file_writes_disk` | 005 | a response touching a CLOSED tempdir file → the file on disk reads the renamed text; an over-limit file skipped + counted. |
| `headless::rename_overlap_and_version_skip` | 006/009 | an overlapping file's batch skipped (file unchanged); a version-mismatched open doc skipped; the summary counts both. |
| `headless::rename_no_provider_flashes` | 007 | F2 with no host / no `renameProvider` → a flash, no buffer change, no draft. |
| **A LIVE WIRE CHECK (P4)** | 001/002/004/005 | via the rust-analyzer tee + seeded bundle: F2 a fn used in two probe files → the real `prepareRename` then `rename` frames (assert the `newName`+position PAYLOAD, not the method); BOTH files byte-checked on disk (open one through its buffer, the other closed); ⌘Z reverts the open half. |

**Uncoverable-by-unit:** the inline draft's pixel render (the harness renders no pixels — gpui test
platform `draw()` is a no-op) → carried by the `renaming_tab` reuse + the live capture; the F2 keymap
dispatch → the headless drives exercise it via the dispatch path.

### Risks / decisions
- **R1 (highest) — the closed-file write path.** New disk-touching machinery. Contained: the pure
  `resolve_text_edits`/`apply_text_edits` is proven at MSI 100; the writer is a thin shim over it;
  size/binary limits gate it; overlap/out-of-range reject BEFORE any write; the live drive byte-checks
  a real closed-file rename. **v1 uses plain `fs::write`** (matching `save_active`, not temp+rename) —
  an atomic write for both paths is a named follow-up, stated not hidden.
- **R2 — per-file undo (D4).** A rename over 3 open files = 3 undo units. Honest v1; the multibuffer-era
  single-transaction is the upgrade. The flash says N files so the user knows the scope.
- **R3 — open-in-another-surface (D3).** `apply_workspace_edit` searches ALL projects' surfaces for an
  open buffer before treating a path as closed — else a file open elsewhere gets a disk write that its
  buffer then overwrites on save. The enumerate-all is load-bearing, tested.
- **R4 — stale apply (D8).** `apply_rename_response` inherits the #312 supersede + live-identity
  guards; REQ-009's version check is the per-file second layer.
- **D-des-1 — `resolve_text_edits` is the shared core**, not `apply_text_edits`. The buffer path needs
  char-offset ranges (to feed `buffer.edit` under an undo group), the String path needs the spliced
  result; sharing the resolve step (offset-convert + sort + overlap-reject) is what makes #314's
  formatting a thin reuse rather than a second applier.
- **D-des-2 — every new NAMED pure fn gets a direct unit** (the extracted-helper mutation-surface
  rule); run `cargo mutants --list -f workspace_edit.rs` / `-f editor_rename.rs` for the REAL set.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`marley_lsp/src/workspace_edit.rs` (NEW, pure)** — `LspTextEdit`/`FileEdits`/`ResolvedEdit` +
  `EditApplyError{Overlap,Inverted}`/`WorkspaceEditReject{ResourceOpsUnsupported}`;
  `parse_workspace_edit` (both shapes, resource-op reject, per-edit `filter_map`, `path_from_file_uri`
  normalize, `OptionalVersioned` null→None); `resolve_text_edits` (the heart — `position_to_offset`
  bridge, sort-desc via `Reverse`, adjacency-survives/overlap-rejects window check, inverted reject);
  `apply_resolved` (splice via `char_to_byte`); `apply_text_edits` (the #314-reusable String wrapper);
  `parse_prepare_rename` (3 shapes); `rename_support` (RenameSupport off raw caps);
  `prepare_rename_params`/`rename_params`.
- **`marley_app/src/editor_rename.rs` (NEW, pure)** — `RenameKey` (identity), `RenameDraft` (draft
  data), `rename_summary` (conditional skipped-suffix + singular/plural), `version_conflict`
  (both-Some-and-differ). 3 direct unit tests written (the pure-module mutation-surface rule).
- **`marley_lsp/src/lib.rs` / `marley_app/src/lib.rs`** — module + re-exports.
- **`marley_app/src/lsp_host.rs`** — `RequestPurpose::{PrepareRename,Rename}(RenameKey)`;
  `rename_support()` + `lsp_version_for()` accessors (mirroring `completion_trigger_chars`, caps kept
  encapsulated).
- **`marley_app/src/editor_surface.rs`** — `has_path` + `buffer_for_path_mut` (the cross-surface
  buffer access D3 needs).
- **`marley_app/src/app.rs`** — 2 fields; `begin_rename`/`open_rename_draft`/
  `apply_prepare_rename_response`/`confirm_rename`/`send`-via-`request`/`apply_rename_response`/
  `apply_workspace_edit`/`apply_one_file`/`locate_open_file`; the 2 drain arms; the draft key block
  (mirrors `renaming_tab`); `rename_draft_overlay` (caret-anchored field, block cursor); the
  `text_input_blocked` line.
- **`marley_app/src/keymap.rs`** — F2 → `rename-symbol` (KeyContext::Editor); dispatch arm.

### Deviations from design (with reason)
- **Reused `position::Position`** instead of a new `Position` in workspace_edit — position.rs already
  has the exact type + the `position_to_offset` bridge; no reason to duplicate.
- **`resolve_text_edits` is the shared core** (D-des-1 confirmed in code): the buffer path calls it
  then `buffer.edit` per range; the closed path calls `apply_text_edits` (which wraps it). #314 reuses
  `resolve_text_edits` directly.
- **No focused-file guard on `apply_rename_response`** (unlike hover/definition): a rename EDITS the
  named files by uri, so a tab switch since the request is fine; the supersede key + the per-file
  version check (REQ-009) are the correct guards. Stated in the fn doc.
- **`apply_prepare_rename_response` DOES keep the focused-file guard** — the prepare answer only opens
  a draft AT the caret, so if the user left the file, the draft would open in the wrong place.
- **`locate_open_file` carries `#[cfg_attr(test, mutants::skip)]`** — it walks the live `self.shell`
  (untestable without a full multi-project app), behaviorally verified by the headless open-buffer
  drive. Without the skip app.rs was 69 mutants (its 5 index-return mutants); with it, back to 64.

### Verified
`cargo check --workspace --all-targets` clean; `cargo clippy -p marley -p marley_lsp --all-targets`
**0 warnings** (fixed 2: `sort_by`→`sort_by_key(Reverse)`, a needless `&` on the `Flash::new` arg);
`cargo fmt --all --check` clean. **The `mutants::skip` detach trap did NOT fire** — app.rs still **64**
(`def_label` canary present, no sibling detached). Real mutant sets traced (not guessed —
`PR-claude-trace-the-real-cargo-mutants-list`): **workspace_edit.rs = 33**, **editor_rename.rs = 6**.
`resolve_text_edits`'s overlap window (`w[1].end > w[0].start`) + the sort are the mutation hotspot —
Phase 4 pins adjacency-survives vs overlap-rejects at the exact boundary (the popup_origin discipline).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Three parallel critics over the ~820-line diff (correctness/apply-path, parse-robustness/provenance,
reuse/idiom-fit) + my own independent review. The critics were slow to flush (7-12 min, transcripts
appear only at completion — a stat-poll of the output files misleads, they sit at 156 B then jump), but
all three returned strong findings. **A HIGH bug I would have shipped**, two MEDs, and several verified-clean
confirmations. Critics probe-proved their findings; both MEDs on the write path were caught independently.

### REAL — fixed in #322

| # | Sev | Finding | Fix |
|---|---|---|---|
| **F-CORR-1** | **HIGH** | **An open-file rename misroutes to the blind DISK-WRITE path under a non-canonical (symlinked) project root.** `fe.path` (the server's edit target) is CANONICAL — every path the server learns routes through `LspHost::absolute()`→`canonicalize()`. But `f.view.path` (an open file) is `root.join(rel)` with the root stored VERBATIM (no canonicalize). `has_path`/`locate_open_file` compared them with raw `==`. Under a symlinked root — `/tmp`→`/private/tmp` on macOS, which the drive fixtures and MY OWN live-drive probe use — they differ → the open file reads as CLOSED → `apply_one_file` falls to `fs::read`/`fs::write` on DISK. A clean buffer → the rename lands on disk, buffer untouched, ⌘Z reverts nothing, a spurious conflict banner, yet reports success; a DIRTY buffer → server positions (vs the synced buffer) applied to the last-SAVED disk text → **on-disk corruption** while the buffer shows un-renamed text. | New `same_file(a,b)` in editor_surface.rs comparing CANONICAL forms (fallback to raw `==` when a side is absent), used by `has_path` + `buffer_for_path_mut` — the editor surface adopts the same keying discipline the LSP host already uses. |
| **F-PARSE-1** | **MED (→HIGH if untrusted servers)** | **My OWN first containment guard was bypassable via `..` traversal.** `Path::starts_with` is component-wise and does NOT resolve `..`, so `file:///<root>/../../.zshrc` starts_with the root → the guard passed → `fs::write` resolved the `..` to `~/.zshrc`: a real RCE primitive (append shell code to a dotfile; the file exists so it survives the read-gate + size/binary checks). Symlinks inside the root are a second vector. The naive `file:///etc/passwd` WAS blocked; only traversal/symlink escaped. | Canonicalize the target (the file must exist → `canonicalize()` succeeds, collapsing `..`+symlinks), `starts_with` a canonicalized root, and READ/WRITE the canonical path so the traversal can't sneak back. Reject on canonicalize error. (`marley_project::resolve_under_root` is NOT a containment primitive — it returns absolute unchanged; correctly NOT used.) |
| **F-CORR-2** | **MED** | **`lsp_version_for` is the lone `docs` accessor that skips `absolute()`** — a FAIL-OPEN REQ-009 check. Every other `docs` access (`reconcile`/`needs_text`/`did_save`/`diagnostics_for`) normalizes via `self.absolute()`; the file's own invariant says the store is keyed that way. A spelling divergence the module elsewhere defends against → `docs.get` misses → `None` → `version_conflict(Some(v),None)=false` → the version check is bypassed → a version-stale edit applies to the open buffer at wrong spans. | `self.docs.get(&self.absolute(path))`, matching the siblings. |
| **F-DEAD-1** | MED/LOW | **`parse_prepare_rename` was dead** (both the reuse AND parse critics flagged it independently): `apply_prepare_rename_response` used the prepareRename result only as a `is_null` can-I-rename gate, then re-seeded from the CLIENT `word_query` at the LIVE caret — discarding the server's authoritative range AND (secondary) desyncing the seed from the fire position on a same-file caret move (my SELF-2). | WIRED it: seed the draft from the server's OWN token range (`seed_from_range` via the #309 position bridge), fall back to `word_query` only on `{defaultBehavior}`/None (D7). Fixes the dead export AND the caret-desync in one move. |
| **F-DEAD-2** | LOW | `apply_resolved` was `pub` + re-exported but its only caller is `apply_text_edits` in-module (the #311-F3 rule). | Made private; dropped from the `lib.rs` re-export. |
| **SELF-3** | LOW | An empty-`edits` file on the closed path would `fs::write` identical content — a needless mtime bump that trips the extchange watcher / git. | `apply_workspace_edit` skips a no-edit `FileEdits` (neither applied nor skipped). |

### REAL — deferred with reason (follow-ups, not #322)
- **F-CORR-3 (LOW) — `resolve_text_edits` same-start overlap is order-dependent** (an empty insert `[S,S)` + a replace `[S,S+k)` accept/reject flips on the server's list order). **Not reachable for rename** (occurrences are disjoint full-token replaces — confirmed by the critic), but LATENT for #323 code-actions / #314 organize-imports where insert+replace-at-a-point is plausible. → a note for #323/#314; #322 never produces it.
- **F-CORR-4 (LOW) — non-atomic `fs::write`** on the closed path (a mid-write failure truncates a file the user may not have open). Matches the codebase-wide `save_active` convention → a workspace-wide atomic-write follow-up, not a #322 regression.
- **F-CORR-6 (LOW) — `RenameKey` has no per-request nonce**: two renames at the SAME caret produce an identical key, so a same-caret back-to-back WithPrepare race (a second F2 before the first answer, slow server) could let a superseded rename's WorkspaceEdit pass the guard. Very low reachability; the Basic path is immune. → noted; a nonce is the fix if it ever surfaces.
- **The TOCTOU on a closed file changed on disk between request and response** (no version guard for closed files, which aren't in `docs`) — inherent small window, acceptable v1.

### REJECTED / VERIFIED-CLEAN (independently confirmed by the critics)
- **The pure `workspace_edit.rs` engine is CORRECT** — the correctness critic traced overlap (adjacency survives / true overlap rejects / transitively complete), the inverted-range guard (`position_to_offset` is monotonic → clamping can't invert a valid range), `apply_resolved`'s splice (overlap-rejection guarantees the untouched prefix, so `char_to_byte` on the mutating string is stable), and the buffer last-to-first loop + undo group (records `[high,low]`, `undo().rev()` restores → one ⌘Z reverts the file) — all sound. My own hand-trace agreed.
- **Guard split is correct** (all three critics + me): `apply_rename_response` rightly OMITS the focused-file guard (a WorkspaceEdit names files by uri, so a tab switch is irrelevant; supersede key + version check are the right guards); `apply_prepare_rename_response` rightly KEEPS it (it seeds caret-anchored UI).
- **Provenance CLEAN (§20)**: hand-rolled `serde_json::Value` walking, same idiom as definition.rs/completion.rs — not a `lsp-types`/rust-analyzer typed-struct lift; field names are LSP 3.17 wire names (protocol, not structure); the enums are Marley-local. No secrets, no unsanitized input to a process spawn.
- **Reuse is right**: `begin_rename` reuses `lsp_position_for` (#312); `confirm_rename` reuses `draft.key`; the draft key block faithfully mirrors `renaming_tab` (no ⌘V leak — platform-gated); `version_conflict`/`rename_summary` correctly pull branch logic out of `mutants::skip` shims into tested pure seams.

### Owed to Phase 4 (the critics' one shared LOW: no in-file tests yet)
`workspace_edit.rs` shipped from implement with NO test module — the ONLY pure LSP module without one (the phase split deferred them). **The Phase-4 test plan already has the rows** (both parse shapes, resource-op reject, overlap/inverted/adjacency, u32-overflow-skip, char clamp, prepare shapes, rename_support) + a NEW row owed: **`same_file` with a tempdir+symlink fixture** to kill its 5 mutants and PROVE the F-CORR-1 fix (two symlink-equal files→true, two different→false, a non-existent path→raw fallback). Validate must not skip these.

### Verified after the fixes
`cargo check --workspace` clean; `cargo clippy -p marley -p marley_lsp --all-targets` **0 warnings**;
`cargo fmt --all --check` clean. The `mutants::skip` detach trap did NOT fire — app.rs still **64**
(the two new shim helpers `word_at_caret_seed`/`seed_from_range` are skip'd + did not leak).
`same_file` added 5 mutants to editor_surface.rs (Phase 4's symlink test kills them); `workspace_edit.rs`
still 33.

status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate

## Phase 4 — Validate

### Tests added (18 new; all green — 1299 workspace tests pass)
**`workspace_edit.rs` (9):** both parse shapes agree (version rides documentChanges, documentChanges
wins, non-`file:` skipped); the resource-op reject (create/rename/delete, order-independent even after
a good edit); junk degrades (null/`{}`/`documentChanges:null`/`[null,3,"str"]` + a mixed array keeping
the good); the **u32-overflow SKIP** (a 2^32 line drops the edit, not truncated to 0); resolve orders
last-to-first + **ADJACENCY survives / OVERLAP rejects / INVERTED rejects / empty-insert ok** (the
exact boundaries — the `>`/`>=`/`==`/`<` mutants); apply splices last-to-first + the **multibyte
char↔byte bridge** (`aébc` replace at char 2 lands on byte 3, killing the char_to_byte 0/1 mutants); the
3 prepareRename shapes; the rename_support table (incl. `prepareProvider:false`→Basic, which a
true-only test would miss); the request-params shape.
**`editor_rename.rs` (3, from implement):** rename_summary (conditional suffix + singular/plural),
version_conflict truth table, RenameKey identity.
**`editor_surface.rs` (1):** **`same_file` with a tempdir+symlink fixture** (F-CORR-1) — a symlinked-dir
path matches its real file (true), two different files (false), a non-existent path via the raw
fallback — kills all 5 same_file mutants and PROVES the HIGH fix.
**Headless drives (5):** open-buffer rename + **one ⌘Z reverts the whole file** (REQ-004); closed-file
**written to disk** (REQ-005); overlap → **file skipped, "renamed in 0 files (1 skipped)"** (REQ-006);
a superseded-key answer dropped (the guard); F2 with **no host → "LSP: not ready", no draft** (REQ-007).
The open-buffer drive doubly proves the F-CORR-1 fix — the file opens by canonical path, the
WorkspaceEdit names the canonical uri, and `same_file` routes it to the BUFFER (not a disk write).

**Mutation (traced, not guessed):** `cargo mutants -f workspace_edit.rs` → **24 caught, 8 unviable, 0
missed = MSI 100** (the 8 unviable are `Default::default()` on the non-Default types; the parse_prepare_
rename `&&` mutant was ELIMINATED — the redundant `contains_key` guard duplicated `range_of`'s own
check and made the `&&`→`||` mutant EQUIVALENT/unkillable, so I removed the guard). `same_file`'s 5
mutants are killed by the symlink test (verified by construction; the full gate:5 is authoritative — a
local `cargo mutants -f editor_surface.rs` baseline hit the flaky `resize_real_pty_succeeds` PTY test).

### Keymap roster guard updated
The F2 binding bumped `all_chords` 54→55 and the scoped roster 13→14; both asserts + their #322
breakdown comments updated (the `all_chords_lists_every_binding` regression guard).

### The LIVE drive — DONE, full multi-file rename against real rust-analyzer
Seeded a two-file probe (`greet` defined in `helper.rs`, called in `main.rs`) under a `/private/tmp`
root, opened `helper.rs`, F2 on `greet`:
- **REQ-001** — a real `textDocument/prepareRename` frame at `{line:0, character:8}` (on `greet`); the
  inline draft opened as a framed accent card `rename → greet█`, **seeded with the server's own token**
  (the F-DEAD-1 wiring — not empty, not the client word by luck).
- Typed `ings` → Enter → **REQ-002** a real `textDocument/rename` frame carrying `newName: "greetings"`
  + the position (the PAYLOAD, not the method name — `PR-claude-synthetic-response-tests-never-prove-
  the-live-wire`).
- **REQ-004** — `helper.rs` (OPEN) → its BUFFER became `pub fn greetings()` (dirty, disk unchanged);
  one **⌘Z reverted it** to `greet`. **This is the live proof of the F-CORR-1 fix**: the file sits under
  `/private/tmp` (the symlinked-root case), and `same_file` correctly routed it to the buffer path
  rather than a blind disk write.
- **REQ-005** — `main.rs` (CLOSED, never opened) → written to DISK as `helper::greetings()`,
  byte-checked. The closed-file read→apply→write + the containment-canonicalize path, live.
Chad's `~/.marley/config/settings.toml` restored byte-identical; the app + tee + rust-analyzer reaped.
(`drive.swift` gained `"f2": 120` for the drive — a harness change, committed with the ticket.)

### Gate — one RED (mutation), fixed; re-running for the receipt
First `--diff` run went RED on gate:5 only: **39 caught / 4 missed → MSI 90.7%**. All 4 survivors were
`LspHost::lsp_version_for` (Some(0)/None/Some(1)/Some(-1)) — the ONE new lsp_host accessor I forgot to
`mutants::skip`. EVERY sibling accessor (`uri_for`/`completion_trigger_chars`/`encoding`) carries the
same `// shim: trivial read over a tested seam` skip; `lsp_version_for` is that category (a trivial
`docs.get(absolute(path))` read — its version DECISION is `editor_rename::version_conflict`, tested at
MSI 100, and killing its value needs a synced-doc-version hook that doesn't exist). Added the skip to it
+ `rename_support` (consistent), verified both gone from `--list` with NO detach, re-staged, re-running.
(An honest RED caught + fixed at source — no floor lowered, no suppression.) Coverage had ALREADY
passed this run — workspace_edit 279/279 = 100%, TOTAL 100% (the line-109 `changes`-non-array fix held).

**Re-run GATE GREEN [diff] — 15/15, mutation 39 caught / 0 missed = MSI 100.0%.** cov 100 / miri /
visual all green; 1299 workspace tests. The receipt is written.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 5 — Complete
- **§21 docs:** CHANGELOG `Added` entry; crate-map `marley_lsp` row gained the #322 **WorkspaceEdit
  engine** paragraph; editor.md's "Still missing" corrected to **"Rename (#322, M21) SHIPS"** with the
  canonical-routing + containment notes.
- **Knowledge (forge):** AAR `eadbc5a6` submitted (completed, effectiveness 5; 3 novel findings →
  distillation/drift/pattern-emergence enqueued). The inspect failure + 2 canonicalization prevention
  rules were recorded at Phase 3.5 (`BF-rename-open-file-misroutes-to-disk-under-symlinked-root-001`,
  `PR-claude-compare-path-identity-canonically-not-raw-eq-001`,
  `PR-claude-canonicalize-before-containment-check-on-external-write-path-001`).
- **The through-line lesson:** the LSP host canonicalizes every path it keys by (its docs map, its
  root) — and the two bugs both came from a NEW seam not adopting that discipline: the editor surface's
  raw `==` open-vs-closed routing (HIGH, → `same_file`), and my own first containment guard's
  component-wise `starts_with` (traversal-bypassable, → canonicalize-then-contain). When one subsystem
  canonicalizes its keys, EVERY subsystem comparing against its values must too.
- **Two validate lessons for the batch:** (1) an equivalent mutant is unkillable — `parse_prepare_
  rename`'s `&&` guard duplicated `range_of`'s own check, so the `&&`→`||` mutant produced identical
  output; the fix was to DELETE the redundant guard, not test around it. (2) EVERY new `lsp_host`
  accessor needs the sibling `mutants::skip` — a trivial `docs`/caps read whose logic lives in a tested
  pure seam; forgetting it on `lsp_version_for` cost one RED gate run (4 survivors, all one fn).
- **The reuse dividend for #323:** `resolve_text_edits` + `apply_workspace_edit` + `parse_workspace_
  edit` are the exact seams #323 code-actions calls; the `DefPicker` shape is its action-picker
  template. #323 adds a request + a resolve step + a picker — the apply is done.
- Archived: spec+notes → `completed/`, ticket → `closed/`.

status: Phase 5 — Complete PASS
