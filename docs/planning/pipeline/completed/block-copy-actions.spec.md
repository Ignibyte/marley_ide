---
pipeline_id: b837406c-308d-469f-96f0-28cca378c788
ticket: forge#45 (22c511c3-3508-404c-9cbb-71fd1a79e00b) · local docs/planning/tickets/open/TICKET-045-block-copy-actions.md
aar_id: bcc0a203-3260-49d3-a586-d9199ff6e646
status: Phase 5 — Complete PASS
title: Block actions — copy command / copy output
type: feature
milestone: M1.G
references:
  - crates/terminal_blocks/src/block.rs (NEW BlockCopy enum + Block::copy_text)
  - crates/marley_app/src/app.rs (the block-header hover copy affordances — shim)
  - docs/specs/SPEC-terminal-blocks.spec.md (R29)
  - docs/specs/SPEC-app-shell.spec.md (R49)
---

## Title
Warp's SIGNATURE block interaction: hovering a command Block reveals actions to copy its COMMAND or
its OUTPUT to the clipboard — no manual selection needed. This is why Warp's blocks beat scrollback.

## Scope
### In
- `crates/terminal_blocks/src/block.rs` (PURE — cov/MSI 100, with the Block model):
  - `BlockCopy { Command, Output }` — a small copy-able `enum` (Debug/Clone/Copy/PartialEq/Eq).
  - `impl Block { pub fn copy_text(&self, what: BlockCopy) -> String }` — `Command` →
    `self.command.clone()`; `Output` → `self.output_text()` (exists — the plain `\n`-joined,
    trailing-trimmed projection). The pure decision: WHICH text an action copies.
- `crates/terminal_blocks/src/lib.rs` — export `BlockCopy`.
- `crates/marley_app/src/app.rs` (SHIM): the block-header render (the #43-highlight header ~947)
  gains hover-revealed copy affordances — a copy-command + a copy-output glyph, shown on the header's
  `.hover` (extends #39's block hover) — each a clickable `div` whose `on_mouse_down` →
  `cx.write_to_clipboard(ClipboardItem::new_string(block.copy_text(BlockCopy::{Command,Output})))`
  (reuses #44's clipboard write). The glyphs sit at the header's right edge.
- SPEC-terminal-blocks R29 (copy_text + Mutation-Targets); SPEC-app-shell R49 (the hover-action shim).
  CHANGELOG + arch doc.

### Out (explicitly deferred)
- A `Both`/`CommandAndOutput` variant (copy command+output together) — later; two clear actions first.
- A right-click context menu (the hover glyphs cover it). Copy-to-a-file / share. An icon font — the
  affordance uses a text glyph (e.g. ⧉) for the first cut.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `copy_text` lives on `Block` (terminal_blocks), with the model + `output_text` it reuses — a
  pure method (cov/MSI 100), not app-shell logic. `BlockCopy` is the tested arm selector.
- D2 — `Output` reuses the EXISTING `output_text()` (no new assembly) — so a block-action copy and a
  drag-copy of the same output agree.
- D3 — The hover-reveal + the click handlers + the glyph are SHIM (app.rs, masked). PURE surface =
  `copy_text` (the Command/Output arm). Clipboard write reuses #44's `ClipboardItem::new_string`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `block.copy_text(BlockCopy::Command)` is called, it shall return the block's command string. | unit (a block with a known command) |
| REQ-002 | WHEN `block.copy_text(BlockCopy::Output)` is called, it shall return the block's `output_text()` (the plain `\n`-joined, trailing-trimmed output). | unit (a block with multi-line output) |
| REQ-003 | WHEN a block has a command AND output, `copy_text(Command)` and `copy_text(Output)` shall return DIFFERENT, correct strings (the arms don't collapse). | unit (asserts both arms distinctly) |
| REQ-004 | WHEN a block is hovered, the render shall reveal copy-command + copy-output affordances; clicking one shall write `copy_text(that)` to the clipboard. | shim + masked visual — chad-verified |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `copy_text`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the `BlockCopy` enum + `copy_text` shape, the header hover-affordance shim, the SPEC
  clauses + mutation targets.
- **P3 Implement** — the enum + method + export + the app.rs hover affordances + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the arm selection (Command vs Output not swapped/collapsed), Output ==
  output_text (agrees with a drag-copy), the shim click wiring, no block-model regression.
- **P4 Validate** — the copy_text unit tests + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #45.
