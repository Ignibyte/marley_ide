# Warp density calibration — Notes

- **Forge ticket:** #216 (54be0f67-0332-41e5-a9d6-31268f1ce23c)
- **AAR:** 2a540c15-0767-4908-af34-f015a9c9f458
- **Local ticket doc:** docs/planning/tickets/open/TICKET-216-warp-density.md
- **Pipeline spec:** warp-density.spec.md

## Phase 1 — Plan
- **Request:** calibrate spacing/density to Warp (spacing half of the Warp-look review). Bounded.
- **Classification / tier:** work pipeline, small/bounded. Systems: app.rs density consts + render
  px_/py_/gap_; possibly layout.rs. Auto-approved (/work 195-222).
- **Discovery (density drivers):** `DOCK_WIDTH=220`, `STATUS_BAR_H=22`, `TOP_BAR_H=30`,
  `PANE_TITLE_H=24`, `TRAFFIC_LIGHT_INSET=92` (app.rs consts); `caption_header` px_3/py_2 (dock header
  padding); rail rows px_2/py_1 (~22px); file-tree rows (~20px). Warp reference: `195-warp-ref.png`.
  Marley already reads fairly Warp-like (post #194/#195) — so the change is bounded to genuine deltas.
- **Decisions:** D1 bounded (only genuine deltas; no churn); D2 measure vs Warp (post-13pt the chrome
  can tighten ~1-2px); D3 exact-value pins on changed consts; D4 auto-approved, document with captures.

## Phase 2 — Design

**Measured Marley (`216-marley-now.png`) vs Warp (`195-warp-ref.png`).** Honest conclusion:
**Marley's density already matches Warp closely** after #194 (colors) + #195 (13pt fonts) — the
terminal is dense, the rail rows + bars are within ~2px of Warp. The ONE clearly-looser element is
the **dock/panel header**: `caption_header` = `.px_3().py_2()` → an ~30px header band, vs Warp's
compact ~22px section headers. Verified the file-tree rows have NO per-row padding (natural line
height — not safely tunable), and bar heights / dock width / pane title are within tolerance.

**Deltas (bounded — the single genuine one; the rest LEFT to avoid churn):**
| driver | current | change | why |
|---|---|---|---|
| `caption_header` vertical padding | `.py_2()` (8px) | **`.py_1()` (4px)** | dock/panel headers ("Workspace"/"Files"/Details/Agents/Forge) → ~22px, matching Warp's compact section headers |
| `PANE_TITLE_H` 24, `STATUS_BAR_H` 22, `TOP_BAR_H` 30, `DOCK_WIDTH` 220, rail/tree rows | — | **unchanged** | within ~2px of Warp; changing = churn, and some (dock width) are load-bearing geometry |

**Architecture / approach.** A single shim render-value change in `caption_header` (app.rs:333, a
`#[cfg_attr(test, mutants::skip)]` helper — no pure logic). No new types, no geometry const change,
so no ripple to `pane_rects`/focus-border/content-row math. §14 clean.

**File manifest.**
- `crates/marley_app/src/app.rs` — `caption_header`: `.py_2()` → `.py_1()` (line 333).

**Regression Test Plan.**
| REQ | test |
|---|---|
| REQ-001 | driven capture — the dock/panel headers render more compact (~22px), matching Warp |
| REQ-002 | driven capture — the shell renders intact (sidebar + terminal + dock, no clip/overlap) |
| REQ-003 | review — a plain spacing value, no Warp asset (§20) |
- No unit test: `caption_header` is a mutants::skip shim (padding value, no pure logic) — validated by
  the driven capture, like other shim-only visual changes (#192). The existing suite still runs at validate.

**Risks / decisions.**
- **R1** — honest bounded scope: only the dock-header padding genuinely differs; the rest is left
  unchanged (documented, per the "don't manufacture churn" rule). A small honest change > churn.
- **R2** — py_1 (4px) + 11pt caption + border_b = ~23px header; comfortably legible, not cramped.

## Phase 3 — Implement
- `caption_header` (app.rs:333): `.py_2()` → `.py_1()`. Consumers confirmed = exactly 2 (dock_panel
  header at 373, files_panel "Files" at 2265) — both get the compact header consistently.
- `cargo fmt` + `cargo check -p marley` clean. No deviations.

## Inspect (Phase 3.5)
Proportionate self-review for a 1-value shim padding change (no agent spawn — disproportionate).
Lenses, verified concretely:
- **Consistency/consumers** — `grep caption_header(` → 2 call sites (dock_panel, files_panel), both
  intended; the change applies uniformly (the two panel headers stay pixel-identical, per #190).
- **Layout intact** — py_1 (4px) + 11pt caption + border_b = ~23px header; legible, not cramped; no
  geometry const touched → no ripple to pane_rects/focus-border/content-row math.
- **Clean-room** — a plain spacing value, no Warp asset (§20).
- **Simplicity** — a single value; no logic. `caption_header` stays a mutants::skip shim (no pure test).
No findings; the change is minimal + correctly scoped.

## Phase 3.5 — Inspect
- …

## Phase 4 — Validate
- Tests RUN: `cargo nextest run -p marley` → 299 passed (no new unit test — shim-only padding).
- Driven capture (`216-marley-compact.png`): the "Workspace"/"Files"/dock headers render COMPACT
  (~22px band, was ~30px), matching Warp's section headers (REQ-001); the shell renders fully intact —
  sidebar + files tree + terminal pane, no clip/overlap (REQ-002).
- Gate: `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15**. (No new pure code; caption_header is a
  mutants::skip shim, so no cov/MSI burden.)
- Pre-existing: none.

## Phase 5 — Complete
- CHANGELOG + app_shell doc; AAR; archive.

## Phase 5 — Complete
- …
