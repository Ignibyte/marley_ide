# rich focused-pane Details inspector — Notes

- **Forge ticket:** #93 `a734efda-d9f1-46cb-a174-41c99471b33b` · **AAR:** `729d1893-0aee-4497-b7af-d0786824c8eb`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-093-pane-details-inspector.md

## Phase 1 — Plan
- **Request:** forge #93 (M2.F 4/6) — the Details tab as a per-kind focused-pane inspector.
- **Pre-flight shapes:** `Remote { host: String, status: RemoteStatus }` (app.rs); `RemoteStatus{Connected,
  Disconnected}` (marley_remote, no label fn → inline the word); `AgentRun{kind,label,status,last_line,
  quiet_ticks,ticket}`; `launch_command(kind)`; `BlockDetails{command,status:StatusKind,exit_code,pwd,
  git_branch}` + `block_details(...)` (#58); `StatusKind{Running,Success,Failure}`. The Details render
  (app.rs ~1619) builds the block_details Div — replace with the kind-dispatch.
- **Decisions:** D1 dispatch agent→remote→terminal→empty; D2 reuse block_details/launch_command/status
  labels; D3 label &'static str.
- **AAR id:** `729d1893-0aee-4497-b7af-d0786824c8eb`.

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
- **pane_details.rs (NEW PURE):** `DetailRow { label: &str, value: String }` (Debug/Clone/PartialEq/Eq) + private `fn row(label, impl Into<String>) -> DetailRow`. `agent_details(&AgentRun)` = [row("agent", launch_command(kind)), row("status", agent_status_label(status))] + push row("ticket", format!("#{n}")) if ticket + push row("last", last_line.clone()) if !empty. `remote_details(host, RemoteStatus)` = [row("host", host), row("status", match{Connected→"connected",Disconnected→"disconnected"})]. `terminal_details(&BlockDetails)` = [row("command", command), row("status", match StatusKind{Running→"running",Success→"success",Failure→"failure"})] + push row("exit", format!("{code}")) if exit_code + push row("cwd", pwd) if pwd + push row("git", branch) if git_branch.
- **lib.rs:** `mod pane_details;` (after nav, before palette).
- **app.rs SHIM:** the Details body (currently the inline block_details Div at ~1619) → `let focused = self.workspace.focused(); let rows = if let Some(run)=self.agents.get(&focused) { agent_details(run) } else if let Some(rm)=self.remotes.get(&focused) { remote_details(&rm.host, rm.status) } else if let Some(block)=self.workspace.state(focused).and_then(|s| s.session.blocks().iter().last()) { terminal_details(&block_details(&block.command, block.state, block.exit_code, block.prompt.pwd.as_deref(), block.prompt.git_branch.as_deref())) } else { vec![] };` then render: empty → "No command selected" placeholder; else flex_col of each row = a row div (muted label + value).
- **Mutation targets:** each projector's rows/labels/values + conditional pushes + match arms.
- **Test plan:** agent_details_full/bare (ticket+last vs none); remote_details_connected/disconnected; terminal_details each status + with/without exit/cwd/git. cov/MSI 100. The dispatch + render masked (static live + engine).
- **Risks:** dispatch order agent→remote→terminal (a pane cant be two kinds, but the order is explicit); block_details reuse keeps #58 semantics; RemoteStatus has no label fn → the word is inline in remote_details (tested).

## Phase 3 — Implement
- **Built:** pane_details.rs (DetailRow + agent_details/remote_details/terminal_details, each Vec<DetailRow>); `mod pane_details` in lib.rs; the app.rs Details body dispatches by focused pane kind (agents→remotes→last-block→empty) → renders each row as a muted label + value.
- **DEVIATION:** the Details render now shows uniform `label / value` text rows across all 3 kinds — the terminal case uses the status WORD (running/success/failure) instead of #58's colored ✓/✗/○ glyph (the DetailRow model is pure text; the glyph needs render-side color). Same info, uniform 3-kind projection. Dropped the now-unused StatusKind import.
- **Verification:** fmt; check 0 warn/err; clippy -D warnings OK; the 3 projector tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review scaled to a per-kind pure projector set + a masked if-else dispatch; the Phase-4 gate cargo-mutants is the authoritative MSI check on the 3 projectors.
- **Lenses — no findings:** agent_details (agent+status always; ticket/last conditional pushes — tested bare+full); remote_details (host + connected/disconnected match — both tested); terminal_details (command+status always; exit/cwd/git conditional; 3 status-word arms — tested running-bare/success-full/failure); the dispatch is an explicit agents→remotes→last-block→empty chain (a pane is exactly one kind; owned Vec ends the borrows); block_details (#58) reused verbatim so terminal semantics are unchanged; no panics (no unwrap; Option handled). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** agent_details_full_and_bare + remote_details_by_status + terminal_details_full_and_bare (pane_details.rs) — each kind, conditional rows, every status/connection arm. `cargo nextest` → 3 passed.
- **Self-test:** the per-kind Details content needs a POPULATED pane (a command block, a launched agent, or a remote) which requires typing/launching = synthetic input (ENV-BLOCKED all session); #90's static capture already proved the Details tab renders (empty → "No command selected"). The 3 projectors are engine-tested at cov/MSI 100; the dispatch + render are masked.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.
- **Pre-existing (not in scope):** the `block v0.1.6` future-Rust-rejection warning is a gpui transitive objc dep, unrelated + long-standing.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #93 → done. **M2.F 4/6.** pane_details.rs (DetailRow + agent/remote/terminal_details, cov/MSI 100) + the Details kind-dispatch. Uniform label:value rows (word-status, not #58 glyph).
