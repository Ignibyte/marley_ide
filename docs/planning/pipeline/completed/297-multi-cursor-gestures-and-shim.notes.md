# 297 — multi-cursor gestures + the app shim · notes

- **Spec:** ./297-multi-cursor-gestures-and-shim.spec.md
- **Forge ticket:** #297 `a8230892-ec68-411c-946e-332e8542c4fd` · **AAR:** `8cecc7af-5a5d-42fc-8bfd-9b81ee7e9feb`
- **Dep:** #296 (the pure multi-cursor core) — SHIPPED at `135b439`.

## Phase 1 — Plan

### What this is
The ticket that makes #296 **real**. The crate can hold N cursors, edit at all of them in one undo unit,
and say where they land — but **no key in Marley can create a second cursor**, and the app surface still
carries a single `caret` + `anchor` per open file. #296's D6 moved the whole app shim here on purpose, so
that the shim lands in the ticket that can actually *drive* it. Two halves, one slice.

### Grounding (an Explore scout mapped the surface; every claim below was then re-verified against the code)
- **State:** `OpenFile{caret: CharOffset, anchor: Option<CharOffset>}` — `editor_surface.rs:21,24`. Container
  `EditorSurface{files, active}` (`:99`). **Caret/anchor are NOT persisted** (`grid_layout` writes paths +
  active index only) → no settings migration.
- **45 production call sites** (33 `app.rs` + 12 `headless_drive.rs`), bucketed:
  - **A — read-only (6)**: → `primary().head()`. Mechanical.
  - **B — motion (4+)**: the ⌘-arrow block (`:6289`), the Left/Right/Home/End/Word/Up/Down ladder (`:6560`),
    F8 (`:4662`), `open_file_at` (`:2737`). Each becomes "move every cursor".
  - **C — edit (12)**: find-replace, paste/cut, Tab indent/dedent, Enter auto-indent, the typing default arm
    (`apply_editor_key`, `input.rs:250`, single-caret by construction). Each becomes `edit_at_selections`…
    **except Tab** (see D6).
  - **D — render (3)**: `EditorDraw.caret` (`:7322`), the selection band (`:3079`), IME `bounds_for_range`
    (`:5855`).
  - **E — structural (9)**: click/drag, undo/redo, ⌘A, ⌘D, efind-select, `selected_text_range`, Esc. **This
    is where the product decisions are** — not a `primary()` shim problem.
- **Render:** the caret is drawn **per row** (`app.rs:3273-3287`), so N carets is mechanical (each row
  filters the carets on it). The selection highlight is **not** a separate primitive — it is baked into the
  syntax-span list by the pure `code_view::row_selection_cols` (`:135`) + `styled_slices_with_marks`
  (`:229`, currently one `Option<Range>`).
- **Gates:** `app.rs` is coverage-excluded and carries 130 justified `mutants::skip` shims.
  **`code_view.rs` is excluded from NOTHING**, and the mutation gate has zero file exclusions → the
  N-range highlight fns face the full 100/100 bar. *Keep new logic pure and put it where it can be tested;
  keep gpui in the skip-annotated shims.*
