# The rail gains an add-project door — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-421-rail-add-project.md
- **Pipeline spec:** 421-rail-add-project.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** Chad's M31 "simple rail" batch (shelf: docs/planning/design-notes/simple-rail-shelf.md —
  418→419→420→421→422, his order): projects on the left, **addable** — an add-project verb in the rail
  itself, wiring the EXISTING picker/new-workspace paths per #418's settled design. Visual grammar he
  picked: beautifului.dev Sidebar Nav (the accent ＋).
- **Classification / tier:** feature, M31, small — wiring-only (a menu kind + two rows dispatching two
  shipped verbs); UI-affecting Zone A (rail) → React-first parity applies; ordered AFTER #418.
- **Recall (§18.3):**
  - **F-#236** (failures.md `BF-claude-index-keyed-state-not-remapped-on-vec-remove`, ~:410):
    `collapsed_projects` aliased the wrong project when a Vec-shifting remove wasn't remapped. Add is the
    APPEND case — no shift — but the spec pins it as a unit (REQ-004) instead of assuming it; the
    close-side remap (`remap_indices_after_remove`, app.rs:8716) stays untouched.
  - **#393 precedent (a8e97fd — "the rail section ＋ becomes a menu of create-verbs"):** the exact shape to
    reuse — a ＋ that opens the ONE menu (target rides the kind, #175; one-modal via
    `close_transient_overlays`; stop-propagation hit-target separation, the #387 lesson). #398's
    `AddToPane` kind is the "new `MenuKind` + const table + full-slice test" extension precedent.
  - **#156 lesson (app.rs:17956):** `open_project_picker` needs `cx`; cx-less `dispatch_action` can't host
    it — the key router special-cases it. The menu dispatch must run in the render listener's `cx`.
  - **F-#386:** if placement lands on a trailing rail ROW, `rail_rows` changes → `cargo check --tests`
    (plain check misses ~14 `#[cfg(test)]` call sites).
- **Discovery (verified 2026-08-12, every line read):**
  - **Doors today:** ⌘O — keymap.rs:144-147 ("open-project") → key-router special case app.rs:17956-17959 →
    `open_project_picker` app.rs:8500-8522. Launcher card app.rs:10626-10720 — recents rows →
    `open_project_path`; "Open Folder…" :10694 → picker (:10698); "New empty workspace" :10717.
    **The ticket's "or the palette" claim is FALSE** — "open-project" appears only in keymap.rs (binding
    :146, pin test :628) and the app.rs router arm; none of the 29 cockpit commands (app.rs:10314-10625)
    is a folder/project verb. Spec corrected to two doors; no palette row added (scoped out).
  - **The picker is gpui-native, already adopted:** `cx.prompt_for_paths(gpui::PathPromptOptions { files:
    false, directories: true, multiple: false, prompt: None })` (app.rs:8501-8506), awaited via `cx.spawn`;
    cancel falls through the `Ok(Ok(Some(…)))` guard (:8509) before `open_project_path`. Not rfd; zero new
    picker code (Prior-art leg 3).
  - **Ordering + active pointers (the F-#236 question, answered):** `Workspace::add_project`
    (tabs.rs:604-607) = `projects.push(project); active = len-1` — appends at the END, existing indices
    unshifted, the new project becomes ACTIVE. Post-add tail (`open_project_path` app.rs:8458-8479):
    `spawn_session_in` → registry insert (#396) → `add_project` → `sync_active_project` (app.rs:8189 —
    Files re-walk, scroll reset) → `record_recent` (#234 MRU) → `persist_grid`. Failed PTY spawn = silent
    no-op (the tail sits inside `if let Ok(session)`).
  - **Index-keyed view state:** `collapsed_projects` app.rs:172, `collapsed_sections` :175 (root-persisted,
    index-keyed in memory), `renaming_tab` :478, `naming_pane` :181 — none needs a remap on append (D4).
  - **Menu machinery:** context_menu.rs (pure state + placement; `MenuKind`/`MenuAction`; const row tables
    + full-slice-equality tests); `open_section_menu` app.rs:7400-7412; section-＋ render ~app.rs:19056-19087.
  - **React POC:** the dock-header ＋ mock already exists — LeftRail.tsx:183-184 (`title="Add project"`,
    dispatches the `twoProjects` demo toggle). #418 re-baselines the rail + settles placement; #421 makes
    the affordance real. Port-map row: MARLEY-PARITY.md:572 (LeftRail.tsx ↔ app.rs rail / tabs.rs /
    context_menu.rs).
  - **Prior-art sweep:** Warp maps no folder-add affordance (00-overview.md:45 — the launcher EXCEEDS Warp);
    Zed's 06-project-fs-search.md maps the worktree model, no add gesture (research only). Published:
    ChatGPT sidebar "New project" row (verb lives in the rail it feeds); VS Code "Add Folder to
    Workspace…" (native picker, append-at-end, focus to the new root) — both match Marley's shipped
    semantics. Reference capture: docs/warp_architecture/observed/beautifului-sidebar-nav-2026-08-12.png +
    notes (the accent ＋ grammar, "settled at #418").
- **Decisions:** D1 reuse-only wiring through the ONE menu machinery (cx constraint named); D2 placement =
  #418's settled token (dock-header ＋ vs trailing "Add project…" row — wiring locked, pixels #418's);
  D3 post-add focus = the shipped `open_project_path` tail (append + activate + sync + recent + persist);
  D4 append-case index safety pinned by unit (F-#236); D5 launcher/⌘O doors unchanged — a door added, none
  removed. EARS: 6 rows (REQ-001…006).

## Phase 2 — Design
- Architecture / approach; file manifest; regression test plan; risks.

## Phase 3 — Implement
- What was built; deviations from design (with reason).

## Phase 3.5 — Inspect
- Critics run; findings table (severity / finding / verdict); fixes.

## Phase 4 — Validate
- Tests RUN (with counts) + gate result; negative smokes; pre-existing notes.

## Phase 5 — Complete
- Docs updated; ledger appends (lessons / failures / prevention rules / ADs — codes listed); archive.

- **Promotion (2026-08-12, the `/work 418-422` batch):** #418/#419/#420 all SHIPPED (2d244f6 /
  9967ef0 / 6254b3d) — the ordering constraint holds and **D2's placement token is settled BY THE
  AS-BUILT #418: candidate (a)** — the persistent accent ＋ in the rail's "Workspace" top band
  (POC DD-3/D4 as-built; today it drives the twoProjects mock — this ticket makes it dispatch
  real verbs). Placement (a) ⇒ NO `rail_rows` row-kind change ⇒ the D2 F-#386 branch is moot
  (still run `cargo check --tests` per discipline). Seams re-verified by symbol (lines drifted
  from #419/#420): `open_project_picker` app.rs:8519 (cx.prompt_for_paths :8520 ✓),
  `new_empty_workspace` :8504, `open_project_path` :8477, `open_section_menu` :7419,
  `Workspace::add_project` tabs.rs:604 (append + active=len-1), `remap_indices_after_remove`
  tabs.rs:877, keymap "open-project" :146 + the pin :628. The Rust ＋ target: the Left dock's
  "Workspace" caption header (`dock_title`/caption_header region, app.rs ~:7253/:18494 — exact
  render site bound at Design). BACKLOG row removed (§19).

## Phase 2 — Design

### §20 confirm + placement binding
Reference confirmed (the beautifului accent-＋ grammar, observed; Warp N/A). **D2 resolved by the
as-built #418: candidate (a)** — the "Workspace" top-band ＋. Rust target: the LEFT dock's caption
header (`dock_panel` → `caption_header`, the #190 pixel-identical pair). No `rail_rows` change ⇒
no new row kind ⇒ the D2/F-#386 branch stays dormant (D6-style `cargo check --tests` still runs).

### The shape (all REUSE through the ONE machinery — the #393/#398 idiom, verified in-tree)
- **context_menu.rs:** `MenuKind::AddProject` (payload-less — workspace-scoped verbs, no target to
  ride); `MenuAction::OpenProjectFolder` + `MenuAction::NewEmptyWorkspace`;
  `ADD_PROJECT_MENU_ITEMS: [(MenuAction, &str); 2]` with the LAUNCHER's exact labels
  ("Open Folder…" app.rs:10735, "New empty workspace" :10754 — one vocabulary, two doors);
  `items_for(AddProject) => &ADD_PROJECT_MENU_ITEMS`. Full-slice-equality test (the #393 idiom).
- **app.rs — the header affordance:** `caption_header` becomes a flex row (visual no-op for
  label-only callers) and `dock_panel` gains `header_action: Option<AnyElement>` (call sites:
  LEFT dock passes the accent ＋; the right dock + files_panel pass None). The ＋: caption-sized
  accent "＋", `on_mouse_down(Left)` → `cx.stop_propagation()` (the #387 hit-target lesson) →
  `open_add_project_menu(x, y)`.
- **app.rs — the open shim** (the `open_section_menu` :7419 shape): `close_transient_overlays()`
  (one-modal, #393) then `context_menu = ContextMenuState::with_kind(x, y, MenuKind::AddProject)`.
- **app.rs — dispatch:** `run_context_menu_action` (:8041 — HAS `cx: &mut Context<Self>`, the D1
  cx constraint satisfied at the shipped seam): two new arms —
  `OpenProjectFolder => self.open_project_picker(cx)`, `NewEmptyWorkspace =>
  self.new_empty_workspace(cx)`. Dispatch-only; the post-add tail (`open_project_path` → append +
  activate + sync + record_recent + persist) is untouched (D3).
- **React half (P3 first):** the POC's top-band ＋ (today `updateState({ twoProjects: true })`)
  opens the POC ContextMenu with the same two rows (a new `{ t: 'AddProject' }` kind in
  components/ContextMenu.tsx); both rows drive the `twoProjects` mock outcome (the AFFORDANCE +
  MENU grammar is what parity pins; the real verbs are Rust-side — recorded mock semantics).

### Test plan (per REQ)
| REQ | Proof |
|---|---|
| REQ-001 | `ADD_PROJECT_MENU_ITEMS` full-slice equality (pure); `Workspace::add_project` append+active unit (augment the existing suite if present); headless smoke: `open_add_project_menu` (pub(crate), the #198 listeners-reuse-methods precedent) → `context_menu` holds AddProject with exactly the 2 rows; the picker dispatch itself = the live drive (P4) |
| REQ-002 | the cancel arm: review + the live drive (a native prompt has no headless double; the guard `Ok(Ok(Some(...)))` is shipped behavior — recorded, not unit-forced) |
| REQ-003 | both doors call the ONE `new_empty_workspace` (review — grep two call sites, zero forks); launcher suite green |
| REQ-004 | the F-#236 pin: pre-add `collapsed = {i}` → `add_project` → the set untouched AND still addressing the same roots; `active == len-1` (pure unit) |
| REQ-005 | keymap pin :628 green; launcher suite green; `open_section_menu`/#393 suite green — a door added, none removed |
| REQ-006 | the parity pair: POC menu screenshot/DOM vs the live-drive menu capture |

### Risks
- **R1** `caption_header`/`dock_panel` signature ripple (≤4 call sites, compile-visible).
- **R2** the ＋ click must not toggle/collapse anything beneath (stop_propagation pinned by the
  #387 lesson; the header has no other click target today — verified: caption_header has no
  listeners).
- **R3** the POC menu mock's two-rows-one-outcome is scaffolding — recorded in the notes, not a
  parity delta (the menu GRAMMAR is the pinned surface).

## Phase 3 — Implement

- **React-first (verified live at :5173):** ContextMenu.tsx gained the `{ t: 'AddProject' }` kind
  + the two actions + `ADD_PROJECT_MENU_ITEMS` (the launcher's exact labels); LeftRail's ＋ now
  takes `onAddProjectMenu` (no more direct mock toggle); Workspace passes `openAt(e, {t:'AddProject'})`
  and dispatches both actions onto the `twoProjects` mock outcome (recorded scaffolding — the menu
  GRAMMAR is the parity surface). DOM-verified: the ＋ opens the menu with exactly
  ["Open Folder…", "New empty workspace"]. Typecheck green.
- **context_menu.rs:** `MenuKind::AddProject` (payload-less, doc'd why); `MenuAction::
  OpenProjectFolder` + `NewEmptyWorkspace`; `ADD_PROJECT_MENU_ITEMS` const table;
  `items_for(AddProject)` arm.
- **app.rs:** the dispatch arms in `run_context_menu_action` (which carries the `cx` the picker
  needs — dispatch-only onto the two shipped verbs); `caption_header` became a flex row with an
  optional trailing `action` slot (label-only callers pixel-identical); `dock_panel` passes
  `header_action` through (right dock + files_panel pass None); the LEFT dock passes the accent
  ＋ (`＋` U+FF0B, caption-sized, `cursor_pointer`, stop_propagation → `open_add_project_menu(x,y)`);
  the open shim (`pub(crate)` for the headless lane) does `close_transient_overlays()` then
  `ContextMenuState::with_kind(x, y, AddProject)` — the #393 shape.
- **D6:** `cargo check --workspace` + `--tests` green (the exhaustive dispatch match forced the
  two arms — the designed compile-visible fan-out); fmt clean.
- **Deviations:** none.

## Phase 3.5 — Inspect

Two critics (Rust wiring / POC+machinery). Ledger:

| # | Finding | Verdict | Fix |
|---|---|---|---|
| 1 | The new fn spliced between `open_section_menu`'s doc + `#[cfg_attr(test, mutants::skip)]` and the fn — the doc AND the mutants mask attached to the NEW fn, silently stripping the sibling shim's mask (a future diff touching it would chew missed mutants) | **REAL (med-high) — the catch** | reordered: the new fn carries its OWN doc + skip; the section-menu doc/attr restored onto their fn |
| 2 | Glyph/hover drift: header ＋ was U+FF0B with no hover vs the section ＋'s ASCII "+" + hover accent (the POC uses one lucide Plus for both) | **REAL (low)** | ASCII "+" + the sibling hover treatment |
| 3 | POC mock skipped the Rust twin's ACTIVATION (`add_project` sets `active = len-1`; the mock left project 1 selected — and re-choosing was a silent no-op) | **REAL (low)** | the mock now selects project 2 (`activeProject: 2` + the selection-hygiene fields) |
| 4 | Stale POC comments (the band tail still said "until #421 wires the real picker"; :657 future-tense) | **REAL (doc)** | both rewritten |
| 5 | `ADD_PROJECT_MENU_ITEMS` private where every sibling table is `pub` | **REAL (nit)** | `pub const` |
| 6 | A live fleet-dispatch draft / editor overlays leave the menu keyboard-dead (`close_transient_overlays` deliberately spares them; the key arms precede the menu arm) | **inherited #393-class, pre-existing** — mouse-recoverable; recorded for the ledger, not this diff |
Clean (verified): the keyboard path is kind-generic (esc/↑/↓/enter over any kind; Enter carries the
listener's real `cx` → the picker works from keyboard); one-modal matches the #393 sibling exactly;
no ancestor click listeners (stop_propagation defensive); the caption_header flex ripple is inert
(3 callers, short labels, no truncation machinery to regress); the cancel guard untouched; labels
BYTE-IDENTICAL across POC/Rust/launcher (hex-verified, real U+2026); F-#236 immune (payload-less
kind, no index-keyed state touched); menuOrigin clamping safe at the ＋'s corner; provenance
in-house. Post-fix: cargo check --tests 0 errors, POC typecheck green, fmt clean.

## Phase 4 — Validate

**Tests written + RUN:**
- `add_project_menu_items_table_and_routing` (context_menu.rs — full-slice equality on the const
  table, the `items_for(AddProject)` routing, and the keyboard wrap over the 2 rows).
- `add_project_appends_without_rekeying_view_state` (tabs.rs — the F-#236/REQ-004 pin: append
  activates `len-1`, existing indices unshifted, an untouched collapsed set still addresses the
  SAME project and the rail renders it collapsed — remap stays CLOSE-only).
- `add_project_menu_opens_with_the_two_doors` (headless_drive.rs — the REQ-001 menu half: the
  pub(crate) shim the render listener calls opens the ONE menu, kind == AddProject, labels exactly
  ["Open Folder…", "New empty workspace"]). `context_menu` widened to pub(crate) for the lane.
- **Run:** `cargo nextest run --workspace` → **2137/2137 passed** (5 skipped); doctests clean;
  fmt clean. REQ-005's suites in the run: the keymap ⌘O pin, the launcher suite, the #393
  section-menu suite — all green unchanged.

**Live app driven (fresh bundle):** the Workspace dock header renders the accent ＋ (the #418
top-band token); clicking it opened the menu at the click point with EXACTLY the two doors
(captured + read); choosing "New empty workspace" ran the WHOLE shipped tail live — the
"chadpeppers" project ($HOME default-dir fallback) APPENDED after Marley·main, ACTIVATED
(its terminal tab took the ONE #419 fill), the Files dock re-walked to the new root, the PTY
spawned (REQ-001 append+activate, REQ-003 the same-fn door — both live). The picker row (a
native dialog) was NOT driven (REQ-002's cancel arm = the shipped guard, review-verified —
recorded). Cleanup: the added project closed via its × (the remove side — #236 remap — ran
clean); Chad's workspace restored byte-visibly (captures at every step, /tmp/marley-421-*.png).

**Parity pair (REQ-006):** the POC ＋ → menu DOM-asserted at P3 (menuOpen: true, items exactly
the two labels — byte-identical to the Rust table, hex-verified at inspect); the Marley menu
captured live above. The POC screenshot backend remains flaky (the #420-noted timeout) — the
DOM assertion + the Marley captures are the pair's receipt.

**Gate:** **GATE GREEN [diff], 15 passed, 0 failed** (coverage ≥100, MSI ≥100 on touched lines, miri, visual). Receipt written.

## Phase 5 — Complete
- **§21:** CHANGELOG entry; app_shell.md M31 #421 entry. Parity: MARLEY-PARITY port-map row
  ticked LANDED (labels byte-identical POC↔Rust↔launcher). The corrected "palette door" claim
  is recorded in the spec's Title (no palette row exists or was added).
- **§19:** PR-claude-splicing-a-fn-between-doc-attr-and-item-steals-them-001 (inspect);
  L-claude-421-one-menu-machinery-absorbs-new-verbs-for-free-001 (complete).
- **Ticket** → closed/; shelf ticked; BACKLOG clean; pair → completed/.
