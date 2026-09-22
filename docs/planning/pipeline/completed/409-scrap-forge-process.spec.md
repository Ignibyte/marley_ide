---
pipeline_id: 56d37ec0-50d2-4cd0-9d77-1909a312a60d
ticket: LOCAL-ONLY · docs/planning/tickets/open/TICKET-409-scrap-forge-process.md
aar_id: 6d71fd90-b213-4bf1-817d-8593b285ce8e
status: Phase 5 — Complete PASS
title: Scrap forge from the pipeline — local tickets + local knowledge ledger + service off
type: chore
milestone: M30 (the pivot)
references:
  - docs/planning/intake/scrap-forge-pivot.md (the owner decision)
  - docs/planning/design-notes/ide-mvp-shelf.md (the local pre-authored-spec method)
---

## Title
Retire the forge sidecar from Marley's dev pipeline (owner decision 2026-08-09,
recorded in the intake doc): tickets and knowledge become purely local files, the
next `/work` picks work from a local backlog, and the forge services on this machine
stop running. No `.rs` changes — the product-layer rip is TICKET-410/411.

## Scope
### In
1. **Knowledge ledger export** — the Marley-scoped forge rows into
   `docs/planning/knowledge/`: `prevention-rules.md` (312), `failures.md` (205),
   `lessons.md` (201 distilled), `architecture-decisions.md` (43). One block per
   entry (code + body + metadata), greppable; counts asserted against the DB at
   export time and recorded in the notes (REQ-002 baseline: 312/205/201/43).
