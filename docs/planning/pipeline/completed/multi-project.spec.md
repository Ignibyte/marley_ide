---
pipeline_id: 54ccb525-b296-4a14-b34b-33bf2551f211
ticket: forge#156 (6ce20570-4560-43a9-819d-940b88da91a7) · local docs/planning/tickets/open/TICKET-156-multi-project.md
aar_id: 671c025b-f3ba-41a2-bd9d-da95e06ce9b8
status: Phase 5 — Complete PASS
title: M9 seq-7 — multi-project (open/switch projects in a workspace)
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/app.rs (SHIM: spawn_session_in, sync_active_project, open_project_path, open-project action, rail Project click)
  - crates/marley_app/src/tabs.rs (PURE algebra — reused; no change expected beyond tests if any)
---

## Title
A workspace holds multiple projects — open a folder as a new project, switch projects from the rail, and the
Files tree, titlebar path, branch, and git cwd all follow the active project.

## Scope
### In
- SHIM `app.rs`: `spawn_session_in(cwd, …)`; `sync_active_project` (rebuild project_root/project_files/file_tree
  from `active_project().root`); `open_project_path(path)` (add_project + a terminal spawned in the folder +
  sync); an `open-project` action → the native folder picker (`prompt_for_paths` + `cx.spawn`); the rail
  `RailLevel::Project` row becomes clickable → `switch_project` + sync.

### Out
- Persisting the open projects across restarts. Closing a project from the rail (algebra exists; not wired).
- Per-project theme / settings. Re-discovering files on external change (still a boot-time / open-time walk).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `self.project_root`/`project_files`/`file_tree` stay as the app's view of the ACTIVE project (a cache),
  rebuilt by `sync_active_project` on open/switch. The ~15 readers (git, titlebar, branch, Files, ⌘P) follow
  automatically — no per-site change.
- D2 — a new project's terminal spawns with `cwd = the project root` (via `spawn_session_in`), so its shell
  starts in the folder.
- D3 — open uses the native directory picker (`prompt_for_paths{directories:true}`), awaited via `cx.spawn`.
- D4 — `marley_project::Project` (discovery) vs `tabs::Project` (the model) stay qualified (name collision).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `open_project_path(dir)` runs, a new Project rooted at `dir` shall be added to the active workspace and become active, and `project_root` shall equal `dir`. | driven / logic |
| REQ-002 | WHEN the active project changes (open or a rail Project-row click), the Files tree, titlebar path, and branch shall reflect the new active project's root. | driven capture |
| REQ-003 (visual) | WHEN a 2nd project is opened, the rail shall list 2 projects; clicking a project row shall switch to it (its tabs shown). | driven capture |
| REQ-004 | `spawn_session_in` shall spawn a session whose cwd is the given directory; `spawn_session` shall be unchanged (env cwd). | logic |
| REQ-005 | gate GREEN; the pure algebra stays cov/MSI 100; the shim wiring masked. | gate |

## Phase Plan
- **P2** — spawn_session_in; sync_active_project; open_project_path; the picker action; the rail Project click; test/capture plan.
- **P3** — implement.
- **P3.5** — 2 critics (sync correctness + the ~15 readers follow / no stale global; the async picker wiring + name-collision + spawn cwd).
- **P4** — gate + a driven capture (open a 2nd project via the picker; switch; Files+branch change). Picker-undrivable fallback noted.
- **P5** — docs, AAR, archive, close #156.
