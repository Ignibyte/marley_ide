# Subsystem 03 — Editor & Multibuffer (the editor element)

> Part of the Zed architecture reference (round 1). Zed is the EDITOR reference for Marley's editing
> surface. This subsystem is **the editor itself** — the element that turns a text buffer into an
> on-screen, editable, multi-cursor view: the `editor` crate (the `Editor` composer, `SelectionsCollection`,
> the `DisplayMap` transform stack, `movement`, the input path, `EditorElement`) and its capstone dependency
> `multi_buffer` (Zed's signature: N files' excerpts editable in one view).
>
> Marley's baseline after M15 is a **plain single-cursor code view** drawing from `marley_editor::Buffer`
> (`crates/editor/`) with an exact offset↔column map + caret. This doc maps the road from there to a real
> Zed-class editor: `SelectionSet` → multi-cursor, a DisplayMap-style transform layer, and the multibuffer.

Paths under `crates/…` (no repo prefix) are the **session Zed clone** (`…/scratchpad/zed-src/crates/`).
Marley paths are absolute from the repo root.

---

## Provenance posture (READ THIS FIRST)

The editor is **the most GPL-sensitive area in the whole project.** Nearly every design idea below lives in
a GPL-3.0 Zed crate; the render/input primitives it stands on are Apache-2.0 `gpui`; the summarized-tree and
coordinate-transform *ideas* are public CS. The boundary that keeps Marley's sellable "brain" clean is: **the
editor may be GPL (open-core editor), the brain must never touch Zed-derived code.** Mark everything.

| Capability | Zed crate(s) / where | License | Marley provenance tag |
|---|---|---|---|
| **`Editor` composer** (owns buffer + display + selections + scroll; the state machine) | `editor` (`editor.rs`) | **GPL-3.0-or-later** | `[Zed-derived]` — study the *decomposition*, write our own |
| **`SelectionsCollection` / multi-cursor** (disjoint anchor selections, columnar, ⌘D) | `editor` (`selections_collection.rs`, `selection.rs`) | GPL-3.0 | `[Zed-derived]` |
| **`DisplayMap` transform stack** (inlay→fold→tab→wrap→block→highlight) | `editor` (`display_map.rs` + `display_map/*`) | GPL-3.0 | `[Zed-derived]` for the *specific stack*; the layered-coordinate idea is `[public: CS/rope literature]` |
| **`movement`** (word/subword/paragraph/bracket/tree-sitter-node motions over display coords) | `editor` (`movement.rs`) | GPL-3.0 | `[Zed-derived]`; word/grapheme segmentation itself is `[public: Unicode UAX#29/#14]` |
| **The input path** (keystroke→action→per-selection edit; IME) | `editor` (`input.rs`) + `gpui` dispatch | GPL-3.0 (logic) / **Apache-2.0** (`EntityInputHandler`, actions, `FocusHandle`) | `[Zed-derived]` for the edit logic, `[gpui Apache-2.0]` for the dispatch/IME traits |
| **`EditorElement`** (gpui `Element`: request_layout/prepaint/paint) | `editor` (`element.rs`) | GPL-3.0 | `[Zed-derived]` for the layout algorithm; `[gpui Apache-2.0]` for the `Element`/`TextSystem`/`LineLayout` primitives it calls |
| **`MultiBuffer` + excerpt anchors** (the capstone) | `multi_buffer` | GPL-3.0 | `[Zed-derived]` |
| **The rope / summarized tree underneath** | `sum_tree` (Apache-2.0), `rope`/`text` (GPL) | mixed | Marley uses **`ropey` (MIT)** already — `[Marley-original]` choice; the *summary-dimension* pattern is `[public]` |
| **Marley's current `Buffer`/`SelectionSet`/`EditOrigin`/motions** | `marley_editor` (`crates/editor/`) | Marley's | `[Marley-original]` (modeled on the design, independently written) |

The load-bearing facts:

1. **The clean primitives are already Apache-2.0.** The gpui `Element` trait, `FocusHandle`, the action
   dispatch, `EntityInputHandler` (IME), `WindowTextSystem`/`LineLayout` (shaping) — everything the editor
   element *paints and receives input through* is `gpui`, which Marley already depends on. Marley can adopt
   those designs verbatim.
2. **The GPL is in the *logic*, not the primitives.** Zed's value is the *decomposition*: how selections are
   modeled as an anchor array, how six display transforms compose, how a keystroke fans out to N cursors. We
   re-derive that decomposition in Marley's own crates, tagged `[Zed-derived]`, and keep it inside the
   (intended-GPL) editor/terminal layer — never in the brain.
3. **Marley's rope is already independent.** `marley_editor::Rope = ropey::Rope` (MIT). We are *not* porting
   `sum_tree`/`rope`/`text`. The transform stack below is reimplemented as summary-carrying structures over
   ropey (or a thin Marley sum-tree), which is a public data-structure pattern, not Zed code.

---

## 0. The big picture — one pipeline, six stages

Zed's editor is a **pipeline from a text model to pixels**, with selections threaded through it:

