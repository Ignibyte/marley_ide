---
pipeline_id: 5b7a8b2e-2d65-44b6-8430-5a064feb9489
ticket: forge#394 (adf0b9d6-08e1-43c6-8c7c-26c48670c2a5) · local docs/planning/tickets/open/TICKET-394-content-registry.md
aar_id: 9dc2d03c-82d0-4c65-8a04-1d66055cf754
status: Phase 5 — Complete PASS
title: The ContentId registry — the pure refcounted lifecycle (M27 spine, slice-1; terminals migrate in #396)
type: feature
milestone: M27
references:
  - docs/marley_architecture/pane-composition-model.md (#388 — THE decision-complete source; D5 says start P2 from it alone)
  - docs/marley_architecture/app_shell.md (the monolithic-RootView + side-table pattern record)
---

## Title
Train slice 1 of the #388 pane-composition model: the **pure, gpui-free, generic
`ContentRegistry<C>`** — a Marley-owned refcounted registry (NOT gpui `Entity`;
AD-claude-pane-content-id-registry-001) with an explicit **per-id view-count** and
**drop-on-last-close** (release the last view → return the owned content for teardown). This is the
content-agnostic **lifecycle SPINE**; nothing consumes it yet (the #371 `orchestration_for`
wired-but-unused precedent). **The D5 SPLIT (Opus design 2026-07-22): the terminal-OWNERSHIP
migration — the L-sized ripple (Content enum + ~46 accessor sites + 7 close-path reaper-moves + the
persistence rebuild + the agents/remotes re-key) — moved to #396** (`cd706b38`), which loads terminals
onto this spine. #394 ships ONLY the proven lifecycle mechanism.

## Scope
### In
- **The pure generic registry (`content_registry.rs`, gpui-free):** a `ContentId` newtype (monotonic id
  allocation) + `ContentRegistry<C>` with `insert(C) -> ContentId` (acquires 1 initial view),
  `acquire_view(id)` (increment; `None` if unknown), `release_view(id) -> Option<C>` (decrement; returns
  `Some(content)` EXACTLY on the last-view drop so the caller reaps it — the drop-on-last-close mechanism),
  `get`/`get_mut`/`view_count`/`len`. The **explicit view-count is the BINDING catch**
  (PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001: a bare `HashMap` does NOT refcount).
  Generic over `C` so the lifecycle is proven purely with a test double (no live PTY needed) — the #388
  open-Q2 genericity simplification.
- **Gate-safety (the #371 template):** `mod content_registry;` (private in `lib.rs`) + `pub use
  content_registry::{ContentId, ContentRegistry};` at the crate root → lib-API-reachable, so the
  wired-but-unused registry is NOT dead-code under `-D warnings` (git-proven by `orchestration_for`,
  #371→#376). Every pub item carries a `///` doc (the `#![deny(missing_docs)]` constraint).
- **Zero production change:** the registry is additive + dormant; the full suite is byte-identical.

### Out (explicitly deferred → #396, the slice-2 capstone; + later train slices)
- **The concrete `Content` enum** (6 kinds; Terminal wired) → **#396** (it's constructed when terminals
  actually migrate; declaring a 6-variant enum with live payloads that nothing constructs, just for form,
  is deferred — REQ-005 moves to #396).
- **Terminal ownership migration + the reaper-move** (`PaneContent`→`ContentId`, the ~46 accessor sites,
  the 7 spawn-drop close paths preserving the off-thread reap) → **#396**.
- **The persistence rebuild** (restore rebuilds registry entries; ids never serialize) → **#396**.
- **The agents/remotes/notify_ticks re-key** (`PaneId`→`ContentId`) → **#396**.
- Editor/Cockpit/FileTree/Git/Browser residents (slices 3+); add-to-pane, one-instance-many-views,
  nameable arrangements, the global Panes section (slices 5+).

## Reference (§20)
**N/A — Marley-specific.** An internal ownership refactor per Marley's own design doc
(pane-composition-model.md). The one external analog — gpui's `Entity<T>` — was evaluated and
REJECTED in #388 (the model layer stays gpui-free; RootView stays the one monolithic view); reading
gpui source for that evaluation was adoption (Apache-2.0). No copyleft source read.

### Prior art
1. **The #388 doc itself** — decision-complete: the central fork (own registry vs gpui Entity), the
   refcount catch, the migration-ripple table (incl. the close-teardown row + the EditorSurface birth
   sites), the train ordering. **P2 starts by re-reading it.**
2. **In-repo owners:** the PaneId-keyed side-tables (`agents`, `remotes`, `panes: HashMap<PaneId,
   PaneState<S>>`) — the pattern the registry generalizes; the PTY reaper contract (the close path
   whose ownership moves); the #163/#205 grid codec (the round-trip that must not notice).
3. **Permissive deps** — gpui `Entity<T>` (evaluated + rejected, #388); std `Rc`-style refcount
   semantics reimplemented as an explicit count (no owner crate needed).
4. **Behavior maps** — N/A (internal).

## Locked-In Decisions
- **D1 — Marley-owned registry, not gpui Entity** (#388 AD; the gpui-free model layer + cov/MSI 100
  discipline stand).
- **D2 — Explicit view-count; drop-on-last-close in the registry** (the refcount PR is binding).
- **D3 — Terminal is the only resident this slice;** the enum declares all kinds (a new kind must
  pick its variant here — the #385-comment discipline, one level down).
- **D4 — Codec untouched:** ids never persist; restore rebuilds registry entries from the restored
  shapes.
- **D5 — Split-if-big is a legitimate design outcome:** the #388 train had registry and
  terminals-migrate as separate slices; they are fused here conservatively — if P2 sizes it past one
  slice, split sequentially and say so (never silently absorb).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The registry shall `insert` content acquiring one initial view and allocating a monotonic `ContentId`. | Pure unit — cov/MSI 100. |
| REQ-002 | WHEN `acquire_view` is called on a live id the view-count shall increment; on an unknown id it shall return `None` without mutating. | Pure unit. |
| REQ-003 | WHEN `release_view` drops the LAST view, the registry shall remove the entry and return `Some(content)` (for the caller's teardown); while other views remain OR the id is unknown it shall return `None` and never underflow. | Pure unit matrix: the 2-views-release-one keeps-alive case; drop-on-EXACTLY-last returns the original content; double-release / unknown-id return `None` safely. cov/MSI 100. |
| REQ-004 | `get`/`get_mut`/`view_count`/`len` shall resolve a live id and return `None`/0 for an unknown one. | Pure unit. |
| REQ-005 | The registry shall be additive + dormant — the full existing suite passes byte-identical and `-D warnings` stays green (the `pub use` re-export makes it non-dead-code). | `cargo nextest` full suite unchanged + the gate (gate:2 clippy, gate:14 docs). |

**Moved to #396 (`cd706b38`, slice-2):** the concrete `Content` enum (all 6 kinds, Terminal wired); the
terminal-tab close→reaper firing from the registry's last-view drop; the #163/#205 persistence
round-trip on registry-backed terminals. These need the ownership migration, not just the lifecycle.

## Phase Plan
- **P2 Design** — ✅ the D5 sizing call: **SPLIT** (the #388 train's own slice-1/slice-2 separation +
  the Explore inventory — 2×L ownership + ~46 accessors + 7 close paths); #394 = the pure generic
  `ContentRegistry<C>`, terminals-migrate → #396. The registry shape + the `pub use` gate-safety idiom.
- **P3 Implement** — `content_registry.rs` (the newtype + the generic refcounted registry) + the `lib.rs`
  re-export + `///` docs. NO production wiring (dormant).
- **P3.5 Inspect** — critics: the refcount arithmetic (no underflow / off-by-one on the last-drop
  boundary), drop-on-last-close returns the RIGHT content, double-release / unknown-id safety, the
  `pub use` presence (gate-safety), missing-docs completeness.
- **P4 Validate** — the registry unit matrix (cov/MSI 100 — trace the real mutant list) + full suite
  byte-identical + `--diff` gate. NO driven capture (a dormant pure module — N/A).
- **P5 Complete** — CHANGELOG, pane-composition-model.md slice-1 marked SHIPPED (slice-2 → #396) +
  app_shell.md, archive, close #394.
