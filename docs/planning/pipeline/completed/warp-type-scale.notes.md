# Warp type-scale calibration — Notes

- **Forge ticket:** #195 (9377f953-94ed-41de-80af-307095b55bb8)
- **AAR:** 0ef796a3-e28e-4b95-bb78-283b96c83c5e
- **Local ticket doc:** docs/planning/tickets/open/TICKET-195-warp-type-scale.md
- **Pipeline spec:** warp-type-scale.spec.md

## Phase 1 — Plan
- **Request:** calibrate the type scale to match Warp's font proportions (the font half of
  the Warp-look review; #194 did colors). Auto-approved (/work 195–222).
- **Classification / tier:** work pipeline, small. Systems: `marley_app::typography`
  (`type_scale`, pure) + `app.rs` (`TERMINAL_FONT_SIZE` const; the cell metric derives from it).
- **Discovery (current values):**
  - `typography.rs type_scale`: Command 14.0/Medium, Output 14.0/Normal, Caption 12.0/Normal.
  - `app.rs`: `TERMINAL_FONT_SIZE = 14.0`, `TERMINAL_FONT = "Menlo"`.
  - The cell metric = `fallback_cell(TERMINAL_FONT_SIZE)` + the live em_advance read (app.rs
    ~4123) — DERIVES from the size, so it AUTO-TRACKS a size change (no separate edit).
  - `type_scale` is already exact-value tested (`type_scale_by_role`) — extend those asserts.
  - Warp reference: Warp.app is the host terminal → screencapture its window (the selftest
    mechanism) to measure Warp's terminal + chrome text sizes.
- **Decisions:** D1 recalibrate type_scale + TERMINAL_FONT_SIZE (metric auto-tracks); D2
  measure vs a captured Warp reference; D3 exact-value type_scale pins (f32 literals aren't
  mutated); D4 legibility floor ≥12pt; D5 auto-approved, document with captures.

## Phase 2 — Design

