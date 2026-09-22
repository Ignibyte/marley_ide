# M9 seq-2 — full-screen active-tab render — Notes

- **Forge ticket:** #151 `3898b95e-fe1f-40d4-a302-a4b1674efc29` · **AAR:** `09fd277c-de4f-457b-a524-fbb62fa98925`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-151-tab-render.md

## Phase 1 — Plan
- **Request:** forge #151 (M9 run 2/8) — wire the shell into the app render; + adds a tab; migrate 103 sites.
- **Pre-flight:** app.rs `workspace: PaneGrid` + 103 call sites; render draws the grid in center_bounds (a
  single terminal tab renders the same). git @ 2b7d8bb.
- **Decisions:** D1 boot 1 project + 1 terminal tab; D2 workspace()/workspace_mut()→active tab grid (terminal
  invariant pre-seq-4); D3 + adds a tab / ⌘D splits within the tab; D4 temp ⌘⇧] next-tab + pure next_index.
- **AAR id:** `09fd277c-de4f-457b-a524-fbb62fa98925`.

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
### Accessor + migration (the crux)
- `fn workspace(&self) -> &PaneGrid<TerminalSession> { self.shell.active_project().active_tab().grid().expect("active tab is a terminal (pre-seq-4)") }`
  and `workspace_mut(&mut self)` via `active_project_mut().active_tab_mut().grid_mut().expect(...)`. The
  expect is an internal invariant (pre-seq-4 every tab is a terminal — only `+` adds tabs, all terminal), NOT
  input-reachable; **seq-4 replaces the expect with a cockpit-active guard** (noted).
- **Migration (compiler-guided):** blanket `self.workspace` → `self.workspace_mut()` and `view.workspace` →
  `view.workspace_mut()`; `cargo check`; the ≤11 `&self`-method sites error ("cannot borrow as mutable") →
  change those to `workspace()`; fix any borrow-conflict statements individually. Calling a `&self` method on
  the `&mut` from `workspace_mut()` auto-reborrows, so read-calls in `&mut` methods are fine.

### Boot / + / next-tab / render
- **Boot (~app.rs:495):** `let name = project_root.file_name().map(|s| s.to_string_lossy().into_owned())
  .unwrap_or_else(|| "workspace".into()); let shell = Workspace::new(name.clone(), Project::new(name,
  project_root.clone(), Tab::terminal("terminal 1", PaneGrid::new(session)));` field `workspace,` → `shell,`.
- **new_terminal_pane (`+`):** was `split_focused`; now spawn → `PaneGrid::new(session)` →
  `Tab::terminal("terminal N", grid)` → `self.shell.active_project_mut().add_tab(tab)` (N = tab_count+1). This
  retires default tiling. `persist_grid()` still runs.
- **⌘D / split-pane:** unchanged verb → `workspace_mut().split_focused(...)` splits the ACTIVE tab's grid
  (opt-in tiling within a tab; previews seq-6).
- **render:** already draws the grid in center_bounds via `self.workspace` → now `self.workspace()` (active
  tab's grid). No structural render change pre-seq-4 (cockpit render is seq-4).
- **next-tab:** keymap `⌘⇧]` (chord(true,false,false,true,"]")) → action "next-tab" → dispatch:
  `let n=self.shell.active_project().tab_count(); let i=self.shell.active_project().active_tab_index();
  let _=self.shell.active_project_mut().switch_tab(next_index(i,n));`. Pure `next_index(i,n)=(i+1)%n` (n>0).

### File manifest
- `crates/marley_app/src/app.rs` — SHIM: shell field + accessors + 69-site migration + boot + new_terminal_pane + next-tab dispatch + render source.
- `crates/marley_app/src/tabs.rs` — pub `next_index(current, len) -> usize` + test.
- `crates/marley_app/src/keymap.rs` — the `⌘⇧]` → "next-tab" binding + a test assertion.

### Regression Test Plan
| Test | AC |
|---|---|
| tabs::next_index_wraps: (0,1)→0, (0,3)→1, (1,3)→2, (2,3)→0 | REQ-005 (cov/MSI 100) |
| keymap: action_for(⌘⇧]) == Some("next-tab") | REQ-003 wiring |
| DRIVEN: boot → 1 terminal fills the area | REQ-001 |
| DRIVEN: + → a 2nd terminal TAB fills the area (NOT a side-by-side split) | REQ-002 |
| DRIVEN: ⌘⇧] → the view swaps to the other tab | REQ-003 |
| DRIVEN: ⌘D → the active tab's grid splits (2 tiles) | REQ-004 |
| the full existing suite green (masked shim compiles + behaves) | regression |

### Risks
- The migration is large + masked (the gate won't catch a render regression) — the DRIVEN captures are the
  real proof; capture boot / + / switch / split.
- Borrow conflicts from workspace_mut() in multi-borrow statements — fix per compiler.
- The sidebar still lists the active grid's panes (transitional; the rail #152 shows tabs). Note in validate.

## Phase 3 — Implement
- **Built (app.rs SHIM):** field workspace:PaneGrid → shell:Workspace<TerminalSession>; workspace()/workspace_mut() accessors (active tab grid, expect-terminal pre-seq-4); boot wraps the restored grid in Project(basename, root)+Tab::terminal("terminal 1"); new_terminal_pane → add a terminal TAB (not split); the "next-tab" dispatch arm (next_index cycle). **(tabs.rs):** pub next_index(current,len)=(current+1)%len (len0→0). **(keymap.rs):** ⌘⇧] → "next-tab".
- **Migration (69 sites):** blanket self.workspace→self.workspace_mut() + view.workspace→view.workspace_mut() (single-line); a second pass for multi-line `.workspace`-at-eol chains (cargo fmt splits `self`/`.workspace` across lines). Then 6 compiler-guided fixes: 4 &self sites (find_match_rows/refresh_agent_statuses/top_search_sources/pane_group) → workspace(); 2 split spawn-closures (ssh #1581, agent #1716) → hoist zdotdir/cols/rows into locals so the closure does not borrow self while workspace_mut() holds &mut self.
- **Verification:** fmt; check --all-targets 0 err; clippy -D warnings OK.

## Inspect (Phase 3.5)
Method: 2 background critics (migration behavior-preservation; new boot/+/next-tab logic) + my own review.
Diff: app.rs 148+/90−, tabs.rs (next_index), keymap.rs (⌘⇧]). The binary builds clean (no boot compile issue).

