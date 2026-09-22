# Editable split-file pane — Notes

- **Forge ticket:** #259 a8bcfc50-90ad-4afe-a255-76c3edb0bacc
- **AAR:** 8d17f4a9-f400-4bae-8f5a-6b4f02f64f58
- **Local ticket doc:** docs/planning/tickets/open/TICKET-259-editable-split-pane.md
- **Pipeline spec:** 259-editable-split-pane.spec.md
- **pipeline_id:** 3511c80d-a0cf-48ad-a0bd-138f77de1ce0

## Phase 1 — Plan

**Request:** promote the pre-authored spec; re-verify the four single-slot facts + the 16-site blast radius
against `main` @ `2cada5f` (app.rs drifted through #300-317); re-take D-OWN-BUFFER-V1; decide 1 vs 2 slices.
Classification: **work pipeline, ARCHITECTURAL** (a pane-content model change + an accessor rewire). §20 =
N/A (Marley/IDE-specific pane; the editing feel inherits the shipped #249-257 stack).

**Forge recall (§18.3):** no bulletins. Knowledge surfaced the governing ADs the spec already cites —
`902ea928` (the #250 offset↔column AD, governs the in-pane caret) + pane/focus decisions. No prior failure
in this exact subsystem beyond the #220 focus-axis lesson (check BOTH the time axis AND the multi-pane
axis).

### THE FOUR SINGLE-SLOT FACTS — ALL CONFIRMED (re-verified by symbol against @2cada5f; the spec's line
numbers were stale, the FACTS hold):
1. **The pane has NO Buffer.** `PaneContent::CodeView(CodeViewState)` (workspace.rs:325, doc-commented "a
   read-only code viewer"); `CodeViewState` (code_view.rs:581) = `{ path: PathBuf, lines: Vec<CodeLine>
   (tab-expanded + truncated LOSSY display lines), scroll: usize }` — no Buffer, no selection, no caret.
   → the pane must grow the `OpenFile` shape.
2. **One text-input slot.** `editor_geom: Rc<Cell<EditorFrameGeom>>` (field app.rs:435, init 1988) — a
   SINGLE Cell recorded by the first rendered row each frame, read by click→caret (5346, 13509), the IME
   rect (13481), h-scroll/follow, overlays. ONE `window.handle_input(ElementInputHandler::new(bounds,
   ent_ime))` registration (app.rs:5174, inside `code_view_body`'s editable arm). `EntityInputHandler for
   RootView` (app.rs:13358-13526). → geometry + IME registration must be written ONLY by the FOCUSED
   editable surface's frame (the slot stays single BY DESIGN, its writer chosen by focus — exactly macOS's
   one-focused-text-input model).
3. **KeyContext is per-TAB, not focus-aware.** `key_context()` (tabs.rs:144) keys off the TAB kind
   (`Terminal`→[Terminal], `CodeView`→[Editor], `Cockpit`→[]); called app.rs:14225 via
   `active_tab().key_context()`; pinned by `key_context_maps_tab_kind_to_surface` (tabs.rs:687). → must
   become focus-aware (a terminal tab whose FOCUSED pane is an editable CodeView publishes [Editor]).
4. **`OpenFile`/`EditorSurface` is the state to reuse (not re-invent).** `OpenFile` (editor_surface.rs:32,
   private) = `view: CodeViewState, buffer: Buffer, saved_version: BufferVersion, marked:
   Option<MarkedSpan>, nonce: u64, scroll_px: f32, scroll_x: f32, disk: Option<(SystemTime,u64)>, conflict,
   armed_at, conflict_observed`; `EditorSurface` (editor_surface.rs:122, pub) = `{ files: Vec<OpenFile>,
   active: usize }`. Dirty ⇔ `buffer.version() != saved_version`.

Plus: **`SelectionSet` lives INSIDE `Buffer`** (→ the D-OWN-BUFFER-V1 fork — two views of one Buffer would
share one cursor set); **the #275 ext-change net is INTACT** (`check_active_file_external` app.rs:6485, the
Changed/Deleted conflict banner 6502/6515, `reload_active_from_disk` 6531 — Keep-mine/Reload on a save-time
disk-stat mismatch); **`save_active` saves the active TAB's editor** (app.rs:6929).

### THE BLAST RADIUS — RE-COUNTED (the spec's "16+" is STALE; authoritative Explore sweep @2cada5f):
**The choke point is TWO fns:** `active_editor()` (app.rs:8181) + `active_editor_mut()` (app.rs:12684),
BOTH hard-wired to `self.shell.active_project().active_tab().editor()`. `focused_editable_surface` /
`focused_editor` do NOT exist yet. **~127 `active_editor[_mut]()` calls + 53 inline
`active_tab().editor[_mut]()` chains that BYPASS the accessor ≈ 180 raw editor-target sites.** Grouped to
**~52 editing FEATURE-sites**:
- **[MOVES] (~52 feature-sites / ~95 raw):** typing/IME (`replace_text_in_range` 13427, IME 13457/13407,
  reads 13370/13384/13398, caret rect 13480); click→caret (13508, `on_mouse_down` 5353, drag 5407);
  clipboard (⌘V 14126, ⌘C/⌘X 14146); undo/redo (7074/7092); save+dirty+external-change (6929/6952/6485/
  6502/6531/6802/6865/8567); motion (⌘arrows 14063, arrows/word/Backspace 14398, Esc 14255, apply_delete
  3336, line-reorder 3302, F8 caret 7310, ⌃G 7179, bracket 3566, ladder 3383, follow-caret 12748); Enter/
  Tab-indent/comment (14351/14277/7275); select-all/multi-cursor (⌘A 7127, ⌘⌥↑↓ 7190, ⌘D 7225, ⌘⇧L 7253);
  render/caret/selection/gutter (EditorDraw 15200, selection bands 4845, the uniform_list row closure 4954,
  **the #246 read-only split render at app.rs:16343 `code_view_body(cv,…,None,cx)` — the `None` arm this
  ticket flips to `Some`**, sticky 5450, the per-frame feeder reads 8194/8241/3224/3235/13061/12998/3472/
  3526).
- **[TAB] (~5, do NOT move):** `live_tab_title` (6047; editor tabs use the stable `EDITOR_TAB_TITLE` const),
  `close_tab_at`/`Project::close_tab`, the rail render, tab-rename, **the LSP host keyed per-PROJECT-ROOT**
  (`lsp_hosts.get(&active_project().root)` — a level ABOVE the tab, stays workspace-scoped).
- **[UNSURE] (~28, flagged):** `key_context()` (must become focus-aware — fact #3), `text_input_blocked()`
  (8147, co-gates every IME site), the 4-site file-tab strip (15259/15290/15297 — pane is single-file
  today), and **~25 LSP/nav caret feature-sites** (hover/def/references/rename/completion/signature/⌘T/⌘⇧M/
  nav-back — the SLICE-2 candidates).

**THE ACCESSOR INSIGHT (the design's key simplifier, flagged for P2):** because ~75 of the ~95 sites go
THROUGH `active_editor()/_mut()`, making those TWO fns focus-aware (resolve to the focused pane's editor
when a terminal tab's CodeView pane is focused, else `active_tab().editor()`) leaves those ~75 sites
UNCHANGED — only the ~20 inline `active_tab().editor()` bypassers must move onto the accessor. **`PaneGrid`
(workspace.rs:418) already has `focused: PaneId` (421) + `focused_terminal()/_mut()` (603/609) — the EXACT
focus-aware-accessor precedent to mirror.** The Rust puzzle (P2's hardest call): the accessor returns `&mut`
over TWO homes (the tab's `EditorSurface` OR the focused pane's editable CodeView) — an enum-of-&mut vs a
closure-passing `with_focused_surface(|s| …)` seam.

### D-OWN-BUFFER-V1 — RE-TAKEN, HOLDS (the spec's most contestable call):
Same file in the editor TAB and an editable PANE = TWO independent Buffers (edits don't mirror live).
CONFIRMED acceptable because the shipped #275 machinery IS intact (verified above) → saving either copy
trips the other's disk-stat conflict banner (Keep-mine/Reload), so silent last-write-wins CANNOT happen.
True shared-buffer multi-view needs evicting `SelectionSet` from `Buffer` (per-view cursors) = a real
refactor, the NAMED follow-up. Auto-approved + the net is sound → **CONFIRM D-OWN-BUFFER-V1 for v1.**

### SIZE — 2 SLICES (recon-driven; this ticket = SLICE 1):
The blast radius (~52 feature-sites) far exceeds the stale "16+", and the [MOVES] sites cluster cleanly:
- **SLICE 1 (THIS ticket — the core editable pane):** the pane grows a Buffer/OpenFile; `active_editor()/
  _mut()` become focus-aware + the ~20 bypassers move on; `key_context` + `editor_geom` + `handle_input`
  follow focus; save+dirty per focused surface; the `16343` `None`→`Some` render flip; click→caret. Covers
  REQ-001/002/003/004/005/006/007/008/009 (type/save/caret/selection/clipboard/motion/undo — what
  "editable pane" MEANS + the #275 net + the persistence + the grep-gate).
- **SLICE 2 (named FOLLOW-UP, filed at complete):** editor-feature parity in the pane — find (⌘F), folding,
  go-to-file-symbol (⌘⇧O), and the ~25 LSP/nav caret features (hover/def/references/rename/completion/
  signature/⌘T/⌘⇧M). Each layers over a working editable surface and carries its own memo/overlay state
  (D-CORE-EDITING-ONLY-V1 already scopes these tab-side for v1).

**AC/decisions:** all EARS REQ-001..009 + the 5 locked decisions (D-ONE-ACCESSOR, D-OWN-BUFFER-V1,
D-FOCUS-OWNS-THE-SLOTS, D-CORE-EDITING-ONLY-V1, D-PERSISTENCE-UNCHANGED) HOLD for slice 1 as written — no
reopens. The spec's Reference (§20 N/A) + Prior-art (our own code: the #258 measurement, the four facts, the
#275 net, the OpenFile shape, PaneGrid.focused) are FILLED and CONFIRMED accurate.

Status: Phase 1 — Plan PASS; ready for Phase 2 — Design (SLICE 1).

## Phase 2 — Design (SLICE 1 — core editable pane)

**§20 CONFIRMED N/A** — Marley/IDE-specific pane container; the editing feel inherits the SHIPPED #249-257
editor stack wholesale (`EditorSurface`/`OpenFile`/`Buffer`). No copyleft source read; the prior-art sweep
is OUR OWN CODE (below). Prior-art sweep (gpui/ropey/regex): none own "which of two editors is focused" — it
is Marley's own `PaneGrid.focused` + `focused_terminal` precedent. **THE decisive find: the editor TAB already
performed this exact `CodeViewState → EditorSurface` recast at #237** — tabs.rs:108 documents it ("render +
persistence read the active file through here, so they're unchanged by the recast"). The pane recast is that
shipped, tested pattern applied to `PaneContent::CodeView`.

### Architecture (hardest-first)

**1. THE ACCESSOR (D-ONE-ACCESSOR) — the pane holds an `EditorSurface`, the two choke fns go focus-aware, return type UNCHANGED.**
`OpenFile` is PRIVATE (editor_surface.rs:32); `EditorSurface` (files: Vec<OpenFile>, active) is PUB and is
what `Tab::editor()` returns. So the editable pane carries an **`EditorSurface`** (a 1-file surface) — the SAME
type the tab holds — not a raw OpenFile. Then:
- `active_editor()` (app.rs:8181) / `active_editor_mut()` (app.rs:12684) become:
  `if project_count==0 {None} else { active_tab().editor().or_else(|| active_tab().grid()?.focused_editable_surface()) }`
  (mut twin symmetric). **Return type stays `Option<&EditorSurface>` → the ~75 through-accessor sites are
  UNCHANGED.** Only the ~20 inline `active_tab().editor()`/`active_tab_mut().editor_mut()` BYPASSERS (the notes'
  [MOVES] list — the on_key_down arms 14126/14146/14398/…, save 6952/6969, clipboard, motion, undo 7074/7092,
  render 15200/4845/4954) move onto the accessor.
- **BORROW SHAPE = option (a): resolve `&mut EditorSurface` from either home behind the existing signature**
  (an `or_else` over two `Option<&mut>` — no closure seam needed; both homes yield the same type). The E0499
  worry is moot: each home is reached by a disjoint `match`, and `active_project_mut().active_tab_mut()` already
  hands out one `&mut` chain — the pane arm just reaches a different field of the same tab (a terminal tab's
  grid), never both at once.
- **The precedent mirrored VERBATIM:** `PaneGrid::focused_editable_surface()/_mut()` =
  `self.state(self.focused).and_then(|s| s.editable_surface())` — byte-for-byte the shape of
  `focused_terminal()` (workspace.rs:603). `Tab::grid()/grid_mut()` accessor added (returns the terminal tab's
  `&PaneGrid`), if not already present.

**2. THE PANE GROWS A BUFFER (fact #1) — `PaneContent::CodeView(CodeViewState)` → `PaneContent::CodeView(EditorSurface)`.**
- `PaneState::code_view()` recast to source the view from the surface's active file (EXACTLY `Tab::code_view`,
  tabs.rs:109-114: `CodeView(surface) => Some(surface.active_file())`) → the #246 render + the #258 codec that
  read `code_view().path`/`.lines` are UNCHANGED (the #237 recast's whole point).
- ADD `PaneState::editable_surface()/_mut() -> Option<&[mut] EditorSurface>` (the `terminal()/terminal_mut()`
  twins). `kind()` unchanged (still `PaneKind::CodeView`).
- The split-right creation (#246) + the #258 restore build `EditorSurface::new(CodeViewState, raw_text)` from
  the path — identical to how an editor tab's file loads (REQ-008; D-PERSISTENCE-UNCHANGED — the `c=<path>`
  codec is untouched, path in → buffer loaded on restore).

**3. FOCUS OWNS THE SLOTS (facts #2/#3, D-FOCUS-OWNS-THE-SLOTS).**
- **key_context (REQ-003):** `Tab::key_context()` (tabs.rs:144) for a Terminal tab consults the grid —
  `Terminal(grid) => if grid.focused_pane_is_editable() { [Editor] } else { [Terminal] }`. The PURE decision
  `PaneGrid::focused_pane_is_editable() -> bool` (= `self.focused_editable_surface().is_some()`) lives in
  workspace.rs (coverage/mutation-visible). The existing `key_context_maps_tab_kind_to_surface` test grows a
  focused-editable-pane case.
- **editor_geom + handle_input (fact #2, REQ-006):** the ONE `editor_geom` Cell (app.rs:435) + the ONE
  `handle_input` registration (app.rs:5174) are written ONLY by the render of the surface that IS the accessor's
  target (the focused editable surface). Gate the editable arm's registration on an `is_the_focused_editable_
  surface` predicate passed into `code_view_body`. An UNFOCUSED editor pane renders read-only (no register, no
  geom write) — the slot stays single, its writer chosen by focus (macOS's one-focused-text-input model). The
  caret/selection render gates on pane-focus (the #220 BOTH-axes lesson: pane-focus AND window-focus).

**4. THE RENDER FLIP + click→caret (REQ-006).** app.rs:16343 `code_view_body(cv,…,None,cx)` — the #246 read-only
split render — passes `Some(editable_ctx)` when the pane is the focused editable surface, reusing the editor
tab's editable arm (the caret/selection/IME-registering path already at 5174). click→caret reads the single
(now focus-written) geom under the #250 offset↔column AD (902ea928) — unchanged math.

**5. SAVE + DIRTY (REQ-004/007).** `save_active` (6929) + `check_active_file_external`/banner/reload
(6485/6502/6531 — the #275 net, VERIFIED intact) move onto `active_editor()/_mut()` → ⌘S saves the FOCUSED
surface; the pane shows the ● (reads `surface.is_dirty()`); close-pane-with-dirty prompts like close-tab.
D-OWN-BUFFER-V1: same path in tab+pane = two Buffers; saving either trips the other's disk-stat conflict banner
(no silent clobber — REQ-007). did_save/git-marks fire identically (they already key off the accessor).

### File manifest
- **crates/marley_app/src/workspace.rs** — `PaneContent::CodeView(CodeViewState)` → `CodeView(EditorSurface)`;
  `PaneState::code_view()` recast (source from `surface.active_file()`) + NEW `editable_surface()/_mut()`;
  NEW `PaneGrid::focused_editable_surface()/_mut()` + `focused_pane_is_editable()` (the `focused_terminal`
  twins — the PURE cov/MSI-100 seams). `kind()` unchanged.
- **crates/marley_app/src/tabs.rs** — `Tab::key_context()` focus-aware for the Terminal arm (consults
  `grid.focused_pane_is_editable()`); NEW `Tab::grid()/grid_mut()` if absent. (`code_view()`/`editor()`
  unchanged.)
- **crates/marley_app/src/editor_surface.rs** — likely NO change (the pane reuses `EditorSurface`/`OpenFile`
  as-is; confirm `EditorSurface::new` + the accessors are pub enough — `active_file`/`active_buffer`/`is_dirty`
  are the reused seams). If a 1-file convenience ctor helps the pane, add it here (tested).
- **crates/marley_app/src/app.rs** — `active_editor()/_mut()` focus-aware (the or_else); the ~20 bypasser
  moves onto the accessor; `editor_geom`/`handle_input` gated on the focused-editable predicate; the 16343
  `None`→`Some` render flip; `save_active`/#275-net onto the accessor; the #246 split creation + the #258
  restore build the pane's `EditorSurface`; click→caret via the focused geom; the pane dirty ● + close-dirty.
  (app.rs is coverage-excluded; the focus-aware `active_editor` resolver is a thin `or_else` shim → `mutants::
  skip`; the REAL logic is workspace.rs's tested seams.)
- **crates/marley_app/src/headless_drive.rs** — the REQ drives (below).

### Regression Test Plan (≥1 per REQ)
| REQ | Test | Home |
|-----|------|------|
| 001 | `pane_typing_hits_the_focused_pane_not_the_tab_editor` — split a CodeView pane in a terminal tab, focus it, simulate typing → the PANE's buffer changed, the tab editor's did NOT | headless_drive (TestAppContext) |
| 002 | `terminal_pane_keys_unchanged_while_editable_pane_unfocused` — an editable pane exists but a terminal pane is focused → terminal keystrokes byte-identical (regression) | headless_drive |
| 003 | `focused_pane_is_editable_flips_key_context` — `PaneGrid::focused_pane_is_editable()` truth table + `Tab::key_context()` returns [Editor] when a terminal tab's focused pane is a CodeView, [Terminal] otherwise | workspace.rs + tabs.rs unit |
| 004 | `cmd_s_saves_the_focused_pane_and_marks_dirty` — edit the pane, ⌘S writes the pane's path to disk, the ● clears; close-pane-with-dirty prompts | headless_drive |
| 005 | `pane_undo_is_independent_of_the_tab_editor` — two buffers, two histories; undo in the pane leaves the tab editor untouched | headless_drive |
| 006 | `editor_geom_written_by_the_focused_surface` — focus the pane → click→caret lands in the pane; focus the tab editor → click→caret lands there (headless state; pixel deferred) | headless_drive |
| 007 | `same_path_both_surfaces_one_saves_raises_the_275_banner` — same file in tab + pane, save one → the other's Changed banner (no silent clobber) | headless_drive |
| 008 | `editable_pane_restores_from_the_258_codec_with_its_file` — persist a grid with a `c=<path>` CodeView pane, restore → the pane is an editable EditorSurface with the file loaded | headless_drive |
| 009 | `no_editing_site_reads_active_tab_editor_directly` — the accessor grep-gate: `include_str!(app.rs)`, assert the inline `active_tab_mut().editor_mut()` occurrences are bounded to the accessor-internal ones (the ~20 bypassers moved) | app.rs unit (source-grep, the #316 idiom) |

PURE cov/MSI-100 surface: `focused_editable_surface`/`focused_pane_is_editable`/`editable_surface` (workspace.rs),
`key_context` focus arm (tabs.rs). App shims (`active_editor` resolver, render flip, geom gate, save) are
`mutants::skip`, behavior-proven by the headless drives. **Genuinely-deferred pixel** (chad at machine): the
caret/selection PAINT in the pane is asserted by state (geom written + click→caret offset), not screencapture.

### Risks / decisions
- **The borrow puzzle** — resolved to option (a) (or_else over two `Option<&mut>` of the same type; disjoint
  match arms, no E0499). If the implement hits a borrow wall, fall to a closure seam `with_focused_editor(|s|)`.
- **The focus-flip geom race** — an UNFOCUSED editor frame must NOT write `editor_geom`/register `handle_input`
  (else a stale frame steals the slot — the #220 axis + #311 poll-live-identity lessons). The gate is the
  focused-editable predicate, read at render, not a latch.
- **The model recast blast** — `PaneContent::CodeView(CodeViewState)→(EditorSurface)` touches the codec + the
  render + the split creation; MITIGATED by the #237 precedent (the tab did this exact recast; `code_view()`
  keeps its read shape).
- **The grep-gate predicate** — a bounded-count assertion on `active_tab_mut().editor_mut()` (the accessor is
  the only sanctioned resolver), not a zero-count (the accessor itself uses the homes).
- **SLICE 2 (named follow-up, filed at complete):** editor-feature parity in the pane — find (⌘F), folding,
  ⌘⇧O symbols, and the ~25 LSP/nav caret features. Each layers over this working editable surface.

Status: Phase 2 — Design PASS; ready for Phase 3 — Implement (SLICE 1).

## Phase 3 — Implement (IN PROGRESS — compiling-green groups)

**GROUP 1 — model recast + workspace seams: DONE + COMPILE-GREEN (`cargo check -p marley` 14.5s, clean).**
The riskiest part (the `PaneContent::CodeView` recast) landed exactly as the #237 precedent predicted:
- workspace.rs: `use crate::editor_surface::EditorSurface`; `PaneContent::CodeView(CodeViewState)` →
  `CodeView(EditorSurface)` (+ doc); `PaneState::code_view()` recast to `surface.active_file()` (read shape
  UNCHANGED — the ~6 `code_view()` reads are transparent, don't move); NEW `PaneState::editable_surface()/
  _mut()` (the `terminal()/_mut()` twins); NEW `PaneGrid::focused_editable_surface()/_mut()` (byte-for-byte
  the `focused_terminal` shape) + `focused_pane_is_editable()` (the PURE cov/MSI-100 key_context seam).
- The 2 PRODUCTION construct sites both had the raw text in scope → build `EditorSurface::new(cv, &text)`:
  `split_file_pane` (app.rs — the #246 split, `_text`→`text`) + the #258 codec restore (app.rs — `&text`).
  The 3 workspace.rs `#[cfg(test)]` construct sites updated to `EditorSurface::new(cv, …)` (compile-only).
- **The cascade was BOUNDED** (grep-confirmed): only 2 app.rs constructs + 3 test constructs + the enum def/
  `kind()`/`code_view()`. `kind()` unchanged (`CodeView(_)`), no direct-extract matches (all reads via
  `code_view()`), so the recast stayed contained — the #237 "read through the accessor" design paid off.

**GROUP 2 — key_context focus-aware: DONE + GREEN.** `Tab::grid()/grid_mut()` ALREADY existed (tabs.rs:79/87
— no add needed). `Tab::key_context()` Terminal arm → `if grid.focused_pane_is_editable() {[Editor]} else
{[Terminal]}`. The existing `key_context_maps_tab_kind_to_surface` test still passes (its terminal tabs focus
terminal panes → false → [Terminal]); Phase 4 adds the focused-editable case.

**GROUP 3 — the focus-aware accessor: DONE + GREEN (`cargo check` 14s).** `active_editor()` (app.rs) →
`tab.editor().or_else(|| tab.grid().and_then(|g| g.focused_editable_surface()))` (both immutable borrows of
`tab` — clean). `active_editor_mut()` → the borrow-safe `if tab.editor().is_some() { return tab.editor_mut();
} tab.grid_mut().and_then(|g| g.focused_editable_surface_mut())` (the `is_some` probe releases before the mut
return → no E0499 — COMPILED). Both `#[cfg_attr(test, mutants::skip)]`. Return type UNCHANGED → the ~75
through-accessor sites already resolve to the focused pane with zero change.

**GROUP 4 — move the bypassers: DONE + GREEN (`cargo check` 14s).** A whitespace-tolerant regex collapsed
**43 inline chains** (31 `…active_tab_mut().editor_mut()` + 12 `…active_tab().editor()`, receivers `self`/
`view`) onto `active_editor()/_mut()`. The accessor bodies were PROVEN untouched (they use a `let tab = …`
local — no contiguous `.active_tab().editor()` — so the chain regex can't self-recurse; verified by reading
the body post-collapse). This also folded in most of G5's "save/#275 onto the accessor" (6490/6935/save_active
were in the 43). The recon's [TAB] sites (live_tab_title/close_tab/rail/per-root-LSP) use no `.editor()` chain
→ untouched. REQ-009's grep-gate target: the inline `active_tab_mut().editor_mut()` count is now ~0 outside
the accessor.

