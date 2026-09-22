---
spec_id: editor
component: marley_editor
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Rope-backed editor — buffer, anchors, selection, undo/redo, find, and layout pipeline
goal: A clean-room Rust text-editing core that backs both the input editor and rich-text blocks, distinguishing human from programmatic (agent) writes.
reuses: [ropey, imara-diff, regex-automata]
spec_source: "behavior-only — observable text-editing I/O: a rope-backed buffer with stable anchors across edits, multi-cursor selection, provenance-grouped undo/redo, regex find, line diff, soft-wrap layout, and change notifications. No fork file paths or private type/module/static names. Seams per standards/seam-contracts.md §3."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_editor` is Marley's text-editing core: a rope-backed mutable `Buffer` with stable
`Anchor`s, a multi-cursor selection model owned by the buffer, bounded undo/redo, regex find,
line-level diff, and a layout pipeline that turns buffer content into laid-out lines. It is
the shared engine behind both the command/prompt input editor and rich-text blocks. Every
write carries an `EditOrigin` (`Human` vs `Agent`) so undo grouping and downstream consumers
can tell user typing apart from programmatic/streamed agent output. It is a fresh Rust
implementation that reuses the permissive crates `ropey` (rope), `imara-diff` (diff), and
`regex-automata` (find).

This crate is the single owner of the buffer-core seam types that `marley_syntax` and the
prompt input bind to (seam-contracts §3): `BufferDelta`, `BufferVersion`, `Point`, the `Rope`
alias, and the `BufferEvent` subscription. It depends on `marley_text_offsets` (M0) for the
one workspace `CharOffset`/`ByteOffset` vocabulary (seam-contracts §1) and never declares its
own offset newtypes. The editor↔completer currency is `marley_text_offsets`: `Buffer` owns the
stateful `char_to_byte`/`byte_to_char` conversion the prompt-input integration uses to turn a
completer `Range<ByteOffset>` span into a `Range<CharOffset>` before calling `Buffer::edit`
(§1.1). `marley_editor` imports no `marley_completer` type and vice versa.

## Public surface (the contract)
```rust
// The one workspace offset vocabulary — owner `marley_text_offsets` (seam-contracts §1).
// Private fields, non-interchangeable; this crate declares NO local offset newtype.
use marley_text_offsets::{ByteOffset, CharOffset};
use std::ops::Range;

/// Rope storage type, exported so syntax/layout consumers bind to one alias. §3.4
pub type Rope = ropey::Rope;

/// Provenance of an edit; load-bearing for undo grouping and consumer routing.
pub enum EditOrigin { Human, Agent }

/// Tie-break side for an anchor when text is inserted exactly at its offset.
pub enum AnchorBias { Left, Right }

/// A stable position that survives edits.
pub struct Anchor { /* opaque */ }

/// One contiguous selection: caret at `head`, fixed end at `anchor`.
/// Endpoints are read-only; they are clamped at the `SelectionSet`/`Buffer` boundary
/// (no public field bypasses clamping). §R19
pub struct Selection { /* opaque: { anchor: CharOffset, head: CharOffset } */ }
impl Selection {
    pub fn new(anchor: CharOffset, head: CharOffset) -> Self;
    pub fn anchor(&self) -> CharOffset;
    pub fn head(&self) -> CharOffset;
}

/// An ordered, non-overlapping set of selections (multi-cursor).
/// Construction sorts and merges overlapping/touching members. §R18
pub struct SelectionSet { /* invariant: sorted, disjoint */ }
impl SelectionSet {
    pub fn from_selections(selections: Vec<Selection>) -> Self;   // sorts + merges
    pub fn selections(&self) -> &[Selection];
}

/// Monotonic buffer revision. §3.2 — PRIVATE field; construct/advance via the API.
pub struct BufferVersion(/* private */ u64);
impl BufferVersion {
    pub fn initial() -> Self;
    pub fn next(self) -> Self;
    pub fn as_u64(self) -> u64;
}

/// The change produced by one edit, for incremental consumers. §3.1
pub struct BufferDelta {
    pub char_range: Range<CharOffset>,   // pre-edit replaced span, char-indexed
    pub byte_range: Range<ByteOffset>,   // pre-edit replaced span, byte-indexed (for tree-sitter)
    pub new_char_len: usize,             // chars inserted by the replacement
    pub new_byte_len: usize,             // bytes inserted by the replacement
}

/// Result of applying an edit.
pub struct EditResult { pub delta: BufferDelta, pub version: BufferVersion }

