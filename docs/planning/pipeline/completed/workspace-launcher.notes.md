# workspace-launcher — pipeline notes (forge #234, M13 sprint #26)

Pipeline: b2ad569f-552e-4ad4-8106-5c423b0eb172 · AAR: a6d85f41-c4b5-43c1-a165-84f7ae1687ee
Ticket: forge#234 (9a011201-2545-4a97-ac46-10671cb53415). 7th of the /work 228-237 train; #234 supersedes
#202. Deps #233 (done). chad GRANTED desktop control → driven-validate.

## Phase 1 — Plan (discovery)

**chad direction:** closing everything → a PhpStorm-style landing page (open-a-workspace, recents,
expandable). This RELAXES the M10 never-empties guards so a 0-workspace state is reachable.

**Discovery (background Explore) — the render-branch is VIABLE + keeps the ~90 workspace() sites safe:**
- `impl Render for RootView` app.rs:3479; `fn render` :3481 → `impl IntoElement`, final `root` :6397.
  **Lines 3482-3528 are workspace-FREE** (colors/palette/find-snapshot[gated]/viewport/geometry). The
  **first UNCONDITIONAL workspace call is app.rs:3529** `self.workspace_mut().focused()` — the hard
  floor. `workspace()`/`active_project()` panic at 0 (index into empty `Vec`).
  → **Branch after :3483:** `if self.shell.project_count()==0 { return launcher.into_any_element() }`
  else the unchanged body; final `root.into_any_element()`. Both arms AnyElement.
  ⚠️ the `on_key_down` closure on `root` (:3537+) itself calls `workspace_mut()` on some keys (:3651) →
  the LAUNCHER must own its input (mouse-first v1; a `track_focus` div, no shell key handler).
- **Persistence (settings.rs, marley_settings):** mirror `Workflows: Vec<Workflow>` (:46-50) /
  `RemoteHosts: Vec<RemoteHost>` (:41-45) — TOML array-of-tables. Seam: `settings_file_in(dir)`
  (:159), `load_manager_in(dir)` (:168); `AppliedSettings` (:110) + `applied_defaults` (:176) +
  `applied_from` (:202); `persist_*` one-liners (`persist_workflows` :238). **NO config env override** —
  the `*_in(dir)` seam IS the override (tests → tempdir; prod → `marley_config_dir()`=~/.marley/config,
  paths.rs:38). → add `Recents: Vec<PathBuf>` + `persist_recents` + thread the 3 applied fns.
- **Open path:** `open_project_path(&mut self, path: PathBuf)` app.rs:2680 (spawn_session_in +
  `Project::new` + `add_project` + sync + persist) — the exact call for a recent / Open. `open_project_picker`
  :2702 (native dir picker → open_project_path); dispatched as "open-project" needing cx at :3852. "New
  empty workspace" does NOT exist (a workspace = a folder + ≥1 terminal) → v1 New/Open = the picker.
- **Guard-relax:** `Workspace::close_project` tabs.rs:294 — `LastProject` at 298-300 (Err w/o removing) →
  DROP it. **`adjust_active` tabs.rs:422 — line 430 `next.min(new_len-1)` UNDERFLOWS (usize) at
  new_len==0** (its doc assumes ≥1) → GUARD (leave active as-is when now-empty). `close_project_at`
  app.rs:2789 — the flash `LastProject => "can't close the last project"` (:2809) must allow the close;
  `sync_active_project` (:2645) re-reads the active project → must no-op at 0.
- **Boot:** `RootView::new` :498; `restore_shell` :835; the decision :954 — `restored` else the legacy
  single-cwd seed (:970-993, ALWAYS seeds 1) → boot never 0 today. `ShellLayout::default()` = empty
  projects (grid_layout.rs:223, Default) → the empty-shell path is the else block. v1 KEEPS boot seeding
  (close-all-in-session → launcher is the core); boot-to-launcher = a follow-up.
- **Boundary:** pure `project_count()==0` (tabs.rs:360). `should_show_launcher` BRAND NEW. **Recents
  BRAND NEW** — no workspace MRU exists (history.rs is per-pane command history, unrelated).

**Scope call (D1-D4):** the render-branch (not a fallible accessor) + minimal guarded relaxing +
recents mirroring the settings precedent + a mouse-first landing view. Full launcher; boot-to-launcher
+ keyboard-nav + New-empty deferred.

