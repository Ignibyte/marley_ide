# TICKET-949 — A zsh or fish given with arguments loses Marley's integration

- **Ticket:** LOCAL #949 (bug, the block terminal)
- **Owner:** 8cdbf33f-812b-485d-9b60-8ffdd6fa5c3b
- **Pipeline doc:** (set at Plan)
- **Source ticket:** found while checking #795 on Linux, 2026-10-29.
- **Status:** open

## Summary
`terminal.shell` set to `{"with_arguments": {"program": "zsh", "args": ["-l"]}}` starts a zsh
with no blocks, no prompt editor and "0 blocks" in the status bar: `marley_shell_integration`
(`crates/terminal/src/terminal.rs`) returns a `Shell::WithArguments` unchanged, since #463. For
bash that is needed, since its integration is an `--rcfile` argument the user's own arguments may
contradict (`-l` ignores `--rcfile`). zsh's integration (#465) and fish's (#466) are environment
only, so a zsh or fish with the user's arguments could keep them and still get Marley's
`ZDOTDIR` or `XDG_DATA_DIRS`. Nothing tells the user why the terminal is plain.

## Acceptance
A zsh or fish given with arguments starts with those arguments and Marley's integration; a bash
given with arguments stays plain, and the reason is said where the user looks for it.
