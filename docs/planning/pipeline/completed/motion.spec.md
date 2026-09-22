---
pipeline_id: ed244310-92fb-4533-8eb5-3b755d9fa6aa
ticket: forge#257 (73daf3cd-5cc6-41a8-8013-7a53700ba122) · local docs/planning/tickets/open/TICKET-257-motion.md
aar_id: e218ff82-b653-4edd-b813-0e381b9d79b5
title: Editor keybinding parity — Home/End, ⌘←→, word-wise ⌥←→, vertical, doc
type: feature
milestone: M15
references: [forge#250, forge#251, forge#255, marley_editor]
status: Phase 5 — Complete PASS
---

## Title
Give the editor standard mac caret motion — Home/End (line), ⌥←→ (word), ⌘←→ (line), ⌘↑↓ (document), Up/Down
(vertical) — each with a SHIFT-variant that EXTENDS the #255 selection, reusing `marley_editor::movement` and
generalizing #255's shift-extend to any target.

## Scope
### In
- **Vertical motion** — `Key::Up`/`Key::Down` + a pure `movement::move_up`/`move_down` (the same column on the
  adjacent row via `line_col`/`line_start`/`line_text`, clamped: row to `[0, len_lines-1]`, col to the target
  line's char count). v1 uses the CURRENT column each move (no goal-column memory).
- **A pure `extend_or_go(anchor, caret, target, shift)`** generalizing #255's shift-extend to a PRE-COMPUTED
  target: `shift → (Some(anchor.unwrap_or(caret)), target)`; `!shift → (None, target)`. The app computes the
  target via the movement fn, then calls this.
- **The #251 on_key_down editor branch** handles the `!platform` motions (Home/End, ⌥←→ word, Up/Down) the same
  way it handles Left/Right (#255): compute the target via `move_line_home`/`move_line_end`/`move_word_left`/
  `move_word_right`/`move_up`/`move_down` → `extend_or_go` → set anchor+caret. `key_from_keystroke` maps
  `up`/`down` → `Key::Up`/`Down` (Home/End/⌥←→ already map).
- **A platform-chord editor branch** (mirror #256's ⌘C/⌘X/⌘V branch) for the `platform` motions: ⌘← / ⌘→ (line
  start/end), ⌘↑ / ⌘↓ (document start `CharOffset::zero` / end `len_chars`) — compute the target → `extend_or_go`
  (with shift) → set anchor+caret + return. Editor-tab-guarded.

### Out (explicitly deferred)
- Goal-column memory across a run of vertical moves (v1 uses the current column); page-up/down; smart-home
  (first-non-whitespace); find-motion / go-to-line; multi-cursor motion; word-wise DELETE (⌥⌫).

## Reference (§20)
**The universal macOS text-editor caret-motion convention (Warp's command-input follows it; NO copyleft source
read — clean-room).** ⌘←/⌘→ = line start/end, ⌥←/⌥→ = previous/next word, Home/End = line start/end, ⌘↑/⌘↓ =
document start/end, ↑/↓ = the same column on the adjacent row, and Shift+<motion> extends the selection from a
fixed anchor. These are the OS-standard mac editing bindings (Cocoa text system) — universal, not app-specific;
Marley reimplements them over its own `marley_editor::movement` + #255's selection model. No fresh capture (the
chords are the OS convention; the vertical column math reuses #250's `line_col`/grid — observed capture
`docs/warp_architecture/observed/250-warp-monospace-grid-caret.png`). Clean-room: observe the BINDINGS; reimplement.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the pure additions live in `movement.rs`:** `move_up`/`move_down` + `extend_or_go`. All other motions
  reuse the existing `move_word_left/right` + `move_line_home/end` (+ doc = `CharOffset::zero` / `len_chars`).
- **D2 — `move_up`/`move_down`** = `(row,col)=line_col(caret); target_row = row±1` clamped to `[0, len_lines-1]`;
  `new_col = col.min(line_text(target_row).chars().count())`; `line_start(target_row) + new_col`. Up on row 0 /
  down on the last row → stays on that row (clamped). v1: the current col (no goal column).
- **D3 — `extend_or_go(anchor, caret, target, shift) -> (Option<CharOffset>, CharOffset)`:** `shift → (Some(
  anchor.unwrap_or(caret)), target)` [extend]; `!shift → (None, target)` [move + collapse]. Char-arrows KEEP
  #255's `extend_or_move` (its collapse-TO-EDGE nuance for an unshifted arrow on a selection); the new motions
  use `extend_or_go` (go straight to the target). (`extend_or_move` MAY be refactored to delegate to
  `extend_or_go` for its shift + no-selection paths — design's call; keep behavior identical.)
- **D4 — the !platform motions route through the #251 branch** (a new match arm alongside Left/Right): compute
  the target + `extend_or_go`. `apply_editor_key` is UNCHANGED (the branch intercepts the motions before it; it
  keeps handling Char/Backspace/Enter).
- **D5 — the platform ⌘-motions get a new editor platform-chord branch** (near the #256 clipboard branch): `⌘←`=
  line_home, `⌘→`=line_end, `⌘↑`=doc start, `⌘↓`=doc end → `extend_or_go` → return. Editor-tab-guarded; a
  terminal tab keeps its ⌘-arrow behavior.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Home/End and ⌥←/⌥→ shall move the caret to the line start/end and the previous/next word boundary (reusing `movement`). | pure unit + driven |
| REQ-002 | Up/Down shall move the caret to the same column on the adjacent row, clamped to the row range and the target line's length. | pure unit + driven |
| REQ-003 | ⌘←/⌘→ shall move to line start/end and ⌘↑/⌘↓ to document start/end. | driven + review |
| REQ-004 | The Shift-variant of any motion shall EXTEND the selection from a fixed anchor (#255); the unshifted variant shall collapse and move. | pure unit + driven |
| REQ-005 | `move_up`/`move_down` + `extend_or_go` shall be pure with cov/MSI 100. | gate |

## Phase Plan
- **P2 Design** — `move_up`/`move_down` + `extend_or_go` signatures + the `key_from_keystroke` up/down mapping +
  the #251-branch motion arms + the platform-chord ⌘-motion branch + the `extend_or_move`↔`extend_or_go`
  reconciliation + the test matrix + `cargo mutants --list`. Confirm §20.
- **P3 Implement** — `move_up`/`move_down` + `extend_or_go` (pure) + the key mapping + the two branch changes
  (shim); KEEP the render skip; `cargo check`.
- **P3.5 Inspect** — critics: the vertical col-clamp (row/col edges, multibyte); `extend_or_go` (extend vs
  collapse); the #255 char-arrow behavior UNCHANGED; the platform-branch guard; re-run `cargo mutants --list`.
- **P4 Validate** — pure units cov/MSI 100 (move_up/down + extend_or_go + the reconciliation) + DRIVEN (Home/End/
  ⌥←→/Up/Down move the caret; Shift+motion highlights; ⌘←→ line). Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #257; archive.
