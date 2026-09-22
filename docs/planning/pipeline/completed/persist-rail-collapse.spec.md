---
pipeline_id: 0cc8aa0d-168d-490f-a912-8493da669802
ticket: forge#245 (07560d41-0da6-4b6f-8af2-187f27c4fa37) · local docs/planning/tickets/open/TICKET-245-persist-rail-collapse.md
aar_id: b1f18e31-d587-4589-a563-8655830b5a45
status: Phase 5 — Complete PASS
title: Persist the per-workspace rail collapse-state across restart
type: feature
milestone: M14
references: [forge#236, forge#205, forge#234]
---

## Title
Persist the rail's per-project collapse-state (#236) across a restart — today `collapsed_projects` is in-memory
only, so a project you collapsed springs back open on relaunch.

## Scope
### In
- **A `CollapsedProjects: Vec<String>` setting** (the collapsed project ROOTS) mirroring `Recents` EXACTLY:
  the `define_setting!`, the `AppliedSettings.collapsed` field, `applied_defaults`/`applied_from`, and a
  `persist_collapsed(manager, roots)` fn (like `persist_recents`).
- **Pure helpers** (cov/MSI 100): `collapsed_roots(collapsed: &HashSet<usize>, project_roots: &[String]) ->
  Vec<String>` (SAVE — each in-range collapsed index → its root; out-of-range skipped) and
  `collapsed_indices(saved_roots: &[String], project_roots: &[String]) -> HashSet<usize>` (RESTORE — each
  project whose root is in `saved_roots` → its index; unknown roots skipped).
- **Shim wiring:** `toggle_project_collapse` (and the close-remap path) persist the collapsed set (by root);
  boot restores `collapsed_projects` from the applied `collapsed` setting via `collapsed_indices`.

### Out
- Multi-workspace collapse (one workspace × N projects today; the collapse is per-project-row).
- Persisting the nested-pane expansion (#236 collapses the project row's Tab/Pane children as a unit).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1 — key by the stable project ROOT, NOT the raw index** (indices shift on close — the #236 remap lesson; a
  root survives reordering).
- **D2 — mirror `Recents` exactly** (the `Vec<String>` setting round-trip — the #234/#87 established pattern).
- **D3 — the index↔root mapping is a pure tested fn pair** (`collapsed_roots`/`collapsed_indices`); the shim
  only wires them. Home: tabs.rs (beside `remap_indices_after_remove`, the sibling index-keyed helper) — Phase-2
  confirms.
- **D4 — persist on TOGGLE + on project-CLOSE** (the collapse set changes there). **VERIFY the persist is
  TRIGGERED** (the #243 lesson — grep the mutation path + a driven quit→relaunch), not just that the codec maps.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | `collapsed_roots` shall map each in-range collapsed index to its project root (skipping out-of-range); `collapsed_indices` shall add the index of each project whose root is in the saved set (skipping unknown roots); the pair shall round-trip. | pure unit tests |
| REQ-002 | Toggling a project's collapse shall persist the current collapsed set (by root) to settings; boot shall restore `collapsed_projects` from it. | shim review + driven |
| REQ-003 | Collapsing a project → quit → relaunch shall keep it collapsed (its tab/pane children stay hidden, the chevron ▸). | driven capture |

## Phase Plan
- **P2 Design** — the setting + AppliedSettings wiring (mirror Recents); the pure `collapsed_roots`/
  `collapsed_indices` (home + signatures); the persist trigger points (toggle + close) + the boot-restore;
  `cargo mutants --list`; the test matrix.
- **P3 Implement** — the setting + persist fn → the pure helpers → the shim wiring.
- **P3.5 Inspect** — critic: the mapping correctness (out-of-range/unknown skips), the persist IS triggered,
  the boot-restore, index-vs-root stability, clean-room. AWAIT the critic before Inspect-PASS.
- **P4 Validate** — RUN the helper units + the settings round-trip; gate green; DRIVEN (collapse → quit →
  relaunch → still collapsed).
- **P5 Complete** — CHANGELOG + app_shell.md; AAR; close #245; archive.
