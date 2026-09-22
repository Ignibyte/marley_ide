---
pipeline_id: 6471e72b-23b6-42fb-8ab1-863be04a95c1
ticket: forge#187 (ec3f3366-5130-4463-9157-eead7052982f)
aar_id: 8ef178ef-0572-4e79-96b1-154eca183df6
---

# Notes — M12 #187 agent lifecycle

## Phase 1 — Plan
Ticket #187 (M12 The Agent Cockpit). An exited agent lingers with a stale glyph and no result. Make the
death legible: the row stays, marked ✓/✕ + run duration, and a finish flash fires once. Deps: #174 agent
rows, #180 tail (the row surface), #67 the pump's dead-pane reap. PASS.

## Phase 2 — Design
**Approach.** Duration = tick count (Date::now is banned §14/anti-flake). AgentRun already ticks each pump
via refresh_agent_statuses (quiet_ticks) — add a monotonic `run_ticks` alongside, +1/tick WHILE alive, and
`exit_code: Option<i32>` (None = live, Some = exited). The pump already detects ChildExited (#67) but drops
the code — capture it. The row surface is pure (agent_view.rs) — fold the exit into AgentRow + two new pure
fns. secs = run_ticks/62 (the same ~62 ticks/s quiet_age uses).

**File manifest.**
- `crates/marley_agent/src/lib.rs` — AgentRun += `run_ticks: u32` + `exit_code: Option<i32>`; `new()` inits 0/None.
- `crates/marley_app/src/agent_view.rs` (PURE) — `fmt_duration(secs)`; `agent_finish_flash(label, code, secs)`;
  AgentRow += `exit: Option<(i32, u64)>`; `agent_rows` sets exit + picks ✕ when code≠0; `agent_row_text`
  appends " · exit N · DUR" when exited.
- `crates/marley_app/src/app.rs` (SHIM, cov-excluded) — the pump extracts ChildExited's ExitCode into a
  `Vec<(PaneId, Option<i32>)>`; a dead AGENT is marked Exited + exit_code + the flash ONCE (guard
  exit_code.is_none()), and its row is KEPT (the automatic reap skips agents.remove for an exited agent);
  refresh_agent_statuses increments run_ticks for a live agent.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `fmt_duration_buckets` | REQ-001 — 0s/45s/59s/60s→1m00s/134→2m14s/3599→59m59s/3600→1h00m/3723→1h02m (60/3600 boundaries) |
| `agent_finish_flash_reads_fields` | REQ-002 — "claude finished (exit 0, 2m14s)" |
| `agent_rows_marks_exit` (extend) | REQ-002 — an Exited run w/ exit_code Some(0)→✓ + exit suffix; Some(1)→✕; live→no suffix |
| driven capture | REQ-003 — exit an agent → row stays ✓ + duration + flash |
| gate --diff | REQ-004 |

**Risk.** The borrow in the pump: get_mut(&id) then set status_flash — end the run borrow first (compute the
flash string, then assign). The kept-row: only the AUTOMATIC pump reap keeps an exited agent; a user close
(close_tab_at) still removes it (the user asked). run_ticks must NOT keep climbing post-exit (guard on
exit_code.is_none()). PASS.

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean (only the pre-existing `block v0.1.6` future-incompat
warning). What shipped:
- `marley_agent/src/lib.rs` — AgentRun += `run_ticks: u32` + `exit_code: Option<i32>`; `new()` inits 0/None.
- `agent_view.rs` — `AGENT_TICKS_PER_SEC = 62` (quiet_age now references it too, same value); `fmt_duration`;
  `agent_finish_flash`; AgentRow += `exit: Option<(i32,u64)>`; `agent_rows` picks ✕ on a non-zero exit +
  fills exit=(code, run_ticks/rate); `agent_row_text` appends " · exit N · DUR".
- `app.rs` — pump `dead: Vec<(PaneId, Option<i32>)>`; ChildExited's `ExitCode(Option<i32>)` extracted via
  `find_map` (Err→None); the #187 capture marks a dead agent Exited + exit_code + fires the flash once
  (guard `exit_code.is_some()`, so a kept last-pane re-emitting ChildExited stays idempotent); the auto-reap
  DROPPED `agents.remove(&id)` so an exited row is KEPT; `refresh_agent_statuses` `continue`s on an exited
  agent (freezes duration + Exited status against is_command_running=false) and increments run_ticks/tick.
- Test literals (agent_rows_sorted_by_pane_id, agent_badge) got the two new fields to compile; NEW pure
  tests deferred to Phase 4.

**Deviation:** none. Borrow handled by returning the flash string OUT of the get_mut closure, then assigning
status_flash after the mutable borrow ends. last_agent left to the existing close-path clear (not touched in
the capture) — the tiny Case-A/Case-B ⌘⇧S-after-exit difference is immaterial and the row is visible either way.

## Inspect (Phase 3.5)
2 parallel general-purpose critics (correctness/lifecycle + state-integrity/simplification). Both verified
concretely (ran `cargo check --tests`, `cargo fmt -- --check`, traced every `self.agents` iterator). The
lifecycle logic passed ALL seven correctness risk-questions on BOTH critics (exit-once guard idempotent,
run_ticks frozen at exit, gone-pane degrades gracefully, no status-overwrite, -1 sentinel renders, fmt_duration
boundaries exact, no new panic). Findings + verdicts:

- **[HIGH] Test module didn't compile — 2 more AgentRow literals missed the new `exit` field** (`agent_row_text_full`,
  `agent_row_text_bare`). REAL (both critics, compiler-verified E0063). I updated 2 of the 4 test-literal sites,
  missed these 2. FIXED: added `exit: None,` to both. → failure-record + prevention rule.
