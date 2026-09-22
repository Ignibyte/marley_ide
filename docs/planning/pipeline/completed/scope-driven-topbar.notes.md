# scope-driven-topbar — pipeline notes (forge #235, M13 sprint #26)

Pipeline: f89bd2fd-e6ce-4b0a-b08a-f31eba277fd7 · AAR: 5390a623-f206-495b-a62c-16ab0776a173
Ticket: forge#235 (ca7bdb07). 8th of the /work 228-237 train. Deps #233 (done). chad control granted → driven.

## Phase 1 — Plan (discovery)

**chad feedback #6+#7:** combine the top tabs on the left (they're global/workspace actions) + indicate
which workspace is focused in the top bar (the buttons correspond to the focused workspace).

**Discovery (inline):**
- Top-bar left icons: files (app.rs:6158), new-terminal (6186), new-agent (6208) — absolute at
  `topbar_icon_x(slot, TRAFFIC_LIGHT_INSET, ICON_GAP)` (slots 0/1/2).
- Cockpit tabs (Details/Agents/Forge): app.rs:6227-6270 — absolute `.top(px(4)).right(px(16))`, a flex
  row over `top_tabs(self.right_section)`, each an svg `section_icon` + the Agents count badge (#188).
- The focused workspace = the ACTIVE project (#233). Name = `self.shell.active_project().name`; branch
  = pure `titlebar::branch_from_git_head(project_root/.git/HEAD)` (already computed for the footer #142).
- `titlebar::titlebar_label(cwd, home, branch)` composes "cwd · branch" (the footer). The rail project
  row (#157) shows "name · branch". → the indicator mirrors this for the workspace NAME.

**Scope call:** the INDICATOR (pure `focused_workspace_indicator(name, branch)`, truncated) + render it
after the left icons + RELOCATE the cockpit tabs to a left x after it (all absolute; the indicator is
fixed-width-truncated so no reflow). DEFER: a top-bar switcher (rail #236 switches), a flex refactor.

**Driven plan (control granted):** boot (1 workspace) → open a 2nd (⌘O picker OR the launcher) → ⌘⇧]
switch focus → capture the indicator changing + the left-grouped cockpit tabs.

**ENV:** chad granted control → driven captures on; report at the end of the train.

## Phase 2 — Design
Pure `titlebar::focused_workspace_indicator(name, branch, max)` (mirrors `titlebar_label`'s `· branch`
compose + a char-count truncation to `max` with `…`). Shim: render it absolute at slot 3
(`topbar_icon_x(3,…)`, top 9, 13px, foreground, truncated to `TOPBAR_INDICATOR_MAX_CHARS=22`); relocate
the cockpit tabs `.right(px(16))` → `.left(px(topbar_icon_x(3,…) + TOPBAR_INDICATOR_WIDTH=180))`. The
active project name = `active_project().name`; branch = the same `.git/HEAD` read the footer uses.

## Phase 3 — Implement
titlebar.rs `focused_workspace_indicator` + `focused_workspace_indicator_cases` (6 cases). app.rs: the
titlebar import, the 2 consts, the indicator render block (after the #234 launcher branch → safe),
the cockpit-tabs `.left` relocation. `cargo fmt`/`check --all-targets` clean; **324 tests pass** (+1).

## Phase 3.5 — Inspect
1 critic (correctness/render-safety/mutation/layout) + self-review — **CLEAN, no CRIT/HIGH/MED**:
- MSI 100 — `cargo mutants -f titlebar.rs` = **35/35 caught** (the 9 `focused_workspace_indicator`
  mutants killed by the 6-case test; NO `saturating_sub` mutant — method calls unmutated, #199 lesson).
- Render safety CONFIRMED — the #235 indicator block (`active_project().name`) sits AFTER the #234
  launcher branch (render top) → unreachable at 0 workspaces (same class as #234's 3 bugs, correctly
  ordered here); the `.git/HEAD` read is total (`.ok()`).
- Relocation: only the container position moved (badge/handlers/loop unchanged); no const dead; no overlap.
- 1 LOW/info: the 22-char cap vs 180px reserve is a heuristic — but the tabs are ABSOLUTE so they never
  reflow regardless; no fix. Clean-room §20 clean.
**Phase 3.5 status: Inspect PASS.**

## Phase 4 — Validate
**Tests:** 324 pass (T1 `focused_workspace_indicator_cases`). **Gate:** `scripts/gates.sh --diff` →
**GATE GREEN [diff] 15/15**, cov + MSI 100. **DRIVEN capture (control granted):** `235-a-topbar.png` —
the indicator **"Marley · main"** after the action icons + the cockpit tabs (Details/Agents/Forge)
relocated to the LEFT cluster, no overlap with the center search → **REQ-002 + REQ-003 ✓**. REQ-004
(switch → indicator changes) mechanism-verified — the indicator re-reads `active_project().name` each
render, ⌘⇧] switches `active_project` (#233, tested); only 1 workspace open so no live switch to capture.
REQ-001 unit-tested. **Phase 4 status: Validate PASS.**
