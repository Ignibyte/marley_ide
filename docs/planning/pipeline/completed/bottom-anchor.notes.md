# terminal bottom-anchored — Notes

- **Forge ticket:** #49 `859c30cf-3c09-4d16-a3e2-fafd3d5ab4d5`
- **AAR:** `caab5473-939a-44e8-9f49-01181c607f24`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-049-bottom-anchor.md
- **Pipeline spec:** bottom-anchor.spec.md

## Phase 1 — Plan
- **Request:** forge #49 (M1.H "Real Terminal Feel" seq-1) — bottom-anchor the terminal, a bug chad
  found in live testing (the prompt sat at the top with empty space below).
- **Classification / tier:** work pipeline, `bug`, SHIM-ONLY (one gpui layout call). No pure surface →
  VISUAL-ACCEPTANCE (window capture) + static gates. marley_app only.
- **How it was found (§18 + the AX ask):** chad ran commands in the live app and reported "pwd shows
  but you have to scroll up ... not like a normal terminal." I captured Marley's window by CGWindowID
  (`screencapture -l<id>`) + read the pixels → saw the `pwd` block pinned to the TOP with empty space
  below + the prompt not at the bottom. The M1.G block features (✓ status, `↻ run`/`⧉ cmd`/`⧉ out`)
  render fine — it's purely the vertical anchor.
- **Discovery:** the pane is a `flex_col` (app.rs ~982) with the content rows as direct children,
  defaulting to flex-start (top). The viewport `visible()` (viewport.rs) is correct (following → the
  bottom `capacity` rows). So the fix is `justify_end` on the pane column, not a viewport change.
- **Also learned (launch/reaping):** direct `./marley & disown` launches get REAPED when the tool-call
  shell exits; only `open`-launched instances survive independently. Use `open` for a stable instance
  to capture. (This confused the debugging — the app kept "dying" on me, which was the launch method,
  not a crash: no crash report, no panic, clean exit.)
- **Decisions:** D1–D3 in the spec (shim-only; alt-screen unaffected; verify via capture).
- **Open for Design:** confirm `justify_end` + `overflow_hidden` interaction (when overflowing, the
  render emits exactly `capacity` visible rows = a full column, so justify has no effect + no clip of
  the latest); confirm the alt-screen branch is unaffected.
- **AAR id:** `caab5473-939a-44e8-9f49-01181c607f24`.

## Phase 2 — Design

### The change — `crates/marley_app/src/app.rs` (~982)
Add `.justify_end()` to the pane content `flex_col` (right after `.flex_col()`, before
`.overflow_hidden()`):
```rust
    .flex()
    .flex_col()
    .justify_end()      // bottom-anchor: prompt at the bottom, output above, empty space at the TOP
    .overflow_hidden()
```

### Interaction analysis (why this is correct + safe in every branch)
1. **Cooked view, short content** (content < capacity): render emits `[0, content)` rows → `justify_end`
   packs them at the BOTTOM → prompt at the bottom, empty space at the TOP. FIXES the bug.
2. **Cooked view, overflowing + following**: render emits `[content-capacity, content)` = up to
   `capacity` rows = a full column → `justify_end` has no visible effect; the prompt (last row) is at
   the bottom. Correct.
3. **overflow_hidden interaction** — this is a BONUS: if the rendered rows slightly overshoot the pane
   height (the header's `border_t` + the prompt's padding add a few px beyond `capacity * cell_h`),
   `justify_end` + `overflow_hidden` clips the OLDEST row at the TOP, keeping the prompt visible.
   Today (`justify_start`) the overshoot clips the BOTTOM — i.e. it can CUT OFF THE PROMPT. So the fix
   also removes a latent prompt-clipping risk.
4. **Alt-screen grid** (vim/top): the grid is `capacity` rows = a full column → `justify_end` no
   effect. No regression.

### File manifest
- M `crates/marley_app/src/app.rs` — the `.justify_end()` line on the pane column.
- M `docs/specs/SPEC-app-shell.spec.md` — an R39 note (the pane is bottom-anchored). CHANGELOG; arch.

### Regression Test Plan (VISUAL-ACCEPTANCE — no pure surface)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | Rebuild → `open` → `screencapture -l<windowid>` the empty prompt → the `❯` prompt renders at the BOTTOM of the pane (compare to the top-anchored before/after captures). | window capture (the AX-verify step) |
| REQ-002 | Alt-screen: the grid still fills the pane — build + the interaction analysis (grid = `capacity` rows = full column, `justify_end` no-op). | build + reasoning |
| REQ-003 | `scripts/gates.sh` static gates GREEN; cov/MSI unaffected (the diff is app.rs-only, coverage-excluded). | gate exit 0 + receipt |

Uncoverable by unit tests: the whole change is a gpui layout call in the `mutants::skip` +
coverage-excluded `render`. There is no pure logic — acceptance IS the window capture (exactly the
verification chad asked to make a workflow step) + the static gates. Full multi-command feel is
chad-verified.

