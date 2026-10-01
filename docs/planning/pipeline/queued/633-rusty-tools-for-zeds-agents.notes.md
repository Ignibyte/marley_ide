# Rusty's tools for Zed's agents — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-633-rusty-tools-for-zeds-agents.md
- **Pipeline spec:** 633-rusty-tools-for-zeds-agents.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** Chad, 2026-09-30 (wave 4); plan C2 and D11.
- **Recall (§18.3):**
  - An Explore read of Rusty (2026-10-01): rusty-mcp has no authentication; stdio is
    `rusty-mcp` with no arguments (Rusty's `.mcp.json`), HTTP is `127.0.0.1:4174/mcp`; the brain
    loop is `brain_ask`, `brain_decide`, `brain_no_decision`, `brain_follow_up`, `brain_due`;
    rusty-mcp exposes no agent-session tools; Rusty's constitution: "The back end is MCP only."
  - #501 adds `marley` to Zed's defaults with `update_default_settings`; a user's own entry wins.
  - Privacy: nothing of the user's Rusty (its data paths, its tokens, its pages) goes into Marley's
    public docs or fixtures; the scenario's server is a stand-in.
- **Risk to settle at promotion:** how the scenario puts a stand-in `rusty-mcp` on Marley's own
  search path (the harness may pass Marley's environment, or Marley reads the login shell's PATH).