```
 ┌──────────────┐   excerpts     ┌───────────────────────── DisplayMap ─────────────────────────┐
 │  Buffer(s)   │  (SumTree<     │  InlayMap → FoldMap → TabMap → WrapMap → BlockMap → highlights │
 │  language::  │──Excerpt>)────▶│  (each: Transform + {input,output} summary + coord newtype)    │
 │  Buffer      │   MultiBuffer  │  buffer coords ──────────────────────▶ DisplayPoint / DisplayRow │
 └──────────────┘                └───────────────┬───────────────────────────────────────────────┘
        ▲                                         │ DisplaySnapshot (immutable, per-frame)
        │ apply per-selection edits               ▼
 ┌──────┴───────┐   SelectionsCollection    ┌──────────────┐   Element::{request_layout,          │
 │  input.rs    │◀──(disjoint: Arc<[        │ EditorElement│    prepaint, paint}  →  gpui shapes   │
 │ handle_input │    Selection<Anchor>]>)   │ (element.rs) │    lines via WindowTextSystem         │
 └──────┬───────┘                           └──────────────┘   `[gpui Apache-2.0]`                 │
        │ keystroke → gpui Action → Editor method  ▲
        └──────────────────────────────────────────┘
```

- **`MultiBuffer`** is the text model even for a plain one-file editor — it just runs in `singleton: true`
  mode (one buffer, one excerpt). This is the crucial unification: *the same editor code renders a scratch
  buffer and a 20-file search-results view.*
- **`DisplayMap`** is a **stack of coordinate transforms.** Each layer takes the text from the layer below
  and produces a slightly different text/coordinate space above it (hide folded regions, expand tabs, insert
  soft-wraps, inject inlay hints and block widgets). The top coordinate space is `DisplayPoint`.
- **`SelectionsCollection`** holds the cursors as **anchors** (positions that survive edits), resolved to
  concrete coordinates against a `DisplaySnapshot` on demand.
- **`EditorElement`** is a gpui `Element`: given a `DisplaySnapshot` + selections it lays out visible lines
  and paints them, and routes mouse/key input back to `Editor` methods.
- **`input.rs`** turns one keystroke into **one edit per selection**, applied to the buffer, which invalidates
  the display stack and moves the anchors.

The rest of this doc walks each stage: **Zed design → Marley today → the gap → the reimplementation on our
stack → sequencing → provenance.**

---

## 1. The `Editor` element — the composer `[Zed-derived]`

### Zed design
`Editor` (`editor.rs`, one ~230-field struct) is the **god-object that owns the editing session** and wires
the pipeline together. It is a gpui `Entity` (not itself an `Element` — it *produces* an `EditorElement`).
The load-bearing fields, stripped of the LSP/git/diagnostics noise:

```rust
pub struct Editor {
    focus_handle: FocusHandle,                 // [gpui Apache-2.0] — focus + key routing
    buffer: Entity<MultiBuffer>,               // the text model (§6)
    display_map: Entity<DisplayMap>,           // the transform stack (§3)
    selections: SelectionsCollection,          // the cursors (§2)
    scroll_manager: ScrollManager,             // viewport + autoscroll
    mode: EditorMode,                          // full | single_line | auto_height | minimap
    // multi-cursor state machines (§2):
    columnar_selection_state: Option<ColumnarSelectionState>,
    add_selections_state: Option<AddSelectionsState>,
    select_next_state: Option<SelectNextState>,
    select_prev_state: Option<SelectNextState>,
    selection_history: SelectionHistory,
    // + autoclose, snippets, IME transaction, blink, style, action registry, …
}
```

Two structural ideas matter for Marley:

- **Everything is a handle to a sub-entity.** `buffer`, `display_map`, `wrap_map` (inside display_map),
  `blink_manager` are all `Entity<T>` — gpui's observable, independently-updatable cells. The `Editor`
  observes them and re-renders on change. This is what lets the display map recompute soft-wrap off the main
  update, and lets two editors share one buffer (splits, the multibuffer).
- **`EditorMode` makes one type serve every surface.** `single_line` (the command input), `auto_height`
  (chat box), `full` (a code pane), `minimap`. The same `Editor` is Warp's prompt *and* a file editor,
  toggled by a mode enum — exactly Marley's need (one editor type for the prompt and the file pane).

The `Editor` never draws. `Editor::render` returns an `EditorElement::new(editor, style)` (§5). Input arrives
as gpui **Actions** registered in `register_actions` (`Editor::move_left`, `Editor::select_next`, …) plus raw
text through `EntityInputHandler` (§5).

### Marley today
`marley_editor::Buffer` (`crates/editor/src/buffer.rs`) is the whole model:

```rust
pub struct Buffer { rope: ropey::Rope, selection: SelectionSet, version: BufferVersion, undo: UndoHistory }
```

