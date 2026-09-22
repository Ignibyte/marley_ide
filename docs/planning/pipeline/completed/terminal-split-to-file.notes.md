---
pipeline_id: 6d3239d2-fc4b-40fa-bea0-abbe8bec2a07
ticket: forge#246 (ac5df912-b46a-4dd8-b2af-1f23c2fac7ab)
aar_id: 78f856d1-3ec4-4f05-b9db-b866b9b1f0b4
---

# Notes — Terminal pane split-right to a read-only file view (forge#246)

## Plan (Phase 1)

**Classification:** work pipeline, feature, SMALL. THE LAST in-range ticket of the M14 round. v1 read-only —
independent of the HELD #242 editable editor. Autonomous auto-approved (M14 /work 238-247).

**Intent:** chad's "can you have a terminal open + split right to a file?" Today `split_focused` always spawns a
PTY. Add a read-only file view beside a terminal within one tab.

**Discovery (Explore — verdict SMALL, the pane model is content-agnostic):**
- `PaneContent::CodeView(CodeViewState)` EXISTS (workspace.rs:279); `PaneState::code_view() -> Option<&CodeViewState>`
  (workspace.rs:336, `&self` seam); PaneKind::CodeView first-class.
- `open_pane(axis, dir, content: PaneContent<S>) -> PaneId` (workspace.rs:471) — the SESSION-LESS no-PTY split
  (focuses the new pane); "how files/code/git panes enter the grid". Content-agnostic → CodeView works today.
- Render revival site = app.rs:5566-5586 (the session-less `else` of the terminal branch at 5080; today only
  `if let Some(PaneKind::Git) = kind` draws; CodeView falls through drawing NOTHING). Add a CodeView arm →
  `code_view_body(cv, colors)` (app.rs:2492, the self-contained read-only render already used by the editor tab).
