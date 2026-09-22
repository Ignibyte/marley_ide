# layout persistence (M5 FINALE) — Notes

- **Forge ticket:** #118 `2bfc66bc-19f0-49b3-9d6c-4bd781ab7c27` · **AAR:** `c4710e1d-af95-4cc8-b543-3bb5976a5b57`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-118-layout-persist.md

## Phase 1 — Plan
- **Request:** forge #118 (M5 12/12, FINALE) — persist the workspace layout (the git-panel state).
- **Pre-flight:** #95 persist pattern; app `settings: Option<SettingsManager>` (169); ⌘⇧C toggle (1764);
  applied read in new() (299). Panes/sessions aren't serializable → persist the git-panel state (extensible).
- **Decisions:** D1 git=0/1 blob, malformed→false; D2 #95 round-trip + a new-persist-fn round-trip test.
- **AAR id:** `c4710e1d-af95-4cc8-b543-3bb5976a5b57`.

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
- **settings.rs (PURE):** `pub fn serialize_layout(git_panel_open: bool) -> String` = format!("git={}", if git_panel_open {1} else {0}); `pub fn restore_layout(s: &str) -> bool` = s.strip_prefix("git=").map(|v| v=="1").unwrap_or(false); `define_setting!(WorkspaceLayout: String = String::from(""), "workspace.layout")`; AppliedSettings.layout: String; applied_defaults layout = WorkspaceLayout::default_value(); applied_from layout = manager.get::<WorkspaceLayout>(); `pub fn persist_layout(manager, s: &str) -> Result<(),SettingsError>` = manager.set::<WorkspaceLayout>(s.to_string()). Update ALL 5 AppliedSettings literals (2 prod + 3 test) for `layout`.
- **app.rs SHIM:** new() `git_panel_open: restore_layout(&applied.layout)` (was false); the ⌘⇧C toggle (1764) also persists: after the toggle, `let s = serialize_layout(view.git_panel_open); if let Some(m)=view.settings.as_mut() { let _ = persist_layout(m,&s); }`. Footer unchanged (#94).
- **Mutation targets:** serialize if→1/0; restore strip_prefix+=="1"+unwrap_or(false); applied_from get; persist_layout set.
- **Test plan:** layout_round_trip (restore(serialize(true))==true; restore(serialize(false))==false; restore("garbage")==false); extend settings_round_trip_survives_reload with persist_layout(serialize_layout(true)) + assert reload restore_layout(applied.layout)==true.
- **Risks:** every AppliedSettings literal must add `layout` (compiler-enforced, like #106); the toggle-persist borrow order (compute s before settings.as_mut()); git-panel auto-restore is reasonable IDE behavior.

## Phase 3 — Implement
- **Built:** serialize_layout/restore_layout (settings.rs, pure) + WorkspaceLayout setting + AppliedSettings.layout (applied_defaults="", applied_from=get) + persist_layout + all 5 literals updated; app.rs imports them + new() restores git_panel_open via restore_layout(&applied.layout) + the ⌘⇧C toggle persists serialize_layout(git_panel_open) through the settings manager. Footer unchanged (#94).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a tiny serialize/restore round-trip + a read-write setting mirroring #95/#106; gate cargo-mutants is the MSI authority).
- **Lenses — no findings:** serialize_layout = "git="+(1 if open else 0); restore_layout = strip_prefix("git=").map(=="1").unwrap_or(false) → round-trips (true↔"git=1", false↔"git=0"); a malformed/empty blob (no "git=" prefix) → false; WorkspaceLayout is a String setting (applied_from reads it, persist_layout writes it, both mirror the proven persist_right_section); new() restores the git-panel state, the ⌘⇧C toggle persists it (blob computed BEFORE settings.as_mut() → no borrow clash); a persist failure is swallowed (best-effort). No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** layout_round_trip (true↔git=1, false↔git=0, garbage/empty→false) + settings_round_trip_survives_reload extended (persist_layout(serialize_layout(true)) → reload restore_layout==true) + all AppliedSettings literals carry layout. `cargo nextest` → pass.
- **Self-test:** boot renders normally (the persistence round-trips via the engine test; a relaunch restoring the git panel needs a config write between launches — engine-tested). The footer chrome is #94 (unchanged).
- **Gate finding + fix:** first run RED on gate:14 (rustdoc) — the pub AppliedSettings.layout field doc used an intra-doc link [`restore_layout`] to the pub-fn-in-private-mod restore_layout → private_intra_doc_links (a public→private doc link). Fixed: plain backticks (no link). Re-gate → GREEN 15/15.

## Phase 5 — Complete
- CHANGELOG + app_shell.md M5 note; forge #118 → done. **M5 12/12 — FINALE, M5 COMPLETE.** serialize/restore_layout + workspace.layout (cov/MSI 100) + boot-restore + toggle-persist. Gate caught the public→private doc-link (rustdoc) → prevention rule.