Pure, UI-free, single-cursor. The **render half lives in `marley_app`** (gpui) — the editor element built in
M15 (#250) that maps `Buffer` offsets ↔ columns and draws a caret. There is **no `Editor` composer type**:
`marley_app` holds a `Buffer` directly and drives it. Modes don't exist as an enum; the prompt and the file
view are separate code.

### The gap
- No composer object that owns buffer+display+selections+scroll as one unit, so state (scroll, selection
  history, IME transaction, autoclose) has nowhere to live coherently.
- No `EditorMode` — the prompt and editor duplicate wiring instead of sharing one parameterized element.
- No entity/observation split between the model and the display derivation (Marley recomputes render inline).

### Reimplementation on our stack
Introduce **`marley_editor::Editor`** (a new type, likely in `marley_app` or a new `marley_editor_view`
crate since it touches gpui) that owns:
`buffer: Buffer`, `selections: SelectionSet` (§2), a `DisplayMap` (§3), a `ScrollState`, and a `mode:
EditorMode { SingleLine, AutoHeight, Full }`. Keep `Buffer` pure exactly as it is; the composer is the gpui-
aware shell. Model the `Entity<T>` observation with Marley's existing gpui entity model (Marley already uses
gpui `Entity`/`Context`). The command palette / prompt become `mode = SingleLine` instances of the same
element — collapsing today's duplicate input code.

### Sequencing
**Phase A (foundational, do first):** lift the prompt + file view onto one `Editor` composer with an
`EditorMode` enum, still single-cursor. This is pure refactor value (dedupes input handling) and creates the
seam every later phase plugs into. Depends on nothing below; everything below depends on it.

### Provenance
`[Zed-derived]` for the composition (which sub-models it owns, the mode enum, the "produces an element rather
than being one" split). `[gpui Apache-2.0]` for `FocusHandle`, `Entity`/`Context` observation, action
registration. `[Marley-original]` for `Buffer` (unchanged).

---

## 2. `SelectionsCollection` & multi-cursor `[Zed-derived]`

### Zed design
Zed's cursors are **an array of anchor-based selections plus one pending selection:**

```rust
pub struct SelectionsCollection {
    next_selection_id: usize,
    line_mode: bool,
    disjoint: Arc<[Selection<Anchor>]>,       // the committed, non-overlapping cursors
    pending: Option<PendingSelection>,        // the one being dragged/extended right now
    select_mode: SelectMode,                  // Character | Word | Line | …
    is_extending: bool,
}
```

Each `Selection<T>` is `{ id, start: T, end: T, reversed, goal: SelectionGoal }`. Two design decisions carry
the whole feature:

1. **Selections are stored as `Anchor`s, not offsets.** An anchor (from the `text` crate, wrapped by the
   multibuffer `Anchor`) is a position that **auto-adjusts across edits**. So after any edit, every one of the
   N cursors is still valid without manual re-mapping — the bookkeeping that makes multi-cursor *tractable*.
   Concrete coordinates (`Point`, `DisplayPoint`, `usize` offset) are derived on demand:
   `selections.all::<D>(&display_snapshot)` resolves the anchor array into whatever coordinate space `D` the
   caller wants.
2. **`disjoint` is kept sorted and non-overlapping.** A `MutableSelectionsCollection` (the write side) has
   `select`, `select_ranges`, `insert_range`, `move_with`, `move_heads_with`, `replace_cursors_with`; every
   mutation re-sorts and merges overlaps so the invariant holds. `goal` (a `SelectionGoal`) remembers the
   desired column across vertical motion over ragged lines.

**The three multi-cursor gestures**, as state machines on `Editor`:

- **Columnar / block selection** (`ColumnarSelectionState`) — Alt-drag or `add_selection_above/below`
  (`selection.rs`). `AddSelectionsState { groups: Vec<AddSelectionsGroup { above: bool, stack: Vec<usize> }> }`
  grows/shrinks a vertical stack of cursors, one per row at a fixed column; `build_columnar_selection`
  (`selections_collection.rs`) generates them.
- **⌘D "select next match"** (`SelectNextState { query: AhoCorasick, wordwise: bool, done: bool }`,
  `selection.rs::select_next` / `select_next_match_internal`). First ⌘D on a caret selects the *surrounding
  word* and builds an **aho-corasick** automaton from that text; each subsequent ⌘D streams the automaton over
  the buffer bytes from the newest selection, finds the next occurrence, and **adds a cursor** there (or, with
  `replace_newest`, skips). `select_all_matches` runs the automaton over the whole buffer and adds every hit
  at once. `wordwise` gates matches to whole-word boundaries.
- **Tree-sitter node grow/shrink** (`select_larger_syntax_node`, `SelectSyntaxNodeHistory`) — expands each
  selection to its enclosing syntax node, remembering the stack so you can shrink back.

Once N cursors exist, **every edit and motion just maps over the array** (see §4, §5). That is the entire
trick: the input/movement code is written once and applied per-selection.

### Marley today
`marley_editor::SelectionSet { selections: Vec<Selection> }` with `Selection { anchor: CharOffset, head:
CharOffset }`. Deliberately capped at one member: `SelectionSet::single(...)` is the only public constructor;
`from_members` is `pub(crate)` and "the multi-member sort/merge constructor is deferred." Selections are
**raw `CharOffset`s, not anchors** — after an edit, `Buffer::edit` clamps/adjusts the single selection by
hand. `goal` (desired column) is tracked in the movement layer, not on the selection.

### The gap
- **Offsets, not anchors.** With one cursor, manual adjustment is fine; with N, it is unmanageable. This is
  the single biggest structural prerequisite for multi-cursor.
- No disjoint-sort/merge invariant, no pending selection (drag), no `SelectMode`.
- None of the three gestures (columnar, ⌘D, node-grow).

### Reimplementation on our stack
1. **Add a Marley `Anchor`.** A position that survives edits. On ropey we don't get anchors free, so Marley
   introduces a lightweight anchor: either (a) a `(BufferVersion, CharOffset)` pair re-mapped through the
   edit's `BufferDelta` (Marley already records `BufferDelta { char_range, byte_range, new_char_len, … }` on
   every `EditResult` — enough to shift an offset across an edit), or (b) a bias-tagged offset re-anchored in
   `Buffer::edit`. Start with (a): it reuses the delta we already emit. `[Marley-original]` mechanism,
   `[Zed-derived]` intent.
