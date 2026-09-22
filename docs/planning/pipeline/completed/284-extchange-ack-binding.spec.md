---
pipeline_id: 2077c4fa-d6aa-4e5a-94cf-a8b59f5a0f34
ticket: forge#284 (c439b877-1e0d-4478-b359-f8b11a563a91) · local docs/planning/tickets/open/TICKET-284-extchange-ack-binding.md
aar_id: 18068cb8-5856-43a4-8231-ca2ee680c1f4
status: Phase 5 — Complete PASS
title: Bind external-change acknowledgments to the observed disk state — re-warn on a NEWER write
type: bug
milestone: M17
references: []
---

## Title
Follow-up from #275 inspect F3/F4a. The ⌘S-under-conflict ARM and the banner's Keep-mine both
acknowledge whatever disk state exists at PRESS/CLICK time, not the state the warning DESCRIBED. So an
agent write W1 flags the `Changed` banner → the user arms (1st ⌘S) or clicks Keep-mine → an agent lands
a NEWER write W2 → the armed 2nd ⌘S overwrites W2 under W1's license (`save_active` armed path,
app.rs:4342), and Keep-mine re-snapshots W2 sight-unseen (`ext_keep_mine`, app.rs:4261). The
acknowledgment outlived its referent. Extend D3 ("the next external change re-flags") to IN-FLIGHT
acknowledgments: bind each to the observed `(mtime, len)` and re-warn if the disk moved again.

## Scope
### In
- A pure decision — `acknowledgment_is_stale(acked: (SystemTime, u64), now: &DiskState) -> bool` in
  `extchange.rs`: true when `now` differs from the acknowledged pair (`Present` with a different
  `(mtime,len)`, or `Missing`). cov/MSI 100.
- Store the disk state the `Changed` banner/arm DESCRIBES — a new `OpenFile` field
  `conflict_observed: Option<(SystemTime, u64)>` (set alongside `conflict = Changed`, cleared with it).
  (A separate field, NOT data on `ExtConflict::Changed` — avoids rippling the enum through ~4 app sites +
  the render + tests; equivalent semantics.)
- `save_active` (armed 2nd ⌘S) + `ext_keep_mine`: re-stat, and if `acknowledgment_is_stale(observed,
  now)` → RE-WARN (disarm + refresh `conflict_observed` to the new state + keep the banner + flash "changed
  again"), NOT overwrite/re-snapshot the unseen content. If not stale → proceed exactly as today.
- Headless: the W1-arm → W2-lands → 2nd-⌘S-re-warns trace (buffer NOT written, banner still up); the
  Keep-mine equivalent (a newer write re-flags instead of dismissing).

### Out (explicitly deferred)
- An fs-watcher thread (v1 stays poll-at-choke — the recorded #275 upgrade).
- The same-mtime-same-len rewrite blind spot (the accepted D1 residue — a same-length W2 with the same
  second still slips; unchanged).
- The `Deleted` banner path (its ⌘S recreates; nothing on disk to clobber — no ack to stale).

## Reference (§20)
N/A — Marley-specific: the poll-at-choke external-change model is Marley's own (its core loop is an agent
rewriting files next to the editor). Clean-room; extends the in-repo #275 D3 principle.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — The staleness decision is PURE (`extchange.rs`, cov/MSI 100). The 2 handlers stay `mutants::skip`
  shims (they do fs IO), proven by headless flows.
- D2 — Store the observed disk in a NEW `OpenFile.conflict_observed` field, not on `ExtConflict::Changed`
  (less ripple; the arm's `save_armed: bool` stays — the observed pair is the SAME value the banner shows,
  so one field serves both the arm's 2nd-press check and Keep-mine).
- D3 — On a stale ack: DISARM (`save_armed=false`) + refresh `conflict_observed` to the new disk + keep the
  `Changed` banner + flash. A fresh warn cycle — the user must re-arm against the new state. Safe default.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---------|--------|
| REQ-001 | WHEN the user arms ⌘S on a `Changed` banner and a NEWER write lands before the 2nd ⌘S, the editor shall NOT overwrite — it shall re-warn (disarm, keep the banner, flash) | headless W1/W2/press-2 + unit |
| REQ-002 | WHEN the disk is UNCHANGED since the arm, the 2nd ⌘S shall overwrite + clear the banner + re-snapshot (today's behavior) | headless + unit |
| REQ-003 | WHEN Keep-mine is clicked and a NEWER write landed since the banner appeared, the editor shall re-flag (refresh the observed state, keep the banner) instead of dismissing/re-snapshotting | headless + unit |
| REQ-004 | WHEN the disk is unchanged since the banner, Keep-mine shall dismiss + re-snapshot (today's behavior) | headless + unit |
| REQ-005 | `acknowledgment_is_stale` shall reach 100% line coverage and MSI 100 | gate |

## Phase Plan
- **P2 Design** — the pure fn; the `conflict_observed` field + its set/clear sites; the 2 handler edits; the kill-list + headless flows (a MockPtyChannel-free tempdir file re-write drives W1/W2).
- **P3 Implement** — extchange.rs + editor_surface.rs + app.rs.
- **P3.5 Inspect** — critics (does re-warn leave a consistent state? the disarm timing? the arm/banner coupling?).
- **P4 Validate** — unit + headless; gate green.
- **P5 Complete** — CHANGELOG + editor.md #275 note; AAR; close.
