# Block navigation in display rows — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-629-block-navigation-in-display-rows.md
- **Pipeline spec:** 629-block-navigation-in-display-rows.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, third batch (#628 to #630, and #466): T5 stage two, and fish.
- **Recall (§18.3):**
  - `block_scroll`, the scrollback fraction (bookmark ticks) and search all count grid lines today.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):**
  - AD-claude-628: a header takes exactly its prompt's rows; no row moves.
  - AD-claude-529: the pinned header is an element over row 0, not a row.
  - AD-claude-473: the block keys put a block's first line at the top and select nothing.
  - Brain (consultation 3ea3a69722024284a3ec85891927a547): nothing on this seam.
- **Re-binding:** the queued draft read everything "through #628's map"; #628 shipped without one,
  so this slice is what still shows the shell's prompt (the pinned header's `$ `, search matches
  in a hidden prompt) and a check that the rest holds.

### Design
- **`sticky_header.rs`:** the command reads `$ <first>` while `block_headers` is `ShellPrompt` and
  `<first>` while `Native`.
- **`terminal_element.rs`:** after the search loop, the count of search matches
  (`marley_search_matches`); in #628's hunk, once the hidden rows are known, the first that many
  ranges whose start row (`start().line + display_offset`) is hidden are dropped.

### File manifest
- Marley: `crates/marley_workbench/src/sticky_header.rs`;
  `script/e2e/629-block-navigation-in-display-rows.sh`.
- Zed: `crates/terminal_view/src/terminal_element.rs` (the row widened first).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | three `seq 1 40` blocks; Escape (keys to the shell); Ctrl+Up twice | `top.png` |
| REQ-002 | scroll down a few lines into the middle block's output | `pinned.png` |
| REQ-003 | Ctrl+Shift+B on the first block (selected by Ctrl+Up), back to the bottom | `ticks.png` |
| REQ-004 | Ctrl+F, `seq` | `search.png` |

## Phase 2 — Code (2026-10-01)
- **Built:** `sticky_header.rs` drops `$ ` while the headers are native; in `terminal_element.rs`,
  `marley_search_matches` after the search loop and, in #628's hunk, the search matches whose
  first row is a hidden prompt row left out of the highlighted ranges.
- **Review:** the selection is pushed after the matches, so the count keeps it whatever its rows;
  the filter runs after the hidden rows are known and before the ranges go into the layout state.
- **Rustc found:** the range's line is already an `i32`.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-10-01)
- **Scenario:** `script/e2e/629-block-navigation-in-display-rows.sh` (sway): bash with
  `PS1='[the prompt]\n$ '`, the headers on; `seq 1 40`, `seq 101 140`, `seq 201 240`, `echo the
  seq ends`, then Escape so the keys reach the grid.
- **First run:** the search came last, after Ctrl+Up had selected a block, and #559 held it to
  that block (1/1, behind the pinned header): not a check of REQ-004. The search moved first, with
  no block selected.
- **Second run:**
  - `search` (REQ-004): `seq`, 3/5: the `echo the seq ends` header and the pinned `seq 201 240`
    show no highlight; the output row's match is highlighted at the bottom edge. A comparison run
    with the headers off (a scratch copy of the scenario, not kept) shows `seq` highlighted in
    `$ echo the seq ends`, the match the headers now leave out.
  - `top` (REQ-001): three Ctrl+Up: `seq 101 140`'s header on the top row with its check and
    actions, its output under it.
  - `pinned` (REQ-002): three lines down: the pinned row reads `seq 101 140`, no `$ `, with its
    check and arrow.
  - `ticks` (REQ-003): Ctrl+Shift+B, then back to the live screen: the tick at the right edge about
    a third of the way down, where the middle block starts in the scrollback.
- **Pre-existing, not in scope:** with Zed's search bar open, the last row of a scrolled view
  shows its highlight but not its text, with the headers on or off (the comparison run).

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide (Blocks); `marley_workbench.md` (the sticky header);
  the plan's T5 row; the ledger row of `terminal_element.rs`.
- **Knowledge:** L-claude-629-a-search-after-the-block-keys-is-held-to-the-selected-block-001.
  No bug in Code; the first run's search was a scenario order, not a defect.
- **Brain:** the consultation closed with `brain decide`.