- **[HIGH] `cargo fmt --check` failed (2 violations)** — agent_finish_flash's `format!` >60 cols; the trailing
  `//#187` comment on the run_ticks line re-indented the `#79` block. REAL. FIXED: wrapped the format!, moved the
  `#187` note to its own line above the statement; `cargo fmt` then clean (exit 0).
- **[HIGH/MED] Feature untested → MSI-100 will fail** — no tests for fmt_duration / agent_finish_flash / the ✕-✓
  glyph / the exit tuple / the row suffix. REAL but this is **Phase 4's job** (the design's Regression Test Plan
  already names exactly these). DEFERRED to validate; noted the precise kill-tests (fmt_duration 59/60/3599/3600/3723,
  glyph Some(1)→✕ & Some(0)→✓, exit tuple run_ticks=124→(_,2), the flash string, the suffix).
- **[LOW] Duplicated `run_ticks / RATE`** (agent_rows + the app.rs capture — drift risk). REAL/minor. FIXED:
  extracted `run_secs(&AgentRun) -> u64` in agent_view; both sites + the flash now read it.
- **[MED] Orphaned exited-agent entries unreclaimable until quit** (a non-last dead agent's entry is KEPT but its
  pane-id leaves every grid, so the retain-by-live-pane-id manual closes can't drop it). SIGN-OFF (not a bug): the
  ticket scopes "exited rows accumulate for the session; clear-finished OUT". Bounded in practice (you don't launch
  thousands of agents/session). A future "clear finished agents" affordance is the reclaim path.
- **[LOW] Footer "N agents" counts kept exited agents** (`agent_summary` = agents.len()). SIGN-OFF: consistent with
  the cockpit, which now lists the exited rows too; "0 working" makes liveness clear. `fleet_status_for` is
  grid-scoped so gone-pane entries never leak into a tab glyph (critic-verified).
- **[LOW] Same-frame multi-finish flash is last-writer-wins** (single status_flash slot). SIGN-OFF: matches the
  existing remote-disconnect flash pattern; each kept row still shows its own ✓/✕ + duration, so nothing is lost
  but the transient toast.

Lenses covered: correctness, lifecycle/edge-cases, async/borrow, state-integrity (every agents iterator), clean-room
/secrets, simplification/reuse, mutation-surface, fmt. Post-fix: `cargo fmt -- --check` exit 0; `cargo check --tests -p marley` clean.

## Phase 4 — Validate
**Tests added** (per the Regression Test Plan + the inspect kill-tests):
- `marley_agent::agent_run_new_starts_idle` extended — asserts `run_ticks == 0` + `exit_code == None` (REQ, the new defaults).
- `agent_view::fmt_duration_buckets` — 0/45/59/60/134/3599/3600/3723/7325 → the 60/3600 boundaries + `{:02}` pad (REQ-001).
- `agent_view::agent_finish_flash_reads_fields` — exit 0 / 1 / -1 strings (REQ-002).
- `agent_view::agent_rows_exit_marks_glyph_and_suffix` — Some(0)→✓ + "· exit 0 · 2s"; Some(1)→✕ + "· exit 1 · 2m14s"
  (run_ticks=62·134); a live agent → no suffix, ○ glyph (REQ-002; kills `code != 0`→`== 0`, run_secs `/`→`*`).

**cargo nextest run --workspace**: 748 passed, 5 skipped (foreground, in-transcript).

**Driven live-app capture (REQ-003)** — clean-booted the bundled app (Agents cockpit already the right dock), ⌘⇧A
launched a claude agent (footer "1 agent · 1 working", badge `claude ●`), killed the claude child (badge → `claude ○`,
"0 working" — the #79 status transition), then `exit 0` in the agent shell → `ChildExited(Some(0))`. Captured
(/tmp/mly187_exit0.png + _fleet.png), all firing at once:
- finish flash: **"claude finished (exit 0, 4m46s)"** (= agent_finish_flash("claude",0,286));
- pane badge → **`claude ✓`** (the exited glyph);
- rail row → **`✓ exit`** (kept, marked done);
- ⌘⇧E Fleet row → **"✓ claude (exited · quiet 4m) · … · exit 0 · 4m46s"** (agent_row_text suffix + fmt_duration live);
- the dead pane KEPT (last pane) showing `exit 0`, footer still "1 agent".

**Gate**: `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15. gate:4 coverage 100% lines; gate:5 **mutation 24
caught / 0 missed → MSI 100.0%**. First run surfaced ONE surviving mutant — `replace RootView::refresh_agent_statuses
with ()` (app.rs:1307): the `--diff` scope newly mutation-tested it because I'd modified it, and it's untestable
live-grid shim orchestration. Fixed at source per the codebase boundary: added `#[cfg_attr(test, mutants::skip)]`
matching the 79 sibling shim fns + the directly-preceding `agent_tail_lines` (#180) — NOT a floor-drop; every pure
decision it drives (agent_status_from, run_secs) stays mutation-tested at 100% in the pure crates. Re-run → green.

**Pre-existing exclusions**: none. (Restored `~/.marley/config/settings.toml` from the pre-test backup after the
driven run.)

## Phase 5 — Complete
(pending)
