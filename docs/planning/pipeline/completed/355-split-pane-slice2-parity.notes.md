# Split-pane slice-2 parity (#355) — Notes

- **Forge ticket:** #355 (9ef38bb2-3b70-4c56-b268-1e3288e74a2a)
- **AAR:** 09df44b8-981a-4eab-ad46-38df45ef0f4c
- **Pipeline spec:** 355-split-pane-slice2-parity.spec.md

## Phase 1 — Plan (the investigation)
- **Finding:** the recon's "slice 2 = ~12 features to port, scoped tab-side (D-CORE-EDITING-ONLY-V1)" is
  SUPERSEDED. Verified against the code that every slice-2 overlay is focus-aware + top-level:
  - `hover_card_overlay` (app.rs:11980), `completion_popup_overlay` (12055), `signature_overlay` (12150),
    `rename_draft_overlay` (12213) → read `self.active_editor()` + `self.editor_geom.get()` (both focus-written
    by the pane's `editor_draw_for`).
  - `def_picker_overlay` (12304), `code_action_overlay` (12474), `references_overlay` (12369),
    `goto_overlay` (12268), the ⌘⇧O (`open_file_symbols`) + ⌘T (`workspace_symbols`) pickers → state-driven
    overlays populated by focus-aware handlers.
  - the ⌘F `efind` bar (app.rs:17452) → `efind_open` + `editor_geom`; `fold_projection` (10612) → `active_editor()`.
  - #259 removed EVERY `active_tab().editor()` chain (its F1); the editor-TAB block's only feature overlay was
    the banner, wired to the pane by #357.
- **Classification:** work pipeline; a VERIFICATION ticket (prove + guard parity). No feature re-port expected.
- **Decisions:** D1 verify-not-re-port · D2 representative subset (⌘⇧O/⌘F/fold) proves the shared mechanism ·
  D3 a failing drive = a real gap → fix.

## Phase 2 — Design
- **Drives (headless_drive.rs), extending the #259 `split_file_pane_for_test` setup:**
  - `slice2_go_to_symbol_in_focused_split_pane_headless` — split a pane on a file with symbols, focus it, drive
    ⌘⇧O, assert `open_file_symbols` populated from the PANE's file + Enter jumps the PANE's caret.
  - `slice2_find_in_focused_split_pane_headless` — split, focus, ⌘F + type, assert `efind_matches` over the
    PANE's buffer.
  - `slice2_fold_in_focused_split_pane_headless` — split on a foldable Rust file, focus, ⌥⌘[, assert the PANE's
    fold projection hides rows.
- **Observables:** `open_file_symbols` (state) + caret (via `active_editor`); `efind_matches`; `fold_projection`
  / visible-row count. Mirror the existing per-feature headless tests for the drive keystrokes + asserts.
- **Test plan:** REQ-001..003 as above; if a drive FAILS → a real gap → fix at the source.

## Phase 3 — Implement
- Added 3 headless DRIVES + the `open_split_pane_file` helper (headless_drive.rs):
  `slice2_go_to_symbol_in_focused_split_pane_headless` (⌘⇧O → picker from the PANE's file, Enter jumps the PANE's
  caret), `slice2_find_in_focused_split_pane_headless` (⌘F → `open-editor-find` opens ON the pane, find matches
  the PANE's buffer), `slice2_fold_in_focused_split_pane_headless` (⌥⌘[ → folds the PANE's region).
- **NO production gap found** — parity is delivered by #259 (focus-aware accessor + `editor_geom`) + the
  top-level focus-aware overlays + #357 + #356. The find drive initially returned 0 (`refresh_efind_matches`
  needs the bar OPEN — app.rs:13269); opening it via `open-editor-find` — which gates on `active_editor()`, so
  it opens ON the pane — fixed it AND strengthened the assertion (a count of 2 proves ⌘F opened on the pane).
- **All 3 pass.** Test-only change (no production code).
## Phase 3.5 — Inspect
One general-purpose critic (test-only change). **Verdict: CLEAN** — the drives genuinely exercise a focused
split pane + are regression-sensitive; the "no gap" conclusion holds. **2 LOW → FIXED.**

| Lens | Verdict |
|---|---|
| Do the drives exercise the PANE, not an editor tab? | CLEAN — `open_split_pane_file` boots a terminal-ONLY workspace (`seed_one_project` = `T=t`, no `V=`), and `split_file_pane` sets `self.focused = the new CodeView pane`; `active_editor()` = `tab.editor()`(None) `.or_else(focused_editable_surface())` = the pane. No editor-tab surface to false-positive. |
| Regression-sensitive? | CLEAN — the critic simulated `active_editor()`→tab-only: all 3 FAIL (`fs_open` false; efind count 0; fold `before 0 != 8`). The fold numbers (8→5) mirror the editor-TAB fold test with identical `FOLD_SRC`/caret — surface is the only variable. |
| Find mechanism | CLEAN — `count==2` proves the pane resolved (`refresh_efind_matches` force-closes + returns 0 if `active_editor()` None) AND matched the pane's buffer. |
| "No gap" for the undriven LSP features | CLEAN — hover/completion/def/refs/rename/signature ride the SAME `active_editor()`+`editor_geom`; `editor_geom`'s ONE write is in `editor_draw_for`, called for BOTH the tab AND the focused split pane (app.rs:16656). No tab-only path. |
| 2 LOW → FIXED | (a) the go-to-symbol drive asserted BEFORE `reap_sessions` (a failed assert would hang on the PTY drop) → collect-then-reap-then-assert; (b) the find comment claimed "open-editor-find gates on active_editor" (imprecise) → corrected to the `refresh_efind_matches` force-close mechanism. |

No `failure-record` — test-only, no bug (parity was already delivered; the deliverable is the regression net).
## Phase 4 — Validate
- The 3 slice-2 drives (+ the harness-safe LOW fixes) all pass; the full suite (158 tests) green.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** (3:35) — 15/15. Test-only change → no new
  production cov/MSI; the drives run green. Receipt written.
- The LSP-server slice-2 features (hover/completion/def/refs/rename/signature) are covered by the shared
  `active_editor()`+`editor_geom` mechanism the critic verified (`editor_geom` is written for the focused pane
  via `editor_draw_for`), not a live-LS drive (`fake_ls` = #320) — the documented D-out boundary.
- Pre-existing: none.
## Phase 5 — Complete
- **Docs (§21):** `CHANGELOG.md` → a `### Added` entry (#355); `app_shell.md` → a #355 note on the #259 section.
- **AAR (forge):** `aar-submit` completed, effectiveness **5** (the investigation reframed a "12-feature port"
  into a VERIFICATION — parity was already delivered; the deliverable is the 3-drive regression net). No
  `failure-record`.
- **Lessons:** a recon estimate (D-CORE-EDITING-ONLY-V1: "~12 features to port, tab-side") can be SUPERSEDED by
  the architecture that actually landed (focus-aware accessor + top-level geom-anchored overlays) — VERIFY the
  actual gating before porting, don't inherit a stale scope. The find-drive's initial 0 taught the efind
  bar-open precondition (`refresh_efind_matches` no-ops when closed) — a genuine regression net.
- **Close:** forge #355 → done; ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.
