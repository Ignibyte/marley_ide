# M9 seq-8 — live rail titles + real branch — Notes

- **Forge ticket:** #157 `13cbc967-15c4-4712-9d62-3f9a27ca8171` · **AAR:** `d2091c32-0201-4af7-8ebc-913a24f9242c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-157-live-titles.md

## Phase 1 — Plan
- **Request:** forge #157 (M9 run 8/8, FINAL) — live rail tab titles + branch subtitle.
- **Pre-flight:** branch_from_git_head (titlebar.rs:28) exists; the titlebar already shows the real branch (no
  hardcoded "main"). The rail Tab row uses tab.title (static "terminal N"); a terminal's latest command =
  grid→focused terminal→blocks().last().command.
- **Decisions:** D1 rail_tab_title shows the program (first token) else fallback; D2 only terminal tabs go live;
  D3 project branch per-render from root/.git/HEAD (name-only if none).
- **AAR id:** `d2091c32-0201-4af7-8ebc-913a24f9242c`.

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
### Approach
PURE fn in titlebar.rs + a masked shim render (reuse the rail render + branch_from_git_head).
- **titlebar.rs (PURE, cov/MSI 100):**
  `pub fn rail_tab_title(command: Option<&str>, fallback: &str) -> String` =
  `command.map(str::trim).filter(|c| !c.is_empty()).and_then(|c| c.split_whitespace().next())
  .map(str::to_string).unwrap_or_else(|| fallback.to_string())` — the program token, else the fallback.
- **app.rs (SHIM masked):**
  - `fn live_tab_title(&self, project, tab, fallback: &str) -> String` — navigate
    `shell.projects().get(project) → .tabs().get(tab) → .grid() → g.terminal(g.focused()) →
    term.session.blocks().iter().last() → b.command.as_str()` then `rail_tab_title(cmd, fallback)`. A code/
    cockpit tab has no grid → None → fallback (its file/section title, unchanged).
  - The rail `RailLevel::Tab` arm: `let label = self.live_tab_title(row.project, row.tab.unwrap_or(0),
    &row.label);` and render `label` (+ the filter still matches on the LIVE label). Keep the click/active.
  - The rail `RailLevel::Project` arm: read `projects()[p].root/.git/HEAD` → `branch_from_git_head` →
    `format!("{name} · {branch}")` when Some, else `name`. (branch_from_git_head is already pure+tested.)

### File manifest
- `crates/marley_app/src/titlebar.rs` — PURE: rail_tab_title + tests.
- `crates/marley_app/src/app.rs` — SHIM: live_tab_title; the rail Tab arm (live label) + Project arm (branch subtitle).

### Regression Test Plan
| Test (titlebar.rs) | AC |
|---|---|
| rail_tab_title_cases — Some("cargo build --lib")→"cargo"; Some("  ")→fallback; None→fallback; Some("vim")→"vim"; Some("  git status ")→"git" | REQ-001 |
| DRIVEN: run `sleep 30` in a terminal → the rail tab row shows "sleep" | REQ-002 |
| DRIVEN: the project row shows "Marley · main" (the real branch) | REQ-003 |
| gate: rail_tab_title cov/MSI 100; app.rs shim excluded | REQ-004 |

### Risks
- live_tab_title nav uses .get() (no panic on a stale index) + Option chaining (no panic on an empty grid /
  no blocks). A code/cockpit tab → None → fallback (keeps its title).
- The "Search tabs" filter now matches the LIVE label (running command) — acceptable / arguably better.
- Reading .git/HEAD per project per render is cheap (few projects); a detached HEAD → branch_from_git_head None
  → name-only.
- A fresh terminal's latest block may have an empty command → rail_tab_title → fallback (blank filtered).

