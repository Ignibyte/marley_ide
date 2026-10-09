# TICKET-723 — Docs search over all of Marley's docs

- **Ticket:** LOCAL #723 (feature, Marley's MCP server)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [723-docs-search-over-all-of-marleys-docs.spec.md](../../pipeline/completed/723-docs-search-over-all-of-marleys-docs.spec.md)
- **Source ticket:** Chad, 2026-10-09: "we should probably have a document-search on the marley mcp
  in general marley can use that includes zed and marley documentaton".
- **Status:** closed

## Summary
`docs_search` and `docs_read` (#681) cover Zed's docs and Marley's guide. They gain:
- Marley's walkthrough;
- Marley's changelog, each entry a section of its own, so "what changed" questions find the
  ticket that changed it;
- word forms: a query word also matches its stem, so "terminals" finds "terminal".

The tools' descriptions, the Marley agent's instructions and the guide say so.

## Acceptance
`docs_search` finds a changelog entry and a walkthrough section, and a plural query finds a
singular heading. `docs_read` reads `marley/CHANGELOG.md` by an entry's heading.
