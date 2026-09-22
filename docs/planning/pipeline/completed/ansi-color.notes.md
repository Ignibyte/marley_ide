# ANSI color — Notes

- **Forge ticket:** #31 `49691435-36ef-43f5-88b3-ce95f2d29535`
- **AAR:** `b9177448-b5ee-4484-828f-06bc63509754`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-031-ansi-color.md
- **Pipeline spec:** ansi-color.spec.md

## Phase 1 — Plan
- **Request:** forge #31 (M1.D "The Daily Driver" seq-4, auto-approved) — the biggest visual
  upgrade: preserve per-cell ANSI color from the alacritty grid to the render.
- **Classification / tier:** work pipeline, `feature`, CROSS-CRATE (marley_terminal model +
  marley_app palette/map/render; ui_components read for fg/bg). The largest M1.D ticket.
- **Explore discovery (§18.2) — the load-bearing facts:**
  - `Cell { c: char, fg: Color, bg: Color, flags: Flags }` (alacritty cell.rs:132); default
    fg=`Named(Foreground)`, bg=`Named(Background)`. `Color = Named(NamedColor)|Spec(Rgb)|
    Indexed(u8)` (vte ansi.rs:1128). `Rgb { r,g,b: u8 }` at `alacritty_terminal::vte::ansi::Rgb`.
    `NamedColor`: 8 base (Black=0..White=7), 8 bright (BrightBlack..BrightWhite), Foreground=256/
    Background/Cursor, 8 dim, BrightForeground/DimForeground. `Flags`: BOLD/DIM/ITALIC/INVERSE/… .
  - **`marley_terminal` (pkg name; dir crates/terminal_blocks) is gpui-FREE** (Cargo.toml: only
    alacritty_terminal + rustix + marley_core). MUST stay so → the styled model carries `ansi::
    Color`/`Flags`, the `Color→Hsla` map lives in marley_app (D1).
  - `Block.output: Vec<String>` (block.rs:75); `output_text()` (:82, PUBLIC, `join("\n").trim_end`);
    `set_output` (:87, pub(crate)). The ONLY production set_output caller is apply.rs:89
    (`set_current_output`), fed by session.rs:304 `term_to_rows(&self.term)`. 13 output_text call
    sites (tests + app.rs:583 + 2 integration suites) → KEEPING output_text() keeps them green (D2).
  - `ThemeColors` (ui_components lib.rs:42) has ONLY 7 semantic Hsla + 3 metrics — NO ansi palette.
    Must ADD one (D4 — a separate `AnsiPalette` in marley_app, not extending ThemeColors, to avoid
    the field-equality-test ripple).
  - Render: app.rs:581-588 `for line in row.output { pane.child(line) }` — replace per-line with a
    flex-row of per-run colored divs. `gpui::rgb(0xRRGGBB) -> Rgba`, `Hsla::from(Rgba)` for Spec;
    palette lookup → Hsla for Named/Indexed.
- **Decisions:** D1–D5 in the spec (gpui-free model; output_text preserved; two pure cores;
  original clean-room palette; one-pipeline default with a split option).
- **Open questions for Design:** exact `coalesce_row` input shape (an iterator of tuples vs a
  &Row); the trailing-trim rule (drop trailing runs that are all-space AND default-style, matching
  `trim_end` on the joined string — must keep output_text byte-identical); whether `ansi_color_to_hsla`
  lives in a new marley_app `color.rs`; the exact 16-color palette values (dark + light); whether
  INVERSE/BOLD handled in the map or the render (render: swap/weight); Cursor/Dim* fallback.
- **AAR id:** `b9177448-b5ee-4484-828f-06bc63509754`.

## Phase 2 — Design

### Decision: ONE pipeline (not split). The low-ripple path (output_text preserved) + two small
pure cores keep #31 contained. (D5 resolved — no 31a/31b split.)

