# session row chrome (icon/title/subtitle) — Notes

- **Forge ticket:** #111 `f10d1452-200b-4efb-9480-01cf31197816` · **AAR:** `3e93a25f-f0a2-4a43-b665-7a29ae662e59`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-111-session-row-chrome.md

## Phase 1 — Plan
- **Request:** forge #111 (M5 5/12) — running-state icon + two-line rows (title + subtitle).
- **Pre-flight:** AgentRun.status: AgentStatus{Working,Waiting,Idle,Exited} kept current by the pump;
  agents map gives per-pane status. #109 set the subtitle but didn't render it → #111 renders it.
- **Decisions:** D1 3 distinct glyphs (✳/✧/▸); D2 running = Working||Waiting.
- **AAR id:** `3e93a25f-f0a2-4a43-b665-7a29ae662e59`.

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
- **sessions.rs (PURE):** Session gains `pub running: bool`; `pub fn session_row_icon(is_agent: bool, running: bool) -> &'static str` = match (is_agent,running) {(true,true)=>"✳"(U+2733), (true,false)=>"✧"(U+2727), (false,_)=>"▸"(U+25B8)}; session_rows' icon = session_row_icon(s.is_agent, s.running).
- **app.rs SHIM:** `use marley_agent::AgentStatus` (add to the import); the Session build sets `running: self.agents.get(&id).map(|r| matches!(r.status, AgentStatus::Working | AgentStatus::Waiting)).unwrap_or(false)`; the sidebar row → `div().flex().flex_row().gap_1().child(row.icon).child(div().flex().flex_col().child(row.title).child(div().text_size(px(11.0)).text_color(muted).child(row.subtitle)))` (icon + a title/subtitle column, subtitle muted+smaller). Keep the active-bg + click-focus.
- **Mutation targets:** the 3 session_row_icon arms; session_rows passing running.
- **Test plan:** session_row_icon_states (agent-active→✳, agent-idle→✧, terminal→▸); session_rows_uses_running (a running-agent row → ✳; an idle-agent → ✧; a terminal → ▸). Update sess_in for `running`. cov/MSI 100.
- **Risks:** the 3 glyphs must be distinct; running derives from the real AgentStatus (Working|Waiting); the two-line row must not overflow the dock width (title/subtitle truncate via the dock overflow_hidden).

## Phase 3 — Implement
- **Built:** Session.running + session_row_icon(is_agent,running) (3 glyphs ✳/✧/▸) in sessions.rs; session_rows uses it; sess_in gained running. app.rs imports AgentStatus + sets running (Working|Waiting) per agent + the sidebar row is now two-line (icon + a title/muted-subtitle column).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a small 3-arm icon fn + Session.running + a masked two-line render).
- **Lenses — no findings:** session_row_icon 3 distinct glyphs (agent-active ✳ / agent-idle ✧ / terminal ▸); session_rows now sources the icon from it (running flows in); running derives from the REAL AgentStatus (Working|Waiting, else false — a terminal is false); the two-line row renders title over a muted subtitle, truncated by the dock overflow_hidden. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** session_row_icon_states (4 combos → 3 glyphs) + session_rows_uses_running (idle/active agent + terminal icons) + the #109/#110 tests. `cargo nextest` → pass.
- **Self-test:** LIVE static capture (rows111.png) — the "▸ terminal 1" session row now has a muted "main" subtitle line beneath it (the two-line Warp row: icon + title + subtitle), under the WORKSPACE group. REQ-003 PASS. (▸ = a terminal, running=false; agents get ✳/✧.)
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #111 → done. **M5 5/12.** session_row_icon (3 glyphs) + Session.running (cov/MSI 100) + two-line sidebar rows (icon+title+subtitle) + running from AgentStatus. Live-proven (rows111.png).
