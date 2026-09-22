# per-agent activity age — Notes

- **Forge ticket:** #82 `3172ec5c-98c7-4b46-b32c-7906a0d08d30` · **AAR:** `090a234f-45ce-4539-9a08-814eb8df1a77`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-082-agent-quiet-age.md

## Phase 1 — Plan
- **Request:** forge #82 (M2.E 9/15, FINALE) — "quiet Ns" age in the Fleet.
- **Pre-flight:** #79's `AgentRun.quiet_ticks` (the counter); agent_rows/AgentRow in agent_view.rs; the
  Fleet render (app.rs ~2276). Tick-count clock (Date::now banned, like #77).
- **Decisions:** D1 62 ticks/s, 0→""/<60→s/>=60→m; D2 age computed in agent_rows.
- **AAR id:** `090a234f-45ce-4539-9a08-814eb8df1a77`.

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
- **agent_view.rs:** `pub fn quiet_age(quiet_ticks: u32) -> String` = `let secs = quiet_ticks / 62; if secs == 0 { String::new() } else if secs < 60 { format!("quiet {secs}s") } else { format!("quiet {}m", secs / 60) }`; `AgentRow.age: String` = `quiet_age(run.quiet_ticks)` in agent_rows.
- **app.rs Fleet render:** the status segment becomes `({status}{age_suffix})` where `age_suffix = if row.age.is_empty() { String::new() } else { format!(" · {}", row.age) }` — placed inside the parens after status.
- **Mutation targets:** /62 divisor, secs==0, secs<60 boundary, /60.
- **Test plan:** `quiet_age_buckets` (0→""; 61→""; 62→"quiet 1s"; 3658→"quiet 59s"; 3720→"quiet 1m"; 7440→"quiet 2m"); agent_rows.age copy (fixture "a" quiet_ticks=3720 → age "quiet 1m"). cov/MSI 100.
- **Risks:** tick-count clock (Date::now banned); the age is coarse (per-pump).

## Phase 3 — Implement
- **Built:** quiet_age(quiet_ticks) (secs=ticks/62; 0→""/<60→"quiet Ns"/else "quiet Nm") + AgentRow.age = quiet_age(run.quiet_ticks) in agent_rows (agent_view); the Fleet render shows ` · {age}` inside the status parens. The agent_rows test "a" now has quiet_ticks 3720 → age "quiet 1m" (proves the copy).
- **Verification:** fmt; check --all-targets 0 err; clippy OK. (quiet_age_buckets direct test → validate.)

## Phase 3.5 — Inspect
- **Method:** self-review (a tiny pure quiet_age); the Phase-4 gate cargo-mutants is the authoritative MSI check.
- **Lenses — no findings:** quiet_age (secs=ticks/62; secs==0→"" [<1s], secs<60→"quiet Ns", else "quiet {secs/60}m"); the buckets/boundaries are pinned by the validate tests (0/61→""; 62→1s; 3658→59s; 3720→1m; 7440→2m); the age copy in agent_rows (the "a" fixture 3720→"quiet 1m" kills a constant-age mutant); no panics (integer div, u32). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** quiet_age_buckets (0/61→""; 62→1s; 3658→59s; 3720→1m; 7440→2m) + the agent_rows age copy (a→"quiet 1m"). `cargo nextest` → pass.
- **Self-test (quiet agent → "quiet Ns" climbing):** ENV-BLOCKED → engine-tested (cov/MSI 100).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #82 → done. **M2.E COMPLETE (5/5).** quiet_age + AgentRow.age (cov/MSI 100). Self-test env-blocked; engine-tested.