2. **Grow `SelectionSet` into the real collection.** Flip `from_members` public; keep members **sorted +
   non-overlapping** (merge on insert). Store `Selection { anchor: Anchor, head: Anchor, goal: Option<u32> }`.
   Add a `pending: Option<Selection>` for mouse drag. Resolve to concrete coords via `set.resolve(&buffer)`
   → `Vec<Selection<CharOffset>>` — the analogue of Zed's `all::<D>`.
3. **Gestures, cheapest-first.** `add_cursor_above/below` (columnar) is pure geometry over the offset↔column
   map M15 already built — do it first. ⌘D select-next reuses the **`aho-corasick` crate** (MIT, public) over
   `Buffer::text_in_range` bytes — a near-verbatim port of the automaton approach, no Zed code. Node-grow
   waits on tree-sitter (a later syntax subsystem).

### Sequencing
Anchors (1) → collection (2) are a **hard prerequisite pair** and should land together, right after §1's
composer. Then gestures in order: columnar (pure, no deps) → ⌘D (needs aho-corasick only) → node-grow
(needs tree-sitter, defer). Multi-cursor editing/motion falls out for free once §4/§5 map over the array.

### Provenance
`[Zed-derived]` for the anchor-array model, disjoint invariant, and the three gesture state machines.
`[public: aho-corasick is MIT; whole-word matching is standard]` for the ⌘D automaton. `[Marley-original]` for
the delta-replay anchor built on our existing `BufferDelta`.

---

## 3. The `DisplayMap` transform stack `[Zed-derived]` (idea) / `[public]` (pattern)

This is the heart of "a buffer becomes a view," and the most reusable idea in the whole subsystem.

### Zed design
`DisplayMap` (`display_map.rs`) is **a stack of six coordinate transforms**, each a small entity that takes
the layer below and produces a transformed text+coordinate space above. From the module's own docs, bottom to
top:

| Layer | File | Responsibility | Coordinate it introduces |
|---|---|---|---|
| (base) | `multi_buffer` | the raw text (all excerpts) | `MultiBufferOffset` / `MultiBufferPoint` |
| **InlayMap** | `display_map/inlay_map.rs` | inject inlay hints / inline type annotations that aren't in the file | `InlayOffset(MultiBufferOffset)`, `InlayPoint(Point)` |
| **FoldMap** | `display_map/fold_map.rs` | hide folded regions; replace with a placeholder | `FoldPoint(Point)`, `FoldOffset` |
| **TabMap** | `display_map/tab_map.rs` | expand hard tabs to N columns | `TabPoint(Point)` |
| **WrapMap** | `display_map/wrap_map.rs` | soft-wrap long lines at the wrap width | `WrapPoint`, `WrapRow` |
| **BlockMap** | `display_map/block_map.rs` | insert full-width block widgets (diagnostics, excerpt headers, deleted-diff hunks) between lines | `BlockPoint(Point)` |
| **highlights** | `display_map.rs` | overlay background/text highlight ranges (search, selection, semantic tokens) | — |

The top of the stack is **`DisplayPoint(BlockPoint)`** and `DisplayRow(u32)` — the coordinate the element and
movement code work in. Crucially, `DisplaySnapshot` (the immutable per-frame capture) **nests the layer
snapshots** so any coordinate can be converted through the whole stack:

```
DisplaySnapshot → block_snapshot → wrap_snapshot → tab_snapshot → fold_snapshot → inlay_snapshot → buffer
```

**Each layer follows one uniform contract** (this is the pattern worth copying):

- A **`Transform`** enum — a run of text the layer either passes through (`Isomorphic`) or replaces/injects
  (e.g. `Inlay`, a fold placeholder, a soft-wrap break, a block). Transforms are stored in a **`SumTree`**.
- A **`TransformSummary { input: TextSummary, output: TextSummary }`** — `input` = size in the layer below,
  `output` = size exposed above. An inlay has empty `input` (it replaces nothing) and non-empty `output`; a
  fold has non-empty `input` (the hidden text) and tiny `output` (the placeholder). Summing these up the tree
  gives O(log n) coordinate conversion in *either* direction.
- A **`Snapshot`** capturing the layer's state, and conversion methods `fn <A>_point_to_<B>_point()`.
- A **`sync(snapshot, edits) -> (snapshot, edits')`** function: it takes edits in the *lower* layer's
  coordinates and returns edits in *its own* coordinates. Edits chain up the stack, each layer re-summarizing
  only the touched region. An `Edit<T>` here is "a region to invalidate," not a content diff — so an edit low
  in the buffer only recomputes soft-wrap/blocks for the affected rows, not the whole file.

