# Brand-scrub catalog — `crates/marley_app/src/app.rs`

Goal: **NO Warp / Zed brand mentions in the Marley source.** The reference docs
(`docs/warp_architecture/`, `docs/zed_architecture/`) are OUT of scope and keep their mentions.

Scope of this file: `crates/marley_app/src/app.rs` only. Catalog is read-only intel — a follow-up
applies the rewords under the green gate.

Detector: `grep -niwE 'warp|zed' crates/marley_app/src/app.rs`

## Summary

- **36 lines** flagged; **39 whole-word `Warp` tokens** total (lines 404 / 416 / 6363 each carry two).
- **0 `Zed` mentions.**
- **Category: every hit is a `comment`** (`//` or `///`). No identifiers, strings, test-data, or
  theme-names — the scrub is comment-only, so it cannot change behavior (mutation/coverage unaffected).
- **0 `KEEP?` flags** — none of these is a clean-room / provenance statement (the only load-bearing
  reason to keep a brand name). Every mention is a design-intent citation ("Warp-style", "Warp parity",
  "the Warp arrangement") whose technical meaning survives the reword.
- **3 lines cite Warp as a metric/behavior authority** (2906, 5733, 5980) — reword preserves the
  rationale but review the wording; see the Notes column.

## Reword rule applied

Keep the ticket numbers and the technical meaning; drop the brand name; describe the
behavior/design in Marley's own terms. "(Warp parity)" tags become plain `#NNN:` where the parity
claim added nothing beyond the ticket ref.

## Catalog

