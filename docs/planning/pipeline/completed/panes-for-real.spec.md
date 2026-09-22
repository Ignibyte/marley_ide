---
pipeline_id: e6a11eb8-7334-4026-9492-e66fc9c45134
ticket: forge#23 (7007e51f-377b-4e86-8ea0-c3ad761897e9) · local docs/planning/tickets/open/TICKET-023-panes-for-real.md
aar_id: 57b31936-1ee9-455c-8619-edc4a01cd413
status: Phase 5 — Complete PASS
title: panes for real — per-pane terminal sessions + split render + focused-pane model
type: feature
milestone: M1.C
references:
  - docs/specs/SPEC-app-shell.spec.md (R4/R7-R13 shipped algebra; gains the pane-session/focus/rect Rs)
  - docs/marley_architecture/app_shell.md (the status-line placeholder being replaced)
  - crates/marley_app/src/layout.rs (the pure PaneGroup tree this renders)
---

## Title
Replace the `⬜ panes:N docks:LR` status-line placeholder with a real multi-pane terminal: the
center region renders the `PaneGroup` tree recursively, EVERY pane hosts its own live state
{`TerminalSession` + input `Buffer` + caret}, and a focused-pane model routes all input — cmd-d
splits the FOCUSED pane spawning a fresh integrated-zsh session, cmd-w closes it (dropping the
state kills the PTY — the desired BF-alacritty-pty-drop behavior), click focuses, and the focused
pane wears a visible affordance. The M1.C flagship: after this, Marley IS a splittable terminal.

## Scope
### In
- NEW pure module `crates/marley_app/src/workspace.rs` (name = design call): (a) the rect-layout
  fn (`PaneGroup` + ratios + a pure `Rect` → `Vec<(PaneId, Rect)>`); (b) the focus model; (c) the
  pane-state registry generic over the session handle with a spawn-fn seam (the `PtyChannel`
  mock-precedent) — insert-on-split / remove-on-close / route-to-focused, holding
  {session `S`, `Buffer`, caret} per pane.
- **[AMENDED at Inspect]** `crates/terminal_blocks/src/session.rs` — the pump IDLE FAST-PATH
  (leading `WouldBlock` returns immediately; retry sleeps only mid-burst) + `PtyChannel: Send`.
  The critics MEASURED the old behavior at ~12 ms per idle pane per 16 ms tick (98 ms at 8
  panes) — a frame-budget violation inherent to pump-all; and `Workspace::close(pane)`
  (generalizing close_focused) now RETURNS the state so the app reaps sessions on a thread
  (603 ms main-thread freeze measured otherwise), plus shim auto-close of exited panes.
- `crates/marley_app/src/app.rs` (shim): RootView drops its single session/buffer/caret for the
  workspace; recursive/rect-positioned center render with per-pane block rows + prompt + focus
  border (ThemeColors); per-frame pump over ALL sessions; `Keystroke` routing to the focused
  pane; click-to-focus; `dispatch_action` split/close target the FOCUSED pane.
- SPEC-app-shell AMEND: new EARS clauses for rect layout, per-pane session lifecycle, focus
  routing/after-close, and the registry↔tree invariant (+ AC/Test-Plan/Mutation-Targets rows).
- Headed proof: extend the headed lane (tests/headed_*.rs) to drive a REAL cmd-d split and assert
  two live pane content regions + the focus affordance (blank-detector/content-region pattern —
  the shipped headed-test precedent; strict pixel baselines stay deferred per
  AD-claude-headed-visual-baseline-text-tolerance). If the harness lacks keystroke injection, add
  it as an os_shim helper (same ACCEPTED-UNTESTABLE class as its osascript/screencapture calls).
- CHANGELOG + architecture docs (§21).

### Out (explicitly deferred)
- Dock rendering/toggling (seq-3) and palette Enter-dispatch (seq-4) — `dispatch_action`'s
  split/close retargeting to focused is IN, but no new verbs.
