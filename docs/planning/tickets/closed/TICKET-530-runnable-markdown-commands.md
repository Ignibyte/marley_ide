# TICKET-530 — Shell commands in the Markdown preview go to the terminal

- **Ticket:** LOCAL #530 (feature, prong 1 with the editor: the terminal from a runbook)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/530-runnable-markdown-commands.spec.md
- **Source ticket:** Chad, 2026-09-25, approving all seven items of the Warp once-over (`docs/planning/design-notes/warp-once-over-2026-09-25.md`, item 6, "Runnable commands in the Markdown viewer")
- **Status:** closed

## Summary
Runbooks and READMEs are mostly commands to copy into a terminal. In Zed's Markdown preview a code
block has a copy button and nothing else. Warp's Markdown viewer gives each shell code block a
button that puts the command into the active terminal's input without running it. Marley adds that
button to the preview's shell code blocks (a fence naming `sh`, `shell`, `bash`, `zsh` or `fish`,
or naming nothing): a click clears the prompt's line in the terminal Chad used last, puts the
command there, and takes him to that terminal, where Enter runs it. A terminal busy with a program
gets nothing typed, and a toast says why. A block of several lines goes in as one bracketed paste,
which the shell holds until Enter.

## Acceptance
Hovering a `bash` block in the preview shows the button; a click leaves the command at the prompt
of the terminal used last, not run, with that terminal focused; a `rust` block has no button.
