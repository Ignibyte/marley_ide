# Hide the shell prompt input row while a foreground command runs — Notes

- **Forge ticket:** #193 (968fc19a-2615-4d41-b8c8-b506fc01416d)
- **AAR:** 4410fd5e-e0bd-46ef-ba7f-bb1f83097916
- **Local ticket doc:** docs/planning/tickets/open/TICKET-193-hide-prompt-while-running.md
- **Pipeline spec:** hide-prompt-while-running.spec.md

## Phase 1 — Plan
- **Request:** chad live-app feedback #5 — launching `claude` in a pane shows TWO
  input areas (claude's own `>` and Marley's cooked ❯ prompt row). Warp hides its
  prompt when a foreground program owns stdin. Hide Marley's prompt input row while
  `state.session.is_command_running()`; keep scrollback blocks; restore on exit.
- **Classification / tier:** work pipeline, one slice. Small. Systems: `marley_app`
  (pane render shim in app.rs) + a new PURE fn in `marley_app::nav`.
- **Forge recall (§18.3/§19):** knowledge/docs-search ran but the docs index is
  cross-project-polluted (returned oathstar-studio hits) → best-effort only.
  The load-bearing trap comes from local memory: **the `mutants::skip` detach**
  ([[mutants-skip-detach-trap]]) — inserting a fn between a skip attribute and its
  target rebinds it → unkillable mutants → MSI red. Mitigation locked in D3.
- **Discovery (confirmed live):**
  - `content_rows(state)` at **app.rs:1051** = `fold_visible_rows(&block_line_counts(state), &state.folds).len() + 1` — the `+1` IS the prompt row.
  - `fold_visible_rows` is PURE at **nav.rs:67** (already unit-tested) — the home for the new `content_row_count`.
  - `content_rows` callers (viewport/scroll math): app.rs 999, 1023, 1102, 3516, 4177, 4231.
  - The ❯ input-row render site (pane body, ~app.rs:4110–4260) to be pinned precisely in Design.
- **Decisions:** D1 pure `content_row_count`; D2 single `!is_command_running`
  source for gate+count; D3 skip-detach avoidance + `cargo mutants --list` verify;
  D4 live driven capture (display awake).

## Phase 2 — Design

**Architecture / approach.** The change lives entirely in the cockpit render layer
(`marley_app`) + one pure helper in `marley_app::nav`. No terminal/PTY/session-model
change — `is_command_running()` already exists (terminal_blocks/session.rs:177). Data
flow: the cooked-block pane render (app.rs ~4263–4512) walks a running `row` index over
the fold-aware block rows (pure `fold_visible_rows`) and appends ONE trailing prompt
input row (app.rs:4473–4512). The viewport window `(start,end) = viewport.visible(content, capacity)`
is sized by `content = content_rows(state)` (app.rs:1051), whose `+1` IS that prompt row.
To hide the prompt while a foreground command runs AND keep the count honest, both the
COUNT and the RENDER derive from the same predicate `!state.session.is_command_running()`:
- Extract the arithmetic into a PURE `content_row_count(visible_rows, prompt_visible) -> usize`
  = `visible_rows + usize::from(prompt_visible)` (nav.rs). `content_rows` delegates with
  `prompt_visible = !is_command_running`, so the `+1` becomes conditional. `content_rows`
  stays a `#[cfg_attr(test, mutants::skip)]` shim (it takes a live `TerminalPane` + reads
  the session); the arithmetic it now delegates is the mutation-tested seam.
- Gate the prompt-row render (app.rs:4476) on the same `!is_command_running` so the row
  is not emitted while a command runs. Because `content` already excludes it, the viewport
  math + the `row` walker stay in lockstep — no phantom blank bottom row, no skew.

§14: no new types, no panics, no IO; `is_command_running` is a pure session read; the
prompt-visibility predicate is the same method call at both sites (consistent by
construction), and `content_row_count` centralizes + documents the invariant.

**File manifest.**
- `crates/marley_app/src/nav.rs` — ADD `pub fn content_row_count(visible_rows: usize, prompt_visible: bool) -> usize`
  after `fold_visible_rows` (~line 78; nav.rs has ZERO `mutants::skip` → detach-safe) +
  unit tests in the existing `mod tests` (line 134).
