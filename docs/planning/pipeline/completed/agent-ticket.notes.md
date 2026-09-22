# agent ↔ ticket association — Notes

- **Forge ticket:** #80 `1b383541-41e3-44f6-9f56-956dafc8ffa1` · **AAR:** `0b0b6422-1e04-43a9-b3b9-65140962cf2d`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-080-agent-ticket.md

## Phase 1 — Plan
- **Request:** forge #80 (M2.E 7/15) — remember agent→ticket from a sent line.
- **Pre-flight:** send-to-agent @app.rs:1118, broadcast @1143; agent_badge/agent_rows in agent_view.rs.
- **Decisions:** D1 first #N digit-run; D2 store on delivered; D3 ` · #N` row / ` #N` badge.
- **AAR id:** `0b0b6422-1e04-43a9-b3b9-65140962cf2d`.

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
- **agent_view.rs:** `pub fn extract_ticket_ref(line) -> Option<u64>` = `line.split(char::from(0x23)).nth(1)?.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok()`; `AgentRow.ticket: Option<u64>` + agent_rows copies `run.ticket`; `agent_badge` → `match run.ticket { Some(n) => "{label} {glyph} #{n}", None => "{label} {glyph}" }`.
- **marley_agent:** `AgentRun.ticket: Option<u64>` (new()=None).
- **app.rs SHIM:** send-to-agent, inside `if sent {`: `if let Some(n)=extract_ticket_ref(&line) { if let Some(run)=self.agents.get_mut(&target) { run.ticket=Some(n) } }`. broadcast, inside `if delivered {`: same for every `agent_pane_ids`. Fleet render: a `#{n}` suffix after the label + the `· {last_line}` after the status.
- **Mutation targets:** the split-nth(1), the take_while digit run, the parse; the badge ticket arm; agent_rows ticket copy.
- **Test plan:** `extract_ticket_ref_cases` (mid/leading/none/#-alone/digits-then-letter/first-of-two/double-#); `agent_badge` ticket arm; agent_rows ticket copy (a fixture with Some). cov/MSI 100.
- **Risks:** store only on DELIVERED (#72 rule); overflow→None (parse().ok()).

## Phase 3 — Implement
- **Built:** `extract_ticket_ref(line)` (split # → digit-run → parse) + `AgentRow.ticket` + agent_rows copy + `agent_badge` ticket arm (agent_view); `AgentRun.ticket` (marley_agent); app.rs stores on delivered send/broadcast + the Fleet render shows `#N` after label. Fixtures got ticket (agent_rows "a"=Some(77) proves the copy).
- **Verification:** fmt; check --all-targets 0 err; clippy OK; agent_rows/badge tests pass. (extract_ticket_ref + badge-ticket tests → validate.)

## Phase 3.5 — Inspect
- **Method:** self-review (small pure extract_ticket_ref); gate cargo-mutants is the authoritative MSI check.
- **Lenses — no findings:** extract_ticket_ref (split '#' nth(1) = after the FIRST #; take_while(is_ascii_digit) = the digit run; parse().ok() → None on empty/overflow); the badge ticket arm + the agent_rows ticket copy (fixture Some(77) proves it); the store fires only on DELIVERED send/broadcast (#72 rule); broadcast tags every agent_pane_id. No panics (parse().ok()). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** extract_ticket_ref_cases (mid/leading/none/#-alone/#-nondigit/stops-at-nondigit/first-of-two/double-#) + agent_badge ticket case + agent_rows ticket copy (Some(77)). `cargo nextest` → pass.
- **Self-test (sent #77 → the row shows #77):** ENV-BLOCKED → engine-tested (cov/MSI 100).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(4); forge #80 → done. **M2.E 3/5.** extract_ticket_ref + AgentRun.ticket + badge/row #N (cov/MSI 100). Self-test env-blocked; engine-tested.
