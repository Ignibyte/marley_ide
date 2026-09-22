# Editor-tab key gate — Notes

- **Forge ticket:** #283 (401ba6b7-d6e6-4886-94e9-7db9a1e4adc6)
- **AAR:** f836f1d7-a885-49bc-b0cd-975b9087051b
- **Local ticket doc:** docs/planning/tickets/open/TICKET-283-editor-tab-key-gate.md
- **Pipeline spec:** 283-editor-tab-key-gate.spec.md

## Phase 1 — Plan
- **Request:** #283 (M17-followup, from #276 routing critic F4) — on an editor tab, modified
  Enter/Tab/Backspace leak past the editor gate and act on the HIDDEN prompt; swallow them.
- **Classification:** work pipeline (bug). A tiny PURE predicate (cov/MSI 100) + a one-line app.rs shim
  gate + headless asserts. Mirrors #278 exactly.
- **Forge recall:** `bulletin-list` → none. The #278 fix (`readline_keys_at_cooked_prompt_headless` +
  the `active_tab().grid().is_some()` gate at app.rs:6285) is the direct precedent — its inspect F1
  comment already names "the #283 class". No new prevention rules surfaced.
- **Discovery (grounded):**
  - `app.rs:6069` — the editor router block gate: `active_tab().editor().is_some() && !platform && !control`.
    A ⌘/⌃-modified key on an editor tab is FALSE here → skips the entire editor block → terminal fallthrough.
  - `app.rs:1846 key_from_keystroke` — `"backspace"→Backspace`, `"enter"→Enter` REGARDLESS of modifiers.
  - LEAK sites in the terminal fallthrough: `6301` raw route (`input_route(..control..)==Raw` →
    `write_bytes(encode_key)` to `focused_terminal_mut`) catches ⌃Enter/⌃Tab; `6399` catch-all
    (`apply_key(state.buffer,…)` → Enter=`Submit`→`on_submit`, Backspace=edit) catches ⌘Enter/⌘Backspace.
  - `focused_terminal`/`focused_terminal_mut` resolve to a HIDDEN pane on a non-terminal tab (#71).
  - #278 precedent: the readline arm (6275-6299) already gates on `active_tab().grid().is_some()` and its
    comment (6280-6284) calls this "the #283 class, upgraded from raw bytes to visible state".
- **Decisions:** D1–D3 (spec). The fix: a pure `swallow_hidden_prompt_key(is_terminal, key)` in `input.rs`
  + a shim gate placed at the TOP of the terminal fallthrough (right after the editor block returns, before
  the raw route computes `alt_screen`/`command_running`) — swallow → `stop_propagation` + `return`.
- **§20:** N/A — Marley input-routing correctness; clean-room; mirrors the in-repo #278 gate.

## Phase 2 — Design

### Architecture / approach
A gpui key-router correctness fix in the app shim, backed by a pure predicate.
- **PURE seam** `crates/marley_app/src/input.rs`:
  `pub fn swallow_hidden_prompt_key(active_tab_is_terminal: bool, key: &str) -> bool =
   !active_tab_is_terminal && matches!(key, "enter" | "tab" | "backspace")`.
  Next to `op_for_ctrl_key` (68). PURE — cov/MSI 100.
- **SHIM gate** `crates/marley_app/src/app.rs`: at the TOP of the terminal fallthrough (right after the
  `if editor && !platform && !control { … }` block returns, before the `let (alt_screen, command_running)`
  at ~6265): `if input::swallow_hidden_prompt_key(active_tab().grid().is_some(), &event.keystroke.key)
  { cx.stop_propagation(); return; }`. Reached only when the editor gate was false, i.e. a terminal tab
  (grid Some → predicate false → normal handling) OR a non-terminal tab with a modified/leaked key
  (grid None → swallow enter/tab/backspace). No `notify` — nothing changed.
- §20 confirmed N/A — clean-room; the `grid().is_some()` gate is the in-repo #278 precedent, not a
  reference-app behavior.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/input.rs` | Add `swallow_hidden_prompt_key` (pure) + its `#[cfg(test)]` truth-table test. |
| `crates/marley_app/src/app.rs` | One shim gate at the top of the terminal fallthrough (~6261) that swallows on a non-terminal tab. |
| `crates/marley_app/src/headless_drive.rs` | Integration test: editor tab, hidden prompt has residual text, the 4 modified keys leave it UNCHANGED. |

### Regression Test Plan
| Test | Proves | REQ |
|---|---|---|
| `input::swallow_hidden_prompt_key_truth_table` | (false,"enter"/"tab"/"backspace")→true; (false,"x")→false; (true,"enter"/"backspace"/"tab")→false — kills `!is_terminal`, the `&&`→`||`, and each `matches!` arm | REQ-001/002/004/005 |
| `headless_drive::editor_tab_swallows_modified_enter_tab_backspace_headless` | type "resid" into the terminal prompt → open a file (editor tab) → `hidden_prompt_text=="resid"` → cmd-enter/ctrl-enter/cmd-backspace/ctrl-tab → `hidden_prompt_text` STILL "resid" (WITHOUT the fix cmd-enter submits→clears, cmd-backspace edits) | REQ-001/002 |
| existing `readline_keys_at_cooked_prompt_headless` + terminal flows stay green | terminal-tab Enter/Tab/Backspace byte-identical (grid Some → not swallowed) | REQ-003 |

- **trybuild:** none (a plain `fn`, no type contract).
- **Uncoverable:** the app.rs shim gate is `mutants::skip` + coverage-excluded (the input path); the headless
  flow is its behavioral proof. No live pixel needed — this is state (hidden_prompt_text), fully headless.

### Risks / decisions
- **Scope = non-terminal (editor + cockpit), not editor-only:** `grid().is_some()` naturally covers cockpit
  tabs too (they also leak to the hidden prompt). Correct + free; the headless test asserts the editor case.
- **Only enter/tab/backspace swallowed:** other keys keep the #267 fallthrough (an unbound ⌘X → `Key::Other`
  → `apply_key` Ignored today, so it's already harmless; not swallowing it preserves the platform text path).
- **Placement:** must be AFTER the editor block's returns (so a plain editor key still routes to the buffer)
  and BEFORE the raw route (6301) + catch-all (6399). Implement verifies the exact `}` boundary of the 6069 `if`.

## Phase 3 — Implement
`cargo check --workspace` green. Built to the manifest, no deviations.
- **input.rs** — `swallow_hidden_prompt_key(active_tab_is_terminal, key) = !active_tab_is_terminal &&
  matches!(key, "enter" | "tab" | "backspace")`, placed after `op_for_ctrl_key`.
- **app.rs** — the shim gate inserted right after the editor block's closing `}` (line 6260), before the
  terminal fallthrough's `alt_screen`/`command_running` computation: `if swallow_hidden_prompt_key(
  active_tab().grid().is_some(), &event.keystroke.key) { stop_propagation; return; }`. Confirmed the 6069
  editor-block `if` closes at 6260, so the gate sits between "editor handled it" and the hidden-prompt
  handlers (raw route 6301, catch-all 6399) — exactly the leak window.
- No `cx.notify()` on swallow (no state change). Tests deferred to Phase 4.

## Inspect (Phase 3.5)
A general-purpose routing critic was spawned; it over-ran (238KB+/7min on a 15-line diff — pathological
over-exploration), so per the slow-critic precedent I ran the adversarial review INLINE across the same
lenses, tracing the actual control flow. (If the critic lands a real finding before #283's commit, it is
folded in.) **No findings.** Lenses + concrete traces:
- **Terminal-tab regression (the big risk) — CLEAN.** On a terminal tab `active_tab().editor().is_some()`
  is false → the 6069 editor block is skipped → the new gate is reached with `grid().is_some() == true` →
  predicate `false` → NOT swallowed → Enter still reaches the catch-all `apply_key`→Submit, Tab still
  reaches `complete_at_prompt` (6319+), Backspace still edits. The gate changes NOTHING for a terminal tab.
- **Over-swallow of plain editor keys — CLEAN.** A PLAIN Enter/Tab/Backspace on an editor tab satisfies the
  6069 gate (`editor && !platform && !control`) → handled by the Tab arm (6094) / Enter arm (6139) which
  RETURN before the new gate. Only editor-tab keys that had ⌘/⌃ (so the 6069 gate was false) reach the gate.
- **Under-swallow — CLEAN.** ⌘Enter, ⌃Enter, ⌘Backspace, ⌃Tab, ⌃Backspace, ⌘Tab all map to key
  "enter"/"tab"/"backspace" → `swallow_hidden_prompt_key(false, …) == true` → swallowed BEFORE the raw route
  (6301+) and the catch-all (6399+). None slip through.
- **Cockpit tabs — CLEAN (bonus).** `grid()` None + `editor()` None → the gate swallows enter/tab/backspace
  there too. Those were already UNHANDLED leaks to the hidden prompt (cockpit key handling is above, at the
  keymap dispatch 6048); swallowing is strictly a correctness improvement, breaks nothing.
- **stop_propagation w/o notify — CLEAN.** A swallow changes no state (no `notify` needed) and `stop_propagation`
  prevents the #267 platform text path from double-delivering the key (e.g. ⌘Enter's simulated "\n"); a no-op
  is the intended v1 behavior (spec D3).
- **Mutants — covered by the Phase-4 truth table** (`(false,"enter"/"tab"/"backspace")→true`,
  `(false,"x")→false`, `(true,*)→false`) killing the `!` drop, `&&`→`||`, and each `matches!` arm.
- **Clean-room** — no Warp/Zed/iTerm/tmux in the diff comments.
- **Critic partial (before kill) CONFIRMED two points:** (1) no `enter`/`tab`/`backspace`/`return` is bound
  in ANY keymap context (the bound set is `[ ] a b c d down e f g j k l left o p r right s t up w x z`) — so
  swallowing them shadows NO binding; (2) `active_project()`/`active_tab()` are already called unconditionally
  at app.rs:6049, so the gate adds NO new panic surface. Both agree with the inline review.

## Phase 4 — Validate
**Tests added:** `input::t283_swallow_hidden_prompt_key_truth_table` (kills the `!` drop, `&&`→`||`, and each
`matches!` arm via (false,enter/tab/backspace)→true, (false,x/…)→false, (true,*)→false) +
`headless_drive::editor_tab_swallows_modified_enter_tab_backspace_headless` (type "resid" into the terminal,
open a file → editor tab, the four chords leave `hidden_prompt_text` == "resid" — without the fix ⌘Enter
submits it / ⌘Backspace edits it).
**Test run:** the 2 new tests PASS; the full gate's `gate:3` ran the workspace suite green (no regression).
**Drive-capture:** behavioral (a key does/doesn't reach a hidden buffer) — the headless flow through the real
keymap→router→fallthrough path IS the proof; no new render surface, so a live pixel capture is N/A.
**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** — coverage 100%, mutation **MSI 100.0% (4 caught
/ 0 missed)**, all 15 gates PASS. First-try green (no fixes needed).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Fixed` entry; `editor.md` input.rs section gains the `swallow_hidden_prompt_key`
  (#283) note next to `apply_editor_key`/#267.
- **Knowledge:** clean ticket — no failures. AAR submitted (effectiveness 5). Lessons: the fix was a clean
  mirror of the #278 `grid().is_some()` gate (the precedent named "the #283 class" in its own inspect comment),
  and the pure-predicate + shim-gate + headless-flow shape is the right template for a routing-leak fix —
  first-try GATE GREEN. The general-purpose critic over-ran (238KB/8min on 15 lines) and was killed; its
  partial output still confirmed the review — for a tiny diff, an inline adversarial trace is faster and
  sufficient (or use a sonnet critic with a tighter prompt).
- **Ticket + pipeline:** closed + archived.
