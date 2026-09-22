---
pipeline_id: 978a747b-c132-4d27-b786-12de34facf79
ticket: forge#256 (1743bdba-bcf0-4872-9f9d-6118ef7267bd) · local docs/planning/tickets/open/TICKET-256-clipboard.md
aar_id: cf78b64b-15eb-4705-9f81-25e3ba4bca05
title: Editor copy / cut / paste over the selection (⌘C / ⌘X / ⌘V)
type: feature
milestone: M15
references: [forge#249, forge#253, forge#255, marley_editor]
status: Phase 5 — Complete PASS
---

## Title
⌘C copies the editor selection to the system clipboard, ⌘X cuts (copy + delete), and ⌘V pastes (replacing a
selection, else inserting at the caret) — reusing #255's selection, #249's `Buffer::edit`/`text_in_range`, #253's
undo, and the existing gpui clipboard plumbing the terminal already uses.

## Scope
### In
- **An editor-clipboard branch in `on_key_down`** (before the terminal's cmd-C/cmd-V), guarded on `active_tab()
  .editor().is_some()` + `modifiers.platform && key ∈ {c, x, v}` — so ⌘C/⌘X/⌘V operate on the editor when an
  editor tab is active, else fall through to the terminal handlers (REQ-004).
- **⌘C** — copy `text_in_range(start..end)` of `active_selection()` to the clipboard; no selection → no-op.
- **⌘X** — copy the selection's text, then `edit(start..end, "")` (delete) + collapse (caret = start, anchor
  None); no selection → no-op.
- **⌘V** — read the clipboard; `edit(range, clip)` where `range` = the selection (replace) else `caret..caret`
  (insert); caret = `range.start + clip.chars().count()`, anchor None.
- **A pure `paste_edit(selection, caret, clip_chars)`** returning the edit `(start, end)` range + the post-paste
  caret (the only new pure logic; cov/MSI 100).

### Out (explicitly deferred)
- ⌘C copies the whole line when there's no selection (v1 is a no-op — see D1); multi-cursor clipboard; rich/HTML
  clipboard; a paste history/ring; ctrl-editing (ctrl-A/E/K — #257); bracketed paste (the terminal's own);
  paste-with-newline-normalization.

## Reference (§20)
**The universal editor clipboard convention (Warp's command-input + Zed's editor both do this; NO copyleft
source read — clean-room).** ⌘C copies the selected text; ⌘X copies + deletes; ⌘V inserts the clipboard,
replacing an active selection or inserting at the caret. Marley mirrors this BEHAVIOR over its own
`marley_editor::Buffer` edits + #255's `active_selection` + the same gpui `write_to_clipboard`/`read_from_clipboard`
the terminal (Warp-referenced) copy/paste already uses. ⌘C with NO selection is a no-op in v1 (D1) — a common,
safe default (some editors copy the current line; deferred, not in scope). Clean-room: observe the clipboard
BEHAVIOR; reimplement over the existing gpui plumbing.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — ⌘C / ⌘X with no selection are a NO-OP** (v1). Copy-the-line is deferred (a follow-up); a no-op is the
  safe default and matches "copy the selection" literally. A no-selection ⌘X also no-ops (nothing to cut).
- **D2 — the pure seam is `paste_edit`** (`selection: Option<(CharOffset,CharOffset)>, caret: CharOffset,
  clip_chars: usize -> (Range<CharOffset>, CharOffset)`): range = the selection (replace) else `caret..caret`;
  new caret = `range.start + clip_chars`. cov/MSI 100. COPY (`text_in_range`) + CUT (`edit(range,"")`) reuse
  tested #249 pieces — the only new pure logic is the paste range/caret.
- **D3 — the ops go in the platform-chord region of `on_key_down`** (near the terminal cmd-C/cmd-V, before them),
  NOT the #251 editor key branch (which guards `!platform`, so ⌘-chords never reach it). Each op `return`s after
  handling (so an editor tab intercepts ⌘C/⌘X/⌘V; a terminal tab falls through to the existing handlers).
- **D4 — edits go through `Buffer::edit`** (#249) so #253's undo records them (one ⌘Z undoes a paste/cut) and the
  #252 dirty ● updates. The paste/cut normalize via `active_selection` (min..max), so a backwards selection cuts/
  replaces the right span.
- **D5 — the caret collapses** after cut/paste (anchor None) — the selection is consumed.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘C is pressed with an editor selection the system shall copy the selected text to the clipboard; no selection shall be a no-op. | driven + review |
| REQ-002 | WHEN ⌘X is pressed with an editor selection the system shall copy it and delete it, collapsing to a caret. | driven + review |
| REQ-003 | WHEN ⌘V is pressed the system shall insert the clipboard text (replacing a selection if present, else at the caret) and place the caret after the inserted text. | pure unit + driven |
| REQ-004 | The editor clipboard ops shall apply only when an editor tab is active; a terminal tab shall keep the existing terminal copy/paste. | review + driven |
| REQ-005 | The paste range/caret math (`paste_edit`) shall be pure with cov/MSI 100. | gate |

## Phase Plan
- **P2 Design** — the `paste_edit` signature + the on_key_down editor-clipboard branch (⌘C/⌘X/⌘V, the guard, the
  return-before-terminal placement) + the accessor use (active_selection + active_buffer_caret_anchor_mut) + the
  test matrix + `cargo mutants --list`. Confirm §20.
- **P3 Implement** — `paste_edit` (pure) + the on_key_down branch (shim); KEEP the render/handler skip; `cargo
  check`.
- **P3.5 Inspect** — critics: copy/cut/paste correctness (the right text, the right range, collapse); the
  editor-vs-terminal guard (⌘C on a terminal still does the terminal copy); the paste range/caret (multibyte);
  no data-loss (cut deletes the RIGHT span); re-run `cargo mutants --list`.
- **P4 Validate** — pure units cov/MSI 100 (paste_edit: with/without a selection + multibyte clip) + DRIVEN
  (drag-select → ⌘C → move the caret → ⌘V pastes the text; ⌘X deletes; data-safe no ⌘S). Gate green.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close #256; archive.
