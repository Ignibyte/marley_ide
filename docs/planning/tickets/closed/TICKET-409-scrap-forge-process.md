# TICKET-409 — Scrap forge from the pipeline: local tickets + local knowledge ledger

- **Forge ticket:** LOCAL-ONLY (this ticket retires that field — deliberately never minted in forge)
- **Owner:** pipeline 56d37ec0-50d2-4cd0-9d77-1909a312a60d
- **AAR:** (the final forge AAR — recorded in the notes; the mechanism it closes dies with this ticket)
- **Pipeline doc:** ../../pipeline/active/409-scrap-forge-process.spec.md
- **Source:** docs/planning/intake/scrap-forge-pivot.md (owner decision, 2026-08-09)
- **Status:** closed

## Summary
Chad's pivot: forge is scrapped for Marley — work goes directly ticket-based, local
files only. This ticket de-forges the PROCESS layer end to end: exports the 761
Marley-scoped knowledge rows (312 prevention rules, 205 failures, 201 distilled
lessons, 43 architecture decisions) into a greppable `docs/planning/knowledge/`
ledger; brings all 16 open Marley forge tickets down as local ticket docs plus an
ordered `BACKLOG.md` the next `/work` reads; amends the CONSTITUTION (§18.3 recall,
§19 replaced by the local process, the §3 frontmatter rule) in an isolated
amendment commit; de-forges the 9 pipeline skills, retires the §19 hook trio,
cleans the 3 doc templates and `.mcp.json(.example)`; and turns the forge services
OFF on this machine (processes stopped, LaunchAgents disabled — the DB is kept
intact untouched for the other projects it tracks). History (completed pipelines,
closed tickets, CHANGELOG) is immutable — the pivot changes the law and the live
surfaces, never the record. The product-layer rip (Forge pane, forge_client, fleet
brain seams) is TICKET-410/411, authored from the live seam map.

## Acceptance
`/work` on a clean session resolves the next item purely from local files (BACKLOG →
open ticket docs), recall greps the local ledger, no live process surface mentions
forge, and nothing answers on 127.0.0.1:8080. Full EARS in the pipeline spec.
