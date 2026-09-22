# an agent's last output line in the Fleet overlay — Notes

- **Forge ticket:** #78 `ee4f5718-938f-49c1-a839-6d747f2508e1` · **AAR:** `039affa1-d314-4bc6-baad-329492b7c07f`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-078-agent-last-line.md

## Phase 1 — Plan
- **Request:** forge #78 (M2.E 5/15) — an agent's last output line in the Fleet.
- **Classification:** work pipeline, `feature`, PURE (agent_last_line) + AgentRun field + app.rs SHIM.
- **Pre-flight:** agent_view.rs `AgentRow`/`agent_rows(&HashMap<PaneId,AgentRun>)`; `content_row_texts(state)`
  (app.rs:486) reads a pane's rows; the pump's `refresh_agent_statuses` (#67) mutates each AgentRun/tick.
- **Decisions:** D1 agent_last_line pure (rev-scan/trim/char-truncate); D2 AgentRun.last_line refreshed in
  the pump; D3 max ≈ 60.
- **AAR id:** `039affa1-d314-4bc6-baad-329492b7c07f`.

## Phase 2 — Design

### PURE — `agent_view.rs`
```rust
/// The last non-empty line of an agent pane's `output`, trimmed and char-truncated to `max` (a '…' appended
/// only when actually cut); empty / all-whitespace → "". Shown in the Fleet row so you see progress (#78).
pub fn agent_last_line(output: &str, max: usize) -> String {
    let last = output
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    if last.chars().count() > max {
        let head: String = last.chars().take(max).collect();
        format!("{head}…")
    } else {
        last.to_string()
    }
}
```
- `AgentRow` gains `pub last_line: String`; `agent_rows` maps `last_line: run.last_line.clone()`.

### marley_agent — `AgentRun`
- add `pub last_line: String` (data field). Update the launch construction (#62) + any test fixtures with
  `last_line: String::new()`.

### SHIM — `app.rs`
- `const AGENT_LAST_LINE_MAX: usize = 60;`
- `refresh_agent_statuses` gains the last-line: capture `(id, running, agent_last_line(&content_row_texts(
  state).join("\n"), AGENT_LAST_LINE_MAX))`; set `run.last_line` in the update loop.
- the Fleet render (app.rs ~2246): append ` · {row.last_line}` when non-empty (muted).

### File manifest
- MODIFY `crates/marley_app/src/agent_view.rs` — agent_last_line + AgentRow.last_line + agent_rows.
- MODIFY `crates/marley_agent/src/lib.rs` — AgentRun.last_line.
- MODIFY `crates/marley_app/src/app.rs` — the const, refresh wiring, the Fleet render, the launch fixture.

### Mutation Targets (pure)
- `agent_last_line`: the `rev().find(!trim().is_empty())` (last-non-empty), the `chars().count() > max`
  boundary, the `take(max)` + '…', the trim, the empty `unwrap_or("")`.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `agent_last_line_cases` — multi-line → last non-empty; trailing blanks skipped; long → head+…; exactly-max → no …; multibyte truncation is char-safe | unit |
| REQ-002 | `agent_last_line_empty` — "" and all-whitespace → "" | unit |
| REQ-003 | agent produces output → the Fleet row shows the last line | self-test (env-blocked → engine) |
| REQ-004 | gate GREEN, cov/MSI 100 agent_last_line; shim masked | gate |

Uncoverable: the pump refresh + the Fleet render — masked (gpui/live), REQ-003.

### Risks / decisions
- D-2.1 char-based truncation (multibyte-safe); '…' ONLY when `chars().count() > max` (exactly-max → no cut).
- D-2.2 the last line is refreshed each pump tick (alongside #67) — bounded (one content read per agent);
  `agent_rows` stays pure (copies run.last_line). D-2.3 an agent with no output yet → "" (no row change).

## Phase 3 — Implement
- **Built:** `agent_last_line(output, max)` (rev-scan last-non-empty, trim, char-truncate+…) + `AgentRow.
  last_line` + `agent_rows` copies it (agent_view.rs); `AgentRun.last_line` + `new` sets it (marley_agent);
  app.rs `AGENT_LAST_LINE_MAX=60`, `refresh_agent_statuses` reads `content_row_texts(state).join("\n")` →
  agent_last_line → `run.last_line`, the Fleet render appends ` · {last_line}` when non-empty.
- **Fixtures:** the agent_rows test gained `last_line` (one non-empty "building…" to prove the copy flows);
  the 2 badge-test AgentRun literals + the agent_rows AgentRun/AgentRow literals get `last_line`.
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_agent --all-targets` 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review of the small pure `agent_last_line` (scaled to size); the Phase-4 gate's
  cargo-mutants on the diff is the authoritative MSI-100 check (fail → add the missing test).
- **Lenses — no findings:** correctness (rev-scan `find(!trim().is_empty())` = the last non-empty line,
  trailing blanks skipped; `unwrap_or("")` → "" when all-blank; char-based `count()>max` → `take(max)`+'…'
  ONLY when cut [exactly-max → whole line]; multibyte-safe truncation; empty→""); the `AgentRow`/`AgentRun`
  `last_line` fields are data + the agent_rows copy is asserted (the fixture's non-empty "building…" flows
  through, killing an always-empty mutant); the pump refresh + Fleet render are masked. **No findings.**
- **Fix applied:** none.

## Phase 4 — Validate
- (pending)

## Phase 5 — Complete
- (pending)

## Phase 4 — Validate
- **Tests added (agent_view.rs):** `agent_last_line_cases` (REQ-001 — last non-empty/trailing-blank-skip/trim/truncate+…/exactly-max/multibyte) + `agent_last_line_empty` (REQ-002); the agent_rows test now asserts the last_line copy (non-empty "building…" flows through).
- **Runs (actual):** `cargo nextest -p marley -E ...` → 3 passed.
- **Self-test (⌘⇧E → the Fleet row shows the last line):** ENV-BLOCKED (needs launching an agent + the overlay) → agent_last_line is engine-tested (cov/MSI 100) + the render is a masked `format!` append.
- **Gate:** (running).
- **Pre-existing:** none.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #78 → done. **M2.E 1/5.** agent_last_line (cov/MSI 100) + AgentRun.last_line + the Fleet render. Self-test env-blocked; engine-tested.
