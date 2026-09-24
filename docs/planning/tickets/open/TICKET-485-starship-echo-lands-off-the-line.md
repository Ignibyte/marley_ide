# TICKET-485 — Typed text after the first character goes missing on a long starship prompt

- **Ticket:** LOCAL #485 (bug, prong 1: T0)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (minted at promotion)
- **Source ticket:** found by #483's e2e runner, 2026-09-23
- **Status:** open

## Summary
In a bash terminal with the user's own prompt (starship, with a 140-column path in it), keys
typed at the prompt reach bash (the command runs, and its output prints), but only the first
character shows on the line: the rest never appears, and the cursor sits about 18 columns left
of where it should be, inside the prompt's path. Ctrl-L, which makes readline redraw the whole
line, shows the right text with the cursor after it. With a plain `$ ` prompt the same keys echo
correctly, and `stty size` (65 × 321) matches the grid Marley draws. So readline's incremental
echo lands somewhere else after the first character: a prompt width readline counts wrong, or
Marley's escape scanning (the shell-hook frames, #462; the notification scanner, #478) taking
bytes that belong to the line. Not yet seen with real keys, nor with a short path.

## Acceptance
With the user's starship prompt on a long path, text typed at the prompt shows as typed, with the
cursor after it. The e2e scenario reproduces it first: keys to Marley's window, shots of the line.
