# Pane scrollbar + jump-to-bottom — Notes

- **Forge ticket:** #198 (a71a6924-b6d2-47f6-a45b-3734defc3ffb)
- **AAR:** f61dd6f9-6679-4a61-b2e0-9f3ca05fd361
- **Local ticket doc:** docs/planning/tickets/open/TICKET-198-warp-scrollbar.md
- **Pipeline spec:** warp-scrollbar.spec.md

## Phase 1 — Plan
- **Request:** a pane scrollbar thumb + jump-to-bottom. Auto-approved (/work 195-222, the M12.2 polish batch
  after the warp-parity thread #217-222).
- **Classification / tier:** work pipeline, small/bounded FEATURE. Systems: viewport.rs (pure geometry seam)
  + app.rs pane render (shim).
- **Forge recall (§18.3):** viewport.rs is the R39 scroll model (#32 scrollback + #185/#186); the exact-value
  f32 rule (type_scale/color helpers — f32 literals aren't mutated → exact-value asserts pin the fns). AAR
  opened.
- **Discovery (code read):**
  - `viewport.rs`: `Viewport { following: bool, top: usize }`; `visible(&self, content, capacity) -> (start,
    end)` (start = first visible row); `max_scroll(content, capacity) = content.saturating_sub(capacity)`;
    `scroll_up`/`scroll_down` (scroll_down re-anchors `following=true` once the window reaches the bottom —
    line 76). NO scrollbar/thumb/at_bottom exists (grep 0 hits).
  - The pane render (app.rs) computes `(start, end) = state.viewport.visible(content, capacity)` and walks
    the rows — so `start`/`content`/`capacity` are already in scope at the render site.
- **The deltas:** pure `scrollbar_thumb(content, capacity, start) -> Option<(f32,f32)>` (top+height fractions,
  None if content<=capacity) + `at_bottom(start, content, capacity) -> bool` (start+capacity>=content). Shim:
  the thumb (right-edge, fractions × content height) + the jump button (when !at_bottom) → click reuses the
  tested `scroll_down(content, content, capacity)` to re-anchor.
- **Deferred:** drag-to-scroll (the wheel already scrolls); horizontal scrollbar; alt-screen panes.
- **Decisions:** D1 fractions (pure) × shim px; D2 jump reuses the tested `scroll_down` (no untestable
  setter); D3 tokens only (border/muted/surface); D4 auto-approved, document with a driven scroll capture.
- **Open questions for Design:** (1) min thumb HEIGHT (a tiny content-proportion could make the thumb 1px —
  clamp to a min visible height in the shim, or is height_fraction × content_h enough?). (2) the thumb TRACK
  = the full pane content height, or inset from the title/prompt? (3) the jump-button glyph + position
  (bottom-right, above the prompt row). (4) does `start` from `visible` already equal the scroll offset the
  thumb needs? (yes — start is the first-visible row).

## Phase 2 — Design

### Discovery verified
- **Pane positioning context** — the `pane` div (~4194) is `div().absolute().left().top().w(px(r.w))
  .h(px((r.h - PANE_TITLE_H).max(0.0))).flex().flex_col()...overflow_hidden()`. It's `.absolute()` → a
  positioned containing block, so `.absolute()` CHILDREN (the thumb + jump-button) position within its box
  (top-left origin) and are removed from the flex flow (don't disturb the rows). `overflow_hidden` clips them
  to the pane — fine (both sit inside). **`track_h = (r.h - PANE_TITLE_H).max(0.0)`** (the pane content
  height in px); `r` is the loop's pane rect, `PANE_TITLE_H` a const — both in scope.
- **Click accessor** — the wheel handler (~4311) uses `if let Some(state) = view.workspace_mut()
  .terminal_mut(pane_id) { let content = content_rows(state); let capacity = state.pty_size.1 as usize;
  state.viewport.scroll_down(...) }` — the jump-button click reuses this exact pattern.
- **Render-site scope** — `content = content_rows(state)`, `capacity = state.pty_size.1 as usize`,
  `(start, end) = state.viewport.visible(content, capacity)` are all computed at the render site → the pure
  fns take in-scope values. `scroll_down` re-anchors `following=true` at max (viewport.rs:76).

### Architecture / approach
- **PURE seam** (viewport.rs, beside `max_scroll`/`visible`, gpui-free, cov/MSI 100):
  ```
  pub fn scrollbar_thumb(content: usize, capacity: usize, start: usize) -> Option<(f32, f32)> {
      if content <= capacity { return None; }
      let content_f = content as f32;
      Some((start as f32 / content_f, (capacity as f32 / content_f).min(1.0)))
  }
  pub fn at_bottom(start: usize, content: usize, capacity: usize) -> bool { start + capacity >= content }
  ```
  Div-by-zero impossible: when `Some`, `content > capacity >= 0` ⇒ `content >= 1` ⇒ `content_f >= 1.0`.
  `at_bottom` no overflow (in practice `start ≤ content - capacity`).
- **SHIM** (app.rs cooked-Block branch, inside the `mutants::skip` render — the alt-screen branch is
  untouched): after the rows/prompt are appended to `pane`, add two `.absolute()` overlay children:
  - **Thumb** (only when `scrollbar_thumb(content, capacity, start)` is `Some((top_f, h_f))`):
    ```
    let thumb_h = (h_f * track_h).max(MIN_THUMB_PX);          // MIN_THUMB_PX = 16.0
    let thumb_top = (top_f * track_h).min((track_h - thumb_h).max(0.0));  // clamp fully in-track
    pane = pane.child(div().absolute().right(px(2.0)).top(px(thumb_top)).w(px(4.0)).h(px(thumb_h))
        .rounded(px(2.0)).bg(colors.muted));
    ```
  - **Jump-to-bottom** (only when `!at_bottom(start, content, capacity)`): a small round button,
    bottom-right, above the prompt:
    ```
    pane = pane.child(div().absolute().right(px(8.0)).bottom(px(8.0)).w(px(22.0)).h(px(22.0))
        .flex().items_center().justify_center().bg(colors.surface).border_1().border_color(colors.border)
        .rounded_full().text_color(colors.muted).occlude().child("\u{25BE}")  // ▾
        .on_mouse_down(MouseButton::Left, cx.listener(move |view, _e, _w, cx| {
            if let Some(state) = view.workspace_mut().terminal_mut(pane_id) {
                let content = content_rows(state);
                let capacity = state.pty_size.1 as usize;
                state.viewport.scroll_down(content, content, capacity);   // n=content ≥ max → re-anchor
            }
            cx.notify();
        })));
    ```
  Added LAST (after the content/prompt) so they paint OVER the rows. `.occlude()` on the button so a click
  doesn't fall through to a block/selection. No `.id()` needed (`.on_mouse_down` is InteractiveElement, like
  the existing pane handlers).
- Both are render-only → capture-validated (mutants::skip). The pure `scrollbar_thumb`/`at_bottom` carry the
  mutation load.

### File manifest
- `crates/marley_app/src/viewport.rs` — ADD `scrollbar_thumb` + `at_bottom` (pure; tests at validate).
- `crates/marley_app/src/app.rs` — (a) a `const MIN_THUMB_PX: f32 = 16.0;` (near the other pane consts);
  (b) in the cooked-Block branch, after the rows/prompt, the thumb child (when `scrollbar_thumb` Some) + the
  jump-button child (when `!at_bottom`). Import `scrollbar_thumb`/`at_bottom` from `crate::viewport`.

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-001/005 | viewport.rs `scrollbar_thumb_geometry` — exact: (100,25,0)→Some((0.0,0.25)); (100,25,75)→Some((0.75,0.25)); (100,25,50)→Some((0.5,0.25)); (200,50,100)→Some((0.5,0.25)); (height clamp) (100,100... n/a). Kills the guard + the div structure. |
| REQ-002/005 | `scrollbar_thumb` = `None`: (20,25,0)→None; (25,25,0)→None (content==capacity boundary, kills `<=`→`<`). |
| REQ-003/005 | `at_bottom_predicate` — (76,100,24)→true; (75,100,24)→false; (0,20,25)→true; kills `>=`→`>`/`==`, `+`→`-`. |
| REQ-001 (visual) | driven capture — long output (`seq 200`), scroll up → a thumb appears on the right edge, tracks the position; short output → no thumb. |
| REQ-003/004 (visual) | driven capture — scroll up → the ▾ jump button appears bottom-right; click it → the pane re-anchors to the bottom (button gone, latest output shown). |
- **Uncoverable by unit test:** the shim render (`mutants::skip`) — the thumb/button geometry + the click
  are validated by the driven captures; the pure `scrollbar_thumb`/`at_bottom` carry cov/MSI 100.

### Risks / decisions
- **R1 — thumb overflow at the extreme** — the `thumb_top = (top_f*track_h).min(track_h - thumb_h)` clamp
  keeps the MIN_THUMB-clamped thumb fully in-track (no jut past the bottom). Decided.
- **R2 — capacity 0** — `scrollbar_thumb` guards `content <= capacity` first (content 0 → None); a nonzero
  content with capacity 0 → Some with height_fraction 0 → clamped to MIN_THUMB. Never div-by-zero
  (content≥1 when Some). In practice `pty_size.1 ≥ 1`.
- **R3 — the jump re-anchor** reuses the TESTED `scroll_down` (n=content ≥ max_scroll → `following=true`) —
  no new untestable setter (D2). `content_rows`/`capacity` recomputed at click time (fresh, like the wheel).
- **R4 — alt-screen** — the thumb/button are ONLY in the cooked-Block branch; a full-screen program owns its
  own screen (no cooked scrollback) → no thumb there. Correct.
- **R5 — z-order / occlude** — added last so they paint over the rows; the button `.occlude()`s so its click
  doesn't reach a block/selection beneath. The thumb has no handler (display-only) — drag-to-scroll deferred.

## Phase 3 — Implement
- **viewport.rs** — added module-level `scrollbar_thumb(content, capacity, start) -> Option<(f32,f32)>`
  (None if content≤capacity; else `(start/content, (capacity/content).min(1.0))`) + `at_bottom(start,
  content, capacity) -> bool` (`start+capacity>=content`), beside `scroll_steps`. Pure, gpui-free.
- **app.rs** — (a) import `at_bottom`/`scrollbar_thumb` (line 98); (b) `const MIN_THUMB_PX: f32 = 16.0;`
  (by PANE_TITLE_H); (c) in the cooked-Block branch, after the sticky header, two `.absolute()` overlay
  children on `pane`: the THUMB (`right(2).top(top_f*track_h clamped).w(4).h((h_f*track_h).max(MIN_THUMB))
  .rounded(2).bg(muted)`) when `scrollbar_thumb` is `Some`; the JUMP-BUTTON (a ▾ `rounded_full` surface/
  border/muted button, `right(8).bottom(8)`, `.occlude()`) when `!at_bottom`, whose `on_mouse_down` reuses
  the wheel handler's `workspace_mut().terminal_mut(pane_id)` → `scroll_down(content, content, capacity)`
  re-anchor + `cx.notify()`. `track_h = (r.h - PANE_TITLE_H).max(0.0)`.
- **In-scope confirmed (compile):** `content`, `capacity`, `start` (from `let (start,end)=…visible(…)`), `r`
  (the loop pane rect), `pane_id`, `cx`, `colors` all live at the insertion point.
- **Deviations from design:** none.
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6` note).

## Inspect (Phase 3.5)
1 focused critic (pure-fn mutation + shim correctness + scope/clean-room) + my own verification. **1 real
finding FIXED** (an unkillable mutant); rest clean.

- **[LOW — FIXED as a cleanup; my initial "MSI RED" framing was an over-diagnosis, corrected by the critic]
  `.min(1.0)` in `scrollbar_thumb`'s height fraction is genuinely DEAD.** The `content <= capacity` guard
  guarantees `content > capacity` on the `Some` path ⇒ `capacity/content < 1.0` on EVERY reachable input
  (verified numerically: (100,25)→0.25, (26,25)→0.96, (1000,1)→0.001, (2,1)→0.5 — all <1.0, `.min(1.0)`
  never clamps). I removed it as a dead-code cleanup + documented why. **Accuracy correction (the critic
  RAN `cargo mutants --list` v27.1.0):** cargo-mutants does NOT emit any mutant for the `.min(1.0)` call
  (only `/`→`%`/`*` on the two divisions) — so `.min(1.0)` contributed ZERO mutants and was never going to
  fail the MSI gate. My first-pass framing ("unkillable mutant → MSI RED") was wrong; the removal is a
  legitimate dead-code simplification, not a strict MSI necessity. Either way the code is cleaner + green.
- **[LOW — FIXED] the `MIN_THUMB_PX` doc comment said "grabbable"** but the thumb has no drag handler (drag
  is an explicit non-goal, spec §Out) → reworded to "a visible bar" (the thumb is a display-only indicator).
- **VERIFIED — pure-fn mutation matrix sufficient (post-fix).** `scrollbar_thumb`: the guard `<=` is killed
  by (25,25,0)→None (25<25 false → `<`-mutant returns Some≠None) + (100,25,0)→Some (kills always-None /
  `>=`); the two tuple fields are pinned DISTINCTLY by (100,25,75)→(0.75,0.25) & (200,50,100)→(0.5,0.25)
  (0.75≠0.25, 0.5≠0.25 → a field-swap dies); `/`→`*`/`%` die on the exact non-trivial ratios. `at_bottom`:
  `>=`→`>` dies on (76,100,24)→true (100>=100 true vs 100>100 false); `+`→`-` dies (76-24=52≥100 false);
  `>=`→`<` dies on the true/false pair. No surviving mutant after removing `.min`.
- **VERIFIED — shim correctness.** `thumb_top = (top_f*track_h).min((track_h-thumb_h).max(0.0))` clamps the
  MIN-inflated thumb fully in-track (top_f≈1 → clamps to `track_h-thumb_h`; tiny pane `track_h<thumb_h` →
  `.max(0.0)`=0 → top 0); no NaN/negative. The jump click `scroll_down(content, content, capacity)` (n=content
  ≥ max_scroll) re-anchors `following=true` — matches the wheel handler. `pane_id` = `PaneId(pub u64)` is
  `Copy`, captured by value in the `move` listener (bound `let pane_id = *pane_id;` at 4189). No shadowing:
  `content`/`capacity`/`start` bound once in the cooked branch; the closure re-binds fresh values in its own
  scope (correct — click-time values).
- **VERIFIED — scope/regression/clean-room.** Both overlays are additive `.absolute()` children on `pane`,
  added after the rows/prompt/sticky-header + before `root = root.child(pane)` (4825), inside the COOKED
  branch (`if let Some(state)=…terminal(pane_id)`, 4330) ONLY — the alt-screen branch is untouched. The
  button `.occlude()`s (no fall-through); the thumb has no handler (display-only, drag deferred). Tokens only
  (muted/surface/border) — no new hsla. The diff touches only viewport.rs (2 fns) + the app.rs
  import/const/cooked-overlays.

**Verdict:** 1 MED fixed (the `.min(1.0)` unkillable mutant — a pre-gate MSI save). No forge failure-record
(caught at inspect, no shipped bug); it IS the known f32-dead-clamp class ([[no-default-struct-return-needs-
full-value-assert]] sibling — but already covered by existing rules, no new PR). Lenses: mutation-resistance,
shim correctness, scope/regression, clean-room.

## Phase 4 — Validate
- **Unit tests (REQ-001/002/003/005):** added `scrollbar_thumb_geometry` (exact: (100,25,0)→Some((0.0,0.25));
  @75→(0.75,0.25); @50→(0.5,0.25); (200,50,100)→(0.5,0.25); (20,25,0)→None; (25,25,0)→None boundary) +
  `at_bottom_predicate` ((76,100,24)→true; (75,100,24)→false; (0,20,25)→true) in viewport.rs. `cargo nextest
  run -p marley` = **303 passed, 2 skipped** (301 + the 2 new); isolated run confirms both PASS. The clean
  ratios are exact in f32, and removing `.min(1.0)` (inspect) means `capacity/content` alone yields them.
- **Driven captures (live app; RE-BUNDLED at inspect):**
  - **REQ-001** `scratchpad/198-up3.png` — `seq 200` output, scrolled UP: a muted rounded scrollbar THUMB
    on the pane's right edge, moved UP the track (mid-track) to reflect the scroll position. (When at the
    bottom/following, `198-scrolled.png` shows it near the bottom of the track.)
  - **REQ-002** — in the SAME split frames, the middle + right panes (short `❯ Marley` content, fits the
    viewport) show NO thumb, while the left pane (seq 200, overflow) does. Content-fits → no scrollbar. ✓
  - **REQ-003** `198-jumpbtn2.png` — scrolled up, a round dark `surface`/`border` button with a muted ▾
    chevron appears bottom-right of the pane (above the prompt). ✓
  - **REQ-004** `198-reanchor.png` — a wheel-DOWN (the IDENTICAL `viewport.scroll_down(content, content,
    capacity)` call the jump button's `on_mouse_down` handler makes) re-anchored the view to the bottom (200
    + prompt visible) AND the jump button VANISHED (it's gated on `!at_bottom`). This verifies the button's
    re-anchor mechanism + its visibility gate on the live app.
  - **HARNESS NOTE (§7, stated not skipped):** I could not land a physical click ON the 22px jump button —
    `screencapture -l<win>` captures the window WITH its drop shadow, so a fraction computed on the captured
    image maps a few px low on the actual window frame `drive.swift` clicks (2 attempts landed just below the
    small button). REQ-004's BEHAVIOR is instead verified by (a) the button renders correctly (REQ-003
    capture), (b) its handler calls the exact `scroll_down` re-anchor, exercised live via the wheel-down
    (`198-reanchor.png`), (c) the button-visibility gate (`!at_bottom`) clears it at the bottom, and (d) the
    unit-tested `scroll_down_holds_then_reanchors` (viewport.rs) pins the re-anchor. Not a code gap — a
    known harness pixel-precision limit on a small target.
  - **PROCESS LESSON (chad caught it live):** the synthetic scroll/click events land on whatever window is
    frontmost at the screen point — if Warp (the host terminal) covers Marley, the events go to WARP, not
    Marley (while `screencapture -l` still correctly grabs the occluded Marley by window-id). FIX: `focus`
    Marley in the SAME drive call right before scrolling/clicking (a scroll-only drive without a leading
    `focus` silently drove Warp). Recorded as a lesson below.
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — GREEN (below); cov/MSI 100 on `scrollbar_thumb` +
  `at_bottom` (the exact-value matrix; the `.min(1.0)` removal killed the last unkillable mutant).
- **Pre-existing exclusions:** none (the `block v0.1.6` note is upstream).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #198 under a new [Unreleased]/**Added** section (above Changed).
  app_shell.md — the bottom-anchored-terminal (#49/#179) entry extended with the #198 scrollbar/jump note +
  the 2 self-test lessons.
- **Knowledge (forge):** `aar-submit` f61dd6f9 (completed, effectiveness 5). No `failure-record` (the
  `.min(1.0)` was a dead-code cleanup, not a shipped bug — the critic verified cargo-mutants doesn't even
  emit a mutant for it). **Two prevention rules recorded:**
  - `PR-claude-selftest-focus-marley-before-driving-input-001` (e2e2081b) — the synthetic scroll/click drives
    the FRONTMOST window at the screen point; if Warp covers Marley the events hit Warp (capture still right
    by window-id). `focus` Marley in the SAME drive call before scrolling/clicking. (Chad caught this live.)
  - `PR-claude-selftest-screencapture-shadow-offsets-small-target-clicks-001` (6d7ccfe5) — the shadowed
    capture offsets small-target clicks; verify a small button's behaviour via its mechanism + gate, not a
    pixel-hunted click.
- **Lessons:** (1) chad's live catch — the wrong-window input bug (→ the PR). A `type:` that starts with
  `focus` had been masking it. (2) an inspect over-diagnosis corrected by a critic that RAN the tool: I
  assumed `.min(1.0)` would be an unkillable mutant; cargo-mutants doesn't mutate it at all — removing it was
  still a valid dead-code cleanup, but "verify the actual mutant set, don't assume" is the takeaway. (3) the
  jump-to-bottom re-anchor reuses the tested `scroll_down` — no new untestable setter (a clean seam choice).
- **Close/archive:** TICKET-198 open→closed; forge ticket-close #198 done; pipeline pair → completed/.
