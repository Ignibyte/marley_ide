# B0b: Typing and clicking in the page — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-489-browser-input.md
- **Pipeline spec:** 489-browser-input.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the input half of B0 (the amendment's questions 1, 2 and 5).
- **Classification:** feature; `marley_browser` (the mappings) and `marley_workbench` (the
  tab's handlers). No Zed path.
- **Recall (§18.3):**
  - L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001: on Linux,
    `prefer_character_input` is always false; stopping a key's propagation keeps it from text
    input.
  - PR-claude-raw-input-passthrough-must-filter-platform-chords-001: a new passthrough filters
    the platform chords the other routes filter.
  - The Explore report on gpui's Linux input (compose order, the text step, text-input-v3's
    one-byte commits, `bounds_for_range` in window coordinates).
- **Discovery:** the CDP probe's key, composition, wheel and iframe results.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.
