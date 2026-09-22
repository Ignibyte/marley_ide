---
ticket: TICKET-018
forge: forge#18 (ecdcab72-50ae-4b0d-b1d5-7592b814532f)
status: closed
type: feature
milestone: M1
sprint: M1.B — The Cockpit (seq 2/5)
branch: ticket-018-keymap
pipeline: docs/planning/pipeline/active/keymap.spec.md
spec: docs/specs/SPEC-app-shell.spec.md
aar: f9a6f321-56a2-402c-a78e-2fa85bd209bd
---

# TICKET-018 — app_shell keymap + action dispatch

Sprint M1.B "The Cockpit" seq 2/5. The SPEC-app-shell keymap layer (R19-R20) in `marley_app`: a pure
`KeyBinding` + `Keymap` (binding→action) with `default_bindings()` (cmd-shift-p→open-command-palette,
cmd-d→split-pane, cmd-w→close-pane) + `action_for()` (`None` for an unbound chord), plus the gpui
chord-interception in `RootView`. The enabler for the palette (#19) + pane split/close (#20) chords.

## Acceptance
- `keymap.rs` (KeyBinding + Keymap + default_bindings + action_for) — **cov 100 / MSI 100** (R19 + R20).
- The app.rs shim (the `keymap` field + `binding_from_keystroke` + the `on_key_down` interception) —
  ACCEPTED-UNTESTABLE (app.rs already `mutants::skip` + rust_cov-excluded from #16).
- FULL `scripts/gates.sh` → `GATE GREEN` (gate-15 rides the existing `shell_dark` baseline). §21:
  CHANGELOG + app_shell.md.
