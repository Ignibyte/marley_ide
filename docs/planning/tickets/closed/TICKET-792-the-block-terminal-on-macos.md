# TICKET-792 — The block terminal on macOS

- **Ticket:** LOCAL #792 (bug, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [792-the-block-terminal-on-macos.spec.md](../../pipeline/completed/792-the-block-terminal-on-macos.spec.md)
- **Source ticket:** seen in #788's check on the Mac, 2026-10-16.
- **Status:** closed

## Summary
In a new terminal of the Mac's Marley (zsh, macOS's login shell), the shell's plain prompt
shows under "Last login: …":
- no prompt editor docks under it, and no blocks form (`terminal_list` says `blocks: 0`);
- `terminal_run` refuses with "a program runs in the terminal's foreground".

On macOS, the vendored alacritty starts the shell through `/usr/bin/login -flp <user> <shell>`
(`vendor/alacritty_terminal/src/tty/unix.rs`), so the terminal's child is `login`, not the
shell. Marley's integration injects zsh through a `ZDOTDIR` (`marley.zsh`) and recognizes the
shell, its prompt and its foreground program by process, and some part of that does not hold
under `login`. The block terminal (#624 onward), the prompt editor, autosuggestions and
`terminal_run` are Marley's core, so they have to work on the Mac.

## Acceptance
In a new zsh terminal on the Mac, Marley's prompt editor docks at the prompt, a command run there
forms a block with its exit code, and `terminal_run` types at the prompt.