/// A row/column point. `column` is a UTF-8 BYTE offset within the row
/// (tree-sitter convention); `row` is 0-based. §3.3 — consumed by `marley_syntax`.
pub struct Point { pub row: u32, pub column: u32 }

/// Emitted to subscribers on every state change. §3.5
pub enum BufferEvent {
    Edited { delta: BufferDelta, origin: EditOrigin, version: BufferVersion },
    SelectionChanged { version: BufferVersion },
}

/// Subscription handle wrapping a `Receiver<BufferEvent>`. §3.5
pub struct BufferSubscription { /* opaque: Receiver<BufferEvent> */ }
impl BufferSubscription {
    pub fn try_recv(&self) -> Option<BufferEvent>;
}

pub struct FindOptions { pub wrap: bool, pub case_insensitive: bool }
pub enum FindError { InvalidPattern }

/// Cheap immutable point-in-time view via structural sharing with the rope.
pub struct BufferSnapshot { /* opaque */ }

pub struct Buffer { /* rope + anchors + selection + undo stack + version + subscribers */ }
impl Buffer {
    pub fn new() -> Self;
    pub fn from_str(text: &str) -> Self;
    pub fn len_bytes(&self) -> usize;
    pub fn len_chars(&self) -> usize;
    pub fn len_lines(&self) -> usize;
    pub fn version(&self) -> BufferVersion;
    pub fn char_to_byte(&self, off: CharOffset) -> ByteOffset;   // editor↔completer seam, §1.1
    pub fn byte_to_char(&self, off: ByteOffset) -> CharOffset;   // editor↔completer seam, §1.1
    pub fn point_at(&self, off: CharOffset) -> Point;            // row + UTF-8 byte column, §3.3
    pub fn text_in_range(&self, range: Range<CharOffset>) -> String;
    pub fn snapshot(&self) -> BufferSnapshot;
    pub fn subscribe(&self) -> BufferSubscription;               // §3.5 — Edited emitted before edit() returns
    pub fn selection(&self) -> &SelectionSet;                    // selection lives on the buffer
    pub fn set_selection(&mut self, sel: SelectionSet);          // clamps, merges, emits SelectionChanged, breaks coalescing
    pub fn anchor_at(&self, off: CharOffset, bias: AnchorBias) -> Anchor;
    pub fn resolve(&self, anchor: &Anchor) -> CharOffset;
    pub fn edit(&mut self, range: Range<CharOffset>, replacement: &str, origin: EditOrigin) -> EditResult;
    pub fn undo(&mut self) -> Option<EditResult>;
    pub fn redo(&mut self) -> Option<EditResult>;
    pub fn find(&self, pattern: &str, opts: FindOptions) -> Result<Vec<Range<CharOffset>>, FindError>;
    pub fn find_next(&self, pattern: &str, from: CharOffset, opts: FindOptions) -> Result<Option<Range<CharOffset>>, FindError>;
    pub fn diff(&self, old: &BufferSnapshot) -> Vec<LineChange>;
}

pub struct LineChange { pub old: Range<usize>, pub new: Range<usize> }