`DisplayMap::new` builds the stack in order (`InlayMap::new` → `FoldMap::new(snapshot)` → `TabMap::new` →
`WrapMap::new` → `BlockMap::new`); a buffer edit flows through the same order via each `sync`.

### Marley today
**Plain render, no transforms.** M15's element maps buffer offset ↔ column directly (#250) and draws lines
1:1. No folds, no soft-wrap, no inlays, no block widgets, no injected highlights beyond a hand-written lexer
for color. `DisplayPoint == buffer point`, trivially.

### The gap
The entire stack. But note the *layers are independent*: Marley can add them one at a time, each a self-
contained transform, without a big-bang rewrite — precisely because Zed's design makes each layer a separate
`sync`-able unit.

### Reimplementation on our stack
Introduce **`marley_editor::display`** — a `DisplayMap` holding an ordered list of transform layers over the
`ropey` buffer. Reproduce the **uniform layer contract** (Transform + input/output summary + Snapshot + sync)
as Marley's own generic trait, e.g.:

```rust
trait DisplayLayer {
    type Coord;                 // this layer's point/offset newtype
    fn sync(&mut self, below: &LowerSnapshot, edits: &[InvalidatedRange]) -> Vec<InvalidatedRange>;
    fn to_lower(&self, p: Self::Coord) -> LowerCoord;
    fn from_lower(&self, p: LowerCoord) -> Self::Coord;
}
```

Marley doesn't have Zed's `sum_tree`; back each layer with either a small Marley summary-tree or (for early
layers with few transforms) a sorted `Vec<Transform>` with binary search — the *contract* is what matters, not
the container. Add layers in **value order for a terminal-first editor:**

1. **TabMap** (expand tabs) — trivial, unblocks correct columns. First.
2. **WrapMap** (soft-wrap) — the highest-value visual feature for a narrow terminal pane; needs the gpui
   `WindowTextSystem` line width, which Marley already calls to lay out lines.
3. **Highlight overlay** — promotes the hand lexer's colors and adds search/selection background highlights as
   a proper injected-range layer.
4. **FoldMap** — code folding; pure once the summary tree exists.
5. **BlockMap** — the enabler for diagnostics rows, excerpt headers (§6), and diff hunks. Do this before the
   multibuffer, since excerpt headers *are* blocks.
6. **InlayMap** — inline hints; last, needs LSP.

### Sequencing
This is the **long pole** and runs in parallel with everything after §2. TabMap + WrapMap first (they make the
plain view correct and comfortable). BlockMap is the gate for §6 (multibuffer headers). FoldMap/InlayMap trail
LSP/tree-sitter. Each layer is independently shippable behind the uniform contract.

### Provenance
The *specific six-layer stack and its order* is `[Zed-derived]`. The underlying **pattern** — a persistent
summarized tree with dual input/output dimensions for O(log n) bidirectional coordinate mapping — is
`[public: rope/piece-table and Zed-published CS]`; Marley implements it independently over `ropey`.
`[gpui Apache-2.0]` for the text-shaping (`LineLayout`, `WindowTextSystem`) the WrapMap consults for widths.

---

## 4. `movement` — motions over display coordinates `[Zed-derived]` / `[public]`

### Zed design
`movement.rs` is a set of **pure functions over `(&DisplaySnapshot, DisplayPoint) -> DisplayPoint`.** The key
architectural choice: **motions operate in *display* space, not buffer space**, so "move down" moves down one
*visual* row (respecting soft-wrap and folds), and "line end" stops at the wrapped-line end. Examples:
`left`/`right`/`up`/`down`, `line_beginning`/`line_end`, `previous_word_start`/`next_word_end`,
`previous_subword_start`/`next_subword_end` (camelCase-aware), `start_of_paragraph`/`end_of_paragraph`,
`start_of_excerpt`/`end_of_excerpt` (multibuffer-aware), and the generic `find_boundary` /
`find_preceding_boundary` that walk chars through a **`CharClassifier`** (word/punct/whitespace classes,
language-configurable) — the single primitive under all word/subword motions. `up`/`down` carry a
`SelectionGoal` so vertical motion over ragged/wrapped lines returns to the intended column.

Motions plug into selections via `MutableSelectionsCollection::move_with` / `move_heads_with` /
`move_cursors_with` — **the motion is written once and mapped over all N cursors.** `clip_point(point, Bias)`
snaps a raw point back onto valid text (past a fold, into a tab, at line end).

