# launch an agent from the new-session affordance (M7) — Notes

- **Forge ticket:** #136 `a5c4b8e1-548f-4df0-8ef4-34fd6126c2d4` · **AAR:** `2f7c8955-0e7a-4bee-9274-8a8de5f309f1`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-136-agent-launch.md

## Phase 1 — Plan
- **Request:** forge #136 (M7 run 5/5 FINAL) — a 🧠 agent-launch icon → the tested "new-agent" path. Shim-only.
- **Pre-flight:** the "new-agent" dispatch (app.rs) = split_focused(spawn) + write launch_command(Claude) +
  AgentRun tag + last_agent; the #135 "+" icon; agents show in the sidebar + Agents cockpit (#91/#111).
- **Decisions:** D1 reuse "new-agent" via dispatch_action; D2 🧠 glyph after the "+".
- **AAR id:** `2f7c8955-0e7a-4bee-9274-8a8de5f309f1`.

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
- **app.rs (SHIM):** a 🧠 (ᾞ0) icon in the top bar at left:76 (after the "+") → on_mouse_down → self.dispatch_action("new-agent") + cx.notify. Shim-only (reuse the tested new-agent path).
- **Test plan:** none new (new-agent path + launch_command already tested).
- **Risks:** dispatch_action is &mut self (callable from the listener); placement after the + (left:46→76).

## Phase 3 — Implement
- **Built (app.rs SHIM):** a 🧠 icon in the top bar at top:5 left:76 (after the "+") → on_mouse_down → self.dispatch_action("new-agent") + cx.notify. Shim-only (reuse).
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (one icon reusing the tested new-agent dispatch).
- **Lenses — no findings:** the 🧠 click → dispatch_action("new-agent") — the exact tested M2.D path (split_focused spawn → write launch_command(Claude) → AgentRun tag → last_agent). The agent CLI launch stays via the existing seam (spawn_session in marley_command; launch_command pure/tested). Distinct glyph (🧠) vs the cockpit Agents 🤖. No unwrap/panic (dispatch_action swallows the split Result as before). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** none new (shim-only; the new-agent path + launch_command already tested).
- **Self-test:** LIVE capture (below) — the 🧠 agent-launch icon next to the "+".
- **Gate:** GREEN [diff] 15/15. LIVE capture (agent136.png): the 🧠 agent-launch icon renders in the top bar after the "+" (top-left cluster: 📁 + 🧠). REQ-001 PASS; click→dispatch_action("new-agent") code-reviewed (env-blocked), the new-agent path already tested.

## Phase 5 — Complete
- CHANGELOG; forge #136 → done. **M7 5/5 — sprint #18 COMPLETE.** A 🧠 agent-launch icon → the tested new-agent path. Top bar: 📁 + 🧠 · search · 📋🤖🔨. (Renamed docs to topbar-agent-launch to avoid the M2.D agent-launch name collision.) cov/MSI 100.
