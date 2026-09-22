---
pipeline_id: bae3b413-15f0-4f71-95e4-e9d3375739ff
ticket: forge#27 (ef3f9837-2d8b-4009-b930-96839267d156) · local docs/planning/tickets/open/TICKET-027-delete-marley-spike.md
aar_id: 69be8de9-21c6-4cb3-a9a3-7e2df3fa7d8d
status: Phase 5 — Complete PASS
title: delete marley_spike — the M0 throwaway
type: chore
milestone: M1.C
references:
  - crates/marley_spike/ (the crate being deleted)
  - scripts/gates.sh (the rust_cov exclude regex naming marley_spike)
  - docs/specs/SPEC-foundation-spike.spec.md (component: marley_spike + gate-15 visual_acceptance)
---

## Title
Delete `marley_spike` — the M0 viability spike, explicitly THROWAWAY ("deleted once the M0 gate is
green" — TICKET-006 / forge #11). It still ships in the workspace, so every FULL gate run pays to
cover + mutate dead code. Its write-first-exemplar role is superseded by `marley_app` (the live
gpui + PTY exemplar). Remove the crate and sweep every reference.

## Scope
### In
- Delete `crates/marley_spike/` (the whole crate — `src/`, `tests/`, `Cargo.toml`, the committed
  headed baseline). The workspace `members = ["crates/*"]` glob drops it automatically — no root
  Cargo.toml edit.
- `scripts/gates.sh` — remove `marley_spike/src/(app|term_io)\.rs|marley_spike/src/bin/` from the
  rust_cov `--ignore-filename-regex` (line ~196) + the explanatory comment (line ~177).
- `docs/specs/SPEC-foundation-spike.spec.md` — DELETE (its `component: marley_spike` +
  `visual_acceptance` is a gate-15 binding to a crate that will no longer exist; a dangling
  component spec is exactly the trap gate-15 guards against).
- `docs/marley_architecture/marley_spike.md` — DELETE (the as-built doc for the deleted crate).
- `docs/marley_architecture/crate-map.md` — remove the `MSP` mermaid node + the table row.
- Lineage prose that references the spike as a live crate: `terminal_blocks.md`, `app_shell.md`
  ("the marley_spike gpui boot it promotes"), and the precedent COMMENTS in
  `crates/marley_app/tests/headed_shell.rs` + `crates/ui_components/tests/headed_widgets.rs` —
  reworded to not name a deleted crate (keep the historical fact, drop the live-crate reference).
- `CHANGELOG.md` — a `### Removed` entry (§21). The historical spike entries stay (archival).

### Out (explicitly deferred)
- No behavior change to any surviving crate; no test logic changes beyond rewording two doc
  comments. The forge `PR-claude-write-first-not-research-first-known-api-exemplar-001` lesson is
  NOT edited (its exemplar is now marley_app; the lesson text is historical record).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — DELETE `SPEC-foundation-spike.spec.md` (not mark-historical): a spec with
  `component: marley_spike` + a non-N/A `visual_acceptance` but no `crates/marley_spike` dir is
  cruft — `visual_g` SILENTLY SKIPS it (`[ -d crates/$comp ] || continue`; confirmed at design, so
  it wouldn't FAIL the gate, but a dangling component spec is exactly the silent-skip trap the
  gate-15 component-dir rule exists to avoid). Removing the spec removes the binding cleanly.
  (Historical review docs elsewhere remain archival; this one names a deleted crate.)
- D2 — The workspace glob means NO root Cargo.toml edit; verify the build post-delete proves
  nothing depended on the spike (it was a leaf — no crate deps on it).
- D3 — Precedent comments reworded, not deleted — the blank-detector-pattern lineage is a real
  fact; it just shouldn't name a crate that's gone.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the change is applied, the system shall contain no `crates/marley_spike` directory and no reference to `marley_spike` in any workspace manifest, `scripts/gates.sh`, or surviving crate source/tests. | `grep -rn marley_spike crates/ scripts/ Cargo.toml` returns nothing (comments reworded) |
| REQ-002 | WHEN `cargo build`/`cargo nextest run --workspace` runs after the deletion, the workspace shall build and every surviving test shall pass — proving nothing depended on the spike. | `cargo nextest run --workspace` green |
| REQ-003 | WHEN gate-15 (`visual_g`) runs, it shall find no component spec bound to a missing crate directory — the `SPEC-foundation-spike.spec.md` binding is gone. | gate:15 PASS + no `SPEC-foundation-spike` in `docs/specs/` |
| REQ-004 | WHEN `scripts/gates.sh --diff` runs over the staged deletion, every gate shall be GREEN and write a commit-valid receipt (the deletion removes `crates/**/*.rs`, so a receipt is required). | gate exit 0 + receipt |
| REQ-005 | WHEN the architecture docs are read after the change, they shall not present `marley_spike` as a live crate (the as-built doc + crate-map node/row removed; lineage prose reworded). | review of crate-map.md / terminal_blocks.md / app_shell.md |

## Phase Plan
- **P2 Design** — confirm the exact gates.sh edit + the `visual_g` behavior with a missing
  component dir (why the spec must go); the precise doc rewordings; the delete manifest.
- **P3 Implement** — delete + sweep + CHANGELOG.
- **P3.5 Inspect** — verify no dangling reference (grep), no crate depended on the spike, gate-15
  reasoning holds.
- **P4 Validate** — `git rm` staged + `scripts/gates.sh --diff` GREEN.
- **P5 Complete** — docs, AAR, archive, close #27 → the M1.C sprint's final ticket.
