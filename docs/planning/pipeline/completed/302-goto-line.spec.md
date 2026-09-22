---
pipeline_id: d44b0710-76ed-4f74-9f50-fb1b23afe102
ticket: forge#302 (8b8263f4-46ef-4251-b187-d037acf1c095) · local docs/planning/tickets/open/TICKET-302-goto-line.md
aar_id: 215921e4-4469-4e60-8865-983f05ce6225
status: Phase 5 — Complete PASS
title: Go to line (⌃G) — a two-field parse, an inline overlay, a centered jump
type: feature
milestone: M19
references: [the renaming_symbol inline-draft overlay (app.rs:214/:11990 — THE smallest precedent), text_input_blocked (app.rs:7488 — the leak gate every overlay needs), the #212 parse_line_col shape (the input grammar's prior art), scroll_editor_to_row CENTERS already (ScrollStrategy::Center, app.rs:11031), EditorSurface::set_single_caret (editor_surface.rs:253), ⌃G verified FREE everywhere (terminal ⌃G = raw BEL to the PTY, untouched — Editor-scoped), the NavStack 5-site push idiom]
---

## Title
⌃G opens a small "Go to line" overlay; type `50` or `50:12`, Enter jumps — caret placed, view centered —
Esc cancels and restores the prior caret. In a 2000-line file today you scroll. The jump machinery already
exists end-to-end (`scroll_editor_to_row` centers by construction; `set_single_caret` places); this ticket
is the parse, the overlay, and the restore — deliberately the smallest ticket on the shelf.

## Scope
### In
- **The pure seams (cov/MSI 100):** `parse_goto(input) -> Option<(usize, Option<usize>)>` — 1-based line,
  optional 1-based col; accepts `N` and `N:C`; rejects empty / non-digit / bare `:` / trailing `:` / `0`
  (a PARTIAL input mid-typing must be a clean `None`, never a panic — the overlay renders the miss as an
  inert state, not an error). The grammar mirrors the shipped #212 `parse_line_col` compiler-ref shape
  (verify the exact fn at promotion; adopt its digits/clamp conventions rather than invent).
  `clamp_goto(line, col, buffer) -> CharOffset` — past-EOF clamps to the LAST line; a col past the line's
  end clamps to line end; 1-based→0-based at this seam ONLY (the off-by-one lives in one place). Note the
  ropey phantom row: `"a\nb\n"` has 3 lines and line 3 is legitimately targetable (it's where the caret
  goes for "end of file").
- **The overlay = the `renaming_symbol` shape copied whole** (the smallest shipped precedent): ONE state
  field `goto_line: Option<GotoDraft { input: String, origin: (CharOffset, f32) }>` (the origin captures
  the caret AND scroll for Esc-restore); an inline `match` arm HIGH in the overlay ladder (escape → restore
  + close; enter → commit; backspace → pop; single-char digits/`:` → push; **`"space"` handled explicitly**
  — gpui names the spacebar, the #177 lesson every overlay re-learns); `cx.stop_propagation()` after; and
  **one line in `text_input_blocked`** (app.rs:7488) so the IME/platform-text fallback cannot leak digits
  into the buffer — the leak is the classic bug here, pinned by a drive.
- **The jump:** `set_single_caret(clamped_offset)` → `scroll_editor_to_row(row)` (already
  `ScrollStrategy::Center` — no new centering code) → **PUSH the NavStack** (a far intra-file jump is
  exactly the jump_to_sticky_header class; the 5-site origin-capture idiom, so ⌃- returns). Esc restores
  the captured origin WITHOUT a push (a cancel is not a navigation).
- **Live preview (cheap, observed):** while typing, the view scrolls to the would-be target (the caret
  does NOT move until Enter); Esc restores the original scroll. This is what VS Code/Zed do and it is one
  `scroll_editor_to_row` call per keystroke — no extra machinery.
- **The chord:** ⌃G `(F,T,F,F,"g")` — verified FREE in every context; on the TERMINAL, ⌃G today streams
  raw BEL (0x07) to the PTY via the raw route, which an Editor-scoped row leaves untouched. Roster 67→68,
  scoped 22→23 (individual assert first).
### Out (explicitly)
- `:C` column syntax beyond simple 1-based clamp (no `+N` relative, no percentages); go-to-BYTE/offset;
  a persistent line-number input in the status bar; any change to the terminal's ⌃G/BEL behavior.

## Reference (§20)
VS Code / Zed = OBSERVED (⌃G; `N:C`; clamp-never-error; live preview scroll; Esc restores). The overlay +
parse are Marley-original over shipped seams.

### Prior art
1. **Behavior maps / observed** — the flow above, incl. clamp-past-EOF-never-error and the live preview.
2. **Published material** — none needed.
3. **OUR OWN CODE — everything but the parse already exists:** the #212 `parse_line_col` owns the input
   grammar's shape (adopt); `scroll_editor_to_row` already centers AND drags the horizontal twin
   (app.rs:11017 — the #336 hook-the-primitive lesson, inherited free); `set_single_caret` places;
   `renaming_symbol` is the overlay template; `text_input_blocked` is the known leak gate. No new deps,
   no new render machinery beyond one small chip.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-CLAMP-NEVER-ERROR** — past-EOF/past-EOL clamp; only a non-parse is inert.
- **D-ONE-BASED-AT-ONE-SEAM** — the 1↔0 conversion lives in `clamp_goto` only.
- **D-PUSH-ON-COMMIT-ONLY** — Enter pushes the NavStack; Esc restores silently.
- **D-LIVE-PREVIEW-SCROLL** — scroll tracks typing; the caret moves on Enter only.
- **D-COPY-THE-RENAME-OVERLAY** — no new overlay abstraction; the smallest shipped shape, verbatim.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | parse `50` → line 50; `50:12` → line 50 col 12; reject ``/`abc`/`50:`/`:5`/`0` as inert None | pure truth table |
| REQ-002 | clamp a past-EOF line to the last line and a past-EOL col to line end (never an error) | pure |
| REQ-003 | place the caret at the target and CENTER the view on Enter | headless |
| REQ-004 | restore the prior caret AND scroll on Esc (the origin capture) | headless |
| REQ-005 | push the NavStack on commit so ⌃- returns; NOT on Esc | headless |
| REQ-006 | keep typed digits out of the buffer while the overlay is open (text_input_blocked + stop_propagation) | headless — the leak row |
| REQ-007 | preview-scroll to the would-be target while typing, caret unmoved until Enter | headless |
| REQ-008 | resolve ⌃G on an editor tab only; the terminal's raw ⌃G/BEL path is byte-identical | keymap unit + terminal drive |

## Phase Plan
P2 confirm the #212 parse fn's exact shape (adopt vs mirror) + the origin-capture fields (caret + which
scroll representation) + the chip's render slot; P3 pure parse/clamp first (truth tables), then the
overlay + jump + chord; P3.5 critics on the restore path (scroll vs caret independence), the leak gate,
the phantom-row target, the 1-based seam; P4 tables + drives + gate; P5 docs.
Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
