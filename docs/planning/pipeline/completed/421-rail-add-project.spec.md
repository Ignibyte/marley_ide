---
pipeline_id: 6c90ac65-cede-44e9-961d-1d34ef0eaf68
ticket: docs/planning/tickets/open/TICKET-421-rail-add-project.md
status: Phase 5 — Complete PASS
title: The rail gains an add-project door — the #418 rail's ＋ wires the two SHIPPED verbs through the ONE menu machinery
type: feature
milestone: M31
references:
  - docs/planning/design-notes/simple-rail-shelf.md
  - docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png
  - docs/warp_architecture/observed/beautifului-2026-08-12-notes.md
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/context_menu.rs
  - crates/marley_app/src/keymap.rs
  - marley-web/artifacts/marley-ide/src/components/LeftRail.tsx
  - marley-web/docs/MARLEY-PARITY.md
---

## Title
The M31 simple-rail batch's wiring slice: projects become addable **from the rail itself**. Today the doors
are ⌘O (keymap.rs:144-147 → the key-router special case app.rs:17956-17959 → `open_project_picker`
app.rs:8500-8522) and the empty-workspace launcher card (app.rs:10626-10720 — recents → `open_project_path`,
"Open Folder…" :10694, "New empty workspace" :10717). **There is NO palette door** — the ticket's "or the
palette" claim is wrong; "open-project" reaches the picker only via the key router, and none of the 29
cockpit commands (app.rs:10314-10625) is a folder/project verb. The rail's only ＋ is the Section-header ＋
(render ~app.rs:19056-19087 → `open_section_menu` :7400) and it creates tab content, never projects. Per
#418's settled design, the rail gains an add-project verb that opens a small menu — the ONE context-menu
machinery (context_menu.rs) — carrying the two existing verbs: "Open folder…" (`open_project_picker`) and
"New empty workspace" (`new_empty_workspace`, app.rs:8485-8495). **No new project machinery**: placement is
#418's call; this ticket locks the wiring.

## Scope
### In
- **The rail affordance** at #418's settled placement (D2), opening a new add-project menu kind through the
  ONE machinery (a `MenuKind` arm + a const row table — the #393/#398 extension idiom, context_menu.rs).
- **Two rows, two shipped verbs (D1):** "Open folder…" → `open_project_picker` (app.rs:8500-8522, the gpui
  native directory prompt); "New empty workspace" → `new_empty_workspace` (app.rs:8485-8495). Dispatch only;
  the post-add tail is `open_project_path` (app.rs:8458-8479) unchanged.
- **The F-#236 pin (D4):** a unit proving add appends at the END — existing project indices unshifted,
  index-keyed view state still addressing the same projects, the new project active.
- **Parity:** the React-first build in marley-web against #418's design (see below) + the Validate pair.
- **Regression pins:** the launcher card suite, the keymap ⌘O pin (keymap.rs:626-628), the #393
  section-menu suite — all green unchanged.

### Out (explicitly deferred)
- **Any new project machinery** — no recents submenu in the rail, no drag-a-folder-onto-the-rail, no
  multi-select picker (`multiple: true` stays false), no workspace-switcher header (the beautifului header
  chip is #418 visual grammar, not a verb).
- **A palette "Open Folder…" command** — none exists today (corrected above) and none is added here; if
  wanted it is its own one-row micro-ticket.
- **Close/reorder affordances** — the remove side of F-#236 (`remap_indices_after_remove`, app.rs:8716)
  stays untouched; no project reordering.

## Reference (§20)
**The beautifului.dev Sidebar Nav capture — the M31 rail's design source (chad-picked), observed and saved:**
docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png + beautifului-2026-08-12-notes.md —
the accent action row ("New task" + a filled accent ＋ circle) is the add-verb grammar this affordance
follows; the notes record "add-verb as an accent ＋ … settled at #418". **Warp — checked, honestly thin:**
docs/warp_architecture/subsystems/ maps no workspace/folder-add affordance anywhere; 00-overview.md:45
records Marley's multi-workspace launcher as EXCEEDS ("Warp has no multi-workspace launcher") — the rail
project model is `[Marley-original]`. Research, not source; no AGPL/GPL source consulted.

### Prior art
1. **Behavior maps — checked.** Warp: nothing mapped (above). Zed: 06-project-fs-search.md maps the
   worktree/project model (Marley's `Project` ≈ Zed's *Worktree*, :53) but no add-folder gesture is mapped;
   research only. In-house is the real precedent: the #393 section-＋ menu (a8e97fd — ＋ opens a menu of
   reuse-only verbs, target rides the kind) and #398's `AddToPane` (a new `MenuKind` + const table is the
   documented extension shape).
