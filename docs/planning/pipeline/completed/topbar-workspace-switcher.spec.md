---
pipeline_id: 69541642-66c0-42ee-a098-94349d1282e1
ticket: forge#244 (3f762a4d-e498-4375-82ba-dfc38cdf511b) · local docs/planning/tickets/open/TICKET-244-topbar-workspace-switcher.md
aar_id: c3aa9110-dc55-4647-8679-d53252dc0b44
status: Phase 5 — Complete PASS
title: Make the top-bar focused-workspace indicator a click-to-switch popover
type: feature
milestone: M14
references: [forge#233, forge#235, forge#166, forge#229]
---

## Title
Make the #235 top-bar focused-workspace indicator ("name · branch") INTERACTIVE — click it to open a popover
listing the open workspaces; click a row to switch the focused workspace. Today it is a display-only `String`.

## Scope
### In
- **A pure display model** `titlebar::workspace_switcher_rows(names: &[String], active: usize) ->
  Vec<SwitcherRow>` (`SwitcherRow { index: usize, label: String, active: bool }`) — one row per project, `active`
  true only for the active index (out-of-range active → none flagged; empty names → empty vec). Mirrors the
  `rail_rows` active-marking but as a dedicated tested model.
- **The indicator becomes a click target** — the existing indicator `div` (app.rs:6437) gets the top-bar click
  idiom to TOGGLE a new `workspace_switcher_open` state on RootView.
- **A popover overlay** mirroring the #166 `context_menu` shape: a full-screen `inset_0().occlude()` dismiss
  backdrop + a popover box anchored below the indicator, rows from `workspace_switcher_rows`; a row's Left
  mouse_down switches the focused workspace (the SHIPPED rail-click idiom: `switch_project(i)` + guarded
  `sync_active_project()` + `persist_grid()`) and closes the popover. The active row is highlighted.

### Out (explicitly deferred)
- **Keyboard nav** of the popover (esc/↑/↓/enter) — the click path is the AC; a follow-up if wanted.
- **A branch per row** — `Project` stores no branch (it is derived per-render from `<root>/.git/HEAD`); v1 rows
  are name-only. The active indicator label itself still shows the branch (unchanged).
- **The multi-workspace re-architecture** (chad's PhpStorm-multi-window vision) — this switches the active
  PROJECT within today's single `Workspace` container; it does NOT change the top-level model.
- **Click-to-cycle** as the interaction (rejected — a no-op with one workspace, can't jump to a specific one).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — POPOVER, not click-to-cycle.** Matches chad's PhpStorm/IDE north star, discoverable with a single
  workspace (lists 1 row), reuses the battle-tested #166 overlay.
- **D2 — reuse the rail-click switch idiom, NOT `cycle_project`.** `cycle_project` is key-only with no RootView
  wrapper; the popover rows call `switch_project(i)` + guarded `sync_active_project()` + `persist_grid()` (the
  idiom at app.rs:4548-4551), byte-identical to a rail project-row click.
- **D3 — name-only rows** (branch is not stored; KISS). The pure model carries `label` = `project.name`.
- **D4 — opening the switcher CLOSES any co-open overlay, and its dismiss backdrop must not silently hide one**
  (the #229 lesson — `PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays`): toggling the
  switcher open sets `context_menu=None` / `agent_launcher=None` / any co-open overlay off first; the toggle
  click `.occlude()`s so it doesn't fall through to its own dismiss backdrop.
- **D5 — the pure model lives in `titlebar.rs`** beside `focused_workspace_indicator` (the indicator's module).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `workspace_switcher_rows(names, active)` shall emit one `SwitcherRow` per name in order, with `active` true only for the row whose index equals `active` (out-of-range `active` → no row flagged; empty `names` → empty vec). | pure unit tests |
| REQ-002 | WHEN the focused-workspace indicator is clicked, the system shall toggle a popover listing the open workspaces; WHEN a popover row is clicked, the system shall switch the focused workspace (`switch_project` + `sync_active_project` + persist) and close the popover; WHEN the dismiss backdrop is clicked, the system shall close the popover. | shim review + driven |
| REQ-003 | The open popover shall visually highlight the row for the active workspace. | driven capture |

## Phase Plan
- **P2 Design** — confirm D1-D5 + the `SwitcherRow`/`workspace_switcher_rows` signature; the `workspace_switcher_open`
  field + toggle + overlay render manifest (anchor geometry below the indicator); the co-open-overlay close set
  (D4); `cargo mutants --list -f titlebar.rs`; the test matrix.
- **P3 Implement** — the pure model + tests-to-compile → the field + toggle → the overlay render.
- **P3.5 Inspect** — critic: the model correctness (active-marking/out-of-range/empty), the switch idiom is
  byte-identical to the rail click, the D4 overlay-coexistence (no silent hide, toggle occludes), clean-room.
- **P4 Validate** — RUN the pure units; gate green; DRIVEN (click indicator → popover with active row
  highlighted → click row → dismissed + still focused → open → click backdrop → dismissed).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #244; archive.
