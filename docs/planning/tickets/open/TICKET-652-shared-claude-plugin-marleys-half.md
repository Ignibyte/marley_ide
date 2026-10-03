# TICKET-652 — Agent reports reach Marley through `$MARLEY_BIN report`, and the rail and resume read them first

- **Ticket:** LOCAL #652 (feature, prong 2 C1, a terminal's own seat; design note B2, Marley's half)
- **Owner:** claude-opus-5-5, 2026-10-03
- **Pipeline doc:** ../../pipeline/queued/652-shared-claude-plugin-marleys-half.spec.md
- **Source ticket:** `design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md` (B2, Chad's
  "Yes, with the harness"); rustal-harness `docs/planning/MARLEY_REQUESTS.md` MREQ-009 and its
  D169 (one plugin, two hosts)
- **Status:** open

## Summary
Marley and rustal-harness share one Claude Code plugin whose mod reports a session's state to the
program that started it; the harness builds it and asks Marley (MREQ-009) which variable tells
the mod it runs under Marley and how the mod reaches Marley. This ticket is Marley's answer and
its state side. Every local terminal names `MARLEY_BIN` beside `MARLEY_TERMINAL_ID`: a small
Python program Marley writes into its data directory, as it writes its URL opener, whose
`report` and `release` take exactly the arguments `rh report` and `rh release` take (the
harness's TICKET-099 contract) and hand them to Marley over a 0600 Unix socket. Marley knows the
reporter by its processes, holds one authority per terminal, and lets the reports set the
terminal's state on the rail and the session a restart resumes, with #519's hook frames kept as
the fallback and as the source of the row's details. Loading the shared plugin into Marley's
terminals (through `CLAUDE_CODE_PLUGIN_DIRS`, behind a setting off by default, with #648's
version row) is decided here and built in the next slice, because it waits on the harness adding
Marley's host to the mod and naming a license for its files. Prompts in, approvals, and retiring
the hook frames are later slices.

## Acceptance
Local terminals carry `MARLEY_BIN` (empty in tasks and remote terminals); `"$MARLEY_BIN" report`
and `release` are taken for the terminal the reporter runs in, with TICKET-099's fields, rules and
refusal names, and a stale `seq`, another process, an unknown caller or a missing Marley refused;
while a terminal holds an authority its row shows the reported state with the frames' details and
a frame's permission wait over a reported `working`; without one, the frames move the row as
before; a restart resumes the reported session; a release ends the seat within one second.
Twenty-one EARS criteria in the spec; the reply to MREQ-009 is in the notes.
