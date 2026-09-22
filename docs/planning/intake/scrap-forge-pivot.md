---
status: promoted
created: 2026-08-09
ticket: TICKET-409 (process), TICKET-410/411 (product — authored from the seam map)
pipeline_spec: docs/planning/pipeline/completed/409-scrap-forge-process.spec.md (done);
  docs/planning/pipeline/completed/410-browser-forge-identity-rip.spec.md (done, def60cf);
  docs/planning/pipeline/completed/411-forge-client-fleet-brain-rip.spec.md (done —
  THE PIVOT IS COMPLETE: process + product both landed 2026-08-09)
---

# Scrap forge — the pivot to purely local ticket-based work

## The decision (Chad, 2026-08-09)

Forge — the MCP knowledge/ticket sidecar the pipeline dogfoods (§19) AND the product
surface Marley grew toward (the Forge operator cockpit) — is **scrapped for Marley,
entirely**: process and product. Work goes **directly ticket-based**: local ticket
docs, local queued specs, a local greppable knowledge ledger. Scope calls made
explicitly:

- **Scope: everything now** — the pipeline/process layer AND the product surface
  (the Forge URL pane, `forge_web_base`, `marley_forge_client`, the fleet/brain
  endpoint wiring). The product rip un-ships most of M29's forge-facing intent while
  keeping the generic embedded-browser infrastructure question open to design.
- **Service fate: Marley unplugs; the service stays.** The forge instance on this
  machine also tracks druplit/Rusty/PCB Bros/monorpgmaker/Oathstar — Marley removes
  its own wiring only. (Operationally the service is fragile here anyway: TCC-broken
  LaunchAgents, a stale-lockfile postgres after reboots, and a start-all.sh that
  builds the tool-less OSS binary — recorded 2026-08-09.)
- **Knowledge: exported before unplugging.** Marley-scoped prevention rules,
  failures, lessons, and architecture decisions dump into `docs/planning/knowledge/`
  — the recall step becomes grep, not RPC.

## Why (recorded, not relitigated)

The forge loop's real value was recall + capture, and both survive locally: the
completed-pipeline notes archive already carries every inspect ledger and lesson;
the exported ledger makes the distilled rules greppable. What forge added on top —
semantic search, cross-project promotion, sprint bookkeeping over MCP — cost a
running service + DB + bearer plumbing that went down with every reboot and went
stale whenever it was down (the #405/#406 statuses drifted within a day). The
local shelves (IDE-MVP, integrity-five) already proved batch work runs fine on
files alone. `ticket-next` was FIFO — the service never actually chose work.

## What this does NOT do

- **History is immutable.** Completed pipeline docs, closed tickets, the CHANGELOG,
  and architecture-doc history keep their forge ids and forge-era language — the
  pivot changes the LAW and the LIVE surfaces, never the record.
- The forge service/DB on this machine is untouched (other projects' tracker).
- Ticket NUMBERING continues the same sequence (local counter = max existing + 1),
  so #409 follows #408 with no renumber.

## The ticket train

1. **TICKET-409 — de-forge the process layer + export the knowledge ledger**
   (docs/tooling, no `.rs`): CONSTITUTION §18.3/§19 amendment (isolated commit per
   the amendment rule), the 9 pipeline skill files, the §19 hook trio, the 3
   templates, `.mcp.json(.example)`, `docs/planning/knowledge/` export. Runs FIRST so
   later tickets run under the new local-only law.
2. **TICKET-410/411 — the product rip** (authored from the live seam map, in
   flight): the browser's forge identity vs its generic infrastructure; the
   `marley_forge_client`/fleet/brain seams. The fleet's fate (a non-forge brain
   endpoint vs removal) is a named fork to decide at authoring.