2. **Published (observed behavior).** ChatGPT's sidebar: a dedicated "New project" affordance row lives in
   the rail itself, projects list beneath — the add verb belongs to the list it feeds. VS Code multi-root:
   "Add Folder to Workspace…" opens the native folder picker and APPENDS the folder to the end of the roots
   — no reshuffle, focus moves to the new root. Both match the semantics Marley already ships.
3. **Our permissive deps — the picker is already adopted, not built.** `open_project_picker` uses **gpui's
   own native prompt** — `cx.prompt_for_paths(gpui::PathPromptOptions { files: false, directories: true,
   multiple: false, prompt: None })` (app.rs:8501-8506), awaited off the UI thread via `cx.spawn`; the
   cancel arm (`Ok(Ok(None))`) short-circuits before `open_project_path`. No rfd, no new dep, nothing to
   build — this ticket adds zero picker code. ropey/regex/alacritty_terminal own nothing near this seam.

## React-first (parity)
**UI-AFFECTING — Zone A (frozen shell: the rail).** Port-map row: `components/LeftRail.tsx` ↔ app.rs (rail)
/ tabs.rs / context_menu.rs (marley-web/docs/MARLEY-PARITY.md:572); menu rows mirror in
`components/ContextMenu.tsx`. The POC already mocks the dock-header ＋ (LeftRail.tsx:183-184, `title="Add
project"` — today it just toggles the `twoProjects` demo flag); #418 re-baselines the whole rail and settles
the real placement, then this ticket makes the affordance dispatch real verbs. Plan line: **build & visually
verify in marley-web first (`pnpm --filter @workspace/marley-ide run dev` → localhost:5173) against #418's
settled design, then port 1:1; Validate captures the React↔Marley parity pair.** Ordering: **after #418**
(shelf order 418 → 419 → 420 → 421) — this spec binds to #418's placement token, not to pixels.