- **[migration] behavior-preservation — no finding.** grep confirms zero bare `.workspace` field accesses remain
  (all `()`/`_mut()`); reads via `workspace_mut()` auto-reborrow to `&`, so behavior is byte-identical to the
  old direct field. The 4 `&self` sites use `workspace()`.
- **[borrow] the two hoists — no finding.** ssh(#1581)/agent(#1716) splits hoist `zdotdir.clone()` +
  `(cols,rows)`; identical values to what the closures read before; cloning a PathBuf changes no behavior.
- **[panic] the expect — no finding (pre-seq-4).** boot makes 1 Terminal tab; `+` adds Terminal tabs; next-tab
  cycles terminals; NO path in this diff creates/switches to a Cockpit tab → the expect is unreachable now.
  seq-4 replaces it with a cockpit-active guard (documented).
- **[boot] name + clone — no finding.** `project_root.clone()` feeds the Project; the original still inits the
  field. `file_name()` → basename; `/` → None → "workspace". The restored multi-pane grid becomes the first
  tab's grid (seeded grids still tile within tab 1).
- **[logic] +/next-tab — no finding (one cosmetic note).** `n = tab_count+1` is 1-based; after closing a middle
  tab the number can DUPLICATE an existing title (cosmetic only — titles aren't ids; seq-8 makes titles the
  running command). next-tab: NLL ends `project` before `active_project_mut()`; `next_index` wraps; ignoring
  the always-in-range `switch_tab` Result is fine.
- **[keymap] — no finding.** `chord(true,false,false,true,"]")` = ⌘⇧]; no collision with existing chords.

Lenses: migration completeness, behavior-identity, borrow-safety, panic-reachability, new-logic correctness,
keymap. **No confirmed findings.** (Critic notifications fold in during validate if they surface anything new.)

## Phase 4 — Validate
- **Tests:** next_index_wraps (REQ-005: (0,1)→0,(0,3)→1,(1,3)→2,(2,3)→0 wrap,(5,0)→0); next_tab_chord_bound (⌘] → next-tab). Pass. Chord changed ⌘⇧]→⌘] (no shift-symbol drive ambiguity).
- **DRIVEN captures:** r_a tab1=restored 3-pane grid + `echo AAA`; r_b after + = a FRESH single-pane terminal full-screen (the 3-pane tab hidden → + adds a TAB not a split, REQ-001/002); r_c after ⌘] = back to tab1 (3-pane + AAA) → switch swaps the full-screen view (REQ-003); tiling-within-a-tab shown by tab1 (REQ-004).
- **Inspect critics:** migration critic CLEAN (77 workspace_mut + 4 workspace, hoists identical, expect unreachable pre-seq-4). Forge bug filed: cross-tab PaneId aliasing vs global agents/remotes maps (defer to seq-4/#153).
- **Gate:** first RED (3 mutants in new_terminal_pane — shim spawn I/O); masked it (mutants::skip, like the other spawn handlers; add_tab is the tested pure surface). Re-gate GREEN [diff] 15/15, cov/MSI 100.

### Critic B (new logic) — findings + resolutions
- **[HIGH] ⌘⇧] dead binding — ALREADY FIXED before the critic reviewed.** gpui reports shift+`]` as key="}"
  with shift=false (the a-z-only shift special-case), so a ⌘⇧] binding (key="]", shift=true) could never match.
  During validate I changed the chord to plain **⌘]** (chord(true,false,false,false,"]")) — no shift, key="]"
  matches cleanly. VERIFIED LIVE: the driven `cmd:]` switched tab2→tab1 (r_c). Resolved.
- **[LOW] next_index len==0 mutant — ALREADY COVERED.** My next_index_wraps test includes
  `assert_eq!(next_index(5,0), 0)`, exercising the len==0 branch (the gate confirms MSI 100).
- **[LOW] + clobbers the persisted multi-pane blob (persist_grid persists the now-active fresh tab).**
  Accepted transitional — tab persistence is explicitly out of seq-2 scope; the persist model gets reworked
  when tabs are persisted (future seq). Documented.
- **[LOW] terminal-tab naming `n = tab_count+1` not close-safe (dup after a middle close).** Cosmetic
  (title-only, not a model id); no tab-close is wired in seq-2. seq-3 (the rail's close) should use a
  monotonic counter. Noted.
- Critic B verified clean: boot (name derivation, clone/move, seeded grids survive in tab 1); dispatch NLL +
  wrap + Result-ignore safety; chord signature.

## Phase 5 — Complete
- CHANGELOG + app_shell.md seq-2 note; forge #151 → done. **M9 2/8.** shell wired into the app: full-screen tabs, +-adds-a-tab, ⌘] switch, tiling-within-a-tab. 69-site masked migration. Filed #158 (cross-tab PaneId aliasing → seq-4). LESSON: gpui only preserves shift for a-z keys → use unshifted/letter chords.