**Pure seams (cov/MSI 100):** a new launcher model module (or in tabs.rs) — `should_show_launcher`,
`push_recent` (dedup-to-front, cap), the rows model; + the `adjust_active(0)` fix (tabs.rs). **Shim:**
the render-branch + `render_launcher` view + wiring + the guard-relax edits + recents-on-open.

**Driven validation plan (control granted):** bundle → boot (seeded workspace) → close each workspace
via the rail Project × (`close_project_at`) → at 0 the launcher renders (capture) → click a recent /
Open Folder → a workspace opens, launcher clears (capture). READ the PNGs.

**Deps:** #233, #156, #202 (supersede). **ENV:** chad granted control + stepped away → driven captures
ON; report at the end of the train for a new goal.

## Phase 2 — Design

**PURE — new `crates/marley_app/src/launcher.rs`** (cov/MSI 100):
```rust
pub fn should_show_launcher(open_workspace_count: usize) -> bool { open_workspace_count == 0 }
/// Front-dedup MRU, capped: prepend root, drop an existing copy, truncate to cap.
pub fn push_recent(recents: &[String], root: &str, cap: usize) -> Vec<String> {
    let mut out = vec![root.to_string()];
    for r in recents { if r != root { out.push(r.clone()); } }
    out.truncate(cap);
    out
}
```
**PURE — tabs.rs guard-relax:** (1) drop `close_project`'s `len==1 → LastProject` (298-300) so the last
close removes → empty; (2) `adjust_active` += `if new_len == 0 { return 0; }` at the top (the `new_len-1`
underflow) + drop the "≥1" doc claim.

**settings.rs:** `define_setting!(Recents: Vec<String> = Vec::new(), "workspace.recents")` (paths as
strings, TOML array); `persist_recents(manager, &[String])` (mirrors `persist_workflows`); a `recents:
Vec<String>` field threaded through `AppliedSettings` + `applied_defaults` + `applied_from`.

**app.rs shim:**
- `use crate::launcher::{should_show_launcher, push_recent};` + `const RECENTS_CAP: usize = 10;` + a
  `recents: Vec<String>` field on RootView (init from `applied.recents` at boot).
- **render-branch** at app.rs:3529 (right BEFORE the first `self.workspace_mut().focused()`, after the
  workspace-free colors/geometry): `if should_show_launcher(self.shell.project_count()) { return
  self.render_launcher(colors, bounds, cx).into_any_element(); }` + the final `root` → `root.into_any_element()`.
- **`render_launcher(colors, bounds, cx) -> impl IntoElement`** — a `track_focus` full-screen centered
  landing (bg=background): a title ("Open a workspace"), the recents rows (each = basename + path,
  `on_mouse_down` → `open_project_path(PathBuf::from(root))`), and an "Open Folder…" button
  (`on_mouse_down` → `open_project_picker(cx)`). MOUSE-FIRST — no shell key handler (the shell's
  `on_key_down` calls `workspace_mut()`; the launcher owns its own, none for v1).
- **`open_project_path`** (2680) += push the root to `self.recents` (`push_recent(.., RECENTS_CAP)`) +
  `persist_recents`. **boot-seed** (970-993) += push the seeded cwd root to recents too (so a fresh boot's
  workspace is a clickable recent → enables the recents-click driven test).
- **`close_project_at`** (2789): `close_project` no longer returns `LastProject` → drop that flash arm;
  on `Ok`, if `project_count()==0` SKIP `sync_active_project` (the launcher renders), else sync.
- **`sync_active_project`** (2645) += `if self.shell.project_count()==0 { return; }` guard (belt+suspenders).

**File manifest:** `launcher.rs` (NEW pure) · `tabs.rs` (guard-relax + a test update) · `settings.rs`
(Recents + persist + applied) · `app.rs` (render-branch + render_launcher + recents field/wiring + close/sync
guards) · `lib.rs`/`main.rs` (register the `launcher` module) · docs at Complete.

