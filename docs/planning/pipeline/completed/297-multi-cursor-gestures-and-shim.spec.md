---
pipeline_id: 565f8739-4854-4816-86b6-1a812ddfb1ba
ticket: forge#297 (a8230892-ec68-411c-946e-332e8542c4fd) · local docs/planning/tickets/open/TICKET-297-multi-cursor-gestures-and-shim.md
aar_id: 8cecc7af-5a5d-42fc-8bfd-9b81ee7e9feb
status: Phase 5 — Complete PASS
title: Multi-cursor gestures + the app shim — make the #296 core reachable, visible, and drivable
type: feature
milestone: M19
references: [docs/planning/pipeline/completed/296-multi-cursor-core.spec.md]
---

## Title
Wire the app's editor surface onto the N-cursor `SelectionSet` #296 shipped, render N carets and N
selection bands, route every edit through `edit_at_selections`, teach motion to move all N (which
requires a per-cursor **goal column**), and add the three gestures that let a human *create* a second
cursor: **⌘⌥↑/↓**, **⌘-click**, and **Esc**. Without this ticket the #296 core is unreachable — no key
in Marley can produce a second cursor today.

## Scope

### In
- **The surface conversion.** `OpenFile{caret: CharOffset, anchor: Option<CharOffset>}`
  (`editor_surface.rs:21,24`) → one `SelectionSet`. 45 production call sites (33 `app.rs`,
  12 `headless_drive.rs`) go with it, via the three accessors `active_caret` / `active_selection` /
  `active_buffer_caret_anchor_mut`. Caret/anchor are **not** persisted (`grid_layout` serializes paths +
  active index only), so there is no settings round-trip to migrate.
- **N-caret render** — `EditorDraw.caret: Option<(row,col)>` → many. The caret is already drawn *per row*
  (`app.rs:3273-3287`), so each row filters the carets on it.
- **N-selection highlight** — the pure `code_view::row_selection_cols` + `styled_slices_with_marks` take a
  *slice* of ranges instead of one `Option<Range>`. **`code_view.rs` is excluded from no gate** — this is
  held to 100% coverage and MSI 100.
- **Every edit through `Buffer::edit_at_selections`** — typing, backspace, paste, cut, Enter-auto-indent,
  find-replace. One undo unit per gesture.
- **N-cursor motion + the per-cursor goal column.** Arrows move *every* cursor.
- **The three gestures**: ⌘⌥↑/↓ add a cursor above/below at the goal column; ⌘-click adds a cursor, or
  removes one already there (never the last); Esc collapses to the primary.
- **The pure seams** (all `crates/editor`, cov/MSI 100): `add_cursor_vertical`, `toggle_cursor_at`,
  `collapse_to_primary`, and whatever carries the goal column.
- **Fix the `open_file_at` stale-anchor bug** (below) — the conversion fixes it for free; prove it.

### Out (explicitly deferred)
- **⌘D-adds-cursor → #298.** The `<=` adjacent-range over-merge #296 flagged is #298's problem; the exact
  refinement is documented on `from_selections`.
- **Multi-cursor-aware comment / move-line / delete ops → #299 / #300 / #303.**
- **⌘A stays single-selection** (select-all is one range by definition).
- **gpui's `EntityInputHandler::selected_text_range`** returns exactly one `UTF16Selection` — the trait's
  shape is imposed by the platform, not by Marley. It reports the **primary**; the IME cannot address N
  cursors and that is not a Marley limitation to fix here.

