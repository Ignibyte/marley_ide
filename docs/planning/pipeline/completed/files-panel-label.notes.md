---
pipeline_id: f9399f07-4685-4339-bf0f-21245d511b69
ticket: forge#190 (7f2d0ce2-60eb-4a97-a83d-a1f922c6660f)
aar_id: fe8cb735-d10e-46d7-bcb7-52c3191549c4
---

# Notes — M12.1 #190 the "Files" panel is the tab navigator

## Phase 1 — Plan
**Live reproduction (debug bundle, this session).** Split a tab 3 ways, then clicked `AEServer.8` in the file
tree (`files_panel`). It WORKED: a code tab "AEServer.8" opened in the rail AND became active AND the center
switched to the code view (23 numbered troff lines rendered). So the file-click functional path is NOT broken —
the ticket's three hypotheses (opened-but-not-switched / body-doesn't-render / read-fails-silently) are all
disproven. Trace confirms: click → `path_at` → `viewer_open_path(&p, false)` (always `Some` for a file) →
`open_file_in_viewer` → reads bytes, guards size/binary, `open_or_switch_code(state)` which pushes/reuses the
code tab AND sets `self.active = idx` (tabs.rs:247). Render dispatch (app.rs:3982-4032): active tab's
`code_view()` → `code_view_body` full-screen. All correct.

**Actual root cause — a stale label.** There are TWO left panels:
1. **Left dock** (`regions.left`, ⌘B, default open) — titled by `dock_title(DockSide::Left)` = **"Files"**
   (layout.rs:63, tested layout.rs:372). But since #152/#154 it renders the **Workspace→Project→Tab navigator**
   (`rail_rows` — terminal 1/2/3/4 + a "Search tabs" filter box, app.rs:3587-3720). Its "Files" title is stale
   — it dates to #56 when the left dock WAS the file tree.
2. **`files_panel`** (app.rs:2151, toggled by 📁 / `files_open`, default `false` — settings.rs:78) — the ACTUAL
   file/directory browser (`self.file_tree.visible_rows()`). It has **no title header**.

So a user hunting for "the file browser" reads the panel labeled **"Files"**, clicks its rows (which are TABS),
and nothing file-opens; the real browser is the unlabeled adjacent panel (and hidden by default on a fresh
config). This is chad's feedback #3 exactly ("I cant seem to click on a file in the file browser to view it").