- Loader to reuse: `open_file_in_viewer` (app.rs:2242) — resolve_under_root (#190) + stat + viewer_size_ok
  (#196) + read + is_probably_binary (#106) + `CodeViewState::new`. Ends in `open_or_switch_code`; the split
  path swaps that for `open_pane`.
- Grid ops (resize #130, focus, close) are PaneId-keyed / content-agnostic → a CodeView pane works in all for
  free. This is why the change is SMALL: a render arm + a thin app method + a trigger.
- Trigger hooks: the palette command list (app.rs:~3631) + `action_for_command` (palette.rs:127) + `dispatch_
  action` (~3343); the ⌘P finder (`handle_finder_key` ~1683, Enter → open_file_in_viewer). The finder is the
  natural file-chooser.

**Landmines (from discovery):**
1. **Persistence** — the serializer records a cwd only for Terminal leaves; a CodeView leaf → bare `'c'` (no
   path); restore (app.rs:851) DROPS CodeView/FileTree leaves → a split file pane vanishes on restart. v1
   NON-PERSISTED (D5); VERIFY the drop is graceful (no crash/malformed grid).
2. **Pane title** — app.rs:5637 hardcodes `project_root.file_name()` for EVERY pane → a CodeView pane reads
   "Marley", not the filename. Fixed by the pure `pane_display_name` (D4).
3. **Borrow shape** — `code_view_body(&self)` vs the loop's `workspace_mut()`; use the `&self` `code_view()`
   accessor or clone the CodeViewState (D6).

**Decisions:** D1 reuse open_pane (no PTY) · D2 reuse the guard ladder · D3 finder-split-mode trigger · D4 pure
pane_display_name (filename for CodeView) · D5 non-persisted + no-crash restore · D6 the `&self` borrow.

**Risks / load-bearing:**
- The pure seam is small (`pane_display_name`) — most of #246 is shim + reuse of heavily-tested pieces
  (open_pane, code_view_body, the guard ladder). Validation leans on the pure seam + the D5 no-crash + DRIVEN.
- **DRIVEN may be ENV-BLOCKED** (the mac locked mid-#247 validation — black capture). If still locked at
  validate: units + mechanism (open_pane/code_view_body are shipped+tested; the split reuses the ladder) +
  re-verify-when-unlocked (`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`). A
  split-pane is very visual → flag the re-verify clearly.
- The finder-split mode flag must be CLEARED on finder close (no stuck "next Enter splits" state).
- D5 — a dropped CodeView leaf in a 2-leaf split: does restore collapse it to the terminal or malform? VERIFY.

**Test plan (finalized at design):** REQ-004 pure `pane_display_name`; REQ-005 the codec/restore no-crash
(build a grid with a CodeView leaf → serialize → restore → assert the terminal survives, no panic); REQ-001/002/
003 driven (+ mechanism/env-block fallback).

**AAR:** 78f856d1-3ec4-4f05-b9db-b866b9b1f0b4 (opened).

**Phase 1 status: Plan PASS — autonomous auto-approved (M14). Ready for Phase 2 — Design.**

## Design (Phase 2)

**Approach.** One pure seam (`pane_display_name`) + shim wiring, all reusing shipped pieces (open_pane,
code_view_body, the guard ladder, the finder). No new subsystem, no PTY. gpui app layer.

**Confirmed anchors (re-read):**
- Render session-less else: app.rs:5566-5586 — `let kind = self.workspace_mut().state(pane_id).map(|s| s.kind())`
  then only `if let Some(PaneKind::Git) = kind` draws; CodeView falls through. Add the CodeView arm here.
- Pane title: app.rs:5637 — `title_name = self.project_root.file_name()…` (hardcoded project basename for EVERY
  pane) → feeds `pane_title(kind, name)` (workspace.rs:216: Terminal/CodeView show the `name`, else fixed).
- `open_pane(&mut self, axis: PaneAxis, dir: SplitDirection, content: PaneContent<S>) -> PaneId` (workspace.rs:471)
  — no-PTY split. `PaneAxis::Horizontal` + `SplitDirection::After` (mirrors split_focused_pane).
- Finder: `handle_finder_key` (app.rs:1663, masked) — the `"enter"` arm: `⌘↵` → `viewer_open_path(&path,false)`
  → `open_file_in_viewer(p)` (1679-1684); plain `↵` inserts the path. The finder is OPENED at app.rs:3272-3273
  (`self.finder = FinderState::new(); self.finder_open = true;`) — the ⌘P path.
- Commands: static ids 0,1,2,4,5,6,7,8 used; **CommandId(9) is TAKEN (`SAVE_WORKFLOW_ID`, app.rs:286)** → the new
  command is **CommandId(10)**. `action_for_command` (palette.rs:127) maps ids→verbs; command list app.rs:~3632.
  dispatch_action (~3343) has the split-right/split-down arms.
- `pane_title(kind, name)` (workspace.rs:216); `PaneState::code_view() -> Option<&CodeViewState>` (workspace.rs:336).

**Decisions locked:**
- **D1/D2** reuse `open_pane` (no PTY) + the guard ladder. **D6** the render arm uses `self.workspace()` (`&self`)
  + `.code_view().cloned()` (an owned CodeViewState) so `code_view_body(&self,…)` doesn't fight the borrow.
- **D3 (trigger) — the finder-split mode, set at OPEN time (robust).** A `finder_split: bool` on RootView (init
  false). The normal ⌘P open (app.rs:3272-3273) ALSO sets `finder_split = false` (a normal finder is not split);
  the new command opens the finder with `finder_split = true`. `handle_finder_key`'s `"enter"` arm: WHEN
  `finder_split` → `split_file_pane(resolved)` + `finder_split = false` (the arm already closes finder_open at
  its end); else the existing ⌘↵/plain-↵ behavior. **This is single-writer-per-open** — the flag is always set
  correctly by whichever open path runs, so NO fragile reset at the 4 finder-close sites (a stuck-true flag
  while the finder is CLOSED is unread — `finder_split` is only acted on inside `handle_finder_key`, and the
  next open resets it). Implementer greps `finder_open = true` to confirm 3273 is the only normal-open site.
- **D4 (pure seam) — `pane_display_name` in workspace.rs** beside pane_title: `pub fn pane_display_name(kind:
  PaneKind, project_name: &str, code_view_path: Option<&std::path::Path>) -> String` = `match kind {
  PaneKind::CodeView => code_view_path.and_then(|p| p.file_name()).map(|s| s.to_string_lossy().into_owned())
  .unwrap_or_else(|| project_name.to_string()), _ => project_name.to_string() }`. The render (5637) computes
  `name = pane_display_name(kind, project_basename, cv_path.as_deref())` and feeds `pane_title(kind, &name)`
  where `title_name` went. (For CodeView → filename; Terminal/Git/no-path → project name, unchanged.)
- **D-command** CommandId(10) "Split Right → File" → `action_for_command(10) => Some("split-right-file")` +
  a command-list entry (mirror CommandId(8) "Split Down") + `dispatch_action("split-right-file")` = open the
  finder in split mode (`self.finder = FinderState::new(); self.finder_open = true; self.finder_split = true;`).
  Every cockpit command must resolve to a verb (#204 invariant) — the action_for_command arm satisfies it.
- **D-refactor — extract `load_code_view_state(&self, path: PathBuf) -> Option<CodeViewState>`** (masked) = the
  open_file_in_viewer guard ladder (resolve_under_root/#190 + stat + viewer_size_ok/#196 + read + is_probably_
  binary/#106 + CodeViewState::new). Refactor `open_file_in_viewer` to `if let Some(cv) = self.load_code_view_
  state(path) { self.open_or_switch_code(cv); self.persist_grid(); }`. `split_file_pane(&mut self, path)` = `if
  let Some(cv) = self.load_code_view_state(path) { self.workspace_mut().open_pane(PaneAxis::Horizontal,
  SplitDirection::After, PaneContent::CodeView(cv)); self.persist_grid(); }`. (DRY; the inspect critic verifies
  open_file_in_viewer is unchanged in behavior. Re-read 2242-2275 for the exact ladder.)
- **D5 (non-persist no-crash)** — a codec round-trip test (serialize a grid containing a CodeView leaf → parse →
  no panic, the terminal leaf survives); the app-level restore-DROP of CodeView leaves (app.rs:851) is the
  shipped #163 behavior (FileTree/Git panes already exercise it) — mechanism. True path-persistence deferred.

## File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/workspace.rs | ADD `pub fn pane_display_name(kind, project_name, code_view_path) -> String` beside `pane_title`. + `#[cfg(test)]` (Phase 4). |
| crates/marley_app/src/app.rs | (1) render CodeView arm (~5581): `let cv = self.workspace().state(pane_id).and_then(|s| s.code_view()).cloned(); … if let Some(cv) = &cv { body = body.child(self.code_view_body(cv, &colors)); }` (switch the `kind` line to `workspace()`). (2) title (5637): compute `pane_display_name(kind, project_basename, cv_path)` → feed `pane_title`. (3) `finder_split: bool` field + init `false` + `= false` at the ⌘P open (3273). (4) `handle_finder_key` enter arm: the `finder_split` branch → `split_file_pane`. (5) `load_code_view_state` (extract) + refactor `open_file_in_viewer` + new `split_file_pane` (masked). (6) CommandId(10) command-list entry + `dispatch_action("split-right-file")` arm (opens the finder in split mode). |
| crates/marley_app/src/palette.rs | `action_for_command`: `CommandId(10) => Some("split-right-file")`. |

Shims (render arm, split_file_pane, load_code_view_state, the finder branch, the command/dispatch) are coverage+
mutation excluded / masked. The ONLY pure seam is `pane_display_name`.

## Regression Test Plan
| # | Test | Proves | Kills |
|---|---|---|---|
| T1 | workspace.rs `pane_display_name_cases`: `(CodeView, "proj", Some(Path::new("/a/app.rs")))` == "app.rs"; `(CodeView, "proj", None)` == "proj"; `(Terminal, "proj", Some(path))` == "proj"; `(Git, "proj", None)` == "proj" | REQ-002/004 | match arms; body→"" |
| T2 | codec no-crash (grid_layout or workspace test): build a grid, `open_pane` a `CodeView(CodeViewState::new(...))` beside a terminal, run the serialize→parse round-trip (mirror existing serialize tests) — assert no panic + the terminal leaf present | REQ-005 | — |
| — | `cargo mutants --list -f workspace.rs` — the viable set for `pane_display_name` (post-impl). | MSI 100 | — |
| REQ-001/003 | DRIVEN (env-block fallback): "Split Right → File" → finder → pick a file → a read-only file pane appears beside the terminal (no PTY); drag the divider (resize); close the file pane. If the mac is LOCKED → units + mechanism (open_pane/code_view_body shipped+tested; split reuses the ladder) + re-verify-when-unlocked. | REQ-001/003 | — |

**Uncoverable by unit:** the render/split/finder shim (gpui `RootView`) — driven-only.

**Risks:** (1) the finder_split single-writer-per-open (D3) — set at every open; a stuck flag is unread while
closed. (2) D-refactor must not change open_file_in_viewer behavior (critic verifies). (3) D5 dropped-leaf
restore graceful (existing #163). (4) borrow shape (D6 — clone the cv). (5) CommandId(10) not 9 (taken).

**Phase 2 status: Design PASS — manifest + test plan + the trigger/pure-seam/refactor/no-crash decisions locked.
Ready for Phase 3 — Implement.**

## Implement (Phase 3)

Built to the manifest; `cargo fmt` + `cargo check -p marley --all-targets` clean (only the pre-existing
unrelated `block v0.1.6` warning).

- **workspace.rs** — `pane_display_name(kind, project_name, code_view_path) -> String` beside `pane_title`.
- **app.rs** — (1) render CodeView arm: the session-less else grabs `code_view = self.workspace().state(pane_id)
  .and_then(|s| s.code_view()).cloned()` (the `&self` accessor + clone, D6) and, after the Git arm, `if let
  Some(cv) = &code_view { body = body.child(self.code_view_body(cv, &colors)); }`. (2) pane title: computes
  `pane_display_name(kind, project_basename, cv_path.as_deref())` → `pane_title(kind, &title_name)`. (3)
  `finder_split: bool` field + init `false` + `= false` at the ⌘P open (the only normal open). (4) the
  handle_finder_key enter branch: `if self.finder_split { split_file_pane; finder_split=false } else if ⌘↵ {…}
  else {…}`. (5) `load_code_view_state` (extracted) + refactored `open_file_in_viewer` + new `split_file_pane`
  (open_pane, no PTY). (6) CommandId(10) command + the `"split-right-file"` dispatch arm (opens the finder in
  split mode). **palette.rs** — `action_for_command(CommandId(10)) => Some("split-right-file")`.

**Deviations from design (with reason):**
- **`load_code_view_state` is `&mut self`, not `&self`.** The guard-ladder's failure arms set `self.status_flash`
  ("can't open X (…)") — that needs `&mut self`. Consequence (a WIN): the split path now shows the SAME
  reject-feedback as the viewer path (consistent UX), and `open_file_in_viewer`'s behavior is byte-preserved.
- **PaneAxis/SplitDirection import** — they're defined in `crate::layout` and only PRIVATELY re-exported from
  `workspace` (the initial `use crate::workspace::{…, SplitDirection}` failed E0603). They were ALREADY imported
  in app.rs from `crate::layout` (line 74-77, used unqualified by `split_focused_pane`); only `pane_display_name`
  was added to the workspace import. `split_file_pane` uses the existing layout-imported names.
- **CommandId(10)** (not the design's placeholder "9") — `CommandId(9)` is `SAVE_WORKFLOW_ID` (#204). The
  action_for_command arm keeps the "every cockpit command resolves to a verb" invariant (#204).

**Phase 3 status: Implement PASS. Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

Two general-purpose critics (refactor-safety + data/no-crash) + my review. Both folded before PASS.

**F1 — [MEDIUM, critic-1, FIXED] the refactor DROPPED `#[cfg_attr(test, mutants::skip)]` from `open_file_in_
viewer`.** Extracting `load_code_view_state` MOVED the skip onto the new helper (+ gave split_file_pane its own),
leaving the refactored `open_file_in_viewer` (a `&mut self` fs-IO method, no unit test can construct a RootView)
UNMASKED → cargo-mutants' "replace body with ()" mutant would survive → gate:5 (MSI 100) RED on the diff. This
is the KNOWN **mutants::skip detach trap** (memory: a structural edit around a masked shim detaches its attr).
**Fix:** restored `#[cfg_attr(test, mutants::skip)]` on `open_file_in_viewer`. **Verified:** `cargo mutants
--list -f app.rs | grep` → open_file_in_viewer / load_code_view_state / split_file_pane all ABSENT (masked).
Recorded `BF-claude-mutants-skip-detached-by-refactor-extract` + `PR-claude-recheck-mutants-skip-after-
refactoring-shims-001`.

**F2 — [LOW, critic-1, FIXED] `finder_split` dangled on ESC.** The design's reset-at-open is provably
sufficient (both critics confirmed not-stuck — the only 2 `finder_open=true` sites both set finder_split), but
ESC (+ the switcher-dismiss) left it `true` until the next open — a future 3rd open path could inherit a stale
`true` and surprise-split. **Fix:** the ESC arm now also `finder_split = false` (the natural "cancel the split
intent" point). Belt-and-suspenders over the robust reset-at-open.

**F3 — [LOW → Phase-4 test work] the pane_display_name test MUST include the CodeView+path case** (load-bearing:
`CodeView + None → project` alone does NOT kill the delete-CodeView-arm mutant, since both real+mutant return
project_name; `CodeView + "/a/foo.rs" → "foo.rs"` kills all 3 viable mutants). Also add the `CommandId(10) =>
"split-right-file"` row to `action_for_command_maps_every_row` (palette.rs) for symmetry. Noted for validate.

**Both critics — the rest CLEAN (concrete cites):**
- **Refactor behavior-preserving (critic-1 lens 1)** — line-by-line vs `git show HEAD:` original: identical guard
  ladder, messages, `CodeViewState::new(full,…)`, `open_or_switch_code`+`persist_grid` on success. No change to
  the ⌘P/file-tree/link viewer path.
- **split_file_pane no PTY + graceful** (open_pane is session-less "cannot fail"; a rejectable file → status_flash
  + no split). finder_split state machine not stuck. pane_display_name correct (incl "/" → project). Borrow sound
  (cloned cv releases the `&self` before code_view_body).
- **D5 no-crash (critic-2 lens 2) — SAFE BY CONSTRUCTION.** restore rebuilds ADDITIVELY from a flat kinds list
  (grid_layout restore_grid → a flat `Vec<PaneKind>`; app.rs restore_panes ADDS panes skip(1), the drop arm
  `FileTree|CodeView => {}` does nothing) — a dropped CodeView is "a split that never happens", NOT a pruned
  tree node → no dangling/zombie/crash. `[terminal|codeview]` → `"H:t,c"` → restores to a clean `[terminal]`
  (cwd intact). `persist_grid` after split is HARMLESS (the terminal layout is not corrupted). #246 is the FIRST
  to persist a droppable leaf in a split (post-#154 FileTree=dock, CodeView=tab; Git is re-opened not dropped)
  — but the additive rebuild makes it safe; restore_panes is untouched by the diff.
- **CommandId(10)** — 10 is free (9=SAVE_WORKFLOW); in cockpit_commands + action_for_command + dispatch_action →
  the `every_cockpit_command_resolves_to_a_verb` invariant (app.rs:7056) holds.
- **Mutation** — pane_display_name: 3 viable (String::new, "xyzzy", delete-CodeView-arm), all killable by the
  F3 test. Clean-room + no reachable unwrap/expect.

**Phase 3.5 status: Inspect PASS — F1 (MED, the skip-detach trap) + F2 (LOW) fixed & verified; D5 proven safe by
construction; refactor behavior-preserving. F3 is Phase-4 test work. Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added — RAN `cargo nextest run -p marley pane_display_name action_for_command_maps` → 2/2 pass:**
- `workspace.rs pane_display_name_cases` — the load-bearing `CodeView + Some("/a/b/foo.rs") → "foo.rs"` (kills
  String::new / "xyzzy" / delete-CodeView-arm) + `CodeView+None → "proj"` + `Terminal → "proj"` + `Git →
  "proj"`. Kills all 3 viable mutants (gate MSI confirms).
- `palette.rs action_for_command_maps_every_row` — added the `CommandId(10) => "split-right-file"` row (F3, the
  explicit-list test).

**REQ-005 (persist+restore no-crash) — mechanism-verified (critic-2's by-construction proof).** Restore rebuilds
ADDITIVELY from a flat kinds list (`restore_grid` → `Vec<PaneKind>`; `restore_panes` ADDS panes skip(1), the
`FileTree|CodeView => {}` drop arm does nothing) — a dropped CodeView leaf is "a split that never happens", not a
pruned tree node → no dangling/zombie/crash. `restore_panes` is UNTOUCHED by the diff. No forced brittle codec
test (serialize_grid needs app context) — the proof + the untouched restore path carry it.

**Driven capture — ENV-BLOCKED (the mac is STILL locked).** Bundled #246, opened it: the app launched ALIVE on
chad's restored session (a terminal + editor files) — so the #246 render changes (the CodeView pane arm + the
`pane_display_name` title) do NOT crash the normal path. But the display is locked — `winforpid` found the window
(19086) yet `screencapture -l` fails "could not create image from window" (the locked-display symptom). Per
`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`: never a password; units + mechanism:
- **REQ-001 (split → read-only file pane, no PTY)** — `split_file_pane` reuses the shipped `open_pane` (session-
  less, "cannot fail", no PTY — critic-confirmed) + the unchanged `open_file_in_viewer` guard ladder; the
  render arm reuses the shipped `code_view_body`. Mechanism-solid; the byte-simple render arm is 3 lines.
- **REQ-002 (title = filename)** — `pane_display_name` unit-proven; the render wires it (byte-simple).
- **REQ-003 (resize/close like any pane)** — the grid ops are content-agnostic (PaneId-keyed — critic-confirmed);
  the × close (`workspace_mut().close(pane_id)`) is the unchanged shipped path.
- **Re-verify when unlocked (30s, no ticket):** ⌘⇧P → "Split Right → File" → finder → pick a file → a read-only
  file pane beside the terminal (titled by filename); drag-resize; ×-close.
- Data safety: this ticket cleared NOTHING — chad's settings.toml is intact (his `shell` key confirmed present);
  the app was quit (not left running).

**Gate — `scripts/gates.sh --diff` → GATE GREEN [diff] 15/15** — cov 100 + MSI 100 on workspace.rs
`pane_display_name` (3 viable mutants killed); `open_file_in_viewer`/`load_code_view_state`/`split_file_pane`
confirmed MASKED (the F1 fix — verified absent from `cargo mutants --list`). One rustfmt red fixed at source (the
test edits post-dated the last `cargo fmt`). No pre-existing failures in scope.

**Phase 4 status: Validate PASS — 2 units (mutants killed), REQ-005 by-construction, the driven env-blocked but
the no-crash boot + shipped-reuse mechanism carry the AC, gate green. Ready for Phase 5 — Complete.**

## Complete (Phase 5)

- **Docs (§21):** CHANGELOG.md `### Added` (above #247) — "Split a terminal right into a read-only file view".
  app_shell.md M14 — a #246 bullet (the content-agnostic pane model; the revived CodeView render arm +
  split_file_pane [open_pane, no PTY] + the finder_split command + pane_display_name; the load_code_view_state
  extraction [behavior-preserved]; v1 non-persisted [additive-restore drop, no-crash]; the mutants::skip detach).
- **Knowledge:** `aar-submit 78f856d1` completed, effectiveness 5 — discovery correctly called it SMALL (the
  content-agnostic pane model); the open_file_in_viewer refactor was behavior-preserving (critic line-by-line vs
  `git show HEAD:`); the HIGH-risk D5 (restore of a dropped CodeView leaf) proven SAFE BY CONSTRUCTION (additive
  restore) by an independent critic; the one real find = the recurring **mutants::skip DETACH trap** (via
  extract-and-relocate) — caught by the refactor-safety critic, fixed, + a sharper rule filed. Recorded at
  inspect: `BF-claude-mutants-skip-detached-by-refactor-extract` (cb67fba3) + `PR-claude-recheck-mutants-skip-
  after-refactoring-shims-001` (bddf2b4a); materialized that + `PR-claude-selftest-locked-screen-blocks-capture-
  fall-back-to-mechanism` (the env-blocked driven fallback).
- **Close:** forge #246 (`ac5df912`) → done; TICKET-246 open→closed.
- **Archive:** spec status `Phase 5 — Complete PASS`; the spec+notes → `pipeline/completed/`.

**Phase 5 status: Complete PASS. Run `/commit` to deliver.**
