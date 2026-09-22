# terminal text selection model — Notes

- **Forge ticket:** #43 `9a6582b5-76c3-4bde-9c2a-258ce5469d1c`
- **AAR:** `98d605b0-1892-4622-9d1b-69280eba076d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-043-text-selection.md
- **Pipeline spec:** text-selection.spec.md

## Phase 1 — Plan
- **Request:** forge #43 (M1.G "Block Workflows & Selection" seq-1, auto-approved) — the pure
  selection model, the FOUNDATION for copy (#44).
- **Classification / tier:** work pipeline, `feature`, a PURE model (text_selection.rs — cov/MSI 100)
  + a SHIM (PaneState field + mouse handlers + highlight render). marley_app only.
- **Discovery (§18):**
  - The render walks a linear ROW sequence — `content_rows` (app.rs:264) = per block `1 +
    output_styled().len()`, + the prompt. Each row's text = a `StyledLine` (`Vec<StyledRun{text}>`)
    joined (or the command header string). So the selection maps over `&[String]` (plain row text).
  - `PaneState<S>` (workspace.rs) = session/buffer/caret/history/pty_size/viewport/scroll_remainder →
    add `selection: Option<Selection>` (init None in `PaneState::new`, line 153).
  - Distinct from `crates/editor/src/selection.rs` (#28 — the prompt buffer's caret Selection); this
    is a NEW terminal-output selection in marley_app.
  - Deps #32 (viewport rows + top offset for the coord map) + #34 (cell metric).
- **Decisions:** D1–D4 in the spec (char cols; clamp; per-pane; distinct from editor::Selection).
- **Open questions for Design:** `row_slice` via `chars().collect()` (simple, char-safe) vs
  `char_indices` (zero-alloc) — lean collect for clarity; whether `GridPos` derives `Ord` (yes) so
  `normalized` is a simple compare; the highlight color (accent low-opacity vs a new selection role —
  lean accent-opacity, no new token).
- **AAR id:** `98d605b0-1892-4622-9d1b-69280eba076d`.

## Phase 2 — Design

### KEY RE-SCOPE (avoids dead-code): `row_selection` is #43's core, `selected_text` → #44
The #43 SHIM is the drag + HIGHLIGHT; the highlight needs, PER rendered row, which columns are
selected — that's `row_selection`. `selected_text` (the joined copy text) is only USED by #44's copy
shim, so shipping it in #43 would be dead-code (only tests would call it — the #41 lesson). So #43
ships `row_selection` (used by the highlight render → live + tested); #44 adds `selected_text` (built
on the same geometry, used by the copy shim).

### PURE — `crates/marley_app/src/text_selection.rs` (NEW, gpui-free)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridPos { pub row: usize, pub col: usize }   // col in CHARS; Ord = (row, col) lexicographic

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection { pub anchor: GridPos, pub head: GridPos }

impl Selection {
    /// (start, end) with start <= end — a backward drag reads the same as forward.
    pub fn normalized(self) -> (GridPos, GridPos) {
        if self.anchor <= self.head { (self.anchor, self.head) } else { (self.head, self.anchor) }
    }
}

/// The selected char span `[from, to)` on `row` (of `row_len` chars), or `None` if `row` is outside
/// the selection. The first row starts at `start.col`, the last ends at `end.col`, middle rows are
/// whole; clamped to `row_len`. Used by the highlight render AND (in #44) `selected_text`.
pub fn row_selection(sel: Selection, row: usize, row_len: usize) -> Option<(usize, usize)> {
    let (start, end) = sel.normalized();
    if row < start.row || row > end.row {
        return None;
    }
    let from = if row == start.row { start.col } else { 0 };
    let to = if row == end.row { end.col } else { row_len };
    Some((from.min(row_len), to.min(row_len)))
}
```