**Decision (chad away).** AskUserQuestion (relabel / relabel+show-by-default / a-specific-file-failed) timed out.
Proceeding with the **relabel** fix — the essential one for chad, whose config already persists `files_open`
open, so he sees the real browser; he just needs it labeled and the tab dock un-labeled. Dropped the
default-open change (global product call, doesn't affect chad). **Flag the direction + the "Workspace" word for
chad at delivery.**

**Fix (2 edits):**
- `dock_title(DockSide::Left)`: "Files" → "Workspace" (layout.rs:63); update the assertion at layout.rs:372.
- `files_panel`: prepend a "Files" title header (app.rs:2151), matching the `dock_panel` header idiom.

**Regression Test Plan** — see spec REQ-001/002/003. Pure unit on `dock_title`; driven captures for the two
labels + the (working) file-click→code-view path.

**Risks.** Naming taste (D2). The palette command "Files" (cockpit_commands, id at app.rs:3125) toggles
`files_open` — it refers to the BROWSER, so it stays "Files" (correct after this change). Verify no other code
hardcodes the left-dock "Files" label (design step).

## Phase 2 — Design
**Where the label comes from (confirmed).** The left dock's visible "Files" header is rendered by `dock_panel`
(app.rs:327-365) whose header child is `dock_title(side)` (line 362). The left dock's content — the tab tree
(`files` div, "Search tabs" + `rail_rows`, built 3587-3875) — is wrapped in `dock_panel(DockSide::Left, …)` at
app.rs:3877. So `dock_title(DockSide::Left)` IS the string the user reads at the top of the tab panel. The real
browser `files_panel` (app.rs:2151) is placed at app.rs:3898 in a plain `div().bg(surface).border_r_1()` with
**no header**.

**Blast radius (confirmed via grep).** `dock_title` has exactly ONE consumer (`dock_panel`, line 362) — nothing
else hardcodes the left-dock label. The other `"Files"` literals are unrelated and stay: the palette command
"Files" (app.rs:3125) toggles `files_open` (it opens the BROWSER — correct name), and `PaneKind::FileTree =>
"Files"` (workspace.rs:186, the retired tiled file-pane kind). So the rename is isolated to `dock_title` + its
one unit test.

**Approach.**
- `dock_title(DockSide::Left)`: return **"Workspace"** instead of "Files" (layout.rs:65). Pure; the left dock IS
  the Workspace→Project→Tab navigator. Update the unit test (layout.rs:372) to assert "Workspace" — keep the
  `DockSide::Right == "Details"` assertion so an arm-swap mutant still dies.
- `files_panel` (app.rs:2151): prepend a **"Files" caption header** as the body's first child, matching the
  `dock_panel` header style — `.w_full().px_3().py_2().border_b_1().border_color(colors.border)`, caption size
  (`type_scale(Role::Caption)`), `text_color(colors.muted)`, child `"Files"`. Skip-based scrolling re-renders
  rows each frame, so a first-child header stays pinned at the top (doesn't scroll away).
- Fix the stale comment at app.rs:3587 ("renders the project's file tree (#56)") → it renders the Workspace→Tab
  navigator; note the file tree now lives in `files_panel`.

**File manifest.**
- `crates/marley_app/src/layout.rs` — `dock_title(Left)` "Files"→"Workspace"; update the layout.rs:372 assertion.
- `crates/marley_app/src/app.rs` — add the "Files" caption header to `files_panel` (2151); correct the stale
  3587 comment. (Shim render — `mutants::skip`/cov-excluded; proven by driven capture.)

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `dock_title_by_side` (layout.rs, updated) | REQ-001 — `dock_title(Left) == "Workspace"`, `dock_title(Right) == "Details"` (kills the body→`""`/`Default` mutant and an arm-swap) |
| driven capture A | REQ-001 — the tab panel's header reads "Workspace" (not "Files") |
| driven capture B | REQ-002 — the file-browser panel shows a "Files" header atop the directory tree |
| driven capture C (before/after) | REQ-003 — clicking a file row in the "Files" browser opens+shows its code view (regression-lock) |

No trybuild/integration rows — this is a label + one pure string. The two render changes are shim-only (the
app-view is `mutants::skip` + coverage-excluded), so the driven captures are their proof (§7). No uncoverable
gaps beyond the usual GUI-shim (covered by capture).

**Risks / decisions.** D2 naming ("Workspace" vs "Tabs"/"Sessions") — flagged for chad. If the dock title
"Workspace" reads redundantly against the internal "WORKSPACE" section header, that's a taste follow-up (the fix
still resolves the mislabel either way). No behavioral/logic change; purely presentational.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean (only the pre-existing `block v0.1.6` future-incompat
warning).
- `layout.rs` — `dock_title(DockSide::Left)` "Files" → "Workspace"; doc comment records the #190 rationale;
  updated the `dock_title_by_side` test assertion to "Workspace" (kept `Right == "Details"`).
- `app.rs` — `files_panel` now prepends a "Files" caption header (dock_panel header idiom: `px_3 py_2 border_b`,
  Caption size, muted) as the body's first child (pinned above the skip-scrolled rows). Corrected the stale
  3602 comment (left dock = Workspace→Tab navigator, not the file tree).
**Skip-detach check:** none — no new fn was inserted adjacent to a `mutants::skip` shim (a header child + a
pure string swap + comment edits only). **Deviation:** none.

## Inspect (Phase 3.5)
Two independent critics over the diff (correctness+consumer-sweep; simplification+visual-consistency).

**Correctness critic — CLEAN on all 4 lenses.**
- Consumer sweep: `dock_title` has exactly ONE consumer (`dock_panel`); no other test/golden/harness asserts the
  left dock == "Files". The other `"Files"` literals (palette command, `PaneKind::FileTree`) are unrelated.
- **Row-index alignment (the key risk) — SAFE.** The "Files" header is added OUTSIDE the `.enumerate()` loop, so
  it consumes no row index; `path_at(index)` keys on `visible_rows().enumerate()` (data-derived), so clicks open
  the correct file. Header is non-interactive → can't swallow row-0's click. I verified this independently too.
- Build + `dock_title` test pass.
- [LOW] no automated test for the render header (REQ-002/003) — by design (`files_panel` is `mutants::skip` +
  cov-excluded shim); the driven captures in Phase 4 are its proof. No fix.

**Simplification critic — 3 real polish items, all fixed:**
- [LOW] **Header block duplicated verbatim** vs `dock_panel` (must-stay-identical coupling → drift risk).
  **FIXED:** extracted `fn caption_header(colors, label: impl Into<SharedString>) -> Div` (`mutants::skip`, a
  render shim); both `dock_panel` and `files_panel` now call it. Inserted ABOVE `dock_panel`'s doc/skip so
  dock_panel's `mutants::skip` stays attached (verified — see below).
- [LOW] **4px header inset** — `files_panel`'s header sat 4px low (body's `.py_1()`) while `dock_panel`'s header
  is flush. **FIXED:** restructured `files_panel` to mirror `dock_panel` exactly — a flush `caption_header`, then
  a `.py_1()` `rows_div`. Safe because the wheel-scroll handler lives on the placement div (app.rs:3915), not the
  panel body.
