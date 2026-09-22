# palette dispatch — Notes

- **Forge ticket:** #25 `9668f539-15ec-4237-ad5e-c67ecc724126`
- **AAR:** `37a1fd65-6965-43d2-90b4-bec585b455b7`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-025-palette-dispatch.md
- **Pipeline spec:** palette-dispatch.spec.md

## Phase 1 — Plan
- **Request:** forge #25 (M1.C seq-4, auto-approved run) — selection model + real Enter dispatch.
- **Classification / tier:** work pipeline, `feature` — one slice (palette.rs pure additions +
  the app.rs overlay/dispatch shim + spec).
- **Forge recall (§18.3):** bulletins none. RECONCILE CHECK done (the ticket's mandate):
  SPEC-command-palette.spec.md is the M2 `marley_search_core` spec (Query/QueryFilter/SearchMixer/
  fuzzy_rank/tantivy, R1–R20) — it does NOT spec the M1 palette selection model → new
  SPEC-app-shell R32, no fork. Priors: the dispatch-table-per-row mutation pattern; the
  caller-notifies dispatch convention (#24's toggle_dock_state); the #23/#24 headed deferral.
- **Discovery:** palette.rs owns Command/CommandId/ScoredCommand/filter_commands (pure);
  handle_palette_key currently folds Enter into "escape|enter → close"; `Dialog::content_summary`/
  `KeyboardShortcut::parse→keys()` are the #17 pure surfaces (their gpui render halves live in
  ui_components/src/render/ — shim); `ThemeRegistry::default_for(Appearance)` is the flip path;
  `cockpit_commands` ids {0,1,2,4,5} survive, 3 ("Open Settings") drops (D5).
- **Decisions:** D1–D5 in the spec (clamp not wrap; reset-on-edit; pure dispatch table routed
  through the ONE dispatch_action; appearance-flip via default_for; Open Settings removed).
- **Open questions for Design:** PaletteState owning the query (moving palette_query into it) vs
  a thin index-only model over (query, len) — ownership is cleaner for reset-on-edit invariants;
  whether `activate` takes the filtered slice or a len + index lookup at the call site; the
  chip-render shape in the overlay (KeyboardShortcut::keys → per-key chip strings).
- **AAR id:** `37a1fd65-6965-43d2-90b4-bec585b455b7`.

## Phase 2 — Design

### Architecture / approach
PURE (palette.rs):
```rust
pub struct PaletteState { query: String, selected: usize }   // owns the query (D-2.1)
impl PaletteState {
  pub fn new() -> Self                       // empty query, selected 0 (palette-open resets)
  pub fn query(&self) -> &str
  pub fn selected(&self) -> usize            // render highlight index
  pub fn push(&mut self, text: &str)         // append + selection reset (R32 reset-on-edit)
  pub fn backspace(&mut self)                // pop + selection reset
  pub fn move_up(&mut self)                  // saturating_sub(1)
  pub fn move_down(&mut self, filtered_len: usize)  // min(selected+1, len.saturating_sub(1)); len 0 → 0
  pub fn activate(&self, filtered: &[ScoredCommand]) -> Option<CommandId>
  // = filtered.get(self.selected).map(|s| s.command.id) — `.get` handles BOTH the empty list
  // (None — the tested REQ-002 arm) and the never-reachable OOB uniformly, with NO dead clamp
  // (the invariant: every edit resets selection; the list only changes on an edit).
}
pub fn action_for_command(id: CommandId) -> Option<&'static str>
// 0→split-pane, 1→close-pane, 2→toggle-theme, 4→toggle-left-dock, 5→toggle-right-dock, _→None
// (id 3 was "Open Settings" — REMOVED per D5; the gap is deliberate + tested to map to None).
```
PURE (keymap.rs): `KeyBinding::display(&self) -> String` — "cmd-shift-b"-style (modifier order
cmd/ctrl/alt/shift + key) — feeds `KeyboardShortcut::parse` (#17) for the row chips.
PURE (themes.rs): `pub fn toggled_appearance(a: Appearance) -> Appearance` — the dark↔light flip
decision (registry lookup + clone stay in the shim).

SHIM (app.rs, existing exclude): `palette_query: String` → `palette: PaletteState` (open resets
via `PaletteState::new`); `handle_palette_key` routes escape (R18 unchanged) / enter (activate →
`action_for_command` → the ONE `dispatch_action` verb router → dismiss; empty list → dismiss
only) / "up"/"down" (move) / backspace / chars; `dispatch_action` gains `"toggle-theme"` →
`toggle_theme_state()` (= `ThemeRegistry::builtin().default_for(toggled_appearance(current))
.clone()` — caller notifies, `set_theme(_, cx)` keeps R23's public contract); the overlay renders
per-row selection highlight (accent bg + `on_accent` text when `i == palette.selected()`) and a
right-side chord chip text via `KeyboardShortcut::parse(&binding.display()).keys()` joined;
`cockpit_commands` drops "Open Settings".

### File manifest
- M `crates/marley_app/src/palette.rs` — PaletteState + action_for_command + tests.
- M `crates/marley_app/src/keymap.rs` — `KeyBinding::display` + tests.
- M `crates/marley_app/src/themes.rs` — `toggled_appearance` + test.
- M `crates/marley_app/src/app.rs` — the field swap, handle_palette_key rewrite, toggle-theme
  arm + `toggle_theme_state`, Open-Settings removal, overlay highlight + chips.
- M `crates/marley_app/src/lib.rs` — export `PaletteState`/`action_for_command`/`toggled_appearance`.
- M `docs/specs/SPEC-app-shell.spec.md` — R32 (selection model) + an R17 SELECTED-command note +
  AC/Test-Plan/Mutation-Targets rows. CHANGELOG; arch doc at complete.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `palette_selection_clamps_at_both_ends` (len 3: down×5 → 2, up×5 → 0); `palette_selection_resets_on_edit` (push AND backspace both reset a moved selection); `palette_selection_stays_in_bounds_after_shrinking_edit` (moved to 2, narrowing push → reset to 0 IS the bounds mechanism — asserted) | unit |
| REQ-002 | `activate_returns_selected_command` (selected≠0 → that exact id); `activate_on_empty_is_none`; `move_down_on_empty_stays_zero` | unit |
| REQ-003 | `toggled_appearance_flips_both_ways` (Dark→Light, Light→Dark — kills the arm swap); registry `default_for` already R21-covered; the shim glue = review | unit + review |
| REQ-004 | `action_for_command_maps_every_row` (all 5 verbs asserted exactly + unknown id → None + the REMOVED id 3 → None) | unit |
| (chips) | `keybinding_display_formats_chords` (cmd-shift-b → "cmd-shift-b"; bare key; each modifier arm asserted — kills per-arm drops + ordering swaps) | unit |
| REQ-005 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the overlay render + Keystroke plumbing (app.rs — existing exclude); headed
palette-selection baseline rides the #23 desktop-session deferral.

### Risks / decisions
- D-2.1 PaletteState OWNS the query — reset-on-edit is an internal invariant, not a caller
  convention; app.rs can't forget to reset.
- D-2.2 `activate` uses `.get(selected)` — one uniform arm, no dead defensive clamp (the
  M1.A no-dead-clamp rule).
- D-2.3 `move_down` takes `filtered_len` per call (the caller has just computed the filter);
  `min(selected+1, len.saturating_sub(1))` is len-0-safe (→ 0).
- D-2.4 Arrow key names are gpui's "up"/"down" (verified against gpui Keystroke conventions at
  implement via cargo check + the existing "escape"/"enter"/"backspace" precedent).
- Enter always dismisses (activation or not) — matches R17's activate-then-dismiss and keeps the
  empty-Enter UX unsurprising.

## Phase 3 — Implement
- **Built (per manifest):** palette.rs — `PaletteState` (query-owning; push/backspace reset;
  move_up saturating; move_down `min(sel+1, len.saturating_sub(1))`; activate via `.get` — no
  dead clamp) + `action_for_command` (5 verbs; the removed CommandId(3) falls to `_ => None`);
  keymap.rs — `KeyBinding::display` (cmd/ctrl/alt/shift order); themes.rs — `toggled_appearance`;
  app.rs — `palette: PaletteState` field (open resets via `new()`), `handle_palette_key` rewrite
  (escape/enter/up/down/backspace/chars — Enter activates → `action_for_command` → the ONE
  `dispatch_action`, always dismisses), the `"toggle-theme"` arm + `toggle_theme_state`
  (registry default_for(toggled_appearance)), the overlay's selected-row highlight
  (accent/on_accent) + chord chips (`KeyboardShortcut::parse(&binding.display()).keys().join`),
  "Open Settings" removed with the id-gap comment; lib.rs exports. SPEC R32 + AC row + Test-Plan
  + Mutation-Targets; CHANGELOG entry.
- **Deviations from design:** none.
- **Verification at this phase:** workspace check 0 errors; clippy `-D warnings` silent;
  49 lib tests pass. The R32 unit suite is Phase 4.

## Phase 3.5 — Inspect
- **Critics run:** 2 (correctness/dispatch-consistency + gpui/provenance/simplification), both
  probing pure surfaces via path-dep crates (93 + independent assertions) and reading gpui/
  ui_components sources.
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | MED | `KeyBinding::display` joins modifiers with `-` but `KeyboardShortcut::parse` splits on `+` → the parse→keys→join round-trip is an IDENTITY no-op; the ticket's headline "#17 widgets' first real consumer" does nothing (chip renders "cmd-shift-b" as one token, not tokenized). BOTH critics found it independently. | REAL | `display` → `join("+")` (+ doc); the chip now tokenizes to "cmd shift b". |
  | F2 | MED | `action_for_command` (id→verb) is a second source of truth keyed on the SAME magic ids as `cockpit_commands` — reorder/insert/renumber a command → silent wrong-verb or no-dispatch; the per-row table test doesn't catch the cockpit↔table drift. | REAL (design smell, not a live bug — all 5 rows currently consistent) | Kept the pure table (ticket-SPEC'd + per-row mutation coverage — moving verbs onto `Command` would make them untested shim literals, the WRONG direction for the bar) + added `every_cockpit_command_resolves_to_a_verb` (walks `cockpit_commands()`, asserts each id → `Some` verb) — the exact drift guard, failing loudly. |
  | F3 | LOW | "R23's first real caller" imprecise — `toggle_theme_state` sets `self.theme` directly (mirrors the cx-less `toggle_dock_state`; caller notifies), does NOT call `set_theme`. | REAL (wording) | CHANGELOG reworded to "the first runtime theme switch … the dispatch caller notifies, the same cx-less pattern as `toggle_dock_state`". The code pattern is consistent + kept. |
  | F4 | LOW | `format!("  {chip}")` appends a stray whitespace child even for unbound rows (empty chip). | REAL (render wart) | Guarded `if !chip.is_empty()`. |
- **Rejected / accepted:** the ArrowDown `.len()` recompute (5-item static list, negligible; Enter
  is not double — palette closes first) — left; `SharedString` slice `.join(" ")` confirmed to
  compile via `Borrow<str>`; arrow keys confirmed gpui `"up"`/`"down"` (events.rs:334-335);
  `palette_query` field fully removed (0 refs).
- **Provenance lens:** CLEAN — generic identifiers, no Zed/Warp, no secrets, spawn inputs
  untouched.
- **Post-fix verification:** fmt/clippy clean; 50 lib tests pass incl. the new consistency test.

## Phase 4 — Validate
- **Tests added:** palette.rs — `palette_selection_clamps_at_both_ends`,
  `palette_selection_resets_on_edit` (push AND backspace), `move_down_on_empty_stays_zero`,
  `activate_returns_selected_command`, `activate_on_empty_is_none`,
  `action_for_command_maps_every_row` (5 verbs + removed id 3 → None + unknown → None);
  keymap.rs — `keybinding_display_formats_chords` (bare/each-modifier/all-four ordering, the
  `+` separator); themes.rs — `toggled_appearance_flips_both_ways` (+ composed with
  `default_for`); plus the inspect `every_cockpit_command_resolves_to_a_verb` drift guard.
- **Runs (actual):** `cargo nextest run --workspace` → **468 passed, 6 skipped**; doctests green.
- **Visual (gate:15):** PASS headless; the palette-selection headed baseline rides the #23
  desktop-session deferral (forge follow-up).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 first try**; coverage 100% lines;
  mutation **21 caught / 0 missed → MSI 100.0%**; receipt written.
- **Pre-existing failures:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG entry (chip/set_theme wording corrected at inspect); `app_shell.md`
  palette bullet extended (PaletteState + action_for_command + consistency test) and the
  deferred/shipped lists updated (seq-4 done). SPEC R32 landed at implement.
- **AAR capture:** `PR-claude-shared-vocabulary-across-a-parse-emit-seam-must-agree-001` (the
  display↔parse separator no-op — the emit-side twin of TICKET-022's decode-order rule);
  aar-submit `completed`. Lesson: a parse/emit seam degrades SILENTLY to identity when the two
  sides disagree on grammar — a round-trip assertion is the cheap guard; both critics caught it
  independently only by reading the OTHER side's split char.
- **Ticket:** forge #25 → done; local doc → closed/; pipeline pair archived.
