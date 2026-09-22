---
pipeline_id: 38caaa4e-6ec7-4536-a8da-42dedb446e3b
ticket: forge#222 (a25d9c71-3ee0-4c01-ae97-ae5f7f2bb418) · local docs/planning/tickets/open/TICKET-222-warp-palette-chips.md
aar_id: 456b5924-efd2-4101-b7c0-e553d69866de
status: Phase 5 — Complete PASS (all phases PASS; the capture confirms REQ-001..004; the FINAL /work 195-222 ticket)
title: Warp visual parity — command-palette keycap chips (the final M12.2 warp-parity ticket)
type: feature
milestone: M12.2
references: []
---

## Title
Match Warp's command-palette content. #221 already rounded/framed the palette CARD, so this is the
CONTENT: the shortcut is rendered as plain space-joined TEXT (`.keys().join(" ")` → "cmd b"), not the
bordered rounded KEYCAP CHIPS Warp shows. ui_components already has `KeyboardShortcut::render(colors)`
(a chip per key) but it's minimal (a bare `surface` bg — no border/rounding/padding/gap) AND the palette
doesn't use it. Wire the palette to it + upgrade it to a real keycap, right-aligned.

## Scope
### In
- **D-A** — the palette command-row (app.rs ~4917-4938): render the shortcut via
  `KeyboardShortcut::parse(&binding.display()).render(colors)` (keycap chips) instead of the
  `.keys().join(" ")` plain text.
- **D-B** — upgrade `KeyboardShortcut::render` (ui_components render/keyboard_shortcut.rs): per-chip
  `.border_1().border_color(colors.border).rounded(colors.corner_radius)` + horizontal padding + a gap
  between chips (+ a `muted` text color — keycaps read dimmer). A real Warp keycap, not a bare bg.
- **D-C** — right-align the chips in the row: the title in a `.flex_1()` cell, the chips at the end.

### Out (explicitly deferred)
- The backdrop SCRIM (Warp dims the bg behind the palette) — a genuine Warp element but it touches all
  overlays / is a separate bounded add; defer to a follow-up.
- The search-field styling (`🔍 {query}` plain) — minor.
- The selected-row highlight (`bg(accent)` + `on_accent` text) — readable + defensible; kept.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — reuse the existing pure `KeyboardShortcut::parse`/`keys` (cov/MSI 100) — no new pure logic. The
  chip RENDER (`KeyboardShortcut::render`) is a `mutants::skip` / ACCEPTED-UNTESTABLE shim → capture-validated.
- **D2** — the keycap uses existing tokens (`border`/`surface`/`corner_radius`/`muted`) — no new hsla; §20.
- **D3** — `KeyboardShortcut::render` currently has NO consumers (the palette will be the first), so the
  upgrade affects only the palette.
- **D4** — auto-approved (/work 195–222, the FINAL ticket): document with the palette capture.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a palette command has a keybinding, its row shall render the shortcut as one keycap chip per key (via `KeyboardShortcut::render`), not plain space-joined text. | Driven capture (palette open; chips) + review (the palette calls `.render`, not `.join`) |
| REQ-002 | The keycap chips shall be right-aligned in the command row (the title takes the leading space). | Driven capture + review (`flex_1` title) |
| REQ-003 | Each keycap chip shall render with a visible border, rounded corners, and horizontal padding (a keycap, not a bare bg), with a gap between chips. | Driven capture (crop a chip) + review (`.border_1().rounded().px_*` + `.gap_*`) |
| REQ-004 | The change shall not alter the pure `KeyboardShortcut::parse`/`keys`, the command titles, the selected-row highlight, or `filter_commands`. | Review + capture |

## Phase Plan
- **P2 Design** — confirm the gpui keycap idiom (per-chip `border_1`+`border_color`+`rounded`+`px_1`, row
  `gap_1`; a `muted` text color) + the `AnyElement` composition into the palette row; the exact palette-row
  change (title `flex_1`, chips at end); confirm no consumers break; file manifest + test plan.
- **P3 Implement** — upgrade `KeyboardShortcut::render` + wire + right-align the palette row.
- **P3.5 Inspect** — critics: the chips render correctly (no clipping in the palette's `overflow_hidden`
  card), the parse/keys unchanged, the selected-row highlight still reads over the chips; clean-room.
- **P4 Validate** — driven capture (palette shortcuts as right-aligned keycap chips); gate.
- **P5 Complete** — CHANGELOG + app_shell doc; AAR; close. **This finishes the /work 195–222 range.**
