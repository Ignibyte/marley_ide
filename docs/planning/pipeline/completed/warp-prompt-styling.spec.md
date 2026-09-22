---
pipeline_id: 4fe2e86c-8462-47a5-93d8-74a432d61050
ticket: forge#218 (6167542d-7df6-454c-9364-243637980ab9) · local docs/planning/tickets/open/TICKET-218-warp-prompt-styling.md
aar_id: 97bc81fe-e922-470f-8719-099afe411db2
status: Phase 5 — Complete PASS (Plan · Design · Implement · Inspect · Validate all PASS; driven captures confirm REQ-001..005)
title: Warp visual parity — prompt input-row styling (no strip, dim chevron/cwd, block caret)
type: feature
milestone: M12.2
references: []
---

## Title
Match Warp's prompt look. The genuine deltas (confirmed at discovery — code at
app.rs ~4591-4632 + `prompt.rs`; Warp reference `scratchpad/195-warp-ref.png`;
current Marley `scratchpad/217-atrest2.png`):
Marley's prompt renders a **filled gray rounded strip** (`.bg(surface).rounded(...)` —
after #194 `surface` is gray on the near-black terminal → a visible box), a **bright
accent** `❯` marker, a **bright foreground** cwd segment, and a **thin 2px accent bar**
caret. Warp: **no strip** (prompt on the pane bg), a **dim gray** `❯`, a **dim** cwd
breadcrumb, and a **solid block** caret. Make Marley match.

## Scope
### In
- The prompt input-row render (app.rs ~4603-4630, gated by #193 `prompt_visible`):
  - **D-A** — remove the input-strip `.bg(colors.surface)` + `.rounded(colors.corner_radius)`;
    the prompt sits directly on the pane background (like Warp).
  - **D-B** — the `❯` marker color `colors.accent` → `colors.muted` (a dim chevron).
  - **D-C** — the Cwd segment color `colors.foreground` → `colors.muted` (a dim
    breadcrumb); the Git segment stays `colors.accent`.
  - **D-D** — the caret: the 2px accent bar → a Warp-style **block cursor** — reverse-video
    the grapheme at the caret (bg = cursor/accent color, fg = pane bg); a standalone block
    when the caret is at end-of-line (nothing after it).
- A small **pure** helper beside `prompt.rs` to split the buffer at the caret into
  `(before, caret_grapheme, after)` so the block-cursor render is driven by tested logic,
  not ad-hoc slicing in the shim.

### Out (explicitly deferred)
- Blink animation on the block cursor (Warp's cursor blinks; Marley's is static for now —
  a follow-up, not a parity blocker at rest).
- The cwd/git **segment model** (`prompt_segments`/`pwd_label`, #37/R43) — unchanged; only
  the render colors move.
- The #193 prompt-visibility gate (hide while a command runs) — unchanged.
- Sidebar / cursor-in-scrollback / block styling — separate warp-parity tickets
  (#219 / #220 / #217-done).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — the block cursor reverse-videos the caret grapheme (`bg`=cursor color, `fg`=pane
  bg). At EOL (no grapheme under the caret) it draws a standalone filled block of one
  cell width. Design confirms the exact width/height (cell metric) + the cursor color token.
- **D2** — a new pure `split_caret_grapheme(text, caret) -> (before, Option<grapheme>, after)`
  (or equivalent) beside `prompt.rs`, cov/MSI 100 with exact-value tests; the shim consumes
  it. Keeps the caret logic testable (the existing `split_at_caret` is byte-halves only).
- **D3** — colors come from the existing `colors` tokens (`muted`, `accent`, `background`);
  no new hardcoded hsla. Clean-room — measured/original, no Warp assets (§20).
- **D4** — auto-approved (/work 195–222): document with the Warp-vs-Marley captures.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The prompt input row shall render with NO filled background strip or rounded corners (it sits on the pane background). | Driven capture (prompt on bg, no gray box) + review (no `.bg`/`.rounded` on the input row) |
| REQ-002 | The `❯` prompt marker shall render in the muted color, and the Cwd segment shall render in the muted color while the Git segment stays accent. | Driven capture + review |
| REQ-003 | WHEN the caret is within the typed text, the prompt shall render a block cursor that reverse-videos the grapheme at the caret (cursor-colored cell, pane-bg glyph). | Driven capture (block over a mid-line char) + unit test on the split helper |
| REQ-004 | WHEN the caret is at end-of-line, the prompt shall render a standalone block cursor of one cell. | Driven capture (block at EOL) + unit test (helper returns `None` grapheme → standalone block) |
| REQ-005 | The pure caret-split helper shall return the text before the caret, the grapheme at the caret (or none at EOL), and the text after, for all boundary cases (empty, caret 0, caret at len, multi-byte). | Unit tests (exact-value, cov/MSI 100) |

## Phase Plan
- **P2 Design** — confirm the cursor color token + cell width/height for the block; the exact
  render change (drop `.bg`/`.rounded`; recolor `❯`+Cwd; block-cursor sub-render from the
  helper); the pure helper signature + test plan; file manifest.
- **P3 Implement** — the pure `split_caret_grapheme` helper + the input-row render change.
- **P3.5 Inspect** — critics: the caret helper handles multi-byte/grapheme boundaries (no
  panic, no mid-codepoint slice); the block cursor doesn't regress the #88 gapless caret line;
  no color-token misuse; clean-room.
- **P4 Validate** — helper unit tests (exact-value) + driven captures (no strip, dim chevron,
  block caret mid-line & EOL); gate green.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close.
