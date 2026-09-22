---
pipeline_id: a7323c75-766d-4c08-ae23-708e4af97575
ticket: docs/planning/tickets/open/TICKET-428-multibuffer-editing.md
status: Phase 5 — Complete PASS
title: Editable multibuffer — excerpt edits write through to the real Buffers
type: feature
milestone: M32
references:
  - docs/planning/design-notes/display-map-shelf.md
  - docs/planning/pipeline/completed/427-multibuffer-core.spec.md
  - docs/zed_architecture/subsystems/03-editor-multibuffer.md
  - docs/planning/pipeline/completed/397-editors-onto-registry.spec.md
  - docs/planning/pipeline/completed/275-external-file-change.spec.md
  - docs/planning/pipeline/completed/282-grouped-undo.spec.md
---

## Title

The B-c chain's step 4 — the #427 stitched view becomes a real editor. A caret lives in
the excerpts; typing, backspace, ⌦, and newline route through the app's ONE insert
mechanism to the correct underlying file's ONE registry Buffer (the #397
one-instance-many-views contract — an edit is instantly visible in the file's own tab);
excerpt rows/spans REBASE over edits (the #269 anchor + delta-log machinery, the folds'
resolve-and-re-derive shape); ⌘Z/⌘⇧Z is one cross-excerpt story routed to the target
buffers' own histories; ⌘S saves every touched file through `save_editor_by_id`'s
background-consent semantics, the #275 conflict net intact. Dirty headers render. Edits
at excerpt boundaries follow a pinned, test-named policy.

## Scope

