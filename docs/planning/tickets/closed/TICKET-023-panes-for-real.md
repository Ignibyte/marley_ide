# TICKET-023 — panes for real: per-pane terminal sessions + split render + focused-pane model

- **Forge ticket:** #23 `7007e51f-377b-4e86-8ea0-c3ad761897e9` (feature, M1.C — The Wired Cockpit, seq-2 FLAGSHIP)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `57b31936-1ee9-455c-8619-edc4a01cd413`
- **Pipeline doc:** ../../pipeline/active/panes-for-real.spec.md
- **Source ticket:** forge sprint #3 `b295976b-4f32-4edc-be3c-aecd80b02785` (M1.C — The Wired Cockpit)
- **Status:** closed

## Summary
M1.B shipped the pure PaneGroup algebra but the app renders a `⬜ panes:N` status line over ONE
shared terminal session. Make it real: recursive rect-tiled center render, every pane owning its
own {TerminalSession, Buffer, caret} (split spawns a fresh integrated zsh; close drops the state,
killing the PTY), and a focused-pane model routing all input — cmd-d/cmd-w act on the focused
pane, click focuses, the focused pane wears a ThemeColors affordance.

## Acceptance
Pure workspace layer (rect layout / focus / spawn-seamed registry) at cov 100/MSI 100; the
registry↔tree invariant holds after every op; a headed-lane test drives a REAL cmd-d and sees two
live pane regions + the focus affordance; FULL gate GREEN [--diff]. Full EARS in the pipeline
spec (REQ-001..007).
