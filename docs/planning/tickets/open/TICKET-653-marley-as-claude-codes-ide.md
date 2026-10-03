# TICKET-653 — Marley as Claude Code's IDE: the link, diagnostics, selection and open file

- **Ticket:** LOCAL #653 (feature, prong 2: Claude Code on its own tools, design note B3; with
  #549's send selection)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/653-marley-as-claude-codes-ide.spec.md
- **Source ticket:** `docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`
  (B3, and B7 for the version checks); Chad, 2026-10-02: "we need to brain storm integration into
  claude and codex using their tools instead of fighting them", and on B3: "Yes, with version
  checks"
- **Status:** open

## Summary
Claude Code connects to an editor that serves its IDE MCP server: a loopback WebSocket server
whose port and token sit in a lock file in `~/.claude/ide/` (or `$CLAUDE_CONFIG_DIR/ide/`). With
`marley.claude_code_ide` on, Marley serves one for each open project, writes its lock file, and
puts its port in the environment of that project's new terminals, so `claude` started in a Marley
terminal connects to its own project's Marley and no other. Claude Code then gets the editor's
selection and open file with each prompt, reads Zed's language-server diagnostics through
`mcp__ide__getDiagnostics`, and send selection's reference goes in as an IDE mention instead of a
typed `@path#La-b`, which stays as the fallback. The docs name the transport, the lock file's place
and mode, the token's header and the diagnostics tool; the lock file's fields, the port variable,
the selection notifications and tools, and the mention are undocumented. Each of those registers
with #648's table of tested Claude Code versions and stays off outside it, with the reason shown.
Off by default. Diffs (`openDiff`) come in a later slice.

## Acceptance
With nothing set, Marley writes no lock file and adds no variable to a terminal. Turned on, a
Claude Code in a new terminal of the project finds the lock file, connects with its token (a wrong
one is refused), lists the diagnostics tool and the selection tools, and receives the selection,
the open file and, on send selection, a mention for its own terminal only; a Claude Code with no
known link gets the typed reference as before. A Claude Code version outside #648's table gets no
link, or no selection and mention, and Marley says why. Stale lock files of Marley's own are
removed; turning the switch off removes the live one.
