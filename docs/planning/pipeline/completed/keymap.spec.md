---
pipeline_id: 247ca15e-e6dc-4ad0-b3f1-fa3551357cc3
ticket: forge#18 (ecdcab72-50ae-4b0d-b1d5-7592b814532f) · local docs/planning/tickets/open/TICKET-018-keymap.md
aar_id: f9a6f321-56a2-402c-a78e-2fa85bd209bd
sprint: M1.B — The Cockpit (cbc92bf0) seq 2/5
status: Phase 5 — Complete PASS
title: app_shell keymap + action dispatch (cmd-shift-p / cmd-d / cmd-w)
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-app-shell.spec.md
  - ../../../marley_architecture/app_shell.md
---

## Title

TICKET-018 — the [`SPEC-app-shell`](../../../specs/SPEC-app-shell.spec.md) keymap layer (R19–R20) added to
`marley_app`: a pure `KeyBinding` value type + `Keymap` (binding→action-name) with `default_bindings()` +
`action_for()`, plus the gpui key-chord → named-action interception in `RootView` (shim). M1.B "The
Cockpit" seq 2/5 — the enabler that unlocks the palette-open (#19) + pane split/close (#20) chords.

## Scope

### In
- **`keymap.rs` (PURE)** — `KeyBinding { cmd, ctrl, alt, shift: bool, key: String }` (a value type,
  constructible in a plain `#[test]`); `Keymap { bindings: Vec<(KeyBinding, String)> }`;
  `Keymap::default_bindings() -> Self` (R19: cmd-shift-p→"open-command-palette", cmd-d→"split-pane",
  cmd-w→"close-pane"); `action_for(&self, &KeyBinding) -> Option<&str>` (R20: `None` for an unbound chord).
- **`app.rs` (SHIM)** — a `keymap: Keymap` field on `RootView` (`default_bindings()`); a
  `binding_from_keystroke(&Keystroke) -> KeyBinding` translation; in `on_key_down`, a bound chord is
  intercepted (NOT forwarded to the terminal prompt) — the actual palette/pane HANDLERS land in #19/#20.
- §21 — CHANGELOG + note the keymap layer in `docs/marley_architecture/app_shell.md`.

### Out / deferred
- The action HANDLERS (opening the palette → #19; splitting/closing panes → #20). #18 delivers the keymap
  + the interception wiring only.
- User-defined keybindings / persistence → the settings crate (#21 / M2).

## Acceptance Criteria (EARS — adopt SPEC-app-shell R19–R20)
- **AC-R19** — `Keymap::default_bindings().action_for(b)` shall return `Some("open-command-palette")` for
  the cmd-shift-p chord, `Some("split-pane")` for cmd-d, and `Some("close-pane")` for cmd-w. Verify:
  `default_keymap_maps_named_chords` unit test.
- **AC-R20** — WHEN `action_for` is queried with a binding not in the keymap, it shall return `None`.
  Verify: `action_for_unbound_is_none` unit test.
- **AC-gate** — `keymap.rs` (KeyBinding + Keymap + default_bindings + action_for) is cov 100 / MSI 100;
  the app.rs shim (the field + `binding_from_keystroke` + the interception) is ACCEPTED-UNTESTABLE (app.rs
  is already `mutants::skip` + rust_cov-excluded from #16 — no gates.sh change); FULL `scripts/gates.sh`
  → `GATE GREEN [diff]` (gate-15 rides the existing `shell_dark` headed baseline — no new UI surface).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **The pure/shim seam** — `KeyBinding` + `Keymap` are PURE (gpui-free; a `KeyBinding` is plain bools +
   a `String`, so `action_for`/`default_bindings` unit-test with zero gpui). The gpui `Keystroke`→
   `KeyBinding` translation + the `on_key_down` interception are SHIM in app.rs (already excluded).
2. **Interception, not handling** — a bound chord is consumed (kept off the terminal prompt) but its
   handler is a #19/#20 concern; #18's dispatch recognises the action string. No stub handler logic that
   would be dead/untested — the interception is a single `if action_for(&b).is_some() { return }` guard.
3. **Frozen contract** — the `KeyBinding` / `Keymap` / `default_bindings` / `action_for` signatures are a
   frozen M2-facing contract (per the spec).

## Phase Plan
- **P2 Design** — the `KeyBinding` shape + `Keymap` storage + the mutation map (the 3 mappings + the
  unbound-None) + the `binding_from_keystroke` shim shape. Brief (small ticket).
- **P3 Implement** — keymap.rs + the app.rs field/translation/interception + lib.rs re-export.
- **P3.5 Inspect** — mutation of `action_for` (the find/eq) + `default_bindings` (each mapping); confirm
  only app.rs is the shim.
- **P4 Validate** — the 2 unit tests (R19/R20) + FULL gate (incl the still-#[ignore] headed_shell).
- **P5 Complete** — §21; close #18; → #19 command palette.
