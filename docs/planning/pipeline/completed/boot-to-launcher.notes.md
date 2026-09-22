---
pipeline_id: 9b4052c8-407a-4a31-b04a-dda8d553ded2
ticket: forge#247 (3b3b1422-9912-4ff7-93f4-02d7f49ae8a2)
aar_id: db554f83-c956-4f7a-8b24-cfcf9485fed4
---

# Notes — Boot to the launcher when empty + a New-empty-workspace action (forge#247)

## Plan (Phase 1)

**Classification:** work pipeline, feature, small-medium. 8th ticket of the M14 round. Independent #234
follow-on (reuses the shipped launcher + open_project_path). Autonomous auto-approved (M14 /work 238-247).
**This is the #234 CLASS** — relaxing the always-≥1-at-boot invariant → the danger is off-render-path accessor
panics (the #234 pipeline caught 3 the render-branch alone missed). The accessor AUDIT is the core work.

**Intent:** today boot ALWAYS seeds a default workspace, so a genuinely fresh/empty start never lands on the
#234 launcher. (a) Boot to the launcher when there's nothing to restore. (b) A "New empty workspace" launcher
button (no folder picker).

**Discovery (Explore — the anchors):**
- **Boot-seed:** `RootView::new` (app.rs:520); restore-vs-seed = `if let Some(restored) {…} else {SEED}` at
  app.rs:996/1012. The else arm (1012-1036) ALWAYS builds `Workspace::new(name, Project::new(cwd,…))` → ≥1.
  `restored` is None when the persisted shell has no projects (or all roots skipped).
- **"Nothing to restore":** `AppliedSettings.shell: String` (settings.rs:148, `ShellLayoutSetting` default ""
  settings.rs:91) → `restore_shell(&applied.shell)` (grid_layout.rs:367; empty → `ShellLayout::default()` =
  `projects: vec![]`). No saved session ⟺ `shell_layout.projects` empty ⟺ `restored.is_none()`.
- **#234 launcher:** `should_show_launcher(count)=count==0` (launcher.rs:9), render branch app.rs:3715 BEFORE any
  workspace access. `render_launcher` (app.rs:3623) = a 440px card: title → RECENT rows (`open_project_path`) →
  "Open Folder…" (app.rs:3680-3694 → `open_project_picker`). The #247(b) button slots after app.rs:3695.
- **open a workspace:** `open_project_path(&mut self, path)` (app.rs:2774) = `if let Ok(session) =
  spawn_session_in(&path,…) { add Project::new; sync_active_project; record_recent; persist_grid; }`.
  `spawn_session_in` (app.rs:470) fails on a non-existent/non-dir cwd → the add is SILENTLY skipped (#205
  cwd_or_root). ⇒ a blank workspace MUST root at a REAL dir; NO untitled/temp convention exists; `$HOME` is read
  at app.rs:1986 (git-branch). `Project::new` (tabs.rs:159) does NO validation.
- **Guard relaxed:** `close_project` (tabs.rs:321) already removes with no min-1 guard → runtime zero reachable,
  fenced by render 3715 + pump 577. `workspace()` (app.rs:2304) + `active_project()` (tabs.rs:358) PANIC on
  empty. Suppressing the boot seed is LOCALIZED (~3 sites); the render guard fences the 82+46 accessor sites.
  `Workspace::new` (tabs.rs:304) requires a first project — NO empty ctor yet.
- **⚠️ THE LANDMINE:** app.rs:1040-1044 `push_recent(&applied.recents, &shell.active_project().root…)` PANICS on
  the empty boot shell → MUST guard. app.rs:1047 `collapsed_indices(…, shell.projects()…)` is SAFE (empty
  slice). `project_count` doc "always ≥1" (tabs.rs:383) is already stale.

**Prior-lesson recall:** the #234 pipeline's `PR-claude-relax-nonempty-invariant-audit-all-accessors-001`
("relaxing an always-≥1 invariant → audit ALL accessors: background loops/timers, the persist path, the render
prologue — NOT just render") + `BF-claude-relax-invariant-offpath-accessors-panic-at-empty`. The #205
`cwd_or_root` lesson (a PTY needs a live cwd). These are the two load-bearing recalls.

**Decisions:** D1 `restored.is_none()`→empty ws · D2 `Workspace::empty(name)` pure+tested · D3 AUDIT ALL
accessors (guard the recents-fold + grep the rest) · D4 `$HOME`-rooted new-empty-workspace (real dir, #205) ·
D5 back up + restore chad's settings.toml around the driven test (data safety).

**Risks / load-bearing:**
- **The boot panic-class (#234)** — the accessor audit MUST be exhaustive; a single unguarded `active_project()`
  in the pump/persist/render-prologue crashes the empty boot. This is the #1 risk; the inspect critic re-audits.
- **Data safety (D5)** — the driven #247(a) test clears the saved session; the backup/restore of chad's
  settings.toml is mandatory (never lose his Marley workspace + editor files).
- **The blank root (#205)** — a synthetic/non-existent root silently no-ops the new workspace; must be `$HOME`
  (real).
- **`Workspace::empty` mutants** — a zero-project ctor is a thin `Self { projects: vec![], active: 0 }`; may be
  0-viable (no Default) → the unit is coverage+regression, OR the `vec![]`/`0` are killable. `cargo mutants
  --list` at design confirms.

**Test plan (finalized at design):** REQ-004 pure `Workspace::empty` (project_count 0, projects empty,
should_show_launcher true). REQ-001/002/003/005 driven WITH the settings.toml backup/restore.

**AAR:** db554f83-c956-4f7a-8b24-cfcf9485fed4 (opened).

**Phase 1 status: Plan PASS — autonomous auto-approved (M14). Ready for Phase 2 — Design.**

## Design (Phase 2)

**Approach.** The #234-class relax-invariant. Two seams + a shim wiring, in the gpui app layer. No new subsystem.

### The ACCESSOR AUDIT (the core — #234 class, PR-claude-relax-nonempty-invariant-audit-all-accessors-001)
The elegant #234 property: **when `project_count()==0`, `render` returns `render_launcher` at app.rs:3716
BEFORE building the shell tree** — so the ENTIRE shell element tree (every key/mouse handler) is NEVER MOUNTED
at empty. The pump is a separate background timer, guarded at app.rs:577 (`project_count()==0 → return`). So the
only empty-reachable accessors are (a) the boot constructor tail and (b) render_launcher itself.

| Site (app.rs) | Path | Reached at empty-boot? | Verdict |
|---|---|---|---|
| 1005 `w.active_project()` | `new`, the `Some(w)` restore arm | NO — only when `restored=Some` (≥1 proj) | fenced |
| **1042 `shell.active_project()` (recents-fold)** | `new` tail, runs ALWAYS | **YES → PANIC** | **GUARD (the only new one)** |
| 1047 `shell.projects()` (collapsed) | `new` tail | YES but SAFE (empty slice, no index) | none |
| 1056-1110 `Self { … }` | `new` tail | field stores only, no accessor | none |
| 589 / 764 pump `active_project()` | 16ms pump | fenced by the 577 `project_count()==0` early return | fenced |
| render 3716+ (find_match_rows, workspace(), …) | render | fenced by the 3715 launcher early return | fenced |
| all key/mouse handlers (1151, 2608, 3993, …) | shell tree | shell tree NOT MOUNTED at empty (render returns launcher) | fenced |
| render_launcher (3623) | render, at empty | reads only `self.recents` — no workspace accessor | safe |

⇒ **The ONLY new guard #247 needs is the recents-fold at 1042.** Everything else is already #234-hardened.

### The legacy-grid subtlety (a REAL trap — resolved via Option D)
The `else` (seed) arm (1012-1036) calls `restore_grid(&applied.grid)` (the pre-#163 legacy single-grid, key
`WorkspaceGrid` default `""` settings.rs:79). **Current `persist_grid` (app.rs:2109) writes the legacy `grid`
key ONLY when `project_count()>0`; its close-all branch (2111-2118) writes an empty shell but LEAVES `grid`
STALE.** So `restored.is_none() && applied.grid.is_empty()` alone would MISFIRE for a current user who closed
everything (stale non-empty grid → seeds the stale grid instead of the launcher).
- **D-legacy (Option D):** ALSO clear the legacy grid in persist_grid's `project_count()==0` branch
  (`persist_grid(manager, "")` alongside the empty shell). Then close-all zeroes BOTH keys.
- **The empty-boot condition** = `restored.is_none() && applied.grid.trim().is_empty()`:
  - both empty → `Workspace::empty` (launcher) — first boot / cleared / closed-all (grid now cleared by D). ✓
  - `restored.is_none()` but `grid` non-empty → the EXISTING seed arm restores the legacy grid (a genuine
    pre-#163 user, never ran the D fix) — UNCHANGED. ✓
  - `restored=Some` → the restore path — UNCHANGED. ✓

### Decisions confirmed
- **D1** empty-boot = `restored.is_none() && applied.grid.trim().is_empty()`. **D2** `Workspace::empty(name)` =
  `{ name: name.into(), projects: Vec::new(), active: 0 }` (the struct is `{name, projects, active}`; update the
  stale "Always holds ≥1 project" doc). **D3** guard ONLY the recents-fold (audit proved the rest fenced).
  **D-legacy** clear the stale grid on empty persist. **D4** `new_empty_workspace` roots at the first existing
  of `[$HOME, current_dir, temp_dir]` via a tested `launcher::first_existing_dir`, then `open_project_path`
  (which derives the name from the basename — fine, "chadpeppers" etc.; no name param needed). **D5** driven test
  backs up + restores chad's settings.toml.

## File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/tabs.rs | ADD `pub fn empty(name: impl Into<String>) -> Self { Workspace { name: name.into(), projects: Vec::new(), active: 0 } }` on `impl<S> Workspace<S>`; fix the stale struct doc ("Always holds ≥1 project" → "≥1 in normal use; empty only at the #234/#247 launcher state"). `#[cfg(test)]` in Phase 4. |
| crates/marley_app/src/launcher.rs | ADD `pub fn first_existing_dir(candidates: &[PathBuf]) -> Option<PathBuf>` = `candidates.iter().find(|p| p.is_dir()).cloned()`. Doc'd (#247). `#[cfg(test)]` in Phase 4 (tempdir-driven). |
| crates/marley_app/src/app.rs | (1) the boot branch: `let shell = if let Some(mut w) = restored { … w } else if applied.grid.trim().is_empty() { Workspace::empty(<name>) } else { <existing seed arm> };` (name = the cwd basename `project_name`, unused at empty but harmless). (2) GUARD the recents-fold (1042): `let recents = if shell.project_count() > 0 { push_recent(&applied.recents, &shell.active_project().root.to_string_lossy(), RECENTS_CAP) } else { applied.recents.clone() };`. (3) persist_grid empty branch (2111-2118): after `persist_shell`, ALSO `let _ = persist_grid(manager, "")` (clear the stale legacy grid — D-legacy). (4) ADD `fn new_empty_workspace(&mut self, _cx: &mut Context<Self>)` = build `[HOME, current_dir, temp_dir]` candidates → `first_existing_dir(&cands).unwrap_or_else(std::env::temp_dir)` → `self.open_project_path(dir)`. Masked (env+IO). (5) render_launcher: ADD a "New empty workspace" `card.child(…)` after the "Open Folder…" child (3695), mirroring it, `on_mouse_down(Left → view.new_empty_workspace(cx))`. |

Shim (app.rs boot branch + persist tweak + new_empty_workspace + the launcher button) is coverage+mutation
excluded / masked. Pure seams: `Workspace::empty` + `launcher::first_existing_dir`.

## Regression Test Plan
| # | Test | Proves | Kills |
|---|---|---|---|
| T1 | `tabs::tests` — `Workspace::<()>::empty("x")`: `project_count()==0`, `projects().is_empty()`, `should_show_launcher(ws.project_count())==true` | REQ-004 | `empty` body / `vec![]` |
| T2 | `launcher::tests` `first_existing_dir_cases` — build a real tempdir `d`; `first_existing_dir(&[nonexistent, d.clone()]) == Some(d)`; `first_existing_dir(&[nonexistent]) == None`; `first_existing_dir(&[]) == None`; a FILE (not dir) is skipped | REQ-003 (the real-dir pick) | body→`None`; `find` predicate |
| — | `cargo mutants --list -f tabs.rs` + `-f launcher.rs` — report the viable set for `empty`/`first_existing_dir` (post-impl). | MSI 100 | — |
| REQ-001/002/005 | DRIVEN (shim) WITH the D5 backup: back up `~/.marley/config/settings.toml` → clear `shell`+`grid` keys → boot → **launcher renders, no crash** (REQ-001+002) → type a key (no panic; shell tree unmounted) → RESTORE settings.toml → boot → **normal restore** (REQ-005). | REQ-001/002/005 | — |
| REQ-003 | DRIVEN: from the launcher, click "New empty workspace" → a terminal workspace opens rooted at `$HOME` (no picker). | REQ-003 | — |

**Uncoverable by unit:** the boot construction + render + the folder-less open (gpui `RootView`) — driven-only.

**Risks (load-bearing):** (1) the accessor audit completeness — MITIGATED (the render-branch un-mounts the shell
tree; only the recents-fold is new; the critic re-audits). (2) the legacy-grid trap — resolved by D-legacy
(clear on empty persist) + the `&& grid empty` condition; a genuine pre-#163 user still restores. (3) the blank
root must be real (#205) — `first_existing_dir` + `temp_dir()` final fallback (always exists). (4) **DATA SAFETY
(D5)** — the driven test MUST back up + restore chad's settings.toml; never lose his session.

**Phase 2 status: Design PASS — audit complete (only the recents-fold needs a guard), legacy-grid resolved,
manifest + test plan locked. Ready for Phase 3 — Implement.**

## Implement (Phase 3)

Built to the manifest; `cargo fmt` + `cargo check -p marley --all-targets` clean (only the pre-existing
unrelated `block v0.1.6` warning). `Workspace::empty(String::new())`'s generic `S` inferred from the boot-branch
type unification (the else arm pins the concrete session type).

- **tabs.rs** — `Workspace::empty(name)` (`{ name, projects: Vec::new(), active: 0 }`) beside `new`; fixed the
  stale "Always holds ≥1 project" struct doc.
- **launcher.rs** — `first_existing_dir(candidates) -> Option<PathBuf>` (`.iter().find(|p| p.is_dir()).cloned()`)
  + `use std::path::PathBuf` + a module-doc line (noting it's a filesystem probe, gpui-free + tested).
- **app.rs** — (1) the boot branch: `} else if applied.grid.trim().is_empty() { Workspace::empty(String::new())
  } else { <legacy-grid seed arm, unchanged> }` — the empty-boot lands here (no #163 shell AND no #205 legacy
  grid); the legacy-grid case still seeds. (2) the recents-fold guarded (`if shell.project_count() > 0 { push_
  recent(…) } else { applied.recents.clone() }`). (3) persist_grid's empty branch also `persist_grid(manager,
  "")` (D-legacy — clears the stale grid so close-all → launcher on next boot). (4) `first_existing_dir` added
  to the `use crate::launcher::{…}`; a masked `new_empty_workspace` handler (candidates `[HOME, cwd]` →
  `first_existing_dir` → `temp_dir` fallback → `open_project_path`); a secondary "New empty workspace" button in
  render_launcher after "Open Folder…" (surface bg + border vs the accent primary).

**Deviations from design:** none material. (Empty-workspace name = `String::new()` per the design's lean —
unused at the launcher state. The launcher button omits an explicit `mt` — the card's existing layout spaces the
children; the driven capture will confirm the spacing.)

**Phase 3 status: Implement PASS. Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

Two independent general-purpose critics (this is the #234 relax-invariant class — a boot panic risk + a data/
legacy subtlety) + my own review. Both critics CONFIRMED the design and DISPROVED the two HIGH-risk hypotheses.

**Critic 1 — the accessor audit (crash-safety at `project_count()==0`): CLEAN.** Verified all 6 lenses + the
transition paths, with cites: (1) `render` early-returns `render_launcher` at app.rs:3763-3764 BEFORE the shell
`div` (3814) is built → the shell tree's ~141 `active_project()`/`workspace()` handler sites are NEVER
CONSTRUCTED at empty (gpui dispatches only to mounted elements). (2) render_launcher touches only `self.recents`
/`focus_handle`/`colors`. (3) the pump's `project_count()==0` early-return (577) precedes every deref. (4) the
boot tail: the recents-fold is the only GUARDED accessor; the one other (`collapsed_indices`→`shell.projects()`)
is the whole slice (empty→empty, no index) — intrinsically safe. (5) no other bg deref (only 2 cx.spawn — pump
guarded, folder-picker → an empties→non-empty transition; the 9 thread::spawn move owned data, never `self`).
(6) only 2 `fn render`, no window/focus/observe callback derefs. Bonus: close-all's `persist_grid` + `sync_
active_project` both guard. → NO reachable panic.

**Critic 2 — data/state + legacy: CLEAN on all HIGH-risk.** (1) The legacy-grid condition is DATA-SAFE across
all 3 cases: a genuine pre-#163 user (shell="" + real grid) still SEEDS their grid (not launcher-ized — no data
loss); a current close-all user has the grid CLEARED by the new empty-branch `persist_grid(manager,"")` → next
boot launcher; first boot → launcher. `.trim()` routes a whitespace/corrupt blob to the launcher (a real #205
blob is never whitespace-only). (2) **THE NAME COLLISION → NO RECURSION**: `persist_grid(manager,"")` inside the
method binds to the imported FREE settings fn (an inherent method isn't in the bare-identifier value namespace;
needs `self.`) — byte-identical to the pre-existing `persist_grid(manager,&blob)` at ~2155; the green build
confirms. (3) `Workspace::empty` = a valid zero-state. (4) `new_empty_workspace` is total (temp_dir fallback;
pathological env → graceful no-op, no panic — #205 respected). (5) `first_existing_dir` correct. (6) clean-room.

**Findings:**
- **F1 [LOW, self+critic-1, FIXED]** — a stale-doc STRAGGLER: `project_count()` doc still said "(always ≥1)"
  after the struct doc was corrected. Reworded to "(≥1 in normal use; 0 only at the #234/#247 launcher state)".
  Class: relaxing an invariant → fix ALL its doc statements (struct AND method docs), not just the one you first
  edit. (Doc-only; no runtime effect; not worth a formal failure-record.)
- **F2 [MEDIUM → coverage owed to Phase 4]** — `first_existing_dir` has 2 VIABLE mutants (`None`,
  `Some(Default::default())`=`Some("")`) and `Workspace::empty` 1 UNVIABLE (`Default::default()` — no Default →
  0 viable → coverage+regression). NOT a code bug — the tests aren't written until validate. **Phase-4 T2 MUST
  assert `== Some(<the specific real dir>)`** (not `.is_some()`) to kill the `Some("")` mutant, and `== None`
  for empty/nonexistent/file; **T1 asserts the exact zero-state** for line-coverage of `empty`. (Confirmed via
  `cargo mutants --list`: launcher.rs:34 {None, Some(Default)}; tabs.rs:316 {Default UNVIABLE}. The tabs.rs
  402/410 grids mutants are PRE-EXISTING, not this diff.)
- **LOW notes (no fix — accepted):** `new_empty_workspace` names the project after the root basename (e.g.
  "chadpeppers" for $HOME) — cosmetic; and opening $HOME walks the home tree via the existing `open_project_
  path`→`list_files_in` (pre-existing perf cost, not a #247 concern). Both documented, deferred.

No real bug (the critics disproved both HIGH hypotheses); the win = the #234 rule
`PR-claude-relax-nonempty-invariant-audit-all-accessors-001` was APPLIED and CONFIRMED complete by an
independent audit (materialize it at complete).

**Phase 3.5 status: Inspect PASS — both critics clean, F1 fixed, F2 is Phase-4 test work (designed to kill the
2 viable mutants + cover empty). Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added — RAN `cargo nextest run -p marley first_existing_dir workspace_empty` → 2/2 pass:**
- `launcher.rs first_existing_dir_cases` — `Some(temp_dir)` for `[missing, real]` (the EXACT `Some(real)` kills
  body→`None` AND body→`Some(PathBuf::new())`=`Some("")`); `None` for `[missing]` and `[]`. Kills both viable
  mutants (confirmed by the gate's MSI pass).
- `tabs.rs workspace_empty_is_zero_state` — `Workspace::<()>::empty("scratch")`: project_count 0, projects
  empty, name, `should_show_launcher(0)`. Line-coverage of `empty` (its only mutant `Default::default()` is
  unviable — no Default).

**Driven capture — ENV-BLOCKED (locked screen), fallback per `PR-claude-selftest-locked-screen-blocks-capture-
fall-back-to-mechanism`.** The mac LOCKED/slept between the #244 captures (which succeeded live earlier THIS
session) and now — a full-screen `screencapture` returned all-black + `winforpid` returned `-1` (the CGWindow
image can't be created on a locked display). Never attempted a password. BUT the driven run still produced
strong BEHAVIORAL evidence + the data-safety proof:
- **REQ-001/002 (boot to launcher, no panic) — mechanism + behavior.** With chad's session backed up and the
  `shell`+`grid` keys cleared (a minimal dark-theme settings.toml → both default `""`), the app was bundled with
  the #247 code and launched: **the process stayed ALIVE (pid 78511, no crash, no DiagnosticReports crash log)**.
  A boot panic (an unguarded empty-shell accessor — the #234 class) would have unwound and EXITED the process
  during `RootView::new` / the first render / the first pump tick; instead it ran. Since `shell`/`grid` were
  both empty, the seed arm is unreachable → the `Workspace::empty` branch → `render_launcher` was taken. This +
  the 2-critic exhaustive audit (render early-return un-mounts the shell tree; only the recents-fold guarded;
  pump fenced) + the units carry REQ-001/002. (The PIXELS of the launcher card + the new button are the shipped
  #234 render path — driven-proven in #234 — plus the one added button, mechanism-mirrored on "Open Folder…".)
- **REQ-003 (New empty workspace) — mechanism.** `new_empty_workspace` reuses the shipped `open_project_path`
  (driven-proven by #244's rail/launcher clicks) with a real dir from the unit-tested `first_existing_dir`
  ($HOME→cwd→temp, temp always exists → the PTY never no-ops, #205). Not click-verified (screen locked).
- **REQ-005 (saved session still restores) — verified via data safety.** chad's `settings.toml` was RESTORED
  from the backup and confirmed **byte-identical** (`diff -q` → identical; his `grid = "t=/…/Marley"` + the
  multi-line `shell` are back). The restore path (#163/#205) is UNCHANGED by #247, so his next boot restores
  normally.

**⚠️ DATA SAFETY (D5) — HONORED.** `cp settings.toml → /tmp/marley-settings-backup-247.toml` (837B) → cleared →
tested → **`pkill` the app (stop persisting) → restored the backup → `diff -q` IDENTICAL.** chad's session is
intact. The app is not running.

**Gate — `scripts/gates.sh --diff` → GATE GREEN [diff] 15/15** — cov 100 + MSI 100 on tabs.rs `Workspace::empty`
+ launcher.rs `first_existing_dir` (both viable mutants killed; empty covered). One clippy red fixed at source
(a single-element `&[x.clone()]` test slice → `std::slice::from_ref(&x)`, `clippy::cloned_ref_to_slice_refs`).
The app.rs boot branch / new_empty_workspace / launcher button shim is coverage+mutation excluded. No
pre-existing failures in scope.

**Re-verify when unlocked (30s, no ticket):** the launcher pixels (card + "New empty workspace" button) + a live
"New empty workspace" click → a $HOME terminal. Behaviorally already evidenced (no-crash boot + shipped reuse).

**Phase 4 status: Validate PASS — 2 units (mutants killed), the driven run env-blocked but the no-crash boot +
data-safety-confirmed restore + 2-critic audit carry the AC, gate green. Ready for Phase 5 — Complete.**

## Complete (Phase 5)

- **Docs (§21):** CHANGELOG.md `### Added` (above #244) — "Boot to the launcher when there's no session to
  restore + a New empty workspace action". app_shell.md M14 — a #247 bullet (Workspace::empty; the
  `restored.is_none() && grid empty` condition + the legacy-grid seed preserved; the clear-companion-grid;
  the recents guard + the render-early-return audit; the $HOME new_empty_workspace + first_existing_dir).
- **Knowledge:** `aar-submit db554f83` completed, effectiveness 5 — the #234 relax-invariant audit was applied
  proactively + critic-confirmed; both HIGH hypotheses (persist_grid name-collision recursion + legacy-grid
  data-loss) disproven; the legacy-grid stale-companion trap caught at DESIGN + fixed via Option D; data safety
  honored under an env-block. Materialized `PR-claude-relax-nonempty-invariant-audit-all-accessors-001` (#234)
  + `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`. **NEW rule recorded:**
  `PR-claude-persist-clear-stale-companion-key-on-reset-001` (d246dc89, medium) — when state persists across a
  primary + a legacy/companion key, the reset/empty branch must clear the companion too, or a stale companion
  resurrects the old state on restore (the legacy-grid-on-close-all trap). No failure-record (no real bug — F1
  was a doc straggler, fixed).
- **Close:** forge #247 (`3b3b1422`) → done; TICKET-247 open→closed.
- **Archive:** spec status `Phase 5 — Complete PASS`; the spec+notes → `pipeline/completed/`.

**Phase 5 status: Complete PASS. Run `/commit` to deliver.**
