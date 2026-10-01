# TICKET-573 — English at the prompt, second stage: a System One reading for the lines the rules leave open

- **Ticket:** LOCAL #573 (feature, prong 1 T3 after #557 (the local rules); the System One layer's typed-line use, on #565)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/573-english-at-the-prompt-second-stage.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the System One uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md` and the typed-line use of
  `docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md` ("Where a System
  One model fits": "only for lines the local rules leave open, asked after about 250 ms without
  typing so Enter never waits on the network"; "opt-in, redacted, or a local model"), on the layer
  TICKET-565 builds, with his rules: "local first and then jev second", and "we need probably every
  aspect of this configurable and turned off / on where the system will use or wont use it.
  Otherwise this becomes a jev required system." #557's D1 names this ticket as its second stage.
  Re-bound at promotion (2026-09-30) to #627's prompt editor, the default input since then.
- **Status:** closed

## Summary
#557 labels the line typed at a shell prompt by local rules (a first word not on the PATH reads as
English; flags, pipes and paths read as a command; a command's name followed by three or more
words with an English marker reads as English) and shows a hint in the suggestion slot after the
cursor, with Ctrl+Shift+Enter to send the line to an agent. Its weakest rule is the middle: `find
all the large files in this repo`, `kill the dev server`, `rm the old build folder`, `echo what is
this`, a command's name followed by plain words, which the shell would run on those words. This
ticket asks #565's layer about such lines, a choice among command, request, comment, a command
followed by English, and cannot tell, only after 250 ms without typing and never in Enter's path,
and shows the reading in #557's slot: the words, and for the dangerous middle a warning in the
warning color before Enter. The line leaves the box only for a project on the allow list, through
#516's redactor, and never when it holds a candidate secret; a local provider, when #565 has one,
keeps it on the machine. Off by default as the use `typed_line` in `marley.system_one.uses`.

## Acceptance
In the prompt editor, a line #557 reads as English shows its hint, and Ctrl+Shift+Enter asks the
agent. On the `replay` provider with the project listed, `find all the large files in this repo` typed
and left for 250 ms shows the replayed reading in the slot, and in `act` `rm the old build folder`
shows a warning naming `rm`; typing again within 250 ms makes no call; a line with a candidate
secret makes no call; Enter pressed before the reading runs the shell's line at once with no wait;
the mode `off`, or the project unlisted, makes no call and leaves #557's slot as it was.
