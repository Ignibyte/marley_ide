---
pipeline_id: 3511c80d-a0cf-48ad-a0bd-138f77de1ce0
ticket: forge#259 (a8bcfc50-90ad-4afe-a255-76c3edb0bacc) · local docs/planning/tickets/open/TICKET-259-editable-split-pane.md
aar_id: 8d17f4a9-f400-4bae-8f5a-6b4f02f64f58
status: Phase 5 — Complete PASS. GATE GREEN [diff] 15/15 (coverage 100% workspace.rs+tabs.rs seams; mutation MSI 100% 14/14; the F1 blank-pane fix drive-proven). SCOPE = SLICE 1; follow-ups forge #355/#356/#357. Inspect caught + FIXED a real HIGH (a shared 'static render closure re-read `active_tab().editor()` = None for a split pane's Terminal host → blank pane; the G4 regex missed it — receiver was `entity.read(app)`). 2 documented slice-1 limitations (unfocused pane shows stale `cv.lines`; no #275 banner in the pane render — both follow-ups; no data loss). Validate must add the workspace-seam mutation-kill cluster + a focused-pane-renders-≥1-row test. Deviation: REQ-004 close-dirty prompt scoped out (editor tab has none). SLICE 2 = named follow-up.
title: Editable split-file pane — a second editing surface, and the four single-slot assumptions it breaks
type: feature
milestone: M15
references: [the #258-plan MEASUREMENT: 16+ sites key editing off active_tab().editor() (the ticket records the line list), THE FOUR SINGLE-SLOT FACTS (recon 2026-07-17): (1) the split pane is PaneContent::CodeView(CodeViewState) — a LOSSY file read, NO Buffer (workspace.rs:325, app.rs:4305); (2) ONE FocusHandle app-wide (app.rs:144) + PaneGrid.focused is the only pane-focus notion; (3) KeyContext publishes PER TAB (tabs.rs:144) — a terminal tab says Terminal even when its CodeView pane is focused; (4) editor_geom is ONE Cell + handle_input registers THE editor (app.rs:405/:4877 — two editable panes would fight, last-painted wins); SelectionSet lives INSIDE Buffer (buffer.rs:36 — two views of one Buffer share one cursor set); the #275 extchange conflict machinery (the same-path safety net); save_active saves the active TAB's editor (app.rs:6384)]
---

## Title
The #246/#258 split pane shows a file beside your terminal — read-only. This makes a FOCUSED split pane a
real editing surface: typing, caret, selection, save, undo, clipboard, motion — the editor stack, in a
pane. It is the batch's one genuinely ARCHITECTURAL ticket: not a feature bolted on, but the first time
"the editor" stops being a singleton. Four shipped single-slot assumptions break, each named below with
its fix shape — the ticket IS those four seams plus a rewire.

## Scope
### In
- **The pane grows a buffer:** `PaneContent::CodeView(CodeViewState)` (a lossy display-lines snapshot)
  becomes an EDITABLE pane state carrying the OpenFile shape (`view + Buffer + saved_version + nonce +
  marked + disk-stat` — the editor_surface.rs:32 fields, reused not re-invented; design decides
  extract-a-shared-struct vs parallel fields). The #258 `c=<path>` grid-codec persistence keeps working
  (it stores the PATH; the buffer loads on restore exactly as the editor tab's files do).
- **THE SHARED ACCESSOR (the ticket's spine):** `focused_editable_surface() / _mut()` — resolves to the
  active tab's `EditorSurface` (today's path) OR the focused terminal-tab pane's editable CodeView. The
  #258 plan MEASURED the blast radius: 16+ sites key editing off `active_tab().editor()` (typing/IME,
  save, clipboard, motion, undo, render, caret click). Each site moves to the accessor — a mechanical
  rewire made safe by the type (one accessor, two backing stores). Sites that are genuinely TAB-scoped
  (tab titles, tab close-dirty checks) deliberately do NOT move; the diff review names each.
- **Focus routes keys (single-slot fact #3):** `key_context()` becomes focus-aware — a terminal tab whose
  FOCUSED pane is an editable CodeView publishes `KeyContext::Editor` (so ⌘F/⌘A/F12/⌘⇧O… resolve there),
  else Terminal as today. Clicking a pane focuses it (`PaneGrid.focused` — exists); the caret/selection
  render gates on pane-focus exactly as the cursor dim already gates on window-focus (#220's axis lesson:
  check BOTH the time axis and the multi-pane axis).
- **Geometry + IME follow focus (single-slot fact #4):** `editor_geom` (one Cell) + `handle_input`
  registration are written ONLY by the focused editable surface's frame — the unfocused editor renders
  without registering. One focused text-input surface at a time is exactly macOS's model; the slot stays
  single ON PURPOSE, its writer chosen by focus. Click→caret math inside the pane reuses the shipped
  layout seams (the #250 offset↔column AD governs).
- **Save + dirty:** ⌘S saves the FOCUSED editable surface's file (the accessor); the pane shows the ●
  dirty marker (the tab idiom); close-pane-with-dirty prompts like close-tab does. `did_save`/git-marks
  bumps fire identically.
- **THE SAME-PATH FORK, pinned honestly (D-OWN-BUFFER-V1):** the same file open in the editor TAB and an
  editable PANE = **two independent Buffers** (edits do not mirror live). This is v1-acceptable because
  the SHIPPED #275 external-change machinery is the net: saving either copy trips the other's disk-stat
  conflict banner (Keep mine / Reload), so silent last-write-wins clobbering CANNOT happen — the conflict
  is surfaced, by machinery that already has tests. TRUE shared-buffer multi-view (one Buffer, N views)
  requires evicting `SelectionSet` from `Buffer` (per-view cursors) — a real refactor, NAMED as the
  follow-up, out of v1. **Promotion re-takes this fork explicitly** (it is the spec's most contestable
  call; if chad wants live mirroring, the ticket grows a precursor).
### Out (explicitly)
- Shared-buffer live mirroring (the named follow-up above). Splits INSIDE the editor tab (this ticket
  edits the terminal-tab pane; an editor-tab split grid is its own future). Folding/inlay/LSP overlays in
  the pane (v1 = core editing: type/save/caret/selection/clipboard/motion/undo; the overlay features stay
  editor-tab-only and each names its pane-extension later). Drag-to-rearrange panes. Per-pane find bars.

## Reference (§20)
N/A — Marley/IDE-specific pane container (the ticket's own prior call, kept): the editing FEEL inherits
the shipped #249-257 editor stack wholesale; no external behavior is being matched beyond "IDEs let you
edit in splits," which is not a copyable mechanism.

### Prior art
1. **Behavior maps / observed** — one focused text-input surface at a time (the macOS model) is the
   pattern the geometry decision leans on; Zed/VS Code panes confirm focus-follows-click, nothing deeper
   taken.
2. **Published material** — none needed.
3. **OUR OWN CODE — the sweep IS the spec:** the #258 plan already measured the 16-site blast radius (the
   ticket body carries the line list — re-verify at promotion, don't re-count blind); the recon named the
   four single-slot assumptions with file:line; the #275 conflict machinery is the same-path net (shipped,
   tested); the OpenFile shape (editor_surface.rs:32) is the state to reuse; `PaneGrid.focused` already
   models pane focus. Nothing external owns any of this — the work is genuinely ours, which is exactly why
   the fork gets pinned rather than borrowed.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-ONE-ACCESSOR** — `focused_editable_surface()` is the ONLY new dispatch concept; no site may keep a
  private `active_tab().editor()` editing path (grep-gated at the end).
- **D-OWN-BUFFER-V1 + THE-275-NET** — independent buffers; conflicts surfaced by shipped machinery; live
  mirroring = the named follow-up. (THE FORK — promotion re-takes it.)
- **D-FOCUS-OWNS-THE-SLOTS** — KeyContext, editor_geom, and handle_input all keyed by the focused
  editable surface; the slots stay single deliberately.
- **D-CORE-EDITING-ONLY-V1** — the overlay/LSP feature set stays tab-side; each extends later.
- **D-PERSISTENCE-UNCHANGED** — the #258 `c=<path>` codec is untouched (path in, buffer loaded on boot).

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | route typing (incl. IME), backspace, Enter, and motion to the FOCUSED editable pane | headless |
| REQ-002 | keep the terminal pane's keys byte-identical while an editable pane exists but is UNFOCUSED | headless regression |
| REQ-003 | publish Editor key-context when the focused pane is an editable CodeView (⌘F/⌘A/F12 resolve there), Terminal otherwise | keymap/headless |
| REQ-004 | save the focused pane's file on ⌘S, with the dirty ● and close-dirty prompt | headless |
| REQ-005 | undo/redo the pane's buffer independently of the tab editor's | headless |
| REQ-006 | register text-input geometry for exactly the focused editable surface (click→caret correct in BOTH surfaces, whichever is focused) | headless state + deferred pixel |
| REQ-007 | surface the #275 conflict banner when the same path is edited in both surfaces and one saves (no silent clobber) | headless — the fork's net |
| REQ-008 | restore an editable pane from the #258 grid codec with its file loaded (persistence unchanged) | headless |
| REQ-009 | leave NO editing site reading active_tab().editor() directly (the accessor grep-gate) | unit/grep-gate |

## Phase Plan
P2 re-verify the 16-site list against live code (numbers drifted since #258) + design the accessor's
exact type (borrow shape: &mut over two homes is the Rust puzzle — an enum of &mut, or a closure-passing
seam) + the pane-state struct reuse vs parallel; P3 the pane state + accessor first, then the rewire in
site groups (typing → save → clipboard/motion → render/caret), each group compiling green before the
next; P3.5 critics on the focus-flip races (geom written by a stale frame), the same-path conflict flow
end-to-end, the KeyContext flip, the grep-gate; P4 drives per REQ + gate; P5 docs (editor.md's
architecture section — "the editor is no longer a singleton" is a real doc change). **Size note:** this
is plausibly a 2-slice ticket (accessor+typing+save = slice 1; clipboard/motion/undo polish = slice 2) —
promotion decides with the re-verified site count in hand.
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