### SHIM — `crates/marley_app/src/workspace.rs` + `app.rs`
- `PaneState<S>` gains `selection: Option<Selection>` (`PaneState::new` → `None`).
- app.rs: a mouse-down on a pane maps the event `(x, y)` → `GridPos` (`row = viewport_top +
  (y ÷ cell.h)`, `col = x ÷ cell.w` via the #34 cell metric) → `selection = Some(Selection{anchor,
  head: anchor})`; drag updates `head`; each rendered row calls `row_selection(sel, row, row_len)` →
  paints a highlight bg (`accent` low-opacity) over `[from, to)`; the selection clears on a
  click-outside / a new command (`on_submit`).

### File manifest
- A `crates/marley_app/src/text_selection.rs` — GridPos/Selection/normalized/row_selection + tests.
- M `crates/marley_app/src/lib.rs` — `mod text_selection;`.
- M `crates/marley_app/src/workspace.rs` — `PaneState.selection` + init.
- M `crates/marley_app/src/app.rs` — mouse→GridPos, drag, the highlight via `row_selection`, clear.
- M `docs/specs/SPEC-app-shell.spec.md` — the selection clause (R47) + Mutation-Targets. CHANGELOG; arch.

### Mutation Targets
- `normalized` — the `anchor <= head` compare (a backward-selection test → same pair kills `>`).
- `row_selection` — the `row < start.row || row > end.row` out-of-range guard (each disjunct); the
  `row == start.row` (from = start.col else 0); the `row == end.row` (to = end.col else row_len); the
  `.min(row_len)` clamps. Killed by a multi-row selection asserting each row's exact span + an
  out-of-range row (None) + a clamp fixture.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `normalized_orders_backward` — `Selection{(2,2),(0,1)}.normalized() == ((0,1),(2,2))`; a forward one stays | unit |
| REQ-002 | `row_selection_spans` — sel `(0,1)..(2,2)`, row_len 3: row 0 → `(1,3)`, row 1 → `(0,3)`, row 2 → `(0,2)`; single-row sel `(0,1)..(0,4)` row 0 → `(1,4)` | unit |
| REQ-003 | `row_selection_out_of_range_and_clamp` — a row `< start.row` or `> end.row` → `None`; `end.col`/`from` past `row_len` clamped; a BACKWARD sel gives the same spans as forward | unit |
| REQ-004 | the drag-highlight render | shim + masked visual — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs mouse→GridPos map + the highlight paint (shim exclude; needs a live window +
mouse).

### Risks / decisions
- D-2.1 `row_selection` (not `selected_text`) is #43's surface — it's what the highlight USES, so it's
  live + tested; `selected_text` (the `\n`-joined copy text, built on `row_selection`) is #44's, used
  by the copy shim. This keeps each ticket's pure fn non-dead. The spec REQ is updated accordingly.
- D-2.2 `GridPos` derives `Ord` → `normalized` is a one-line compare (no hand-rolled tuple order to
  get wrong). D-2.3 char cols + `.min(row_len)` clamps → an out-of-range mouse drag never panics.
- Adding `PaneState.selection` ripples only `PaneState::new` (one init) — the workspace tests
  construct via `new`, so they stay green.

## Phase 3 — Implement
- **Built (per manifest):** `text_selection.rs` (NEW, gpui-free) — `GridPos{row,col}` (Ord),
  `Selection{anchor,head}` + `normalized`, `row_selection(sel,row,row_len) -> Option<(from,to)>`;
  `lib.rs` `mod text_selection;`; `workspace.rs` `PaneState.selection: Option<Selection>` + init None;
  `app.rs` — a `pane_grid_pos(position,rect,cell_w,cell_h,state)` shim (window pos → GridPos, inverse
  of the render), on_mouse_down starts the selection, on_mouse_move (left held) extends `head`, the
  output-row render highlights via `row_selection` (accent @ 0.3 alpha), on_submit clears it.
  SPEC-app-shell R47 + row 47 + Mutation-Targets; CHANGELOG.
- **Deviations from design:** the highlight is per-ROW (tints a row when `row_selection` returns a
  non-empty span) rather than per-CHAR span for the first cut — simpler over the existing color-run
  spans; the exact per-char highlight is a later refinement (`row_selection` already returns the span,
  so the pure surface is unchanged). Command-header rows aren't highlighted yet (output rows are the
  main selectable content) — noted.
- **Verification at this phase:** `cargo check --workspace` 0 errors; fmt; clippy `-D warnings` 0;
  docs gate 0; 98 marley lib tests pass; `row_selection` is USED by the highlight (not dead). The
  `row_selection` + `normalized` unit tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a verbatim geometry probe [8/8] + a real scoped cargo-mutants [isolated dir] + the
  workspace test suite). Verdict: **GREEN — the pure model is correct; proceed to Validate.**
- **row_selection MSI CONFIRMED:** 16 mutants → 15 caught / 1 unviable (`normalized → Default` —
  GridPos has no Default) / **0 survivors** with the 3 planned tests. Geometry correct (3-row spans,
  single-row both-branches, clamp, backward==forward). **Ord field-order right** — `row` declared
  before `col` → `GridPos{0,9} < GridPos{1,0}` (row dominates the normalize, the load-bearing anchor).
- **Findings table (all LOW):**
  | # | Sev | Finding | Verdict | Carry-forward |
  |---|---|---|---|---|
  | F1 | LOW (Phase-4) | The `from.min(row_len)`/`to.min` clamp is GATE-INVISIBLE — cargo-mutants doesn't mutate `.min`, and coverage doesn't force it; the design's REQ-003 clamp fixture is correctness-only. | REAL (write it deliberately) | P4 includes the clamp fixtures (end.col=99→row_len; from>row_len→empty span). #44's `selected_text` slices `from..to` on real char-vecs and DEPENDS on this clamp to not panic. Same family as #42's structural-test-since-MSI-can't-force. |
  | F2 | LOW (shim) | `pane_grid_pos` divides by a uniform `cell_h`, but command headers are taller (border_t + padding), so the highlight can DRIFT a row once headers are on-screen. | ACCEPTED (shim, masked) | A first-cut coord limit (precise per-row Y-offsets need the render to track them — a later refinement); chad's visual check should drag across a header. Noted. |
  | F3 | LOW (shim) | Whole-row highlight (not per-`[from,to)` span) + a zero-width `Some(selection)` on every mouse-down. | ACCEPTED (Phase-3 deviations, guarded by `to>from`) | The per-char span highlight is a later refinement; the zero-width selection is harmless. |
- **Verified CLEAN:** `cargo check --workspace` 0 err; the only `PaneState{…}` literal is in `::new`
  (all other construction via `new`) → workspace tests 21 pass, no ripple break; `row_selection` is
  LIVE (app.rs highlight, not dead); `selected_text` correctly deferred to #44 (doc-only here);
  `pane_grid_pos` guards (`.max(0.0)` + `cell>0.0`, float→usize saturates) → no panic; no unwrap/slice
  in the pure module.
- **No code fix** — the pure surface is correct. F1 is the Phase-4 clamp fixtures.

## Phase 4 — Validate
- **Tests added (text_selection.rs):** `normalized_orders_backward` (forward stays; backward flips;
  ROW-dominates-COL kills a field-order regression); `row_selection_spans` (3-row per-row spans +
  single-row both-branches); `row_selection_out_of_range_and_clamp` (before/after → None; `to`/`from`
  clamp — the F1 gate-invisible fixtures; backward==forward).
- **Runs (actual):** `cargo nextest run -p marley` → 104 passed (all 3 new PASS);
  `cargo nextest run --workspace` → 519 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **15 caught /
  0 missed → MSI 100.0%** (row_selection + normalized; the 1 `Default` mutant unviable). Receipt
  written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` — the text-selection bullet.
  SPEC-app-shell R47 at implement.
- **Knowledge captured:** no new prevention rule — the LOW-1 (`.min` clamp gate-invisible → write the
  fixture deliberately) is the same family as the already-captured `no-default-struct-return` +
  `sanitizer-loop` rules; noted in the ledger. aar-submit `completed` (score 5). Win: the
  `row_selection`-not-`selected_text` re-scope (design) kept #43's pure surface USED by its own
  highlight shim (no dead-code, the #41 lesson applied proactively), and the `GridPos` field order
  (row before col) makes the derived `Ord` correct — the critic confirmed both.
- **Ticket:** forge #43 → done; local doc → closed/; pipeline pair archived. 1 of 6 in M1.G.
