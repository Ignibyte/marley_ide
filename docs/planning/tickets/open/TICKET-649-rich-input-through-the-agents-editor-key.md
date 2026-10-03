# TICKET-649 — Rich input through the agent's own editor key

- **Ticket:** LOCAL #649 (feature, prong 1 T7: T7e's rich input; Claude Code and Codex on their
  own tools, B4)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/649-rich-input-through-the-agents-editor-key.spec.md
- **Source ticket:** `docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`
  (the fights table's rich input row, idea B4); Chad, 2026-10-02, on B4 ("rich input through the
  agents' own editor key (a `marley edit` command as `$EDITOR`; the overlay only for agents with
  no editor key)"): "Yes"
- **Status:** open

## Summary
Claude Code, Codex, Gemini CLI and OpenCode each have a key that writes the prompt to a file, runs
`$VISUAL` or `$EDITOR` on it, and reads the file back as the draft when the editor exits. Marley's
rich input (#481) draws a Zed editor over the agent's prompt instead, takes Ctrl-G while an agent
runs (so Claude Code's, Codex's and Gemini CLI's own Ctrl-G never reaches them), and hands the text
over as a paste, a 200 ms wait and an Enter (#594). This ticket gives the terminals Marley opens for
agents its own editor, `marley-edit`: a program in Marley's data directory, named in `VISUAL` and
`EDITOR`, that opens the file in a center tab of that terminal's workspace and returns when the
tab closes, as `zed --wait` does. With `marley.agent_editor_in_tab` on, Ctrl-G and the agent bar's
Rich Input button send the agent its own editor key, so the prompt comes back through the agent's
own path, unsent, with nothing typed by Marley. The overlay stays for terminals Marley did not open
for the agent, for remote projects and restored terminals, and with the switch off, which is the
default because the switch changes what Ctrl-G does and which editor programs in agent terminals
get.

## Acceptance
With the switch on, an agent Marley starts in a local project sees `VISUAL` and `EDITOR` naming
`marley-edit`, even when the user's `.bashrc` exports its own; a New Terminal keeps the user's.
Ctrl-G there reaches the agent as its editor key and opens no overlay; the agent's editor opens its
file as a tab with the focus; saving keeps the tab open and the agent waiting; closing the tab
hands the saved text back to the agent as its draft, not sent, and puts the focus on the agent's
terminal. The bar's button does what Ctrl-G does. An agent run by hand in a New Terminal, or any
agent with the switch off, gets the overlay as before. `marley-edit` run where it cannot reach
Marley says why and exits 1.
