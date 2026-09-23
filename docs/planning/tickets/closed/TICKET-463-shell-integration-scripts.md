# TICKET-463 — Shell integration for bash, and its injection at spawn

- **Ticket:** LOCAL #463 (feature, prong 1: T0c)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/463-bash-shell-integration.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D5)
- **Status:** closed

## Summary
Real shells start sending Marley's hooks. Zed's spawn of a local interactive shell injects
Marley's integration. bash comes first, since it is the dev box's shell: it gets `--rcfile` with
Marley's script, which sources the user's `~/.bashrc` first and then reports each prompt and
command. zsh is #465 and fish is #466.

## Acceptance
A real interactive bash spawned by Zed, with no frames printed by hand, leaves a Finished block
with exit 0 and output "hi" for a typed `echo hi`.
