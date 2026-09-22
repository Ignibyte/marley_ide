---
pipeline_id: 9b4052c8-407a-4a31-b04a-dda8d553ded2
ticket: forge#247 (3b3b1422-9912-4ff7-93f4-02d7f49ae8a2) · local docs/planning/tickets/open/TICKET-247-boot-to-launcher.md
aar_id: db554f83-c956-4f7a-8b24-cfcf9485fed4
status: Phase 5 — Complete PASS
title: Boot to the launcher when empty + a New-empty-workspace launcher action
type: feature
milestone: M14
references: [forge#234, forge#205, forge#163]
---

## Title
(a) Boot straight to the #234 launcher when there is NO saved session to restore, instead of force-seeding a
default workspace; (b) add a "New empty workspace" action to the launcher (create a workspace without the folder
picker). A #234 follow-on.

## Scope
### In
- **(a) Suppress the boot force-seed when nothing to restore.** In `RootView::new`, when `restored.is_none()`
  (the persisted shell has no projects — a genuinely fresh / cleared / all-roots-vanished start), build an EMPTY
  workspace (`project_count()==0`) so the #234 launcher renders, instead of seeding a default cwd workspace.
- **A zero-project `Workspace` constructor** (`Workspace::empty` / `Workspace::new_empty`) — `projects: vec![],
  active: 0`. Pure + tested.
- **(a') The accessor audit** — every accessor reachable with the empty-boot shell must be guarded (the #234
  relax-invariant class). The known one: the boot recents-fold (`push_recent(&…, &shell.active_project().root…)`
  panics on empty). Design GREPs the whole `RootView::new` tail + the 16ms pump + the render prologue for the
  rest.
- **(b) A "New empty workspace" launcher button** — rooted at a REAL default dir (`$HOME`, fallback current_dir
  /temp), created via the shipped `open_project_path` (spawn + add_project + sync + record_recent + persist), no
  folder picker.

### Out (explicitly deferred)
- Keyboard-nav of the launcher recents (a #234 deferred item, separate).
- A true "untitled / no-directory" scratch workspace (the PTY needs a real cwd — the #205 lesson; v1 roots at a
  real dir).
- Any change to the SAVED-session boot (it must keep restoring normally — a regression guard, not a change).
- The multi-workspace re-architecture (this is single-Workspace launcher behavior).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — the "nothing to restore" condition is `restored.is_none()` (⟺ the persisted `shell` layout parsed to
  zero projects). Build an EMPTY workspace there; do NOT seed.
- **D2** — add a zero-project `Workspace::empty(name)` (`projects: vec![], active: 0`) in tabs.rs; pure + tested.
  `Workspace::new` (requires a first project) is unchanged.
- **D3 — AUDIT ALL accessors** for the empty-boot state (PR-claude-relax-nonempty-invariant-audit-all-accessors-001,
  the #234 rule): guard the boot recents-fold (app.rs:1042) + every other `active_project()/workspace()/
  projects()[..]` reachable at boot / in the pump / in the render prologue BEFORE the 3715 launcher guard.
  Mirror #234's "3 latent panics the render-branch alone missed."
- **D4** — "New empty workspace" roots at a REAL dir (the PTY needs a live cwd — #205); `$HOME` first, fallback
  `current_dir()` then `temp_dir()`. Reuse `open_project_path`. Display name: DESIGN decides (lean the root's
  basename, as `open_project_path` already derives, or "untitled").
- **D5** — data safety: the driven #247(a) test CLEARS the saved session — it MUST back up + restore chad's
  `~/.marley/config/settings.toml` (his persisted Marley workspace + editor files). Never lose his session.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the persisted shell has no projects to restore, the app shall boot to the launcher (`project_count()==0`), NOT seed a default workspace. | driven (cleared session → launcher) |
| REQ-002 | The empty boot state shall not panic any boot / pump / render accessor (the recents-fold + every audited site guarded). | review + driven (empty boot runs, no crash) |
| REQ-003 | WHEN "New empty workspace" is clicked on the launcher, the app shall create and show a workspace rooted at a real default dir, without a folder picker. | driven |
| REQ-004 | `Workspace::empty(name)` shall yield `project_count()==0` and an empty `projects()`, and `should_show_launcher` on that count shall be true. | pure unit |
| REQ-005 | WHEN a saved session exists, the app shall still restore it on boot (regression). | driven (restore chad's settings → normal boot) |

## Phase Plan
- **P2 Design** — the `restored.is_none()` branch + `Workspace::empty`; the FULL accessor audit (grep the
  `new` tail + pump + render prologue) + each guard; the launcher button + the `$HOME` root handler; the pure
  test matrix; `cargo mutants --list`.
- **P3 Implement** — `Workspace::empty` → the boot branch + guards → the launcher button + handler.
- **P3.5 Inspect** — critic: the accessor audit is COMPLETE (no unguarded empty-boot panic — the #234 class);
  the blank root is a real dir; the saved-session path unchanged; clean-room.
- **P4 Validate** — pure units; gate green; DRIVEN with the settings.toml BACKUP/RESTORE (cleared→launcher→
  new-empty-workspace; then restore→normal boot). Protect chad's session.
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #247; archive.
