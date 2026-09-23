# TICKET-463 — Shell integration for zsh, bash and fish

- **Ticket:** LOCAL #463 (feature, prong 1: T0c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from T0 at #461's plan)
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D5)
- **Status:** open

## Summary
`assets/shell_integration/` carries Marley's hooks for zsh, bash and fish. The only surviving
script is the gpui era's zsh one, in `/srv/stacks/marley/crates/marley_app/src/shell_integration.rs`.
Zed's interactive spawn injects them: `ZDOTDIR` for zsh, `--rcfile` for bash,
`XDG_DATA_DIRS` for fish, with the user's own rc still sourced. The place is
`TerminalBuilder::new`, after `insert_zed_terminal_env`, for local interactive shells only.
Known traps:
- with `Shell::System`, alacritty picks `$SHELL` itself, and on macOS makes it a login shell,
  which bash's `--rcfile` does not reach;
- `template.env` is reused by clones, so the injection must be safe to run twice;
- `ShellKind` does not tell bash from zsh.

## Acceptance
A real interactive shell spawned by Zed, with no printed frames, produces a Finished block
with exit 0 and output "hi" for a typed `echo hi`.