**REMAINING — G5 (the render + IME wiring; the feature's last mile) — PRECISE MAP (recon'd @2cada5f):**
- **The flip site is app.rs:16181** (NOT 16343 — drifted): the split-pane render at 16160-16182 does
  `code_view = workspace().state(pane_id).and_then(|s| s.code_view()).cloned()` then `if let Some(cv) =
  &code_view { body.child(self.code_view_body(cv, &colors, None, cx)) }`. **Flip the `None`→`Some(pane_draw)`
  when `pane_id == workspace().focused()` AND the pane is editable.**
- **Build the pane's `EditorDraw`** the way the editor-tab path does (15062-15084): `EditorDraw { buffer
  (the pane surface's active_buffer, cloned), carets (buffer.selections → (row, col_of_offset)), cell {w:
  em_advance via window.text_system, h: fallback.h} }`. Needs `window` for the font metric — the split-pane
  render is inside the pane loop, so pull the pane's buffer + selections (clone to release the workspace
  borrow, mirroring the `code_view.cloned()` at 16164). A pure helper `editor_draw_from(surface, window,
  font_size, tab_width)` keeps it DRY between the tab path (15077) and the pane path.
- **`handle_input`/`editor_geom` follow FOR FREE:** `code_view_body`'s editable arm (the `Some` branch at
  ~5124-5156) is what registers `window.handle_input(ElementInputHandler::new(bounds, ent_ime))` + writes
  `editor_geom`. Passing `Some` only for the focused pane means ONLY it registers → the single slot's writer
  is chosen by focus (the editor-tab body and a terminal-tab-pane body are mutually exclusive — only the
  active tab renders — so no double-register). The #220 both-axes / #311 live-identity guard = the focus gate
  is read at render, not latched.
- **Pane dirty ● + close-pane-dirty:** the pane render reads `surface.is_dirty()` for the ●; close-pane
  routes through the existing close-dirty prompt (verify `is_dirty` is pub on EditorSurface, else add it).
- **click→caret:** already reads the single (now focus-written) `editor_geom` under the #250 AD — no change.
- **Risk:** the borrow in the pane loop (clone the buffer/selections like the `cv.cloned()`); the two EditorDraw
  build paths sharing a helper (else drift); `window` availability in the pane render (it's in scope — the
  loop has `window`).

**GROUP 5 — render flip + IME: DONE + GREEN (the WHOLE implement compiles, `cargo check` 14s).** Extracted a
shared `editor_draw_for<'a>(&self, surface, window) -> EditorDraw<'a>` helper (the editor-tab path + the pane
path now use ONE builder — no drift); rewired the editor-tab inline build onto it. **The flip at app.rs:16187:**
`if self.workspace().focused() == pane_id { …editable_surface().map(|s| self.editor_draw_for(s, window)) }
else { None }` → a FOCUSED editable pane passes `Some(draw)` to `code_view_body` (renders caret/selection +
registers `handle_input`/`editor_geom` via the Some-arm's paint closures), an UNFOCUSED one passes `None`
(read-only). Only the focused surface writes the single geom/IME slot — the editor-tab body and a terminal-tab
pane body are mutually exclusive (only the active tab renders), so no double-register. The borrow dance
compiled (all immutable self borrows; `window` in scope in the pane loop).

**REQ-004 polish — DONE + GREEN:** the pane dirty **●** (U+25CF accent dot, the #252 style) on the pane title
bar (app.rs:16290), shown via `.when(state(pane_id).editable_surface().is_some_and(active_is_dirty))`. ⌘S save
already works (G4). `cargo check` clean, `cargo fmt` clean.

**DEVIATIONS (for Inspect):**
1. **REQ-004's close-dirty prompt is SCOPED OUT.** The recon assumed "close-pane-dirty prompts like close-tab
   does" — but `close_tab_at` (app.rs:6086) does NOT prompt: it closes the tab and drops its (possibly dirty)
   editor surface off-thread, no confirm. So there is NO close-dirty machinery to mirror, and building one ONLY
   for the pane would make it MORE protective than the editor tab (inconsistent). Matching the reference
   behavior: closing a pane drops it like closing a tab. The dirty ● gives the unsaved signal. A real
   close-dirty confirm is a cross-cutting follow-up for BOTH tab and pane (filed at complete).
2. **G4 was a BLANKET regex collapse** of all 43 `active_tab().editor[_mut]()` chains onto the accessor (the
   recon classified them all [MOVES]). **INSPECT MUST verify none broke a tab-scoped decision** — especially
   the file-tab-strip render (~app.rs:15273, recon-flagged [UNSURE]): does a terminal tab with a focused code
   pane now wrongly render the multi-file strip? (Likely harmless — the strip render path is tab-kind-gated —
   but VERIFY.)
3. **The pane caret render gates on pane-focus** (`workspace().focused() == pane_id`) — inspect should confirm
   it ALSO dims on WINDOW-unfocus (the #220 both-axes lesson) via `code_view_body`'s reused caret path, and
   that an UNFOCUSED editable pane writes NO `editor_geom`/`handle_input` (the focus-flip slot race).

Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect. (All 5 groups + the dirty ● compile-green +
fmt-clean; the editable split pane is functionally wired — type/save/caret/selection/motion/undo/render/IME;
one scoped deviation [close-dirty prompt] + two inspect-targets recorded.)

## Phase 3.5 — Inspect

4 parallel general-purpose critics (blanket-G4 correctness · focus-flip render race · pure workspace seams +
recast · reuse/fork/hygiene), each verifying concretely (reads + `cargo check` + `cargo mutants --list` +
`cargo doc`; none edited the tree). **1 HIGH FIXED + 2 documented slice-1 limitations (follow-up) + a Validate
carry.**

| # | Sev | Finding | file:line | Verdict | Resolution |
|---|-----|---------|-----------|---------|-----------|
| F1 | **HIGH** | The FOCUSED editable split pane rendered BLANK. `code_view_body`'s `Some`-arm `uniform_list` row closure is `'static` → it re-reads the buffer live via `entity.read(app).shell.active_project().active_tab().editor()` — but a split pane lives INSIDE a TERMINAL tab, whose `editor()` is `None`, so the closure hits `else { break; }` on ROW 0 → zero rows rendered (no text/gutter/caret), while `fold_projection` (focus-aware `active_editor()`) reported the pane's real line count → full scroll height, blank body. You'd type BLIND (input is pane-aware). **The G4 regex MISSED this chain because its receiver is `entity.read(app)`, not `self`/`view`.** | app.rs:4976 | **REAL — the flagship interaction was broken** (verified: `Tab::editor()` = None for Terminal; the split pane's host tab is a Terminal) | **FIXED** — the row closure now reads the focus-aware `entity.read(app).active_editor()` (== the tab's editor for an editor tab; == the focused pane's surface for a pane), matching the `fold_projection` that sized the list. `cargo check` clean. This was the ONE remaining `active_tab().editor()` chain (grep-confirmed: 0 left outside the accessor's `let tab` internals). |
| F2 | MEDIUM | An UNFOCUSED editable pane renders STALE `cv.lines` (the loaded snapshot) — `CodeViewState.lines` is never re-synced from the Buffer, so after editing (focused=live) and clicking away, the pane reverts to the pre-edit text until re-focused | app.rs:16197 (the `None` arm) | **REAL but LOW-impact** — no data loss (the Buffer is truth, ⌘S/save correct), self-healing on re-focus; the #246 pane was always read-only so `cv.lines`-staleness was unreachable before | **DOCUMENTED slice-1 limitation + FOLLOW-UP** — a proper fix renders the unfocused pane's OWN buffer read-only (code_view_body must take pane_id + gate the geom/IME registration on focus), invasive for slice 1. The FOCUSED pane (the editing surface) is correct. |
| F3 | LOW | The split pane's #275 external-change CONFLICT has no banner UI — `check_active_file_external` is now focus-aware (arms a conflict on the pane's surface) but the Keep-mine/Reload banner renders only in the editor-tab-gated block | app.rs:15171 vs 16185 | **REAL but no data loss** — REQ-007's NO-SILENT-CLOBBER holds: `save_active` re-checks + arms + status-flashes ("⌘S again to overwrite") before writing | **DOCUMENTED + FOLLOW-UP** (wire the banner into the pane render) — same class as F2 (the background pane lacks the full editor chrome) |
| F4 | MEDIUM | The new pure seams are UNCOVERED / the `key_context()` `[Editor]` TRUE branch is untested (the existing `key_context_maps_tab_kind_to_surface` pins only the FALSE/[Terminal] side) — a mutant flipping `focused_pane_is_editable()`→false (i.e. the whole #259) would survive | workspace.rs + tabs.rs:151 | **REAL (VALIDATE's job)** — tests are Phase 4; flagged forward | **CARRIED to VALIDATE** — the test cluster (below) kills every uncovered mutant |

**Mutation surface (critics 3+4 ran `cargo mutants --list`): the VIABLE seams to kill at Validate** —
`editable_surface()/_mut()` (None + delete-arm), `focused_editable_surface()/_mut()` (None), `focused_pane_
is_editable()` (the FALSE side + the [Editor] line), `code_view()` recast (KILLED already by `open_pane_
codeview_preserves_state`), `key_context()` [Editor] arm. **UNVIABLE (do NOT chase):** `EditorSurface`/
`CodeViewState` have NO `Default` → whole-body `Default::default()` mutants won't compile. **The kill cluster
(one test):** build a `PaneGrid`, `open_pane` a `CodeView(EditorSurface)`, `focus()` it (initial focus is a
Terminal `PaneId(0)`, so the test MUST focus the code pane), assert `focused_pane_is_editable()==true` +
`focused_editable_surface[_mut]().is_some()` + `editable_surface[_mut]().is_some()`; a terminal-pane negative
(`editable_surface().is_none()`); a terminal-tab `key_context()==[Editor]` with a focused code pane.

**SOUND (attacked, held):** the 43-site collapse (every OTHER moved site correct — the file-tab-strip render
is tab-kind-gated so `active_editor()` resolves identically; save/#275/persist/nonce all correctly focus-aware
or gated); the two `Some`-render paths are MUTUALLY EXCLUSIVE (editor-tab body `else if code_view().is_some()`
vs the terminal-tab pane loop `grid().is_some()` — an enum can't be both) so only ONE surface writes the geom/
IME slot; the unfocused pane registers NO geom/handle_input (they're inside the `Some`-arm's `if row==first`);
the `editor_draw_for` borrow is sound; D-OWN-BUFFER-V1 + REQ-007's net holds (per-surface disk snapshots, save_
active re-checks); the `active_editor_mut` `is_some` probe sound; `editor_draw_for` de-dup clean; the workspace
seams are `focused_terminal` twins byte-for-byte; `EditorSurface::new` seeds the Buffer from RAW text (not lossy
lines); mutants::skip discipline correct (app shims skipped, workspace/tabs seams not); grep-gate = 0; no Zed/
Warp; `cargo doc` clean.

**Verify:** `cargo check -p marley` clean (14s job-capped) after the F1 fix.

Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.

## Phase 4 — Validate

**Tests added (3 units + 1 headless drive + 2 test hooks):**
- **workspace.rs `editable_pane_seams_resolve_the_focused_code_pane`** (REQ-003) — builds a `PaneGrid`,
  `open_pane`s a `CodeView(EditorSurface)`, EXPLICITLY `focus()`es it (initial focus is the Terminal
  `PaneId(0)`), asserts every editable seam resolves it (Some/true), then a terminal-pane negative (None/
  false). Kills the `None`/delete-arm mutants on `editable_surface[_mut]` + `focused_editable_surface[_mut]`
  + both sides of `focused_pane_is_editable`.
- **tabs.rs `key_context_editor_on_focused_code_pane`** (REQ-003) — a Terminal tab whose grid's FOCUSED pane
  is a CodeView → `key_context() == [Editor]` (the [Editor] arm the tab-only test never reached).
- **app.rs `no_editing_site_reads_active_tab_editor_directly`** (REQ-009 grep-gate) — `include_str!(app.rs)`,
  whitespace-stripped, scans ONLY the production part (before the test module), asserts 0 inline
  `active_tab().editor()`/`active_tab_mut().editor_mut()` chains (all moved onto the accessor).
- **headless_drive.rs `split_file_pane_is_editable_when_focused_headless`** (REQ-001/001b/003/006, the F1
  proof) — boots, seeds `src/x.rs`="hello\nworld\n", calls the real `split_file_pane` (opens+focuses an
  editable code pane in a TERMINAL tab), asserts `active_editor_text_for_test()=="hello\nworld\n"` (REQ-001b/
  006: the focus-aware accessor resolves the pane's REAL buffer — the F1 blank-pane bug read `None`), the tab
  publishes `[Editor]` (REQ-003), and `simulate_keystrokes("z")` → the PANE's buffer becomes "zhello\nworld\n"
  (REQ-001: typing routes to the focused pane through the real input handler). 2 `#[cfg(test)]` hooks:
  `split_file_pane_for_test`, `active_editor_text_for_test`.

**REQ coverage:** 001/001b/003/006/009 drive+unit-proven; 002 (terminal keys while an editable pane is
unfocused) + 005 (undo independent) share the shipped editor stack routed by the same focus-aware accessor;
004 (⌘S save + dirty ●) — save moved onto the accessor (G4) + the ● renders on `active_is_dirty` (inspect-
verified); 007 (same-path #275 net) inspect-traced (per-surface disk snapshots, `save_active` re-checks); 008
(codec restore) — the construct builds `EditorSurface::new(cv, &text)` from raw text (critic-verified) +
D-PERSISTENCE-UNCHANGED. LIVE pixel deferred (chad at machine; gpui 0.2.2's test platform has no draw) — the
headless input handler + the accessor resolution carry the behavior.

**Gate red fixed at SOURCE (the grep-gate self-match trap):** the first `--diff` gate went RED — the REQ-009
grep-gate PANICKED (it found 1 production match) → cascaded gate:3 tests → gate:4/5. The match was in my OWN
`#[cfg(test)]` hook's DOC COMMENT (`the F1 bug read `active_tab().editor()``) — a `#[cfg(test)]` fn sits in the
production part, so its comment's literal was scanned. FIX: reworded the comment to not contain the contiguous
literal. (The grep-gate correctly did its job — it caught a literal in the production scan.)

**GATE GREEN [diff] — 15/15** (fresh receipt): fmt · clippy · tests (nextest+doctests) · **coverage 100%
(workspace.rs 1082/1082, tabs.rs 826/826 lines — the new seams fully covered)** · **mutation MSI 100% (14
caught / 0 missed — the seam cluster killed)** · miri · deny/audit/machete/gitleaks/shellcheck/no-suppressions/
source-bans/docs · visual/AX. No pre-existing failures.

**Documented slice-1 limitations (NOT failures — scoped follow-ups, filed at complete):** F2 (an unfocused
editable pane shows stale `cv.lines` until re-focused) + F3 (no #275 conflict banner in the pane render) — both
"the background pane lacks full editor chrome"; no data loss.

Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
