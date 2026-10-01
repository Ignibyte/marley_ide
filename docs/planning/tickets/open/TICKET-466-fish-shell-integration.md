# TICKET-466 — Shell integration for fish

- **Ticket:** LOCAL #466 (feature, prong 1: T0c, third shell)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/466-fish-shell-integration.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D5)
- **Status:** open

## Summary
fish gets Marley's integration by prepending a Marley directory to `XDG_DATA_DIRS`, whose
`fish/vendor_conf.d/marley.fish` fish sources at startup. It is hooked on the `fish_preexec`,
`fish_postexec` and `fish_prompt` events, at #463's injection point. fish 4.9.2 is on the dev box since
2026-09-30.

Since #474, a command's frame carries the terminal's nonce (`MARLEY_SHELL_NONCE`, taken out of
the environment before the user's files run), as bash's and zsh's do.

## Acceptance
A real interactive fish spawned by Zed leaves a Finished block with exit 0 and output "hi" for a
typed `echo hi`.