2. **Open-ticket export** — all 16 open Marley forge tickets (#6, 223, 225, 226,
   227, 271, 318, 319, 320, 332, 333, 353, 354, 365, 366, 407) → faithful local
   docs `docs/planning/tickets/open/TICKET-<n>-<slug>.md` (title, full description,
   type, tags, created date, forge-era id recorded as provenance) + an ordered
   `docs/planning/tickets/BACKLOG.md` index — the local ticket-next. Ordering
   principle (design pins the exact list): interactive-sized bugs first, then debt,
   then chores/features; overnight-shaped work (#407 FULL audit) flagged as such;
   blocked (#6, needs a Windows runner) last.
3. **CONSTITUTION amendment** (ISOLATED commit per the amendment rule — this file +
   the hooks that enforce the changed sections): §18.3 recall becomes local (grep
   the ledger + completed notes); §19 becomes the Local Knowledge & Tickets process
   (ledger append at complete, local ticket docs + BACKLOG as the tracker); §3's
   "canonical link to the forge ticket" line → the local doc IS canonical; the §19
   unwired-hooks ratchet paragraph goes with it. Loosening reason recorded in the
   section itself, per the amendment rule.
4. **Skills de-forge** (9 files in `.claude/commands/`): `work` (pre-flight drops
   the forge check; step 3 = read BACKLOG + grep ledger; "next" = top non-blocked
   BACKLOG row), `pipeline/plan` (always LOCAL-ONLY; numbering = 1 + max across
   open+closed; recall = grep; no aar-open), `pipeline/design` + `implement`
   (recall = grep), `pipeline/inspect` (capture = append to failures/prevention
   ledger), `pipeline/complete` (ledger append + local close + BACKLOG upkeep; no
   aar-submit/ticket-close), `spec` (queues local tickets + BACKLOG rows),
   `commit` + `lib-hook-helpers.sh` (scrub references).
5. **Hook retirement** — delete the §19 trio `enforce-mcp-config.sh`,
   `enforce-docs-before-code.sh`, `enforce-completion.sh` (§19 records them as not
   wired as hard blockers; verify unwired in `.claude/settings.json` before
   deleting); scrub forge references from `lib-hook-helpers.sh` and settings.
6. **Config** — remove the forge server from `.mcp.json` (local, gitignored) and
   `.mcp.json.example` (committed).
7. **Service off (machine ops, at Phase 5 — after every export is verified):** stop
   the forge-mcp/forge-sched processes; `launchctl bootout` + disable the two
   LaunchAgents (plists renamed `.disabled` — reversible); verify 127.0.0.1:8080
   refuses. Postgres/redis stay (shared brew services); the forge DB is kept intact
   untouched (druplit/Rusty/PCB/monorpgmaker/Oathstar live there).
8. **Assistant memory** — post-commit, the forge-ops memory is rewritten to the
   post-pivot fact set (forge off; don't re-wire; DB dormant for other projects).

### Out (explicitly deferred / never)
- ALL `.rs` — the product surface (Forge URL pane, `forge_web_base`,
  `marley_forge_client`, fleet/brain wiring) is TICKET-410/411, authored from the
  in-flight seam map. Architecture docs describing that product surface move with
  those tickets (§21 there, not here).
- History scrub — completed pipeline docs, closed tickets, CHANGELOG entries, and
  design-note archives keep every forge reference; the record is immutable.
- The forge DB and the other projects' use of it; postgres/redis services.

## Reference (§20)
N/A — Marley-specific, no reference-app analog: this changes Marley's OWN
dev-process tooling (tracker + knowledge recall), not any user-observable product
behavior Warp or Zed could be a reference for. No behavior map consulted; the wall
is untouched (no product code in the diff).

### Prior art
- **In-repo (decisive):** the pre-authored-spec SHELF method
  (`design-notes/ide-mvp-shelf.md`, `integrity-five-shelf.md` — runs 6–7, 15
  tickets shipped from local spec files with forge as mirror only) proves the
  local-files queue at production scale; `docs/planning/tickets/{open,closed}` +
  frontmatter is the already-canonical local ticket shape; the completed-pipeline
  notes archive already carries every inspect ledger and lesson (recall-by-grep has
  a corpus). **TICKET-006 is the process precedent**: an automated system
  (spec-provenance gate-16, `marley_spec_provenance`) removed by owner decision
  with the reason recorded in §20 — de-automation with a recorded reason is an
  established move in this repo. The gate-is-test discipline (§7/§15) covers
  no-`.rs` verification.
- **Published:** none needed — no external protocol/spec governs a local ticket
  workflow.
- **Permissive deps:** N/A — no crate we ship owns a dev-process seam ("none:
  checked the dep surface conceptually; a Rust crate cannot own this").

## React-first (parity)
N/A — no UI delta: zero `.rs` in the diff; nothing the user sees changes. The
product-surface tickets (410/411) will carry their own React-first sections.

## Locked-In Decisions
- D1 — **History is immutable.** The pivot edits law + live surfaces only; every
  archived forge id/reference stays. "As if it never existed" applies to the LIVE
  process going forward, not the record.
- D2 — **The DB outlives the service.** Nothing deletes forge data; the service
  stops. Reversal (another project restarts it) must not be foreclosed.
- D3 — **Numbering continues** — local counter = 1 + max ticket number across
  open+closed; no renumbering of exported tickets (#407 stays #407).
- D4 — **The amendment is its own commit** (CONSTITUTION + enforcing hooks only),
  before the mechanical de-forge commit — two commits, per the amendment rule.
- D5 — **Exports precede the off-switch.** The service/DB is read for the ledger
  and ticket exports and verified (counts) before anything stops.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the pivot lands, the live process surfaces (`.claude/**`, `docs/planning/_templates/**`, `docs/planning/pipeline/_templates/**`, `CONSTITUTION.md`, `.mcp.json.example`) shall contain zero forge-the-system references — the English words forged/forging are filtered, and the documented exception SET is: the CONSTITUTION preamble's two PRODUCT mentions (the thesis "+ Forge + ops" layer and "Forge MCP client", both marked for TICKET-410/411 by the adjoining parenthetical) and §19's amendment record (the in-section loosening reason + intake pointer the amendment rule mandates) | the S1 pinned grep (design) → exactly the documented exception, nothing else (negative smoke in notes) |
| REQ-002 | The knowledge ledger shall carry every Marley-scoped row: 312 prevention rules, 205 failures, 201 distilled lessons, 43 architecture decisions | block-count per ledger file == the DB count captured at plan (script output in notes) |
| REQ-003 | Every open Marley forge ticket (16) shall exist as a local open ticket doc AND a BACKLOG.md row | count + number-set match, listed in notes |
| REQ-004 | WHEN `/work next` runs post-pivot, the work skill shall resolve the next item from the top non-blocked BACKLOG row without any forge call | skill text asserts it (static); behavioral proof = the first post-pivot `/work` (410/411 authoring rides it) |
| REQ-005 | WHEN the ticket completes, no forge process shall be running and no LaunchAgent shall retry it | `curl 127.0.0.1:8080` → connection refused; `launchctl list` lacks both labels; recorded in notes |
| REQ-006 | The historical record shall be untouched | `git status`/diff shows no modification under `pipeline/completed/`, `tickets/closed/`, or existing CHANGELOG entries |
| REQ-007 | The two-commit shape shall hold: an amendment commit touching only CONSTITUTION.md + the enforcing-hook surface (the three deletions, the `lib-hook-helpers.sh` arming edit, and the `settings.json` comment that cited the removed hooks), then the mechanical commit | `git log --stat` of both commits, pasted in notes |

## Phase Plan
- **P2 Design** — the ledger block format; the BACKLOG ordering (exact list); the
  export script shape (psql → md, deterministic); the §19 replacement text; the
  per-skill edit list; settings.json verification for the hook trio.
- **P3 Implement** — export ledger + tickets; amend CONSTITUTION; edit skills,
  hooks, templates, configs. No `.rs`.
- **P3.5 Inspect** — critics over the diff: fidelity of exports (spot-check rows),
  REQ-001 grep truth, amendment isolation, no history touched.
- **P4 Validate** — gate-is-test: `scripts/gates.sh --fast` green (static set incl.
  shellcheck on edited hooks) + the negative smokes (REQ-001/002/003/006).
- **P5 Complete** — CHANGELOG + (process-doc) architecture touch-ups, the FINAL
  forge capture (aar-submit under the old law, closing the loop), local close +
  archive, then the service off-switch (REQ-005) and the memory rewrite.
