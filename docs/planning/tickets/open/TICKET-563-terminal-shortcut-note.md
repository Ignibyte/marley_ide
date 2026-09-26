# TICKET-563 — A note the first time Marley takes a key a terminal program would have received

- **Ticket:** LOCAL #563 (feature, prong 1 T7: CLI agents in the terminal)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/563-terminal-shortcut-note.spec.md
- **Source ticket:** The Orca second pass of 2026-09-25 (`docs/planning/design-notes/orca-second-pass-2026-09-25.md`), the five smaller details, item 4; Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** open

## Summary
Marley takes Ctrl-G for the rich input whenever a CLI agent runs in the terminal (#481), on
purpose, and the program never sees the key. Claude Code binds Ctrl-G itself, so a user who
presses it for Claude's own feature gets Marley's editor and no explanation. Orca shows a
"Terminal shortcut handled" toast naming the action and the keys, once per action. Marley does
the same: the first time an action of Marley's takes a key from a terminal program, a toast in
that workspace names the key and the action, once per action per profile, remembered in Zed's
key-value store. A key that falls through to the program (`cx.propagate()`) never shows it. #525's
Ctrl-I joins the same list when it lands.

## Acceptance
Ctrl-G at a shell prompt reaches the shell and shows nothing; the first Ctrl-G with a stand-in
agent in the terminal opens the rich input and shows a toast naming Ctrl-G and Rich Input; the
second shows none; after a quit and a launch on the same profile the third shows none either.