**Regression Test Plan:**
| # | test | REQ | asserts |
|---|---|---|---|
| T1 | `should_show_launcher_boundary` (launcher.rs) | REQ-001 | (0)=true, (1)=false, (5)=false |
| T2 | `push_recent_prepends` / `_dedups_to_front` / `_caps` (launcher.rs) | REQ-004 | prepend new; move existing to front (no dup); truncate to cap (oldest drops) |
| T3 | `close_last_project_empties` (tabs.rs, UPDATE the old refusal test) | REQ-002 | 1-project workspace: `close_project(0)` = Ok(project) + `project_count()==0` |
| T4 | `adjust_active_empty_returns_zero` (tabs.rs) | REQ-002 | `adjust_active(0,0,0)==0`, `adjust_active(2,1,0)==0` (no underflow) |
| T5 | Recents settings round-trip (settings.rs, tempdir) | REQ-004 | persist_recents + load_manager_in → applied.recents matches |
| T6 | DRIVEN (control) | REQ-001/002/003 | bundle → boot (1 seeded workspace, now a recent) → close it via the rail × → **launcher renders** (capture) → click the recent → **workspace reopens, launcher clears** (capture); READ both PNGs |

**Mutation:** run `cargo mutants --list -f launcher.rs -f tabs.rs` at implement/validate. Predicted:
should_show_launcher `==0` (killed by T1); push_recent `r != root` (killed by dedup) + truncate/loop;
adjust_active new-guard `==0` (killed by T4 + the existing adjust tests). Confirm the real set.

**Phase 2 status: Design PASS.**

## Phase 3 — Implement

Built to the manifest:
- **launcher.rs (NEW pure):** `should_show_launcher(count)==0` + `push_recent(recents, root, cap)`
  (prepend + dedup + truncate) + 4 tests. Registered `mod launcher;` (lib.rs, alphabetical).
- **tabs.rs:** dropped `close_project`'s `len==1 → LastProject` guard; **removed the `LastProject`
  variant** (dead once the producer went — clippy would flag a never-constructed variant); added
  `adjust_active` `if new_len==0 { return 0 }` (the underflow) + doc; updated `close_last_is_refused`
  (tab-only now) + added `close_last_project_empties` + `adjust_active_empty_returns_zero`.
- **settings.rs:** `Recents: Vec<String> = [] ("workspace.recents")` + `persist_recents` + `recents`
  threaded through `AppliedSettings`/`applied_defaults`/`applied_from` + the 3 test fixtures.
- **app.rs shim:** `use launcher::{should_show_launcher, push_recent}` + `persist_recents` +
  `AnyElement`; `const RECENTS_CAP=10`; a `recents: Vec<String>` field (boot-init = `push_recent(applied
  .recents, boot_active_root, CAP)` so the current workspace is a recent); the **render-branch** (before
  the first `workspace_mut().focused()`: `if should_show_launcher(project_count()) { return
  self.render_launcher(&colors, cx); }`) + the final `root.into_any_element()`; **`render_launcher`** (a
  new `impl RootView` method — a centered card: "Open a workspace" title, recent rows [basename + path,
  hover, click → `open_project_path`], "Open Folder…" [accent, click → `open_project_picker`];
  `track_focus`, mouse-first); `record_recent` helper; `open_project_path` records the root;
  `sync_active_project` 0-guards; `close_project_at` drops the `LastProject` arm (sync no-ops at 0).

**Deviations:** (1) `render_launcher(colors, cx)` — dropped the `bounds` param (flex-center + size_full
needs no bounds). (2) recents recorded at boot too (not just explicit opens) — better UX + enables the
recents-click driven test.

`cargo fmt` + `cargo check --all-targets -p marley` clean; **322 tests pass** (+6). Ready for Inspect.
**Phase 3 status: Implement PASS.** (T5 recents settings round-trip + the driven capture at Validate.)

## Phase 3.5 — Inspect

2 critics: a DEDICATED 0-workspace PANIC-SAFETY audit + a correctness/state/mutation critic. This was
the invasive ticket → the panic-safety critic was essential; the render-branch alone was INSUFFICIENT.

**Panic-safety critic — 3 reachable panics at 0 workspaces (all FIXED):**
- **[CRITICAL] `persist_grid` (app.rs:2061)** — `close_project_at` calls it AFTER the last close;
  `workspace_mut()` is unconditional → panics on the empty project vec. Fires FIRST, synchronously in
  the click handler (crash before the launcher even shows). FIX: guard at 0 → persist only the 0-safe
  shell layout (`build_shell_layout`).