### In
- **Input routing**: the multibuffer tab becomes a text-input target — the
  `EntityInputHandler` path (`text_input_blocked` gains no false block; the
  active-surface resolution learns the multibuffer) delivers printables/IME to the
  selected excerpt's target buffer via the standing `replace_text_ctx` →
  `buffer.edit_*` chain; backspace/⌦ via `apply_editor_key_multi`'s buffer fns; Enter =
  newline (the router's indent-aware shape or a pinned simpler v1 — design decides).
  SINGLE caret v1, excerpt-addressed, owned by the multibuffer (never fighting the
  buffer's `SelectionSet` home-parking).
- **The live excerpt index**: per-file excerpt positions become ANCHOR-derived
  (`Buffer::anchor_at`/`resolve_anchor` over the delta log) and re-derive per frame
  memoized on `(nonce, version)` — the `fold_projection_inner` precedent. Open-instance
  text reads LIVE; boundary policy (an edit at an excerpt's first/last row: grow vs
  clamp) pinned by unit table.
- **Instance lifecycle**: files already OPEN get a pinned registry view at materialize
  (released at tab/project close — the cockpit pinned-view precedent, PR-claude-427-a
  discipline); a NOT-open file's instance is created lazily at FIRST EDIT (500-file
  result sets stay cheap); un-edited not-open files keep #427 snapshot text.
- **Undo**: per-buffer `UndoHistory` does the reverting; the multibuffer keeps an edit
  JOURNAL of target ids so ⌘Z/⌘⇧Z walk cross-excerpt order; every new edit action
  classifies itself per AD-claude-edit-post-state-span (earned tags, no asserted flags).
- **Save**: ⌘S in the multibuffer = `save_editor_by_id(id, cause)` per DIRTY touched
  instance — background-consent semantics (a `Changed` conflict HOLDS the write and
  surfaces; `Deleted` recreates; no silent clobber). No new save machinery.
- **Render**: dirty marker on the header band; the caret drawn in the selected excerpt
  line; edited text re-renders live (the standing `cx.notify` + memo-key idiom).
- **React-first**: the POC MultibufferView gains the editable interaction first.

### Out (explicitly deferred)
- Replace-all + results-as-the-default-form (#429); the problems form (#430).
- Multi-cursor / selection drag inside the multibuffer; cross-excerpt text selection.
- Excerpt context expansion (growing a window on demand) and re-running the search.
- Format-on-save inside the multibuffer (plain save v1; the latch stays editor-tab).
- The DisplayMap/singleton unification (still the chain's later prize).
- Persistence (the tab stays transient — #427 D6 holds).

## Reference (§20)

**Zed (the editor reference — same-gpui-stack).** Behavior matched: editing an excerpt
writes through to the real file's buffer; positions are anchor-composed (a per-buffer
anchor plus its excerpt) so selections and excerpt ranges survive edits; undo is one
cross-buffer story; the surface is "a single editor over slices of many buffers".
Research: `docs/zed_architecture/subsystems/03-editor-multibuffer.md` §6 (MultiBuffer =
`buffers: BTreeMap<BufferId, BufferState>` + `history` cross-buffer undo; the composed
`Anchor = Min | Excerpt(ExcerptAnchor) | Max`; "editing an excerpt writes through to the
real file"). The roadmap's standing caveat holds: adopt the CONTRACT (write-through,
composed anchors, one undo story), not the container — no SumTree, no
MultiBufferSnapshot; Marley's per-line row model + per-buffer anchors admit the smaller
shape. Clean-room: deconstruction docs only; no Zed source read.

### Prior art

1. **Behavior maps** — the §6 chapter above; `docs/zed_architecture/crates/search.md`
   phase-2 ("results anchored to buffers, not offsets" — the upgrade hinge #427 stored
   absolute rows for).
2. **Published** — none new needed (LSP `didSave` is already wired into the save tail).
3. **In-tree (the highest-yield leg — Explore sweep 2026-08-14, post-#427 tree):**
   EVERYTHING 428 needs already exists; the ticket is wiring + one live index, not new
   machinery. The ONE insert mechanism (`EntityInputHandler` → `replace_text_ctx` →
   `Buffer::edit/edit_at_selections/edit_ranges_restoring_placing`, `app.rs:17994/18050`,
   `ime.rs:120`, `buffer.rs:250-717`); backspace/⌦ (`input.rs:261`); newline
   (`app.rs:19283`). The #269 anchor machinery (`anchor.rs`: `Anchor{version,offset,bias}`,
   `rebase_offset`; `Buffer::anchor_at/resolve_anchor/edits_since` over the delta log,
   `buffer.rs:204-236`) with folds as the production resolve-and-re-derive precedent
   (`fold_projection_inner`, `app.rs:14431-14467`, memo keyed on the anchor vec +
   (nonce, version)). Per-buffer `UndoHistory` + the #282/#338 grouped-undo discipline
   (`undo.rs`, AD-claude-edit-post-state-span). The instance model: #397
   one-instance-many-views (SUPERSEDES #259's two-buffer fork — the ticket's open
   question is settled by the live tree), `resolve_open`/`find_open` (`content.rs:258-306`),
   `editors_under` (`content.rs:343`), the cockpit's pinned non-tab view
   (`content.rs:353-360`), D-OPEN-SELECTION-HOME parking (`app.rs:17152`). Saves:
   instance-addressed `write_and_mark`/`save_editor_by_id` + `extchange::external_action`
   (AD-claude-354; `app.rs:10128-10216`, `extchange.rs:47`) — the background-consent
   path IS the multibuffer's ⌘S; no save-all exists and none is invented. Dirty:
   `is_dirty`/`mark_saved` (`editor_surface.rs:152-166`).

## React-first (parity)

**UI-AFFECTING — the #427 surface gains interaction (zone B; the POC is the design
source).** POC files: `views/MultibufferView.tsx` (caret in an excerpt line, editable
rows writing through to the shared doc store, dirty dot on the header band),
`App.tsx`/`pages/Workspace.tsx` (doc-store write-through so the file's editor view
shows the same text). Implement builds + screenshots the POC FIRST (dev server, READ
the PNG), then ports 1:1; validate captures the parity pair; complete adds the
MARLEY-PARITY row update.

## Locked-In Decisions

- **D1 — The live tree settles the instance fork: #397 rules.** Excerpt edits route to
  the file's ONE registry instance through the standing insert mechanism — no second
  Buffer, no divergent copy, no new edit API. (The ticket's #259 alternative is
  historical; #397 explicitly superseded it.)
- **D2 — Anchor-derived excerpts, fold-style.** Excerpt line positions become #269
  anchors resolved per frame (memo `(nonce, version)`); open-instance text reads LIVE.
  The snapshot fields stay for un-edited, not-open files. The boundary policy (edit at
  a window's edge rows) is pinned by a unit table at design.
- **D3 — Undo routes; buffers revert.** A multibuffer edit journal (target ids in
  order) routes ⌘Z/⌘⇧Z to the right buffer's own `UndoHistory`. Earned-tag discipline
  (AD-claude-edit-post-state-span) governs any new action classification.
- **D4 — ⌘S is N × `save_editor_by_id`.** Background-consent semantics per touched
  dirty instance; the #275 arm/conflict behavior is the net, unchanged.
- **D5 — Lazy instances, pinned views.** Open files: pin a view at materialize, release
  at close (PR-claude-427-a). Not-open files: instance born at first edit. Un-edited +
  not-open: snapshot text, zero cost.
- **D6 — The multibuffer owns its caret.** Excerpt-addressed single caret v1, mapped to
  buffer offsets at the edit boundary; it never writes the buffer's `SelectionSet`
  except transiently inside an edit call (respecting selection home-parking).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the multibuffer tab is active with an excerpt line selected, typed printable text shall insert into the underlying file's ONE registry Buffer at the caret's excerpt position, visible in the file's own editor tab. | Headless drive (type in mb → read the editor tab's buffer). |
| REQ-002 | Backspace, ⌦, and Enter (newline) shall route to the same target buffer through the standing edit fns, with the multibuffer caret placed per the edit. | Headless drives + pure units. |
| REQ-003 | Excerpt rows, spans, and the caret shall REBASE over buffer edits (an edit above an excerpt shifts its displayed rows; boundary edits follow the pinned grow/clamp policy), re-derived per frame memoized on (nonce, version). | Pure unit table + drive. |
| REQ-004 | ⌘Z (and ⌘⇧Z) in the multibuffer shall undo (redo) the most recent multibuffer-originated edit in its TARGET buffer, cross-excerpt order preserved, the reversal visible in both surfaces. | Headless drive. |
| REQ-005 | ⌘S in the multibuffer shall save every DIRTY touched file via `save_editor_by_id`; WHEN a file changed externally since its snapshot, the save shall HOLD with the #275 conflict surfaced, never silently clobbering. | Drive with a disk mutation. |
| REQ-006 | A dirty excerpted file's header band shall render a dirty marker; a completed save shall clear it. | Model units + render assert + live capture. |
| REQ-007 | Materializing shall pin one registry view per ALREADY-OPEN excerpted file and closing the tab shall release every pinned/edited instance view (view_count returns to the pre-materialize count; no leak, no premature drop of an edited buffer). | Headless drive (view_count accounting). |
| REQ-008 | The new pure surface shall hold 100% coverage / 100% MSI; the gate shall be GREEN [diff]. | gate:4/5; `scripts/gates.sh --diff`. |

## Phase Plan

- **P2 Design** — the live-index shape (anchors per excerpt line vs per window); the
  input-routing seam (`active_editor_mut` vs a parallel multibuffer arm at each gate:
  typing/undo/save/Enter); the caret model + edit-boundary mapping; the journal type;
  the lazy-instance birth path (reuse `open_editor_instance`); the boundary-policy
  table; the POC build plan; the test table per REQ.
- **P3 Implement** — POC FIRST (build + screenshot + READ), then Rust to the manifest.
- **P3.5 Inspect** — critics on: the insert-path classification (earned tags), anchor
  rebase math, view-lifecycle accounting (the #427 F-/PR- class), save-consent arms,
  selection-home interaction, provenance.
- **P4 Validate** — the REQ tests + drives; live drive + parity pair; gate GREEN.
- **P5 Complete** — CHANGELOG + editor.md/app_shell/roadmap; MARLEY-PARITY row; ledger;
  close + archive.
