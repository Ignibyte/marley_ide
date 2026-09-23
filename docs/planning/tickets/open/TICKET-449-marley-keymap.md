# TICKET-449 — The Marley keymap

- **Ticket:** LOCAL #449 (feature, workbench shell W5b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (not specced; split from #441 at its promotion)
- **Source ticket:** ../../pipeline/completed/441-marley-terminal-routing.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
The terminal keys still reach the bottom Terminal Panel after #441: `` ctrl-` `` is Zed's
`terminal_panel::Toggle`. A Marley keymap in `marley_workbench`, loaded with `KeymapFile::load`
and tagged `KeybindSource::Default`, binds `` ctrl-` `` and `ctrl-~` to Marley actions, plus a
chord for New Agent. It hooks in with one line at the end of `load_default_keymap` in
`crates/zed/src/zed.rs` (a touchpoint, with its ledger row). Each Marley action does the Zed
action it shadows when the layout is Zed.

## Acceptance
In the Marley layout `` ctrl-` `` shows or hides the project's center terminal and `ctrl-~`
opens a new one, with the bottom panel left closed. In the Zed layout both keys do what Zed's
own bindings do. The bindings survive a keymap reload and lose to a user binding on the same
keys. The EARS criteria come at its promotion. The draft is in #441's queued spec, REQ-005 and
REQ-006, under `docs/planning/pipeline/completed/` once #441 closes.