### PURE-1 — `marley_terminal` (gpui-free), NEW `crates/terminal_blocks/src/styled.rs`
```rust
use alacritty_terminal::vte::ansi::{Color, Flags};   // wait: Flags is alacritty term::cell::Flags
pub struct StyledRun { pub text: String, pub fg: Color, pub bg: Color, pub flags: Flags }
pub type StyledLine = Vec<StyledRun>;

/// Coalesce a row's cells into maximal same-(fg,bg,flags) runs, then trailing-trim so the joined
/// run text == the old `raw_row.trim_end()` (keeps output_text byte-identical).
pub fn coalesce_row(cells: impl IntoIterator<Item = (char, Color, Color, Flags)>) -> StyledLine {
    let mut runs: Vec<StyledRun> = Vec::new();
    for (c, fg, bg, flags) in cells {
        match runs.last_mut() {
            Some(r) if r.fg == fg && r.bg == bg && r.flags == flags => r.text.push(c),
            _ => runs.push(StyledRun { text: c.to_string(), fg, bg, flags }),
        }
    }
    // Trailing-trim == str::trim_end across run boundaries:
    while let Some(last) = runs.last_mut() {
        let t = last.text.trim_end();
        if t.is_empty() { runs.pop(); }
        else if t.len() != last.text.len() { last.text.truncate(t.len()); break; }
        else { break; }
    }
    runs
}
```
(NOTE: `Flags` is `alacritty_terminal::term::cell::Flags` — Explore §1; `Color` is `vte::ansi::Color`.
Confirm exact import paths at implement.)

- `session.rs`: `term_to_styled_rows(term) -> Vec<StyledLine>` — walk grid like `term_to_rows` but
  yield `(cell.c, cell.fg, cell.bg, cell.flags)` per cell → `coalesce_row` per line. Swap the
  call at session.rs:304 to feed it (`set_current_output`).
- `block.rs`: `output: Vec<StyledLine>`; `output_text()` reworked to flatten (concat run.text per
  line → `join("\n").trim_end()` — byte-identical, D2); ADD `output_styled(&self) -> &[StyledLine]`;
  `set_output(Vec<StyledLine>)`. `apply.rs:87` param type follows.

### PURE-2 — `marley_app`, NEW `crates/marley_app/src/color.rs`
```rust
const ANSI_RGB: [(u8,u8,u8); 16] = [ /* Marley-tuned ORIGINAL 16 (clean-room D4) */ ];
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

pub struct AnsiPalette { pub colors: [Hsla; 16], pub fg: Hsla, pub bg: Hsla }
pub fn hsla_from_rgb(r: u8, g: u8, b: u8) -> Hsla { Hsla::from(gpui::rgb(((r as u32)<<16)|((g as u32)<<8)|b as u32)) }
impl AnsiPalette {
    pub fn from_theme(t: &ThemeColors) -> Self {   // NO Appearance dep — ANSI-16 is appearance-independent
        AnsiPalette { colors: ANSI_RGB.map(|(r,g,b)| hsla_from_rgb(r,g,b)), fg: t.foreground, bg: t.background }
    }
}
pub fn ansi_color_to_hsla(color: Color, p: &AnsiPalette) -> Hsla {
    match color {
        Color::Named(NamedColor::Foreground | ::BrightForeground | ::DimForeground | ::Cursor) => p.fg,
        Color::Named(NamedColor::Background) => p.bg,
        Color::Named(n) => p.colors[named_index(n)],     // 0..15 base/bright (+dim→base)
        Color::Spec(rgb) => hsla_from_rgb(rgb.r, rgb.g, rgb.b),
        Color::Indexed(i @ 0..=15) => p.colors[i as usize],
        Color::Indexed(i @ 16..=231) => { let x=i-16; hsla_from_rgb(CUBE[(x/36) as usize], CUBE[((x%36)/6) as usize], CUBE[(x%6) as usize]) },
        Color::Indexed(i) => { let l = 8 + (i as u16 - 232)*10; hsla_from_rgb(l as u8, l as u8, l as u8) },  // 232..=255
    }
}
pub struct RunPaint { pub color: Hsla, pub bold: bool }
pub fn run_paint(run: &StyledRun, p: &AnsiPalette) -> RunPaint {
    let inverse = run.flags.contains(Flags::INVERSE);
    RunPaint {
        color: if inverse { ansi_color_to_hsla(run.bg, p) } else { ansi_color_to_hsla(run.fg, p) },
        bold: run.flags.contains(Flags::BOLD),
    }
}
```
`named_index(NamedColor) -> usize` maps Black..White→0..7, BrightBlack..BrightWhite→8..15, Dim*→0..7.

