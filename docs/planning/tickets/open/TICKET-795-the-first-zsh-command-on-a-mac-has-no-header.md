# TICKET-795 — The first command in a new zsh on a Mac has no block header

- **Ticket:** LOCAL #795 (bug, the block terminal, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** (set at Plan)
- **Source ticket:** #792's visual check on the Mac, 2026-10-16 ("Seen, not in scope" in its
  notes).
- **Status:** open

## Summary
In a new zsh terminal on the Mac (a login shell since #792), the first command drew without
Marley's block header, and the second had one. The first prompt is drawn before the hooks
install at the first `precmd`, so its prompt's frames may not reach Marley. Not yet checked
against Linux's bash and zsh: a terminal's first command there is the comparison that decides
whether this is macOS's or zsh's.

## Acceptance
The first command typed in a new terminal is a block under its header, as every later one is.
