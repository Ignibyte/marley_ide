# B2: The agent sees and drives the browser — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-492-browser-tools-for-agents.md
- **Pipeline spec:** 492-browser-tools-for-agents.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; Chad's "first class access to what the user sees … cursor
  like experience".
- **Classification:** feature; `marley_browser` (the rings, the snapshot, the redaction),
  `marley_mcp` (the browser family), `marley_workbench` (answering calls, the chip). No Zed
  path.
- **Recall (§18.3):**
  - orchestration-shell.md §8 and §10: "don't scrape what you can query"; write tools behind
    grants (D2 here changes the default for this family, with its reasons).
  - The handoff's token measurements (agent-browser's interactive snapshot about 3,400 tokens;
    Playwright MCP about 11,900 plus a 4,600-token schema).
  - The probe: the main frame's AX tree leaves cross-site iframes out.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