### Risks / decisions
- D-2.1 SHIM-ONLY, no pure surface → visual-acceptance ticket; the capture is the test. D-2.2
  `justify_end` is strictly SAFER than `justify_start` under `overflow_hidden` (clips the oldest top
  row, never the prompt). D-2.3 The viewport `visible()` is unchanged (already correct). D-2.4 Verify
  via `open`-launched instance (survives) + CGWindowID capture; the earlier "keeps dying" was the
  `& disown` launch getting reaped, NOT a crash.

## Phase 3 — Implement
- **Built:** `app.rs` — `.justify_end()` on the pane content `flex_col` (between `.flex_col()` and
  `.overflow_hidden()`), with a comment on the bottom-anchor + the overflow-clip-safety. SPEC-app-shell
  R39 amended (bottom-anchored + the prompt-clip note); CHANGELOG `### Fixed`.
- **Deviations:** none — the single-line change as designed.
- **Verification at this phase:** `cargo check -p marley` 0 err; fmt. Visual verification (the prompt
  moves to the bottom) is Phase 4 via window capture.

## Phase 3.5 — Inspect
- **Critic:** 1 (read the render + the inverse hit-test + reasoned about gpui/taffy flexbox semantics).
  Verdict: the `justify_end` flip is correct + safe, but it shipped ONE HIGH coupled regression.
- **Findings:**
  | # | Sev | Finding | Verdict | Action |
  |---|---|---|---|---|
  | F1 | **HIGH** | `justify_end` bottom-anchors the RENDER, but `pane_grid_pos` (the click→cell map, the render's INVERSE, from #43) is STILL top-anchored: `row = start + local_y/cell_h`. After the anchor flip, when content < capacity the rows paint at the bottom with `capacity - visible` empty rows above — so a click lands `top_pad` rows too LOW → drag-select highlights nothing + cmd-C copies the wrong/empty text, in THIS ticket's exact target scenario. Non-crashing (range-checked) but silently broken. | REAL | **FIXED** — extracted a pure `row_at(local_row, start, end, capacity)` (viewport.rs) that maps the pointer bottom-anchored (`start + local_row.saturating_sub(capacity - (end-start))`); `pane_grid_pos` now reads `(start, end)` + calls it. Gives the ticket a real cov/MSI surface guarding the exact regression. |
- **Verified CORRECT (critic, concrete):** the prompt `input_row` is the LAST child → `justify_end`
  pins it to the bottom (nothing appended after it); overflow clips the OLDEST top row (taffy flex-end
  spills off the start) so the prompt is NEVER clipped (strictly safer than the old default, which
  clipped the prompt — the CHANGELOG note holds); the alt-screen grid emits the FULL `capacity` rows
  (no trailing trim) → `justify_end` is a no-op there (no regression); the pane div holds ONLY content
  rows + prompt (docks/palette/find-bar are separate `root` overlays) → nothing misplaced.
- **Lesson:** a rendering TRANSFORM and its INVERSE (hit-testing) are a COUPLED pair — changing one
  (the render anchor) silently breaks the other (the click map) unless updated in lockstep. The
  original spec/design missed `pane_grid_pos` entirely.

## Phase 4 — Validate
- **Test added (viewport.rs):** `row_at_bottom_anchored` — full pane (visible==capacity, no pad →
  start+local); short content (capacity 40, visible 6 → 34 empty rows: first-visible→start,
  last→start+5, empty-band-click→start); a scrolled/held window. Kills the top_pad + saturating_sub +
  `+start` mutants.
- **Runs (actual):** `cargo nextest run -p marley` → 115 passed; `--workspace` → 539 passed, 5 skipped.
- **VISUAL VERIFICATION (the AX-verify step chad requested) — DONE:** rebuilt → `open`-launched (PID
  survives) → `screencapture -l<windowid>` the empty prompt → **the `❯` prompt now renders at the
  BOTTOM of the pane with empty space ABOVE it** (before: prompt at the top, empty below). Captured +
  read the pixels directly. `marley_fixed.png` in the session scratchpad.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **4 caught /
  0 missed → MSI 100.0%** (the new `row_at`). Receipt written. No PTY hang.
- **Pre-existing:** none. **Chad-verify pending:** the full multi-command feel (the capture confirms
  the anchor + the row_at math guards the click-map; chad re-tests the live drag-select/scroll).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Fixed` (at implement); `app_shell.md` — the bottom-anchor + row_at
  bullet. SPEC-app-shell R39 amended.
- **Knowledge captured:** failure `BF-claude-render-anchor-changed-without-updating-inverse-hit-test`
  (f36870b3, validation) + prevention rule
  `PR-claude-render-transform-and-inverse-hit-test-change-in-lockstep-001` (bf1529a8, high) — a render
  transform and its inverse hit-test are a coupled pair; inspect must diff a render-geometry change
  against every consumer of the inverse mapping. aar-submit `completed` (score 5). **The headline:**
  window-capture visual verification WORKS (I captured Marley by CGWindowID + read the pixels → saw
  the top-anchor bug AND confirmed the fix), and the adversarial inspect caught a HIGH silent
  regression (the click-map desync) that no gate would catch — both the render and the hit-test are
  masked shim.
- **Ticket:** forge #49 → done; local doc → closed/; pipeline pair archived. Sprint #8 M1.H stays
  OPEN (more terminal-feel work possible — tab-completion, etc.).
