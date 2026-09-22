---
pipeline_id: 44748a1f-858c-47f6-9ebf-18426c95cdd1
ticket: forge#25 (9668f539-15ec-4237-ad5e-c67ecc724126) · local docs/planning/tickets/open/TICKET-025-palette-dispatch.md
aar_id: 37a1fd65-6965-43d2-90b4-bec585b455b7
status: Phase 5 — Complete PASS
title: palette dispatch — ↑/↓ selection + Enter actually runs the command
type: feature
milestone: M1.C
references:
  - docs/specs/SPEC-app-shell.spec.md (R14/R17/R18/R23; gains the selection-model clause R32)
  - docs/specs/SPEC-command-palette.spec.md (RECONCILED — it is the M2 marley_search_core spec [Query/SearchMixer/fuzzy_rank]; it does NOT spec the M1 selection model, so R32 is new SPEC-app-shell territory, no fork)
  - crates/marley_app/src/palette.rs (filter_commands — the list the selection runs over)
---

## Title
Make R17 real. Today Enter just closes the palette overlay and NO command dispatches — "Toggle
Theme" has never actually switched a theme. Build the PURE `PaletteState` selection model
(↑/↓ over the filtered list, clamp at the ends, selection resets on query edit, activate → the
SELECTED command) and wire activation into `dispatch_action` for the real cockpit verbs —
including the FIRST runtime theme switch (R23's `set_theme` finally has a caller). Selection
highlight + per-row chord chips render via the #17 widget vocabulary.

## Scope
### In
- `crates/marley_app/src/palette.rs` — the PURE `PaletteState` (query + selected index over the
  filtered length): `move_up`/`move_down` (clamp at ends), `reset_selection` on query edit,
  `activate(&[ScoredCommand]) -> Option<CommandId>` (None on an empty list), index kept in
  bounds after a shrinking edit; + the PURE `action_for_command(CommandId) -> Option<&'static str>`
  dispatch table (split-pane / close-pane / toggle-theme / toggle-left-dock / toggle-right-dock).
- `crates/marley_app/src/app.rs` (shim) — `handle_palette_key` grows ArrowUp/ArrowDown/Enter
  routing through PaletteState; Enter dispatches the activated verb then dismisses (empty list →
  dismiss only); `toggle_theme_state()` (flip via `ThemeRegistry::builtin().default_for(opposite
  appearance)` — the caller notifies, the `set_theme(_, cx)` public API keeps R23's contract);
  the overlay renders the selection-highlight row (accent/surface from ThemeColors) + per-row
  chord chips via `KeyboardShortcut::parse`/`keys`.
- "Open Settings" settled: DROPPED from the M1 list (design confirms) — no settings UI exists
  until seq-5/M2; an inert command is the exact lie this ticket removes.
- SPEC-app-shell: NEW R32 (selection model) + R17 note (activation = the SELECTED command) +
  AC/Test-Plan/Mutation-Targets rows.
- CHANGELOG + architecture doc (§21).

### Out (explicitly deferred)
- `marley_search_core`/SearchMixer (M2 — SPEC-command-palette's actual scope); palette
  scrollback/paging; mouse selection in the overlay; settings UI (seq-5/M2); headed baselines
  (the #23 environmental deferral — the palette-selection baseline joins the desktop-session
  follow-up list).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Selection CLAMPS at the ends (no wrap) — deterministic, one less arithmetic edge; wrap is
  a future UX call.
- D2 — Selection resets to the TOP on every query edit (the filtered list changed under it).
- D3 — The dispatch table is a pure fn in palette.rs (a wrong-verb mutant per row must die);
  app.rs matches the returned action through the SAME `dispatch_action` the keymap uses (one
  verb router).
- D4 — Theme toggling flips by APPEARANCE via `default_for` (registry-driven; by_name stays for
  seq-5's persistence).
- D5 — "Open Settings" is REMOVED from `cockpit_commands` (ids stay stable for the survivors).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the palette is open, ArrowDown/ArrowUp shall move the selected index over the CURRENT filtered list clamping at both ends; WHEN the query is edited, the selection shall reset to the top; WHEN the filtered list shrinks below the selected index, the selection shall stay in bounds. | PaletteState unit tests (clamp both ends, reset-on-edit, shrink-rebound) |
| REQ-002 | WHEN Enter activates a non-empty filtered list, the system shall dispatch the SELECTED command's verb exactly once and dismiss the overlay; WHEN the filtered list is empty, Enter shall dismiss with NO dispatch; Escape semantics (R18) shall be unchanged. | PaletteState::activate unit tests + dispatch-table tests; shim review |
| REQ-003 | WHEN the toggle-theme verb dispatches, the system shall switch the active theme to the OPPOSITE appearance's default (dark↔light) so `active_theme()` changes (R23) and every region repaints with the new ThemeColors. | pure appearance-flip decision unit-tested; `set_theme`/`active_theme` contract (existing R23/R24 coverage); shim review |
| REQ-004 | WHEN `action_for_command` is queried, the system shall map every surviving CommandId to its exact verb (split-pane, close-pane, toggle-theme, toggle-left-dock, toggle-right-dock) and an unknown id to None. | per-row unit assertions (a wrong-verb mutant per row dies) |
| REQ-005 | WHEN `scripts/gates.sh --diff` runs over the staged change, every gate shall be GREEN with coverage 100%/MSI 100% on the touched pure surface. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — PaletteState exact shape (does it OWN the query string or borrow the filtered
  len?), the appearance-flip pure seam, chip render shape, R32 text.
- **P3 Implement** — palette.rs + app.rs + spec + CHANGELOG.
- **P3.5 Inspect** — critics (2): selection/index edges + dispatch wiring & theme flip.
- **P4 Validate** — tests + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #25.
