# Block actions — copy command / copy output — Notes

- **Forge ticket:** #45 `22c511c3-3508-404c-9cbb-71fd1a79e00b`
- **AAR:** `bcc0a203-3260-49d3-a586-d9199ff6e646`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-045-block-copy-actions.md
- **Pipeline spec:** block-copy-actions.spec.md

## Phase 1 — Plan
- **Request:** forge #45 (M1.G "Block Workflows & Selection" seq-3, auto-approved) — Warp's signature
  hover-to-copy on a block.
- **Classification / tier:** work pipeline, `feature`, a THIN pure surface (`BlockCopy` + `copy_text`
  in terminal_blocks — cov/MSI 100) + a SHIM (the header hover affordances). terminal_blocks + marley_app.
- **Discovery (§18):**
  - `Block` (block.rs) has the `command` field + `output_text()` (plain, `\n`-joined, trailing-trimmed
    — the exact text a copy-output should yield). So `copy_text` is `match what { Command =>
    self.command.clone(), Output => self.output_text() }`.
  - The block-header render (app.rs ~947, gained the #43 selection tint) is where the hover copy
    affordances attach — extends the existing `.hover` tint (#39).
  - #44 gave `ClipboardItem::new_string` + `cx.write_to_clipboard` — reused for the click.
  - The ui_components button (#17) isn't a clean fit for a small hover glyph → the affordance is a
    clickable `div` (shim, like the header itself).
  - SPEC-terminal-blocks next number = R29 (R28 = paste); SPEC-app-shell next = R49 (R48 = copy).
  - Deps #36/#43 (header) + #44 (clipboard) — done.
- **Decisions:** D1–D3 in the spec (copy_text on Block; Output reuses output_text; hover shim).
- **Open questions for Design:** the glyph(s) — one combined "copy" affordance with two targets vs two
  glyphs (lean two: a ⌘-glyph for command, a ▤-glyph for output, or labelled); whether `BlockCopy`
  derives Copy (yes — a trivial enum). The pure surface is unchanged either way.
- **AAR id:** `bcc0a203-3260-49d3-a586-d9199ff6e646`.

## Phase 2 — Design

### PURE — `crates/terminal_blocks/src/block.rs` (with the Block model)
```rust
/// What a block-copy action copies (R29).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockCopy {
    /// The command line.
    Command,
    /// The command's output.
    Output,
}

impl Block {
    /// The text a block-copy action copies (R29): the command line, or the plain output
    /// (`output_text` — so a block-action copy agrees with a drag-copy of the same output).
    pub fn copy_text(&self, what: BlockCopy) -> String {
        match what {
            BlockCopy::Command => self.command.clone(),
            BlockCopy::Output => self.output_text(),
        }
    }
}
```
`lib.rs` re-exports `BlockCopy`.

### SHIM — `crates/marley_app/src/app.rs` (the block-header render ~947)
- The header becomes a hover GROUP; at its right edge two clickable copy glyphs (a command glyph + an
  output glyph) are revealed on hover (opacity 0 → 1 via `group_hover`, extending #39's block hover).
- Each glyph's `on_mouse_down(Left, …)` captures `(pane_id, block_index)` and looks the block up AT
  CLICK time — `state.session.blocks()… index …` → `cx.write_to_clipboard(ClipboardItem::new_string(
  block.copy_text(BlockCopy::{Command,Output})))`. Deferring `copy_text` to the click (not per frame)
  keeps the render cheap AND makes `copy_text` live (used by the shim, not dead). Reuses #44's write.

### File manifest
- M `crates/terminal_blocks/src/block.rs` — `BlockCopy` enum + `Block::copy_text` + tests.
- M `crates/terminal_blocks/src/lib.rs` — export `BlockCopy`.
- M `crates/marley_app/src/app.rs` — the header hover copy glyphs + the deferred-lookup click. Import
  `BlockCopy`.
- M `docs/specs/SPEC-terminal-blocks.spec.md` — R29 + Mutation-Targets. `SPEC-app-shell.spec.md` — R49.
  CHANGELOG; arch.

### Mutation Targets
- `copy_text` — the `Command` arm (→ command), the `Output` arm (→ output_text), the whole-fn return.
  A mutant swapping/collapsing the arms → caught by REQ-003 (both arms asserted DISTINCTLY on a block
  that has both a command and different output); a `""`/`"xyzzy"` whole-fn → caught by REQ-001/002.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `copy_text_command` — `open_running("ls -la",…)`; `copy_text(Command)` == `"ls -la"` | unit |
| REQ-002 | `copy_text_output` — `set_output(plain_lines(&["file1","file2"]))`; `copy_text(Output)` == `"file1\nfile2"` (== `output_text()`) | unit |
| REQ-003 | `copy_text_arms_distinct` — a block with BOTH; `copy_text(Command)` == command AND `copy_text(Output)` == output AND the two differ (arms don't collapse) | unit |
| REQ-004 | hover → the copy affordances appear → click copies | shim + masked visual — chad-verified |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs hover reveal + click + clipboard write (shim exclude; needs a live pointer).

### Risks / decisions
- D-2.1 `copy_text` on `Block` (terminal_blocks) — a pure method beside `output_text` it reuses, so a
  block-action copy and a drag-copy (#44) of the same output produce identical text. D-2.2 The click
  looks the block up by `(pane_id, index)` at click-time (not a per-frame `copy_text` clone) — cheap +
  keeps `copy_text` live. D-2.3 The affordance is a clickable glyph `div` (the #17 button isn't a fit
  for a small hover glyph) — all masked shim. D-2.4 A `Both` variant is deferred — two clear actions.

## Phase 3 — Implement
- **Built (per manifest):** `block.rs` — `BlockCopy { Command, Output }` enum + `Block::copy_text(what)`
  (Command → command.clone(), Output → output_text()); `lib.rs` exports `BlockCopy`. `app.rs` — the
  block loop gained `.enumerate()` (the block index); the header render gained two hover-brightened
  copy glyphs (`⧉ cmd` / `⧉ out`) pushed right by a `flex_1` spacer, each `on_mouse_down` → deferred
  `(pane_id, block_index)` lookup → `cx.write_to_clipboard(copy_text(what))` + `cx.stop_propagation()`
  (so the copy click doesn't also start a pane selection). Import `BlockCopy`. SPEC-terminal-blocks
  R29 + SPEC-app-shell R49 + row 49; CHANGELOG.
- **Deviations:** (1) the closure form `copy_action(label, what)` didn't compile (it captured `cx` by
  move → couldn't be called twice; `cx` needed after) → inlined the two glyphs so each borrows
  `cx.listener` directly. (2) The affordances are always-visible-but-muted, brightening on their OWN
  hover (not group-hover-revealed) — avoids gpui group-scoping risk; a true hover-reveal is a later
  refinement. `cx.stop_propagation()` added so the copy click doesn't leave a stray empty selection.
- **Verification at this phase:** `cargo check -p marley` 0 err; fmt; clippy `-D warnings` 0 (marley +
  marley_terminal); docs 0; 91 terminal_blocks tests pass. `copy_text` is USED by the shim glyphs
  (live). Unit tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (a real scoped cargo-mutants on block.rs + a full symbol/wiring trace). Verdict:
  **PASS — code correct, plan reaches MSI 100, no blocking issues.**
- **Mutants:** 26 on block.rs → 23 caught (all pre-existing — no regression) / 1 unviable / **2 missed
  = exactly the two `copy_text` whole-fn mutants** (`String::new()`, `"xyzzy".into()`), Phase-4-pending.
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | MED (advisory) | cargo-mutants emits ONLY whole-fn mutants for `copy_text` — NO arm-swap or per-arm-body mutant. So MSI 100 does NOT prove the Command/Output arms aren't swapped; the hand-written `copy_text_arms_distinct` (asserting command != output on both arms) is the SOLE swap guard. | REAL (test is load-bearing) | P4 MUST write `copy_text_arms_distinct` with a block whose command ≠ output; not collapse to a one-arm smoke test. (Same family as the MSI-blind structural-test lessons.) The critic verified no planned fixture has command==output. |
  | F2 | LOW | The copy-glyph click doesn't focus the pane (`stop_propagation` suppresses the pane's focus-on-mouse-down too). | ACCEPTED | Intended Warp-style non-focus affordance; no change. |
- **Verified CORRECT:** arm mapping (Command→command, Output→output_text — not swapped); `copy_text(Output)`
  is byte-identical to `output_text()` (no re-implementation); render `.enumerate()` and click
  `.nth(block_index)` traverse the SAME append-only `BlockList::iter()` (grep-confirmed NO remove/pop/
  clear/truncate/reorder API → indices never shift → the clicked glyph copies the right block);
  `stop_propagation` runs on both the found + not-found paths; a stale index → `.nth()` None → no
  write/panic; `copy_text` has no unwrap/expect; clean-room (trivial 2-arm match).
- **No code fix** — the pure surface is correct. F1 is the Phase-4 `arms_distinct` test.

## Phase 4 — Validate
- **Tests added (block.rs):** `copy_text_command` (Command → "ls -la"); `copy_text_output` (Output →
  "file1\nfile2" AND == output_text()); `copy_text_arms_distinct` (F1 — a block with command "echo hi"
  ≠ output "hi"; asserts BOTH arms distinctly → the swap guard MSI can't provide).
- **Runs (actual):** `cargo nextest run -p marley_terminal` → 99 passed (all 3 new PASS);
  `--workspace` → 525 passed, 5 skipped.
- **Env hang (handled, #27 pattern):** the FIRST gate run stalled at gate:3 — the real-PTY integration
  test `workspace_two_real_sessions_are_independent` hung >240s under load (unrelated to #45's diff:
  block.rs + the app shim, nothing PTY). Killed the runners + swept temp dirs; the 2nd run passed
  clean.
- **Gate:** `scripts/gates.sh --diff` (2nd run) → **GREEN [diff] 15/15**, coverage 100%, mutation
  **2 caught / 0 missed → MSI 100.0%** (the two `copy_text` whole-fn mutants, killed). Receipt written.
- **Pre-existing:** none (the hang is environmental, not a failure).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `terminal_blocks.md` — the copy_text bullet.
  SPEC-terminal-blocks R29 + SPEC-app-shell R49 at implement.
- **Knowledge captured:** no new failure (inspect found no bug — the pure surface was correct). The F1
  advisory (MSI blind to an arm-swap → the arms_distinct command≠output test is the guard) is the same
  family as the already-captured MSI-blind lessons; noted in the ledger + the arch doc. aar-submit
  `completed` (score 5). Win: the critic proved cargo-mutants emits ONLY whole-fn mutants for a 2-arm
  match (no arm-swap mutant), so a one-arm smoke test would reach MSI 100 while a swap survives — the
  distinctness test is the real guardrail.
- **Ticket:** forge #45 → done; local doc → closed/; pipeline pair archived. 3 of 6 in M1.G.
