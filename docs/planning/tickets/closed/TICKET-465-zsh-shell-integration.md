# TICKET-465 — Shell integration for zsh

- **Ticket:** LOCAL #465 (feature, prong 1: T0c, second shell)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/465-zsh-shell-integration.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D5)
- **Status:** closed

## Summary
zsh gets Marley's integration through `ZDOTDIR`, at #463's injection point. The gpui era's zsh
script (`/srv/stacks/marley/crates/marley_app/src/shell_integration.rs`, Marley's own) is the
start: `add-zsh-hook preexec` and `precmd`, and `__marley_quote`. The injected `ZDOTDIR` must
source the user's own `.zshenv`, `.zprofile`, `.zshrc` and `.zlogin` from their original
`ZDOTDIR`, or `$HOME`. zsh 5.9 is on the dev box.

## Acceptance
A real interactive zsh spawned by Zed leaves a Finished block with exit 0 and output "hi" for a
typed `echo hi`, with the user's own rc files still sourced.
