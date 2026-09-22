---
pipeline_id: 6b095477-1dad-4fff-9311-30ebe3261fbe
ticket: forge#391 (41161e92-e28d-46b9-a938-750e2d83e1bc) · local docs/planning/tickets/open/TICKET-391-no-terminal-totality.md
aar_id: 18a91168-f535-4d32-9585-4897457e0d65
status: Phase 5 — Complete PASS
title: No-terminal totality — decouple workspace() from the ≥1-terminal invariant (enabler audit)
type: chore
milestone: M27
references:
  - docs/planning/pipeline/completed/387-section-actions.spec.md (the guard truth table this enables relaxing)
  - docs/marley_architecture/app_shell.md (the M10 never-empties record)
---

## Title
The **behavior-neutral enabler** for "terminate the last terminal": make every app-layer site that
assumes a project holds ≥1 TERMINAL tab **total** over a no-terminal (and zero-tab) project. The M10
#161/#159 invariant ("the app's `workspace()` accessor requires ≥1 terminal tab") is what
`TabError::LastTerminal` protects — the #202 parking record counted **~90 dependent sites**. This
ticket dissolves the ASSUMPTION; #392 then dissolves the GUARD. The guards stay ON here: zero behavior
change, provable by the unchanged suite.

## Scope
**Design-phase D4 SPLIT (2026-07-23):** the Explore fan-out found **~40 reachable sites** (always-run
render/pump/persist + ~30 user-triggered handlers + 6 product questions) — more than one clean slice. Per
the drafted D4, **#391 is scoped to the ALWAYS-RUN render/pump/persist totality** (the app can DISPLAY +
PERSIST a no-terminal project); the **user-handler totality + the 6 product questions move to #392** (which
IS "make the empty workspace work" — the handler behavior is its natural scope). The full audit ledger
lives in this ticket's `.notes.md` (the REQ-004 deliverable; #392 starts from it).

### In (the always-run slice)
- **The `try_workspace`/`try_workspace_mut() -> Option<&(mut) PaneGrid>` contract** (D-c; an empty-grid
  sentinel is rejected — a 0-pane grid orphans `focused`). Total twins of `workspace()`/`workspace_mut()`,
  `None` when `terminal_grid_index().is_none()`. The panicking accessors stay for the terminal-GATED sites.
- **The 5 always-run site-groups made total:** `render` body's `focused` (14865 → `Option`), the resync
  loop (5529), the PTY-resize loop (16801), `cockpit_body`'s Details inspector (5332/5338 → an empty state
  when no focused terminal), and `persist_grid`'s legacy single-grid block (4868-4899 → a second guard
  mirroring the #234 `count==0` arm, clearing the legacy grid key).
- **Tests:** a no-terminal `Project` (a cockpit-only project; `terminal_grid_index()==None` already tested)
  drives the fixed seams; the persist guard + the pure `try_workspace` predicate at cov/MSI 100; the FULL
  suite byte-identical (guards on). The audit LEDGER in the notes.
- **Codec honesty:** confirm a zero-`T=` project line parses (still gated by the boot guards — #392 flips
  reachability).

### Out (explicitly deferred)
- **The user-handler totality → #392** (splits/creates/focus-nav/agents/remotes/overlays/the raw-key
  router + the 6 product questions — the full list is in this ticket's notes ledger).
- Removing `LastTab`/`LastTerminal` (that is #392 — this ticket keeps refusals byte-identical).
- The empty-workspace UI (the #392 empty-center state).
- Any user-visible UX change — this is a pure internal-contract chore (guards stay on).

## Reference (§20)
**N/A — Marley-specific.** An internal totality refactor of Marley's own accessor contracts; no
user-visible behavior, no reference app to match. No copyleft source read.

### Prior art
1. **In-repo owners:** the #387 `close_tab_refusal` truth table (the guard being enabled-for-relax);
   `terminal_grid_index` + the `cockpit_only` test (the pure layer ALREADY models no-terminal);
   the #234/#247 zero-PROJECT totality work (persist_grid's `project_count()==0` guard + the launcher)
   — the same class of audit one level up; #205's cwd-restore codec tiers (the restore-shape
   precedent). The #202 parking record (the ~90-site inventory rationale).
2. **Behavior maps** — N/A (internal).
3. **Published material** — N/A.
4. **Permissive deps** — "none: this is Marley's own accessor contract; checked nothing external —
   no owner exists outside the repo."

## Locked-In Decisions
- **D1 — Behavior-neutral:** the guards still refuse; the suite proves byte-identical outcomes. The
  hypothetical states are reached only by tests.
- **D2 — Totality by type, not by convention:** prefer `Option`-returning contracts over "callers
  promise to check" — a site that can't get a terminal must be FORCED to handle it.
- **D3 — The ledger is a deliverable:** every touched site listed in notes; inspect critics verify the
  enumeration (a missed site = #392 panics later — this ticket carries the risk so #392 doesn't).
- **D4 — Risk-carrier sizing:** if the audit balloons past one slice, split by subsystem (pump/persist/
  render) into sequential tickets — surfaced at design, not silently absorbed.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Every audited site shall be total over a no-terminal project (no panic path via unwrap/expect/index on terminal existence). | The audit ledger + clippy/gate + targeted units constructing no-terminal projects. |
| REQ-002 | With the guards still in place, observable behavior shall be byte-identical (the full suite passes unchanged; refusals unchanged). | `cargo nextest` full suite; the #387 guard tests still green untouched. |
| REQ-003 | A zero-`T=` project line shall parse in the shell codec (restore reachable only in tests until #392). | Codec unit. |
| REQ-004 | The notes shall carry the complete audit ledger (site → treatment), verified by inspect. | Doc review at inspect. |

## Phase Plan
- **P2 Design** — Explore fan-out enumerates the real site list; decide the `workspace()` no-terminal
  contract (Option vs sentinel); slice-splitting call (D4).
- **P3 Implement** — the totality refactor per the ledger.
- **P3.5 Inspect** — critics attack the ENUMERATION (grep-sweep for missed `workspace()`/index sites)
  + the contract choice's ergonomics.
- **P4 Validate** — no-terminal/zero-tab constructor units per seam; full suite green (REQ-002);
  `--diff` gate.
- **P5 Complete** — CHANGELOG (internal chore entry), app_shell.md invariant note updated, archive.
