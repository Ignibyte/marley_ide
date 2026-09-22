# Warp command-block styling — Notes

- **Forge ticket:** #217 (2c09c9bc-f7b4-4a29-9cf8-2f6233b92ef0)
- **AAR:** 8ca6ef52-b09e-48b0-b885-f814a707bebe
- **Local ticket doc:** docs/planning/tickets/open/TICKET-217-warp-block-styling.md
- **Pipeline spec:** warp-block-styling.spec.md

## Phase 1 — Plan
- **Request:** match Warp's command-block look. Auto-approved (/work 195-222).
- **Classification / tier:** work pipeline, small/bounded. Systems: app.rs block render (shim).
- **Discovery:** the block affordances (copy_cmd/copy_out/rerun, app.rs ~4404-4456) are ALWAYS added
  as header children — only `.hover(|d| d.text_color(accent))` brightens the text; they're always
  VISIBLE. Warp reveals block actions on hover. → the genuine delta = hover-reveal. gpui group-hover
  API to confirm at design (grep didn't conclusively find it in the gpui source — verify).
- **Decisions:** D1 group-hover reveal (fallback: per-block hovered flag); D2 actions still reachable
  via the #175 right-click menu; D3 auto-approved, document with captures.

## Phase 2 — Design
**gpui APIs verified:** `.group(name)` (div.rs:601), `.group_hover(name, |StyleRefinement| ...)`
(div.rs:680), `.opacity(f32)` (styled.rs:652). gpui has a `Visibility` enum internally but NO fluent
`.invisible()` — so opacity is the reveal mechanism.

**Mechanism.** (1) The block HEADER div gets `.group("block-actions")` — a shared group name;
`group_hover` scopes to the nearest ancestor group, so each block's affordances respond to their OWN
header's hover (no cross-block bleed). (2) Each affordance div (`copy_cmd`/`copy_out`/`rerun`,
app.rs ~4404-4456) gets `.opacity(0.0)` by default + `.group_hover("block-actions", |s| s.opacity(1.0))`
to reveal on block-hover; the existing `.hover(|d| d.text_color(accent))` (direct-hover brighten) stays.
Fallback if `StyleRefinement` lacks `.opacity()` at compile: default `text_color(muted.opacity(0.0))` +
`group_hover(|s| s.text_color(muted))`.

**Blind-click:** an opacity-0 affordance is still hit-testable, but a non-issue — you can't click one
without moving the mouse onto it, which hovers the block and reveals it first. The #175 right-click block
menu remains the always-available path to the same actions.

**Architecture.** A shim render change only (the block-header render, `#[cfg_attr(test, mutants::skip)]`
`fn render`). No new types/logic. §14 clean.

**File manifest.**
- `crates/marley_app/src/app.rs` — the block-header render: `.group("block-actions")` on the header div
  (~4368); each of `copy_cmd`/`copy_out`/`rerun` (~4404-4456) gains `.opacity(0.0)` + `.group_hover(...1.0)`.

**Regression Test Plan.**
| REQ | test |
|---|---|
| REQ-001 | driven capture — a block at rest = clean header (no ⧉/↻ affordances visible) |
| REQ-002 | driven capture — hover a block → the affordances appear; they still copy/rerun (also via #175 menu) |
| REQ-003 | review + capture — the status glyph / command / separator unchanged |
- No unit test: shim render (opacity/group-hover — no pure logic); validated by driven capture (like #216/#192).

**Risks.**
- **R1** — group name shared ("block-actions"): verified group_hover scopes to nearest ancestor → per-block correct.
- **R2** — opacity vs the direct `.hover` text_color: when a block is hovered, group_hover sets opacity 1;
  a direct affordance hover additionally brightens the text — both compose (opacity is separate from text_color).

## Phase 3 — Implement
- Block header (app.rs ~4371): added `.group("block-actions")`.
- The 3 affordances (copy_cmd/copy_out/rerun): added `.opacity(0.0)` + `.group_hover("block-actions", |s| s.opacity(1.0))`
  (replace_all on the shared `.text_color(muted).hover(...)` — verified EXACTLY 3 occurrences, no bleed).
- `.opacity()` compiled on the `group_hover` StyleRefinement closure (no fallback needed).
- `cargo fmt` + `cargo check -p marley` clean.

## Inspect (Phase 3.5)
Proportionate self-review (shim render change, gpui mechanism pre-verified). Lenses:
- **Correctness/scope** — `grep -c group_hover("block-actions"` = 3 (the affordances only); the header
  `.group()` once. group_hover scopes to the nearest ancestor group → each block independent (no bleed).
- **Function preserved** — the affordances keep their `on_mouse_down` (copy/rerun still fire when
  revealed); the #175 right-click block menu (the always-available path) is untouched.
- **Compose** — opacity (reveal) is orthogonal to the direct-hover text_color brighten; both apply.
- **Clean-room** — a gpui idiom (group-hover reveal), no Warp asset.
No findings.

## Phase 3.5 — Inspect
- …

## Phase 4 — Validate
- **Driven capture (REQ-001, at rest):** `scratchpad/217-atrest2.png` — an `echo hello` block with the
  mouse parked on the sidebar. The block header reads `▾ ✓ echo hello` with a CLEAN right edge — NO
  ⧉/↻ affordances. (Every prior capture #196/#216 showed `↻ run ⧉ cmd ⧉ out` always-on.) PASS.
- **Driven capture (REQ-002, hover):** `scratchpad/217-hover.png` — mouse moved onto the same block
  header. `↻ run  ⧉ cmd  ⧉ out` now appear at the far-right of the header (opacity 0→1 via group-hover).
  The affordances keep their `on_mouse_down` (copy/rerun still fire) + the #175 right-click menu is the
  always-on path. PASS.
- **REQ-003 (glyph/command/separator unchanged):** the `✓` status glyph, the `echo hello` command text,
  and the `border_t` block separator are byte-identical between the at-rest and hover captures — the
  change is purely the affordance opacity. PASS.
- **Tests:** `cargo nextest run -p marley` — GREEN (no new unit test; shim render validated by the driven
  captures per the design, like #216/#192).
- **Gate:** `scripts/gates.sh --diff` (staged) — GREEN (receipt written).

## Phase 5 — Complete
- …
