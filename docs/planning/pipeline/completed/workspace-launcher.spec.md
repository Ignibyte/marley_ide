---
pipeline_id: b2ad569f-552e-4ad4-8106-5c423b0eb172
ticket: forge#234 (9a011201-2545-4a97-ac46-10671cb53415) · local docs/planning/tickets/open/TICKET-234-workspace-launcher.md
aar_id: a6d85f41-c4b5-43c1-a165-84f7ae1687ee
status: Phase 5 — Complete PASS
title: Launcher / landing page — the open-a-workspace home shown when no workspace is open
type: feature
milestone: M13
references: [forge#233, forge#202, forge#156]
---

## Title
The PhpStorm-style landing page shown when NO workspace (project) is open — chad's "closing everything
→ a landing type page to open a workspace." It RELAXES the M10 never-empties guards so a zero-workspace
state is reachable (superseding #202, whose zero-tab premise was unreachable), then the top-level render
branches to a launcher — a title + a list of recent workspaces + an "Open Folder" action — instead of a
blank/blocked screen. Opening a workspace (a recent, or the folder picker) records it in a persisted
recents list and returns to the shell. Deps #233 (the workspace foundation). chad granted desktop
control → driven-validated (close all → launcher → open → clears).

## Scope
### In
- **Pure (cov/MSI 100):** `should_show_launcher(project_count) -> bool` (the `== 0` boundary);
  `push_recent(recents, root, cap) -> Vec<..>` (dedup-to-front, capped MRU); the launcher display model
  (recents → rows). Plus the `adjust_active` `new_len == 0` fix (the usize-underflow guard, tabs.rs).
- **Guard-relax (tabs.rs):** drop `Workspace::close_project`'s `LastProject` guard (298-300) so the last
  workspace CAN close → 0; guard `adjust_active` for `new_len == 0` (430 underflows today).
- **settings.rs:** a persisted `Recents: Vec<PathBuf>` (mirroring `Workflows`/`RemoteHosts` — TOML,
  the `*_in(dir)` config seam) + `persist_recents` + threaded through `AppliedSettings`/`applied_*`.
- **app.rs shim:** the render-branch (`if project_count()==0` → the launcher, ELSE the existing shell
  body — keeps the ~90 `workspace()` sites in the ≥1 arm); the launcher VIEW (landing: title, recents
  rows, "Open Folder"); wiring (Open → `open_project_picker`, a recent row → `open_project_path(root)`);
  `close_project_at` allows the last close + `sync_active_project` tolerates 0; push the opened root to
  recents (persisted) on `open_project_path`.
- **Close/relabel #202** as superseded by #234.

### Out (deferred)
- **Boot-to-launcher when the persisted shell is empty** — v1 keeps boot seeding a cwd project (the
  launcher is reached by closing all workspaces IN a session, the core ask); a fresh/empty boot showing
  the launcher is a small follow-up.
- **A first-class "new EMPTY workspace"** — unmodeled today (a workspace always binds a folder + ≥1
  terminal); v1's "New/Open" both use the folder picker.
- Richer launcher UX (keyboard nav of recents, remove-from-recents, "expandable" sections) — v1 is
  mouse-driven: a recents list + Open Folder.
- The scope-driven top bar (#235), rail highlight (#236), editor tabs (#237).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — render-branch, not a fallible accessor.** `render` branches at the top (after colors/palette,
  before the first `workspace()` at app.rs:3529): 0 projects → the launcher (an early `return
  launcher.into_any_element()`); ≥1 → the unchanged shell body (`root.into_any_element()`). This keeps
  every `workspace()`/`active_project()` call (incl the `on_key_down` closure's) unreachable at 0 — NO
  ~90-site fallible-accessor change. The launcher owns its own (mouse-first, safe) input.
- **D2 — recents mirror the settings `Vec<struct>` precedent** (`Workflows`/`RemoteHosts`): TOML, the
  `settings_file_in(dir)`/`load_manager_in(dir)` seam (tests pass a tempdir), `persist_recents`. Pushed
  (dedup-to-front, cap ~10) on `open_project_path`.
- **D3 — guard-relax is minimal + guarded.** Drop only the `LastProject` refusal; fix the
  `adjust_active(new_len==0)` underflow; `close_project_at`/`sync_active_project` no-op their
  active-project work at 0 (the render shows the launcher). No other guard (LastTab/LastTerminal) is
  touched.
- **D4 — v1 is mouse-first + close-all-reaches-launcher.** The core deliverable is the reachable
  zero-state + the landing page; boot-to-launcher + keyboard nav + New-empty are follow-ups.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE the open-workspace count is 0, `render` shall show the launcher (not the shell); WHERE ≥1, the shell. | pure `should_show_launcher` test + driven capture (close all → launcher) |
| REQ-002 | Closing the last open workspace shall be ALLOWED (→ 0 → launcher), not refused. | tabs.rs `close_project` test (len==1 removes) + `adjust_active(0)` test + driven |
| REQ-003 | The launcher shall offer "Open Folder" (→ the directory picker → opens a workspace) and a list of recent workspaces, each opening on click and returning to the shell. | shim review + driven (click Open → shell; click recent → shell) |
| REQ-004 | Opening a workspace shall record its root in a persisted, dedup'd, capped recents list surviving restart. | pure `push_recent` test + settings round-trip + driven |
| REQ-005 | #202 shall be closed/relabeled as superseded by #234. | forge #202 status |

## Phase Plan
- **P2 Design** — the pure signatures (`should_show_launcher`/`push_recent`/rows + the `adjust_active`
  fix) + tests; the settings `Recents` schema; the render-branch + `render_launcher` shim; the
  guard-relax edits; the recents-on-open wiring; run `cargo mutants --list`.
- **P3 Implement** — pure (tabs.rs + a launcher model module) → settings.rs → app.rs shim.
- **P3.5 Inspect** — critics: the 0-state safety (no panic path), the guard-relax correctness (underflow),
  recents round-trip, render-branch type/scope, clean-room.
- **P4 Validate** — RUN the pure tests; gate green; DRIVEN capture (bundle → close all → launcher → click
  Open/recent → shell), READ the PNGs.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #234 + supersede #202.