- Drag-to-resize ratios, pane maximize/zoom, tabs (M2); session/layout persistence (seq-5/M2);
  scrollback viewport work; any `marley_terminal`/`marley_editor` source change (consumers only).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Per-pane state = {`TerminalSession`, `Buffer`, caret} owned by a registry GENERIC over the
  session type with a spawn-fn seam, so registry/focus/routing logic is mock-tested at cov
  100/MSI 100 without a PTY; the real spawner closure (integrated-zsh `SessionOptions`, shared
  ZDOTDIR) is the only shim line.
- D2 — Rect layout is a PURE fn over a marley_app-local `Rect` value type (gpui `Bounds`
  conversion in the shim); children tile the parent along the split axis by ratio. **[AMENDED at
  Design]** NO last-child remainder arm: the tree is binary with equal (0.5) ratios and f32
  halving is exact, so a remainder arm is an UNKILLABLE equivalent mutant (D-2.2) — extents are
  `parent × ratio_i` with sum-exactness asserted; revisit at M2 drag-resize.
- D3 — Close-the-focused-pane keeps the PTY-killing drop semantics (BF-alacritty-pty-drop is the
  feature here); focus-after-close lands on the collapse SURVIVOR subtree deterministically
  (exact rule = design; must be a pure decision with mutation targets).
- D4 — Fixtures for rect/focus tests are built via the REAL `PaneGroup::split/close` API
  (PR-claude-build-test-fixtures-via-the-real-api) — flat 2x1 AND nested 2x2.
- D5 — The headed proof follows the shipped blank-detector/content-region pattern (drive cmd-d,
  assert two live regions + affordance); strict pixel baselines remain the deferred follow-up.
- D6 — Invariant: after every workspace op, registry keys == `group.panes()` exactly — spec'd,
  asserted in tests after each op sequence.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the rect-layout fn is applied to a PaneGroup and a bounds Rect, the system shall tile the bounds recursively — a Split's children along its axis, child i's extent = parent extent × ratios[i], origins accumulating so children sum EXACTLY to the parent (exact for the binary equal-ratio tree; no remainder arm — see D-2.2's equivalent-mutant finding), a Leaf owning its full rect — for flat 2x1 AND nested 2x2 trees built via the real split API. | unit tests (exact rect equality per pane) |
| REQ-002 | WHEN the split-pane action is dispatched, the system shall spawn a NEW session via the spawn seam for the new pane whose state {session, Buffer, caret} is independent of every other pane's; WHEN close-pane is dispatched, the system shall drop the closed pane's state (the real session's drop kills its PTY). | registry unit tests with a drop-tracking mock session; a real two-session integration test |
| REQ-003 | WHILE a pane is focused, the system shall route key input and command submission ONLY to that pane's buffer/caret/session, and split-pane/close-pane shall target the focused pane; focusing an absent pane shall be rejected without state change. | unit tests |
| REQ-004 | WHEN the focused pane is closed, the system shall move focus deterministically to the collapse survivor (per D3's design rule) and shall refuse to close the last pane (`LastPane`, tree + registry unchanged). | unit tests |
| REQ-005 | WHEN any workspace operation completes, the registry's pane keys shall equal `group.panes()` exactly. | unit assertions after every op sequence |
| REQ-006 | WHEN the headed lane drives cmd-d on the launched shell, the system shall render two live pane regions (each non-blank below the titlebar, split along the expected axis) and a visible focus affordance on exactly the focused pane. | headed `#[ignore]` test (AX + capture analysis), run in the headed lane this pipeline |
| REQ-007 | WHEN `scripts/gates.sh --diff` runs over the staged change, every gate shall be GREEN with coverage 100% and MSI 100% on the pure surface (workspace.rs fully in the denominator; only the app.rs shim excluded). | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — workspace module shape (types/generics/exact fn signatures), the
  focus-after-close rule, rect math (ratio→px + remainder), the pump-over-N + render recursion
  shim shape, harness keystroke-injection check, SPEC-app-shell clause text, full test plan with
  mutation targets.
- **P3 Implement** — workspace.rs + app.rs rewrite + SPEC amendment + CHANGELOG.
- **P3.5 Inspect** — critics on: registry/tree divergence, focus edge cases, pump starvation,
  rect rounding, gpui render correctness, drop/kill semantics.
- **P4 Validate** — the planned tests + headed-lane run + gate GREEN.
- **P5 Complete** — docs, AAR, archive, close #23.