### SHIM — `marley_app/src/app.rs` render (replaces app.rs:581-588)
Per block: `pane.child(block.command)` then, per `line in block.output_styled()`, a
`div().flex().flex_row()` whose children are per-run `div().text_color(paint.color)` (`.font_weight(
FontWeight::BOLD)` when `paint.bold`) `.child(run.text.clone())`, with `paint = run_paint(run,
&palette)`, `palette = AnsiPalette::from_theme(&self.theme)`. REMOVE `terminal_view.rs`
(`block_row`/`BlockRow` are superseded → avoids dead-code gate:9).

### File manifest
- A `crates/terminal_blocks/src/styled.rs` — StyledRun/StyledLine/coalesce_row + tests.
- M `crates/terminal_blocks/src/lib.rs` — `mod styled;` + exports.
- M `crates/terminal_blocks/src/block.rs` — output styled; output_text flatten; output_styled; set_output.
- M `crates/terminal_blocks/src/session.rs` — term_to_styled_rows; call site :304.
- M `crates/terminal_blocks/src/apply.rs` — set_current_output param type.
- A `crates/marley_app/src/color.rs` — AnsiPalette/ANSI_RGB/hsla_from_rgb/ansi_color_to_hsla/run_paint + tests.
- M `crates/marley_app/src/lib.rs` — `mod color;` + exports; drop `mod terminal_view`.
- D `crates/marley_app/src/terminal_view.rs` — removed (superseded).
- M `crates/marley_app/src/app.rs` — the per-run colored render.
- M `docs/specs/SPEC-terminal-blocks.spec.md` (styled model) + `SPEC-app-shell.spec.md` (colored render).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `coalesce_row_merges_runs_and_trailing_trims` — fixture red"ab"+def" "+green"cd"+def"  " → runs ["ab"(red)," "(def),"cd"(green)] (trailing "  " dropped; concat "ab cd" == raw.trim_end); a style change mid-row splits; an all-default-space row → `[]` | unit (marley_terminal) |
| REQ-002 | `ansi_named_and_spec` — Named(Red)→colors[1], BrightBlack→colors[8], Foreground→fg, Background→bg; Spec(0x11,0x22,0x33)→hsla_from_rgb(0x11,0x22,0x33) | unit (marley_app) |
| REQ-003 | `ansi_indexed_cube_and_grayscale` — 0→colors[0],15→colors[15]; 16→(0,0,0),231→(255,255,255),196→(255,0,0),21→(0,0,255); 232→(8,8,8),255→(238,238,238),243→(118,118,118) | unit (marley_app) |
| REQ-004 | `output_text_stays_plain_after_styling` — a styled Block → output_text() == expected plain (byte-identical) + RUN the existing session/apply/integration suites (unchanged, green) | unit + suites |
| — | `run_paint_inverse_swaps_and_bold` — INVERSE → color from bg; BOLD → bold true; plain → fg + false | unit (marley_app) |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt + gate:15 colored baseline | gate |

