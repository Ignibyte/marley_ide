# Next/prev diagnostic navigation — Notes

- **Forge ticket:** #290 `af6d6c64-5811-4ecc-ba98-8603cb2654e5`
- **AAR:** `4799d92a-ca36-4108-9fcd-e2fb23c35b16`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-290-diagnostic-nav.md
- **Pipeline spec:** 290-diagnostic-nav.spec.md

## Phase 1 — Plan
- **Request:** F8/⇧F8 walk the caret through the #289 diagnostic rows, wrapping,
  scroll-following. Builds on #289 (rows) + #270 (follow) + #212 (caret).
- **Classification / tier:** work pipeline slice, `feature`. Crate: `marley_app`
  (`code_view.rs` pure nav, `keymap.rs` bindings, `app.rs` dispatch). Input path →
  Validate: pure fixtures + a headless F8-routes smoke.
- **Forge recall:** #289 (SHIPPED) `open_file_diagnostic_rows()` (sorted rows);
  #270 `follow_editor_caret`; #212 caret placement; keymap `KeyContext::Editor`
  bindings (⌘D `select-next-match`) + `action_for`; `binding_from_keystroke`
  (app.rs:1885) passes `keystroke.key` verbatim (so "f8" binds); the keymap lookup
  (6207) precedes the editor key branch → F8 resolves via the keymap.
- **Discovery (grounded):** keymap.rs:248 the ⌘D Editor binding format
  (`KeyBinding::chord(cmd,ctrl,alt,shift,key)` + `Some(KeyContext::Editor)`);
  `chords_unique_scoped` (331) the invariant; app.rs `dispatch_action` handles the
  action strings; `Buffer::line_col`/`line_start` for the caret row/target;
  `active_editor_mut().active_buffer_caret_anchor_mut()` for the caret set.
- **Decisions:** D1–D4 in the spec. Crux: pure `next/prev_diagnostic` (strictly
  after/before + wrap) is the only new LOGIC; the rest reuses #289/#270/#212.
- **Risk:** low. The nav math is a small pure fn (cov/MSI 100). The F8 routing is
  verified by grounding (keymap-before-editor-branch, verbatim key) + a headless
  smoke. The caret-set + follow reuse shipped seams.

## Phase 2 — Design

