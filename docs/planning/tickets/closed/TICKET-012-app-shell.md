---
ticket: TICKET-012
forge: forge#16 (c17a08e5-9f30-4c0a-a0d6-1a95fd2b71e6)
status: closed
type: feature
milestone: M1
sprint: M1.A — The Usable Terminal (seq 5/5, FINALE)
branch: ticket-012-app-shell
pipeline: docs/planning/pipeline/active/app-shell-minimal.spec.md
spec: docs/specs/SPEC-app-shell.spec.md
aar: 1090af94-6457-4151-80a3-1accaa2f2916
---

# TICKET-012 — marley_app (app-shell, minimal) — THE FINALE

Sprint M1.A seq 5/5, the LAST ticket of the /goal. The first runnable Marley: `run()` boots one "Marley"
gpui window (crate **`crates/marley_app`**, package `marley`, lib `marley_app`, bin `marley`) mounting a
live `marley_terminal` pane + the `marley_editor` input; type → Enter → the command runs → a Block
appears; Dark-themed from `marley_ui_components`. The minimal cut of SPEC-app-shell (docks, pane algebra,
palette, keymap, multi-theme → M1.B). **End state: `cargo run -p marley` opens a usable bare terminal.**

## Acceptance
- The PURE wiring surface (`themes` ThemeRegistry, `input` apply_key/submit_line, `terminal_view`
  rows_from_blocks) — **cov 100 / MSI 100**.
- The gpui/PTY shim (`app.rs` RootView Render + run() + pump-timer, `bin/marley.rs`) — ACCEPTED-UNTESTABLE
  (mutants::skip + rust_cov exclude), proven by a HEADED visual test (AX "Marley" + the `shell_dark`
  masked baseline).
- Deps gpui + the 3 mount crates; NO nucleo/marley_core. FULL `scripts/gates.sh` → `GATE GREEN` INCL
  **gate-15** (the `crates/marley_app` dir makes the component asserted, not silently skipped). §21
  CHANGELOG + arch doc.

## Notes
The terminal mount is the brain-observation anchor (AD-claude-brain-agent-session-supervision-001) —
RootView holds the session addressable. The live event-loop pump is the one surface the spike never
exercised (prototype first). See the pipeline spec/notes for the crate-layout trap + the full cut.