- `crates/marley_app/src/app.rs`:
  - line 77 — extend `use crate::nav::{…}` with `content_row_count`.
  - `content_rows` (1050–1053) — body → `content_row_count(fold_visible_rows(&block_line_counts(state), &state.folds).len(), !state.session.is_command_running())`;
    update the doc ("+ the one prompt line" → "+ the prompt line WHEN no foreground command
    is running (#193)"). Stays a skip'd shim.
  - prompt input-row render (4473–4512) — compute `let prompt_visible = !state.session.is_command_running();`
    once (near the `content` calc at 4266) and change the gate at 4476 to
    `if prompt_visible && row >= start && row < end`; comment ties it to `content_rows` (#193).

**Regression Test Plan.**
| REQ | Test (kind) | Asserts |
|---|---|---|
| REQ-002 | `nav::tests::content_row_count_excludes_prompt_when_running` (unit+mutation) | (0,false)=0, (1,false)=1, (5,false)=5 — kills the `+1`/arithmetic mutants when the prompt is hidden |
| REQ-003 | `nav::tests::content_row_count_includes_prompt_when_idle` (unit+mutation) | (0,true)=1, (5,true)=6 — the prompt row is counted when visible; kills the bool-swap + `+0`/identity mutants |
| REQ-001 | driven capture | claude running → NO ❯ row (the `content_row_count(_,false)` test proves the count side) |
| REQ-003 | driven capture | after claude exits → the ❯ prompt row returns |
| REQ-004 | driven capture | scrollback blocks stay visible while claude runs (`content_row_count(v,false)=v` proves every block row is still counted) |
| REQ-005 | review + the REQ-002/003 mutation tests | one `!is_command_running` expression drives both `content_rows` + the render gate; the `prompt_visible` param's mutation-killed tests prove the bool actually toggles the count |

- **trybuild:** N/A — no type-safety/newtype contract added (a plain `usize` fn).
- **Uncoverable by unit (→ driven capture):** `content_rows`' live `is_command_running`
  wiring + the gpui render gate need a real `TerminalSession` (PTY) + the GUI; the selftest
  harness drives `claude` + reads pixels (REQ-001/003/004). Display is awake — unblocked.

**Risks / decisions.**
- **R1 skip-detach (D3):** nav.rs has zero `mutants::skip` → adding `content_row_count`
  cannot detach one; app.rs's fn set is unchanged (only bodies/import edited). Validate
  verifies `cargo mutants --list -f nav.rs` lists `content_row_count` (mutable) and the
  app.rs shims stay absent.
- **R2 gate composition:** `sticky_block` already returns None at the prompt row
  (nav.rs:116); find operates on blocks; scroll sizes off `content`, which now shrinks by 1
  while running — `viewport.visible` clamps (content ≥ 0). Edge: zero blocks + running
  command → content=0 → renders nothing (safe); `content_row_count(0,false)=0` covers it.
- **R3 input routing already bypasses the buffer while a command runs** (app.rs:3436/3442 —
  keystrokes reach the program, not `state.buffer`), so hiding the buffer's prompt row is
  consistent with existing behavior — which is why this is correct for ANY foreground
  command, not just agents.

## Phase 3 — Implement
- **Built exactly to the manifest — 2 files, 5 edits:**
  - `nav.rs` — added pure `pub fn content_row_count(visible_rows, prompt_visible) -> usize`
    = `visible_rows + usize::from(prompt_visible)`, inserted after `fold_visible_rows` (before
    `fold_block_at_row`'s doc). No tests (Phase 4).
  - `app.rs:77` — extended `use crate::nav::{…}` with `content_row_count` (rustfmt reflowed).
  - `app.rs` `content_rows` — body now delegates to `content_row_count(fold_visible_rows(…).len(), !state.session.is_command_running())`; doc updated. Stays `#[cfg_attr(test, mutants::skip)]`.
  - `app.rs` render — `let prompt_visible = !state.session.is_command_running();` computed once
    beside the `content` calc; the prompt input-row gate became `if prompt_visible && row >= start && row < end`.
- **Deviations:** none.
- **Compile:** `cargo fmt` + `cargo check -p marley` clean (only the pre-existing `block v0.1.6`
  future-incompat dep warning — not ours).
- **D3 (skip-detach):** deferred the `cargo mutants --list -f nav.rs` verify to Phase 4 (run
  alongside mutation) — low risk (nav.rs has no skips; app.rs's fn set is unchanged, only bodies/import).

## Phase 3.5 — Inspect
Two independent general-purpose critics over the diff, in parallel: (A) correctness /
off-by-one / lockstep, (B) skip-detach / mutation-killability / reuse. Both verified
concretely (read viewport.rs/app.rs/nav.rs/session.rs; ran `cargo mutants --list`).

| # | Finding | Verdict | Action |
|---|---|---|---|
| 1 | [MED, critic A] `content_row_count` has no unit test → would fail the cov/MSI-100 gate | **Not a defect — expected.** Tests are Phase 4 by design; the test plan (REQ-002/003) already specifies `(0,false)=0,(5,false)=5,(0,true)=1,(5,true)=6`. | Carried to Phase 4 (no code change) |
| 2 | [correction, critic B] cargo-mutants generates **4** mutants for the helper (`body→0`, `body→1`, `+→-`, `+→*`), NOT the 6 hypothesized (no `usize::from→0/1`) | Accurate — verified via `cargo mutants --list -f nav.rs`. All 4 killed by the planned cases; `{(5,false),(5,true)}` alone suffices. | Phase 4 asserts sized to the real 4 |

**Clean lenses (concrete evidence):**
- **Viewport safety:** `viewport.visible` uses `saturating_sub` for `start`/`max_scroll`; `content=0 → (0,0)` (no underflow/panic). Existing test `visible_following_shows_bottom_capacity` covers it.
- **No stale scroll offset:** the stored `top` is clamped on READ (`top.min(max_scroll(content,cap))`) and only mutated by scroll — a ±1 content change from the prompt can never point it past content. `held_window_holds_as_content_grows` covers the shrink-clamp.
- **Row walker:** running ⇒ `content=N=fold_visible_rows.len()`; `end=(start+cap).min(N)≤N`, so the last block row `N-1<end` still emits and the prompt (row `N`) is gated off — no hidden block, no phantom/blank bottom row. `content` + `prompt_visible` are two reads of `is_command_running()` in one synchronous render → cannot diverge in a frame.
- **Copy/find/selection alignment:** `content_row_texts`/`observed_row_texts` build from blocks only and NEVER included the prompt row; `active_find_row` derives from them so it can't equal the prompt index; `sticky_block(start)` only indexes real block rows while running. Hiding the prompt makes `content_rows == content_row_texts.len()` while running — strictly MORE consistent than before.
- **Edge cases:** "0 blocks + running" is impossible (`is_command_running()` ⇒ a running block pushed into `blocks` ⇒ `N≥1`); a folded last block still contributes its header row; on EXIT `is_command_running()` flips false → `prompt_visible=true`, `content` grows by 1, the following-viewport re-anchors → prompt reappears (REQ-003 ✓).
- **Skip-detach:** nav.rs has zero `mutants::skip` (nothing to detach); the app.rs shims (`content_rows`/`content_row_texts`/`observed_row_texts`/`pane_grid_pos`) all stayed absent from the mutable list; the render gate lives inside the skip'd `fn render` → 0 live mutants in 4260–4500.
- **Reuse/idiom:** no existing helper did this; `usize::from(bool)` is the idiomatic branchless conversion; extracting the arithmetic turns the previously-unkillable inline `+1` into a mutation-covered pure fn.

**Net: no confirmed defects, no source fixes.** Finding 1 is the Phase 4 test (planned); Finding 2 sharpens the Phase 4 assert set.

## Phase 4 — Validate
**Test added:** `nav::tests::content_row_count_adds_prompt_row_only_when_visible` —
`(5,true)=6`, `(0,true)=1`, `(5,false)=5`, `(0,false)=0` (kills all 4 real mutants:
body→0, body→1, +→-, +→*; the 0-cases document the empty-pane boundary).

**Tests RUN:**
- `cargo nextest run -p marley content_row_count` → 1 passed.
- `cargo nextest run --workspace` → **768 passed, 5 skipped** (incl. the new test).

**Live driven capture (display awake — bundle → open → drive → screencapture → READ):**
Validated with `sleep 12` (deterministically toggles the same `is_command_running`
path an agent triggers; more reliable than depending on `claude` being on the bundled
app's PATH). Three states, all read from pixels:
1. **Idle** (`193-1-idle.png`) — the `❯ Marley` prompt input row is shown at the pane bottom.
2. **Running** (`193-2-running.png`) — typed `sleep 12`+Enter; while it runs the pane bottom shows the `▾ ○ sleep 12` running BLOCK (with `↻ run ⧉ cmd ⧉ out`) and **NO `❯` prompt row below it** (REQ-001 ✓); the block stays visible (REQ-004 ✓); no phantom blank bottom row.
3. **After exit** (`193-3-after.png`) — the block shows green **`✓ sleep 12`** and the **`❯ Marley` prompt row has returned** below it (REQ-003 ✓).
REQ-005 holds by construction (both `content_rows` and the render gate read the same
`!is_command_running`) + is proven by the gate's mutation pass.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15 passed / 0 failed.**
gate:5 mutation MSI 100% (all `content_row_count` mutants killed; NO `mutants::skip`
detached — D3 verified by the gate), gate:4 coverage 100%, gate:15 visual/AX green.
One mid-validate fix: `cargo fmt` re-flowed the new test's inline comment (blank-line
separated for readability); re-ran green.

**Pre-existing:** none in scope. (Only the unrelated upstream `block v0.1.6`
future-incompat dep warning, not ours.)

**Side observation (not this ticket):** the pane's tab title tracked the running command
(`terminal 2` → `sleep` → back) — so a command-aware tab-title default already exists in
some form; check what's there when planning **#201** (rename may be the only gap).

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