/// One laid-out display line. `width` is in monospace display columns. §R20
pub struct LayoutLine { pub range: Range<CharOffset>, pub width: usize }
/// `soft_wrap_width` is in monospace display columns. §R20
pub fn layout(buf: &Buffer, soft_wrap_width: Option<usize>) -> Vec<LayoutLine>;
```

## EARS Requirements
- **R1.** The system shall store buffer text in a `ropey::Rope` and report `len_bytes()`, `len_chars()`, and `len_lines()` consistent with the stored content.
- **R2.** The system shall, for any valid `CharOffset`, return a `ByteOffset` from `char_to_byte` that round-trips back to the same `CharOffset` via `byte_to_char`.
- **R3.** WHEN `Buffer::edit(range, replacement, origin)` is called, the system shall replace exactly the characters in `range` with `replacement` and return an `EditResult` whose `BufferDelta` records `char_range` (= `range`), `byte_range` (the byte span of `range` before the edit), `new_char_len` (chars in `replacement`), and `new_byte_len` (bytes in `replacement`).
- **R4.** WHEN an edit is applied, the system shall advance the buffer's `version` to `BufferVersion::next(previous)` so each applied edit increments the underlying `as_u64()` by exactly one.
- **R5.** WHEN an edit inserts or removes characters strictly before an existing `Anchor`'s resolved offset, the system shall shift that anchor by the net character delta so it references the same logical position.
- **R6.** WHILE an `Anchor` has `AnchorBias::Left`, WHEN text is inserted exactly at the anchor's resolved offset, the system shall keep the anchor before the inserted text.
- **R7.** WHILE an `Anchor` has `AnchorBias::Right`, WHEN text is inserted exactly at the anchor's resolved offset, the system shall move the anchor to after the inserted text.
- **R8.** WHEN `Buffer::snapshot()` is called, the system shall return a `BufferSnapshot` that shares the rope's persistent structure with the originating `Buffer` (no full-text copy) and whose content is unaffected by subsequent edits to that `Buffer`.
- **R9.** WHILE the most recent undo group originates from `EditOrigin::Human`, WHEN a contiguous `Human` insertion adjacent to the prior insertion is applied with no intervening `set_selection` call, the system shall coalesce it into that same undo group.
- **R10.** WHERE an edit's `EditOrigin` is `Agent`, the system shall record it as its own discrete undo group and shall never coalesce it with an adjacent `Human` group.
- **R11.** WHEN `Buffer::undo()` is called and the undo stack is non-empty, the system shall restore the content and the `SelectionSet` to the snapshot recorded with the most recent undo group, move that group to the redo stack, and return `Some(EditResult)`.
- **R12.** WHEN `Buffer::redo()` is called and the redo stack is non-empty, the system shall re-apply the most recently undone group and return `Some(EditResult)`.
- **R13.** IF a non-undo/non-redo edit is applied WHILE the redo stack is non-empty, THEN the system shall clear the redo stack.
- **R14.** WHILE the undo stack holds the configured maximum (default 30) groups, WHEN a new undo group is pushed, the system shall drop the oldest group so the stack length never exceeds the maximum.
- **R15.** WHEN `Buffer::find(pattern, opts)` is called with a valid pattern, the system shall return every non-overlapping match as a `Range<CharOffset>` in ascending start order using a `regex-automata` matcher.
- **R16.** IF `find` or `find_next` is called with a syntactically invalid regex pattern, THEN the system shall return `Err(FindError::InvalidPattern)` and leave the buffer unchanged.
- **R17.** WHEN `find_next(pattern, from, opts)` is called with `opts.wrap == true` and no match starts at or after `from`, the system shall return the first match from the buffer start, or `None` when the buffer contains no match; WHEN `opts.wrap == false` and no match starts at or after `from`, the system shall return `None` without scanning before `from`.
- **R18.** WHEN a `SelectionSet` is constructed (`from_selections`) from members that overlap or touch, the system shall merge them into one selection spanning the union of their ranges, keeping the set sorted and disjoint.
- **R19.** WHEN `Buffer::set_selection(sel)` is called, the system shall clamp every member endpoint of `sel` into `[CharOffset::from(0), len_chars()]` at the `SelectionSet`/`Buffer` boundary before storing it (endpoints are never set by direct public-field construction).
- **R20.** WHEN `layout(buf, Some(w))` is called with `w > 0` (where `w` and each `LayoutLine.width` are counted in monospace display columns), the system shall emit `LayoutLine`s such that no line's `width` exceeds `w` unless a single grapheme cluster is itself wider than `w`.
- **R21.** WHEN `layout(buf, None)` is called, the system shall emit exactly one `LayoutLine` per buffer line as delimited by line endings.
- **R22.** WHEN `Buffer::diff(old)` is called, the system shall return the line-level `LineChange` set computed via `imara-diff`, mapping disjoint old line ranges to new line ranges in ascending order.
- **R23.** WHEN an edit is applied, the system shall emit a `BufferEvent::Edited { delta, origin, version }` to every `BufferSubscription` before `edit` returns.
- **R24.** WHEN `Buffer::set_selection(sel)` is called, the system shall store the clamped/merged `SelectionSet`, emit `BufferEvent::SelectionChanged { version }` (carrying the current `version`) to subscribers, and close the current undo group so a subsequent `Human` insertion starts a new group (breaking R9 coalescing).
- **R25.** WHERE `FindOptions.case_insensitive` is true, the system shall match `pattern` case-insensitively (Unicode simple case folding) in both `find` and `find_next`; WHERE it is false, the system shall match case-sensitively.
- **R26.** WHEN `Buffer::find(pattern, opts)` is called, the system shall ignore `opts.wrap` (wrap governs only `find_next` traversal) and return all matches across the whole buffer.
- **R27.** WHEN `Buffer::point_at(off)` is called for a valid `CharOffset`, the system shall return a `Point` whose `row` is the 0-based line containing `off` and whose `column` is the UTF-8 byte offset of `off` within that row.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | Lengths in bytes/chars/lines match stored rope content (R1) | planned |
| 2 | `char_to_byte`∘`byte_to_char` is identity on valid offsets (R2) | planned |
| 3 | `edit` replaces exactly `range` and returns a correct `BufferDelta` (char+byte ranges, char+byte lens) (R3) | planned |
| 4 | Each applied edit advances `version` so `as_u64()` rises by exactly 1 (R4) | planned |
| 5 | Anchor before an edit shifts by net char delta (R5) | planned |
| 6 | Left-bias anchor stays before insert at its offset (R6) | planned |
| 7 | Right-bias anchor moves after insert at its offset (R7) | planned |
| 8 | Snapshot shares rope structure (no full copy) and is immune to later edits (R8) | planned |
| 9 | Contiguous Human inserts with no `set_selection` between them coalesce into one undo group (R9) | planned |
| 10 | Agent edits are discrete and never coalesce with Human (R10) | planned |
| 11 | `undo` restores prior content + recorded selection snapshot and returns `Some` (R11) | planned |
| 12 | `redo` re-applies last undone group and returns `Some` (R12) | planned |
| 13 | New edit while redo stack non-empty clears redo stack (R13) | planned |
| 14 | Undo stack length never exceeds default max 30 (R14) | planned |
| 15 | `find` returns ascending, non-overlapping matches (R15) | planned |
| 16 | Invalid regex yields `Err(InvalidPattern)`, no mutation (R16) | planned |
| 17 | `find_next` wraps to first match when `wrap==true` and none follows `from`; returns `None` when `wrap==false` (R17) | planned |
| 18 | `SelectionSet::from_selections` merges overlapping/touching members to a sorted, disjoint union (R18) | planned |
| 19 | `set_selection` clamps endpoints to `[0, len_chars()]` at the boundary (R19) | planned |
| 20 | Soft-wrap lines never exceed width `w` (monospace columns) (R20) | planned |
| 21 | No soft-wrap yields one LayoutLine per buffer line (R21) | planned |
| 22 | `diff` returns ascending disjoint line changes (R22) | planned |
| 23 | `Edited` event delivered to a subscriber before `edit` returns (R23) | planned |
| 24 | `set_selection` emits `SelectionChanged` and breaks undo coalescing (R24) | planned |
| 25 | `case_insensitive` find matches fold case; sensitive find does not (R25) | planned |
| 26 | `find` ignores `wrap` and returns all whole-buffer matches (R26) | planned |
| 27 | `point_at` returns 0-based row + UTF-8 byte column (R27) | planned |

## Visual / Behavioral Acceptance
N/A — `marley_editor` is a headless engine. It produces a `Vec<LayoutLine>` model, not pixels;
rendering/painting of those lines is owned by a separate UI spec (M1 panel/render). No
AXUIElement or screenshot assertions apply to this component (quality-bar gate 15 is satisfied
vacuously: `visual_acceptance: N/A`).

## Test Plan
- **Unit:** one named test per requirement, all in-crate:
  - R1 `len_reports_match_content` · R2 `offset_round_trip` · R3 `edit_replaces_range_and_reports_delta` · R4 `version_increments_by_one` · R5 `anchor_before_edit_shifts` · R6 `left_bias_anchor_stays_before` · R7 `right_bias_anchor_moves_after` · R8 `snapshot_shares_structure_and_is_isolated` · R9 `human_inserts_coalesce_without_intervening_set_selection` · R10 `agent_edit_is_discrete` · R11 `undo_restores_content_and_selection` · R12 `redo_reapplies_group` · R13 `edit_clears_redo_stack` · R14 `undo_stack_bounded_at_30` · R15 `find_returns_sorted_nonoverlapping` · R16 `invalid_regex_errors_without_mutation` · R17 `find_next_wraps_and_no_wrap_returns_none` · R18 `from_selections_merges_overlapping_touching` · R19 `set_selection_clamps_to_bounds` · R20 `soft_wrap_respects_width` · R21 `no_wrap_one_line_per_buffer_line` · R22 `diff_returns_ascending_disjoint_changes` · R23 `edited_event_emitted_before_edit_returns` · R24 `set_selection_emits_event_and_breaks_coalescing` · R25 `case_insensitive_find_folds_case` · R26 `find_ignores_wrap_field` · R27 `point_at_reports_row_and_byte_column`.
  - The subscription seam (R23/R24) is exercised by calling `Buffer::subscribe()`, then draining `BufferSubscription::try_recv()` after `edit`/`set_selection` and asserting the event + its `BufferVersion`.
  - Coverage: 100% on all touched lines (quality-bar gate 4).
- **Integration:** drive the `Human` vs `Agent` write path end-to-end — a simulated human typing sequence (`EditOrigin::Human`, single-char inserts), a `set_selection` call, then an `Agent` streamed-block insert — assert undo collapses the human run up to the selection change in one step, the selection change breaks coalescing, and the agent block pops separately; assert `text_in_range`/`snapshot()` read back the combined content; and exercise the editor↔completer seam by mapping a `Range<ByteOffset>` (completer-style span) through `byte_to_char` and feeding it to `edit`. This is the cross-component seam the input editor and rich-text blocks bind to (seam-contracts §1.1, §3).
- **Visual:** none (headless component).
- **Regression:** full `cargo nextest run --workspace` green; property tests (offset round-trip, snapshot isolation, selection merge invariants) must keep passing across edits.

## Mutation Targets
`cargo mutants` must kill every viable mutant on the testable surface (MSI 100%):
- Off-by-one in `edit` char/byte range math, `new_char_len`/`new_byte_len`, and the `BufferVersion::next` increment (R3, R4).
- Anchor shift direction and bias tie-break comparison operators (`<`, `<=`) (R5–R7).
- Undo/redo stack push/pop ordering, redo-clear branch, the coalescing predicate ("no intervening `set_selection`"), and the `== 30` bound (R9–R14).
- Match-ordering, overlap, wrap branch, and `case_insensitive` fold flag in find (R15, R17, R25), and the error branch (R16).
- `SelectionSet` merge predicate (overlap-or-touch) and `set_selection` clamp bounds (R18, R19).
- `SelectionChanged` emission + coalescing-break branch in `set_selection` (R24).
- Soft-wrap width comparison (monospace columns) and single-line fallback (R20, R21).
- `point_at` row/column derivation (line index vs UTF-8 byte column) (R27).
- No ACCEPTED-UNTESTABLE lines are expected; if `ropey`/`imara-diff` re-export glue produces an unkillable mutant, record it as an explicit ACCEPTED-UNTESTABLE decision with the wrapped-crate reason.
- No `unsafe` in this crate is anticipated; if introduced, it requires a `// SAFETY:` note and `cargo +nightly miri test` (quality-bar gate 6).