- [LOW] **Stale field comment** (app.rs:131) still called the left dock the "Files" dock. **FIXED:** now points
  to `files_panel`; noted the left dock is the "Workspace" navigator. Also tidied the `dock_panel` doc (was "M2
  fills it with a file tree").
- [INFO] the local var `let mut files` (the tab-nav div) is misleadingly named — LEFT (low value; the adjacent
  comment already clarifies it's not the file tree).

**Skip-detach verification (the recurring trap):** `cargo mutants --list -f app.rs | grep -E
'caption_header|fn dock_panel|fn files_panel'` → **none listed** — all three render helpers stay skip'd, so the
new `caption_header` insertion did NOT detach `dock_panel`'s attribute. `cargo check -p marley` clean.

Verdict: **Phase 3.5 PASS** — correctness clean; 3 simplification fixes applied; no functional/logic change.

## Phase 4 — Validate (ROOT-CAUSE CORRECTION — the real bug found here)
**REQ-001 + REQ-002 proven live** on the #190 build (chad's real Marley-repo workspace, fresh launch): the left
dock header reads **"Workspace"**; the file browser shows a **"Files"** header (captures
`marley-190-labels.png`). `dock_title` unit test passes.

**REQ-003 exposed the REAL bug.** Driving a repo-file click showed NOTHING opened. Disambiguated methodically:
- Directory toggles WORKED (clicks land, index correct) — so not a click/index bug. (A stuck ⌘ modifier from
  earlier synthetic chords was ALSO corrupting some clicks — fixed by `drive.swift clearmods`; a harness note,
  not code. Prevention rule worth recording.)
- **A/B test (git-stash the #190 code, rebuild):** the PRE-#190 build ALSO fails to open repo files → the
  file-open failure is NOT caused by #190's labeling change; it is pre-existing. But it IS chad's actual
  complaint, so it's in scope to FIX.
- **Root cause:** `FileTree::path_at` / `list_files_in` yield paths RELATIVE to the project root (lib.rs
  docstrings). `open_file_in_viewer` did `std::fs::read(&path)` on that relative path → resolved against the
  process CWD. A bundled app launched via `open`/LaunchServices has CWD `/`, so `read("crates/…/buffer.rs")` →
  `/crates/…` → `Err(_)` → silently swallowed. (The earlier AEServer.8 "success" only worked because THAT
  instance's project root was `/`, so relative==absolute.) Call site 5154 already did it right
  (`project_root.join(&fr.path)`); the tree click (2204) and finder ⌘↵ (1384) did not.

**Fix (functional — Phase-3 addition made here):**
- `marley_project::resolve_under_root(root, path)` — PURE: relative → `root.join`, absolute → unchanged.
  Unit-tested (`resolve_under_root_joins_relative_keeps_absolute`) + **mutation 1/1 caught (100% MSI)**.
- `open_file_in_viewer` routes `path` through it before `std::fs::read`, so EVERY caller (tree click, ⌘P-finder
  ⌘↵, terminal file-ref) opens from any CWD. `full` also used for the size/binary flash + `CodeViewState::new`.

**REQ-003/REQ-004 PROVEN live (post-fix):** clicking `audit.toml` now opens its code view — line-numbered,
syntax-highlighted contents render in the center AND an "audit.toml" tab appears in the Workspace tree (capture
`marley-190-fixed.png`). The exact bug chad reported is resolved.

**Because this added FUNCTIONAL code mid-validate, re-running Inspect on the path-resolution change before
finalizing.** (status → re-inspect.)

### Re-Inspect (the file-open fix) — one correctness critic, verdict CORRECT
- **All 4 `open_file_in_viewer` callers correct.** Tree click / ⌘P-finder ⌘↵ / search `HitKind::File` pass
  RELATIVE paths → now joined → absolute. The diff-header (app.rs:5161) passes `project_root.join(..)` = already
  ABSOLUTE → `resolve_under_root` passes it through unchanged (no double-join: `project_root` is always absolute,
  traced through `discover_in`). Verified concretely.
- **Dedup IMPROVED (bonus fix).** Before, sites stored a mix of relative (tree/finder) and absolute (diff)
  paths → opening the same file two ways made TWO tabs. Now all callers yield the same absolute
  `CodeViewState.path` → `open_or_switch_code` dedups correctly. Latent pre-existing bug fixed for free.
- **Edge cases benign** (empty/`..` unreachable from `path_at`'s down-only walk; absolute-under-different-root is
  intended passthrough). Build + full `marley_project` suite (18) green; `resolve_under_root` mutation 1/1.
- **[LOW → FIXED] Parallel un-hardened reader.** The session-restore path (`TabLayout::Code`, app.rs:740) had the
  SAME bare `std::fs::read(PathBuf::from(path))` — a pre-fix build persists a RELATIVE `cv.path`, so those tabs
  would silently drop on restore under a bundled app. **Folded the fix in:** routed it through
  `resolve_under_root(&root, ..)` too. Now both the open AND restore readers are CWD-independent.

Verdict: **Re-Inspect PASS** — fix correct, one same-class reader hardened, no regressions.

### Phase 4 close
**Tests added/run.** `resolve_under_root_joins_relative_keeps_absolute` (marley_project, REQ-004) + the updated
`dock_title_by_side` (REQ-001). `cargo nextest run -p marley_project -p marley` → **307 passed**.
**Driven (final shipping build).** Rebuilt with all fixes; `clearmods` + click `audit.toml` → its code view
renders (line-numbered, syntax-highlighted) + an "audit.toml" tab in the "Workspace" tree (`marley-190-final.png`).
Left dock reads "Workspace"; file browser reads "Files". REQ-001/002/003/004 all proven.
**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 (cov 100%, mutation MSI 100%, visual/AX).
Skip-detach re-checked clean (caption_header/dock_panel/files_panel/open_file_in_viewer all still absent from the
live mutant list). No pre-existing failures in scope. **Phase 4 PASS.**

## Phase 5 — Complete
**Docs.** CHANGELOG — two `### Fixed` entries (the file-open fix + the relabel). `app_shell.md` — updated the
`dock_panel` note (Left→"Workspace", the shared `caption_header`, the files_panel "Files" header) and added the
`resolve_under_root` path-resolution note under seq-5.

**Knowledge (forge).** `aar-submit` (aar `fe8cb735…`, completed, effectiveness 4 — solid fix, but I initially
mis-scoped). `failure-record BF-file-open-relative-path-cwd-190` (the silent relative-path read). Two
prevention rules: `PR-claude-clearmods-before-driven-clicks-001` (the stuck-modifier harness trap) and
`PR-claude-drive-tiling-split-via-context-menu-001` (from #189).

**Lessons.**
- **Don't declare "works for me" from one lucky environment.** I first concluded the file-click "works"
  (AEServer.8 opened) and scoped #190 as labeling-only — but that instance's project root happened to be `/`, so
  a relative-path read coincidentally resolved. The real bug only shows with a real project root ≠ CWD (chad's
  persisted repo). The **A/B test** (git-stash the change, rebuild) is what proved the file-open failure was
  pre-existing, not my labeling change — do that before blaming/absolving a diff.
- **A silent `Err(_) => {}` hid a user-facing failure for a long time.** Prefer surfacing read errors (a status
  flash) over swallowing them; the swallow is why "clicking a file does nothing" was so hard to diagnose.
- **The driven harness leaks modifier state** across synthetic chords — `clearmods` before plain clicks. Cost
  many wasted captures before I read `drive.swift` and found the `clearmods` action.
- Extracting the resolution to a PURE `resolve_under_root` (vs inlining in the skip'd shim) bought a real
  regression test + 100% mutation on the exact logic that was broken — the right call for a load-bearing fix.

forge wired — captured in forge (above) AND locally here (§19).
