---
pipeline_id: dc812f5e-7175-4b19-8f53-b391bc9c874e
ticket: forge#44 (e352a858-a281-4757-91a7-10a7f630019d) · local docs/planning/tickets/open/TICKET-044-copy-selection.md
aar_id: 5203985e-387c-4339-970b-ecbdc34b15ff
status: Phase 5 — Complete PASS
title: copy the selection (cmd-C)
type: feature
milestone: M1.G
references:
  - crates/marley_app/src/text_selection.rs (extend — selected_text + copy_payload + row_slice)
  - crates/marley_app/src/app.rs (~612 — the cmd-C handler beside cmd-V)
  - docs/specs/SPEC-app-shell.spec.md
---

## Title
#42 gave paste (cmd-V); this gives COPY. cmd-C writes the current text selection (#43) to the
clipboard — the #1 daily-driver gap (you couldn't copy terminal output at all). The copied text is
built on #43's `row_selection`, so it matches the highlight exactly.

## Scope
### In
- `crates/marley_app/src/text_selection.rs` (PURE, gpui-free — cov/MSI 100), extending #43:
  - `row_slice(s: &str, from: usize, to: usize) -> String` — the char-safe slice `[from, to)`,
    clamped to the string's char length (the helper #43 deferred; `to <= from` → empty).
  - `selected_text(rows: &[String], sel: Selection) -> String` — for each `row` in
    `start.row..=end.row`, reuse `row_selection(sel, row, rows[row].chars().count())` → `row_slice` →
    join with `\n`. So the copied text is EXACTLY the highlighted span (#43 uses the same
    `row_selection`).
  - `copy_payload(rows: &[String], selection: Option<Selection>) -> Option<String>` — `None` when
    there is no selection OR the selected text is empty (cmd-C with nothing selected is a no-op, NOT
    an empty clipboard write); else `Some(text)`. The empty-guard is the tested decision.
- `crates/marley_app/src/lib.rs` — export `copy_payload` if the shim needs it (it's in-crate).
- `crates/marley_app/src/app.rs` (SHIM, ~612 beside the cmd-V handler): a cmd-C case builds the
  content `Vec<String>` from the focused pane's blocks (each block's command header + its
  `output_styled` lines, matching the render row order + the selection indices) → `if let Some(text)
  = copy_payload(&rows, state.selection) { cx.write_to_clipboard(ClipboardItem::new_string(text)) }`.
  The selection stays highlighted after copy (Warp keeps it).
- SPEC-app-shell (the copy clause + Mutation-Targets). CHANGELOG + arch doc.

### Out (explicitly deferred)
- A right-click "Copy" context menu (cmd-C covers it for the first cut; the menu is a later cut).
- Copy-on-select (auto-copy the selection) — a preference for later. Copying the prompt row / the
  alt-screen grid — the first cut copies the Block-list rows (matching #43's selection scope).
  Rich-text / HTML clipboard (plain text only).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `selected_text` REUSES #43's `row_selection` for the per-row span, so the copied text and the
  drag-highlight are guaranteed identical (one geometry, no drift).
- D2 — `copy_payload` returns `None` for an absent OR empty selection — cmd-C then does nothing
  (never clobbers the clipboard with `""`). This is the tested empty-guard.
- D3 — PURE: `row_slice` + `selected_text` + `copy_payload` (cov/MSI 100); the cmd-C keystroke
  handling + the clipboard write + the rows-build are SHIM (app.rs, masked). gpui
  `cx.write_to_clipboard(ClipboardItem::new_string(text))` mirrors #42's read.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `row_slice(s, from, to)` is called, it shall return the chars `[from, to)` of `s`, clamped to its char length (`to ≤ from` → empty), respecting char boundaries (multi-byte). | unit (`"hello"[1..3]`→`"el"`; clamp; empty; `café`) |
| REQ-002 | WHEN `selected_text(rows, sel)` is called, it shall return the selected text — each row's `row_selection` span sliced, joined with `\n` — matching what the #43 highlight marks. | unit (single/multi row) |
| REQ-003 | WHEN `copy_payload(rows, selection)` is called with no selection or an empty selected span, it shall return `None`; otherwise `Some(the selected text)`. | unit (`None`/`None`/`Some`) |
| REQ-004 | WHEN cmd-C is pressed with a non-empty selection, the app shall write the selected text to the clipboard; with no selection it shall do nothing. | shim + masked visual (chad: select → cmd-C → paste round-trips) |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `row_slice` + `selected_text` + `copy_payload`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `row_slice`/`selected_text`/`copy_payload` shapes (reusing `row_selection`),
  the cmd-C shim + the rows-build, the SPEC clause + mutation targets.
- **P3 Implement** — the 3 pure fns + the app.rs cmd-C handler + spec + CHANGELOG.
- **P3.5 Inspect** — critics: `row_slice` char-boundary + clamp, `selected_text` reuses row_selection
  (matches highlight), `copy_payload` empty-guard, the shim rows-build order, no #43 regression.
- **P4 Validate** — the row_slice/selected_text/copy_payload unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #44.
