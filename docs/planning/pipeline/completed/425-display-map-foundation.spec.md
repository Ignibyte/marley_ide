---
pipeline_id: f4afa740-b024-41c9-ad10-8de2cd1cbed8
ticket: docs/planning/tickets/open/TICKET-425-display-map-foundation.md
status: Phase 5 — Complete PASS
title: The display-map foundation — one facade for every buffer-row↔display-row conversion
type: feature
milestone: M32
references:
  - docs/planning/design-notes/display-map-shelf.md
  - docs/zed_architecture/subsystems/03-editor-multibuffer.md
  - docs/zed_architecture/crates/editor.md
  - docs/planning/pipeline/completed/305-code-folding.notes.md
  - docs/planning/pipeline/completed/352-fold-overlay-projection.notes.md
---

## Title

The B-c chain's step 1: introduce the ONE display-map facade through which the editor's
render/geometry/scroll paths convert between buffer rows and display rows (slots), with
typed row spaces at the cross-space seams. Today the facade composes exactly one layer —
folds, delegating to the shipped `marley_syntax::FoldProjection` — and changes NOTHING the
user sees. Why now: soft wrap (#426) breaks row↔line identity and the multibuffer (#427+)
breaks row↔buffer identity; both need one place to insert a layer, and today the
projection is consumed raw at 14 production crossing sites in `app.rs` plus a memoized
builder — a seam-multiplication hazard (the F-#352 row-vs-slot class) the chain would
otherwise multiply.

## Scope

### In
- A new pure module owning the `DisplayMap` facade (row-space pipeline; folds-only today):
  `visible_count`, `buffer_row(slot)`, `slot_of(row)`, `folded_headers`,
  `viewport_offset` — the exact surface the 14 crossing sites consume.
- Typed row spaces (`BufferRow`, `DisplayRow`) at every facade signature; unwrap only at
  the gpui `uniform_list` rim (`item_count`/range/`scroll_to_item` are `usize` — verified
  in gpui-0.2.2 source) and where the design pins the Buffer line-API boundary.
- Re-route all production crossing sites + the memoized builder (`fold_projection()`)
  through the facade; straggler audit (grep the ORIGINAL pattern, any receiver).
- trybuild compile-fail pinning the row-space type safety (§7).
- Unit + property tests for the facade (identity fast path; equality with direct
  `FoldProjection` under folds; boundary probes at run edges).

### Out (explicitly deferred)
- Soft wrap and any second layer (#426).
- Excerpts / multibuffer / any base-space change (#427–#430).
- Any `LineLayout`/column-model change (the #331 two-boundary maps stay as-is).
- Any change to `marley_syntax::FoldProjection`'s internals or its fold semantics.
- Any user-visible behavior change whatsoever.

## Reference (§20)

**Zed (the editor reference — same-gpui-stack), architecture-level.** Zed's editor turns
"a buffer becomes a view" into an ordered stack of coordinate transforms (`DisplayMap`),
and every motion/geometry consumer speaks the top coordinate — the behavior Marley matches
is that ONE authority owns row-space conversion, so display features (folds today; wrap,
blocks, excerpts next) compose instead of colliding. Cited research (behavior maps, not
source): `docs/zed_architecture/crates/editor.md:79-100` (the six-layer stack),
`docs/zed_architecture/subsystems/03-editor-multibuffer.md` §3 (the uniform layer
contract). Marley adopts the CONTRACT, not the container (roadmap: "Marley's per-line
render admits a smaller shape") — no SumTree, no six layers; a facade over the shipped
`FoldProjection`. This ticket is user-invisible by design, so there is no behavioral
capture to match; the §20 value here is the architecture reference plus the wall (no Zed
source read — the deconstruction docs only).

### Prior art

1. **Behavior maps** — `docs/zed_architecture/crates/editor.md:79-100, 254-260` (stack +
   recommended Marley layer order), `crates/multi_buffer.md:219` ("render as one
   `uniform_list` with excerpt-header block rows" — validates the chain's later shape),
   `subsystems/03-editor-multibuffer.md` §3 (the Transform/summary/snapshot/sync
   contract). Research only.
2. **Published** — the dual-dimension summarized-sequence pattern (rope / piece-table CS)
   is public; Marley's Vec-backed half-open-run `FoldProjection` already implements the
   small-shape variant (AD-claude-305).
3. **Our permissive deps (the adoption leg, swept 2026-08-14):**
   - **gpui 0.2.2** (`~/.cargo/registry/src/…/gpui-0.2.2`): `uniform_list` takes
     `item_count: usize` and `scroll_to_item(ix: usize, …)` (`elements/uniform_list.rs:22-24,146`)
     — the rim's outer contract is `usize`; the typed slots unwrap exactly there, nowhere
     deeper. **And gpui ships `LineWrapper::wrap_line`**
     (`text_system/line_wrapper.rs:6-33`) — recorded here as #426's named seam so the wrap
     ticket's sweep starts at adoption, not invention.
   - **ropey** (via `marley_editor::Buffer`): owns line indexing (`len_lines`,
     `line_text`, `line_col`) — the buffer-row space's ground truth; the facade sits above
     it, never re-derives it.
   - **In-tree prior art is the headline:** `FoldProjection`
     (`crates/syntax/src/fold.rs`, half-open runs, cov/MSI 100), `LineLayout`'s
     two-boundary column maps (`code_view.rs:41-101`), and the h-scroll clipper/shift
     split — 425 PROMOTES existing Marley art into one facade rather than importing
     anything. `marley_text_offsets` is the workspace's newtype-vocabulary owner
     precedent (sole-owner rule, seam-contracts §1) and the candidate home for the row
     newtypes.

## React-first (parity)

N/A — no UI delta: a byte-identical internal refactor (REQ-002/003/004 pin the
no-visible-change contract); nothing the user sees changes, so there is nothing to design
in the POC first.

## Locked-In Decisions

- **D1 — A facade, not a framework.** One `DisplayMap` type owns every
  buffer-row↔display-row conversion the editor list makes; internally it composes exactly
  one layer today (delegating to `marley_syntax::FoldProjection`, unchanged). No generic
  `DisplayLayer` trait ships until a second layer exists (#426) — the contract is the
  deliverable, the container stays small (roadmap's "smaller shape"; the Zed container is
  `[Zed-derived]` anyway).
- **D2 — Typed row spaces at cross-space seams.** `BufferRow` and `DisplayRow` newtypes
  (private fields, §14) appear in every facade signature; a buffer-row where a display-row
  is required fails to compile (trybuild-pinned). Unwrap at the gpui rim only (its
  `uniform_list`/`scroll_to_item` contract is `usize`, verified in registry source). The
  F-#352 comparison class (`row == first` across spaces) becomes unrepresentable at the
  facade.
- **D3 — Byte-identical, proven not claimed.** The pre-425 suite passes UNCHANGED (any
  edit to an existing test is an inspect flag); facade outputs are property-equal to
  direct `FoldProjection` for the same inputs; the no-fold path keeps #305's
  identity-and-no-parse guarantee via the existing memo.
- **D4 — The #331 two-boundary column maps stay untouched and visible.** The facade is
  rows-only; `LineLayout`'s caret-map/code-map fork remains the column authority
  (AD-claude-two-boundary-maps), and the design documents where columns sit relative to
  the row map for #426 to consume.
- **D5 — Newtype owner is a design fork with a locked constraint.** ONE owner, private
  fields, no downstream re-declaration (§14/seam-contracts §1). If any crate beyond
  `marley_app` speaks the types in its API, they live in `marley_text_offsets`; if the
  facade fully contains them, app-local is acceptable. Design decides with the site
  manifest in hand.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The editor render/geometry/scroll paths shall route every buffer-row↔display-row conversion through the single `DisplayMap` facade: after the change, `FoldProjection` shall have no production consumer in `crates/marley_app` outside the facade module. | grep audit (original-pattern, any-receiver — the PR-137 straggler rule) recorded at inspect; review. |
| REQ-002 | WHEN no folds are active, the facade shall preserve the identity fast path — `visible_count == len_lines`, `buffer_row(s) == s`, `slot_of(r) == r` — and shall not trigger a tree parse. | Unit tests on the facade; the #305/#352 memo observable (parse-count/no-parse path) asserted. |
| REQ-003 | WHEN folds are active, facade results (`visible_count`, `buffer_row`, `slot_of`, `folded_headers`, `viewport_offset`) shall equal direct `FoldProjection` results for the same fold set, for all probed inputs including run boundaries. | Property/equivalence tests over generated fold sets; boundary probes at `[start,end)` edges. |
| REQ-004 | The full pre-existing workspace test suite shall pass with zero modifications to existing tests. | `cargo nextest run --workspace` green + diff audit (no existing test file/block edited) at inspect. |
| REQ-005 | A `BufferRow` passed where a `DisplayRow` is required at the facade boundary (and the converse) shall fail to compile. | trybuild compile-fail cases (§7). |
| REQ-006 | The new pure surface shall hold 100% line coverage and 100% MSI. | gate:4 + gate:5 (`scripts/gates.sh --diff`) exit codes. |

## Phase Plan

- **P2 Design** — facade API + module home; the newtype owner call (D5) with the full
  14-site manifest; the memo re-home (`fold_projection()` → the facade accessor); how the
  sizer and the row closure share one snapshot per frame (PR-137); the exact unwrap rim;
  regression test plan (unit + property + trybuild + the no-parse observable); risks.
- **P3 Implement** — the pure module + re-routed sites; `cargo check --tests` after every
  signature move (F-#386).
- **P3.5 Inspect** — independent critics vs the diff; the REQ-001 straggler grep; the
  REQ-004 no-test-edits audit; provenance check (no Zed source).
- **P4 Validate** — write + RUN the planned tests; `scripts/gates.sh --diff` green.
- **P5 Complete** — archive, ledger capture (§19), close the ticket; CHANGELOG + arch-docs
  (§21: editor.md gains the display-map section).
