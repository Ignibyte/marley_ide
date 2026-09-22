# Block chrome — Notes

- **Forge ticket:** #36 `a3b9ca14-91d1-4f9b-9cc7-002d5839fee9`
- **AAR:** `2adddcce-a9a6-4a95-a047-c7bc0543f367`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-036-block-chrome.md
- **Pipeline spec:** block-chrome.spec.md

## Phase 1 — Plan
- **Request:** forge #36 (M1.E "The Warp Look" seq-3, auto-approved) — Warp's signature Block card:
  a styled command header + an exit-status indicator. The look people recognize as "Warp".
- **Classification / tier:** work pipeline, `feature`, a PURE decision surface (block_status —
  cov/MSI 100) + a SHIM header render (app.rs, masked). marley_app only.
- **Discovery (§18):**
  - `BlockState { Pending, Running, Finished }` + `ExitCode(pub Option<i32>)` (block.rs) — both
    `Copy` (derives), so the pure fn takes them by value.
  - `Block { command: String, state, exit_code, … }`. Current render (app.rs:760) = for each block
    in the viewport window, `pane.child(block.command.clone())` then the #31 colored output rows.
  - Render windows PER ROW (#32 `visible(content, capacity)` → [start,end)) — so a block spans rows
    and is NOT a single clip unit → a wrapping card fights the windowing → do a styled HEADER +
    separator (D1).
  - marley_app modules: app/color/history/input/keymap/layout/palette/settings/shell_integration/
    themes/viewport/workspace → add `block_status`.
  - Deps #34 (mono font) + #35 (palette + `success`) are DONE — the indicator uses success/danger/
    border.
- **Decisions:** D1–D4 in the spec (header not wrapping-card; 3 kinds with Pending+Running folded +
  Finished/None→Failure; distinct glyph AND color; pure decision + shim layout).
- **Open questions for Design:** the exact glyphs (✓/✕/○ vs ●/•) — leaning ✓/✕/○ for grayscale
  legibility; where block_status's types import from (BlockState/ExitCode from marley_terminal
  re-export, ThemeColors/Hsla from ui_components/gpui); the header weight/indent for the command.
- **AAR id:** `2adddcce-a9a6-4a95-a047-c7bc0543f367`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/block_status.rs` (NEW)
```rust
use gpui::Hsla;
use marley_terminal::{BlockState, ExitCode};
use marley_ui_components::ThemeColors;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind { Running, Success, Failure }

/// Classify a Block by state + exit code (R42). State wins: only a Finished block reads its code.
pub fn exit_status_kind(state: BlockState, exit: ExitCode) -> StatusKind {
    match state {
        BlockState::Pending | BlockState::Running => StatusKind::Running,
        BlockState::Finished => match exit.0 {
            Some(0) => StatusKind::Success,
            _ => StatusKind::Failure,   // non-zero OR None (finished without a captured code)
        },
    }
}

/// The indicator glyph + color for a kind (R42) — distinct glyph AND color (reads in grayscale).
pub fn status_indicator(kind: StatusKind, colors: &ThemeColors) -> (&'static str, Hsla) {
    match kind {
        StatusKind::Running => ("○", colors.border),
        StatusKind::Success => ("✓", colors.success),
        StatusKind::Failure => ("✕", colors.danger),
    }
}
```
- `BlockState`/`ExitCode` from `marley_terminal` (re-exported, both `Copy` → by value); `ThemeColors`
  from `marley_ui_components`; `Hsla` from `gpui`.

### SHIM — `crates/marley_app/src/app.rs` (~760, mutants::skip region)
Replace `pane.child(block.command.clone())` with a header row (in scope: `colors` =
`self.theme.colors.clone()` : `ThemeColors`):
```rust
let kind = exit_status_kind(block.state, block.exit_code);
let (glyph, gcolor) = status_indicator(kind, &colors);
let header = div().flex().flex_row().gap_2()
    .border_t_1().border_color(colors.border)     // separator between blocks
    .child(div().text_color(gcolor).child(glyph))
    .child(div().font_weight(FontWeight::MEDIUM).text_color(colors.foreground).child(block.command.clone()));
