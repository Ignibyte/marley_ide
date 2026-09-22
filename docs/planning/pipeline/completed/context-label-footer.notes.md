---
pipeline_id: 2fbefb5b-91d4-460c-a9cc-89047bb07cfa
ticket: forge#192 (fc082ebd-f80d-4a47-9178-a14993d6bfb9)
aar_id: 52e851c9-c9ac-4da7-b1fe-5829c8d2a9a0
---

# Notes — M12.1 #192 cwd/branch context → footer

## Phase 1 — Plan
Current top-right render (app.rs ~5360-5370): `div().absolute().top(8).right(130).text_size(12).text_color(muted)
.child(self.titlebar_context_label())`. The footer (app.rs ~5237-5270): a `flex_row items_center gap_3 px_3`
strip pinned `bottom(0)`, drawing the `cockpit_status(sprint, agents, focused)` segments left-to-right (index 1 =
agents = clickable → Agents cockpit tab). `titlebar_context_label` (app.rs:1699, mutants::skip shim) reads
$HOME + `.git/HEAD` then calls the PURE `titlebar::titlebar_label` (unit+mutation tested — UNCHANGED).

**Approach (shim placement move).** Delete the top-right block. In the footer, after the segment loop:
`footer = footer.child(div().flex_1()).child(div().child(self.titlebar_context_label()))` — the `flex_1` spacer
consumes the middle, pushing the context to the RIGHT. Muted 12px is inherited from the footer's `text_color` /
`text_size`. No pure-logic change; proven by driven capture.

**Risks.** The footer agent segment (index 1) is clickable (`.occlude()` + on_mouse_down); the new context child
is plain text after a spacer — no overlap. A very narrow window: the `flex_1` spacer shrinks to 0 and the context
sits right after the segments (may wrap/clip) — acceptable, same as any footer overflow. `flex_1` is already used
in the shell (e.g. the left dock `files` div), so it's a known-good gpui builder.

## Phase 2 — Design
**Architecture.** Pure render-shim placement move inside `RootView::render` (app.rs). No new types, no IO change,
no crate boundary crossed. `titlebar_context_label` (shim) + `titlebar::titlebar_label` (pure, tested) are
untouched — only WHERE the shim's output is drawn changes. Fits the shell's footer (#94) / titlebar (#142) split.

**File manifest.**
- `crates/marley_app/src/app.rs` — (a) delete the top-right context block (~5360-5370); (b) in the footer render
  (~5269, after the `cockpit_status` loop, before `root = root.child(footer)`), append
  `footer = footer.child(div().flex_1()).child(div().child(self.titlebar_context_label()));`.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| driven capture (footer) | REQ-001 — cwd/branch renders in the footer, right-aligned |
| driven capture (top-right) | REQ-002 — the top-right titlebar no longer shows cwd/branch |

Shim-only placement — NO new unit test (the pure `titlebar::titlebar_label` that produces the string is unchanged
and already unit+mutation tested; `cockpit_status` is untouched, its ordering test still holds). Per §7 a
render-shim change is proven by the driven capture + the gate's static/coverage/mutation set staying green (no
pure logic added or removed). No uncoverable gaps beyond the GUI shim itself (covered by capture).

**Risks/decisions.** D1 (right-aligned label, not a cockpit_status segment) + D2 (flex_1 spacer) in the spec. No
behavior/logic change; purely presentational. Reversible.

## Phase 3 — Implement
Built to the manifest; `cargo check -p marley` clean.
- `app.rs` — footer: after the `cockpit_status` loop, `footer = footer.child(div().flex_1()).child(div().child(
  self.titlebar_context_label()))` (right-aligned via the spacer). Deleted the top-right `.top(8).right(130)`
  block (left a `#142/#192` breadcrumb comment). `titlebar_context_label` now has exactly ONE caller (the
  footer, app.rs:5275) — confirmed by grep.
**Skip-detach check:** N/A — no fn inserted; only a render statement moved and a block deleted. **Deviation:** none.

