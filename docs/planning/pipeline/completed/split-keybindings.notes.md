# Split keybindings — Notes

- **Forge ticket:** #197 (59f70ac0-a0bc-438f-8a84-9a25344e1090)
- **AAR:** da6cfe79-3922-4077-9b77-844dc2c3e128
- **Local ticket doc:** docs/planning/tickets/open/TICKET-197-split-keybindings.md
- **Pipeline spec:** split-keybindings.spec.md

## Phase 1 — Plan
- **Request:** add ⌘-chords + palette entries for Split Right / Split Down (tile-split),
  which are right-click-menu-only today.
- **Classification / tier:** work pipeline, small. Systems: `marley_app::keymap` (pure —
  the chord→action table + a collision guard) + app.rs shim (`dispatch_action` arms +
  palette entries).
- **Forge recall (§19):** relied on the in-repo keymap + prior chord lessons (memory):
  `PR-claude-new-chord-shadowed-by-hardcoded-key-001` (#64 — a new chord can be shadowed
  by a hardcoded `key=="x"` intercept BEFORE keymap dispatch → grep for hardcoded
  intercepts of the chosen keys + self-test-drive the real keystroke); the keymap-collision
  test catches clashes.
- **Discovery (confirmed in code):**
  - `keymap.rs default_bindings()` — a `Vec<(KeyBinding, String)>`; `action_for` finds the
    action. ~30 chords already bound (⌘⇧p/b/r/a/f/e/s/g/o, ⌘p/d/w/b/o/r/t/[/], ⌘k,
    ⌘⌥-arrow focus, ⌘-arrow block-jump, ⌘1-9). New bindings = push 2 tuples.
  - **⌘D → "split-pane" (keymap:78) → `new_terminal_pane()` (dispatch app.rs:2879, #135)**
    — a NEW TERMINAL, NOT a tile-split. Misnomer. The real tile-split is
    `split_focused_pane(axis)` (app.rs:2270), invoked ONLY from the context menu
    (`MenuAction::SplitRight`→Horizontal / `SplitDown`→Vertical, 2297-2298).
  - The context menu already lists "Split Right" / "Split Down" (context_menu.rs #166).
  - The palette command list lives in `palette.rs` (`Command`/`CommandId`) — add 2 entries.
- **Decisions:** D1 dispatch to split_focused_pane(Horizontal/Vertical); D2 ⌘D misnomer
  (optional rename split-pane→new-terminal); D3 chords picked at design (avoid ⌘⇧-arrow —
  the #131 bug pinned `None`); D4 pure keymap collision guard cov/MSI 100; D5 autonomous.

## Phase 2 — Design

**Architecture / approach.** Pure `marley_app::keymap` (the chord→action table + a new
collision guard) + app.rs shim (`dispatch_action` arms → the existing
`split_focused_pane(axis)`; the `cockpit_commands()` palette list). Two new *actions*
(`"split-right"` / `"split-down"`) reached three ways: the keymap chord, the palette
command, and the already-present context menu (#166). No new split machinery.

**Chords chosen (collision-verified):**
- **Split Right = ⌘⇧L** · **Split Down = ⌘⇧J** — the **vim direction** mnemonic (`l`=right,
  `j`=down) for the dev/terminal audience; consistent ⌘⇧ modifier; reliable letter keys.
- Verified FREE: neither is in `default_bindings()`, and grep found **no hardcoded key
  intercept** of `"l"`/`"j"` in app.rs (unlike ⌘⇧D → the #102 git-diff overlay at
  app.rs:3410, and ⌘⇧C → source-control at 3421 — both would have silently shadowed a new
  binding; the `PR-claude-new-chord-shadowed-by-hardcoded-key` lesson paid off).
- No split-left/up (splits always go `After`).

**The `split-pane` misnomer — RENAME (D2/D4: yes).** ⌘D → keymap `"split-pane"` →
`new_terminal_pane()` (app.rs:2879, #135) — a NEW TERMINAL, not a split; and the palette
has CommandId(0) title **"Split Pane"** (same binding/action). Shipping real "Split Right/
Down" next to a fake "Split Pane" is actively confusing, so rename the action
`"split-pane"`→`"new-terminal"` and the palette title "Split Pane"→"New Terminal" (⌘D
behavior UNCHANGED). Blast radius (grep-confirmed, 3 files): keymap.rs (binding L79, doc
L64, test L216), palette.rs (`action_for_command` L129, test L252), app.rs (dispatch L2879,
`cockpit_commands` CommandId(0) title/keywords).

**Collision guard (pure, testable both ways).** Add `pub fn chords_unique(chords: &[KeyBinding]) -> bool`
(HashSet dedup; true iff no chord repeats) + `impl Keymap { pub fn all_chords(&self) -> Vec<KeyBinding> }`.
Requires adding `Hash` to `KeyBinding`'s derive (it's `Debug,Clone,PartialEq,Eq` today;
String is Hash). Testable unique→true / dup→false (kills the `==`/branch mutants) + a real
assertion `chords_unique(default_bindings().all_chords()) == true`.

**File manifest.**
- `crates/marley_app/src/keymap.rs` — add `Hash` to `KeyBinding` derive; 2 new bindings
  (⌘⇧L→`"split-right"`, ⌘⇧J→`"split-down"`); rename `"split-pane"`→`"new-terminal"` (binding
  L79 + doc L64); add `chords_unique()` + `all_chords()`; update tests (renamed action + 2
  new chords + the collision + `all_chords`).
- `crates/marley_app/src/palette.rs` — `action_for_command`: rename CommandId(0)
  `"split-pane"`→`"new-terminal"`; add CommandId(7)→`"split-right"`, CommandId(8)→`"split-down"`;
  update `action_for_command_maps_every_row`.
- `crates/marley_app/src/app.rs` (shims: `dispatch_action`, `cockpit_commands` — both
  `mutants::skip`) — dispatch: rename arm `"split-pane"`→`"new-terminal"`, add
  `"split-right"`→`split_focused_pane(PaneAxis::Horizontal)`, `"split-down"`→`(Vertical)`;
  `cockpit_commands`: rename CommandId(0) title/keywords → "New Terminal"; add CommandId(7)
  "Split Right" (⌘⇧L, keywords split/right/pane) + CommandId(8) "Split Down" (⌘⇧J).

**Regression Test Plan.**
| REQ | test |
|---|---|
| REQ-001 | keymap: `action_for(⌘⇧L) == Some("split-right")`; driven capture (⌘⇧L → focused pane splits right) |
| REQ-002 | keymap: `action_for(⌘⇧J) == Some("split-down")`; driven capture (⌘⇧J → splits down) |
| REQ-003 | `chords_unique(&[a,b])==true`, `chords_unique(&[a,a])==false`, `chords_unique(default.all_chords())==true`; + regression: `action_for(⌘⇧D)==None` (stays the hardcoded diff — didn't accidentally bind it) |
| REQ-004 | palette: `action_for_command(7)==Some("split-right")`, `(8)==Some("split-down")`, `(0)==Some("new-terminal")`; driven (palette → "Split Down" → splits) |
- Uncoverable-by-unit → driven capture: `dispatch_action` + `cockpit_commands` +
  `split_focused_pane` are shims — the live app proves the chord actually splits.

**Risks / decisions.**
- **R1 rename blast radius** — grep-confirmed to 3 files + 2 tests; inspect re-greps for any
  stray `"split-pane"`.
- **R2 shadowing** — ⌘⇧L/⌘⇧J verified free of hardcoded intercepts (they fall through the
  early-return intercept block at app.rs:3362–3423 to keymap dispatch); driven capture is
  the proof they actually reach the split.
- **R3 non-terminal focus** — `split_focused_pane` targets the active tab's terminal grid;
  on a cockpit/code tab it must be a safe no-op (verify it doesn't panic). Driven: chord on
  a code tab does nothing bad.

## Phase 3 — Implement
Built to the manifest across 3 files:
- **keymap.rs** — `Hash` on `KeyBinding`; bindings ⌘⇧L→`"split-right"`, ⌘⇧J→`"split-down"`;
  renamed ⌘D action `"split-pane"`→`"new-terminal"` (+ the doc); added `pub fn chords_unique()`
  + `Keymap::all_chords()`; a `debug_assert!(chords_unique(&keymap.all_chords()))` in
  `default_bindings` self-checks the no-duplicate-chord invariant. Updated the existing ⌘D test
  assert to `"new-terminal"` (kept green).
- **palette.rs** — `action_for_command` CommandId(0)→`"new-terminal"`, +CommandId(7)→`"split-right"`,
  +CommandId(8)→`"split-down"`; updated the existing CommandId(0) assert.
- **app.rs** (shims: `dispatch_action`, `cockpit_commands` — `mutants::skip`) — dispatch: renamed
  arm + `"split-right"`→`split_focused_pane(Horizontal)`, `"split-down"`→`(Vertical)`;
  `cockpit_commands`: CommandId(0) "Split Pane"→"New Terminal", +CommandId(7) "Split Right" (⌘⇧L)
  + CommandId(8) "Split Down" (⌘⇧J).
- **Deviation from design:** added the `debug_assert!` (not in the manifest). Reason: `keymap`
  is a crate-private mod, so a `pub fn chords_unique` used only by tests trips `dead_code` under
  gate:2 `-D warnings`. The debug_assert gives the guard a real PRODUCTION use (a startup
  self-check) — resolves it cleanly, NO `#[allow]` suppression (§0). It's also the right pattern.
- NEW tests (chords_unique both-ways, ⌘⇧L/⌘⇧J resolution, palette 7/8, the ⌘⇧D-stays-None
  regression) = Phase 4.
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing `block v0.1.6` dep warning).

## Phase 3.5 — Inspect
Two general-purpose critics over the diff (keymap/palette/app): (A) correctness/shadowing/
dead-code; (B) mutation/reuse. Both ran commands (`cargo mutants`, `cargo clippy`).

**Verdict: no code defects.** One HIGH test-design gap for Phase 4; two LOW.

| # | Finding | Verdict | Action |
|---|---|---|---|
| 1 | [HIGH-mut, B] the `all_chords -> vec![]` mutant SURVIVES the planned `chords_unique(all_chords())==true` test — `chords_unique(&[])` is vacuously true, so an empty roster passes | **Real (test-design)** | Phase 4 asserts `all_chords()` CONTENT: `assert!(km.all_chords().contains(&⌘⇧L))` + ⌘⇧J (kills vec![] + validates the new chords are bound) |
| 2 | [LOW, A] `docs/specs/SPEC-app-shell.spec.md` R19 still says `cmd-d → "split-pane"` (doc drift; no code dispatches off the spec) | **Real (doc)** | Fix in Phase 5 doc pass (re-sync R19 to `"new-terminal"`) |
| 3 | [LOW/OBS, A] ⌘⇧L on a pure-cockpit active tab splits the FIRST terminal grid (not visible until switched) | **Rejected (scope)** | Pre-existing shared `workspace_mut()` behavior (⌘C/⌘V/⌘W identical), panic-safe, NOT a #197 regression |

**Clean lenses (concrete evidence):**
- **Rename complete** — `grep "split-pane"` finds only comments + a test fixture; every producer
  (keymap, palette) + consumer (dispatch) uses `"new-terminal"`/`"split-right"`/`"split-down"` in lockstep.
- **No shadowing** — traced the app.rs key router (3234–3498): the hardcoded intercepts match
  escape/⌘⇧D/⌘F/⌘⇧C/⌘C/⌘V, NONE match `"l"`/`"j"` → both fall through to `keymap.action_for`
  → `dispatch_action` (same path as the working ⌘⇧P). gpui reports shifted letters lowercase.
- **`split_focused_pane` panic-safe** — the "a project always has ≥1 terminal tab" invariant
  (`close_tab` returns LastTerminal) makes `terminal_grid_index()` always resolve; no panicking unwrap.
- **`chords_unique` correct** — `HashSet::insert` false-on-dup ⟹ `all()` false ⟺ a dup; KeyBinding
  now derives Hash; empty → vacuously true.
- **debug_assert + dead_code — GATE PASSES** (ran it): gate:2 is `clippy --all-targets` (dev) → exit 0;
  even `--release --lib` → exit 0 (the `debug_assert!` call survives cfg-strip in `if false {}`;
  `all_chords` is re-exported pub API). No suppression, no latent release wart.
- **CommandId 7/8** — ids 0,1,2,4,5,6,7,8 match `action_for_command`; the existing
  `every_cockpit_command_resolves_to_a_verb` test iterates + asserts each resolves.
- **REUSE** — `chords_unique`'s `iter().all(insert)` is the idiomatic, mutation-lean form (0 internal
  mutants; the `len()==len()` alternative would add a `==` mutant). Keep it.

**Mutant roster (ran):** `chords_unique→true` (kill: `(a,a)==false`), `chords_unique→false` (kill:
`(a,b)==true`), `all_chords→vec![]` (kill: **content assert** — finding 1), + 2 unviable `Default::default()`
(KeyBinding/Keymap have no `Default`). **Phase-4 lesson to capture:** a uniqueness/predicate assertion
over a COMPUTED collection does NOT kill a `→ empty` mutant (vacuous truth) — also assert the collection's
size/contents.

## Phase 4 — Validate
**Tests added:**
- keymap.rs: `split_chords_bound` (action_for ⌘⇧L→"split-right", ⌘⇧J→"split-down", ⌘⇧D→None regression);
  `chords_unique_detects_dups` ((a,b)→true, (a,a)→false, default.all_chords()→true);
  `all_chords_lists_every_binding` (contains ⌘⇧L + ⌘⇧J + `.len()==36` — **kills the `all_chords→vec![]`
  mutant the inspect HIGH flagged**).
- palette.rs: extended `action_for_command_maps_every_row` with CommandId(7)→"split-right", (8)→"split-down".

**Tests RUN:** `cargo nextest run -p marley` (4 new) → pass; `--workspace` → **774 passed, 5 skipped**.

**Live driven capture (rebuilt with #197):**
- `197-1-splitright.png` — ⌘T (fresh terminal tab) then **⌘⇧L**: the tab splits into **two side-by-side
  panes** (vertical divider, pane 1 | pane 2; focused pane cyan-bordered). REQ-001 ✓.
- `197-2-splitdown.png` — **⌘⇧J**: a pane splits **top/bottom** (horizontal divider; now pane 1/2/3).
  REQ-002 ✓. (Both captures also re-confirm the #194 near-black-terminal / gray-panel theme live.)

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15.** gate:5 mutation MSI 100%
(chords_unique + all_chords mutants killed, incl the `vec![]` one), gate:4 coverage 100%, gate:2 clippy
clean (the `debug_assert!` gives the helpers a live production use — no dead_code, no suppression).

**Pre-existing:** none (only the upstream `block v0.1.6` dep warning).

## Phase 5 — Complete
- …
