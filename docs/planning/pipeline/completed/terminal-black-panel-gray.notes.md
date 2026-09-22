# Terminal-black pane + gray panel surfaces — Notes

- **Forge ticket:** #194 (1469c76c-b4ac-4072-9d92-57ca7fe96a07)
- **AAR:** 209075eb-b1d7-4713-80b6-94078239581f
- **Local ticket doc:** docs/planning/tickets/open/TICKET-194-terminal-black-panel-gray.md
- **Pipeline spec:** terminal-black-panel-gray.spec.md

## Phase 1 — Plan
- **Request:** chad live direction (2026-07-09) "review the warp ui for font sizes and
  colors. I like the terminal area being black with the panel being the slight gray
  color." This ticket = the COLOR half (font = #195). Make the terminal pane read
  near-black and the panels a distinct gray.
- **Classification / tier:** work pipeline, one slice. Systems: `ui_components`
  (`ThemeColors::default_for(Dark)`) + `marley_app` render token assignment. Taste-gated.
- **Forge recall (§19):** docs index cross-project-polluted → relied on local memory +
  the in-repo M1.E #35 palette lessons: (a) accent lightness must clear WCAG vs
  `on_accent` (button labels); (b) cargo-mutants does NOT mutate `hsla` literals →
  guard token values with EXACT-value regression asserts (see the existing
  `palette_success_and_light_accent_pinned` test) + a contrast test; (c) `ThemeColors`
  has no `Default` → full-value asserts.