pane = pane.child(header);
```
Output rows unchanged (#31 colored spans). The exact border/gap/weight is shim styling.

### `crates/marley_app/src/lib.rs` — add `mod block_status;`

### File manifest
- A `crates/marley_app/src/block_status.rs` — the pure module + tests.
- M `crates/marley_app/src/lib.rs` — `mod block_status;`.
- M `crates/marley_app/src/app.rs` — the header-row render (imports `block_status::{exit_status_kind,
  status_indicator}`).
- M `docs/specs/SPEC-app-shell.spec.md` — R42 + AC + Mutation-Targets. CHANGELOG; arch doc.

### Mutation Targets
- `exit_status_kind` — the `Pending|Running` vs `Finished` arm; the `Some(0)` (the `0` literal +
  `Some` vs `_`). Killed by: `Running→Running`, `Pending→Running`, `Finished+Some(0)→Success`,
  `Finished+Some(1)→Failure`, `Finished+None→Failure`, AND `Running+Some(0)→Running` (state-first —
  kills a mutant that reads exit before state). A `Some(0)→Some(1)` mutant flips both the 0 and 1
  fixtures → caught. Arm-delete → non-exhaustive → unviable.
- `status_indicator` — the 3 arms (glyph + color). Killed by asserting each kind → the EXACT glyph +
  the EXACT ThemeColors role (a glyph→"" or a color-role swap is caught).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `kind_by_state_and_exit` — Running/Pending→Running; Finished+Some(0)→Success; Finished+Some(1)/Some(-1)/None→Failure; Running+Some(0)→Running (state-first) | unit |
| REQ-002 | `indicator_glyph_and_color_per_kind` — vs `default_for(Dark)`: Running→("○",border), Success→("✓",success), Failure→("✕",danger) | unit |
| REQ-003 | the header render (indicator + command + separator) | shim + masked multi-block visual (green ✓ + red ✕) — chad-verified |
| REQ-004 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs header div layout (shim exclude; needs a live window).

### Risks / decisions
- D-2.1 `exit.0` match with `Some(0)→Success`, `_→Failure` folds non-zero AND None into Failure —
  simplest + the None case (finished w/o code) is an anomaly, safest as Failure not a fake Success.
- D-2.2 State-checked FIRST (a Running block never shows Success even if a stale `Some(0)` lingers) —
  tested explicitly.
- D-2.3 Distinct glyph per kind (not color-only) — accessible + survives a grayscale/colorblind view.
- The header (not a wrapping card) respects the #32 per-row viewport windowing (D1) — a full card is
  deferred.

## Phase 3 — Implement
- **Built (per manifest):** `block_status.rs` (NEW) — `StatusKind{Running,Success,Failure}` +
  `exit_status_kind` (state-first; `Some(0)→Success`, `_→Failure`) + `status_indicator` (✓/✕/○ ×
  success/danger/border); `lib.rs` `mod block_status;`; `app.rs` — the header row (indicator glyph in
  its color + the command in MEDIUM weight/foreground + a `border_t` separator) replacing the bare
  `pane.child(block.command)`, importing the two pure fns. SPEC-app-shell R42 + AC row 42 +
  Mutation-Targets; CHANGELOG.
- **Deviations from design:** none.
- **Verification at this phase:** `cargo check -p marley` 0 errors; fmt; clippy `-D warnings` 0;
  docs gate 0. The block_status unit suite is Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (real-file `cargo mutants --list` + a verbatim-copy stub probe carrying the design's
  planned tests + full run). Verdict: **PASS, no MSI blocker.**
- **Mutant reality (corrects the design's worry):** the real file generates exactly **4 mutants** —
  1 UNVIABLE (`exit_status_kind → Default::default()`: StatusKind has no `Default`) + 3 viable
  (`delete Some(0) arm`; `status_indicator → ("", default)` ×2), **all killed** by the planned tests
  → MSI 100, `missed.txt` empty. cargo-mutants does NOT generate `Some(0)→Some(1)`, arm-body swaps,
  or `|`-arm deletion (non-exhaustive → not generated) — the design over-worried; the tests kill the
  real set. block_status is NOT in the shim-exclude, so the cov/MSI-100 target genuinely applies.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | LOW | Failure glyph `✕` (U+2715 multiplication-X) is the least-certain to exist in Menlo (#34); `✗` (U+2717 ballot-X) is the conventional pair with `✓` (U+2713) and more commonly present. | REAL (took it — can't verify glyph coverage headless) | Failure glyph `✕`→`✗` in block_status.rs + SPEC. |
  | F2 | LOW | Running `○` in `border` (dark L 0.22 on bg L 0.09, Δ0.13) may read faint. | ACCEPTED (running is the transient/low-emphasis state; spec-sanctioned R42) | Chad can bump to a dimmed `foreground` in the #39 polish pass; noted. |
- **Verified CLEAN by the critic:** state-first (a `Running`+`Some(0)` block → Running, never
  Success — structural: `state` is the outer match); total/no-panic (`exit.0` is field access, not
  unwrap; exhaustive matches); the header SHIM (app.rs:765) calls the pure `exit_status_kind` +
  `status_indicator` with NO inline re-derivation (correct pure/shim seam); border/success/danger are
  distinct in `default_for(Dark)` so the color asserts discriminate.
- **Reachability note for P4:** `mod block_status` is PRIVATE (not re-exported) → the tests MUST be
  in-crate `#[cfg(test)]` (matches color/history/viewport/… pattern), not an external probe.
- **Lesson:** `PR-claude-status-glyphs-from-common-matched-set` — pick terminal-font glyphs from a
  commonly-present MATCHED set (✓ U+2713 / ✗ U+2717, both Dingbats) over visually-similar but
  less-available codepoints; can't verify coverage headless, so choose conservatively. For #37 (`❯`
  prompt marker) + #38 (dock icons).

## Phase 4 — Validate
- **Tests added (block_status.rs, in-crate #[cfg(test)]):** `kind_by_state_and_exit` — every combo:
  Running/Pending→Running, Running+Some(0)→Running (state-first — kills read-exit-first), Finished+
  Some(0)→Success, Finished+Some(1)/Some(-1)/None→Failure; `indicator_glyph_and_color_per_kind` —
  each kind → exact (glyph, ThemeColors role) vs `default_for(Dark)` (○/✓/✗ × border/success/danger).
- **Runs (actual):** `cargo nextest run -p marley` → 96 passed (both new tests PASS);
  `cargo nextest run --workspace` → 499 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **3 caught /
  0 missed → MSI 100.0%** (exactly the critic's 3 viable mutants). Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `app_shell.md` — the block-chrome bullet
  (header + indicator + the pure block_status seam). SPEC-app-shell R42 at implement.
- **Knowledge captured:** `PR-…-status-glyphs-from-common-matched-set` (the ✕→✗ glyph fix — pick
  from a common matched Dingbats set, can't verify coverage headless). aar-submit `completed` (score
  5). Win: the critic ran a REAL cargo-mutants and corrected the design's over-worry — the actual
  mutant set is just 3 viable (not the imagined Some(0)→Some(1)/arm-swaps), all killed first try →
  MSI 100. The pure decision (state-first) + the tested indicator make the signature look verifiable
  without a headed harness.
- **Ticket:** forge #36 → done; local doc → closed/; pipeline pair archived. 3 of 6 in M1.E.