- **Keys:** ⌘⌥↑/↓ is the global pane-focus binding (`keymap.rs:189-205`, M6 #131) — but `Keymap::action_for`
  already ranks a context-scoped row above a global one for the same chord, the precedent shipped for ⌘D
  (`keymap.rs:244-252`), ⌘F and ⌘A, and proven in `headless_drive.rs:140`. **Two Editor-scoped rows, no
  modifier-ladder surgery.**
- **Goal column:** does not exist (`movement.rs:115` — "No goal-column memory (v1)"). #296's spec assigned it
  here, and it is load-bearing — see D3.

### The two claims I verified myself rather than inherit
1. **The undo-fidelity gap is REAL.** `SelSnapshot{anchor, caret}` (`undo.rs:24-31`) is one pair; `undo()`
   returns `HistoryMove{caret, selection}`; the app applies `*c = mv.caret; *a = mv.ranged_anchor();`
   (`app.rs:4573-4577`). The group loop inverts **all N** records → **the text is fully correct** after ⌘Z;
   exactly **one cursor** survives. Sizing for the fix: 34 `SelSnapshot` mentions across 5 files, and both
   `SelSnapshot` and `HistoryMove` are `Copy` (a `SelectionSet` is not) → that derive ripples.
   **→ Surfaced as the OPEN DECISION for Phase 2, not resolved here** (planning does not make design calls).
2. **The `open_file_at` stale-anchor bug is REAL.** `app.rs:2736-2737` writes the caret and never the anchor,
   so a file:line click-through with a live selection paints a spurious selection from the old anchor. The
   conversion fixes it structurally; REQ-008 pins it.

### Classification
Work pipeline, `feature`, M19. **One shippable slice** — it looks large, but it cannot be halved: the shim
without the gestures is unreachable code, and the gestures without the shim have nothing to act on. The
things that *could* have bloated it are already out: ⌘D-adds-cursor (#298), the multi-cursor-aware ops
(#299/#300/#303), and ⌘A / the IME's single-range trait (structurally single by definition).

### Risks
- **The 45-site refactor is the bulk of the risk**, and buckets A/D are mechanical while bucket E needs real
  decisions. Design must bucket them explicitly rather than sweep them.
- **The E0499 borrow shape**: `active_buffer_caret_anchor_mut` exists *specifically* to dodge a double-`&mut`
  (17 of 33 app sites use it). Its `SelectionSet` replacement must keep dodging it.
- **The goal column is the subtlest thing here** — get it wrong and two cursors merge on the first short line
  and never return. REQ-002 exists to pin exactly that.
- **A green gate is necessary, not sufficient** (PR-claude-a-fuzzer-proves-the-transform-not-the-invariant-001,
  earned on #296 last commit): spawn the adversarial critic at inspect and WAIT for it.

---

## Phase 2 — Design

### The decision that reframed the whole ticket
Grounding turned up one fact that changes the architecture: **the app never reads `buffer.selection()` and
never calls `buffer.set_selection()`** — zero hits in `crates/marley_app/src`. The `Buffer` has owned a
`SelectionSet` all along, and the app has been ignoring it and keeping a *parallel* `caret` + `anchor` in
`OpenFile`. Two sources of truth for the cursor, one of them vestigial.

So #297 does **not** "add a `SelectionSet` to `OpenFile`". It **deletes `OpenFile.caret` and
`OpenFile.anchor` and makes the `Buffer` the single owner** (§14 single-owner shared types). Three things
fall out for free:

1. **The E0499 problem dissolves.** `active_buffer_caret_anchor_mut` — the 3-way `&mut` borrow that exists
   *solely* to hand out `&mut Buffer` and `&mut caret` at once (17 of 33 app sites) — has nothing left to
   dodge, because the caret now lives *inside* the buffer. It becomes `active_buffer_mut() -> &mut Buffer`.
2. **`OpenFile::new` / `from_files` / `reload_active` get simpler** — `Buffer::from_text` already seeds
   `single(caret(0))`, so the caret seeding those three sites do by hand disappears.
3. **The `open_file_at` stale-anchor bug (REQ-008) cannot be re-introduced**, because there is no longer a
   free-floating anchor to forget.

This is the #296 lesson applied one level up: *duplicate state that nothing enforces is a bug waiting to
happen.* We're deleting the duplicate rather than teaching it to sync.

### D7 — the undo decision: **(A), grow the snapshot** — and it makes the code SMALLER
Sized against the real code (every `SelSnapshot` consumer: `undo.rs` struct + 2 methods + the coalesce
guard + tests; `buffer.rs` `HistoryMove` + 5 fns + tests; `find.rs` ×1; `app.rs` ×4).

**`SelSnapshot` is DELETED. `UndoGroup.sel_before/sel_after` and `HistoryMove` carry a `SelectionSet`.**

| | before | after |
|---|---|---|
| `undo.rs` | `SelSnapshot{anchor, caret}`; `sel_before/after: Option<SelSnapshot>` | type gone; `Option<SelectionSet>` |
| `buffer.rs` | `HistoryMove{caret, selection: Option<SelSnapshot>}` + `ranged_anchor()` + `snapshot_of()` | `HistoryMove{selections: SelectionSet}` — **`ranged_anchor` and `snapshot_of` both DELETED** |
| `app.rs` undo site | `*c = mv.caret; *a = mv.ranged_anchor();` | `b.set_selection(mv.selections)` |

Two functions #296 was *forced* to write purely to bridge the impedance mismatch —
`Buffer::snapshot_of` (SelectionSet → one pair) and `HistoryMove::ranged_anchor` (the "collapsed ⇒ no
selection" rule) — **evaporate**, because a `SelectionSet` expresses both natively (a caret *is*
`anchor == head`). Option (B) would instead have forced *new* lossy `SelectionSet → (caret, anchor)` code at
the undo site. **(A) is both more correct and less code.** The only real cost is the `Copy` derive on
`SelSnapshot`/`HistoryMove` (a `Vec`-backed set can't be `Copy`) — bounded, because `UndoGroup` already
holds a `Vec` and so was never `Copy`; `undo()`/`redo()` clone the set once per history move (N is tiny).
`undo.rs`'s coalesce guard keys off `sel_before.is_none()` and is unaffected.

### D8 — the goal column lives ON `Selection`, and its reset is therefore FREE
`Selection` gains `goal_col: Option<usize>` (a **char** column, matching `move_up`/`move_down`'s existing
semantics; still `Copy`, still `PartialEq`/`Eq`).

Rejected alternatives: a parallel `Vec<usize>` beside the set (drifts — the exact bug class #296 just
taught us); a single goal on `OpenFile` (**wrong**: ⌘-click can put cursors in *different* columns, so the
goal is intrinsically per-cursor).

The elegance is in the reset. The goal must survive a *vertical* motion and reset on *everything else*, and
co-locating it on `Selection` makes that automatic: **every existing constructor (`Selection::new`,
`Selection::caret`) sets `goal_col: None`**, so a horizontal motion, an edit, a click, a merge — all of
which build fresh `Selection`s — reset it *by default*. **You cannot forget to reset it; only the vertical
path deliberately carries it forward.** Named reset points, all free: horizontal motion, any edit
(`selections_after_multi_edit` emits fresh carets), click/⌘-click, Esc, and a *merged* member (a union
range has no vertical goal — an honest truth, not a fudge).

**One deliberate change to #296's code:** `Buffer::set_selection`'s clamp currently rebuilds via
`Selection::new(...)`, which would silently drop the goal. It must **preserve** it — the goal is the
*desired* column and is meaningful regardless of where the clamp actually landed. (`edit_at_selections`'s
incoming clamp does NOT need to preserve it: its output is fresh carets with no goal, which is correct.)

### D9 — "primary" = **member 0, the topmost cursor** — no new invariant
The forge ticket said "the last-added", but `from_selections` **sorts**, so insertion order is not
recoverable from an ordered set. The alternative — a `primary: usize` index on `SelectionSet` — means a NEW
invariant (`primary < len`) that must be maintained through every sort and merge. **#296's `SelectionSet`
just took five critic findings to make solid, and its whole lesson is that an invariant you don't enforce
at every entry point is fiction. I am not re-opening that type for a cosmetic Esc target.**

`primary()` = `selections[0]` — safe by #296's non-empty invariant. **Stated honestly:** ⌘⌥↓ then Esc
returns you to where you started (correct — you added downward); ⌘⌥↑ then Esc leaves you at the *topmost*
cursor rather than your origin. A known, documented v1 deviation, not a silent one.

### D10 — reuse the shipped motion helpers per-member; only VERTICAL is new
`extend_or_move` / `extend_or_go` already implement the shift-extend and collapse-to-edge semantics, over an
`(Option<anchor>, caret)` pair. A `Selection` maps to that pair exactly (`is_caret() → None`), so the N-cursor
motion **reuses them verbatim per member**. The only genuinely new motion is the goal-aware vertical:
`move_up`/`move_down` already compute `(row, col)` then `col.min(target_len)` — the goal version substitutes
`goal.unwrap_or(col)`, and the existing `move_up`/`move_down` become one-line calls into it (reuse, not a
fork).

### D11 — `code_view`: `row_selection_cols` needs NO change; only `styled_slices_with_marks` does
`row_selection_cols(sel_start, sel_end, row_start, row_nchars, layout)` takes plain `usize`s — so N
selections just call it N times. Add one thin pure fan-out (`row_selection_cols_all`) so the loop lives
where it can be tested rather than in coverage-excluded `app.rs`.
`styled_slices_with_marks(syntax, selection: Option<Range>, marks)` genuinely must change: it derives cut
points from the selection and labels each slice `selected`. It takes `selections: &[Range<usize>]`; cuts
come from all of them; `selected` = "inside ANY". Existing callers pass a 0-or-1 element slice. **Changed,
not duplicated** — `code_view.rs` is excluded from no gate, so both fns face cov 100 / MSI 100.

### Architecture / §20
Fits the existing shape: pure logic in `crates/editor` (cov/MSI 100), pure view math in `code_view.rs`
(cov/MSI 100), gpui confined to `app.rs`'s `mutants::skip`-annotated render/input shims. **§20 confirmed:**
the reference is Zed/VS Code multi-cursor *behavior* — add-cursor-above/below, ⌘-click add/remove, Esc
collapse, type-at-every-cursor, one undo unit, and the goal column that survives a short line. Every one of
those is reimplemented here from observed behavior on Marley's own `SelectionSet` + the shipped
`Buffer`/undo machinery; the keymap collision is resolved with Marley's *own* shipped context-shadowing
mechanism, not the reference's. No Zed or VS Code source read or translated.

### File manifest

**`crates/editor/` (pure — cov/MSI 100)**
| File | Change |
|---|---|
| `selection.rs` | `Selection` gains `goal_col: Option<usize>` + `with_goal`/`goal_col()`; `SelectionSet::primary()` (member 0); the merge drops the goal on a *merged* member. |
| `undo.rs` | **`SelSnapshot` DELETED.** `UndoGroup.sel_before/sel_after: Option<SelectionSet>`; `begin_group`/`end_group` take a `SelectionSet`. |
| `buffer.rs` | `HistoryMove{selections: SelectionSet}`; **`snapshot_of` + `ranged_anchor` DELETED**; `begin/end_undo_group` take a set; `undo`/`redo` return the set; `set_selection`'s clamp **preserves** the goal column. |
| `movement.rs` | NEW `move_vertical_goal(buffer, off, dir, goal) -> CharOffset`; `move_up`/`move_down` re-expressed through it. |
| `multi_cursor.rs` | **NEW.** `add_cursor_vertical` (extends the cursor column from the bottom-most going down / top-most going up; no-op at last/first line), `toggle_cursor_at` (add, or remove an existing caret; **never empties**), `collapse_to_primary`, and the N-cursor motion drivers `move_all_horizontal` / `move_all_vertical` / `move_all_to`. |
| `find.rs` | `replace_all`'s one `SelSnapshot` construction → `SelectionSet`. |
| `lib.rs` | Re-exports: drop `SelSnapshot`, add the `multi_cursor` seams. |

**`crates/marley_app/`**
| File | Change |
|---|---|
| `editor_surface.rs` | **`OpenFile.caret` + `.anchor` DELETED** (the `Buffer` owns the set). `active_caret()` / `active_selection()` / `active_anchor()` become compat shims over `primary()` — **keeping ~25 call sites unchanged**. `active_buffer_caret_anchor_mut` → `active_buffer_mut()`. NEW `active_selections()` (render) + `set_single_caret(off)` (the jump op — fixes REQ-008 structurally). |
| `code_view.rs` | `styled_slices_with_marks` takes `&[Range<usize>]`; NEW `row_selection_cols_all`. |
| `app.rs` | `EditorDraw.caret: Option<(row,col)>` → `carets: Vec<(row,col)>` + the per-row filter; the selection band takes N; the motion ladder + ⌘-arrow block → `move_all_*`; typing/backspace/paste/cut → `edit_at_selections`; the mouse-down ⌘-click branch; Esc collapses; undo/redo → `set_selection(mv.selections)`; `open_file_at` → `set_single_caret`. **Tab indent/dedent keeps its raw `edit()` calls (D6).** |
| `keymap.rs` | 2 Editor-scoped rows: `add-cursor-above` / `add-cursor-below`, shadowing the global M6 #131 pane-focus rows via the shipped `action_for` precedent. |
| `input.rs` | `apply_editor_key` is single-caret by construction — the typing/backspace path routes through `edit_at_selections` instead. |
| `headless_drive.rs` | `set_editor_caret` fixture → the set shape; new N-cursor drive tests. |

### D6 re-verified (the nesting contract)
`grep` for `begin_undo_group` in the app returns **exactly one** site — Tab indent/dedent
(`app.rs:6475/6485`). It keeps its raw `edit()` calls; it must NOT call `edit_at_selections`, because
`begin_group` **overwrites** an open group and silently discards its records. No other site nests.
(Block-indenting *N* cursors is a #299/#300-class op and is out of scope here.)

### Regression Test Plan

| REQ | Test | Where | Proves |
|---|---|---|---|
| **REQ-002** | **`goal_column_survives_a_short_line_and_two_cursors_do_not_merge`** — **written FIRST**: 2 cursors at col 10 on long rows, a 3-char row between; Down, Down; assert both clamped to 3 en route, both **restored to col 10** after, and the set still has **2 members**. | `multi_cursor.rs` | **The invariant that makes multi-cursor survivable.** Without the goal, both cursors clamp to the short row's end, merge, and never come back. |
| REQ-001 | `add_cursor_vertical_extends_the_column` — down adds below the bottom-most at its goal column; up adds above the top-most; **no-op at the last/first line**; adding onto an existing caret collapses (via `from_selections`). | `multi_cursor.rs` | The gesture. |
| REQ-003 | `typing_at_n_cursors_is_one_undo_unit` — 2 cursors, type; both inserted; **one `undo()`** → original text **and** (D7) **both cursors restored**. | `buffer.rs` + headless | The N-caret edit + the D7 undo fidelity. |
| REQ-004 | `styled_slices_with_marks_labels_every_selection` (N bands, incl. overlapping/adjacent/empty) + `row_selection_cols_all` (a row hit by 2 selections → 2 col ranges; a row hit by none → empty). | `code_view.rs` | N-band render math. **cov/MSI 100, no exclusion.** |
| REQ-005 | `toggle_cursor_at_adds_removes_and_never_empties` — add at a fresh offset; remove at an existing caret; **removing the last is a NO-OP**. | `multi_cursor.rs` | The ⌘-click semantics + the never-empty guard. |
| REQ-006 | `collapse_to_primary_keeps_member_zero` + a headless Esc test. | `multi_cursor.rs` + headless | Esc / plain-click collapse. |
| REQ-007 | `move_all_moves_every_cursor_and_merges_collisions` — 2 cursors, Right ×N until they collide → the set merges to 1 (proving re-canonicalization); shift-extend pins every anchor. | `multi_cursor.rs` | N-cursor motion. |
| REQ-008 | `open_file_at_jump_clears_a_live_selection` — a live selection, then a file:line jump → exactly one caret, no spurious selection. | headless | The confirmed latent bug, pinned. |
| REQ-009 | `scripts/gates.sh --diff` | gate | cov 100 + MSI 100 on every new pure seam. |
| — | **THE LIVE DRIVE** (Validate, machine unlocked): open a file → ⌘⌥↓ → **two carets on screen** → type → **both lines** → ⌘Z → both revert → Esc → one caret. Plus ⌘-click add/remove. **Capture pixels and READ them.** | `scripts/selftest` | The only proof that a pane actually *works*. |

Uncoverable: none new. gpui render/input shims in `app.rs` stay coverage-excluded + `mutants::skip`-annotated
with justifications, exactly as the other 130 do.

### Risks
- **The undo type change ripples through `find.rs` + every existing undo test.** Bounded (I enumerated every
  consumer), and it *deletes* two functions — but it will make the diff look bigger than the behavior change.
- **`Selection` gaining a field touches `PartialEq`.** The merge/dedup machinery compares `start()`/`end()`,
  not `PartialEq`, so #296's invariant code is untouched; only test fixtures that compare whole `Selection`s
  could be affected.
- **The goal column is the subtlest thing in the ticket.** REQ-002 is written first, deliberately.
- **A green gate is necessary, not sufficient** — on #296 the critic found three real bugs *after* the gate
  went green. Inspect spawns adversarial critics and **waits**.

---

## Phase 3 — Implement

Built to the manifest. `cargo clippy -D warnings` clean; **1066/1066 workspace tests pass**.

### What shipped
**`crates/editor`** — `Selection` gained `goal_col`; `SelectionSet::primary()`; **`SelSnapshot` DELETED**
(`UndoGroup` + `HistoryMove` carry a `SelectionSet`, and `Buffer::snapshot_of` + `HistoryMove::ranged_anchor`
evaporated with it, exactly as D7 predicted); `movement::move_vertical_goal` + `VDir` (with `move_up`/
`move_down` re-expressed through it); **new `multi_cursor.rs`** — `add_cursor_vertical`, `toggle_cursor_at`,
`collapse_to_primary`, `move_all_char` / `move_all_horizontal` / `move_all_vertical` / `move_all_to`.

**`crates/marley_app`** — `OpenFile.caret` + `.anchor` DELETED (the `Buffer` owns the set); the compat
accessors (`active_caret`, `active_selection`) kept ~25 call sites unchanged; `active_buffer_caret_anchor_mut`
(the E0499 three-way) replaced by the plain `active_buffer_mut`; N-caret render; N-band highlight;
`edit_at_selections` on every edit path; ⌘-click / Esc / plain-click; two Editor-scoped keymap rows.

### Three things the design did not foresee (each found by the code, not by guessing)

**1. Typing does NOT go through `apply_editor_key` — it goes through the IME.** `app.rs:6544` returns early
for a printable char *deliberately*, with a comment naming the platform text path "the ONE insert mechanism,
IME included". So the N-cursor insert had to land in **`ime::replace_text`**, not the key ladder. Had I
followed the design's assumption, multi-cursor typing would have silently done nothing and I'd have found out
at the live drive. The split there is now honest: **no platform range and no composition in flight ⇒ ordinary
typing ⇒ every cursor**; a platform-addressed range or a live composition names exactly ONE span, so the set
collapses (the `NSTextInputClient` protocol cannot address N cursors — that is the platform's shape, not a
Marley limitation). `ime.rs`'s ops now read/write the buffer's own set, and its test fixture's `caret`/`anchor`
became *views* onto the primary — one owner, which was the whole point.

**2. A GREEN test caught a regression the gate would have shipped.** Routing typing through
`edit_at_selections` made every keystroke a *bracketed* undo group — and `UndoHistory::record` refuses to
coalesce into a bracketed group (its guard is literally `sel_before.is_none()`). So **⌘Z would have undone one
character at a time instead of a typed word**, destroying the #253/#282 feel on every keystroke in the editor.
`ime::tests::undo_ladder_through_composition_matches_accepted_reality` failed and said so. The fix is also the
truer contract: **one cursor ⇒ no undo group.** A group exists to make N edits atomic; with one edit there is
nothing to make atomic, and bracketing it anyway only breaks coalescing. Single-cursor typing is now
byte-for-byte what it was before #297.

**3. Enter and Tab need a PER-CURSOR replacement, which `edit_at_selections` could not express.** Enter's
auto-indent clones the leading whitespace of *each* cursor's own line; one shared replacement string cannot
say that. The options were to compute the resulting carets in `app.rs` — real offset math in a
coverage-excluded file, exactly what the design forbade — or to generalize the primitive. Generalized:
`selections_after_multi_edit` now takes a per-member `&[usize]` (the constant case is just `vec![n; len]`),
and **`Buffer::edit_at_selections_with(set, f, origin)`** computes each cursor's replacement against the
PRE-edit buffer (so cursor 3's indent is read from the same text cursor 1's was), with
`edit_at_selections` becoming `edit_at_selections_with(|_, _| replacement)`. Enter now indents correctly at
every cursor.

### Deviations from design (with reason)
- **`ime.rs` was in scope after all** (see 1). The design's file manifest missed it because the design trusted
  the scout's "typing → `apply_editor_key`" claim instead of reading the key router. Cost: a real rewrite of
  two IME ops + their fixture. Lesson recorded for inspect.
- **`edit_at_selections_with` + the per-member `selections_after_multi_edit`** are new primitives the design
  did not name (see 3). Justified: the alternative was untested math in an excluded file.
- **`apply_editor_key` and `code_view::paste_edit` are DELETED** — both became dead once typing moved to the
  IME path and paste moved to `edit_at_selections`. Dead code is deleted, not suppressed (§0). Their tests went
  with them.
- **A no-op member is now SKIPPED inside `edit_at_selections`** rather than applied: `edit()` on an empty range
  with an empty replacement would record an empty undo record AND bump the version, *falsely marking the file
  dirty*. Not hypothetical — a backspace with one caret at offset 0 (nothing to delete) and another mid-line
  produces exactly that mixed set.
- **Tab/⇧Tab block indent stays PRIMARY-only** (D6 honored: it keeps its own undo group and raw `edit()` calls
  — `begin_group` overwrites an open group and would silently discard the whole indent). Multi-cursor block
  indent is a #299/#300-class op.
- **⌘A, ⌘D, the find bar, and `selected_text_range` collapse to one selection**, as the spec's Out section said.

### The keymap roster guard fired, as designed
`keymap::tests::all_chords_lists_every_binding` asserts an exact binding count — it failed on the two new rows
and forced me to state them explicitly (both the count and the fact that ⌘⌥↓ now appears TWICE: Editor-scoped
add-cursor-below, and still the global focus-down it shadows). That is a roster guard doing its job.

---

## Phase 4 kill set — TRACED, not guessed (prepared during inspect)

`cargo mutants --list -f <file>` on the new pure seams. **`Selection`/`SelectionSet` derive no `Default`**, so
every `-> Default::default()` body mutant is **UNVIABLE** (won't compile) and does not count — the #203 lesson
(a body mutant's viability depends on a `Default` derive). The VIABLE set every test must kill:

**`crates/editor/src/multi_cursor.rs` (23 listed):**
| Line | Mutant | The test that must kill it |
|---|---|---|
| 26:19 | `delete !` in `as_pair` | a RANGE member vs a CARET member must behave differently (shift-extend from a range) |
| 43 | `goal_of -> 0` / `-> 1` | a test asserting a SPECIFIC goal column (not 0, not 1) |
| 81:25 | `row == 0` → `!=` | ⌘⌥↑ at the FIRST line is a no-op; at a non-first line it adds |
| 82:31 | `row + 1 >= len_lines` → `<` | ⌘⌥↓ at the LAST line is a no-op |
| 82:27 | `row + 1` → `row - 1` / `row * 1` | the last-line boundary must be exact |
| 111:37,51 | `sel.start() <= off` / `off <= sel.end()` → `>` | ⌘-click exactly ON a cursor's offset removes it (BOTH boundaries) |
| 111:44 | `&&` → `\|\|` | a click OUTSIDE a member must not match it |
| 115:20,43 | the `len() == 1` guard → true/false/`!=` | ⌘-clicking the ONLY cursor is a NO-OP (never empties) |

**`crates/marley_app/src/code_view.rs` (the N-range seams):**
| Line | Mutant | The test that must kill it |
|---|---|---|
| 167 | `row_selection_cols_all -> vec![]` / `vec![(0,0)]` / `(0,1)` / `(1,0)` / `(1,1)` | a row crossed by TWO selections yields TWO specific, non-trivial bands |
| 263:18 | the `s.end > s.start` cut guard → `==` / `<` / `>=` | an EMPTY and an INVERTED selection in the slice contribute no cuts |
| 284:58,68 | `selected` = `s.start <= a && a < s.end` → `>` / `==` / `<=` | both boundaries of "inside ANY selection" |
| 284:63 | `&&` → `\|\|` | a slice outside every selection is NOT selected |

**`crates/editor/src/movement.rs`:** 5 on `move_vertical_goal` (the first/last-row target + the
`goal.min(target_len)` clamp).

Note what has NO mutant: `members.first()`/`last()` (cargo-mutants does not mutate method calls) and the
`let-else`. So finding **M3** below is a pure COVERAGE failure (an uncoverable line), not a mutation one — the
two gates catch different things, which is exactly why both exist.

---

## Inspect (Phase 3.5)

Five adversarial critics, run in parallel and **waited for**. They found **nine real bugs, six of them HIGH**,
in code where the gate was green: clippy clean, 1066 tests passing, 100% line coverage, MSI 100, and a
50,000-step differential fuzzer. I reproduced every one myself before fixing it.

**The headline: the flagship gesture of this ticket destroyed its own cursor on the very next keypress.**

### C1 [HIGH→fixed] — `<=` merged touching RANGES; ⌘⌥↓ then ⇧↓ destroyed a cursor
`from_selections` merged any two members whose ranges TOUCHED. Add a cursor below (⌘⌥↓), then extend down
(⇧↓ — the most ordinary next keypress): the two line-spanning ranges abut *exactly* at the shared line-start
offset, and the merge unioned them into one. **Reproduced:** `n=2 [(0,0),(5,5)]` → ⇧↓ → `n=1 [(0,10)]`. Because
that is a MOTION and not an edit, **⌘Z could not bring the cursor back.**

**This was a bug I shipped in #296**, and it was *defended by a test*. `selection.rs`'s merge assertion demanded
the union, with the rationale *"otherwise two abutting selections would each insert at the seam and double up"*.
**That rationale is false.** Two abutting ranges replace two DISJOINT spans, and the back-to-front sweep applies
them with zero interference — `[0..3]`+`[3..6]` over `"abcdef"` with `"X"` must give `"XX"`; it gave `"X"`.

**FIX:** a caret/range-asymmetric `overlaps`: `<=` where either side is a CARET (same-offset carets are one
cursor and MUST collapse, or that cursor inserts twice), `<` where both are RANGES (merge only on a true
overlap). The pinning test was replaced by its inverse.

### C1-companion [HIGH→fixed] — the fix made a NEW state reachable, and nothing guarded it
Once touching ranges survive, a DELETE over two of them lands both cursors on the SAME offset.
`selections_after_multi_edit` ended in raw `from_members` (no canonicalization) → two carets on one offset →
the disjointness invariant the whole crate rests on, broken. **FIX:** `from_selections`. See C9 — the fuzzer
could not see this.

### C2 [HIGH→fixed] — the #298 refinement I documented was catastrophic
I had written that #298 should use `sel.start() < cur.end() || sel.is_caret() || cur.is_caret()`. A critic RAN
it: `is_caret()` **short-circuits the whole guard to `true`**, so *any two carets anywhere in the document*
merge into one giant range. Two carets 16 apart → merged. A future implementer following my comment would have
shipped total multi-cursor failure. **FIX:** the doc now points at `overlaps`.

### C3 [HIGH→fixed] — ⌘⌥↑ over a selection was a silent no-op
`add_cursor_vertical` grew from `head()` — the end you *dragged*. Over a forward selection that is the BOTTOM,
so ⌘⌥↑ seeded the new caret INSIDE the range, where `from_selections` swallowed it. **Reproduced:** `[(5,15)]`
→ ⌘⌥↑ → `[(5,15)]`. **The keypress did nothing.** Mirror image on a backwards drag: no-op *and* it flipped
anchor/head. **FIX:** grow from the outer EDGE in the direction of travel (`start()` up, `end()` down).

### C4 [MED→fixed] — a collapsed caret pair lost its goal column
The merge dropped `goal_col` on the grounds that "the union of two cursors is a RANGE". For a caret+caret
collapse **the union is a CARET** — so the survivor was stranded at whatever column the short line had clamped
it to, permanently. **FIX:** a merge that yields a caret keeps a goal.

### C5 [MED→fixed] — REQ-002's "NEVER MERGE" was a promise the code could not keep
Cursors on the SAME row that clamp to the same column occupy the same OFFSET, and one offset is one cursor:
they collapse, here as in every editor. The spec and three docstrings all promised more. **FIX:** state the real
guarantee — cursors on **distinct rows** never merge.

### C6 [HIGH→fixed] — multi-cursor ⌫ + ⌘Z, then the next keystroke ATE the restored text
⌫ turns each bare caret into a one-char *consuming* range. Those ranges were recorded as the undo snapshot, so
⌘Z handed the user back SELECTIONS they never made — and the next typed character REPLACED them.
**Reproduced:** `"abcdef"` carets@2,5 → ⌫ → ⌘Z → type `X` → `"aXcdXf"`, when it must be `"abXcdeXf"`. The
single-cursor path was fine, so this was a 1-vs-N divergence in a **data-losing** direction.
**FIX:** backspace moved INTO the crate as `Buffer::backspace_at_selections`, over a new
`edit_ranges_restoring(restore, targets, …)` that names both halves explicitly — the ranges to edit, and the
cursors to restore. (My first fix took the buffer's *current* selection instead; a critic's regression test
caught that it silently restored one cursor when the caller edited at two. The seam now states it rather than
guessing.)

### C7 [HIGH→fixed] — a MIXED set silently dropped the caret's backspace
The key ladder branched on "does ANY cursor hold a selection?" and, for a selection PLUS a bare caret, took the
replace-every-selection path — which deleted the range and **dropped the caret's backspace entirely**.
`"abcdef"` with a selection `1..3` and a caret at 5 → `"adef"`; it must be `"adf"`. **FIX:** ⌫ routes through the
multi-cursor arm unconditionally. (`selection_replacement` became dead and was deleted.)

### C8 [HIGH→fixed] — a stale selection/marked span PANICKED ropey
`undo`/`redo` restored the TEXT but never updated `self.selection`, leaving the buffer **internally
inconsistent** — its own cursors pointing past its own EOF. `edit_at_selections` clamped defensively, but the
IME's `edit_target` did not, and it feeds a rope range built from app-held state. **Reproduced:** a panic at
`ropey/src/rope.rs:952`. **FIX at both roots:** `undo`/`redo` now restore the cursors with the text (the buffer
is never half-updated), and `edit_target` clamps every arm (the marked span lives in APP state, which the buffer
cannot keep in step).

### C9 [MED→fixed] — the 50k-step fuzzer could not SEE the region the change touched
The differential fuzzer's generator advanced `cursor = end + 1`, forcing a gap of ≥1 between every pair — so in
**50,000 steps it never once produced two adjacent selections**, which is precisely the state C1 made legal. It
was validating the equivalence claim only over the region the change did not touch, and C1-companion's fix was
guarded by **nothing**: reverting it left the entire gate green. (cargo-mutants cannot help either — it does not
mutate method calls, so `from_selections`→`from_members` has no mutant.)
**FIX:** widened the generator to `cursor = end`. **It failed on the first run** — and the divergence was the
ORACLE's, which computes raw landing offsets and must canonicalize like production before comparison. Plus a
hand-written fixture for the exact production path (two ⌘-clicked carets one char apart, then ⌫).

### C10 [MED→fixed] — a typed run at N cursors undid ONE CHARACTER AT A TIME
Typing `"hello"` at 3 cursors produced **5 undo steps**; at 1 cursor it correctly produces 1. Each keystroke
opened its own undo group, and `UndoHistory::record` refuses to coalesce into a BRACKETED group (its guard is
`sel_before.is_none()`).

**The damning part:** my own comment six lines above the offending branch says a group-per-keystroke *"would
make ⌘Z undo one CHARACTER at a time instead of a typed word — destroying the #253/#282 feel"*. I fixed that for
N=1 and then did precisely it for N≥2 — reintroducing the exact harm the rule exists to prevent, at the moment
there are *more* cursors to lose.
**FIX:** group-level coalescing in `end_group` — a group merges into the previous one when it is the same run
continued (same cursor count, `prev.sel_after == group.sel_before` so nothing moved between keystrokes, every
record a pure 1-char insert of matching origin). Because the cursors did not move, each new char lands exactly
at the end of its own record's text, so appending is contiguous by construction — no offset arithmetic. A typed
run at 3 cursors is now ONE ⌘Z, and a cursor move between keystrokes correctly breaks the run.

### Docs corrected (critic 3, LOW — but one of them is the very trap this ticket is about)
- **`selections_after_multi_edit`'s header still asserted "the results are strictly increasing"** — which the C1
  merge change quietly falsified (`gap` can now be 0). **The same stale-rationale trap that pinned C1 for an
  entire ticket, one function later, in the same file, hours after I wrote the post-mortem.** Corrected.
- The IME's N→1 collapse is framed as inherent to *compositions*, but an explicit `replacementRange` also
  arrives for press-and-hold accents and Text Replacement/autocorrect on a stock US keyboard. Said so.

### C5-critic MED/LOW (all fixed)
- **Gate 14 was RED** — two intra-doc links to private items (`rustdoc -D warnings`).
- **My Phase 3 deletion over-reached**: it removed tests for `apply_key`'s `Up|Down|Other` arms and for
  `submit_line` — code #297 never touched — dropping the coverage floor. Both restored. *Deleting a test for
  code you did not change is how a floor silently falls.*
- `movement::move_up`/`move_down` became dead (the ladder uses `move_all_vertical`) → deleted; their tests
  re-expressed against `move_vertical_goal`.
- `apply_editor_key_multi -> KeyOutcome` was never read → `-> ()` (which also makes its body mutant *viable*).
- A per-keystroke `Vec` clone in the app's ONE insert path → removed (`set` is already locally owned).

### Rejected, with reasons (verified, not waved away)
- **The focus-gate rule does not apply here.** The editor is a TAB, not a `PaneGrid` pane — `is_focused` lives
  only in the terminal pane loop. Exactly one editor surface renders (the active tab's), as before.
- **`move_all_char` vs `move_all_horizontal` is a real distinction, not duplication.** `extend_or_move` has an
  unshifted-over-a-selection branch that collapses to the edge WITHOUT moving; `extend_or_go` does not. A critic
  ran it: selecting `2..5`, unshifted Right → `move_all_char` gives 5 (collapse), `move_all_horizontal` gives 6
  (walks past). Merging them would silently regress the #255 feel.
- **The IME discriminator is correct** — verified against gpui's source, not assumed: ordinary typing arrives
  with `replacement_range = None` on both delivery paths (`window.rs:1746` passes `None`; AppKit's
  `insertText:replacementRange:` passes `NSNotFound`, which `to_range()` maps to `None`). A real range arrives
  only for autocorrect/dictation, where collapsing to the platform-named span IS correct.

### Ledger
**10 real findings (6 HIGH), all fixed at source; 3 rejected with verification.** Every fix carries a permanent
regression test (9 new in `multi_cursor.rs`, plus the widened fuzzer and the new `selection.rs` fixture).
**1077/1077 tests pass · clippy clean · rustdoc clean.**

**The lesson of this ticket, and it is a hard one:** a green gate is not evidence. Here it was actively
misleading — **a test asserted the bug and defended it**, and **a fuzzer's generator excluded the very state the
change made reachable**. Coverage, mutation and fuzzing each have blind spots, and on this diff those blind
spots OVERLAPPED. The only thing that found these was an adversary who tried to break the code and ran what it
claimed.

---

## Phase 4 — Validate

**`cargo nextest run --workspace` → 1094 passed, 0 failed** (5 skipped). Doctests ok.
**`scripts/gates.sh --diff` → GATE GREEN [diff], 15/15** — including **coverage ≥ 100% lines** and
**mutation MSI ≥ 100%** on every changed file.

### THE LIVE DRIVE — multi-cursor proven on real pixels
The machine was unlocked, so every acceptance criterion was driven on the running app and the PNGs read.
Fixture `mc297.txt` (ALPHA_ONE / BRAVO_TWO / CHARLIE_3 / DELTA_FOUR), opened by clicking a `path:line` ref in
the terminal.

| Step | What the pixels showed |
|---|---|
| click line 1 col 6, **⌘⌥↓** | **TWO CARETS** — `ALPHA_\|ONE` and `BRAVO_\|TWO`, bars at the SAME display column on consecutive rows. The Editor-scoped keymap row shadowed the global pane-focus binding, and the N-caret render drew both. **REQ-001, REQ-004.** |
| type `ZZ` | `ALPHA_ZZONE` / `BRAVO_ZZTWO` — the text landed at **BOTH** cursors, both carets advanced past what they typed. **REQ-003 (insert half).** |
| **⌘Z** (once) | Text back to `ALPHA_ONE` / `BRAVO_TWO` **and BOTH carets restored at column 6**. Three fixes proven at once: REQ-003 (one undo step for N cursors), **C10** (a two-character *run* undid as ONE step, not two) and **C6** (undo returns both cursors, not one). |
| **Esc** | One caret; line 2's is gone. **REQ-006.** |
| **⌘-click** line 4 | Two carets — `ALPHA_\|ONE` (preserved) and `DELTA_\|FOUR` (added). **REQ-005.** |

**A bonus proof, from a mistake.** My first drive typed through the harness's `focus` verb — which *clicks* —
and the cursors collapsed to one at the click point. That is not a bug: it is **D4**, the plain-click collapse,
behaving exactly as designed. The app was right and my drive script was wrong; re-running without the stray
click gave the correct result above.

### Tests added
- **`multi_cursor.rs`** — REQ-002 (the survivability invariant: two cursors on DISTINCT rows cross a SHORT
  line, clamp, **keep their goal columns**, spring back, and do NOT merge); the `as_pair` kill (a bare caret
  CANNOT kill its `delete !` mutant — a caret flows identically through `unwrap_or` and `.filter` — so the test
  uses a REAL RANGE that must pin its anchor on shift-extend and collapse-to-edge unshifted); the
  `add_cursor_vertical` document-edge no-ops; `toggle_cursor_at`'s add/remove/never-empty (with the lone cursor
  at a **NON-ZERO** offset — at offset 0 the `len()==1`→`false` mutant's output is identical and survives);
  `move_all_*` behavior assertions (these have **ZERO viable mutants** — no `Default` derive — so only
  hand-written assertions can pin them); and the three coalescing-break cases below.
- **`code_view.rs`** — `row_selection_cols_all` (a row crossed by TWO selections → two specific bands; a row
  crossed by none → empty) and `styled_slices_with_marks` (N bands, the unselected gap between them, an EMPTY
  and an INVERTED selection contributing no cuts, and the `[start, end)` boundary). **These had to be DIRECT
  assertions: llvm-cov already reported the file at 100% lines because the render tests execute these
  functions incidentally — which satisfies coverage while killing NONE of the mutants.** Coverage and mutation
  ask different questions, and "the line ran" is not an answer to the second.
- **`keymap.rs`** — the shadow RESOLVES (not merely exists): ⌘⌥↓ → `add-cursor-below` in `KeyContext::Editor`,
  and still `focus-down` globally and on a terminal.
- **`editor_surface.rs`** — REQ-008: a jump clears a live selection AND every extra cursor.
- **`headless_drive.rs`** — the whole flow through the REAL key path: ⌘⌥↓ → type → one ⌘Z → Esc.
- **`indent.rs`** — the C11 regressions (below).
- **`ime.rs`** — a composition begun over a live selection replaces it (the only way to reach `edit_target`'s
  selection arm).

### Two more real bugs, found during Validate
- **C11 [HIGH] — Tab silently destroyed the extra cursors.** Critic 4 (the last to report) drove it end to end:
  three cursors → Tab → **one**. Both branches of the Tab arm wrote a single-member set. Deferring multi-cursor
  *indent* to #299/#300 is a fair scope call; **silently deleting the user's cursors is not** — they got no
  indent AND lost their place, and the next keystroke went somewhere they never asked for. It was the only edit
  key in the diff that did this. **FIX:** a pure `indent::rebase_selections` carries EVERY cursor through the
  same `rebase_through` the arm already used for the primary (the bare-caret pad is expressed as a
  one-element edit list so it rides the identical path). Three regression tests.
- The `coalesces_into` guard I added for C10 had **three surviving mutants** (`&&`→`||`), each widening what
  counts as "the same typed run". Killed with the three cases the run must BREAK on: a **type-over** (its
  records remove text — the single-cursor rule refuses this too), a **multi-char paste** (not "one more
  character"), and an **Agent write** after a Human run (the provenance boundary `EditOrigin` exists for).

### Two dead lines the 100% floor correctly refused
- The differential fuzzer's `if cursor > len { break }` became **unreachable** when I widened its generator
  (C9): `cursor = end` and `end` is already `.min(len)`. Removed.
- `ime::edit_target`'s `a != caret` guard became **unreachable-false**: `anchor` is now derived as
  `(!primary.is_caret()).then(…)`, so it is `Some` *only* when anchor ≠ head. It was a leftover from when the
  anchor was a free-floating `Option` that could equal the caret — a state the buffer-owns-the-cursor refactor
  made unrepresentable. Removed.

No pre-existing failures; nothing excluded.
