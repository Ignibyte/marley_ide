---
pipeline_id: 13bd5e6d-c798-47a0-aa63-5b9a4b11faa3
ticket: docs/planning/tickets/open/TICKET-420-editor-files-rail-rows.md
status: Phase 5 — Complete PASS
title: Open editor files as per-file rail rows — the #237/#240 rail PROJECTION reversed; the one-surface model stays
type: feature
milestone: M31
references:
  - docs/planning/design-notes/simple-rail-shelf.md
  - docs/planning/pipeline/queued/418-simple-rail-react-design.spec.md
  - docs/planning/pipeline/queued/419-rail-single-selection-dots.spec.md
  - docs/planning/pipeline/completed/editor-surface-tabs.spec.md
  - docs/planning/pipeline/completed/editor-rail-label.spec.md
  - docs/planning/pipeline/completed/398-add-to-pane-cross-link.spec.md
  - crates/marley_app/src/tabs.rs
  - crates/marley_app/src/editor_surface.rs
  - crates/marley_app/src/titlebar.rs
  - marley-web/artifacts/marley-ide/src/components/LeftRail.tsx
  - docs/warp_architecture/observed/beautifului-2026-08-12-notes.md
---

## Title
The M31 simple-rail batch, third slice: open editor files RETURN to the rail as per-file rows.
The disappearance was deliberate, twice: #237 (`abbdd3e`) folded N files into ONE editor surface =
one rail row (chad feedback #9, anti-clutter — "all files under a single pane"), and #240
(`b748fa8`) froze that row's label to `EDITOR_TAB_TITLE = "Editor"` (tabs.rs:363; chad live #2,
his pick: stable label over track-active). Today `open_file_in_viewer` (app.rs:5704-5785) →
`Project::open_or_switch_code` (tabs.rs:557-570) joins the ONE editor tab, and `rail_rows` emits
`label: tab.title` for it (tabs.rs:1212) — it never walks the surface's files; per-file rows exist
only as `RailLevel::CrossRef` for pane-MOUNTED files (tabs.rs:1230-1250 via `pane_mounts`
:989-1024).

