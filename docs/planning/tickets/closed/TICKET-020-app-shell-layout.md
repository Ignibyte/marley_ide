---
ticket: TICKET-020
forge: forge#20 (ed1f5164-ab0d-4ffd-906c-ff76b01bf95c)
status: closed
type: feature
milestone: M1
sprint: M1.B — The Cockpit (seq 4/5)
branch: ticket-020-app-shell-layout
pipeline: docs/planning/pipeline/active/app-shell-layout.spec.md
spec: docs/specs/SPEC-app-shell.spec.md
aar: aba53b1c-44e6-4b14-af2c-11de0836b138
---

# TICKET-020 — app_shell layout: docks + PaneGroup algebra

Sprint M1.B "The Cockpit" seq 4/5. The SPEC-app-shell workspace layout (R4-R13) in `marley_app`: the pure
`PaneGroup` tree algebra (single / split[Before/After+axis] / close[collapse + LastPane] / panes[depth-
first] / neighbor[the R12 boundary rule incl nested 2×2] / ratios[equal renorm]) + `DockSide`/`DockState`
+ the 3-region `RootView` render + cmd-d/cmd-w → split/close. The biggest tested surface of the Cockpit.

## Acceptance
- `layout.rs` (the PaneGroup algebra + DockState toggle) — **cov 100 / MSI 100** (R6-R13; the spec's
  mutation targets on split/close/neighbor/ratios).
- The app.rs shim (the 3-region render + dock reflow + keymap wiring) — ACCEPTED-UNTESTABLE (already
  excluded).
- FULL `scripts/gates.sh` → `GATE GREEN` + a headed both-docks/split assertion. §21: CHANGELOG +
  app_shell.md.
