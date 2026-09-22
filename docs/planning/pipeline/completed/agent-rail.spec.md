---
pipeline_id: dc2134b2-4ad6-4722-b7ae-0620734482e8
ticket: forge#167 (409beb81-022d-428d-b68e-7ccda8886411) · local docs/planning/tickets/open/TICKET-167-agent-rail.md
aar_id: 96ea54f1-f7e3-4eca-8705-82c342780fef
status: Phase 5 — Complete PASS
title: M10 — rail agent icons + globally-unique PaneIds (absorbs #158)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/workspace.rs (PURE: PaneGrid::new_with_base + PANE_ID_BLOCK)
  - crates/marley_app/src/agent_view.rs (PURE: fleet_status_for)
  - crates/marley_app/src/app.rs (SHIM: the block counter, safe close-cleanup, the rail glyph)
---

## Title
Rail tab rows show their agents' status glyph — and the cross-tab PaneId aliasing (#158) dies at the source:
every grid mints ids from its own 2³²-sized block, so a PaneId belongs to exactly one pane, ever.

## Scope
### In
- PURE `workspace.rs`: `PANE_ID_BLOCK = 1 << 32`; `PaneGrid::new_with_base(session, base)` (`new()` =
  base 0 — every existing test untouched).
- PURE `agent_view.rs`: `fleet_status_for(pane_ids, agents) -> Option<AgentStatus>` — the tab's aggregate
  (Working > Waiting > Idle > Exited).
- SHIM: a `pane_blocks: u64` counter (`next_pane_block()` = `{blocks += 1; blocks << 32}`); non-boot grids
  (⌘T / new project) mint from a fresh block; tab/project close removes the closed grids' ids from
  agents/remotes (+ a targeted `last_agent` clear) — replacing the #161/#162 blanket clears; the rail Tab
  arm renders the `agent_status_glyph` before the label when `fleet_status_for` is Some (accent while
  Working, muted otherwise).

### Out
- Persisting agent runs; per-PANE rail glyphs (tab-level is the sessions-list parity #152 deferred);
  Fleet-view changes (it already iterates the map, which unique ids make correct).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — id-block uniqueness over per-pane storage: 26 map sites + the `&HashMap` helpers stay byte-identical;
  the aliasing class is structurally dead; close-cleanup by id becomes exact.
- D2 — the boot grid keeps base 0 (deterministic tests + persistence unchanged — PaneIds are never persisted).
- D3 — aggregate priority: any Working → Working; else Waiting; else Idle; else Exited.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `new_with_base(s, B)` shall mint its first pane as `PaneId(B)` and splits as `B+1…`; `new()` shall stay base 0. | unit |
| REQ-002 | `fleet_status_for` shall return None for agent-less ids and the highest-priority status otherwise (Working > Waiting > Idle > Exited), mutation-killed on the priority. | unit |
| REQ-003 (visual) | WHEN an agent runs in a tab, that tab's rail row shall show the status glyph; a plain tab shall show none; the glyph shall survive opening a second tab (the cross-tab case #158 broke). | driven capture |
| REQ-004 | Closing a tab/project shall remove exactly its grids' ids from agents/remotes. | critic/code |
| REQ-005 | gate GREEN; the pure fns at cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (the uniqueness argument end-to-end: every grid-creation site
covered; the retain-cleanup exactness; the glyph lookup). P4 tests + driven + gate. P5 docs + close #158 too.
