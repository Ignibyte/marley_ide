# B5: Record what already happened in the Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-499-flight-recorder.md
- **Pipeline spec:** 499-flight-recorder.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar C (`browser-handoff.md`).
- **Classification:** feature; `marley_browser` (the recorder's rings and the recording's
  format, pure), `marley_workbench` (feeding it from the input path and the frames, Record this,
  the tools), `marley_mcp` (two rows). No Zed path expected.
- **Recall (§18.3):** #492's rings and `redact_url`; `L-claude-492-chromium-shows-a-password-by-length-001`
  (a snapshot writes no values); `F-claude-489-the-browser-decoded-frames-unoptimized-001` (the
  frames' cost in the debug build: the recorder keeps the JPEG bytes and decodes nothing).
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