**Warp reference captured** (`195-warp-ref.png` — Warp.app's host window, 1919×906pt @ 2×).
Measured Warp's type scale: terminal output text ≈ 13pt (line height ≈ 16pt); sidebar session
labels ≈ 12–13pt; the dim group headers ("Forge"/"Dev 1") ≈ 10–11pt caption. Warp runs ~1pt
DENSER than Marley (Marley is 14/14/12), consistent with Warp's 13pt default.

**Proposed calibration (measured, Marley-original):**
| token | current | proposed |
|---|---|---|
| `app.rs TERMINAL_FONT_SIZE` (terminal cell text) | 14.0 | **13.0** |
| `type_scale(Command)` | 14.0 / Medium | **13.0 / Medium** |
| `type_scale(Output)` (body) | 14.0 / Normal | **13.0 / Normal** |
| `type_scale(Caption)` | 12.0 / Normal | **11.0 / Normal** |
Rationale: match Warp's density (a ~1pt tightening); keep the terminal→caption relationship
(~1.18×); 13pt ≥ the 12pt legibility floor; weights unchanged (Warp similarly emphasizes the
command line vs output). The cell metric = `fallback_cell(TERMINAL_FONT_SIZE)` + em_advance read,
so it DERIVES from 13.0 → columns auto-stay-aligned; no separate metric edit.

**Architecture / approach.** Pure `marley_app::typography` (the `type_scale` size literals) + one
`app.rs` shim const (`TERMINAL_FONT_SIZE`). No new types, no logic — a value recalibration. §14
clean (pure table + a shim const).

**File manifest.**
- `crates/marley_app/src/typography.rs` — `type_scale`: Command 14→13, Output 14→13, Caption 12→11
  (sizes only; weights kept). Update the `type_scale_by_role` exact-value asserts to match + add the
  ≥12 floor assert.
- `crates/marley_app/src/app.rs` — `TERMINAL_FONT_SIZE` 14.0 → 13.0 (line 321). The cell metric
  auto-tracks. No render edit (spans already read `type_scale`/`TERMINAL_FONT_SIZE`).

**Regression Test Plan.**
| REQ | test |
|---|---|
| REQ-001 | `typography::tests::type_scale_by_role` updated → Command{13,Medium}, Output{13,Normal}, Caption{11,Normal} (exact-value pins; f32 literals aren't mutated, so the full-value assert is the guard) |
| REQ-002 | driven capture — `ls -l` / a table aligns at 13pt (the cell metric derives from `TERMINAL_FONT_SIZE`) |
| REQ-003 | new `type_scale_output_clears_legibility_floor`: `assert!(type_scale(Role::Output).size >= 12.0)` (kills a shrink-below-floor regression) |
| REQ-004 | review — measured/observed, original values (§20) |
- Uncoverable-by-unit → driven capture: `TERMINAL_FONT_SIZE` is a shim const (like the other app.rs
  consts); its effect (density + column alignment) is proven by the Marley capture next to the Warp ref.

**Risks / decisions.**
- **R1** — 13pt matches Warp's default but is a subtle change; auto-approved + captured, chad can bump.
- **R2** — `TERMINAL_FONT_SIZE` (app.rs) and `type_scale(Output)` (typography) are SEPARATE consts both
  now 13.0 — a future-divergence smell; consolidating them is a separate cleanup (out of scope), noted.
- **R3** — the cell metric derives from `TERMINAL_FONT_SIZE`, so columns stay aligned at 13pt — verify
  in the driven capture (a table/`ls -l`).

## Phase 3 — Implement
- `typography.rs` `type_scale`: Command 14→13 (Medium kept), Output 14→13 (Normal), Caption 12→11
  (Normal); updated the `type_scale_by_role` exact-value asserts to 13/13/11.
- `app.rs` `TERMINAL_FONT_SIZE` 14.0 → 13.0. No render edit — the cell metric derives from it, spans
  already read `type_scale`/`TERMINAL_FONT_SIZE`.
- Deviations: none.
- `cargo fmt` + `cargo check -p marley` clean.

## Phase 3.5 — Inspect
One focused general-purpose critic (consistency/consumers + clean-room/legibility) over the diff,
scaled to the 4-value change; verified concretely by grep + reading the render.

**Verdict: #195 is correct + internally coherent, no source fix.** Key confirmations:
- **Nothing stranded at 14** — `grep px\(14 app.rs` → 0 hits; the terminal fully moved to 13. (The
  only remaining 14 in the type domain is a dead, unreachable `fallback_cell` guard default in
  workspace.rs:108, const is 13>0 → else never taken.)
- **Cell metric tracks** — `fallback_cell(TERMINAL_FONT_SIZE)` + the `em_advance(px(TERMINAL_FONT_SIZE))`
  read + the prompt caret bar `px(TERMINAL_FONT_SIZE)` all derive from the one const → columns stay
  aligned at 13pt; no 14-derived height survives.
- **Right target** — `type_scale(Role::)` controls the terminal command header (13), output body (13,
  the main win), the sticky header, the block-action affordances (Caption 11), and the dock
  `caption_header` (11) — the dominant surface vs Warp.
- **Coherent** — all chrome literals are 9/11/12/13; the terminal was the lone 14 OUTLIER, now 13 =
  joint-largest, none exceeded by chrome. A flat Warp-like band.
- **Clean-room / legibility** — measured/original numbers; 13pt ≥ the 12pt floor.

| # | Finding | Verdict | Action |
|---|---|---|---|
| 1 | [LOW] the CHROME text sizes are hardcoded `px()` literals (~20) bypassing `type_scale` — typography's own doc says the opposite; a future density tune must hand-edit each | **Real, pre-existing, out of #195 scope** | **Filed as a follow-up** (route chrome through type_scale + add roles) — a warp-parity sibling to do BEFORE #216/#219 so they calibrate through type_scale |
| 2 | [LOW] the last stale `14.0` = workspace.rs:108 `fallback_cell` non-positive guard default (dead branch) + its doc | **Real, cosmetic, unreachable** | Deferred into the same follow-up (it's the cell-metric fallback, not the Warp calibration) — no functional/visual impact today |

**Net: no source change in #195** (surgical + coherent); the two LOWs are pre-existing architecture,
captured in the consolidation follow-up ticket. Lenses: consistency/consumers, cell-metric derivation,
clean-room, legibility, simplification.

## Phase 4 — Validate
**Tests:** `type_scale_by_role` updated to the new 13/13/11 exact values (the typo guard — f32
literals aren't mutated); added `type_scale_terminal_text_clears_legibility_floor` (Output+Command
≥12.0, REQ-003). RUN: `cargo nextest run -p marley type_scale` → 3 pass; `--workspace` → **780 passed, 5 skipped**.

**Live driven capture** (`195-marley-term.png`, next to the Warp reference `195-warp-ref.png`):
`ls -la` in a fresh terminal at 13pt — **all columns align** (permissions / link-count / owner /
group / right-aligned sizes / dates / filenames), proving the cell metric tracks `TERMINAL_FONT_SIZE=13`
(REQ-002); the terminal text now reads at Warp's ~13pt density (denser than the old 14pt). Legible
(REQ-003). (The `ls -la` file paths also render as underlined #196 links — that feature intact.)

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15.** cov 100 / MSI 100 on typography.rs.

**Pre-existing:** none in scope (the chrome hardcoded-`px()` literals + the workspace.rs:108 stale
`14.0` are captured in follow-up #223, not this change).

## Phase 5 — Complete
- …
