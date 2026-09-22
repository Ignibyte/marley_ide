---
pipeline_id: ecbb8ecc-23c3-4736-bec3-b0c306a28142
ticket: forge#296 (2fb2b40a-d84c-4846-9a20-8265b0532ef8) · local docs/planning/tickets/open/TICKET-296-multi-cursor-core.md
aar_id: d68a1852-579a-4110-90ce-7683688f7c11
status: Phase 5 — Complete PASS
title: Multi-cursor core — the ordered-disjoint SelectionSet + the N-caret edit (the deferred "M1.B")
type: feature
milestone: M19
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
Land the multi-member `SelectionSet` the crate explicitly deferred ("M1.B"): sort+merge to an
ORDERED + DISJOINT set, apply an edit at all N selections via the shipped BACK-TO-FRONT sweep
inside ONE undo group, and compute where the N carets land. The keystone of M19.

## Scope
### In (all PURE, `crates/editor`)
- **`SelectionSet::from_selections(Vec<Selection>) -> SelectionSet`** — sort by start, then MERGE:
  same-offset carets COLLAPSE to one; OVERLAPPING or ADJACENT selections UNION. Result is always
  ordered + disjoint (the invariant every M19 op relies on) and NEVER empty (an empty input is a
  programming error, not a state — the set always has ≥1 member).
- **`selections_after_multi_edit(set, replacement_char_len) -> SelectionSet`** — the ONLY real math:
  after replacing every selection's range with the same `replacement`, selection *i* becomes a CARET
  at `start_i + replacement_len + Σ_{j<i}(replacement_len − len_j)` (the cumulative delta of the
  edits BEFORE it). Typing over a selection collapses it to a caret after the inserted text (the
  universal behavior).
- **`Buffer::edit_at_selections(&mut self, set, replacement, origin) -> SelectionSet`** — the thin
  driver: `begin_undo_group(before)` → apply `edit()` at each range **BACK-TO-FRONT** → `end_undo_group(after)`
  → return `selections_after_multi_edit(...)`. Back-to-front needs NO rebasing (an edit at a higher
  offset cannot shift a lower one) — this REUSES the shipped `find::replace_all` idiom verbatim.

### Out (explicitly deferred — scope corrected at Design, D6)
- **The ENTIRE app-side shim moves to #297.** Design grounding found the app's `EditorSurface` does NOT
  use the crate's `SelectionSet` — each `OpenFile` holds its own `caret: CharOffset` +
  `anchor: Option<CharOffset>`, and **~61 call sites** depend on that shape (`active_caret` 16,
  `active_buffer_caret_anchor_mut` 22, `active_selection` 19). Converting the surface to an N-cursor set
  is a real refactor — and #296 has NO gesture to create a second cursor, so the shim would be
  un-drivable dead weight bolted onto the keystone. So: **#296 ships the PURE crate core only**
  (fully proven by units + the differential fuzzer); **#297 owns the app integration** (N-cursor surface
  state + the N-caret render + the typing route) *together with* the gestures that make it reachable and
  live-drivable. This keeps each slice shippable and puts the UI change in the ticket that can prove it.
- ⌘D-as-cursor (#298); the multi-cursor-aware ops (#299/#300/#303) — each its own ticket.
- Per-caret goal-column memory + multi-cursor MOTION (arrows move all cursors) — #297 introduces the
  goal column, so motion-with-N-cursors rides with it.

## Reference (§20)
Zed / VS Code multi-cursor semantics: an ordered, disjoint selection set; merge-on-overlap; an edit
applies at every cursor; the whole gesture is ONE undo unit. Marley matches with its own
`SelectionSet` + the shipped `Buffer`/undo-group machinery. Clean-room §20 — observed behavior only;
no Zed source read or translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — BACK-TO-FRONT sweep, NOT a cumulative rebase.** Grounding found the shipped
  `find::replace_all` already does exactly this ("offset-stable under back-to-front application —
  later replaces sit above it") inside one undo group. Reuse that idiom; the buffer edits then need
  NO rebasing. (The plan's original "rebase each later offset" framing was the harder, needless path.)
- **D2 — the one real computation is the RESULTING selections**, done FORWARD (a cumulative shift),
  because `replace_all` never needed them (the find bar has no caret) — so this is genuinely new and
  is the pure seam that must be fuzzed.
- **D3 — ONE undo unit via the shipped `begin_undo_group`/`end_undo_group` (#282)** — the mechanism
  already exists ("a whole block indent / replace-all undoes in a single step"); no new undo work.
- **D4 — the set never empties.** `from_selections` on an empty Vec is unreachable by construction;
  every mutator preserves ≥1 member.
- **D5 — a typed char over a selection COLLAPSES it to a caret** after the inserted text (universal).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `from_selections` shall return an ORDERED, DISJOINT set: same-offset carets collapse to one; overlapping OR adjacent selections merge into their union; an already-disjoint input passes through in sorted order. | selection.rs unit: the merge truth table (collapse / overlap-union / adjacent-union / disjoint-passthrough / unsorted input sorted). |
| REQ-002 | WHEN an edit is applied at N selections, the buffer text shall equal the text produced by applying those same edits one-at-a-time, and the resulting N carets shall land after each inserted text, shifted by the cumulative delta of the edits before them. | **DIFFERENTIAL FUZZER** (REQ-004) + unit fixtures (2 carets, 3 carets, a caret + a range selection, a delete). |
| REQ-003 | WHEN an N-caret edit is applied, ⌘Z shall revert ALL N in ONE undo step (and redo re-apply all N). | buffer unit: multi-edit → `undo()` once → text is the original; `redo()` → back. |
| REQ-004 | The N-caret edit's offset math shall be proven by a DETERMINISTIC DIFFERENTIAL FUZZER: the shipped back-to-front path shall agree with an independent front-to-back oracle (which rebases each remaining selection through the REAL `rebase_offset` + the actual `BufferDelta`) on BOTH final text AND final selections, across randomized buffers/selection-sets/replacements. | `#[test]` seeded-LCG fuzzer (the `t288_differential_fuzz` shape), ≥50k steps, 0 divergences. |
| REQ-005 | `from_selections` + `selections_after_multi_edit` + `edit_at_selections` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — the merge algorithm + the forward-shift math + the `edit_at_selections` driver; the render/typing shim; the fuzzer's two-implementation oracle; the test plan.
- **P3 Implement** — the 3 pure fns + the shim (N-caret render + typing route).
- **P3.5 Inspect** — the merge edges (adjacent vs overlapping vs same-offset), the forward-shift off-by-ones, the empty/single-member cases, the undo-group pairing, the back-to-front ordering.
- **P4 Validate** — the merge truth table + the DIFFERENTIAL FUZZER (the REQ-004 authority) + a headless 2-caret type/undo test; gate green [diff]. The live N-caret pixel drive is deferred to #297 (no gesture exists yet) — documented.
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #296.