- **Discovery (current DARK tokens, ui_components/src/lib.rs `default_for(Dark)`):**
  - `background: hsla(0.62, 0.14, 0.09)` — tinted near-black (L=0.09), the window/pane fill.
  - `surface:    hsla(0.62, 0.12, 0.13)` — raised surface (L=0.13).
  - `foreground: hsla(0.62, 0.05, 0.90)` — near-white text.
  - `accent:     hsla(0.52, 0.58, 0.55)` — teal-cyan; `on_accent: hsla(0.62,0.30,0.10)`.
  - `border:     hsla(0.62, 0.10, 0.22)` · `muted: hsla(0.62,0.08,0.60)` ·
    `success: hsla(0.40,0.50,0.55)` · `danger: hsla(0.99,0.62,0.62)`.
  - Observed live (193-1-idle.png, current build): the terminal pane and the
    Workspace/Files panels both read a similar dark tone — they DON'T separate. That's
    exactly chad's complaint. The design must (a) darken the terminal pane toward black
    and (b) give the chrome a distinctly lighter gray, and confirm which token each
    render site pulls (the ANSI palette #31 resolves against these too).
- **Decisions:** D1 recalibrate existing tokens (prefer background=terminal-black +
  surface/gray=panels over a new token); D2 pure WCAG contrast guard as the seam +
  exact-value regressions; D3 keep accent/on_accent AA; **D4 TASTE GATE — chad away:
  plan+design only, propose shades, PAUSE for approval before implement.**

## Phase 2 — Design

**Token→region map (confirmed in app.rs).** The dark theme already has TWO surface
tokens and the render already splits them — just too close in lightness:
- `colors.background` (L=0.09) → the **window root** (app.rs:3212), the **terminal pane
  body** (4016), the **code-view body** (4032). This is "the terminal area."
- `colors.surface` (L=0.13) → the **chrome**: docks/`dock_panel` (3930), the sidebar
  rail, the Files panel, the prompt input row (4500), the cockpit panels, list entries,
  the footer/status strip (5260). This is "the panels."

So the separation EXISTS but is only ΔL≈0.04 → they read the same (chad's complaint).
**Mechanism = recalibrate the two existing tokens, no new token, minimal render churn:**
push `background` toward black and lift `surface` to a clearer gray so the gap becomes
ΔL≈0.10. A short render AUDIT at implement confirms every chrome region sets
`.bg(colors.surface)` (any that inherit the now-near-black `background` get switched).

**Proposed DARK palette (current → proposed):**
| token | current hsla | proposed hsla | effect |
|---|---|---|---|
| `background` (terminal/root/code) | (0.62, 0.14, 0.09) | **(0.62, 0.16, 0.05)** | near-black terminal; keeps a whisper of the cool Marley tint |
| `surface` (panels/chrome/raised) | (0.62, 0.12, 0.13) | **(0.62, 0.09, 0.155)** | a distinct, more-neutral gray — clearly lighter than the terminal |
| `border` | (0.62, 0.10, 0.22) | **(0.62, 0.10, 0.26)** | separators still read on the darker surfaces |
| `foreground`, `accent`, `on_accent`, `danger`, `success`, `muted` | — | **unchanged** | keep identity; near-white text & teal accent gain contrast on black |

Separation becomes L 0.05 (terminal) vs 0.155 (panels) — a clear ~3× lightness step.

**Contrast check (approx WCAG; the pure helper exact-tests it at validate):**
| pair | ratio (approx) | bar |
|---|---|---|
| foreground on terminal-bg | ~17:1 | AAA 7:1 ✓ |
| foreground on panel-surface | ~12:1 | ✓ |
| muted (caption) on panel-surface | ~4.6:1 | AA ✓ |
| accent (teal) on terminal-bg | high | ✓ |
| accent vs on_accent (button labels) | unchanged (M1.E #35 AA held) | re-verify ✓ |
| each #31 ANSI color on terminal-bg | brighter than today | **ANSI "black"/"bright-black" is the one to watch** — the contrast test flags it if it dips below the min (a #31-palette follow-up if so, not this ticket) |

**PURE seam (testable, cov/MSI 100).** Add `contrast_ratio(a: Hsla, b: Hsla) -> f32`
(+ `relative_luminance(Hsla) -> f32`) in `ui_components` — WCAG relative luminance
(sRGB→linear gamma decode + 0.2126/0.7152/0.0722, ratio `(Lmax+0.05)/(Lmin+0.05)`).
This is real mutation-rich math (coefficients, the +0.05, gamma threshold, max/min) and
it PROVES REQ-003 rather than eyeballing it. `hsla` literals aren't mutated by
cargo-mutants (M1.E #35), so the token values themselves are guarded by exact-value
regression asserts (as the existing `palette_success_and_light_accent_pinned` does).

**File manifest.**
- `crates/ui_components/src/lib.rs` — set the 3 dark tokens in `default_for(Dark)`;
  update the exact-value asserts (`default_for_light_differs_from_dark`,
  `palette_success_and_light_accent_pinned`); ADD `relative_luminance` + `contrast_ratio`
  + their tests.
- `crates/marley_app/src/*` — render AUDIT only: confirm each chrome region uses
  `colors.surface`; switch any that inherit `background` (determined at implement). No
  logic change.

**Regression Test Plan.**
| REQ | test |
|---|---|
| REQ-001 | `default_for(Dark).background == hsla(0.62,0.16,0.05)` + `L(background) < L(surface)` (ordering) + driven capture (terminal near-black) |
| REQ-002 | `default_for(Dark).surface == hsla(0.62,0.09,0.155)` + driven capture (panels a distinct gray) |
| REQ-003 | `contrast_ratio(foreground, background) >= 7.0`; `contrast_ratio` known values (black↔white ≈ 21, equal ≈ 1); each #31 ANSI vs background ≥ min |
| REQ-004 | LIGHT theme values unchanged (existing `default_for_light_*` regressions) |
| REQ-005 | review — values are Marley-original (§20) |
- Uncoverable-by-unit → driven capture (the rendered look): REQ-001/002 visual half.

**Risks / decisions.**
- **R1 — `surface` is shared** by chrome AND in-terminal raised bits (the prompt input
  row, block hover tint, dialogs). Lifting it grays the prompt-input strip inside the
  black terminal. That's a Warp-like look (a raised input on black) and I think it reads
  well — but it's a TASTE call, flagged for chad. If he wants the prompt strip to stay
  near-black, we'd add a separate token (more churn) — decide on his feedback.
- **R2 — ANSI "black" on near-black** may dip below the min contrast; the REQ-003 test
  surfaces it. Fixing the ANSI palette is a #31 follow-up, out of this ticket's scope.
- **R3 — accent/on_accent** unchanged, but re-assert AA so darkening didn't regress it.

**TASTE GATE (D4) — PAUSED HERE.** These are proposed values, grounded in chad's
direction + the current palette; nothing implemented or committed. Awaiting his approval
of the shades (or a "render it so I can see" → I'll apply + live-capture the real app
before any commit).

## Phase 3 — Implement (PREVIEW — held for chad's shade approval)
- Applied the proposed dark shades in `ui_components/src/lib.rs` `default_for(Dark)`:
  `background` → hsla(0.62,0.16,0.05) (near-black), `surface` → hsla(0.62,0.09,0.155)
  (gray), `border` → hsla(0.62,0.10,0.26); foreground/accent/on_accent/danger/success/
  muted UNCHANGED. Updated the one exact-value assert (`d.background`).
- Render audit: NO render edits needed — panels pull `colors.surface`
  (`dock_panel`:360, `files_panel`:3930), terminal + root pull `colors.background`.
- `cargo fmt` + `cargo check -p marley` clean.
- **Deferred to post-approval (finish implement + validate):** the pure
  `contrast_ratio()`/`relative_luminance()` helper + its tests + the REQ-002
  surface/ordering asserts + the light-theme regressions.
- **Live-captured:** `194-2-terminal.png` (terminal tab) — terminal pane near-black,
  Workspace/Files panels a distinct gray, separation ΔL 0.04 → ~0.10. `194-1-proposed.png`
  (code tab) shows the same on the code view. Matches chad's ask.
- **HELD** for chad's visual approval of the shades before finalize (contrast test →
  inspect → validate → commit). If he nudges: adjust the 3 hsla values + re-capture.
- **APPROVED — chad (2026-07-09): "this is looking really good actually."** Shades LOCKED.
  Finished implement: added the pure `relative_luminance(Hsla) -> f32` +
  `contrast_ratio(Hsla, Hsla) -> f32` helpers in `ui_components/src/lib.rs` (WCAG relative
  luminance via `gpui::Rgba` + the gamma piecewise; `cargo check -p marley_ui_components`
  clean). → inspect → validate (contrast tests + REQ-002 surface/ordering asserts + ANSI
  capture) → commit.

## Phase 3.5 — Inspect
Two independent general-purpose critics over the diff (lib.rs only): (A) WCAG-math
correctness + legibility (COMPUTED every ratio); (B) reuse / clean-room / mutation
(ran `cargo mutants --list`, enumerated the full mutant set + a killing input each).

**Verdict: correct, no defects.** All text pairs clear their WCAG bar; nothing regressed
below a bar. Ratio table (NEW): foreground/background **15.53** (AAA), foreground/surface
12.01, muted/surface **5.08** (AA, the canary — surface lightened toward muted, margin 0.58),
accent/background 8.97, accent/on_accent **8.27 unchanged** (M1.E #35 button-label lesson
holds), success/bg 9.07, danger/bg 5.54. Reuse: no existing contrast helper (not a dup).
Conversion: `gpui::Rgba::from(Hsla)` gives straight 0..1 sRGB (verified in gpui source).
Clean-room: no Warp asset/hex — the tokens are our own `hsla` fractions (§20 ✓).

| # | Finding | Verdict | Action |
|---|---|---|---|
| 1 | [HIGH-mut, B] mutants #9/#10 in the `c/12.92` near-black branch SURVIVE unless a very-dark NON-black color is tested (all tokens' channels are >0.03928 → hi-branch; pure black `c=0` doesn't distinguish `/`vs`*`vs`%`) | **Real (test-design)** | Validate MUST assert `relative_luminance(hsla(0.,0.,0.02,1.)) ≈ 0.001548` |
| 2 | [MED, B] the changed `surface` + `border` dark tokens are NOT pinned by exact-value asserts (only `background` is); hsla literals aren't mutated, so an exact assert is the only typo guard | **Real** | Validate adds `assert_eq!(d.surface, hsla(0.62,0.09,0.155,1.))` + border |
| 3 | [LOW, A] muted/surface is the tightest text pair (5.08) and the diff moved it toward the bar | **Real (defensive)** | Validate adds a regression `assert!(contrast_ratio(d.muted, d.surface) >= 4.5)` |
| 4 | [LOW-nit, A] `relative_luminance` ignores alpha (misleading for a reusable helper; app has translucent tints) | **Real** | **FIXED now** — doc note "opaque input only" |
| 5 | [LOW, A] panel/border non-text separation < WCAG 1.4.11 3.0 (surface/bg 1.29, border/bg 1.87) | **Rejected (scope)** | NOT caused by this diff (surface/bg improved 1.11→1.29); decorative separators exempt; the pane fill delta + teal focus border delineate. No fix. |
| 6 | [nit, A] threshold 0.03928 is WCAG 2.0 (2.1 uses 0.04045) | **Rejected** | Numerically negligible (deep in the toe); 2.0 is a valid standard |

**MSI-100 killing set (from critic B, for validate):** white→1.0, black→0.0, mid-gray(0.5)→0.2140,
**near-black(0.02)→0.001548**, contrast(white,black)→21.0, contrast(white,mid-gray)→3.977, +
a real-world sanity `contrast(d.foreground, d.background)=15.53`. No equivalent/unkillable mutants
in the emitted set (the would-be-equivalent `>=`→`>` wasn't generated; `>=`→`<` is killable).
**Lesson to capture at complete:** a pure fn with a piecewise branch needs a test input that
reaches EACH branch with a *distinguishing* (non-degenerate) value, else the rarely-hit branch's
op-mutants survive — the near-black `c/12.92` branch here.

## Phase 4 — Validate
**Tests added** (`ui_components/src/lib.rs mod tests`) — the critic's exact MSI-100 killing set:
- `relative_luminance_wcag_known_values` — white→1.0, black→0.0, mid-gray(0.5)→0.2138,
  **near-black(0.02)→0.001548** (the load-bearing one: the only non-degenerate input on the
  `c/12.92` branch — kills mutants #9/#10).
- `contrast_ratio_wcag_known_values` — (white,black)=21, (black,white)=21 (order-independent),
  equal=1, (white,mid-gray)≈3.977.
- `dark_194_shades_pinned_and_legible` — REQ-002: exact-value pins for `surface`+`border` +
  `L(background) < L(surface)`; REQ-003: `contrast_ratio(fg,bg) ≥ 7` (15.53),
  `accent,on_accent ≥ 4.5` (8.27), the canary `muted,surface ≥ 4.5` (5.08).
- REQ-004: light theme unchanged (existing `default_for_light_*` regressions).

**Tests RUN:** `cargo nextest run -p marley_ui_components` → 32 passed; `--workspace` → **771 passed, 5 skipped**.

**Live driven capture (new theme, on the real app):**
- `194-2-terminal.png` — terminal pane **near-black**, Workspace/Files panels a **distinct gray** (REQ-001/002 ✓).
- `194-3-ansi.png` — code view: syntax-highlight colors (green comments, colored keywords/strings) legible on near-black.
- `194-4-ls.png` — a terminal ran `ls -G /`: **ANSI blue directory names legible on the near-black** bg (REQ-003 ANSI ✓); the ❯ prompt returned after the command (also re-confirms #193). ANSI gained contrast (bg darkened) — no illegible ANSI color; no #31 follow-up needed.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15.** gate:5 mutation MSI 100%
(all ~33 helper mutants killed, incl the near-black branch), gate:4 coverage 100%, gate:15 visual/AX.

**Pre-existing:** none (only the unrelated upstream `block v0.1.6` dep warning).

## Phase 5 — Complete
- …
