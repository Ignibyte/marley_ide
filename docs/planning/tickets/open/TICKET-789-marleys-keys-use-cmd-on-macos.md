# TICKET-789 — Marley's own keys use Cmd on macOS

- **Ticket:** LOCAL #789 (feature, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** (set at Plan)
- **Source ticket:** the move's plan, Phase 5 item 1 (Chad, 2026-10-16, "lets make it happen").
- **Status:** open

## Summary
Marley's keymap (`crates/marley_workbench/keymap.json`) binds its own actions with `ctrl-` (29
bindings), where Zed binds its own on macOS with `cmd-`, so on a Mac Marley's keys sit beside
Zed's Cmd keys on Ctrl. Zed's `secondary-` modifier is Cmd on macOS and Ctrl elsewhere. Marley's
app-level bindings (undo close terminal, bookmark, the browser's address bar, new tab and reload,
close, take over, find in block and the like) become `secondary-`, which changes nothing on
Linux. The keys that stand for a terminal's own (the prompt editor's `ctrl-w`, `ctrl-e`, `ctrl-b`,
the nulls for `ctrl-r`, `ctrl-o`, `ctrl-q`, the rich input's `ctrl-c`) stay `ctrl-` on both, as a
shell's do on a Mac. Each binding's choice is checked against Zed's macOS keymap for a clash.

## Acceptance
On the Mac, Marley's own actions answer Cmd where Zed's answer Cmd, the shell keys in the prompt
editor answer Ctrl, and on Linux every binding is as before.
