# 428-multibuffer-editing — pipeline notes

## Phase 1 — Plan (2026-08-14)

**Recall (§18.3).** Knowledge: AD-claude-edit-post-state-span-and-earned-undo-tags-001
(the N-cursor engine owns post-selection; every undo tag COMPUTED — the corruption class
was real, redo-visible only), AD-claude-354-saves-are-instance-addressed (the write tail
takes ContentId; background origin = strict consent — `save_editor_by_id` is the
multibuffer's save), F-claude-354 ×2 (save/reload epoch races), F-claude-427-b +
PR-claude-427-a (insert owns view one — governs the D5 pinning), PR-claude-buffer-dont-
drop-events-during-transient-states. Completed archive: 259 (the two-Buffer fork —
HISTORICAL, superseded), 397 (one-instance-many-views — the ruling contract), 275
(extchange machinery), 282 (grouped undo).

**Explore sweep (very thorough, post-#427 tree)** — the full seam map with file:line
cites is in this phase's transcript; headlines: the ONE insert mechanism is the
`EntityInputHandler` (`app.rs:17994/18050` → `ime.rs:120` → `buffer.rs` edit fns); a
multibuffer tab currently DEAD-ENDS at `active_editor_mut` (typing, ⌘Z, ⌘S all gate on
it); `Buffer` already ships anchors + a delta log (`anchor_at`/`resolve_anchor`/
`edits_since`, #269) with folds as the per-frame resolve-and-re-derive precedent;
`save_editor_by_id` + `extchange::external_action` is the exact background-consent save
shape; instances drop at zero views (the mb must pin views — cockpit precedent at
`content.rs:353-360`); `SelectionSet` lives IN `Buffer` with D-OPEN-SELECTION-HOME
parking (the mb caret must stay excerpt-addressed, D6). #427's snapshot-dead fields:
`ExcerptLine.text/.row/.spans`, `cum` — D2 turns them live for open instances.

**Prior-art sweep.** (1) Zed §6: write-through + composed anchors + cross-buffer
history — contract adopted, container (SumTree) not. (2) Published: none new (didSave
wired). (3) In-tree: EVERYTHING exists — insert chain, anchors/delta log, grouped undo,
instance-addressed saves + conflict net, pinned views, dirty flags. The ticket is
wiring + one live excerpt index. The ticket-doc's open fork (#397 vs #259) is settled
BY THE TREE: #397 superseded #259 (recorded in `editor_surface.rs:6-13` + the 397 spec).

**Classification**: work pipeline, L. One shippable slice: single-caret editing
(type/⌫/⌦/⏎) + live rebase + routed undo + N×save_editor_by_id + dirty headers + pinned
lifecycle. Multi-cursor, selection, replace-all, context expansion, unification: out.

Ticket promoted (BACKLOG row removed). Spec authored fresh against the post-#427 tree
per the shelf's no-pre-authoring method.

## Phase 2 — Design

### Architecture (§20 confirmed: Zed §6's CONTRACT — write-through, anchor-composed
positions, one undo story — adopted; no SumTree/MultiBufferSnapshot container; the
clean-room wall holds: deconstruction docs only)

**A. The live excerpt index — WINDOW ANCHORS (D-WINDOW-ANCHORS).** Each `FileExcerpts`
with a live target gains per-WINDOW anchor pairs in the target buffer:
`(start: Anchor{Bias::Left} at the window's first-line START offset, end:
Anchor{Bias::Right} at the offset one line PAST the last)`. Per frame a sync pass
resolves each pair → a line range → re-derives that window's `ExcerptLine`s from the
LIVE buffer text (rows, texts; the frozen match-row set marks the wash — spans stay
as-built, line-level truth). A newline typed inside a window GROWS it by a row; a
join shrinks it; edits at either edge GROW (the bias choices make this the arithmetic
default — pinned by the boundary unit table); a window deleted to empty DROPS from the
render (pinned). The rebuild is memoized on the per-file `(nonce, version)` vec (the
`fold_projection_inner` resolve-and-re-derive precedent exactly). Un-edited NOT-open
files keep #427 snapshot lines (zero cost at 500 files).

**B. Targets + lifecycle (D5 refined).** The model gains
`targets: HashMap<usize /*fi*/, ContentId>` + `windows: HashMap<usize, Vec<(Anchor,
Anchor)>>`. At materialize: for every result file ALREADY open (`find_open` hit) —
`acquire_view` (the pin; the cockpit's non-tab-holder precedent) + mint windows from
the built rows. At FIRST EDIT of an un-targeted file: `open_editor_instance` (the one
birth closure — resolve_open handles the raced-open case) → pin + mint windows, then
edit. Close (tab + project): release every target via `release_editor_views` (the
folds-scrub-returning editor release), THEN the mb id via `release_multibuffer_views`
(which asserts mb-kind-only — targets must NOT go through it).

**C. The caret is an ANCHOR (D-CARET-IS-AN-ANCHOR).** `caret: Option<(usize /*fi*/,
Anchor)>` in the target buffer (`Bias::Right`), resolved per frame → (row, col) →
display slot; re-anchored explicitly after every mb-originated edit at the post-edit
offset. Click places it: the body writes an `mb_geom` Cell at paint (the editor_geom
idiom) and the click maps (x, y) through the mono cell arithmetic. The caret never
writes the buffer's `SelectionSet` except transiently INSIDE an edit call — each edit
builds a single-caret set at the caret's resolved offset, calls the standing buffer fn,
and re-anchors from the returned selection (selection home-parking untouched).

**D. Input routing — four gate arms.**
1. `EntityInputHandler`: each method gains a multibuffer branch when the active tab is
   a mb with a caret — `replace_text_in_range` resolves (target buffer, `&mut
   model.marked`) and calls the SAME `replace_text_ctx` (auto-close + Rust probe
   included — no special-cased editing semantics); marked-range queries read
   `model.marked`. `text_input_blocked` needs no change (overlay roster only).
2. The key router's mb arm (`handle_multibuffer_key`) grows: Backspace/⌦ →
   `apply_editor_key_multi`-equivalent buffer fns on a transient single-caret set;
   Enter → NEWLINE at the caret (the router's indent-aware shape); ↑/↓ keep moving the
   selection (and carry the caret when placed); ⌘⏎ (platform) on a line = JUMP (the
   #427 Enter-jump relocates to ⌘⏎ — click now places the caret instead of jumping;
   the 427 drive updates accordingly, recorded as the deliberate behavior change).
3. `dispatch_action("undo"/"redo")`: an mb arm ahead of the `active_editor_mut` gate —
   journal-routed (E below).
4. `dispatch_action("save")`: an mb arm — for each dirty target, `save_editor_by_id
   (id, SaveCause::<the explicit variant>)` (background-consent semantics; the #275
   net is that fn's own arms — nothing new).

**E. The undo journal (D3 refined).** `journal: Vec<(ContentId, BufferVersion /*pre-
edit*/)>` with consecutive same-id dedupe (a typed run in one file = one entry;
in-buffer coalescing owns the rest); `redone: Vec<(ContentId, BufferVersion /*post*/)>`.
⌘Z: peek top → `buffer.undo()` on that target → pop when `version() <= pre` (a
multi-group run converges by repeated ⌘Z); push the inverse onto `redone`. ⌘⇧Z
mirrors. The journal ops are PURE fns in multibuffer.rs over injected version numbers
(unit-tested); the shim owns only the buffer calls. Classification per
AD-claude-edit-post-state-span: every mb edit uses the standing engine fns with
identity spans (caret at insert end) — no new tags asserted.

**F. Render.** Header band gains the dirty dot (target `is_dirty` read in the frame
closure); the caret paints as the editor's bar glyph at (slot, col) via mono cell
arithmetic; edited text re-renders through the standing notify + memo-key idiom (the
sync pass runs at render top beside `resync_editable_pane_views`).

### File manifest

**marley-web (FIRST):**
- `views/MultibufferView.tsx` — caret on click (line + col via char measure), editable
  excerpt lines (keydown: printable/backspace/enter) writing through the shared doc
  store, dirty dot on the header band, ⌘⏎ jump on the caret line.
- `pages/Workspace.tsx` / `App.tsx` — doc-store write-through (the file's editor view
  shows the same text live); dirty set surfaced to the rail/header.

**crates (the port):**
- `crates/marley_app/src/multibuffer.rs` — `caret`/`marked`/`targets`/`windows`/
  `journal`/`redone` fields (marley_editor types); pure: `mint_windows` (rows →
  half-open char-range pairs), `rebuild_file` (resolved ranges + live text + frozen
  match rows → `Vec<ExcerptLine>` + cum refresh), journal push/undo-step/redo-step
  ops over injected versions, caret slot/col helpers; the boundary unit table lives
  here.
- `crates/marley_app/src/app.rs` — materialize pins targets + mints windows;
  `sync_multibuffer_live` at render top (memo on versions); the four gate arms (D);
  first-edit birth; caret paint + `mb_geom` + click mapping in `multibuffer_body`;
  dirty header; close/project release extensions; jump relocated to ⌘⏎.
- `crates/marley_app/src/content.rs` — a targets-release helper (editor-kind ids out
  of the model) if `release_editor_views` needs a sibling; accessor growth as needed.
- `crates/editor/src/*` — expected ZERO changes (anchors/edit fns/undo suffice); any
  need discovered at implement is a recorded deviation.

### Regression Test Plan

| REQ | Test | Where |
|---|---|---|
| REQ-001 | Drive: materialize with the fixture file OPEN + dirty-able → click/place caret → simulate typing → the file's editor-tab buffer contains the insert at the excerpt offset; the mb line re-renders the same text | headless_drive |
| REQ-002 | Drives: backspace joins/deletes at caret; ⌦; Enter inserts newline and the window GROWS by a row; units for the transient single-caret set mapping | headless_drive + multibuffer.rs |
| REQ-003 | Unit table (boundary policy): edit above window shifts rows; insert at start-edge/end-edge grows; interior newline grows; join shrinks; window deleted-to-empty drops; caret anchor rebases (typed text lands caret after) | multibuffer.rs units + a drive assert on rendered rows |
| REQ-004 | Drive: edits in TWO files → ⌘Z reverts the last (cross-excerpt order), again reverts the first; ⌘⇧Z re-applies; both surfaces agree; journal units (dedupe, converge-on-multi-group, redo mirror) | headless_drive + units |
| REQ-005 | Drive: dirty target + external disk mutation → mb ⌘S holds that file (conflict set, text NOT written) while the clean-dirty sibling saves; no-external case saves + `is_dirty` clears | headless_drive |
| REQ-006 | Model/render: dirty dot appears on the edited file's header, clears after save; live capture at validate | drive assert + live capture |
| REQ-007 | Drive: view_count of an OPEN file's instance +1 at materialize; close tab → back to pre; a first-edit-born instance's count 1 while tab lives → 0 at close; an EDITED buffer's undo history survives while the tab lives | headless_drive |
| REQ-008 | `scripts/gates.sh --diff` GREEN | gate |
| parity | POC ↔ live at the SAME state: caret visible in an excerpt, one edited line, dirty dot on the header — pixels sampled | validate |

Uncoverable: none new (app.rs stays the shim; every decision lives in multibuffer.rs
or the editor crate's already-tested fns).

### Risks / decisions (recorded)
- **The Enter/click semantic change to #427** (click places caret; jump moves to ⌘⏎)
  is deliberate and drive-updated — the editable surface's click MUST place a caret
  (Zed's own behavior); recorded so 429 builds on ⌘⏎-jump.
- **Window-anchor grow-at-edges** is the pinned boundary policy (bias arithmetic
  default); revisit only with a failing use-case.
- **Journal convergence** (pop when `version <= pre`) assumes undo strictly decreases
  toward the recorded pre-version per group — the delta-log ordering guarantees it;
  the unit table pins a multi-group run.
- **IME in excerpts** rides the same `replace_text_ctx` with `model.marked` — the
  marked-overlay RENDER inside mb rows is v1-minimal (composition text renders as
  buffer text; underline styling deferred, recorded).
- 500-file sets: only OPEN files pin at materialize; birth is per-edited-file — no
  mass instance creation.

## Phase 3 — Implement

**React-first (built + verified in pixels FIRST).** The POC MultibufferView became
editable: click places a caret (mono col from offsetX), printable/⌫/⏎ splice BOTH the
mb lines and the SHARED doc store (`docs` — the #397 stand-in), dirty dot on the
header band, ⌘S cleans, ⌘⏎ jumps, Esc drops the caret; ProjectSearch's corpus gained
the ONE doc-backed file (`code_syntax.rs`, lines verbatim from SEED_CONTENTS) so
write-through demos. tsc green. Captures READ: scratchpad/428-poc-edit.png (caret +
"XY" spliced + dirty dot) and 428-poc-writethrough.png (the editor tab shows the
same edited line + dirty chip) — write-through PROVEN in the browser before Rust.

**Rust (to the manifest).**
- `crates/editor` (recorded deviation from "expected zero"): `UndoHistory::depth()` +
  `Buffer::undo_depth()` — 3-line accessors the journal's one-entry-per-group dedupe
  measures. (A first splice put `undo_depth` between `undo`'s doc and fn — MY OWN
  PR-claude-427-b class, caught by the docs lint and relocated properly.)
- `multibuffer.rs`: `targets`/`windows`/`match_anchors`/`caret`/`journal`/`redone`
  fields (#269 anchors; `MintedAnchors` alias); pure `window_row_runs`,
  `run_char_range`, `rebuild_lines` (merge + empty-window drop), `journal_note_edit`
  (one entry per undo group — VERSIONS DON'T REWIND (the delta log appends through
  undo), so the design's version-convergence test was replaced by the depth-equality
  dedupe at implement, recorded), `refresh_cum`, `slot_of_row`.
- `app.rs`: `SaveCause::Sweep`; `mb_marked` + `mb_live_versions` fields; materialize
  pins (find_open → acquire_view → `mint_mb_anchors`); `sync_multibuffer_live` at
  render top (two-phase borrow: resolve+rebuild immutable, write+`refresh_cum`
  mutable; memo (nonce, version)); the input-handler mb arm (`mb_replace_text` — the
  SAME `replace_text_ctx`, transient single-caret set, `Code` context: the non-Rust
  editor path's exact behavior, tree probe stays editor-tab); key arms ←/→ (line-
  clamped), ⌫ (window-TOP-edge blocked — the pinned policy), ⌦ (bottom-edge
  symmetric), Esc (caret drop), Enter = newline WITH caret / jump WITHOUT (the #427
  drive-compatible split; ⌘⏎-from-caret deferred — Esc+Enter jumps); `mb_place_caret`
  (click → caret at LINE END v1 — precise column mapping deferred to the geom hook,
  recorded; ←/→ walk from there) + `ensure_mb_target` (lazy birth via
  `open_editor_instance`, mint against live text, release-on-race); `mb_undo`/
  `mb_redo` (journal-routed, stale entries drain, caret follows the edit site);
  `mb_save_all` (N × `save_editor_by_id(_, Sweep)`); caret bar (measured `em_advance`
  at the Command size) + header dirty dot in `multibuffer_body` (+ `window` param);
  close-tab + close-project release the pinned targets as EDITOR views (folds/marks
  scrub applies) before the mb id.

**Deviations from design (all recorded above):** editor-crate accessor pair; journal
dedupe by depth not version; click-caret at line end v1; no auto-indent on mb Enter;
IME composition methods stay editor-only (the commit text arrives via
`replace_text_in_range`; underline render deferred); ⌘⏎-jump-with-caret deferred
(Esc + Enter).

`cargo check --workspace --tests` clean; `cargo clippy -p marley -p marley_editor
--tests` zero warnings; fmt applied.

## Inspect (Phase 3.5)

Three critics over the uncommitted diff (edit-path/undo; anchor-lifecycle/save;
simplification/provenance/parity). Every confirmed finding fixed at source.

**Confirmed → fixed:**
1. **CRITICAL — typing was UNREACHABLE.** The OS/IME seam registers per-frame from
   the EDITOR body's paint-scoped canvas only; a multibuffer frame registered
   nothing, so `replace_text_in_range` never fired and `mb_replace_text` was dead
   code (raw-ladder keys worked; letters did nothing — headless drives calling the
   method directly would have masked it). Fix: the same zero-size-canvas
   `window.handle_input(ElementInputHandler)` registration in `multibuffer_body`.
2. **HIGH (2 critics) — the journal was unsound three ways.** (a) a type-over
   NO-OP (version unchanged, nothing recorded) still journaled an entry — mb ⌘Z
   could then pop it and drain a group the multibuffer never made; (b) the
   adjacent-only dedupe let a COALESCED continuation (same file, other-file entry
   in between) push a duplicate — the extra ⌘Z ate pre-mb history; (c) the pop
   never verified the stored depth — a tab-side edit/undo between made entries
   stale and an unguarded `undo()` reverted foreign work. Fixes: `mb_after_edit`
   takes the PRE-edit version and journals only on change; `journal_note_edit` is
   retain-then-push (one entry per (id, depth) group, moved to top on
   continuation); `mb_undo` pops then undoes ONLY when `undo_depth() == depth`
   (else drains); `mb_redo` mirrors with `depth - 1`.
3. **HIGH (2 critics) — the "transient" selection set wasn't.** The park/restore
   choke (`sync_editor_selection_home`) early-returns on mb frames (`last ==
   current`), so the mb's single-caret set REPLACED the user's cursors on return
   to the file's tab — and could corrupt the parked set via the twin-view park.
   Fix: `mb_edit_transient` saves the buffer's live set, runs the edit on the
   caret set, reads the post-edit head, and RESTORES the saved set (clamped; the
   undo group recorded its own restore — the AD's trap concerns the recorded set,
   not the live one).
4. **HIGH — the reload epoch hole (the F-claude-354 class).** `reload()` re-mints
   the nonce and resets the buffer; the stored anchors belonged to the dead epoch
   and `resolve_anchor` degrades them to clamped garbage — wrong rows, and a
   placed caret writing at meaningless offsets. Fix: `minted` (fi → nonce) stamps
   every mint; the frame sync RE-MINTS from the current excerpt rows against the
   fresh text on mismatch (the lazy-birth shape) and drops a caret pointing into
   the dead epoch.
5. **HIGH/MED (3 critics) — ⌘S swept UNTOUCHED dirty files.** `targets` holds
   every open result file; sweeping targets ∩ dirty saved files dirtied only in
   their own tabs (the #354 consent breach; the spec's own word is "touched").
   Fix: the model's `touched` set (inserted on real edits) — the sweep is touched
   ∩ dirty, with an outcome flash ("N saved · M held (changed on disk)"). POC
   twin fixed the same way (`touchedRef`; it cleaned ALL docs).
6. **MED (2 critics) — arrows left the caret armed** — Enter/typing then acted on
   an invisible, possibly other-file row. Fix: ↑/↓ drop the caret (+ composition),
   symmetric with Esc — a recorded deviation from the design's "carry" wording.
7. **MED — the ⌦ window-end guard read the STALE model** (rows refresh at render
   top; a key-repeat burst within a frame could eat the boundary newline below
   the window). Fix: both boundary guards judge against the RESOLVED window
   anchors (`mb_window_edges`), the live truth.
8. **MED — ⌫ pairing asymmetry**: `(` paired on insert but ⌫ left the widowed
   half. Fix: ⌫/⌦ route through the editor's own `apply_editor_key_multi`
   (pairing-aware under auto-close, #338 symmetry) — which also deleted the
   duplicated transient-edit blocks.
9. **MED parity — dirty-dot position**: Rust drew it right-aligned by the count;
   the POC (the reference) puts it LEFT-adjacent after the path. Fix: moved into
   the path cell.
10. **LOW — materialize's acquire could strand on the mint-miss arm** (the
    F-claude-427-b symmetric-release grep): else-arm now releases. **LOW —
    `mb_live_versions` never pruned**: both close paths retain-prune by mb id.
    **LOW — a fresh caret inherited a stale composition**: `mb_place_caret`
    clears `mb_marked`. **LOW — POC header comment claimed snapshot files were
    "read-only"**: reworded (they splice locally, no write-through).

**Rejected / recorded, no action:** `mb_marked` stays (IME composition arms are
the RECORDED v1 deferral; commits arrive via `replace_text_in_range` once C1
registered the handler) · `run_char_range` O(text) per line at mint —
materialize/birth-only, accepted · frozen `match_anchors` len — line-truth only
(the render is a full-row wash); pinned as a #429 spec note (replace must not
consume them as splice ranges) · ⌘V/⌘C/⌘X dead on an mb tab — v1 scope, recorded
· Enter-at-EOF one-keystroke caret hide + the seam-remnant on partial window
deletion — cosmetic, PINNED as validate unit-table rows · aliased-path double
pin/release — balanced · shift-modified keys pass the gate — #427's shipped gate,
unchanged · POC keydown resubscribe churn — functional-setState makes it
harmless; recorded.

**Verified clean by the critics:** the AD-claude-edit-post-state-span discipline
(every mb edit rides the engine's self-classifying paths; no asserted tags);
window anchor math (grow-at-edges arithmetic, interior grow/shrink, EOF clamps,
merge-on-collision); lifecycle refcounts across materialize/birth/both closes
(+ the jump path); `SaveCause::Sweep` rides the background arm only; persistence
untouched (transient tab; runtime-only state); provenance clean (brand scrub
zero hits; all cited codes real).

**Verify:** `cargo check --workspace --tests` clean · multibuffer suite 18/18 ·
clippy zero warnings · POC tsc clean · fmt applied.

Ledger appends: F-claude-428-{a,b,c}, PR-claude-428-{a,b}.

## Phase 4 — Validate

**Tests added.**
- `multibuffer.rs` — 8 new units (20 total): `window_row_runs` groupings,
  `run_char_range` (row starts, EOF clamps, empty runs, CHAR-not-byte),
  `rebuild_lines` (out-of-order heal, abut-merge, empty-run drop, rebased span
  marks, hairline on the second run only, injected text), `journal_note_edit`
  retain-then-push semantics (real-edit clears redo; coalesced continuation
  MOVES its entry past an interleaved other-file entry; a new group stacks),
  `refresh_cum` recount + selection re-clamp (incl. emptied → 0),
  `slot_of_row` hits/misses.
- `marley_editor::buffer` — `undo_depth_counts_groups_and_the_open_transaction`
  (0 → 1 ungrouped → 2 with an OPEN transaction → stable at close → drains):
  kills all six accessor mutants the drives were self-consistent under.
- `headless_drive.rs` — 5 drives on the REAL router:
  `multibuffer_typing_writes_through_headless` (REQ-001/002/006 — typed chars
  land in the real buffer, the live rebuild re-reads the excerpt line, touched
  marks, dirty sets), `multibuffer_boundaries_pin_the_window_edges_headless`
  (REQ-002/003 — Enter GROWS the window; backspace blocked at the top edge;
  interior backspace edits), `multibuffer_undo_routes_cross_file_headless`
  (REQ-004 — cross-file order, redo replay, and the STALE-entry drain after a
  tab-side undo — the liveness guard proven), `multibuffer_save_consent_headless`
  (REQ-005 — touched-only saves to DISK; an untouched tab-dirtied sibling NOT
  swept; an external mutation HOLDS the write + the outcome flash),
  `multibuffer_target_lifecycle_headless` (REQ-007 — pin +1/release, lazy birth
  holds ONE view, drops at close).

**Runs.** `cargo nextest run --workspace`: **2225 passed** (2231 by gate time),
7 skipped; doctests green. One regression caught and fixed mid-validate:
`sync_multibuffer_live` at render top panicked AT THE LAUNCHER
(`active_project()` on an empty shell) — `active_mb_id` now zero-project-safe;
the pre-existing launcher drive pinned it. The save-consent drive also caught
that the inspect's touched-sweep splice had NOT applied (a silent fmt-anchor
miss — `mb_save_all` still swept targets): re-applied against the real text and
DRIVE-PROVEN; a full grep-audit then verified every other inspect fix present
(depth guards x4, transient helper x5, edge resolver x3, retain+push, arrows
drop, re-mint, pins, prunes, canvas, dot).

**Live drive (typed char REQUIRED — PR-claude-428-a).** Chad's restored
workspace kept routing keys terminal-side, so the drive moved to a SEEDED
SCRATCH HOME (the shell blob written with TOML-legal unicode escapes for the
0x1F unit separators — the no-hand-blob rule bit once en route, recorded) with
throwaway files: materialize "delta" → two groups, washes, disjoint-window
hairline, in-scroll footer, `focus: search results` → click the match line →
**type Z live** → "gamma deltaZ" with the caret bar + the header dirty dot
("alpha.txt ●1") → Esc+Enter jump → the file's editor tab shows "gamma deltaZ"
+ dirty chip, caret at the match, mb tab still open → undo + quit; the scratch
disk file verified UNTOUCHED. Captures: scratchpad/428-live-mb.png,
428-live-typed.png, 428-live-writethrough.png.

**Parity pair (POC ↔ live, sampled).** POC scratchpad/428-parity-poc.png ↔ live
428-live-typed.png: caret bar 2px teal both (POC x[761,762] ↔ live x[370,371]);
dirty dot 6px teal, LEFT-adjacent after the path both; match wash (17,30,34) ↔
(18,28,31); selected wash (21,37,40) live-only (the POC mock has no keyboard
selection — recorded). The pixel pass CAUGHT a real Rust delta: the wash hugged
the text (427's "full-row" reading was an artifact of long lines) — `w_full()`
on the row div fixed it and the re-driven capture samples the wash edge-to-edge
(x240 → x940). PARITY HOLDS.

**Gate.** Run 1: RED x2 — clippy (`get(..).is_none()` in a drive →
`contains_key`) and mutation (6 missed, all on the new `undo_depth`/`depth`
accessors — the drives were SELF-CONSISTENT under the mutation since writer and
checker read the same fn; the direct unit pins the values). Run 2:
**GATE GREEN [diff] — 15/15** (cov 100, MSI 100; log:
scratchpad/gate-428-run2.log). Pre-existing exclusions: none new.

## Phase 5 — Complete

Docs (§21): CHANGELOG entry; editor.md's multibuffer section gained the #428
editable face; roadmap records phases 1+2 shipped. Parity: the POC already
matches (the touched-⌘S + comment fixes were back-ported at inspect);
MARLEY-PARITY's multibuffer row extended with the #428 pixel pairs + recorded
gaps. Knowledge: F-claude-428-{a,b,c} + PR-claude-428-{a,b} (at inspect),
AD-claude-428-excerpt-editing-rides-existing-machinery-001,
L-claude-428-assert-your-splices-and-drive-your-fixes-001. Ticket closed;
pipeline archived.