- **[CRITICAL] the PTY pump (app.rs:552, cx.spawn, every 16ms)** — `active_project()` at :568/:569
  (+ :744) → panics. A BACKGROUND loop independent of render. FIX: early-return the pump's update
  closure at `project_count()==0` (`return;` → the closure yields `()`, stays alive).
- **[HIGH] `find_match_rows` (app.rs:1154)** reached via render:3612 — the render PROLOGUE, BEFORE my
  launcher guard (my guard comment was factually wrong). FIX: hoisted the guard to the VERY TOP of
  render (after colors, before the find snapshot).
- Verified SAFE: boot (always ≥1), the launcher's own handlers (add-before-touch), key dispatch (only
  root's on_key_down, ≥1), sync_active_project (guarded), adjust_active(0), build_shell_layout (0-safe),
  the picker async callback, the thread reapers.

**Correctness/state/mutation critic — corroborated the 3 panics + 1 HIGH gate gap + 2 LOW:**
- **[HIGH] `persist_recents` had NO round-trip test** → gate 4 (cov 100) + gate 5 (MSI 100) RED on
  settings.rs (the `→Ok(())` mutant survives). FIX: added `recents_setting_round_trips` (persist → load
  → get; kills the mutant + covers the line).
- [LOW] stale `close_project_at` doc → FIXED; [LOW] launcher.rs "R?" placeholders → FIXED (REQ-001).
- CLEAN: guard-relax correctness (`LastProject` variant fully gone, `rg` exit 1), push_recent (all 4
  cases), launcher.rs + tabs.rs mutation (all killed), render_launcher wiring, clean-room §20.

**Root-cause + captures:** D1 analyzed accessor-reachability ONLY inside render; the invasive part of
relaxing a never-empties invariant is that BACKGROUND accessors (the 16ms pump), the PERSIST path
(close→persist_grid), and the render PROLOGUE (find_match_rows) all deref the active project OFF the
guarded render path. Recorded `BF-claude-relax-invariant-offpath-accessors-panic-at-empty` (06b152f8) +
`PR-claude-relax-nonempty-invariant-audit-all-accessors-001` (7bd12a82) +
`PR-claude-new-persist-setting-needs-round-trip-test-001` (846df4a5).

All fixes at source; `cargo check --all-targets` clean; **323 tests pass** (+1). **Phase 3.5 status:
Inspect PASS.**

## Phase 4 — Validate

**Tests:** `cargo nextest run -p marley` → **323 passed, 2 skipped**. Plan rows green: T1
`should_show_launcher_boundary`, T2 `push_recent_*` ×3 (launcher.rs); T3 `close_last_project_empties`
+ T4 `adjust_active_empty_returns_zero` (tabs.rs); T5 `recents_setting_round_trips` (settings.rs); T6
below.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15**, incl **gate:4 cov ≥100%** +
**gate:5 MSI ≥100%** (launcher.rs + tabs.rs + settings.rs pure seams; the round-trip test closed the
persist_recents gap). (One clippy fail first — the `Err`-arm removal left a single-arm `match` →
clippy autofix to `if let Ok`; re-ran GREEN.)

**T6 — DRIVEN CAPTURE (control granted → done live, the launcher's whole point):** bundled → `open
Marley.app`. Sequence (WIN stayed 18743 throughout — NO crash):
1. `234-a-initial.png` — the shell, 1 workspace "Marley · main".
2. clicked the rail project × (`clickat:0.114,0.161`) → **NO PANIC** (window alive) →
   `234-b-afterclose.png` = **the launcher renders**: "Open a workspace" + "RECENT WORKSPACES" + the
   **Marley** recent (basename + full path `/Users/.../ignibyte/Marley` — the boot-recorded cwd) + the
   teal "Open Folder…" button, centered. **REQ-001 + REQ-002 + REQ-004 ✓.**
3. clicked the Marley recent (`clickat:0.499,0.515`) → `234-c-reopened.png` = **the shell returns**,
   workspace reopened in the Marley root. **REQ-003 ✓.**
The 3 inspect panic fixes HELD at runtime — closing the last workspace didn't crash (persist_grid
guard), and the 16ms pump ran through the 0-state without panicking (pump guard). A definitive
end-to-end validation.

**REQ-005** (supersede #202) — at Complete. **No pre-existing failures in scope. Phase 4 status:
Validate PASS.**