## Reference (§20)
**Zed / VS Code multi-cursor gestures** — add-cursor-above/below on ⌘⌥↑/↓, ⌘-click to add or remove a
cursor, Esc to collapse to one; typing inserts at every cursor; the whole gesture is one undo unit; a
cursor walking vertically past a short line clamps to its end but **restores its column** on the next long
one (the goal column). Marley matches the *behavior* with its own `SelectionSet` + the shipped
`Buffer`/undo machinery. Clean-room §20 — observed behavior only; no Zed or VS Code source read or
translated. The keymap collision (⌘⌥↑/↓ is Marley's global pane-focus, M6 #131) is resolved with Marley's
own already-shipped mechanism, not the reference's.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the shim and the gestures ship together.** #296's D6 moved the whole app shim here deliberately:
  a shim with no gesture is un-drivable dead weight, and a gesture with no shim has nothing to drive. This
  is one slice; it cannot be halved.
- **D2 — ⌘⌥↑/↓ is claimed via the Editor `KeyContext`, not a modifier-ladder change.** The chord is
  currently the global pane-focus binding (`keymap.rs:189-205`). `Keymap::action_for` already ranks a
  context-scoped binding above a global one for the same chord — the shipped precedent is ⌘D (global
  `new-terminal`; Editor-scoped `select-next-match` "SHADOWS it there"), and ⌘F and ⌘A. So #297 adds two
  Editor-scoped rows and the chord falls through the existing ladder (the ⌘-motion block excludes `alt`;
  the editor router requires `!platform`) into the keymap dispatch. **Cost, stated plainly: while an editor
  pane is focused, ⌘⌥↑/↓ no longer moves pane focus.** That matches the reference editors and is the
  intended trade.
- **D3 — the goal column is load-bearing, not polish.** Nothing in the tree has one (`movement.rs:115`:
  "No goal-column memory (v1)"). Without it, two cursors moving vertically through a **short** line both
  clamp to its end, `from_selections` merges them, and **they never come back** — multi-cursor silently
  destroys itself on the first short line. The set must carry a per-cursor goal column.
- **D4 — a plain click COLLAPSES the set; ⌘-click adds/removes.** Today a click sets `caret = anchor = abs`
  (`app.rs:3336-3338`). With N cursors, a plain click must collapse to a single cursor at the click — this
  is a real behavior change, not a rename.
- **D5 — Esc collapses to the primary.** The Editor-context Esc handler already exists
  (`app.rs:6425-6444`) and today only clears the anchor; it becomes the collapse point.
- **D6 — the Tab indent/dedent site must NOT use `edit_at_selections`.** It already brackets its own
  `begin_undo_group`/`end_undo_group` (`app.rs:6475-6485`), and #296's contract is that
  `edit_at_selections` must never be nested inside another group — `begin_group` **overwrites** an open
  group and silently discards its records. That site keeps its raw `edit()` calls.

- **D7 — RESOLVED at Design: grow the undo snapshot (option A).** `SelSnapshot` is **deleted**;
  `UndoGroup.sel_before/sel_after` and `HistoryMove` carry a `SelectionSet`. This makes the code *smaller*:
  the two functions #296 was forced to write purely to bridge the impedance mismatch —
  `Buffer::snapshot_of` and `HistoryMove::ranged_anchor` — both evaporate, because a `SelectionSet`
  expresses "collapsed ⇒ a caret" natively. Option (B) would have forced *new* lossy conversion code at the
  undo site instead. ⌘Z now restores all N cursors.
- **D8 — the goal column lives ON `Selection`** (`goal_col: Option<usize>`, a char column). Rejected: a
  parallel `Vec` (drifts), or one goal on `OpenFile` (wrong — ⌘-click puts cursors in different columns, so
  the goal is intrinsically per-cursor). The payoff: **every existing constructor sets it to `None`**, so a
  horizontal motion, an edit, a click, or a merge resets it *by default* — you cannot forget to reset it.
  Only the vertical path carries it forward.
- **D9 — "primary" = member 0 (the topmost cursor).** `from_selections` sorts, so "last-added" is not
  recoverable; tracking a `primary` index would add a NEW invariant to a type that just took five critic
  findings to harden. Documented deviation: ⌘⌥↓-then-Esc returns you to where you started (correct);
  ⌘⌥↑-then-Esc leaves you at the topmost cursor, not your origin.
- **D10 — the app's `OpenFile.caret`/`.anchor` are DELETED; the `Buffer` becomes the single owner of the
  cursor set.** The app never read `buffer.selection()` — it kept a *parallel* caret. Deleting the duplicate
  (rather than teaching it to sync) dissolves the E0499 3-way borrow entirely and fixes REQ-008 structurally.

## OPEN DECISION — RESOLVED at Phase 2 (see D7). Kept for the record:
**⌘Z after an N-cursor edit restores the text of every cursor, but only ONE cursor.** Verified in the real
code: `SelSnapshot{anchor, caret}` (`undo.rs:24-31`) is a single pair; `undo()` hands back
`HistoryMove{caret, selection}`; the app applies `*c = mv.caret; *a = mv.ranged_anchor();`
(`app.rs:4573-4577`). #296's `edit_at_selections` groups all N edits into one undo unit — so the group loop
*does* invert every record and **the text is fully correct** — but the snapshot that group carries describes
one caret.

- **(A) Grow the snapshot to hold a `SelectionSet`.** Correct, and matches the reference. Sizing: 34
  `SelSnapshot` mentions across 5 files; `SelSnapshot` and `HistoryMove` are `Copy` today and a
  `SelectionSet` (a `Vec`) is not, so that derive ripples. **Note it is arguably the *cheaper* end state:**
  once the app holds a `SelectionSet`, option (B) forces new lossy `SelectionSet → (caret, anchor)` code at
  the undo site, while (A) makes it `*sel = mv.selections`.
- **(B) v1 = "undo restores the primary cursor"** + a follow-up ticket.

Phase 2 decides **with the real diff in hand**. If (B): file the follow-up, and say so out loud in the
notes *and* at the live drive — do not let a green gate imply fidelity we didn't ship.

## Confirmed latent bug (fix + prove here)
`open_file_at` (`app.rs:2736-2737`) writes `*editor.active_caret_mut() = caret` and **never touches the
anchor**. With a live editor selection, a file:line click-through jumps the caret and leaves the stale
anchor behind → `active_selection()` reports a spurious selection from the old anchor to the new caret.
The N-cursor conversion fixes it structurally (a jump collapses the set to one cursor); a regression test
must pin it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘⌥↓ (or ⌘⌥↑) is pressed in the editor, the system shall add a cursor on the line below (above) at the **goal column**, clamped to that line's length, and shall be a no-op at the last (first) line. | Pure unit on `add_cursor_vertical` + **live drive** (⌘⌥↓ → two carets render). |
| REQ-002 | WHERE a cursor walks vertically across a SHORT line, the system shall clamp it to that line's end and **restore its original column** on the next line long enough — and two cursors so moving shall NOT merge. | Pure unit: the goal column survives a short-line traversal; the set still has 2 members. **This is the invariant that makes multi-cursor survivable.** |
| REQ-003 | WHEN a character is typed with N cursors active, the system shall insert it at every cursor, and a single ⌘Z shall revert **all N insertions in one step**. | `edit_at_selections` (shipped) + a headless test + **live drive** (type → both lines get it; ⌘Z → both revert). |
| REQ-004 | WHILE N cursors are active, the system shall render **N carets** and **N selection bands**. | Pure unit on the N-range `code_view` seams (cov/MSI 100) + **live pixel capture showing two carets**. |
| REQ-005 | WHEN ⌘-click lands on an offset with no cursor, the system shall ADD a cursor there; WHEN it lands on an existing cursor, the system shall REMOVE it; the set shall never empty (removing the last is a no-op). | Pure unit on `toggle_cursor_at` (incl. the never-empty guard) + **live drive**. |
| REQ-006 | WHEN Esc is pressed with N cursors, the system shall collapse to the primary cursor; WHEN a plain (unmodified) click occurs, the system shall likewise collapse to a single cursor at the click. | Pure unit on `collapse_to_primary` + **live drive** (Esc → one caret). |
| REQ-007 | WHEN an arrow key moves the cursors, the system shall move **every** cursor, and shall re-canonicalize so two cursors that collide merge into one. | Headless test over a 2-cursor set. |
| REQ-008 | WHEN `open_file_at` jumps the caret WHILE a selection is live, the system shall NOT leave a spurious selection behind (the confirmed stale-anchor bug). | Regression test pinning the fixed behavior. |
| REQ-009 | The new pure seams (`add_cursor_vertical`, `toggle_cursor_at`, `collapse_to_primary`, the goal-column carrier, the N-range `code_view` fns) shall be pure at 100% line coverage and MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — **lock the undo decision (A or B) with the real diff size**; the goal-column carrier (on the
  `Selection`? beside the set?); the N-range `code_view` signatures; how the 45 sites are bucketed into a
  `primary()` shim vs a real rewrite; the click/⌘-click/Esc semantics; the keymap rows; the test plan.
- **P3 Implement** — the pure seams first, then the surface conversion, then render, then the gestures.
- **P3.5 Inspect** — adversarial critics on the diff. **Spawn them and WAIT** — on #296 the critic found
  three real bugs *after* the gate was green (PR-claude-a-fuzzer-proves-the-transform-not-the-invariant-001).
  Lenses: the invariant under N cursors, the goal column across short lines, undo/redo fidelity, the E0499
  borrow shape, click/drag, IME with N cursors.
- **P4 Validate** — units + headless + the gate; then the **LIVE DRIVE** (the machine is unlocked): open a
  file → ⌘⌥↓ → two carets → type → both lines → ⌘Z → both revert → Esc → one caret. Capture pixels, READ
  them, paste what was seen.
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #297.