### Approach
Two pure nav fns + 2 keymap rows + 2 dispatch arms, reusing #289/#270/#212. §20 confirmed (F8/⇧F8
next-error convention — our own nav math + bindings).
- **`next_diagnostic(rows, current) -> Option<usize>`** (code_view.rs): `rows.iter().copied().find(|&r| r > current).or_else(|| rows.first().copied())`.
- **`prev_diagnostic(rows, current)`**: `rows.iter().rev().copied().find(|&r| r < current).or_else(|| rows.last().copied())`. Both `None` on empty. `rows` is #289's sorted set.
- **keymap.rs:** push `("f8"→"next-diagnostic", Editor)` + `("shift-f8"→"prev-diagnostic", Editor)`; `chords_unique_scoped` stays green (F8 unused).
- **app.rs `dispatch_action`:** two arms — `rows = self.open_file_diagnostic_rows()`; `if let Some(s)=active_editor_mut() { cur = line_col(caret).0; if Some(row)=next/prev_diagnostic(&rows,cur) { start=line_start(row); (_b,c,a)=active_buffer_caret_anchor_mut(); *c=start; *a=None; } }`; `self.follow_editor_caret()`. Borrow: `rows` via `&self` first; the `active_editor_mut` borrow ends before `follow_editor_caret`.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/code_view.rs` | ADD pure `next_diagnostic` + `prev_diagnostic`. |
| `crates/marley_app/src/keymap.rs` | ADD the two Editor-scoped F8 / ⇧F8 bindings. |
| `crates/marley_app/src/app.rs` | ADD the `next-diagnostic` / `prev-diagnostic` dispatch arms. |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | `next_diagnostic`: rows [2,8,14], 5→8, 8→14 (strict-after), 20→2 (wrap), empty→None. | REQ-001/002/003 |
| T2 | `prev_diagnostic`: rows [2,8,14], 10→8, 8→2 (strict-before), 0→14 (wrap), empty→None. | REQ-001/002/003 |
| T3 | keymap.rs: `chords_unique_scoped(scoped_entries())` true with the F8 rows; `action_for(f8,[Editor])`=="next-diagnostic". | REQ-004 routing |
| T4 | Headless: F8 on the editor reaches the dispatch (no panic; no-op with no diagnostics); the caret-move is carried by T1/T2 + the #289/#270 seams. | REQ-004 |
| — | cov/MSI 100 on `next_diagnostic` + `prev_diagnostic`. | REQ-005 |

Uncoverable: the dispatch arms are app.rs-excluded; the nav logic is T1/T2, routing is T3 (keymap, not excluded) + T4.

### Risks / decisions
- **R1 (F8 routing):** verified by grounding; T3 asserts `action_for` + uniqueness, T4 the headless routing. If gpui doesn't deliver "f8", T4 fails → switch to a ⌘-chord.
- **D-strict:** strictly-after/before (D1) so repeated F8 advances off a diagnostic row; wrap at ends.

## Phase 3 — Implement
- **Built to the manifest, no deviations.** `code_view.rs`: pure `next_diagnostic` (`iter().find(r>current)` else `first()`) + `prev_diagnostic` (`rev().find(r<current)` else `last()`), both `None` on empty. `keymap.rs`: `f8`→`next-diagnostic` + `shift-f8`→`prev-diagnostic`, both `KeyContext::Editor`. `app.rs`: the `next-diagnostic | prev-diagnostic` dispatch arm (rows via `open_file_diagnostic_rows`, caret row `line_col().0`, target row → `*caret=line_start(row)`, `*anchor=None`, then `follow_editor_caret`).
- `cargo check -p marley --all-targets` clean (borrow: `rows` computed via `&self` before the `active_editor_mut` borrow, which ends before `follow_editor_caret`); `cargo fmt` + `cargo clippy` clean.

## Inspect (Phase 3.5)
Inline adversarial trace (small diff — 2 pure nav fns + 2 bindings + 1 dispatch arm). **No defects.**

| Angle | Verdict | Evidence |
|---|---|---|
| Strict-after / before + wrap | SAFE | `next([2,8,14], 8)` → 14 (`>` skips 8, so repeated F8 advances); `next(_, 14/20)` → wrap to 2; `prev(_, 8)` → 2; `prev(_, 2/0)` → wrap to 14. |
| Empty → None | SAFE | `find` → None then `first()/last()` → None → None (a no-op). |
| F8 / ⇧F8 uniqueness | SAFE | Both new in `KeyContext::Editor`, distinct (shift), F8 unused elsewhere → `chords_unique_scoped` stays true (T3 asserts). |
| Dispatch borrow | SAFE | `rows` computed via `&self` first (owned); the `active_editor_mut` borrow ends at the if-let close; `follow_editor_caret` (`&self`) runs after. `cargo check` clean. |
| None no-op + caret set | SAFE | `if let Some(row)` skips on empty (caret unchanged); on Some, `*c = line_start(row); *a = None` (collapse to the row start); `follow_editor_caret` non-strict. |
| MSI | Phase-4-gated | `>`→`>=`/`<` on next (killed by the "8→14" strict case); `<`→`<=` on prev (killed by "8→2"); the wrap `or_else` (killed by the 20→2 / 0→14 cases); fn-replacement (exact values). |

No `failure-record` (no bug). No new prevention rule (a clean small nav fn reusing #289/#270/#212).

## Phase 4 — Validate

## Phase 5 — Complete

## Phase 4 — Validate
- **Tests added:** code_view.rs `diagnostic_nav_walks_strictly_and_wraps` (T1/T2 — strict-after/before, ON-a-row advances, wrap both ends, empty→None); keymap.rs `f8_diagnostic_nav_editor_scoped_and_unique` (T3 — `action_for(f8,ED)`=next-diagnostic, ⇧F8=prev, None on TERM/COCKPIT, `chords_unique_scoped` stays true); headless `f8_diagnostic_nav_routes_and_noops_without_diagnostics_headless` (T4 — F8 on the editor routes to the dispatch without panic, caret unmoved with no diagnostics).
- **Two roster-guard test updates** (the F8/⇧F8 bindings are REAL additions): `all_chords_lists_every_binding` count 45→47 + the scoped count 4→6 (+ contains-assertions for the F8 rows). Legitimate expectation update, not a floor lowering.
- **`cargo nextest run -p marley`: PASS.** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15/15 incl. coverage 100% + MSI 100% on `next_diagnostic`/`prev_diagnostic`.
- **Live-drive:** the F8 routing + no-op are headless-proven (T4); the caret-JUMP-on-diagnostic needs an injected failed block (env-blocked, same as #289) — carried by T1/T2 (the nav math) + #289/#270 (shipped). Re-verify with a real multi-error build → F8 walks the errors, when unlocked.
- No pre-existing failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Added (#290); app_shell.md extended with the #290 F8/⇧F8 diagnostic nav.
- **Knowledge (forge):** AAR `4799d92a` submitted — completed, effectiveness 5. No failure-record (inline clean). No new prevention rule. LESSON: adding a real keymap binding trips the ROSTER-GUARD tests (`all_chords_lists_every_binding` count + the scoped-count assertion) — update those expected counts as part of the change (a legit expectation update, not a floor lowering); grep for `chords.len()` / `scoped.*count` when adding a binding.
- **Ticket** TICKET-290 → closed/ + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** F8/⇧F8 diagnostic nav shipped; the #289 gutter is now navigable. cov/MSI 100, GATE GREEN [diff]. LOCAL commit only.
