---
pipeline_id: ee3b98bb-2824-4d09-b22d-5fe26dc7e8a4
ticket: forge#233 (e29e08e4-e554-474b-93a9-891128b4a370) · local docs/planning/tickets/open/TICKET-233-workspace-foundation.md
aar_id: 159c2334-5c9e-41bf-8e46-6287ff285e2e
status: Phase 5 — Complete PASS
title: Workspace-model foundation — formalize the focused-workspace + switch algebra + pin the terminology
type: feature
milestone: M13
references: [forge#156, forge#150, forge#157]
---

## Title
The foundation slice of the workspace-centric re-arch (#234-237 build on it). Discovery
(`workspace-foundation.notes.md`) shows the multi-workspace CONTAINER already exists — the top level is
a single `tabs::Workspace` holding `Vec<Project>` with one `active: usize` cursor (#156 multi-project +
M9). So "multiple workspaces open, one focused" already maps to today's multi-project + `active`. The
genuine foundation delta is therefore small and CONTAINED: **(1)** pin the workspace terminology (a
vetoable decision — the disruptive code rename is deferred), **(2)** formalize the pure focused-workspace
+ switch algebra and add **workspace cycling** (a keybinding to switch the focused workspace — which does
NOT exist today), **(3)** a minimal focus-aware shim. The invasive re-arch pieces (the type rename, the
"no workspace focused" empty state, relaxing the never-empties guards) are deferred to the tickets that
actually need them.

## Scope
### In
- **Terminology PIN** — document the workspace vocabulary as a Locked-In Decision (D1, VETOABLE): the
  repo (today's `tabs::Project`) is chad's top-level focusable **Workspace**; the singleton container
  (today's `tabs::Workspace`) is the **Session** (the set of open workspaces + focus + the future
  launcher). The code RENAME is explicitly DEFERRED (D2).
- **Pure focus/switch algebra** (`tabs.rs`, cov/MSI 100) — formalize the focused-workspace model (the
  `active` project cursor promoted to a named, first-class concept) + add workspace **cycling**
  (`cycle`-style next/prev over `project_count`, reusing the tested `next_index`/`prev_index`); document
  the close-refocus (`adjust_active`) as part of the algebra.
- **Minimal shim** — a workspace-cycle keybinding (`next-workspace`/`prev-workspace`, a free chord
  distinct from `next-tab`/`prev-tab`) that switches the focused workspace + re-syncs the active
  project's Files / ⌘P / git / titlebar (reusing `switch_project` + `sync_active_project`).

### Out (explicitly deferred — each to the ticket that needs it)
- **The type RENAME** (`tabs::Project → Workspace`, `tabs::Workspace → Session`) — its own mechanical
  pass. Discovery: `tabs::Project → Workspace` COLLIDES with the existing `marley_project::Project` (a
  second `Project` type) + the bare `Project` import in app.rs, and touches ~40 refs in tabs.rs + ~75
  total. High-risk, collision-prone, and the terminology is vetoable → rename ALONE, after chad confirms
  the vocabulary. #233 pins the words + builds the model on the CURRENT type names, so a terminology veto
  costs a doc edit, not a 75-ref revert.
- **The "no workspace focused" empty state** + relaxing the never-empties guards (tabs.rs:151-157/298-299,
  app.rs:2778/2809) + making `workspace()`/`workspace_mut()` fallible (app.rs:2227/2236 `.expect`, ~90
  call sites) — **#234 (launcher)** needs + owns these. #233 keeps the always-focused invariant intact.
- The launcher UI (#234), the scope-driven top bar (#235), the collapsible rail + focused-workspace
  highlight (#236), the editor-surface tabs (#237).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — Terminology (VETOABLE at chad's review).** Session (container) ⊃ Workspace (repo, focusable) ⊃
  Tab ⊃ Pane. chad asked me to pin it and said "go with it"; pinned here. Because the rename is deferred
  (D2), this decision is cheaply reversible — a veto edits a doc, not the code.
- **D2 — Defer the rename.** Pin the vocabulary in docs; build the focus model on the current type names
  (`tabs::Project`/`tabs::Workspace`). Rationale: the `Project` name already denotes TWO types
  (`tabs::Project` + `marley_project::Project`), so the promotion-rename must first disambiguate — a
  contained mechanical pass best done alone, post-veto.
- **D3 — The container already exists.** #156 + M9 built `Vec<Project>` + the `active` cursor; #233 does
  NOT add a container level — it formalizes the focus + adds cycling. (Honest scope: smaller than the
  ticket framed.)
- **D4 — Always-focused invariant preserved.** No empty/none-focused state, no guard relaxed in #233 —
  those are #234's, where the launcher needs them.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The pure algebra shall compute the next/prev focused-workspace index (wrapping) over the open-workspace count, reusing `next_index`/`prev_index`. | `tabs.rs` unit tests (exact-value, wrap cases); cov/MSI 100 |
| REQ-002 | WHEN the user presses the workspace-cycle chord, the shim shall switch the focused workspace to the next/prev open workspace and re-sync its Files/⌘P/git/titlebar. | keymap + shim review; driven capture (env-permitting) / mechanism |
| REQ-003 | The workspace terminology (Session/Workspace/Tab) shall be documented as the pinned decision, with the code rename explicitly recorded as deferred + vetoable. | doc review (the spec D1/D2 + an arch-doc note) |
| REQ-004 | The never-empties guards + the always-focused invariant shall be UNCHANGED (no empty state reachable, no guard relaxed) in #233. | the existing guard tests still green; grep confirms no guard edit |

## Phase Plan
- **P2 Design** — pick the cycle chord (free, distinct from next/prev-tab); design the pure `cycle`
  addition to `Workspace` (reusing next_index/prev_index) + its tests; the shim dispatch; the arch-doc
  terminology note. Run `cargo mutants --list`.
- **P3 Implement** — the pure algebra (tabs.rs) then the shim (keymap.rs + app.rs) + the terminology doc.
- **P3.5 Inspect** — critics: correctness (cycle wrap/close-refocus), no-guard-relaxed, clean-room,
  terminology-doc accuracy.
- **P4 Validate** — RUN the tabs.rs tests; gate green (cov/MSI 100); driven capture of a workspace switch
  (env-permitting) else mechanism.
- **P5 Complete** — CHANGELOG + app_shell.md (the terminology + cycling); AAR; close #233.
