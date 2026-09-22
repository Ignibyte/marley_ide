# background-pane command-finish notification — Notes

- **Forge ticket:** #203 (4f5fac35-0985-4986-888e-203710700ea5)
- **AAR:** ee7be81c-460e-474d-a06f-12f1394d2c0a
- **Local ticket doc:** docs/planning/tickets/open/TICKET-203-warp-bg-notify.md
- **Pipeline spec:** warp-bg-notify.spec.md

## Phase 1 — Plan
- **Request:** forge #203 — notify when a long-running command finishes in a background (unfocused)
  pane; flash the pane's tab (success vs failure) past a duration threshold.
- **Classification / tier:** work pipeline — a bounded feature (one pure decision fn + a masked-shim
  timing/flash wiring). One shippable slice.
- **Forge recall (§18.3):** aar-open → ee7be81c. Deps: flash.rs (#77), block_status.rs
  (`exit_status_kind`), tabs.rs, #193 (command lifecycle), #173 (pump-all-grids). Pivoted here from
  #202 (parked on a product fork — its zero-tab premise is unreachable under the M10 never-empties
  guards; surfaced to chad [AFK] + recorded on the #202 ticket).
- **Discovery (code read):**
  - `flash.rs`: `Flash { message: String, remaining: u32 }` + `new` + `tick(self) -> Option<Flash>`
    (the pump-tick countdown, #77) — reused for the per-tab flash.
  - `block_status.rs`: `StatusKind { Running, Success, Failure }` + `exit_status_kind(state, exit)`
    (Finished+0→Success; non-zero/None→Failure) — reused for the success/failure split.
  - `agent_view.rs`: `fmt_duration(secs: u64)` (#187) is a FORMATTER only.
  - The pump (app.rs:504–560, #173) drains EVERY grid via `term.session.pump()` each 16ms frame — a
    background pane's blocks transition Running→Finished even when unfocused, so the finish is
    observable. `is_command_running()` (session.rs:177 = `blocks().current().is_some()`) is the
    per-pane running signal; the last block's `exit_code` → `exit_status_kind` gives the outcome.
  - Focused pane = `self.workspace().focused()` (the active grid's focused PaneId).
  - **CORRECTION to the ticket:** a terminal `Block` (block.rs:60) has id/index/session_id/command/
    state/exit_code/prompt/output — NO started_at/elapsed/timestamp. The ticket's "elapsed already
    tracked / reuse duration.rs" is wrong (only AGENTS track duration, #187 `AgentRun`). The shim must
    stamp start times to compute elapsed.
- **The bounded delta:**
  - PURE: `should_notify(pane_focused, status: StatusKind, elapsed_secs: u64, threshold_secs: u64)
    -> Notify{No,Succeeded,Failed}` — pane_focused → No; elapsed < threshold → No; else status →
    Succeeded/Failed (Running → No). cov/MSI 100 (run `cargo mutants --list` for the real set; recall
    the syntactic-form lesson — an if/if-let yields only `delete !`, a match-guard yields
    true/false/delete-!).
  - SHIM (masked): per-pane `(was_running: bool, started: Option<Instant>)` in a RootView map; in the
    pump, on false→true stamp `started = Instant::now()`; on true→false compute `elapsed_secs =
    started.elapsed().as_secs()`, read the last block's `exit_status_kind`, and if the pane is not the
    focused pane of the active tab call `should_notify` → on Succeeded/Failed set a per-tab flash
    (RootView `HashMap<(proj,tab), Flash>`); the rail render shows it (styled by outcome); focusing the
    tab (or the countdown) clears it. `Instant` lives only in the masked shim.
- **Decisions:** D1 pure fn types marley-native (u64 secs + StatusKind, not std Duration); D2 boundary
  `elapsed >= threshold` notifies; D3 shim stamps elapsed, Block model unchanged; D4 per-tab flash via
  a RootView map reusing Flash; D5 OS notification OUT (follow-up); clean-room §20.
- **Open for design:** the `should_notify` module home (new `notify.rs` vs fold into block_status); the
  threshold const value; the per-tab flash key + rail render slot; the exact edge-detection storage.

## Phase 2 — Design

### Architecture / approach
The cockpit tab rail. A pure decision fn (`notify.rs`, gpui-free, cov/MSI 100) + a masked shim in
`app.rs` (the pump edge-detection + the per-tab completion badge render). No terminal_blocks change.

**Design refinements settled during discovery:**
- **`colors.danger` is the failure color** (ThemeColors has `success` + `danger`; `status_indicator`
  uses ✓/`success`, ✗/`danger`). The badge reuses them.
- **Persistent-until-viewed badge, NOT a timed flash.** A brief flash while you're away is easy to
  miss; a completion dot that stays on the background tab's rail row until you switch to that tab is
  the useful "did my build finish?" cue (like an unread badge). This DROPS the countdown entirely →
  the pure surface is just `Notify` + `should_notify` (no `TabFlash` countdown type). REQ-005 becomes
  "clears when the tab is viewed" (the "or countdown expires" clause is dropped — recorded).
- **Elapsed via a per-pane pump-tick counter, NOT wall-clock in the model.** A `RootView`
  `notify_ticks: HashMap<PaneId, u32>` counts pump ticks (16ms each) while a pane's command runs;
  `elapsed_secs = ticks * 16 / 1000`. Deterministic, integer, no `Instant` anywhere, no `PaneState`
  change. (`Instant` was the alternative — the #187 shim precedent — but the tick counter is simpler
  and sidesteps both the serialization concern and the cross-field borrow.)

**The pure seam** (`crates/marley_app/src/notify.rs`, NEW):
```rust
use crate::block_status::StatusKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notify { No, Succeeded, Failed }

/// Whether a finished command in a pane warrants a background-completion badge, and its outcome (#203).
/// No badge when the pane is focused (you're watching it) or the command was quick (< threshold);
/// otherwise the outcome. `elapsed_secs`/`threshold_secs` are u64 seconds (Instant is banned in pure code).
pub fn should_notify(pane_focused: bool, status: StatusKind, elapsed_secs: u64, threshold_secs: u64) -> Notify {
    if pane_focused { return Notify::No; }
    if elapsed_secs < threshold_secs { return Notify::No; }   // AT the threshold notifies (>= )
    match status {
        StatusKind::Success => Notify::Succeeded,
        StatusKind::Failure => Notify::Failed,
        StatusKind::Running => Notify::No,   // shouldn't occur on a real finish; defensive
    }
}
```

**The shim** (`app.rs`, masked):
- Two `RootView` fields: `notify_ticks: HashMap<PaneId, u32>` (per-pane running-tick counter; 0 = idle)
  and `tab_flashes: HashMap<(usize, usize), Notify>` (per-tab completion badge, keyed by (proj, tab)).
- `const NOTIFY_THRESHOLD_SECS: u64 = 10;` + reuse the 16ms pump interval (`elapsed_secs =
  ticks * 16 / 1000`). Bare literals (the const-arithmetic lesson).
- **Snapshot before the grids loop:** `let active_foreground_pane: Option<PaneId> =
  if self.shell.active_project().active_tab().grid().is_some() { Some(self.workspace().focused()) }
  else { None };` (None when the active tab is a cockpit/code tab → every terminal finish is
  "unfocused").
- **In the #173 pump loop** (`for grid in view.shell.grids_mut() { for (id, state) in grid.states_mut()`,
  after `pump()`): `view.notify_ticks` is a disjoint field of `view` (≠ `view.shell`), so it's read/
  written in-loop without a borrow conflict. Per pane:
  - `let running = term.session.is_command_running();`
  - `let ticks = view.notify_ticks.entry(*id).or_insert(0);`
  - if `running` → `*ticks += 1;`
  - else if `*ticks > 0` (a command just finished): `let elapsed_secs = *ticks as u64 * 16 / 1000;`
    `*ticks = 0;` read the last block's status `exit_status_kind(last.state, last.exit_code)`
    (`term.session.blocks().iter().last()`); `let pane_focused = Some(*id) == active_foreground_pane;`
    `let n = should_notify(pane_focused, status, elapsed_secs, NOTIFY_THRESHOLD_SECS);` if `n != No`
    push `(*id, n)` to a LOCAL `finishes: Vec<(PaneId, Notify)>`.
- **After the loop (two-phase, mirrors `dead`):** `for (id, n) in finishes { if let Some((p, t)) =
  view.locate_pane(id) { view.tab_flashes.insert((p, t), n); } }` — `locate_pane` borrows `view.shell`,
  so it must run after the grids_mut loop ends.
- **Clear-on-view** at the flash-tick site (~app.rs:652, beside the `status_flash` tick): compute the
  active foreground tab `(ap, at) = (active_project_index, active_project's active_tab_index)` and
  `view.tab_flashes.remove(&(ap, at));`. So a background tab's badge persists until you switch to it
  (then next tick clears it); an active-tab background-split finish is set then removed same tick →
  effectively no badge (the split is on-screen). No per-switch hooking needed.
- **Cleanup on close:** `notify_ticks.remove(&id)` in the dead-pane reap + the manual close paths
  (mirror `remotes.remove(&id)` at app.rs:600). `tab_flashes` self-heals (transient + clear-on-view),
  but prune the closed pane's (p,t) for safety.
- **Rail render** (the `RailLevel::Tab` row, app.rs ~3872, beside the #167 `agent_status` glyph):
  `if let Some(n) = self.tab_flashes.get(&(p, t)) { <a small ● dot: Succeeded → colors.success,
  Failed → colors.danger> }` prepended to the row (mirrors the agent glyph placement).

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/notify.rs` | NEW pure module: `enum Notify { No, Succeeded, Failed }` + `should_notify(pane_focused, status, elapsed_secs, threshold_secs)`. Tests at validate. |
| `crates/marley_app/src/lib.rs` | `mod notify;` |
| `crates/marley_app/src/app.rs` | `notify_ticks` + `tab_flashes` fields (+ `None`/empty init); `NOTIFY_THRESHOLD_SECS` const; the pump edge-detection + two-phase finish handling; the clear-on-view at the tick site; close cleanup; the rail-row badge render. (Masked shim.) |

### Regression Test Plan
`should_notify` — reasoned real mutant set (CONFIRM at validate with `cargo mutants --list -f notify.rs`):
fn-body → each unit variant `{No, Succeeded, Failed}`; `<` → `{==, >, <=}` (the fixed set, NOT `!=`/`>=`);
the bare-bool `if pane_focused` guard likely has no direct mutant (bare var read) — the body-variant
mutants force covering focused vs unfocused regardless.

| # | REQ | Test (exact-value, notify.rs `#[cfg(test)]`) | Kills |
|---|---|---|---|
| T1 | REQ-001 | `should_notify(false, Success, 10, 10)` == `Succeeded` | AT threshold notifies (`<`→`<=`); body→No/Failed |
| T2 | REQ-004 | `should_notify(false, Success, 9, 10)` == `No` | below threshold (`<`→`==`, `<`→`>`) |
| T3 | REQ-001 | `should_notify(false, Success, 15, 10)` == `Succeeded` | above threshold (`<`→`>` the other dir) |
| T4 | REQ-002 | `should_notify(false, Failure, 10, 10)` == `Failed` | the Failure arm; body→No/Succeeded |
| T5 | REQ-003 | `should_notify(true, Success, 15, 10)` == `No` | focused → No even when long+success; body→Succeeded; pins the pane_focused guard |
| T6 | (defensive) | `should_notify(false, Running, 15, 10)` == `No` | the Running arm → No |

**Driven (shim, uncoverable by unit — live GUI):** split a terminal (2 panes) or open a 2nd tab; run
`sleep 12` (> the 10s threshold) in a background pane/tab, keep focus elsewhere → on completion the
background tab's rail row shows a green ● (REQ-001/002); run `sleep 12; false` in a bg pane → a red ●
(REQ-002); a FOCUSED pane's `sleep 12` finish → NO badge (REQ-003); a quick `echo hi` in a bg pane →
NO badge (REQ-004); switch to the badged tab → badge clears (REQ-005). (12s so it clears the 10s
threshold with margin; the harness `focus`-in-same-call per the #198 lesson.)

### Risks / decisions
- **R1 (tick-drift):** elapsed is counted in 16ms pump ticks, not wall-clock — a busy machine that
  ticks slower undercounts. Acceptable for a coarse "past 10s" threshold; the pure fn stays exact
  (secs in). Chosen over `Instant` for determinism + no PaneState/borrow churn.
- **R2 (persistent badge vs flash):** persistent-until-viewed (not a timed flash) — better UX + drops
  the countdown/pure `TabFlash` type. REQ-005 adjusted to "clears when the tab is viewed".
- **R3 ((p,t) key vs reorder/close):** transient + clear-on-view + close-prune make a stale key
  self-heal; acceptable.
- **R4 (pane_focused semantics):** `id == the active tab's keyboard-focused pane`; combined with the
  clear-on-view (active-tab badges removed each tick), the visible effect is "badge only background
  TABS", which is the useful off-screen-notify behavior.
- **R5 (OS notification OUT):** deferred (D5) — platform adapter, un-headless-testable; the badge is
  the core. Follow-up if wanted.

## Phase 3 — Implement
**Built (per manifest, no deviations):**
- `crates/marley_app/src/notify.rs` (NEW pure) — `enum Notify { No, Succeeded, Failed }` +
  `should_notify(pane_focused, status: StatusKind, elapsed_secs, threshold_secs)` (the two early-return
  guards + the status match). gpui-free; imports `crate::block_status::StatusKind`.
- `crates/marley_app/src/lib.rs` — `mod notify;` (alphabetical, between `nav` and `palette`).
- `crates/marley_app/src/app.rs` (masked shim):
  - `use crate::notify::{should_notify, Notify};` (StatusKind not imported — `exit_status_kind`'s result
    passes straight through). `NOTIFY_THRESHOLD_SECS: u64 = 10` const. Two `RootView` fields
    `notify_ticks: HashMap<PaneId, u32>` + `tab_flashes: HashMap<(usize,usize), Notify>` (init to
    `HashMap::new()` in the one constructor beside agents/remotes).
  - Pump: snapshot `active_foreground_pane` before the grids loop; in-loop tick counter + finish-edge
    → `should_notify` → collect into a local `finishes` vec; two-phase after the loop keys each finish
    to `(p,t)` via `locate_pane` and sets `tab_flashes`. `notify_ticks.remove(&id)` on the dead-pane
    reap (beside `remotes.remove`). Clear-on-view: prune the active `(proj,tab)` badge at the flash-tick
    site each pump tick.
  - Rail render: a `●` badge (colors.success / colors.danger) on the Tab row when `tab_flashes` has
    `(p,t)`, placed beside the #167 agent-status glyph.

**The pump borrow held as designed:** `view.notify_ticks.entry(*id)` accessed INSIDE
`for grid in view.shell.grids_mut()` compiles — `notify_ticks` and `shell` are disjoint `RootView`
fields, so the borrow checker allows the simultaneous `&mut`; the `locate_pane`/`tab_flashes.insert`
(which re-borrow `view.shell`) run two-phase AFTER the loop (mirroring the existing `dead` pattern). No
fallback needed.

**Checks:** `cargo fmt` + `cargo check -p marley` clean (no errors, no warnings; only the pre-existing
upstream `block v0.1.6` note). Tests are Phase 4.

## Phase 3.5 — Inspect
2 general-purpose critics in parallel (Critic 1: pure fn + mutation + elapsed conversion; Critic 2:
the shim behavioral — edge-once, wrong-block-status, foreground snapshot, clear-on-view timing, borrow,
cleanup, scope) + self-review. **Both verdicts: SHIP** — no HIGH findings.

| # | Finding | Sev | Verdict | Action |
|---|---|---|---|---|
| F1 | **Clear-on-view didn't set `dirty`** — the badge is removed from state but the pump only repaints when `dirty`; on an otherwise-idle frame the ● lingers on the just-viewed tab until an unrelated repaint. Asymmetric with the SET path (which rides the finishing command's pump events). Undercuts "cleared once you view the tab". | MED | REAL | **FIXED** app.rs:707 → `if view.tab_flashes.remove(&seen).is_some() { dirty = true; }` (mirrors the `status_flash.tick()` dirty pattern). |
| F2 | The real cargo-mutants set for `should_notify` is **4 mutants, 3 viable** (`<` → `{==, >, <=}`); the whole-body mutant is `Default::default()` which is **UNVIABLE** (`Notify` has no `Default` derive → excluded from MSI). The assumed body→`No`/`Succeeded`/`Failed` variant mutants DON'T exist. | LOW | REAL (Phase-4 guidance) | The T1–T6 matrix stands: T1/T2/T3 kill the 3 `<` swaps → MSI 100. Per Critic 1, KEEP T4/T5/T6 even though each kills zero mutants — they're the ONLY guard on the focus short-circuit, the `Failure` arm, and the `Running` arm (no mutation pressure there). Reinforces `PR-claude-cargo-mutants-guard-mutants-depend-on-syntactic-form` (an enum-return fn's body mutant is `Default::default()`, unviable without a Default derive — not the variants). |
| F3 | Duplicated bare literal `16` — `Duration::from_millis(16)` (pump timer) + `*ticks*16/1000` (elapsed) encode the same frame interval independently; changing the pump rate would silently break elapsed. | LOW | REAL | **FIXED** — `const PUMP_INTERVAL_MS: u64 = 16;` used at both sites. |
| F4 | `notify_ticks` leaks a `u32` on each MANUAL close (close_tab/close_project/close-pane) — the scrub is only on the dead-pane reap. Bounded (one per pane ever closed), no aliasing (PaneId globally unique #167) → harmless. | LOW | DEFERRED | Follow-up: scrub `notify_ticks`/`tab_flashes` at the manual-close sites. Harmless bounded growth; not a blocker. |
| F5 | `tab_flashes` positional `(p,t)` keys aren't maintained on tab/project close — a close shifts later indices, so a surviving unviewed badge could briefly show on the WRONG tab until clear-on-view heals it. Transient + narrow (close a lower-indexed sibling while an unviewed badge exists). Matches the `renaming_tab` positional-key precedent. | LOW | DEFERRED | Follow-up (with F4): key by a stable id, or scrub on close. Transient + self-healing; not a blocker. |

**Critic-verified CLEAN** (concerns proven false): edge fires exactly once (`*ticks=0` before the next
frame); a sub-frame command never notifies (ticks stays 0); `is_command_running()` genuinely goes
true→false on finish (Precmd → Finished → `current()` None); **the finish frame's `blocks().last()` is
GUARANTEED the just-finished Finished block** (a Running last block ⟹ `is_command_running()` true ⟹ the
edge branch isn't entered; `Pending` is never minted — `open_running` always mints `Running`); the
foreground snapshot correctly targets the ACTIVE tab's focused pane (the `.grid().is_some()` guard
blocks `workspace()`'s first-terminal fallback); clear-on-view ordering correct (SET before CLEAR,
active-tab badges pruned same tick, bg-tab badges survive); the borrow is a legal disjoint place-borrow
(`shell` ≠ `notify_ticks` fields), `locate_pane` correctly two-phased; elapsed has no overflow (widen to
u64 before `*`); no regression (pure inserts; the rail ● is an added child); clean-room (reuses
`colors.success`/`danger`, generic `●`).

**Post-fix:** `cargo fmt` + `cargo check -p marley` clean (no warnings). F4+F5 → a follow-up ticket at
complete.

## Phase 4 — Validate
**Test written** (crates/marley_app/src/notify.rs `#[cfg(test)]`): `should_notify_gates_background_completions`
— the T1–T6 matrix (T1 at-threshold→Succeeded, T2 below→No, T3 above→Succeeded, T4 Failure→Failed,
T5 focused→No, T6 Running→No). T1/T2/T3 kill the 3 viable `<` mutants; T4/T5/T6 pin the un-mutated
focus/Failure/Running behaviors (kept per inspect F2).

**Ran (ACTUAL):**
- `cargo nextest run -p marley` → **307 passed, 2 skipped** (0.502s) — incl. the new test.
- Targeted mutation `cargo mutants -f notify.rs` → **4 mutants, 3 caught, 1 unviable** = cov/MSI 100 on
  `should_notify` (the 3 `<`-swaps caught; the whole-body `Default::default()` unviable, `Notify` has no
  Default derive — exactly the inspect F2 finding).

**Driven captures (live app, re-bundled after the inspect fixes) — 5 captures:**
- REQ-001/002-success (`203-bg-success.png`): ran `sleep 12` in a tab, ⌘D → new active tab (backgrounds
  the first), waited → the background tab's rail row showed a **green ● Marley**. ✓
- REQ-003 (`203-focused-none.png`): ran `sleep 12` in the FOCUSED tab, stayed on it → **no badge** on its
  row (the background tab's ● persisted, untouched). ✓ focus-gate + persistence.
- REQ-005 (`203-clears-on-view.png`): clicked the badged tab → switched to it → the **green ● cleared**
  (plain "Marley"). ✓ proves the F1 dirty-fix repaints the clear.
- REQ-002-failure (`203-bg-failure.png`): `sleep 12; false` (exit 1) in a background tab → a **RED ●**
  (colors.danger), visually distinct from green. ✓ success-vs-failure distinction.
- REQ-004 (quick→no badge): the trivial complement of REQ-001 (a quick command accrues too few ticks) —
  covered by unit T2; not separately captured.

**Gate:** `git add -A` (staged: app.rs, lib.rs, notify.rs, the 3 pipeline docs — no secrets/.mcp.json) →
`scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15 (rustfmt, clippy -D warnings, tests, coverage
≥100%, mutation MSI ≥100%, miri, audit, deny, machete, gitleaks, shellcheck, no-suppressions,
source-bans, docs, visual/AX). Receipt written for /commit. No pre-existing exclusions.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #203 entry under `[Unreleased] ### Added` (below #201).
docs/marley_architecture/app_shell.md — a #203 note after the #173 pump entry (the pure should_notify +
mutant nuance, the tick-counter shim, the disjoint-field borrow + two-phase locate_pane, the persistent
badge + clear-on-view dirty-fix, the deferred follow-ups).

**Forge capture (§19):**
- `aar-submit` ee7be81c — completed, effectiveness 5. Lessons: (a) the ticket's "elapsed already tracked
  / reuse duration.rs" was WRONG — a terminal `Block` tracks no time (only agents do, #187); the shim
  stamps elapsed via a per-pane pump-tick counter (deterministic, no `Instant`, no `PaneState`/borrow
  churn) — a design-phase correction; (b) the disjoint-field borrow (`view.notify_ticks` inside
  `for grid in view.shell.grids_mut()`) compiles cleanly (distinct RootView fields); the
  shell-re-borrowing `locate_pane` is two-phased after the loop, mirroring the dead-reap; (c) the
  persistent-until-viewed badge (not a timed flash) is better UX + drops the countdown/pure `TabFlash`.
- Inspect already recorded: `failure-record` **BF-claude-pump-state-change-without-dirty-repaint-001**
  (F1 — the clear-on-view didn't set `dirty`) + `prevention-rule-record`
  **PR-claude-pump-state-change-must-set-dirty-to-repaint-001**. Not duplicated here.
- Inspect F2 reinforced `PR-claude-cargo-mutants-guard-mutants-depend-on-syntactic-form` (an enum-return
  fn's body mutant is `Default::default()`, unviable without a `Default` derive — not the variants).

**Follow-up filed:** forge **#226** (f8200e06) — "M12.2 — #203 follow-ups: OS notification + notify-state
cleanup on manual close" (the deferred OS-notify adapter [D5] + F4 `notify_ticks` manual-close scrub +
F5 `tab_flashes` stable-keying).

**Close + archive:** forge ticket #203 → done. Local TICKET-203 → closed/. Pipeline doc pair →
completed/. Spec status → Phase 5 — Complete PASS.
