---
pipeline_id: 590e54d6-9c49-47f2-aea5-1af65f66aaf6
ticket: forge#387 (e2ffcfbb-06dd-4c66-90e4-e25758769e64) · local docs/planning/tickets/open/TICKET-387-section-actions.md
aar_id: 377f2b44-d8bd-4190-87ae-6b665071bd20
status: Phase 5 — Complete PASS
title: Per-section ＋ actions — section-scoped new/open affordances + the never-empties guard re-expressed per section
type: feature
milestone: M26
references:
  - docs/planning/pipeline/queued/385-sectioned-rail.spec.md (HARD DEP — promote #385 first)
---

## Title
The #385 sections gain a hover-revealed ＋ per header, with a section-scoped action, and the
tab-close guard semantics are re-expressed (and pinned by tests) in section vocabulary. Slice 3 of
the M26 sectioned shell — the "opening a certain thing goes under that tab" half becomes a
one-click affordance.

## Scope
### In
- **＋ affordances:** each section header renders a ＋ on hover (the #217 block hover-reveal
  idiom — chrome stays quiet at rest). Actions, all REUSING shipped paths:
  - **Terminal＋** → the existing new-terminal path (the #140 "+"-button lineage /
    `spawn_terminal_tab_in` family): a new terminal tab in that project, focused, filed under
    Terminal.
  - **Editor＋** → the existing open-file flow (the file-open path the palette/tree use); the
    resulting CodeView tab files under Editor.
  - **Browser＋** → `open_or_switch_cockpit` (tabs.rs:340) to the Forge section — the Browser
    section's transitional resident until Phase E (#389).
- **Guard re-expression:** pin with tests, in section vocabulary:
  - Terminal keeps ≥1 — `close_tab`'s `LastTerminal` refusal (tabs.rs:254-264; `TabError` variants at 539/542) unchanged (the
    ~90-site `workspace()`-needs-≥1-terminal reality behind the old #202 fork).
  - Editor/Browser may empty out — closing their last tab succeeds and leaves the empty muted
    header (#385 REQ-004).
  - The `LastTab` arm is expected to be SUBSUMED by `LastTerminal` under section semantics
    (a sole remaining tab must be a terminal, since Terminal never empties) — **verify, don't
    assume**; any divergence found is a design finding surfaced in notes, never a silent behavior
    change.
- Pure decision seams: section→action routing table; guard-outcome truth table.

### Out (explicitly deferred)
- Relaxing the Terminal ≥1 invariant (the welcome/launcher direction — its own future train).
- A Browser ＋ that opens a real webview (→ #389/Phase E).
- Per-section context menus, drag-to-section, close-section actions.
- Keybindings for section actions (⌘T etc. already exist and are untouched).

## Reference (§20)
**N/A — Marley-specific.** Section-scoped ＋ actions on a type-sectioned rail follow from chad's
own sectioned-shell design (2026-07-22); no Warp/Zed analog (Warp's "+" is a flat new-session
button — docs/warp_architecture/subsystems/03-terminal-session-core.md, research map; Zed's panels
have feature-scoped affordances, not type-section creators —
docs/zed_architecture/subsystems/07-workspace-panes-palette.md, research map). No source read.

### Prior art
1. **Behavior maps** — the two subsystem docs above (research). Marley's own shipped top-bar "+"
   (M8 #140: `clickat:0.127,0.018` spawns a second terminal — the proven driveable affordance
   class) is the direct precedent for a driven-validated create-button.
2. **Published material** — IDE convention: per-section header actions revealed on hover (VS Code
   section toolbars) — supports hover-reveal over always-visible.
3. **Our permissive deps** — gpui (Apache-2.0): hover + `on_mouse_down` are stock; no owner for
   the routing seam. In-repo owners to REUSE: the new-terminal path (`spawn_terminal_tab_in` /
   `new_terminal_pane` family, cwd inheritance #281), the open-file flow, and
   `open_or_switch_cockpit` (tabs.rs:340). The ＋ must dispatch to these existing verbs — no new
   creation logic.
4. **In-house lessons** — #217 hover-reveal; the #202 parking record (the never-empties guards and
   their ~90-site dependency — this ticket PINS the guards rather than relaxing them);
   PR-claude-selftest-screencapture-shadow-offsets-small-target-clicks (small-button clicks in the
   driven validation verify via mechanism when the pixel-hunt is unreliable).

## Locked-In Decisions
- **D1 — Reuse-only actions.** Each ＋ dispatches an EXISTING verb; zero new creation logic. The
  new code is the routing decision + the affordance render.
- **D2 — Guards unchanged in behavior**, re-expressed + test-pinned in section vocabulary.
  Relaxation is explicitly out (D-#202's territory).
- **D3 — Hover-reveal** (#217 idiom); no resting-state chrome growth.
- **D4 — LastTab subsumption is verified**, with the truth table in tests; a found divergence is a
  surfaced design finding, not a quiet fix.
- **D5 — Pure seams cov/MSI 100** (routing + guard tables); click wiring masked, live-driven.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the Terminal section's ＋ is clicked, a new terminal tab shall open in that project, focused, filed under Terminal. | Unit on routing; driven click → rail + new pane visible. |
| REQ-002 | WHEN the Editor section's ＋ is clicked, the open-file flow shall start, and a resulting CodeView tab shall file under Editor. | Unit on routing; driven flow. |
| REQ-003 | WHEN the Browser section's ＋ is clicked, the Forge cockpit tab shall open-or-switch, filed under Browser. | Unit on routing; driven click. |
| REQ-004 | WHEN closing the last terminal tab is attempted, the close shall refuse exactly as today (LastTerminal), independent of other sections' contents. | Unit truth table (unchanged outcomes). |
| REQ-005 | WHEN the last Editor (or Browser) tab is closed while a terminal remains, the close shall succeed and the rail shall render that section's empty header. | Unit + capture. |
| REQ-006 | WHILE the pointer is not over a section header, the ＋ shall not render (hover-reveal). | Capture pair (rest vs hover). |

## Phase Plan
- **P2 Design** — affordance geometry on the header row, the routing seam's shape, the guard truth
  table (incl. the LastTab subsumption proof), Editor＋'s exact open-file entry point.
- **P3 Implement** — tabs.rs/app.rs per design.
- **P3.5 Inspect** — critics (routing on the wrong project when multiple projects render; ＋ vs
  header-click collapse (#386) hit-target overlap; guard edge cases: cockpit-only close paths).
- **P4 Validate** — unit matrix + `--diff` gate + driven captures per REQ.
- **P5 Complete** — CHANGELOG, app_shell.md, archive, close #387.
