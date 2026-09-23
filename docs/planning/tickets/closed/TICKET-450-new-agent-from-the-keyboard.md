# TICKET-450 — New Agent from the keyboard

- **Ticket:** LOCAL #450 (feature, workbench shell W5c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/450-new-agent-from-the-keyboard.spec.md
- **Source ticket:** ../../pipeline/completed/449-terminal-keys.spec.md (Out) · ../../../marley/workbench-shell.md (D4, D7)
- **Status:** closed

## Summary
Chad could not find how to start an agent; W4's `+` menu answers that for the mouse. A New
Agent chord opens a picker over the installed agent CLIs and Zed's agents, and starts the
choice in the active project as the `+` menu does. No Zed action exists to catch for it, so it
needs a binding of Marley's own: the Marley keymap of workbench-shell D7, a JSON file in
`marley_workbench` parsed with `KeymapFile::load`, tagged `KeybindSource::Default`, and bound
from one line at the end of `load_default_keymap` in `crates/zed/src/zed.rs` (a touchpoint,
with its ledger row). The chord is chosen at planning and checked against Zed's defaults in
every context it can reach (`PR-claude-new-chord-shadowed-by-hardcoded-key-001`).

## Acceptance
In either layout the chord opens the picker, and a choice starts that agent in the active
project: a CLI in a new center terminal, a Zed agent as a new thread in the Agent Panel. The
binding survives a keymap reload and loses to a user binding on the same keys. Full EARS in
the completed spec. Shipped 2026-09-23.
