# TICKET-632 — The embedded harness: Marley starts the harness's runtime itself

- **Ticket:** LOCAL #632 (feature, prong 2 C1, plan D19)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** ../../pipeline/completed/632-the-embedded-harness.spec.md
- **Source ticket:** plan D19 ("Packaging `rh` with Marley and starting its runtime is its own ticket after #534"); wave 4 of `design-notes/remaining-work-2026-09-30.md`
- **Status:** closed

## Summary
#534 follows a harness whose MCP server `marley.harness` names. This ticket lets Marley run the
harness itself: with `marley.embedded_harness` on, Marley finds `rh` (`MARLEY_RH`, else the search
path), starts `rh --state <data dir>/harness serve` as a process of its own, and follows it through
`rh --state <root> mcp` as #534 does. The runtime's own state shows in the section's header: `rh
mcp` reads the journal even while `serve` is down, so the header follows the child too. `rh` is not
shipped inside Marley yet: the harness has no install path and no license.

## Acceptance
With `rh` found and the setting on, Marley starts the runtime on its own root, the Harness section
shows it connected and lists a session started on that root; when the runtime dies the header says
so and Marley starts it again, the session still there; with `marley.harness` set, Marley starts no
runtime; with `rh` missing, the header says where Marley looked.
