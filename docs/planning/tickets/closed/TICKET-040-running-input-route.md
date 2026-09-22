# TICKET-040 — input routing by running-command state

- **Forge ticket:** #40 `659e95ef-bd2d-4d6f-a464-bb2c3193d1cc` (bug, M1.F — Real Interactivity, seq-1)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `758a132f-9c4a-4be1-8212-3cc820547bcf`
- **Pipeline doc:** ../../pipeline/active/running-input-route.spec.md
- **Source ticket:** forge sprint #6 `73c9385b-91d4-44f9-be99-edf666e6510d` (M1.F — Real Interactivity)
- **Status:** closed

## Summary
BUG chad hit: interactive prompts on the primary screen (Claude Code menus, `read`) don't get arrow
keys — Marley routes them to local history because `input_route` only streams `Raw` on alt-screen.
Fix: `input_route(alt_screen, ctrl, command_running)` (Raw if any) + `TerminalSession::
is_command_running()` (`blocks().current().is_some()`); the app.rs routing passes the new signal, so
while a command runs every key streams to the PTY. Keeps local-edit-at-prompt. Pure surfaces cov/MSI
100. Dep #33.

## Acceptance
`input_route` (8-case truth table) + `is_command_running` (running→true / finished→false) at cov
100/MSI 100; the shim routes to the PTY while running (masked visual — chad verifies arrows against
Claude Code); FULL gate GREEN. Full EARS in the pipeline spec.