### Marley today
`marley_editor::movement` (`crates/editor/src/movement.rs`) already has the right shape — pure functions —
but **over buffer coordinates**, single-cursor: `move_char_left/right`, `move_word_left/right`, `move_up/down`,
`move_line_home/end`, plus `extend_or_move`/`extend_or_go` (the anchor-drop-vs-move logic). Word class is the
simple "alphanumeric or `_`" test (documented as *not* UAX#29); char steps by Unicode scalar, *not* grapheme.

### The gap
- Motions run in **buffer** space, so they can't respect soft-wrap/folds — fine today (no display stack),
  becomes wrong the moment §3 lands.
- Word segmentation is a rough approximation (no subword/camelCase, no UAX#29 word boundaries, no grapheme
  stepping).
- No `CharClassifier` abstraction; no paragraph/bracket/node motions; motions apply to one cursor.

### Reimplementation on our stack
1. **Re-base motions onto the display snapshot** once §3 exists: change signatures to `(&DisplaySnapshot,
   DisplayPoint)`. This is a mechanical lift of the existing pure functions; keep them pure.
2. **Introduce a `CharClassifier`** as the single word/subword primitive and route all word motions through
   `find_boundary`. Upgrade grapheme stepping via the **`unicode-segmentation` crate** (MIT) — the same crate
   Zed uses; the algorithm is the public Unicode standard, not Zed code.
3. **Map over cursors:** add `SelectionSet::move_with(|sel| motion(...))` mirroring Zed's `move_with`, so §2's
   array gets motion for free. `extend_or_move` already encodes the extend-vs-collapse logic — reuse it.

### Sequencing
Grapheme/subword upgrades and the `CharClassifier` can land **now** (pure, no deps, independent value). The
display-space re-base is coupled to §3 (do it as WrapMap lands). `move_with`-over-array is coupled to §2.
Bracket/paragraph/node motions trail tree-sitter.

### Provenance
`[Zed-derived]` for the *display-space* motion model, `CharClassifier`/`find_boundary` factoring, and the
`move_with` fan-out. The segmentation algorithms themselves are `[public: Unicode UAX#29 word, UAX#14 line,
grapheme clusters]` via `unicode-segmentation` (MIT). Marley's existing pure motions are `[Marley-original]`.

---

## 5. The input path — keystrokes → edits `[Zed-derived]` + `[gpui Apache-2.0]`

### Zed design
Two entry points converge on the buffer:

1. **Bound keys → gpui Actions.** A keystroke is matched against the keymap (subsystem 09) to an `Action`;
   `Editor` registered handlers in `register_actions` (`Editor::move_left`, `Editor::select_next`,
   `Editor::backspace`, …). Each handler mutates selections/buffer. `[gpui Apache-2.0]` for the dispatch;
   `[Zed-derived]` for the handler logic.
2. **Raw text / IME → `EntityInputHandler`.** `impl EntityInputHandler for Editor` (`input.rs`) provides
   `replace_text_in_range` (commit text) and `replace_and_mark_text_in_range` (marked/composing IME text) —
   the platform text-input protocol. `[gpui Apache-2.0]` trait; the editor fills it in.

The core is **`Editor::handle_input(text)`** (`input.rs`). For a single character it:
- gathers `selections.all_adjusted(&snapshot)` (all N cursors in buffer coords);
- **loops over every selection**, building one edit per cursor, while handling **autoclose** (typing `(`
  inserts `)`; typing `)` before an auto-inserted `)` steps over it), **auto-surround** (typing `(` with a
  selection wraps it), bracket-pair detection via the language scope, and **linked edits** (rename-linked
  ranges edited together);
- applies all edits to the buffer in one transaction and computes the new selection positions;
- an `ime_transaction` groups IME edits for clean undo.

The takeaway: **input is "compute one edit per selection, apply atomically."** Multi-cursor typing, paste
(one clipboard entry per cursor, or the whole clipboard at each), and deletion are all this same fan-out.

### Marley today
`marley_editor::Buffer::edit(range, text, origin) -> EditResult` is the single mutation primitive: it splices
the rope, records an `EditRecord` for undo, bumps `BufferVersion`, adjusts the one selection, and returns a
`BufferDelta`. Input handling in `marley_app` maps a gpui key event / typed text to a `Buffer::edit` or a
movement call, single-cursor. `EditOrigin { Human, Agent }` is already carried on every edit (the provenance
split Marley wants for agent-driven writes — the exact analogue of Zed's edit source).

### The gap
- One edit, not a per-selection fan-out (blocked on §2's array anyway).
- No autoclose / auto-surround / bracket awareness / linked edits.
- IME: whatever `marley_app` wires today is single-range; no marked-text/composing support surfaced through a
  clean handler.

### Reimplementation on our stack
1. **`Editor::handle_input(text)`** on the §1 composer: `for sel in selections { edits.push(edit_for(sel,
   text)) }`, apply as one grouped transaction (extend `UndoHistory` to group a multi-edit as one undo step —
   Marley's `undo.rs` already records `EditRecord`s; add a group boundary). This makes multi-cursor typing
   fall out of §2 for free.
2. **Autoclose/surround** as a small language-config table (bracket pairs), gated like Zed's — a self-
   contained feature over the same edit primitive. Independent of everything else.
3. **IME** through gpui's input-handler trait (Apache-2.0) — adopt `replace_text_in_range` /
   `replace_and_mark_text_in_range` verbatim as the design; back them with `Buffer::edit`. Marley already
   depends on gpui, so this is available primitive, not a port.
4. Keep `EditOrigin` on every path — human keystrokes = `Human`, agent/brain writes = `Agent` (already true).

### Sequencing
The `handle_input` fan-out is coupled to §2 (needs the array). Autoclose/IME are **independent** and can land
any time (autoclose is pure logic; IME is a gpui trait fill-in). `EditOrigin` needs no work — it exists.

### Provenance
`[gpui Apache-2.0]` for action dispatch, `FocusHandle`, and the `EntityInputHandler`/IME traits. `[Zed-derived]`
for the per-selection fan-out, autoclose/surround/linked-edit logic, and IME-transaction undo grouping.
`[Marley-original]` for `Buffer::edit`/`UndoHistory`/`EditOrigin` (unchanged/extended).

### `EditorElement` — the paint half `[Zed-derived]` + `[gpui Apache-2.0]`
`EditorElement { editor: Entity<Editor>, style, split_side }` implements the gpui **`Element`** trait
(`element.rs`): `request_layout` (reserve space) → `prepaint` (take a `DisplaySnapshot`, compute the visible
`DisplayRow` range from scroll, lay out each line via the gpui `WindowTextSystem` into a `LineWithInvisibles`,
build a `PositionMap` for hit-testing, size the gutter) → `paint` (draw text runs, selections, cursors, gutter,
block widgets; register mouse/scroll handlers). `EditorLayout` caches the frame's computed geometry. The
**`Element` trait, `LineLayout`, text shaping, and hit-testing are all `[gpui Apache-2.0]`**; the *editor-
specific layout algorithm* (mapping display rows to y, painting multi-cursor, gutter/fold indicators) is
`[Zed-derived]`. Marley's M15 element already does the plain version of this over gpui — it grows a
`DisplaySnapshot` input (§3) and multi-cursor painting (§2), not a new architecture.

---

## 6. The multibuffer — the capstone `[Zed-derived]`

### Zed design
`MultiBuffer` (`multi_buffer` crate) is **Zed's signature capability: editable excerpts from N files stitched
into one editor.** It backs multi-file search results, find-all-references, project-wide diagnostics, and the
git diff view — each a single editor over slices of many buffers, where **editing an excerpt writes through to
the real file.**

```rust
pub struct MultiBuffer {
    snapshot: RefCell<MultiBufferSnapshot>,
    buffers: BTreeMap<BufferId, BufferState>,   // the real source buffers
    diffs:   HashMap<BufferId, DiffState>,      // per-buffer git diff
    singleton: bool,                            // one buffer + one excerpt = a plain editor
    history: History,                           // cross-buffer undo
    …
}
pub struct MultiBufferSnapshot {
    excerpts: SumTree<Excerpt>,                 // the stitched slices, in a summarized tree
    diff_transforms: SumTree<DiffTransform>,    // expand deleted-hunk rows for the diff view
    …
}
pub(crate) struct Excerpt {
    path_key: PathKey, buffer_id: BufferId,
    range: ExcerptRange<text::Anchor>,          // which slice of the source buffer
    text_summary: TextSummary, has_trailing_newline: bool, …
}
```

Two ideas make it work:

1. **Excerpts live in a `SumTree`**, so the multibuffer is itself a summarized sequence: converting a
   multibuffer offset to `(buffer, local offset)` is an O(log n) seek, exactly like the rope. The `DisplayMap`
   (§3) sits directly on top of `MultiBufferSnapshot` — it neither knows nor cares whether there's one
   excerpt or a thousand.
2. **Anchors compose.** The multibuffer `Anchor` is an enum `Min | Excerpt(ExcerptAnchor) | Max`, where
   `ExcerptAnchor { text_anchor: text::Anchor, path, diff_base_anchor }` = **a per-buffer anchor plus which
   excerpt it lives in.** So a selection in a multibuffer is still edit-stable, and edits route to the correct
   source buffer. Excerpt headers ("── src/foo.rs ──") are `BlockMap` blocks (§3); expanding an excerpt's
   context grows its `ExcerptRange`.

The unification is the masterstroke: **`singleton: true` — one buffer, one excerpt — is a plain editor.** The
same `Editor`, `DisplayMap`, `SelectionsCollection`, and `EditorElement` render both. There is no separate
"single file" path.

### Marley today
None. `marley_editor::Buffer` is one ropey rope = one file. There is no excerpt concept, no cross-file view.

### The gap
Everything — but the *shape* to build toward is clear, and it depends on the earlier stages: excerpt headers
need BlockMap (§3.5), edit-stable cross-file selections need Marley anchors (§2.1), and the whole thing needs
the composer (§1) and display stack (§3) it sits under.

### Reimplementation on our stack
Introduce **`marley_editor::MultiBuffer`** as a sequence of `Excerpt { buffer_id, range: Range<Anchor>,
summary }` over Marley's source `Buffer`s. Reuse the §3 summary-tree machinery (excerpts are just another
summarized sequence). Make Marley's `Anchor` (§2.1) compose the same way: `ExcerptAnchor { buffer_id,
text_anchor }`. Ship it in the same **`singleton` unification** — a normal file editor is a one-excerpt
multibuffer — so all the earlier work (composer, selections, display, input, element) serves it unchanged.
Excerpt headers are §3.5 BlockMap blocks. This lights up Marley's real targets: **agent/brain surfaces**
(review N files' worth of proposed edits in one pane), project-wide search results, and diagnostics — the
exact "custom panel that reads/writes across a workspace" the brain wants.

### Sequencing
**Last.** Hard-depends on: §1 composer, §2 anchors, §3 display stack **including BlockMap** (headers), §5
input. Do it once those are solid; attempting it earlier means rebuilding it. The `singleton`-first strategy
lets Marley ship the unification (plain editor *is* a one-excerpt multibuffer) before any multi-file UI, de-
risking it.

### Provenance
`[Zed-derived]` for the excerpt-tree model, composed excerpt anchors, the diff-transform expansion, and the
singleton unification. The underlying summarized-sequence container is the same `[public]` pattern as §3.

---

## 7. Roadmap — consolidated sequencing

The dependency order (each phase unblocks the next; independent items noted):

1. **§1 Composer + `EditorMode`** — lift prompt + file view onto one `Editor`. *Prereq for all.*
2. **§2.1 Marley `Anchor`** (delta-replay over existing `BufferDelta`) + **§2.2 real `SelectionSet`** (sorted,
   merged, `pending`). *Prereq for multi-cursor edit/motion.*
3. **§4 pure upgrades (parallel, no deps):** `CharClassifier`, grapheme + subword segmentation via
   `unicode-segmentation`.
4. **§3 display stack:** TabMap → WrapMap (highest visual value) → highlight overlay → **BlockMap** → FoldMap
   → InlayMap. *Runs in parallel with 2–3; each layer independently shippable.*
5. **§2.3 gestures:** columnar (pure) → ⌘D (aho-corasick) → node-grow (needs tree-sitter, defer).
6. **§4 re-base motions to display space** (couples to WrapMap) + **§5 `handle_input` fan-out** (couples to §2)
   + autoclose/IME (independent).
7. **§6 MultiBuffer** — `singleton`-first, then multi-file. *Needs 1, 2, 3-incl-BlockMap, 5.*

Terminal-first bias: **TabMap + WrapMap + the composer + multi-cursor** deliver the felt "real editor" jump;
folds/inlays/multibuffer trail the LSP/tree-sitter subsystems.

---

## 8. Key files (Zed clone — `…/scratchpad/zed-src/crates/`)

- `editor/src/editor.rs` — the `Editor` composer struct (§1), action registration, multi-cursor state fields.
- `editor/src/selections_collection.rs` — `SelectionsCollection` / `MutableSelectionsCollection` (§2), the
  anchor-array + pending model, `move_with`/`select_ranges`/`build_columnar_selection`.
- `editor/src/selection.rs` — the gestures: `add_selection_above/below`, `select_next` (⌘D, `SelectNextState`
  + aho-corasick), `select_all_matches`, `select_larger_syntax_node`.
- `editor/src/display_map.rs` — `DisplayMap`, `DisplaySnapshot` (the nested layer snapshots), `DisplayPoint`,
  `DisplayRow`, the layer-contract module docs (§3).
- `editor/src/display_map/{inlay_map,fold_map,tab_map,wrap_map,block_map}.rs` — the six transforms; each has
  its `Transform`/`TransformSummary`/`Snapshot`/`sync`.
- `editor/src/movement.rs` — pure display-space motions + `CharClassifier`/`find_boundary` (§4).
- `editor/src/input.rs` — `handle_input` (per-selection edit fan-out), `impl EntityInputHandler` (IME) (§5).
- `editor/src/element.rs` — `EditorElement` (gpui `Element`), `EditorLayout`, `PositionMap`,
  `LineWithInvisibles` (§5 paint half).
- `multi_buffer/src/multi_buffer.rs` — `MultiBuffer`, `MultiBufferSnapshot`, `Excerpt`, `ExcerptRange`,
  `singleton` (§6).
- `multi_buffer/src/anchor.rs` — the composed `Anchor` enum (`Min | Excerpt(ExcerptAnchor) | Max`) (§2.1/§6).

**Marley today:** `crates/editor/src/{buffer,selection,movement,undo,types}.rs` (`marley_editor`, MIT-adjacent,
pure) — the single-cursor baseline this doc extends.

---

## 9. The legal boundary (summary)

- **Reusable as-designed, no copyleft risk:** the gpui `Element`/`FocusHandle`/action/`EntityInputHandler`/
  `WindowTextSystem` primitives (`[gpui Apache-2.0]`); the summarized-tree + dual-dimension coordinate-mapping
  *pattern* and the Unicode segmentation *algorithms* (`[public]`); Marley's existing pure `Buffer` (MIT
  ropey).
- **`[Zed-derived]` — re-derive, don't copy; keep inside the (intended-GPL) editor layer:** the `Editor`
  decomposition, the anchor-array selection model + three gesture state machines, the *specific* six-layer
  display stack, display-space motions, the per-selection input fan-out + autoclose, the `EditorElement`
  layout algorithm, and the whole multibuffer/excerpt-anchor design.
- **The hard rule:** none of the `[Zed-derived]` material may leak into the **brain/agent layer** — it inherits
  GPL and becomes unsellable. The brain talks to the editor across a clean interface (issue `Buffer::edit`
  with `EditOrigin::Agent`, read via `text_in_range`); it never links the editor's GPL internals. Legal review
  before release, per `docs/zed_architecture/README.md`.
