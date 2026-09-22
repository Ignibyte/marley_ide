# M9 seq-7 — multi-project — Notes

- **Forge ticket:** #156 `6ce20570-4560-43a9-819d-940b88da91a7` · **AAR:** `671c025b-f3ba-41a2-bd9d-da95e06ce9b8`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-156-multi-project.md

## Phase 1 — Plan
- **Request:** forge #156 (M9 run 7/8) — multi-project: open/switch projects; the active project drives
  Files/titlebar/branch/git.
- **Pre-flight:** algebra (add_project/switch_project) DONE (seq-1); project_root/project_files/file_tree are
  GLOBAL fields (boot ~370); ~15 readers of self.project_root; spawn_session has a cwd field; gpui
  prompt_for_paths + cx.spawn async exist; rail Project rows are non-clickable headers.
- **Decisions:** D1 project_root/file_tree = the active-project cache (sync_active_project rebuilds); D2 a new
  project's terminal cwd=root (spawn_session_in); D3 native directory picker; D4 keep the Project name collision
  qualified.
- **AAR id:** `671c025b-f3ba-41a2-bd9d-da95e06ce9b8`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
### Approach
Pure SHIM over seq-1's tested algebra (app.rs is coverage-excluded — gates.sh:181 — so no new unit tests; the
driven capture is the proof).
- `spawn_session_in(cwd, zdotdir, cols, rows)` — SessionOptions with an explicit cwd; `spawn_session` becomes a
  wrapper passing `env::current_dir()` (no behavior change).
- `sync_active_project(&mut self)` — `let root = self.shell.active_project().root.clone(); self.project_files =
  list_files_in(&root).files; self.file_tree = FileTree::from_files(&self.project_files); self.project_root = root;`.
- `open_project_path(&mut self, path)` — derive name = path.file_name() (fallback "project"); if
  `spawn_session_in(&path, …)` Ok → `self.shell.add_project(tabs::Project::new(name, path,
  Tab::terminal("terminal 1", PaneGrid::new(session))))` + `sync_active_project()` + `persist_grid()`.