Uncoverable: the app.rs div-tree paint (shim); `term_to_styled_rows` grid-walk (exercised by the
session/integration suites via output_text, mirroring term_to_rows' current posture).

### Risks / decisions
- **output_text byte-identical (D2)** — the coalesce trailing-trim must equal `str::trim_end` across
  run boundaries (the pop-empty-then-truncate-last loop). REQ-004 + the existing suites are the
  guard; if any session/apply/integration test shifts, the trim rule is wrong.
- **gpui-free boundary (D1)** — styled.rs/block.rs import ONLY alacritty types; the inspect
  security/provenance lens confirms no gpui/ui_components edge enters marley_terminal.
- **Clean-room palette (D4)** — ANSI_RGB is an ORIGINAL Marley table (not Warp's / an AGPL palette).
- **`term_to_styled_rows` MSI** — if the gate flags a grid-walk mutant (unlikely; output_text
  covers it), add a targeted session test; the mutation-rich logic is in `coalesce_row` (unit).
- Bold→bright promotion, dim, italic, underline, bg-fill: DEFERRED (spec Out) — first cut is
  fg + bold + inverse.

## Phase 3 — Implement
- **Built (per manifest):** marley_terminal — NEW styled.rs (`StyledRun{text,fg:Color,bg:Color,
  flags:Flags}`, `StyledLine`, `coalesce_row` with the trailing-trim loop + `plain_lines` test
  helper); block.rs (`output: Vec<StyledLine>`, `output_text` flatten, `output_styled`,
  `set_output` styled); session.rs (`term_to_rows`→`term_to_styled_rows` + call site :304);
  apply.rs (`set_current_output` styled param); lib.rs (`mod styled` + exports + RE-EXPORT
  `Color`/`NamedColor`/`Flags`). marley_app — NEW color.rs (`AnsiPalette`, `ANSI_RGB` original
  16, `CUBE_STEPS`, `hsla_from_rgb`, `named_index`, `ansi_color_to_hsla`, `run_paint`); app.rs
  render (per-run colored spans + `palette = AnsiPalette::from_theme(&colors)`, FontWeight::BOLD);
  lib.rs (`mod color` + exports, dropped `terminal_view`); REMOVED terminal_view.rs. SPEC-terminal-
  blocks R20b + SPEC-app-shell R38 (+ AC/Test-Plan/Mutation-Targets); CHANGELOG (Added + Removed).
- **Deviations from design:**
  - (D-3.1) marley_app had no direct `alacritty_terminal` dep → RE-EXPORTED `Color`/`NamedColor`/
    `Flags` from marley_terminal (they're already part of the public `StyledRun` API) and imported
    them via `marley_terminal::` — keeps marley_app off a direct alacritty edge.
  - (D-3.2) `AnsiPalette::from_theme(&ThemeColors)` — NO `Appearance` param: the ANSI-16 table is
    appearance-independent, fg/bg come from the theme; app.rs already holds `self.theme.colors`.
  - (D-3.3) three existing tests (apply.rs:211/215, block.rs:263) called `set_output(vec![String])`
    → updated to `crate::styled::plain_lines(&[...])` (needed to compile; intent unchanged — they
    still assert `output_text`).
- **Verification at this phase:** `cargo check` 0 errors; fmt; clippy `-D warnings` 0; docs gate
  (RUSTDOCFLAGS=-D warnings) 0 (proactive, per the #29 lesson); machete clean; marley_terminal lib
  70 + integration 5/3 green (output_text BYTE-IDENTICAL invariant HELD). The two pure suites +
  visual are Phase 4.

## Phase 3.5 — Inspect
- **Critics:** 3 — A (coalesce/output_text byte-identity: 27 explicit + 600K fuzz + a real
  cargo-mutants run), B (color-map arithmetic: 52 probes + a real Spec construction), C
  (boundary/provenance: cargo tree + greps + build).
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | B6 | HIGH | `Color::Spec(rgb)` is UNCONSTRUCTIBLE in marley_app tests — `Rgb` wasn't re-exported from marley_terminal + marley_app has no alacritty dep → the Spec branch (color.rs) is uncoverable (gate:4) + its mutant unkillable (gate:5). | REAL (code) | `pub use alacritty_terminal::vte::ansi::Rgb;` added to marley_terminal lib.rs. |
  | B9 | HIGH | `named_index`'s `_ => 7` arm is UNREACHABLE (all 5 special variants intercepted upstream; all 24 base/bright/dim have explicit arms) → a permanently-uncovered line (gate:4 <100%) + unkillable mutant. | REAL (code) | Removed `named_index`; inlined an EXHAUSTIVE `NamedColor` match (all 29 variants, no `_`) into `ansi_color_to_hsla` — every arm reachable. (NamedColor is not `#[non_exhaustive]`, verified — so exhaustive-no-catch-all compiles.) |
  | A1 | HIGH (test-gap → Phase 4) | `coalesce_row`'s run-PARTITIONING (the style-merge guard styled.rs:37) is INVISIBLE to `output_text` (concat is invariant under repartition) → 7/9 mutants survive on output_text-only tests → MSI 22%. | REAL (Phase-4 constraint) | Phase 4 styled.rs tests MUST assert RUN STRUCTURE (run count + per-run fg/bg/flags), not just text. Two fixtures kill all 7: same-style adjacent → 1 run; different-fg adjacent → 2 runs asserting each `run.fg`. |
  | B5 | LOW (verified-safe → Phase-4 note) | `Indexed(16)` is SAFE (cube branch, no OOB on colors[16]); but Phase-4 tests must probe 15/16 + 231/232 boundaries to kill `<16`/`<232` mutants (else they panic-or-wrong). | REAL (Phase-4 constraint) | Baked into REQ-003 test. |
  | A/C-misc | LOW-deferred | HIDDEN flag not blanked (PRE-EXISTING, not a regression — old term_to_rows pushed .c too); DIM/ITALIC/UNDERLINE/bg-fill deferred (CHANGELOG-noted); wide-char spacer parity HOLDS. | ACCEPTED | none (follow-ups) |
- **Verified CORRECT (all cleared):** byte-identity `concat(coalesce_row(row)) == raw.trim_end()`
  over 600K fuzz rows incl. NBSP/ideographic-space/tab/multibyte + cross-run-boundary trim + empty
  + multi-line; color arithmetic 52/52 (cube corners+mid, grayscale, named table, Spec passthrough,
  run_paint INVERSE/BOLD); **gpui-free boundary HELD** (cargo tree: no gpui/ui_components in
  marley_terminal); no dangling terminal_view refs; marley_app has NO direct alacritty dep;
  palette ORIGINAL (matches no known branded theme); plain_lines gated correctly.
- **Post-fix:** both code fixes compile + clippy `-D warnings` clean. The A1/B5 test constraints are
  recorded for Phase 4. Lesson: `PR-claude-invariant-invisible-logic-needs-structural-tests` —
  when a transform's OBSERVABLE projection (output_text) is invariant under the very transformation
  under test (run partitioning), output-only tests can't kill the logic's mutants; assert the
  intermediate STRUCTURE. Plus `PR-claude-unreachable-match-arm-is-uncoverable` (an upstream-
  intercepted `_` arm fails cov-100 — make the match exhaustive + all-arms-reachable).

## Phase 4 — Validate
- **Tests added:** styled.rs (3) — `coalesce_row_merges_same_style_and_splits_on_change` (RUN
  STRUCTURE: count + per-run fg/bg/flags — the A1 fix that kills the 7 partition mutants),
  `coalesce_row_trailing_trims_across_runs`, `coalesced_runs_flatten_to_trim_end`. block.rs (1) —
  `output_styled_returns_the_runs`. color.rs (4) — `ansi_named_and_spec` (EVERY arm of the
  exhaustive match), `ansi_indexed_cube_and_grayscale` (cube corners/mid + 15/16/231/232
  boundaries + grayscale), `run_paint_inverse_swaps_and_bold`, `palette_from_theme_…`,
  `hsla_from_rgb_maps_each_channel_to_the_right_hsl` (ground-truth HSL).
- **The gate saga (4 runs — three real lessons):**
  1. **Run 1 RED** (cov + MSI): color.rs 88% + the 7 coalesce partition mutants. Fixed the coalesce
     structural tests (already had them) + added from_theme/output_styled.
  2. **Metric confusion resolved:** `rust_cov` uses `--fail-under-lines` = the LINE metric, NOT the
     region % the summary table leads with. A stash-baseline of clean #30 proved #30 is ALSO 99.67%
     REGIONS but 100% LINES — so the "37 missing / 99.6%" was regions (a red herring); only color.rs's
     11 MISSED LINES gated. The all-arms `ansi_named_and_spec` (each exhaustive-match arm is its own
     line) brought lines → 100%. `PR-claude-cov-gate-is-lines-not-regions`.
  3. **Run 3 RED** (MSI 86%): all 7 missed mutants in `hsla_from_rgb`. TWO causes: (a) the color
     tests used `hsla_from_rgb` to build the EXPECTED value → a mutation broke both sides of the
     assert equally → survived (self-oracle); (b) `(r<<16)|(g<<8)|b` packs DISJOINT bits, so `|→^`
     (and `|→+`) are EQUIVALENT mutants (unkillable). FIX: rewrote `hsla_from_rgb` to build
     `gpui::Rgba{r,g,b,a}` DIRECTLY (no bit-ops → no equivalent mutant) + a GROUND-TRUTH HSL test
     (red→h0/s1/l0.5, green→h⅓, blue→h⅔, gray→l0.5/s0 — independent of the impl). Targeted
     `cargo mutants --file color.rs` → 34 caught / 0 missed / 2 unviable → MSI 100%.
     `PR-claude-self-oracle-test-cant-kill-mutants` + `PR-claude-disjoint-bit-pack-is-equivalent-mutant`.
- **Runs (actual):** all suites green (84 marley, 162 combined). Visual gate:15 PASS.
- **Gate:** final `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100% lines,
  mutation **50 caught / 0 missed → MSI 100.0%**, 0 SLOW hangs; receipt written. 480 workspace
  tests pass.
- **Pre-existing:** the workspace REGION coverage sits at ~99.67% (settings/shell_integration/
  workspace/marley_settings have a few uncovered regions on covered lines) — this is the #30
  baseline, unchanged by #31, and does not gate (`--fail-under-lines` is line-based).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG Added+Removed (at implement); `terminal_blocks.md` gains a `styled.rs`
  bullet (StyledRun/coalesce_row + gpui-free note); `app_shell.md` `terminal_view.rs` bullet
  replaced with `color.rs` (palette/map/run_paint). SPEC-terminal-blocks R20b + SPEC-app-shell R38.
- **Knowledge captured (5 novel):** inspect — `PR-…-invariant-invisible-logic-needs-structural-tests`
  (coalesce partitioning invisible to output_text), `PR-…-unreachable-match-arm-is-uncoverable`.
  Phase-4 — `PR-…-self-oracle-test-cant-kill-mutants` (hsla_from_rgb used as its own expected),
  `PR-…-disjoint-bit-pack-is-equivalent-mutant` (`|→^` on disjoint bits), `PR-…-cov-gate-is-lines-
  not-regions`. aar-submit `completed` (5 novel findings).
- **Win:** the Explore-first discovery (the exact alacritty API + the low-ripple output_text-
  preserved path + the gpui-free-boundary constraint) made a 2-crate model change land with all 13
  existing callers untouched. The cost was 4 gate cycles on the mutation/coverage tail — every one
  a real testability defect (structural test, exhaustive arm, self-oracle, equivalent mutant), not
  a floor fight. **The biggest visual upgrade of the sprint — output now renders in color.**
- **Ticket:** forge #31 → done; local doc → closed/; pipeline pair archived. 4 of 6 in M1.D.