| file:line | category | CURRENT text | PROPOSED reword | Notes |
|---|---|---|---|---|
| app.rs:366 | comment (`///`) | `/// The top command-bar height (M7 #132) — the Warp-style bar at the very top holding the global search (and,` | `/// The top command-bar height (M7 #132) — the command bar at the very top holding the global search (and,` | |
| app.rs:382 | comment (`///`) | `/// Height (px) of a pane's title bar (M5 #108) — the Warp "tabs up at the top" strip.` | `/// Height (px) of a pane's title bar (M5 #108) — the "tabs up at the top" strip.` | |
| app.rs:404 | comment (`//`) | `// #195 (Warp parity): match Warp's ~13pt terminal density; the cell metric derives from this.` | `// #195: the ~13pt terminal density (the cell metric derives from this).` | 2 tokens on the line; matches the task's own example reword. |
| app.rs:416 | comment (`//`) | `// #216 (Warp parity): py_1 (was py_2) — a compact ~22px header band matching Warp's section` | `// #216: py_1 (was py_2) — a compact ~22px header band matching the section` | 2 tokens; line 417 ("headers (the panel text shrank…)") is unchanged and stays coherent. |
| app.rs:460 | comment (`///`) | `/// The active/hover fill for a left-rail row (#219, Warp parity). An accent WASH — distinct from the` | `/// The active/hover fill for a left-rail row (#219). An accent WASH — distinct from the` | |
| app.rs:1135 | comment (`//`) | `// #127 (finale): the right side starts FREE for the pane grid — the Warp arrangement. The` | `// #127 (finale): the right side starts FREE for the pane grid — the terminal-first arrangement. The` | |
| app.rs:2906 | comment (`//`) | `// (Warp); the "Files" caption_header sets its own Caption(11), so only the rows inherit this.` | `// the "Files" caption_header sets its own Caption(11), so only the rows inherit this.` | Warp cited as the 12px Nav-size authority (prev line 2905 ends "Nav = 12"). Reword drops "(Warp); "; consider ending 2905 with a period (`Nav = 12.`) so the sentence reads clean. |
| app.rs:3842 | comment (`//`) | `// #161 (Warp semantics): panes close first; the last pane closes the TAB; a cockpit/code` | `// #161 (close semantics): panes close first; the last pane closes the TAB; a cockpit/code` | "(Warp semantics)" → "(close semantics)" keeps the "this is the close-ordering rule" meaning. |
| app.rs:4981 | comment (`//`) | `// #219 (Warp parity): the Project row is clickable — same hover feedback as the` | `// #219: the Project row is clickable — same hover feedback as the` | |
| app.rs:5061 | comment (`//`) | `// #230: the rail Tab (session) row — the one px(13) rail outlier → Nav (12), Warp.` | `// #230: the rail Tab (session) row — the one px(13) rail outlier → Nav (12).` | Trailing ", Warp." was the 12px justification; the ticket + Nav(12) carry it. |
| app.rs:5101 | comment (`//`) | `// #219 (Warp parity): a visible accent-wash highlight (was bg(surface) =` | `// #219: a visible accent-wash highlight (was bg(surface) =` | |
| app.rs:5164 | comment (`//`) | `// #219 (Warp parity): the same visible accent-wash highlight + rounded box +` | `// #219: the same visible accent-wash highlight + rounded box +` | |
| app.rs:5732 | comment (`//`) | `// #217 (Warp parity): a hover group so the block's ⧉/↻ affordances` | `// #217: a hover group so the block's ⧉/↻ affordances` | |
| app.rs:5733 | comment (`//`) | `// reveal only when THIS header is hovered (Warp hides them at rest).` | `// reveal only when THIS header is hovered (hidden at rest).` | Warp cited as the hide-at-rest-behavior authority; the behavior itself survives. |
| app.rs:5763 | comment (`//`) | `// Warp's signature block actions (R49): hover-brightened copy-command +` | `// The signature block actions (R49): hover-brightened copy-command +` | |
| app.rs:5772 | comment (`//`) | `// #217 (Warp parity): hidden at rest, revealed on block-hover.` | `// #217: hidden at rest, revealed on block-hover.` | |
| app.rs:5793 | comment (`//`) | `// #217 (Warp parity): hidden at rest, revealed on block-hover.` | `// #217: hidden at rest, revealed on block-hover.` | |
| app.rs:5818 | comment (`//`) | `// #217 (Warp parity): hidden at rest, revealed on block-hover.` | `// #217: hidden at rest, revealed on block-hover.` | |
| app.rs:5980 | comment (`//`) | `// Warp shows autosuggest only on the active input — mirrors the #186 find-highlight).` | `// autosuggest belongs only on the active input — mirrors the #186 find-highlight).` | Warp cited as the active-input-autosuggest authority; prev line 5979 ends "and", stays coherent. |
| app.rs:5994 | comment (`//`) | `// #218 (Warp parity): no filled strip / rounding — the prompt sits directly on` | `// #218: no filled strip / rounding — the prompt sits directly on` | |
| app.rs:6004 | comment (`//`) | `// #218 (Warp parity): the cwd is a dim breadcrumb (muted); the git` | `// #218: the cwd is a dim breadcrumb (muted); the git` | |
| app.rs:6015 | comment (`//`) | `// #218 (Warp parity): the caret is a BLOCK cursor (was a 2px accent bar) —` | `// #218: the caret is a BLOCK cursor (was a 2px accent bar) —` | |
| app.rs:6025 | comment (`//`) | `// #220 (Warp parity): the block cursor is solid on the FOCUSED pane;` | `// #220: the block cursor is solid on the FOCUSED pane;` | |
| app.rs:6224 | comment (`//`) | `// #108: the pane's title bar — a Warp-style strip at the top (icon + name + ⋮ + ×).` | `// #108: the pane's title bar — a strip at the top (icon + name + ⋮ + ×).` | |
| app.rs:6363 | comment (`//`) | `// #221 (Warp parity): a rounded, framed Warp-style card.` | `// #221: a rounded, framed card.` | 2 tokens on the line. |
| app.rs:6374 | comment (`//`) | `// The selection-highlight row (R32). #222 (Warp parity): the shortcut renders as keycap` | `// The selection-highlight row (R32). #222: the shortcut renders as keycap` | |
| app.rs:6463 | comment (`//`) | `// #221 (Warp parity): rounded, framed card.` | `// #221: rounded, framed card.` | |
| app.rs:6510 | comment (`//`) | `// #221 (Warp parity): rounded, framed card.` | `// #221: rounded, framed card.` | |
| app.rs:6551 | comment (`//`) | `// #221 (Warp parity): rounded, framed card.` | `// #221: rounded, framed card.` | |
| app.rs:6581 | comment (`//`) | `// #221 (Warp parity): rounded, framed card.` | `// #221: rounded, framed card.` | |
| app.rs:6615 | comment (`//`) | `// #221 (Warp parity): rounded, framed card.` | `// #221: rounded, framed card.` | |
| app.rs:6837 | comment (`//`) | `// #221 (Warp parity): rounded card (already framed).` | `// #221: rounded card (already framed).` | |
| app.rs:6910 | comment (`//`) | `// #221 (Warp parity): rounded card (already framed).` | `// #221: rounded card (already framed).` | |
| app.rs:7124 | comment (`//`) | `// #142/#192: the working directory + git branch (Warp-style context) moved OUT of the top-right` | `// #142/#192: the working directory + git branch (prompt context) moved OUT of the top-right` | |
| app.rs:7313 | comment (`//`) | `// #221 (Warp parity): rounded card (already framed).` | `// #221: rounded card (already framed).` | |
| app.rs:7407 | comment (`//`) | `// #221 (Warp parity): rounded card (already framed).` | `// #221: rounded card (already framed).` | |

## Notes for the applier

- All edits are comment-only — no code tokens change, so the green gate (build / test / mutation /
  coverage) is unaffected. A post-apply re-run of the detector should return **0 lines**.
- The repeated `#221 … rounded … card` rows (6463 / 6510 / 6551 / 6581 / 6615 / 6837 / 6910 / 7313 /
  7407) share identical text; a `replace_all` per exact string handles each of the two variants
  ("rounded, framed card." vs "rounded card (already framed).").
- Lines with two tokens (404 / 416 / 6363): the single proposed reword removes both — no second pass
  needed.
- Line 2906: the reword removes the `(Warp); ` prefix that continued from "Nav = 12" on line 2905;
  ending 2905 with a period keeps the sentence tidy (optional, cosmetic).