## Locked-In Decisions
- **D1 — wiring is REUSE-ONLY through the ONE menu machinery.** The affordance opens a new add-project menu
  kind (a `MenuKind` arm + a const row table + full-slice-equality test, the #393/#398 idiom) whose two rows
  dispatch the EXISTING verbs `open_project_picker` / `new_empty_workspace` — no new project machinery, no
  parallel popover, one-modal discipline (`close_transient_overlays` before open, the #393 shape). The cx
  constraint is load-bearing: `open_project_picker` NEEDS `cx` (the #156 lesson, app.rs:17956 — cx-less
  `dispatch_action` cannot host it); the menu dispatch runs in the render listener's `cx`.
- **D2 — placement is #418's settled token (settled-at-418, verified at Design).** The two candidates: (a)
  the rail dock-header ＋ (the POC's existing mock, LeftRail.tsx:183-184; the beautifului accent-＋ grammar)
  or (b) a trailing "Add project…" row after the project list. This spec locks the wiring either way; Phase 2
  binds to whichever #418 shipped. If (b), `rail_rows` gains a row kind → the F-#386 rule applies
  (`cargo check --tests`, ~14 hidden test call sites).
- **D3 — post-add focus is whatever `open_project_path` already does — verified, no new behavior.**
  `Workspace::add_project` (tabs.rs:604-607) appends and sets `active = len-1` → the new project IS the
  active project; then `sync_active_project` (app.rs:8189 — Files re-walk to the new root, scroll reset),
  `record_recent` (#234 MRU for the launcher), `persist_grid`. A failed PTY spawn is a silent no-op (the
  whole tail sits inside `if let Ok(session)`) — unchanged here.
- **D4 — index-shift safety (F-#236): add is the append case, and the test pins it.** Because add_project
  pushes at the END, existing indices never shift — `collapsed_projects` (app.rs:172), `collapsed_sections`
  (:175), `renaming_tab` (:478), `naming_pane` keep addressing the same projects with NO remap; remapping
  (`remap_indices_after_remove`, :8716) remains a CLOSE-only concern. Invariant: **adding a project never
  re-keys existing view state and activates only the appended index** — REQ-004's unit makes it a regression
  pin, not an assumption.
- **D5 — no empty-workspace regression.** The launcher card keeps both rows and the recents list; ⌘O keeps
  working; this ticket adds a door and removes none. No palette row appears (none exists — the corrected
  claim), so no palette surface changes either.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the rail add-project verb is invoked, its menu's "Open folder…" row chosen, and the native picker returns a folder, the workspace shall gain a project row for that folder — appended at the END — and it shall become the ACTIVE project (Files synced to the new root, recent recorded, switch persisted — the shipped `open_project_path` tail, unchanged). | unit over `Workspace::add_project` (append + `active = len-1`) + headless smoke over the menu wiring (the affordance opens the ONE menu with exactly the two rows; row dispatch reaches `open_project_picker`) |
| REQ-002 | WHEN the picker is cancelled, workspace state shall be unchanged — no project added, no active-pointer move, no persist write. | unit/smoke pinning the cancel arm (the `Ok(Ok(Some(…)))` guard at app.rs:8509 falls through before `open_project_path` — existing behavior, now pinned) |
| REQ-003 | WHEN the menu's "New empty workspace" row is chosen, the system shall open a project at the existing default-dir fallback ($HOME → launch cwd → temp dir, app.rs:8485-8495) — the SAME fn the launcher row dispatches, no fork. | smoke + review: both doors call the one `new_empty_workspace`; launcher suite green |
| REQ-004 | WHEN a project is added WHILE index-keyed view state exists (collapsed projects/sections, a live rename draft), that state shall keep pointing at the SAME projects afterward, and only the appended project shall become active. | unit — the F-#236 pin: pre-add `collapsed = {i…}` → add → the set is untouched AND still addresses the same roots; active = the new last index |
| REQ-005 | The existing doors shall be unchanged: ⌘O still opens the picker (keymap.rs:144-147 → app.rs:17957) and the launcher card still renders and dispatches both rows — a door added, none removed. | keymap pin green (keymap.rs:626-628) + launcher smoke + review (and no palette row added — none exists today) |
| REQ-006 | WHEN Validate runs, the React POC affordance (confirmed at localhost:5173 against #418's settled design) and the Marley rail shall present the same add-project affordance and menu. | the React↔Marley parity pair captured (the #398 REQ-008 protocol; env-blocked fallback documented if driving is blocked) |

## Floors (constitution)
Pure seams at **cov/MSI 100**: the new menu row table (the `section_items`/`items_for` full-slice-equality
idiom) and the `Workspace::add_project` pins. MASKED: the app.rs affordance render + dispatch shims (app.rs
is coverage-excluded; `open_project_picker`/`new_empty_workspace`/`open_project_path` are already
`#[cfg_attr(test, mutants::skip)]`). Re-run `cargo mutants --list -f` on the actual touched files after
placement; if D2 lands on the trailing row, `cargo check --tests` before declaring Implement clean (F-#386).

## Phase Plan
- **P2 Design** — re-verify the SHIPPED #418 rail (bind to its real placement token + its React affordance,
  not this spec's prediction); settle the menu-kind shape (recommendation: a NEW workspace-scoped `MenuKind`
  arm — the section ＋ tables are per-section CREATE verbs and stay untouched) and the cx routing (render
  listener's `cx`, never `dispatch_action` — the #156 lesson); exact row labels; both-halves file manifest
  (LeftRail.tsx/ContextMenu.tsx first, then context_menu.rs + app.rs shims); per-REQ test plan.
- **P3 Implement** — React-first: wire the affordance in marley-web at localhost:5173 against #418's design;
  then Rust: the pure row table + kind, then the masked shims (open the menu at the click; dispatch the two
  verbs).
- **P3.5 Inspect** — independent critics vs the diff; lenses: one-modal discipline (`close_transient_overlays`
  before open); hit-target separation (the affordance click stops propagation — the #387 lesson); the cancel
  arm truly untouched; cx routing; F-#236/F-#386 re-checks; provenance (§20).
- **P4 Validate** — write + RUN tests per REQ; the parity pair (REQ-006); gate green (`--diff`); launcher +
  keymap + #393 section-menu suites green unchanged.
- **P5 Complete** — CHANGELOG; tick the shelf note row; ledger capture (§19 — record the corrected "palette
  door" claim); archive; close #421 + drop its BACKLOG row.