#420 reverses the rail **projection only**: the Editor section lists each open view of the editor
surface as its own row — label = file basename (disambiguated), click = switch to the editor tab
AND focus that view, × = close that view — and the frozen "Editor" tab row retires. The reversal
is informed, not amnesiac: #237's clutter concern was N TAB rows in a flat rail; under the #418
grammar the Editor section IS the bounded open-files list (the VS Code "Open Editors" shape —
which #237 itself cited as the model for the SURFACE). The one-surface model, the in-editor
file-tab strip, and `EDITOR_TAB_TITLE` (tab identity + the `TabLayout::Code` restore arm,
app.rs:2380) all stay.

## Scope
### In
- **tabs.rs — the pure projection:** in the Editor section, `rail_rows` (tabs.rs:1032-1254) skips
  the `TabContent::CodeView` tab row (:1176-1223) and instead walks that tab's surface views in
  surface order — one row per view, coordinates carrying (project, editor-tab index, view index),
  `content = Some(cid)`, label = disambiguated basename. Row shape/level = D-FILE-ROW-LEVEL.
- **Label derivation, pure:** `file_basename` (tabs.rs:936) over each view's path +
  `disambiguate_labels` (titlebar.rs:61) across the project's Editor rows. NOT via `ContentLabels`
  — that input is terminal-only BY DESIGN (tabs.rs:921-931: "editor labels derive from the model
  itself (the surface rows carry paths)"); the ticket's "via the registry" is corrected here.
- **app.rs — the row render arm + verbs:** click reuses the CrossRef click body verbatim
  (app.rs:18851-18867: `switch_project` + `sync_active_project` + `Project::switch_tab` +
  persist) then `EditorSurface::activate(i)` (editor_surface.rs:387, the strip-click fn,
  app.rs:19472) + `check_active_file_external()` (the #275 activation choke); × routes a
  (p, t, i)-addressed generalization of `close_editor_file` (app.rs:8678-8703) — see D4. Per-file
  rows join the "Search tabs" label filter like CrossRef rows (:18822-18829).
- **Selection & collapse semantics preserved:** exactly one per-file row is `active` (the surface's
  `active_index`, only while the editor tab is the active tab) — the flag #419's single-selection
  renderer lights; #386 collapse hides the rows, force-expand keeps the active one visible.
- **Test re-pins per D5** + the marley-web reconcile per `## React-first (parity)`.

### Out (explicitly deferred)
- **The surface model** — one editor tab per project (#237), `open_or_switch_code`, the in-editor
  file-tab strip (app.rs:19433-19487), dedupe-by-id (#397): all untouched.
- **`EDITOR_TAB_TITLE` stays** (tabs.rs:363) — the tab's identity, ⌘W vocab, and the restore arm
  still speak it; only its rail projection retires. Ripping the const is not this ticket.
- **Persistence** — zero codec change (ids never serialize, #394; `TabLayout::Code` restores as
  today; the projection is derived fresh each render, the #390 discipline).
- **Dirty-● on rail rows** — the strip keeps it (`file_dirty_flags`, app.rs:19436); rail rows are
  registry-free. If #418's settled design wants it, that is a follow-up shim input, not this slice.
- **CrossRef rows unchanged** — a pane-mounted file keeps its ⊞ row (#398, chad's model A: two
  navigator entries, one instance); coexistence with the new home rows is D-FILE-XREF-COEXIST.
- **Rail-row DnD** — still v2 (recorded at 398-add-to-pane-cross-link.spec.md:76-78).

## Reference (§20)
**The #418 design is the primary reference** — the simple-rail grammar (one selected row, quiet
section headers, per-row × affordance) settled in marley-web and captured against
docs/warp_architecture/observed/beautifului-2026-08-12-notes.md (Sidebar Nav: ONE lit row, flat
icon+label rows; Task Rows: left-edge indicators). **VS Code's "Open Editors" section is the
observed behavior grammar** for the section itself (published docs + observation, no source read):
one row per open editor, click focuses, × closes, the active row highlighted, duplicate basenames
disambiguated, an empty state when nothing is open. Marley adopts click/×/active/disambiguation;
the projection itself (a pure `rail_rows` walk over Marley's own `EditorSurface`) is
Marley-specific — no reference-app source is consultable or needed (clean-room §20 untouched).

### Prior art
1. **Behavior maps — checked, thin, stated honestly.** docs/zed_architecture/subsystems/
   07-workspace-panes-palette.md maps Zed's workspace at the trait level (project panel = a file
   TREE; the tab bar serves the open-files role) — **no "open editors" list panel exists in the
   Zed map to match**; docs/warp_architecture/ has no editor-rail analog at all (terminal-first).
   The paying map is IN-HOUSE: the completed #237/#240 specs record exactly why the rows left
   (anti-clutter over N TAB rows; label stability for the ONE row) — both concerns are resolved by
   the #418 grammar rather than contradicted, and #240's spec even pre-authorized the flip ("a
   trivial future flip if he changes his mind").
2. **Published** — VS Code "Open Editors" (docs + observation, above). Its duplicate-basename
   answer is a muted directory suffix; Marley v1 reuses its own shipped `disambiguate_labels`
   numbering instead (D2) — a directory-suffix upgrade is a recorded candidate, not this slice.
3. **Our permissive deps + in-house seams — the paying leg.** gpui 0.2.2 (Apache-2.0) offers no
   list-item affordance beyond what the rail already uses (elements = div/list/uniform_list;
   rail rows are plain `div` rows, windowed by `rail_skip`) — nothing new to adopt;
   ropey/regex/alacritty_terminal/tree-sitter own nothing near this seam. Everything the ticket
   needs already ships in-house: `EditorSurface::{open_view, activate, close}` + `CloseOutcome`
   (editor_surface.rs:353/387/368/253-261), `files()`/`view_ids()`/`active_index()`
   (:515/:521/:526), `file_basename` (tabs.rs:936), `disambiguate_labels` (titlebar.rs:61), the
   CrossRef row + click-body precedent (tabs.rs:1230-1250, app.rs:18851), `close_editor_file`'s
   outcome routing (app.rs:8678). This ticket recomposes; it invents no mechanism.

## React-first (parity)
**UI-AFFECTING — Zone A (rail); marley-web files: `artifacts/marley-ide/src/components/LeftRail.tsx`
(+ `src/App.tsx` state).** The POC is **AHEAD** here: it already lists `openFiles` as per-file rows
(App.tsx:70-71 `openFiles`/`activeFile`; LeftRail.tsx:270-283 SubItem rows with active + onClose,
:109-111 close re-picks the active file, :293-295 the "no files open" empty caption) — #418 settles
the final look, so the React step is RECONCILE to the settled design, not greenfield. Build &
visually verify in marley-web first (localhost:5173), then port 1:1; Validate captures the parity
pair. **Ordering: after #418** (the design source + re-baselined Zone-A captures) **and soft on
#419** — #420's model work stands alone, but its `active` flag renders under #419's
single-selection + dot grammar, so land it after for one coherent visual delta.

## Locked-In Decisions
- **D1 — the projection walks the editor surface's open views, in surface order, inside
  `rail_rows`.** A pure change: no new state, no registry read, no signature widening (the surface
  is reachable through `ws` via `tab.editor()`; labels come from view paths). Derived fresh each
  render — the #390/#398 discipline.
- **D2 — row identity = ContentId; label = `file_basename` + `disambiguate_labels`.**
  (Corrected from the ticket: `ContentLabels` is terminal-only by design, tabs.rs:921-931 — the
  surface rows carry their own paths; no new shim input.) Disambiguation runs across the
  project's Editor per-file rows, order-of-appearance numbering (titlebar.rs:61).
- **D3 — click = the shipped activation composition, then view focus.** `Workspace::switch_project`
  → `sync_active_project` → `Project::switch_tab(t)` → `EditorSurface::activate(i)`
  (editor_surface.rs:387 — the same fn the strip click drives, app.rs:19472) →
  `check_active_file_external()` (#275) → `persist_grid` (#163). The CrossRef arm (app.rs:18851)
  is the body template.
- **D4 — × = the surface's `CloseOutcome` contract, (p, t, i)-addressed.** Generalize
  `close_editor_file` (app.rs:8678 — today active-tab-only by caller contract, :8684-8688) to the
  row's coordinates: `Removed(id)` → `release_editor_views([id])` (exactly one release);
  `WouldDrain` (last view) → `close_tab_at(p, t)` — **the editor tab closes**, the shipped #397
  contract, and a project drained to zero tabs is legal and renders #395's empty-center hints
  (#392 totality). Never tab-close while >1 view; × never switches tabs first.
- **D5 — the #240-era rail pins re-aim to the new projection.** Re-pin:
  `rail_rows_groups_tabs_by_section_in_fixed_order` (tabs.rs:2061 — the exact-sequence
  `(Tab, "code", Some(0))` arm becomes the per-file row + tab-count drops to 3),
  `rail_rows_preserves_original_tab_index` (:2125 — the original-index guarantee moves onto the
  per-file row's editor-tab coordinate), `rail_rows_collapses_a_non_active_section` (:2620) and
  `rail_rows_force_expands_the_active_section` (:2649) re-key their "codefile" label asserts to
  the file row. `open_or_switch_code_cases` (:1464) **stays green as-is** — its three
  `title == "Editor"` asserts pin tab identity + restore, which survive; only their rail-label
  meaning migrates to the new tests. Green-unchanged: `rail_rows_empty_sections_still_render_headers`
  (:2146), the #398 CrossRef/marker suite (:2887+), `rail_rows_row_count_is_label_independent`
  (label-independence holds by construction — paths are model data).
- **D6 — F-#386 recall: `cargo check --tests` is a phase-plan gate.** The #386 failure was a
  false-clean `cargo check` while ~14 `#[cfg(test)]` call sites were broken
  (knowledge/failures.md:400). D1 avoids the signature widening, but a new `RailLevel` variant
  fans into render matches the same blind way — run it at Implement AND Validate.

**D-FILE (Phase 2 decides, with evidence):**
- **D-FILE-ROW-LEVEL** — new `RailLevel::File` variant vs reusing `Tab`/`CrossRef` shapes.
  Recommendation: a new variant carrying the CrossRef coordinate idiom (`tab = Some(editor tab)`,
  `pane = Some(view index)`, `content = Some(cid)`) — the render arm and #419's renderer
  discriminate honestly; confirm the exhaustive-match fan-out under D6.
- **D-FILE-XREF-COEXIST** — a file open in the surface AND pane-mounted shows its per-file home
  row AND its ⊞ CrossRef row (model A). Confirm against #419's dot semantics so "open elsewhere"
  reads once, not twice.
- **D-FILE-EMPTY-SHAPE** — the zero-files empty state: bind to #418's settled treatment (the POC
  caption "no files open", LeftRail.tsx:293-295) — emitted row vs render-shim arm is P2's call.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN N files (N ≥ 1) are open in a project's editor surface, the Editor section shall emit N per-file rows in surface order and NO "Editor" tab row. | pure `rail_rows` units — the D5 re-pins (`rail_rows_groups_tabs_by_section_in_fixed_order`, `rail_rows_preserves_original_tab_index`) |
| REQ-002 | WHEN a per-file row is clicked, the app shall switch to that project + editor tab and the surface shall focus that view (`activate(i)`), running the #275 activation choke. | unit via the pure model (row coordinates → activate) + headless/driven smoke of the click body |
| REQ-003 | WHEN a per-file row's × is invoked, that view shall close via `EditorSurface::close(i)` — `Removed` releases exactly one view; WHEN it was the LAST view, the editor tab shall close (`WouldDrain` → `close_tab_at`), leaving the #395-legal empty state. | units over the (p, t, i) close seam incl. the last-view case; `editor_surface` + #395 suites green |
| REQ-004 | WHEN two open files share a basename, their rows shall carry distinct labels (`disambiguate_labels` order-of-appearance numbering). | pure unit over the Editor-section label pass |
| REQ-005 | WHEN a project's editor surface holds zero views (no editor tab), the Editor section shall render #418's empty state (header always present; the settled empty treatment). | unit (`rail_rows_empty_sections_still_render_headers` green) + the parity/visual check |
| REQ-006 | The active view's row — and ONLY it among the per-file rows — shall be `active` iff the editor tab is the active tab; #386 collapse hides the rows and force-expand keeps the active one visible; under #419 it is the single lit row. | pure units — the D5 collapse/force-expand re-pins + an exactly-one-active assert |
| REQ-007 | WHEN Validate runs, the React POC rail (localhost:5173) and the Marley rail shall present the same Editor-section behavior — per-file rows, click focus, × close, empty state — the parity pair captured. | React capture + Marley driven/headless capture receipt |

## Phase Plan
- **P2 Design** — verify the SHIPPED shapes and bind to them (the #384-D7 pattern): the surface
  walk (pair `files()` paths with `view_ids()` cids, or a zipped accessor — P2 picks), settle
  D-FILE-ROW-LEVEL / D-FILE-XREF-COEXIST / D-FILE-EMPTY-SHAPE against #418's settled captures;
  the (p, t, i) close-shim signature; the disambiguation pass placement; the exact re-pin asserts
  (D5 list); the marley-web reconcile manifest; per-REQ test plan; `cargo mutants --list -f` on
  tabs.rs + the touched app.rs region.
- **P3 Implement** — React-first reconcile in marley-web (per `## React-first (parity)`); then
  Rust: the pure projection in tabs.rs, the render arm + click/× listeners in app.rs, the
  generalized close fn; `cargo check --tests` before declaring the phase (D6).
- **P3.5 Inspect** — independent critics vs the diff: projection purity (no registry reads in
  `rail_rows`), cross-PROJECT row clicks (switch-project-first correctness), release symmetry on
  every × path (exactly one release per `Removed`; `WouldDrain` releases via the tab close),
  collapse/force-expand + search-filter behavior, CrossRef coexistence, provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; `cargo check --tests` (D6); the React↔Marley
  parity pair (REQ-007); full gates green (`--diff`); D5 green-unchanged suites confirmed.
- **P5 Complete** — CHANGELOG + rail docs touched by the projection change; ledger capture (§19);
  archive; close TICKET-420 + drop its BACKLOG row.
