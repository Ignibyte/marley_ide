# TICKET-795 — The first command in a new zsh on a Mac has no block header

- **Ticket:** LOCAL #795 (bug, the block terminal, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** (set at Plan)
- **Source ticket:** #792's visual check on the Mac, 2026-10-16 ("Seen, not in scope" in its
  notes).
- **Status:** open (waits on a Mac check; not seen on Linux, 2026-10-29)

## Summary
In a new zsh terminal on the Mac (a login shell since #792), the first command drew without
Marley's block header, and the second had one. The first prompt is drawn before the hooks
install at the first `precmd`, so its prompt's frames may not reach Marley. Not yet checked
against Linux's bash and zsh: a terminal's first command there is the comparison that decides
whether this is macOS's or zsh's.

## Acceptance
The first command typed in a new terminal is a block under its header, as every later one is.

## Linux check, 2026-10-29
The first command in a new terminal was a block under its header on Linux in every case tried, at
`57c5d4e5d4`, with zsh 5.9 (shots in that session's scratchpad, not kept):
- a new zsh whose `.zshrc` sources a stand-in for iTerm2's shell integration, as the Mac's
  `.zshrc` sources `~/.iterm2_shell_integration.zsh` (its precmd and preexec after Marley's, OSC
  133 and 1337 marks, a decorated PS1);
- the same shell after a quit and a launch, with restored blocks above it, with restored output
  that holds no command marks (a shell Marley did not integrate), and with a restored block still
  running at the quit;
- the first command sent by an agent's `terminal_run`, as #792's was.

Outside Marley, the same startup run in a PTY (Marley's `.zshenv`, the iTerm2 file, `zsh -l`)
writes `init`, `bootstrapped` and `precmd` before the first prompt, then `preexec` for the first
command, in the same order as for the second.

So what #792 saw is either macOS's own (its zsh, its PTY) or fixed since the Mac's build of
2026-10-16. Deciding needs the current build on the Mac, then a new terminal there and its first
command. The scenario used is kept beside the notes of that session, not in `script/e2e/`.
