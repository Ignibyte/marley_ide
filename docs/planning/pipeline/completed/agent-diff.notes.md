# the agent-diff view — Notes

- **Forge ticket:** #104 `9acb6d71-1661-4727-ab57-ca07ab288f29` · **AAR:** `7bea4cd9-8d1f-492b-a3d5-107bc18f573f`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-104-agent-diff.md

## Phase 1 — Plan
- **Request:** forge #104 (M4 8/10, the payoff) — ⌘-click an agent → the working diff + a +/- summary.
- **Pre-flight:** #102's git_working_diff + the ⌘⇧D overlay + FileDiff/DiffKind exist; the Agents-section
  row click is at app.rs ~1905 (→ focus); add a ⌘-click branch → diff.
- **Decisions:** D1 count Add/Remove; D2 v1 = current working diff; D3 ⌘-click=diff / plain=focus.
- **AAR id:** `7bea4cd9-8d1f-492b-a3d5-107bc18f573f`.

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
- **git_diff.rs (PURE):** `agent_diff_summary(files) -> String` = empty→"no changes"; else adds/removes = count Add/Remove DiffLines across files→hunks→lines; `format!("{} files · +{adds} −{removes}", files.len())` (U+2212 minus).
- **app.rs SHIM:** import agent_diff_summary; the ⌘⇧D overlay header → agent_diff_summary(files) (replaces "working diff (N files)"); the Agents-dock-section row on_mouse_down (~1905): capture `event`, `if event.modifiers.platform { view.diff = Some(view.git_working_diff()); view.status_flash = Some(Flash::new("agent diff")); } else { focus+jumped-to flash }`.
- **Mutation targets:** agent_diff_summary empty, files.len(), the Add/Remove count arms.
- **Test plan:** agent_diff_summary_cases (empty→"no changes"; 1 file 2-add/1-remove→"1 files · +2 −1"; 2 files aggregate). cov/MSI 100. The header + ⌘-click masked.
- **Risks:** v1 = repo-wide working diff (not per-agent); ⌘-click on the Agents row (plain still focuses); the − is U+2212 (not ascii -).

## Phase 3 — Implement
- **Built:** agent_diff_summary (git_diff.rs — "no changes" / "N files · +A −R" counting Add/Remove); the ⌘⇧D overlay header uses it; the Agents-section agent row gains a ⌘-click → self.diff = git_working_diff() + "agent diff" flash (plain-click still focuses).
- **Verification:** fmt; check 0 err; clippy OK; agent_diff_summary test passes.

## Phase 3.5 — Inspect
- **Method:** self-review (a small pure aggregator + a masked ⌘-click affordance reusing #102/#98 patterns).
- **Lenses — no findings:** agent_diff_summary (empty→"no changes"; else files.len() + Add/Remove counts aggregated across files→hunks→lines, Context ignored — tested empty/1-file/2-file); the ⌘-click reuses the proven #102 git_working_diff (read-only) + opens the #103 overlay; plain-click still focuses (mirrors #98 tree); the header shows the summary. No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** agent_diff_summary_cases (empty/1-file/2-file). `cargo nextest` → pass.
- **Self-test:** ⌘-click an agent → the diff needs a synthetic click (ENV-BLOCKED); agent_diff_summary engine-tested cov/MSI 100; the affordance masked (+ #102 live git).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #104 → done. **M4 8/10 — the payoff.** agent_diff_summary (cov/MSI 100) + ⌘-click an agent → the working diff + the +/- header summary. v1 = repo-wide working diff.
