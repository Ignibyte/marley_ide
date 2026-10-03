# TICKET-651 — Codex's approvals answered from the inbox

- **Ticket:** LOCAL #651 (feature, prong 2: B1 part 2, Codex's approvals, on #650)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/651-codex-approvals-and-prompts.spec.md
- **Source ticket:** `docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`
  (idea B1); Chad, 2026-10-02: "we need to brain storm integration into claude and codex using
  their tools instead of fighting them", and for B1 "Yes"
- **Status:** open

## Summary
Once #650 runs Codex's TUI as a client of Marley's own App Server, Marley sees what Codex's server
asks its clients to approve: a command, a file change, extra permissions, an MCP server's
elicitation. The server sends each request to every client on the thread and takes the first
answer. This ticket lists those requests in the rail's inbox as Codex entries and answers them in
place with the decisions each request offers (allow, allow for the session, deny, deny and stop
the turn), sending the answer only to the request the user saw, by its id on the connection it
came from. Codex's TUI closes its own prompt when Marley answers; an entry leaves when the server
says the request is resolved, whoever answered. Marley never answers a request the user did not
pick an answer for, and never with an error, which the server would take as the answer. A wait
the inbox cannot answer, such as Codex's question, keeps #650's entry and is left to the TUI. The
slug keeps the brainstorm's name: sending prompts as `turn/start` and `turn/steer`, and resuming
a thread after a restart, are the next two slices.

## Acceptance
A Codex command, file change, permissions request and elicitation each show in the inbox with
what they would do and the decisions they offer; a click answers that request with that decision
and the entry leaves when the server resolves it; one answered in the terminal first leaves with
nothing sent from Marley; a question keeps #650's entry and gets no answer from Marley; and the
entries leave when the server goes away.
