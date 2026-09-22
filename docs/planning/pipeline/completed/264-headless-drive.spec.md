---
pipeline_id: 370eff15-9eb9-4b4b-9982-2ba842d33e4e
ticket: forge#264 (bdae95c9-2b24-448d-a42d-8296fd230c4d) · local docs/planning/tickets/open/TICKET-264-headless-drive.md
aar_id: 7c42b598-3032-442a-bb76-48ec050727ba
status: Phase 5 — Complete PASS
title: Headless driven testing — adopt gpui VisualTestContext (input injection + state asserts)
type: chore
milestone: M16
references:
  - docs/zed_architecture/crates/gpui.md
---

## Title
Adopt gpui's test harness so driven validation stops depending on an
unlocked foreground GUI. **Ground truth for gpui 0.2.2 (verified in the
vendored source, correcting the ticket's premise):** `VisualTestContext` +
`TestAppContext` EXIST behind the `test-support` feature — headless keystroke
/input/mouse injection into a real window entity tree, `dispatch_action`,
`draw`, run-until-parked — but `Window::render_to_image` /
`capture_screenshot` do NOT exist in 0.2.2 (the reference doc verified those
in a NEWER Zed tree; the 0.2.2 TestWindow's platform `draw(&scene)` is a
no-op — no pixel output). So this slice adopts the INPUT+STATE half — which
retires the worst screencapture hazards for behavioral checks (a locked
screen blocks CGEvents + capture; shadowing drives the wrong window) — and
explicitly defers the PIXEL half to a gpui upgrade ticket.

## Scope
### In
- `gpui = { version = "0.2.2", features = ["test-support"] }` as a
  DEV-dependency of `marley` (the normal dep stays featureless).
- A new **headless driven test lane**: `crates/marley_app/tests/
  headless_drive.rs` using `#[gpui::test]` + `VisualTestContext` — boot the
  RootView in a test window, inject real keystrokes, assert STATE:
  1. boot sanity (workspace ≥1 project/tab; no panic through several
     draw/park cycles);
  2. ⌘T → tab count +1 (the keymap → dispatch path end-to-end, headless);
  3. the #265 ⌘D split headless: terminal tab ⌘D → new tab; editor-tab ⌘D →
     NO new tab (the KeyContext resolution driven through the real
     on_key_down listener).
  (The exact assertable state surface is design's job — RootView fields are
  private; may need `#[cfg(test)]`-gated accessors or window-title/tab-count
  observables.)
- If RootView's PTY boot makes full-boot tests infeasible headless, the
  fallback slice (design decides): drive a REDUCED root (the test constructs
  the workspace model + keymap dispatch without live PTYs) — still proving
  keystroke→dispatch→state headless.
- Harness docs updated: scripts/selftest/README.md + marley_visual_harness.md
  record the new lane + when to use which (headless-first; screencapture only
  for pixel proofs).
- Follow-up ticket filed: pixel capture via a gpui upgrade
  (render_to_image), to retire the screenshot half.

### Out (explicitly deferred)
- Pixel/screenshot capture + gate:15 baseline replacement (needs a gpui
  version with render_to_image — the follow-up ticket).
- Retiring drive.swift (still needed for pixel captures + pointer-geometry
  checks until the pixel half lands).
- Porting the EXISTING headed tests off the harness.

## Reference (§20)
N/A — testing infrastructure on Marley's own Apache-2.0 dependency (gpui's
own public test harness, used as shipped). No reference-app behavior is
matched; nothing user-facing changes.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — test-support only as a dev-dependency (the shipping binary is
  unchanged).
- D2 — This slice = INPUT+STATE; pixels deferred with a filed follow-up (the
  0.2.2 test platform provably cannot produce them).
- D3 — Tests target the REAL RootView boot if feasible (design probes the
  PTY/config-dir requirements; tests must not touch the user's real config —
  the existing `*_in(dir)` seams + env overrides apply), else the reduced
  root fallback with the decision recorded.
- D4 — The lane runs in gate:3 (plain `cargo nextest run`) — headless by
  construction, no #[ignore], no gate:15 conditionality.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The workspace shall compile with gpui `test-support` as a dev-dep and the new test file shall run headless under plain `cargo nextest run` (no GUI, no Accessibility permission, no unlocked screen). | nextest run in-transcript |
| REQ-002 | A headless test shall boot the app's root view in a test window and complete ≥1 draw + run_until_parked cycle without panic. | the test itself |
| REQ-003 | A headless test shall inject ⌘T via simulate_keystrokes and observe the tab count increase by 1 (keystroke → keymap → dispatch_action → state, end-to-end). | the test itself |
| REQ-004 | Headless tests shall reproduce the #265 ⌘D split: on a terminal tab ⌘D adds a tab; on the editor tab ⌘D adds none. | the tests |
| REQ-005 | The tests shall leave the user's real config untouched (config-dir override respected). | design review + a tempdir assertion in the tests |
| REQ-006 | The harness docs shall describe the headless lane and the deferred pixel half; the pixel follow-up ticket shall exist on the forge. | docs review + ticket id |

## Phase Plan
- **P2 Design** — probe RootView::new's dependency surface (PTY spawn, config
  dirs, Assets) under TestAppContext; pick full-boot vs reduced-root; the
  state-observation surface; test file structure.
- **P3 Implement** — Cargo dev-dep + the test file (+ any #[cfg(test)]
  accessors) + doc updates.
- **P3.5 Inspect** — critics: test honesty (no tautologies), config isolation,
  flake risk (PTY timing under run_until_parked).
- **P4 Validate** — nextest runs green repeatedly (3× for flake); gate --diff.
- **P5 Complete** — CHANGELOG, marley_visual_harness.md/testing docs, pixel
  follow-up ticket, AAR, archive, close.