## Phase 3 — Implement
- **titlebar.rs PURE:** rail_tab_title(command, fallback) — trim → filter non-empty → split_whitespace().next() (the program) → else fallback.
- **app.rs SHIM (masked):** live_tab_title(project, tab, fallback) — navigate projects().get→tabs().get→grid→terminal(focused)→session.blocks().last().command → rail_tab_title. The rail Tab arm uses live_tab_title (+ the filter now matches the live label). The rail Project arm shows "name · branch" (branch_from_git_head from projects()[p].root/.git/HEAD; name-only if none).
- **Verify:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Phase 4 — Validate
- **Tests:** rail_tab_title_cases (REQ-001: "cargo build --lib"→"cargo", "vim"→"vim", "  git status "→"git" [leading-ws skipped], "   "/""/None→fallback). Simplified the fn (dropped redundant trim+filter — split_whitespace already skips leading ws + yields None on all-ws — to avoid surviving mutants). titlebar 4/4 pass.
- **Self-test:** live.png — the rail PROJECT row shows the REAL branch: "Marley · main" (git repo) AND "Documents" (NON-git → name-only, no ·) → REQ-003 CONFIRMED both branch + no-branch. The tab rows correctly show the FALLBACK "terminal 1" when no command has run (live_tab_title fallback path, live). BONUS: live2/live3 accidentally proved multi-project (#156) — 2 projects (Marley + Documents) in the rail; the active (Documents) drives the titlebar "~/Documents" + the terminal cwd. LIMITATION (documented): synthetic TYPING (drive.swift type:) does NOT register in the harness this session — a diagnostic (diag_type.png) showed the search box FOCUSED (cyan border) but typed text never appears; clicks + chords (⌘O) work, typing does not. So REQ-002 (a RUNNING command in the tab row) cannot be pixel-proven — it is verified by rail_tab_title cov/MSI 100 (the command→program derivation) + the fallback path shown live + critic verification of live_tab_title. Honest harness boundary, not a skip.
- **Gate:** GREEN [diff] 15/15 (rail_tab_title cov/MSI 100; app.rs shim excluded). Critic NO correctness bugs — 5/5 clean.

## Inspect (Phase 3.5)
Method: 1 background critic (rail_tab_title edges + live_tab_title nav + branch IO + the live filter) + my review + the driven captures.

- **Critic — NO correctness bugs; all 5 checks CLEAN.**
  1. rail_tab_title: non-blank command always yields the program token (never falls back); only empty/all-ws/None
     hit the fallback; total (no unwrap/index); NOT mutants::skip → mutation-gated + the 6-assert test kills the mutants.
  2. live_tab_title: every step is .get()/​.and_then() → a stale index / non-terminal tab / empty grid / no blocks
     all → fallback; borrow-check clean (PaneId Copy); code/cockpit tabs keep their title (UNCHANGED).
  3. Branch subtitle: detached HEAD / non-repo → name-only (no panic); `name · branch` iff Some.
  4. The live-label filter is coherent — terminal titles are always generic ("terminal N", no rename path), so
     no meaningful user title is ever hidden.
- **[INFO — improvement] simplified rail_tab_title during implement** — dropped the redundant `trim`+`filter`
  (split_whitespace already skips leading ws + yields None on all-ws). Behaviorally identical + kills more mutants.
- **[LOW/NIT — accepted] per-render fs read** of .git/HEAD for the branch subtitle matches the EXISTING titlebar
  pattern (app.rs:1247) — an established idiom, graceful degradation for worktree/submodule projects.

Lenses: pure derivation edges, the nav no-panic/borrow, branch IO/detached-HEAD, the filter interaction. No blocking findings.

## Phase 5 — Complete
- CHANGELOG + app_shell seq-8 note (M9 FINALE); forge #157 → done. **M9 8/8 — sprint #20 CLOSES.** Live rail tab titles (rail_tab_title) + real branch subtitle (branch_from_git_head reuse). LESSON: prefer the minimal expression — redundant ops (trim+filter when split_whitespace suffices) are surviving-mutant magnets; synthetic typing not drivable this session (documented).