## Inspect (Phase 3.5)
One critic over the diff (1 file, +8/−11). **Verdict: CLEAN.**
- **Clean removal** — `grep titlebar_context_label` = exactly 2 hits (the shim def + the new footer caller); no
  orphaned `right(130)`/`top(8)` constants, no now-unused import (`titlebar_label` still used by the shim), no
  dangling var. `cargo check -p marley` green.
- **Right-align correct** — `.child(div().flex_1()).child(label)` grows the spacer to push the label to the
  right (inset by the footer `px_3`); the established idiom. Putting the label INSIDE the flex_1 would left-align
  — the two-child split is right.
- **No hitbox collision** — spacer + label are plain divs (no `.occlude()`/listener) → click-through; they're
  disjoint flex siblings from the clickable agent segment (index 1). Its `.occlude()` + on_mouse_down unaffected.
- **Narrow window** — flex_1 → 0, label tightens after the segments; fixed 22px height, no wrap/break. More
  robust than the old `absolute().right(130)` (which had to be hand-positioned to avoid the search/tabs).
- **Style parity** — label inherits the footer's `muted` 12px (same as the old explicit style). No pure logic
  touched (`titlebar_label` / `cockpit_status` unchanged; their tests still hold).
- **[LOW → FIXED]** redundant `div()` wrapper around the label — dropped (`String` is `IntoElement`, so
  `.child(self.titlebar_context_label())` directly).

Verdict: **Phase 3.5 PASS** — placement-only, clean; one redundant wrapper removed.

## Phase 4 — Validate
**Tests.** No new unit test (placement-only; the pure `titlebar::titlebar_label` that builds the string is
unchanged). `cargo nextest run -p marley titlebar` → **5 passed** (`titlebar_label_cases`, `abbreviate_path_cases`,
`branch_from_git_head_cases`, …) — the string logic is intact.
**Driven capture — BLOCKED by display sleep (environment, not code).** After ~12 relaunch cycles this long
session, the physical display went to sleep; `screencapture` (both `-l<win>` and full-screen) returns pure black
(avg brightness ≈1/765) and it would NOT wake via `caffeinate -u`, a synthetic mouse-move, a shift-key event, or
a scroll event (synthetic CGEvents don't wake a slept display). The marley process is alive and the code compiles
+ renders (the earlier #191 driven capture this session rendered fine; nothing in this 8-line placement diff
touches the render pipeline's health). Stated explicitly per §7 rather than silently skipped. **Compensating
evidence:** (a) the inspect critic's thorough static verification — clean removal (grep: exactly 2
`titlebar_context_label` refs, the shim + the footer caller; no orphaned `right(130)`/import/var), correct
right-align via the proven `flex_1` idiom, no hitbox collision with the clickable agent segment, style parity
(inherits footer muted-12px), no pure logic touched; (b) **gate:15 visual/AX PASS** — the headless
`marley_visual_harness` renders offscreen (no display needed) and diffs committed baselines, so the render path is
exercised; (c) build + titlebar tests green. **Follow-up:** a one-glance visual confirm when the display is awake
(flagged to chad) — the code is verified, only the live pixel-capture is deferred.
**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 (cov 100%, mutation MSI 100%, visual/AX). No
pre-existing failures in scope. **Phase 4 PASS** (driven capture deferred — display asleep).

## Phase 5 — Complete
**Docs.** CHANGELOG `### Changed` entry (TICKET-192). `app_shell.md` — the #142 cwd/branch note records the #192
move to the footer.
**Knowledge.** `aar-submit` (aar `52e851c9…`, completed, effectiveness 4 — clean change; the driven capture was
blocked by display sleep). `prevention-rule-record PR-claude-display-sleep-blocks-capture-001` (the display-sleep
→ black-capture harness trap + how to confirm/work around it). No failure-record (no runtime defect).
**Lessons.** A placement move is cheap but still deserves the critic (it caught the redundant `div()` wrapper).
Front-load UI captures early in a session — the display can sleep after hours, and synthetic input won't wake it.
forge wired — captured above + locally (§19).
