# the cockpit as top-bar icons (M7) — Notes

- **Forge ticket:** #134 `696ea5d0-ccd2-4fa8-8a2b-89140e2f68d0` · **AAR:** `aef07d46-eac7-4321-bcbf-fd806c876cc6`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-134-cockpit-icons.md

## Phase 1 — Plan
- **Request:** forge #134 (M7 run 3/5) — cockpit tabs → top-bar icons (section_icon pure + reuse top_tabs).
- **Pre-flight:** the #126 cockpit block (app.rs, top:4 right:16, `.child(tab.label)`); top_tabs (#126, tested);
  RightSection enum (right_dock.rs).
- **Decisions:** D1 section_icon distinct glyph per section, reuse the click; D2 active accent / inactive muted.
- **AAR id:** `aef07d46-eac7-4321-bcbf-fd806c876cc6`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **right_dock.rs (PURE):** section_icon(RightSection)->&str: Details 📋, Agents 🤖, Forge 🔨 (distinct).
- **app.rs (SHIM):** #126 cockpit block — .child(tab.label)→.child(section_icon(tab.section).to_string()); text_size 12→15; active accent/inactive muted + click unchanged. Import section_icon.
- **Test plan:** section_icon_distinct (each section→its glyph; all 3 differ).
- **Risks:** import section_icon; glyph width/gap in the cluster.

## Phase 3 — Implement
- **Built (right_dock.rs PURE):** section_icon(RightSection)->&str (Details 📋 / Agents 🤖 / Forge 🔨, distinct). **(app.rs SHIM):** the #126 cockpit block renders .child(section_icon(tab.section)) (text_size 15) instead of tab.label; active accent/inactive muted + click unchanged. Import section_icon.
- **NOTE:** TopTab.label is now unused by the render but kept (pub, still meaningful data — e.g. a future tooltip); no dead_code warning (pub field).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a glyph mapping + a render swap).
- **Lenses — no findings:** section_icon = a distinct emoji per section (3 arms). The cockpit render swaps the text label for the glyph; the click (right_section + docks[1]=Open + persist_right_section) is unchanged; active=accent / inactive=muted preserved. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** section_icon_distinct (each section→its glyph; all 3 differ). Pass.
- **Self-test:** LIVE capture (below) — the cockpit as icons in the top-bar right.
- **Gate:** (running).

## Phase 4 addendum — active-state fix
- Emoji glyphs ignore text_color, so the "active accented" (REQ-002) was invisible with just the color swap. Added a rounded bg pill for the active icon (colors.border) vs the bare bar (colors.surface) for inactive. Re-gated GREEN.

## Phase 5 — Complete
- CHANGELOG; forge #134 → done. **M7 3/5.** section_icon (pure) + the cockpit as a top-bar icon cluster (active = bg pill). cov/MSI 100.
