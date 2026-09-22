# workspace-foundation — pipeline notes (forge #233, M13 sprint #26)

Pipeline: ee3b98bb-2824-4d09-b22d-5fe26dc7e8a4 · AAR: 159c2334-5c9e-41bf-8e46-6287ff285e2e
Ticket: forge#233 (e29e08e4-e554-474b-93a9-891128b4a370). The spine of the M13 workspace re-arch
(#234-237 dep on it). 6th of the /work 228-237 train (1st of the model tier).

## Phase 1 — Plan (discovery)

**chad direction (2026-07-10):** Marley goes workspace-centric (IDE/PhpStorm-like). #233 = the model +
focus foundation; #234 launcher, #235 top bar, #236 rail, #237 editor tabs. chad asked me to PIN the
terminology + said "go with it" (VETOABLE at review). Auto-approved (/work 228-237).

**Discovery (background Explore over the current model):**
- **The top level is a SINGLE `tabs::Workspace<TerminalSession>`** (`RootView.shell`, app.rs:122):
  `Workspace { name, projects: Vec<Project>, active: usize }` (tabs.rs:269). Nesting: Workspace →
  Project(repo: root/tabs/active) → Tab(TabContent) → PaneGrid.
- **There is NO first-class focus + NO multi-Workspace.** The ENTIRE focus model today = 3 nested
  `active: usize` cursors (`Workspace.active`=active project, `Project.active`=active tab,
  `PaneGrid.focused`=pane). "#156 multi-project" = `Vec<Project>` + `Workspace.active` — so "multiple
  workspaces open, one focused" ALREADY maps to multi-project + the active cursor.
- **Switching the active project** = `Workspace::switch_project(idx)` (tabs.rs:307), driven ONLY by the
  left-rail clicks (app.rs:4130/4267/4309) + boot/agent-jump. **NO keybinding cycles projects** (only
  `next-tab`/`prev-tab` cycle TABS within a project) → the workspace-cycle is a genuine gap #233 fills.
  `sync_active_project` (app.rs:2645) rebuilds Files/⌘P/git-root/titlebar on switch.
- **Naming collision ALREADY exists:** TWO `Project` types — `tabs::Project<S>` (container) +
  `marley_project::Project` (git discovery, lib.rs:13). app.rs imports the latter BARE + writes the
  former fully-qualified. So `tabs::Project → Workspace` must first disambiguate → rename is high-risk.
- **Never-empties guards (M10):** pure `Project::close_tab → LastTab/LastTerminal` (tabs.rs:151-157),
  `Workspace::close_project → LastProject` (tabs.rs:298); shim flashes (app.rs:2778/2809); boot seeds ≥1
  (app.rs:919-994); `workspace()`/`_mut()` `.expect("≥1 terminal")` (app.rs:2227/2236). #234 relaxes
  these for the launcher's empty state.
- **Pure seams (cov/MSI 100):** `tabs.rs` (the Workspace/Project algebra + `next_index`/`prev_index`
  :504/514 + `adjust_active` :404 + `switch_project`/`active_project_index`/`project_count`) — EXTEND
  this. `grid_layout.rs` (the persistence codec) — only if the cycle needs persistence (it doesn't; the
  active-project index already persists via `ShellLayout.active_project`).
- **Rename blast radius:** `Workspace` type → 2 files (tabs.rs 23 + app.rs 12) = tractable; `Project`
  type → ~69 refs / 9 files + the 2-type collision = the risky one. → DEFER the rename (D2).

**Scoping call (D1-D4 in the spec).** The container + multi-open + the active cursor already exist, so
#233 is a CONTAINED foundation: (1) terminology PIN (vetoable; rename deferred), (2) a pure workspace
CYCLE (`Workspace::cycle`-style over `project_count`, reusing next_index/prev_index) + formalizing the
focus/switch algebra, cov/MSI 100, (3) a minimal shim = a `next-workspace`/`prev-workspace` keybinding
routing through the algebra + `sync_active_project`. DEFER (each to its owner): the type rename (own
pass), the "none-focused" empty state + guard-relaxing + fallible `workspace()` (#234 launcher).

**Why conservative:** the terminology is VETOABLE + the rename is collision-prone + chad's AFK/live —
so #233 pins the words + builds the model WITHOUT renaming, making a terminology veto a doc edit (not a
75-ref revert). Nothing pushes without chad's OK; he vetoes/reshapes at the push review.

**Deps:** M9 Workspace→Project→Tab (#150-157), #156 multi-project. **ENV:** chad live + screen-sharing
→ driven capture deferred, mechanism-verify + offer vibe-check (same stance as #228-232).

**Open question for chad (surfaced, not blocking — he delegated + auto-approved):** is the workspace
terminology (Session/Workspace/Tab) + the rename-deferral right, and is a workspace-cycle keybinding the
right #233 slice? Recorded in the #233 plan; he can veto/reshape at the push review.

**AskUserQuestion (model tier + push) — chad AFK (60s no answer).** Proceeded on best judgment:
build #233 as scoped (conservative — the recommended option); HOLD the push (needs explicit OK; prior
approval doesn't carry). Nothing pushes without his word.

## Phase 2 — Design

**Chord (confirmed free):** `next-workspace` = ⌘⇧] `chord(true,false,false,true,"]")`, `prev-workspace`
= ⌘⇧[ `chord(true,false,false,true,"[")` — a level-up parallel to the tab chords ⌘]/⌘[ (keymap.rs
174/183). All ⌘⇧-letter chords are taken (P/L/J/B/R/A/F/E/S/G/O/D) but ⌘⇧-bracket is free.

**Pure (tabs.rs) — the switch algebra, cov/MSI 100:**
```rust
/// Cycle the focused workspace to the next (`forward`) / previous open one, wrapping. Returns whether
/// the focus moved (false when only one is open — the wrap is a no-op). (M13 #233 — the shim binds it
/// to next-workspace/prev-workspace; a project IS chad's "workspace", D1.)
pub fn cycle_project(&mut self, forward: bool) -> bool {
    let current = self.active;
    let target = if forward { next_index(current, self.projects.len()) }
                 else       { prev_index(current, self.projects.len()) };
    if target == current { return false; }
    let _ = self.switch_project(target); // target is a valid in-range index → Ok
    true
}
```
Reuses the tested `next_index`/`prev_index` (:504/514) + `switch_project` (:307). NOT inline in the shim
(next-tab does it inline; a named pure method gives #233 its testable "switch algebra" + a single-source
for #235/#236 to reuse).

**Shim:**
- `keymap.rs` (PURE data, cov 100) — 2 bindings (⌘⇧]→next-workspace, ⌘⇧[→prev-workspace) + 2 mirror
  `action_for` test assertions (like next-tab/prev-tab :357/:371). `chords_unique` stays green (free).
- `app.rs` dispatch — 2 arms mirroring `next-tab` (:3138) but calling the pure cycle + syncing (a
  WORKSPACE switch changes the active project → Files/⌘P/git/titlebar, unlike a TAB switch):
  ```rust
  "next-workspace" => { if self.shell.cycle_project(true)  { self.sync_active_project(); cx.notify(); } }
  "prev-workspace" => { if self.shell.cycle_project(false) { self.sync_active_project(); cx.notify(); } }
  ```

**Terminology doc (REQ-003):** an app_shell.md note — the Session/Workspace/Tab vocabulary pinned (D1),
the code rename deferred (D2). (Written at Complete with the CHANGELOG.)

**File manifest:** `tabs.rs` (+cycle_project +3 tests) · `keymap.rs` (+2 bindings +2 asserts) · `app.rs`
(+2 dispatch arms) · `app_shell.md` (terminology + cycle note, at Complete).

**Regression Test Plan:**
| # | test (file) | REQ | asserts |
|---|---|---|---|
| T1 | `cycle_project_forward_wraps` (tabs.rs) | REQ-001 | 3 projects: 0→cycle(true)→1 =true; 2→cycle(true)→0 (wrap) =true |
| T2 | `cycle_project_backward_wraps` (tabs.rs) | REQ-001 | 0→cycle(false)→2 (wrap) =true; 1→cycle(false)→0 =true |
| T3 | `cycle_project_single_noop` (tabs.rs) | REQ-001 | 1 project: cycle(true)/cycle(false) → active stays 0, both return false |
| T4 | keymap asserts (keymap.rs) | REQ-002 | action_for(⌘⇧])==Some("next-workspace"); action_for(⌘⇧[)==Some("prev-workspace") |
| T5 | dispatch (shim) | REQ-002 | review + driven capture env-permitting: a switch re-syncs Files/git/titlebar |
| T6 | no-guard-relaxed (grep) | REQ-004 | no edit to tabs.rs:151-157/298 or app.rs:2778/2809/2227; guard tests still green |

**Mutants (cycle_project)** — the ACTUAL set (`cargo mutants --list -f tabs.rs`, confirmed at inspect)
is exactly 3: the `-> bool` body `→true` (killed by T3 single-noop, expects false), `→false` (killed
by T1, expects true), and the `target==current` guard `==`→`!=` (killed by T3: it would switch+return
true). The `if forward` is NOT mutated (cargo-mutants doesn't mutate a bare-bool `if`), and the
next_index/prev_index/switch_project calls are UNMUTATED — so my design-time "forward swap-branch"
prediction was off, but the real 3 are all killed → cov/MSI 100. (Lesson mirrors #232: predicted
mutants ≠ ground truth; `--list` the real set.)

**Phase 2 status: Design PASS.**

## Phase 3 — Implement

Built to the manifest, no deviations:
- **tabs.rs (pure):** `Workspace::cycle_project(forward) -> bool` (after `switch_project`; reuses
  `next_index`/`prev_index` over `projects.len()` + `switch_project`; returns whether focus moved) +
  the 3 tests (`cycle_project_forward_wraps` / `_backward_wraps` / `_single_noop`).
- **keymap.rs (pure):** +2 bindings (⌘⇧]→next-workspace, ⌘⇧[→prev-workspace) after prev-tab; the
  roster guard `all_chords_lists_every_binding` bumped 36→38 (29 vec-literal + 9 switch-tab);
  +`workspace_chords_bound` test.
- **app.rs (shim):** +2 dispatch arms after `prev-tab` — `cycle_project(fwd)` then, IF it moved,
  `sync_active_project()` + `persist_grid()` (a workspace switch changes the active project, unlike a
  tab switch; mirrors the rail-click sync).

`cargo fmt` clean; `cargo check --all-targets -p marley` clean; **316 tests pass** (+4). Ready for
Inspect. **Phase 3 status: Implement PASS.**

## Phase 3.5 — Inspect

1 critic (general-purpose) over the diff + self-review (small, additive, mirrors the shipped next-tab
pattern → 1 critic is proportionate).

**Critic — SHIP-clean.** No CRIT/HIGH/MED. It RAN the mutation + tests concretely:
- `cycle_project` correctness CLEAN — no off-by-one/wrap bug; the 3 tests cover advance+wrap (both
  dirs) + single-noop (both dirs).
- **Mutation MSI 100** — `cargo mutants -f tabs.rs -F cycle_project` → **3 caught / 0 missed** (the
  →true/→false/`==`→`!=` set, all killed).
- Dispatch CLEAN — `sync_active_project()` is the exact fn `jump_to_pane` uses on a project change
  (rebuilds Files/⌘P/git/titlebar); a tab switch correctly doesn't call it. No `cx.notify()` needed
  (the caller notifies at app.rs:3858, mirroring next-tab). All methods in scope + `&mut self`.
- keymap CLEAN — ⌘⇧]/⌘⇧[ free (tab chords are shift=false; no cmd+shift bracket exists);
  `chords_unique` + the 38-roster guard + `workspace_chords_bound` all pass.
- **REQ-004 CLEAN** — the diff is purely additive; NONE of the never-empties guards
  (tabs.rs:152/157/299, app.rs:2227/2236/2778/2809) is touched. Always-focused invariant preserved.

**2 LOW/INFO, neither a code fix:** (1) `let _ = switch_project(target)` is safe — `target` is always
in-range under the `active < len` + never-empties invariants, so the swallow + the "always Ok" comment
are correct (a purely-defensive note about an unreachable broken-invariant case). (2) my predicted
"forward swap-branch" mutant doesn't exist — corrected the notes' mutant line above (the #232 lesson).

**Fixes applied:** only the notes' mutant-prediction correction (a doc-accuracy fix, no code change).
Lenses: correctness, mutation/coverage, dispatch/state, keymap-integrity, REQ-004-invariant, clean-room
(no third-party code). **Phase 3.5 status: Inspect PASS.**

## Phase 4 — Validate

**Tests:** `cargo nextest run -p marley` → **316 passed, 2 skipped**. The design's plan rows green:
T1 `cycle_project_forward_wraps`, T2 `cycle_project_backward_wraps`, T3 `cycle_project_single_noop`
(tabs.rs); T4 `workspace_chords_bound` + the updated `all_chords_lists_every_binding`(38) +
`chords_unique_detects_dups` (keymap.rs). T6 (REQ-004) — grep confirmed no guard edit + the guard tests
stay green.

**Driven capture (REQ-002) — env-considerately DEFERRED** (chad live + screen-sharing, same as
#228-232). Mechanism-verified: the dispatch is byte-structurally the next-tab pattern + calls
`sync_active_project()` (the critic confirmed it's the exact fn `jump_to_pane` uses on a project change
→ Files/⌘P/git/titlebar rebuild). VIBE-CHECK offered when his screen's free: ⌘⇧] / ⌘⇧[ cycles the
focused workspace + the sidebar/Files/git should track it. gate-15 visual/AX is headless → green.

**Gate:** `git add -A` (no forbidden paths) → `scripts/gates.sh --diff` → **GATE GREEN [diff] — 15/15**,
incl **gate:4 coverage ≥100%** + **gate:5 mutation MSI ≥100%** (cycle_project 3/3 killed; keymap
additions covered; app.rs dispatch is shim, cov-excluded). Receipt written for /commit.

**No pre-existing failures in scope.** **Phase 4 status: Validate PASS.**