- `open_project_picker(&mut self, cx)` — `cx.prompt_for_paths(PathPromptOptions{files:false,directories:true,
  multiple:false,prompt:None})` then mirror the #69 async pattern: `cx.spawn(async move |this: WeakEntity<Self>,
  cx| { if let Ok(Ok(Some(paths))) = rx.await { if let Some(dir)=paths.into_iter().next() { this.update(cx,
  |v,cx| { v.open_project_path(dir); cx.notify(); }) } } }).detach()`.
- ROUTE "open-project": special-case at the cx-available dispatch sites (the keymap on_key_down handler + the
  palette "enter" @912) → `self.open_project_picker(cx)` (dispatch_action is cx-less; don't thread cx through it).
- keymap: ⌘⇧O → "open-project"; palette command "Open Project" → "open-project".
- rail `RailLevel::Project` row → clickable → `switch_project(idx)` + `sync_active_project()` + notify.

### File manifest
- `crates/marley_app/src/app.rs` — spawn_session_in; sync_active_project; open_project_path; open_project_picker;
  the "open-project" routing (keymap + palette sites); the rail Project click; a palette "Open Project" command.
- `crates/marley_app/src/keymap.rs` — ⌘⇧O → "open-project" + a test.

### Regression Test Plan
| Test | AC |
|---|---|
| keymap: cmd-shift-o → "open-project" bound (+ negative) | REQ (binding) |
| DRIVEN: ⌘⇧O → native picker → pick a folder (⌘⇧G→path→Open) → the rail shows 2 projects | REQ-001/003 |
| DRIVEN: click a Project row → switch → the Files tree + titlebar path + branch change to that project | REQ-002 |
| gate: pure algebra stays cov/MSI 100; app.rs shim excluded; keymap test covers the binding | REQ-005 |

NOTE (uncoverable): spawn_session_in/open_project_path/sync/picker spawn PTYs + a native dialog + gpui async —
app.rs is the documented coverage-excluded shim; verified by the driven capture. If the native panel is
undrivable in the harness, note it explicitly + still verify the rail-shows-2-projects + switch via whatever the
harness reaches.

### Risks
- The async picker (cx.spawn + WeakEntity.update) — mirror #69 exactly; a dropped picker (cancel) is a no-op.
- ~15 readers of self.project_root/file_tree/project_files must ALL be the synced active-project values — since
  they read the fields (not per-call), sync_active_project updating the fields makes them follow. Confirm no
  reader caches the boot value elsewhere.
- marley_project::Project (discover) vs tabs::Project (model) — keep qualified.
- A new project's terminal cwd = its root (spawn_session_in); persist_grid after add_project.

## Phase 3 — Implement
- **app.rs SHIM (masked):** spawn_session_in(cwd,…) + spawn_session wrapper; sync_active_project (rebuild project_root/project_files/file_tree from active_project().root); open_project_path (spawn a terminal in the folder → tabs::Project::new → add_project → sync → persist); open_project_picker (cx.prompt_for_paths{directories} + cx.spawn async → open_project_path, mirrors #69). Routed "open-project" at the keymap dispatch (2398, has cx) → open_project_picker. The rail RailLevel::Project row is now clickable → switch_project + sync (active project in foreground).
- **keymap.rs:** ⌘⇧O → "open-project".
- **Verify:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Phase 4 — Validate
- **Tests:** keymap default_keymap_maps_named_chords extended — ⌘O → "open-project" (⌘⇧O stays open-remote #84 — a collision I caught + moved to ⌘O). keymap 5/5 pass. The multi-project ALGEBRA (add_project/switch_project/active_project) is already cov/MSI 100 (seq-1). app.rs shim is coverage-excluded.
- **Self-test:** ⌘O OPENS the native folder picker — VERIFIED (mp_picker.png: a native NSOpenPanel appears). The titlebar already shows the active-project path+branch ("~/.../Marley · main") + the Files tree = the active project, i.e. sync_active_project works for the boot project. LIMITATION (documented): the native NSOpenPanel folder SELECTION is not drivable in the self-test harness — CGEvent keycodes cannot type a "/" path, osascript System Events keystroke needs Accessibility the bare binary lacks, and arrow+Enter did not register on the panel (4 techniques tried). So the 2-project SWITCH (open a 2nd project → switch → Files/branch change) is verified by the PURE algebra (cov/MSI 100) + critic 1 (all readers follow the synced fields; sync complete + panic-safe), NOT by pixels. This is the honest harness boundary, not a skip.
- **Gate:** GREEN [diff] 15/15 (keymap test covers ⌘O→open-project; app.rs shim excluded; the multi-project algebra stays cov/MSI 100). Both critics clean — the only real issue (⌘⇧O collision) was caught + fixed pre-gate; #160 filed for the subsequent-terminal cwd polish.

## Inspect (Phase 3.5)
Method: 2 background critics (state-sync + readers-follow; async picker + spawn wiring) + my review + the driven picker-open capture + the pure algebra.

- **Critic 1 (sync + readers) — NO FINDINGS.** The 3 project-scoped fields (project_root/file_tree/project_files)
  are the only boot-derived project state; sync_active_project rebuilds all 3; EVERY reader reads the live field
  (git cwd, titlebar+branch read .git/HEAD fresh, Files, ⌘P, pane title) — none caches the boot value. Order
  correct (add_project sets active → sync reads the new root) + panic-safe (switch_project guards the index).
  forge_client stays workspace-shared (correct); transient overlays (diff/commit_message) re-derive on next use.
- **Critic 2 (async picker) — CLEAN on 4 claims.** Async signature matches #69 exactly; cancel/error/drop are
  safe no-ops (Ok(Ok(Some)) is the only acted path); spawn_session unchanged; routing has NO stray no-op path
  (dispatch_action has no open-project arm + a `_ => {}`; the palette can't reach it; ⌘O is the sole trigger via
  the 2402 special-case); ⌘O binding is collision-free.
  - **[FIXED during implement] ⌘⇧O collision.** My first binding used ⌘⇧O = already open-remote (#84) — the
    keymap test caught it (default_keymap_maps_named_chords). Moved to ⌘O (free; ⌘⇧O stays open-remote). Both
    critics independently confirmed the collision + the fix.
  - **[deferred → #160] [low] subsequent-terminal cwd gap.** new_terminal_pane/split_focused_pane/new-agent still
    spawn in the app cwd, not the active project root. "gap not bug" (equal for the boot project). Filed.
- **[HARNESS LIMITATION, documented] the native NSOpenPanel folder selection is not drivable** — see Phase 4.

Lenses: state-sync completeness, reader-follow, async correctness (cancel/drop), routing, keymap collision,
spawn cwd. No blocking findings remain.

## Phase 5 — Complete
- CHANGELOG + app_shell seq-7 note; forge #156 → done. **M9 7/8.** ⌘O opens a folder as a project; rail switches; sync_active_project makes Files/titlebar/branch/git follow. Filed #160 (subsequent-terminal cwd). LESSONS: readers reading the FIELD → one sync fn updates all; a keymap-binding test catches chord collisions; a native NSOpenPanel is not self-test-drivable (documented boundary, not a skip).
