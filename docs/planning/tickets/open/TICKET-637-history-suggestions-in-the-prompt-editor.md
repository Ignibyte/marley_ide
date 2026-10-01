# TICKET-637 — History suggestions in the prompt editor

- **Ticket:** LOCAL #637 (feature, prong 1: the follow-up #627 deferred)
- **Owner:** unassigned
- **Pipeline doc:** none yet
- **Source ticket:** #627's Scope Out ("#484's ghost text and → inside the editor … a follow-up
  ticket"), found unminted by #635's golden run
- **Status:** open

## Summary
#484 draws the rest of a command from the shell's history after the terminal's cursor, and →
takes it. Since #627 the prompt editor holds the line at every prompt by default, and it shows no
such suggestion, so at the default a history suggestion never appears; it still does with
`marley.prompt_editor` off. #627 left the editor's version to a follow-up ticket that was never
minted. This is that ticket: the same suggestion, from the same source (`autosuggest`'s history
and the session's commands), drawn in the prompt editor after its cursor (Zed's edit prediction
inlay, as #573's hint already is), and → at the end of the line taking it.

## Acceptance
In bash with the prompt editor at its default, typing a prefix of a history command shows the
rest dimmed in the editor, → takes it and Enter runs it; a prefix nothing starts shows nothing;
a command run in the session is suggested over the file's (#484's scenario, with the editor on).
