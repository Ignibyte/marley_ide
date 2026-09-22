---
pipeline_id: c59434cb-6b02-4fd0-b15c-2cf6511224ec
ticket: forge#36 (a3b9ca14-91d1-4f9b-9cc7-002d5839fee9) · local docs/planning/tickets/open/TICKET-036-block-chrome.md
aar_id: 2adddcce-a9a6-4a95-a047-c7bc0543f367
status: Phase 5 — Complete PASS
title: Block chrome — command cards + exit-status indicator
type: feature
milestone: M1.E
references:
  - crates/marley_app/src/block_status.rs (NEW — the pure StatusKind decision + indicator mapping)
  - crates/marley_app/src/app.rs (~760 — the block render; the header row is the shim)
  - crates/terminal_blocks/src/block.rs (BlockState{Pending,Running,Finished}, ExitCode(Option<i32>))
  - crates/ui_components/src/lib.rs (ThemeColors.success/danger/border — the indicator colors, #35)
  - docs/specs/SPEC-app-shell.spec.md (gains the Block-card clause + Mutation Targets)
---

## Title
Command Blocks render as bare stacked text (`pane.child(block.command)` then output) with no visual
separation or status. Give each Block Warp's signature look: a styled command HEADER row — an
exit-status INDICATOR (green ✓ success / red ✕ failure / neutral ○ running) + the command in a
distinct weight/color — and a subtle separator between blocks, output indented below.

## Scope
### In
- `crates/marley_app/src/block_status.rs` (NEW, PURE — cov/MSI 100):
  - `StatusKind { Running, Success, Failure }` (Copy).
  - `exit_status_kind(state: BlockState, exit: ExitCode) -> StatusKind` — `Pending|Running →
    Running`; `Finished` + `Some(0) → Success`; `Finished` + `Some(nonzero)|None → Failure`.
  - `status_indicator(kind: StatusKind, colors: &ThemeColors) -> (&'static str, Hsla)` — `Running →
    ("○", colors.border)`, `Success → ("✓", colors.success)`, `Failure → ("✕", colors.danger)`.
- `crates/marley_app/src/lib.rs` — register `mod block_status;`.
- `crates/marley_app/src/app.rs` (SHIM, ~760) — the command row becomes a HEADER: paint the
  indicator glyph in its color + the command in a distinct weight, and a subtle top separator/surface
  between blocks. Replaces the bare `pane.child(block.command.clone())`. Output rows unchanged (#31).
- SPEC-app-shell: the Block-card clause (R42) + Mutation Targets. CHANGELOG + arch doc.

### Out (explicitly deferred)
- A FULL wrapping card container (padding around command+output in one bordered div) — it fights the
  per-row viewport windowing (#32, each row clipped to [start,end)); the HEADER + separator is the
  signature 80% and respects the windowing. The wrapping container is a later cut if wanted.
- Hover affordances on the block (#39). Collapsing/re-running a block, block selection, timing/
  duration display (M2+). The prompt/input chrome (#37).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — The Warp look here = the styled HEADER row (indicator + command) + a block separator, NOT a
  wrapping card — because the render windows per ROW (#32), so a block isn't a single clip unit.
- D2 — 3 `StatusKind`s; `Pending`+`Running` fold to `Running` (both "in progress"); `Finished`+`None`
  (finished without a captured code) → `Failure` (an unknown finish is not a clean success).
- D3 — Distinct GLYPH **and** color per kind (✓/✕/○) so the status reads even in grayscale — not
  color-only. Colors come from #35's `success`/`danger`/`border` roles.
- D4 — `exit_status_kind` + `status_indicator` are PURE (marley_app, cov/MSI 100 — the mutation-rich
  decision); the card div/header layout is SHIM (app.rs, masked visual).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Block is `Pending` or `Running`, `exit_status_kind` shall return `Running`; WHEN `Finished` with exit `Some(0)` it shall return `Success`; WHEN `Finished` with `Some(nonzero)` or `None` it shall return `Failure`. | unit tests (each state/exit combo) |
| REQ-002 | WHEN `status_indicator(kind, colors)` is called, it shall return the kind's glyph + color: `Running →("○", border)`, `Success →("✓", success)`, `Failure →("✕", danger)`. | unit tests (each kind → exact glyph + the ThemeColors role) |
| REQ-003 | WHEN a Block is rendered, the system shall paint a header row with the exit-status indicator (colored per its kind) followed by the command, and a separator between blocks. | shim + the masked multi-block visual baseline (green ✓ + red ✕) — chad-verified |
| REQ-004 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `block_status`; the multi-block visual baseline rides the masked deferral. | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — the `block_status` module (exact signatures, the glyphs, the match arms + mutation
  targets), the app.rs header-row shim, the SPEC clause.
- **P3 Implement** — block_status + the mod registration + the app.rs header render + spec + CHANGELOG.
- **P3.5 Inspect** — critics: exit_status_kind exhaustiveness (every state/exit combo, the None case),
  status_indicator arms killable, glyph+color both distinct, the header render uses the pure decision.
- **P4 Validate** — the block_status unit tests (every arm) + gate GREEN + the masked visual.
- **P5 Complete** — docs, AAR, archive, close #36.
