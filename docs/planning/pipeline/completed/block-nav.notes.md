# clear (cmd-K) + jump-to-block nav (⌘↑/⌘↓) — Notes

- **Forge ticket:** #48 `be79e95e-979b-4850-8a9c-2a73ae89211d`
- **AAR:** `054d392c-b1b1-48a4-b0ca-55b2f566e103`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-048-block-nav.md
- **Pipeline spec:** block-nav.spec.md

## Phase 1 — Plan
- **Request:** forge #48 (M1.G "Block Workflows & Selection" seq-6, FINALE, auto-approved) — cmd-K
  clear + ⌘↑/⌘↓ block navigation.
- **Classification / tier:** work pipeline, `feature`, PURE (nav.rs — cov/MSI 100) + a SHIM (keymap
  bindings + the jump/clear dispatch). marley_app only.
- **Discovery (§18):**
  - The viewport (#32) — `top: usize` (private) + `visible`/`scroll_up`/`scroll_down`; no public
    top-setter, so the jump reuses #47's `scroll_focused_to_row(target)` (already a shim helper).
  - The keymap (#M1.B) — a `(KeyBinding::chord(cmd,ctrl,alt,shift,key), action)` Vec + pure
    `action_for` (+ a per-binding test assertion); the dispatch is `dispatch_action` (app.rs). Adds
    cmd-up/down/k.
  - The session — `write_bytes` (for the `\x0c` clear); `reset_term` is private. No public block-drop,
    so cmd-K sends the shell clear (Ctrl-L), matching a real terminal.
  - `content_row_texts`/`content_rows` (from #44/#32) give the per-block line structure; the shim maps
    blocks → output-line-counts for `block_boundary_rows`.
  - Deps #32 (viewport) + #M1.B (keymap) — done.
- **Decisions:** D1–D3 in the spec (line-counts not BlockList; no-wrap; cmd-K sends `\x0c`).
- **Open questions for Design:** the jump-scroll (reuse scroll_focused_to_row(target) — snaps the
  boundary into view); cmd-K = `\x0c` (Ctrl-L, shell redraws) vs a block-drop (deferred); whether the
  boundary line-counts come from `output_styled().len()` per block (yes — matches the render walk).
- **AAR id:** `054d392c-b1b1-48a4-b0ca-55b2f566e103`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/nav.rs` (NEW, gpui-free)
```rust
/// The content-row index where each block's header starts (R52), from the per-block output-line
/// counts. Block `i` sits after all earlier blocks' `1 (header) + output_lines`. `[2, 0, 3]` →
/// `[0, 3, 4]`.
pub fn block_boundary_rows(output_line_counts: &[usize]) -> Vec<usize> {
    let mut rows = Vec::with_capacity(output_line_counts.len());
    let mut row = 0;
    for &count in output_line_counts {
        rows.push(row);
        row += 1 + count;
    }
    rows
}

/// The viewport row to jump to (R52): forward → the FIRST boundary strictly `> current_top`;
/// backward → the LAST boundary strictly `< current_top`; `None` at the ends (NO wrap).
pub fn jump_target(boundaries: &[usize], current_top: usize, forward: bool) -> Option<usize> {
    if forward {
        boundaries.iter().copied().find(|&b| b > current_top)
    } else {
        boundaries.iter().copied().rev().find(|&b| b < current_top)
    }
}
```

### SHIM — `keymap.rs` + `app.rs`
- keymap: `cmd-up → "jump-block-prev"`, `cmd-down → "jump-block-next"`, `cmd-k → "clear-screen"`
  (`KeyBinding::chord(true, false, false, false, "up"/"down"/"k")`) + their `action_for` assertions.
  (cmd-chords only; plain ↑/↓ stay history/terminal — no conflict.)
- `dispatch_action` arms → `jump-block-prev` → `self.jump_focused_block(false)`, `-next` → `(true)`,
  `clear-screen` → `self.clear_focused()`.
- `jump_focused_block(&mut self, forward)` — EXTRACT the target in a block (so the `&workspace` borrow
  ends before the `&mut` scroll): build `counts = blocks().map(|b| b.output_styled().len())` →
  `block_boundary_rows(&counts)` → `top = viewport.visible(content, cap).0` → `jump_target(&b, top,
  forward)`; then `if let Some(t) = target { self.scroll_focused_to_row(t) }` (reuse #47).
- `clear_focused(&mut self)` — `state.session.write_bytes(b"\x0c")` (Ctrl-L, the shell redraws) +
  `state.viewport = Viewport::new()` (snap back to following).

### File manifest
- A `crates/marley_app/src/nav.rs` — `block_boundary_rows` + `jump_target` + tests.
- M `crates/marley_app/src/lib.rs` — `mod nav;`.
- M `crates/marley_app/src/keymap.rs` — 3 bindings + 3 `action_for` assertions.
- M `crates/marley_app/src/app.rs` — `jump_focused_block` + `clear_focused` + 3 dispatch arms +
  import `block_boundary_rows`/`jump_target`.
- M `docs/specs/SPEC-app-shell.spec.md` — R52 + Mutation-Targets. CHANGELOG; arch.

### Mutation Targets
- `block_boundary_rows` — the `row += 1 + count` accumulation (drop the `+1` header or the `+count`),
  the push-BEFORE-increment order. Killed by `[2,0,3]→[0,3,4]` (each boundary distinct) + `[1,1]→[0,2]`.
- `jump_target` — the forward `> current_top` (vs `>=`/`<`), the backward `< current_top`, the `.rev()`
  (without it, backward returns the FIRST `< top` = 0, not the nearest). Killed by fwd/back from a mid
  position + both ends → None.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `block_boundary_rows_accumulates` — `[2,0,3]`→`[0,3,4]`; `[]`→`[]`; `[1,1]`→`[0,2]`; `[0]`→`[0]` | unit |
| REQ-002 | `jump_target_forward_and_back` — boundaries `[0,3,4]`: fwd from 0→3, 3→4, 2→3; back from 4→3, 3→0 | unit |
| REQ-003 | `jump_target_no_wrap_at_ends` — fwd from 4→None (past last); back from 0→None (before first); fwd from 5→None | unit |
| REQ-004 | ⌘↑/⌘↓ jump + cmd-K clear | shim + masked visual — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs jump/clear dispatch + the keystroke translation + `write_bytes`/scroll (shim
exclude; needs a live window + PTY).

### Risks / decisions
- D-2.1 `jump_target`'s backward uses `.rev().find(< top)` = the LARGEST boundary below the top (the
  nearest previous block), NOT the first — the `.rev()` is load-bearing (a critic target). D-2.2 No
  wrap (None at ends) — the expected block-nav feel. D-2.3 `block_boundary_rows` takes line-counts
  (not `&BlockList`) — pure, and the shim maps `blocks → output_styled().len()` (matching the render's
  content-row walk: 1 header + N output lines). D-2.4 the jump extract-then-scroll avoids the
  `&workspace`/`&mut` borrow conflict (the #46 pattern). D-2.5 cmd-K = `\x0c` (real Ctrl-L) keeps the
  block history (a real terminal's clear); a history-drop is deferred.

## Phase 3 — Implement
- **Built (per manifest):** `nav.rs` (NEW, gpui-free) — `block_boundary_rows` (accumulate `1 + count`,
  push-before-increment) + `jump_target` (fwd `find(> top)`, back `rev().find(< top)`, no wrap);
  `lib.rs` `mod nav;`. `keymap.rs` — cmd-up/down/k bindings (`jump-block-prev`/`-next`/`clear-screen`)
  + their `action_for` assertions. `app.rs` — 3 dispatch arms; `jump_focused_block(forward)`
  (extract-then-scroll: build counts from `output_styled().len()` → `block_boundary_rows` →
  `jump_target(_, viewport top, _)` → `scroll_focused_to_row`); `clear_focused` (`write_bytes(b"\x0c")`
  + `viewport = Viewport::new()`). Import `block_boundary_rows`/`jump_target`. SPEC-app-shell R52 + row
  52 + Mutation-Targets; CHANGELOG (closes M1.G).
- **Deviations:** none. (The extract-then-scroll borrow pattern compiled first try.)
- **Verification at this phase:** `cargo check -p marley` 0 err; fmt; clippy `-D warnings` 0; docs 0;
  108 marley lib tests pass (incl. the 3 new keymap assertions). `block_boundary_rows` + `jump_target`
  are USED by `jump_focused_block` (live). Unit tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a 15-case verbatim probe + a real scoped cargo-mutants + a render/keymap cross-read).
  Verdict: **PASS — code correct on every input; one test-design finding.**
- **Mutants:** 16 on nav.rs (7 block_boundary_rows + 9 jump_target); all unkilled NOW (tests
  Phase-4-pending, expected — MSI 0% until they land). NO `.rev()` mutant is generated (cargo-mutants
  doesn't mutate method calls).
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED (test-design) | The backward `.rev()` is INVISIBLE to MSI — a plausible MSI-100 set using only a 1-boundary-below backward case kills all 6 comparison mutants yet passes on a `.rev()`-removed variant (which would flip ⌘↑ from "nearest previous block" to "always jump to the top"). | REAL (the test is the guard) | P4's `jump_target_forward_and_back` MUST include a backward jump with ≥2 boundaries below `top` — `jump_target(&[0,3,4], 4, false) == Some(3)` (3 with rev vs 0 without). My planned `back from 4→3` case already does this; keep it. Same family as #45's MSI-blind-arm-swap. |
  | F2 | INFO | 16/16 mutants unkilled + MSI 0% now — expected (tests Phase-4-pending), not a regression. | ACCEPTED | P4 lands the tests. |
- **Verified CORRECT:** boundary/render ALIGNMENT — the render walks `row += 1 (header) + output_styled().len()` per block, EXACTLY `block_boundary_rows`, and the shim builds counts from the SAME `output_styled().len()` (all in logical-output-line space) → no off-by-N; the `.rev()` = nearest-previous (`jump_target(&[0,3,4],4,false)==Some(3)`, not 0) + strict `>`/`<` (no jump-to-self: `(_,3,true)==Some(4)`, `(_,3,false)==Some(0)`) + no-wrap ends; the extract-then-scroll borrow (owned `Option<usize>` ends the `&workspace` borrow before `&mut scroll`); NO keymap conflict (cmd-chords only, plain ↑/↓/k fall through); cmd-K writes a FIXED `\x0c` (no injection); clean-room.
- **No code fix** — the pure surface is correct. F1 is honored by the planned test plan.

## Phase 4 — Validate
- **Tests added (nav.rs):** `block_boundary_rows_accumulates` ([2,0,3]→[0,3,4]; []; [1,1]→[0,2]; [0];
  [0,0,0]→[0,1,2]); `jump_target_forward_and_back` (fwd 0→3/3→4/2→3; back 4→3 [the `.rev()` ≥2-below
  guard, F1] + 3→0); `jump_target_no_wrap_at_ends` (fwd 4/5→None; back 0→None).
- **Runs (actual):** `cargo nextest run -p marley` → 114 passed (all 3 new PASS); `--workspace` → 538
  passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **16 caught /
  0 missed → MSI 100.0%** (block_boundary_rows + jump_target). Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (closes M1.G); `app_shell.md` — the block-nav bullet.
  SPEC-app-shell R52 at implement.
- **Knowledge captured:** no new failure (inspect's F1 was a test-design note the plan already
  honored). aar-submit `completed` (score 5). Win: the critic proved the boundary/render ALIGNMENT
  (nav, render, and content_rows all in logical-output-line space → no off-by-N under wrapping) AND
  that `.rev()` is MSI-invisible → the backward ≥2-below test (`jump_target(&[0,3,4],4,false)==Some(3)`)
  is the direction guard (the #45 arm-swap lesson recurring).
- **Ticket:** forge #48 → done; local doc → closed/; pipeline pair archived. **6 of 6 in M1.G.**
- **SPRINT:** forge sprint #7 "M1.G — Block Workflows & Selection" CLOSED. The terminal is now fully
  interactive: select + copy (the daily gap), the Warp block workflows (copy/re-run), find, and
  block navigation.
