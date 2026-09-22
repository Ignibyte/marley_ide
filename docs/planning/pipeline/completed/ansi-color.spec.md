---
pipeline_id: 915e7ed0-3a5a-45e6-a82e-66a993994f81
ticket: forge#31 (49691435-36ef-43f5-88b3-ce95f2d29535) · local docs/planning/tickets/open/TICKET-031-ansi-color.md
aar_id: b9177448-b5ee-4484-828f-06bc63509754
status: Phase 5 — Complete PASS
title: ANSI color — grid cell colors → themed output
type: feature
milestone: M1.D
references:
  - crates/terminal_blocks/src/session.rs (term_to_rows → +term_to_styled_rows; call site :304)
  - crates/terminal_blocks/src/block.rs (output: Vec<String> → Vec<StyledLine>; output_text kept)
  - crates/terminal_blocks/src/apply.rs (set_current_output :87 — styled param)
  - crates/marley_app/src/terminal_view.rs + app.rs (styled block_row + per-run paint :581-588)
  - crates/ui_components/src/lib.rs (ThemeColors — the ANSI palette source)
  - docs/specs/SPEC-terminal-blocks.spec.md + SPEC-app-shell.spec.md
---

## Title
Output is monochrome — `term_to_rows` flattens the alacritty grid to `Vec<String>`, discarding
each cell's `.fg`/`.bg`/`.flags`, so `git status` / `ls --color` / build output render as flat
foreground text even though alacritty already parsed the SGR colors into the grid. Preserve color
end to end: a gpui-free styled model in `marley_terminal` (runs of same-style cells carrying
alacritty's `Color`/`Flags`), a theme ANSI palette + `Color → Hsla` mapping in `marley_app`, and a
per-run colored render.

## Scope
### In
**PURE-1 — `marley_terminal` (gpui-FREE; carries `alacritty_terminal::vte::ansi::{Color, Flags}`):**
- `StyledRun { text: String, fg: Color, bg: Color, flags: Flags }` + `StyledLine = Vec<StyledRun>`.
- `coalesce_row(cells: impl Iterator<Item=(char, Color, Color, Flags)>) -> Vec<StyledRun>` — merge
  adjacent cells with the SAME (fg, bg, flags) into one run; trailing trim equivalent to the old
  `trim_end()` so `output_text()` stays byte-identical. THE mutation-rich pure core here.
- `term_to_styled_rows(term) -> Vec<StyledLine>` — walk the grid → per-row cells → `coalesce_row`.
- `Block.output: Vec<String>` → `Vec<StyledLine>`; KEEP `output_text()` (flatten runs → plain
  `join("\n").trim_end()`) so all 13 callers + integration suites stay green; ADD
  `output_styled(&self) -> &[StyledLine]`. `set_output` + `set_current_output` (apply.rs:87) +
  session.rs:304 pass the styled vec.

**PURE-2 — `marley_app` (gpui; the theme-aware mapping):**
- `AnsiPalette { colors: [Hsla; 16], fg: Hsla, bg: Hsla }` with `AnsiPalette::for_appearance(
  Appearance) -> AnsiPalette` (a Marley-chosen, theme-tuned 16-color table + default fg/bg from
  `ThemeColors`). Clean-room: an ORIGINAL palette, not lifted.
- `ansi_color_to_hsla(color: Color, palette: &AnsiPalette, is_fg: bool) -> Hsla`:
  - `Named(Black..BrightWhite)` → `palette.colors[0..15]`; `Foreground`/`BrightForeground` →
    `palette.fg`; `Background` → `palette.bg`; `Dim*`/`Cursor` → sensible mapping.
  - `Indexed(0..=15)` → `palette.colors`; `Indexed(16..=231)` → the 6×6×6 cube
    (`i-16 → (r,g,b)` each in the 6-step ramp `[0,95,135,175,215,255]`); `Indexed(232..=255)` →
    grayscale ramp (`8 + (i-232)*10`); → `Hsla`.
  - `Spec(Rgb{r,g,b})` → truecolor passthrough (`rgb bytes → Hsla`).

**SHIM — `marley_app` render (app.rs :581-588 / terminal_view.rs):**
- Replace `for line in row.output { pane.child(line) }` with, per line, a `div().flex_row()` whose
  children are one `div().text_color(fg_hsla)[.font_weight(bold)]` per coalesced run; `INVERSE`
  swaps fg/bg. A styled `block_row` (or sibling) reads `block.output_styled()`.

### Out (explicitly deferred)
- Underline/strikeout/italic rendering (parse the flags but a first cut paints fg/bg + bold +
  inverse only); the bg as an actual painted background block (first cut colors the TEXT; bg fill
  can follow). BOLD→bright-color promotion (optional; off by default). The monospace-font fidelity
  (intake `terminal-monospace-font-and-cell-metric.md`, from #30). Scrollback is seq-5; raw grid
  seq-6.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `marley_terminal` STAYS gpui-free: the styled model carries `ansi::Color`/`Flags` (already a
  dep), NEVER `Hsla`. The `Color → Hsla` mapping lives in `marley_app` (has gpui + the palette).
- D2 — `output_text()` is PRESERVED (flatten styled runs → the same plain string) so the 13 call
  sites + both integration suites stay green — the low-ripple path. `output_styled()` is additive.
- D3 — TWO pure cores, tested independently: `coalesce_row` (marley_terminal) + `ansi_color_to_hsla`
  (marley_app). Both cov 100/MSI 100. The render + Block-type change + palette-fill are the wiring.
- D4 — The ANSI palette is an ORIGINAL Marley table (clean-room §20 — do not copy Warp's / any
  AGPL palette); theme-tuned per `Appearance`; fg/bg sourced from `ThemeColors`.
- D5 — Split option (31a model / 31b render) is AVAILABLE if implement balloons; default is ONE
  pipeline (the low-ripple path keeps it contained). Design confirms.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a row of cells is coalesced, `coalesce_row` shall merge each maximal run of adjacent cells sharing the same (fg, bg, flags) into one `StyledRun` whose `text` is those cells' chars in order, and shall drop trailing blank/default cells so the joined text equals the old `trim_end` output. | unit tests (a multi-run fixture: e.g. red "ab" + default " " + green "cd" → 2 runs "ab"/"cd" after trim; a style change mid-run splits) |
| REQ-002 | WHEN `ansi_color_to_hsla` is given a `Named` color it shall return the palette entry for that name (base/bright → `colors[0..15]`, Foreground → `fg`, Background → `bg`); a `Spec(rgb)` shall pass the truecolor through. | unit tests (each region: a named → its slot; Spec(0x11,0x22,0x33) → the matching Hsla) |
| REQ-003 | WHEN `ansi_color_to_hsla` is given `Indexed(i)`, it shall map `0..=15` to the palette, `16..=231` to the 6×6×6 cube via the `[0,95,135,175,215,255]` ramp, and `232..=255` to the grayscale ramp `8+(i-232)*10`. | unit tests (cube corners e.g. 16→(0,0,0), 231→(255,255,255), a mid index; grayscale 232→8, 255→238) |
| REQ-004 | WHEN a Block's output is read as text, `output_text()` shall return the same plain string as before this change (runs flattened, trailing blank trimmed). | unit + the existing session/apply/integration suites unchanged |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `coalesce_row` + `ansi_color_to_hsla`; a colored-output visual baseline asserts the render. | gate exit 0 + receipt + gate:15 |

## Phase Plan
- **P2 Design** — the exact `StyledRun`/`coalesce_row` signature + trailing-trim rule, the palette
  table + `ansi_color_to_hsla` arms (cube/ramp constants), the Block-type change + `output_styled`,
  the render element tree, the two SPEC amendments + mutation targets; CONFIRM one-pipeline vs split.
- **P3 Implement** — marley_terminal styled model + marley_app palette/map/render + specs + CHANGELOG.
- **P3.5 Inspect** — critics: the coalesce boundary/trim, the cube/ramp index math, the named table,
  the output_text-unchanged invariant, the gpui-free boundary held.
- **P4 Validate** — the two pure suites + gate GREEN + a colored visual baseline.
- **P5 Complete** — docs, AAR, archive, close #31.
