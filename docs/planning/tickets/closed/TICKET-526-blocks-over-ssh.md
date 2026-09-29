# TICKET-526 — Blocks keep working over ssh

- **Ticket:** LOCAL #526 (feature, prong 1 T0c: shell integration)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/526-blocks-over-ssh.spec.md
- **Source ticket:** Chad, 2026-09-25, on Warp's Warpify subshells: "this would be awesome" (`docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 3; the SSH notes of `docs/orca_architecture/04-remote-control-and-mobile.md` §2.10 and `05-terminal-and-workspace.md` §2.3)
- **Status:** closed

## Summary
Blocks stop at the first `ssh`: the remote shell has none of Marley's hooks, so the whole session
is one running block, and much of the work on this box is ssh. Marley's bash and zsh
integrations gain an `ssh` function. For an interactive login it adds `-t` and a small POSIX sh
bootstrap as the remote command, which writes Marley's two scripts into a temporary directory on
the host, starts the user's login shell with them and removes the directory once the shell has
read it. Nothing is installed on the host. Commands typed there become blocks as local ones do.
The terminal's nonce (#474) stays on the machine: the wrapper mints a nonce for the connection,
announces it in a frame signed with the terminal's own, and hands it to the remote shell. Every
frame now carries its shell's nonce, so the terminal knows which shell waits at the prompt, and
Rerun only offers a block to the shell that ran it. `docker exec` is the next slice.

## Acceptance
In a Marley terminal, `ssh` to a host with bash or zsh gives blocks for the commands typed there,
with Rerun on the remote blocks while the remote shell waits at its prompt and on the local ones
once the connection ends; `command ssh`, a command given to ssh, and a host with another shell
run as plain ssh.