## Dependencies
- REUSE (permissive): `ropey` (rope/buffer storage), `imara-diff` (line diff), `regex-automata` (find). All MIT/Apache — supply-chain clean per gate 8.
- Marley components: **depends on `marley_text_offsets` (M0)** for the single workspace `CharOffset`/`ByteOffset` vocabulary (seam-contracts §1); this crate declares no local offset newtype. It owns and exports the buffer-core seam types (`BufferDelta`, `BufferVersion`, `Point`, `Rope` alias, `BufferEvent` + subscription) that `marley_syntax` consumes (§3). The editor↔completer seam (`char_to_byte`/`byte_to_char`) is exercised via `marley_text_offsets` only; `marley_editor` imports no `marley_completer` type. Consumed downstream by the M1 input-editor / rich-text-block UI (separate spec) and the agent-write path.

## Out of scope / deferred
- Syntax highlighting / tree-sitter integration (separate `syntax` spec, M1+) — `marley_syntax` consumes this crate's `BufferDelta`/`BufferVersion`/`Point`/`Rope`.
- Markdown / ipynb / mermaid rich-content parsing and block elements (M2+).
- Vim-mode keymaps and readline-style key dispatch (input-binding spec, M2).
- Pixel painting of `LayoutLine`s and viewport/scroll/offset-mapping (render/UI spec, M1+).
- Inline rich-text style markers (bold/italic/link/color) — M2 once rich-text blocks land.

## Clean-room provenance
Behavior-derived from a fork-reference doc describing observable text-editing I/O only (rope
buffer, stable anchors, multi-cursor selection, provenance-grouped undo/redo, regex find, line
diff, soft-wrap layout) — no private module/type/static names and no fork file paths. Per
seam-contracts §11, the `clean_room` line is downgraded to
`behavior-derived from a fork-reference doc; IP-counsel sign-off pending` until the behavioral
wall + sign-off land (open item in `clean-build-plan.md`). The public surface uses
Marley-original identifiers and reuses the shared `marley_text_offsets` offset vocabulary (no
Warp-internal name appears). REUSE crates (`ropey`, `imara-diff`, `regex-automata`) are
MIT/Apache.
