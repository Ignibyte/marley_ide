# persist-rail-collapse — pipeline notes (forge #245, M14 sprint #27)

Pipeline: 0cc8aa0d-168d-490f-a912-8493da669802 · AAR: b1f18e31-d587-4589-a563-8655830b5a45
Ticket: forge#245 (07560d41-0da6-4b6f-8af2-187f27c4fa37). 6th of M14 (#238 config, #239/#240/#241/#243 done).

## Phase 1 — Plan (discovery inline)

**#236 follow-on:** the rail collapse-state is in-memory only → resets on restart.

**Discovery (this session):**
- `RootView.collapsed_projects: HashSet<usize>` (app.rs:149) — collapsed project INDICES; inits empty at boot
  (app.rs:1056); `toggle_project_collapse(p)` (app.rs:2869, insert-or-remove); remapped on close via
  `remap_indices_after_remove` (app.rs:2902); read by `rail_rows(&shell, &collapsed_projects)` (app.rs:4264).
- Stable key: `Project.root: PathBuf` (tabs.rs:146); `Workspace::projects()` (tabs.rs:373) → ordered projects.
- Settings pattern (MIRROR): `Recents: Vec<String>` (settings.rs:54) → `AppliedSettings.recents` (130) →
  `applied_defaults` (192) / `applied_from` (224) → `persist_recents(manager, &[String])` (257,
  `manager.set::<Recents>(...)`). Test fixtures set `recents: Vec::new()` (381/421/460).

**Design (proposed, D1-D4):** a `CollapsedProjects: Vec<String>` setting keyed by root (mirror Recents) + pure
`collapsed_roots` (save) / `collapsed_indices` (restore) in tabs.rs (beside `remap_indices_after_remove`) + shim
wiring (toggle + close persist; boot restore). Persist by ROOT (stable), not index (#236 lesson).

**#243 LESSON applied:** VERIFY the persist is TRIGGERED — grep that toggle_project_collapse actually calls the
persist, + a driven quit→relaunch (a correct codec whose save is never called = silently broken).

**Driven plan (control):** collapse a project (chevron ▸) → quit → relaunch → still collapsed (children hidden).

**ENV:** control granted. Autonomous auto-approved (M14) → run through commit; do not stop; do not push. #242
HELD for chad. **AWAIT the inspect critic before Inspect-PASS** (properly invoke /pipeline:inspect — the #241/#243
lesson).

## Phase 2 — Design

**Approach.** Pure index↔root mapping helpers + the established Recents-style settings round-trip + shim wiring
(persist on the mutation path, restore at boot). No PTY/forge.

**File manifest:**
- **settings.rs:** (1) `define_setting!(CollapsedProjects: Vec<String> = Vec::new(), "rail.collapsed")` (with the
  others). (2) `AppliedSettings` — add `pub collapsed: Vec<String>` (after `recents`). (3) `applied_defaults` —
  `collapsed: CollapsedProjects::default_value()`. (4) `applied_from` — `collapsed: manager.get::<CollapsedProjects>()`.
  (5) EVERY AppliedSettings test fixture (settings.rs ~381/421/460 + any others — compiler will list them) — add
  `collapsed: Vec::new()`. (6) `pub fn persist_collapsed(manager: &mut SettingsManager, roots: &[String]) ->
  Result<(), SettingsError> { manager.set::<CollapsedProjects>(roots.to_vec()) }` (mirror `persist_recents`).
- **tabs.rs (pure, beside `remap_indices_after_remove`):**
  - `pub fn collapsed_roots(collapsed: &HashSet<usize>, project_roots: &[String]) -> Vec<String>` — **D5: iterate
    `project_roots` by index, include those whose index ∈ `collapsed`** (stable insertion order + one-pass
    out-of-range skip): `project_roots.iter().enumerate().filter(|(i,_)| collapsed.contains(i)).map(|(_,r)| r.clone()).collect()`.
  - `pub fn collapsed_indices(saved_roots: &[String], project_roots: &[String]) -> HashSet<usize>` —
    `project_roots.iter().enumerate().filter(|(_,r)| saved_roots.contains(r)).map(|(i,_)| i).collect()`.
- **app.rs:** (a) import `CollapsedProjects`? no — just `persist_collapsed` (settings) + `collapsed_roots`/
  `collapsed_indices` (tabs). (b) shim helper `fn persist_collapsed_state(&mut self)`: build `project_roots =
  self.shell.projects().iter().map(|p| p.root.display().to_string()).collect::<Vec<_>>()`, `roots =
  collapsed_roots(&self.collapsed_projects, &project_roots)`, then `if let Some(mgr) = self.settings.as_mut() {
  let _ = persist_collapsed(mgr, &roots); }` (mirror app.rs:2752). (c) CALL `self.persist_collapsed_state()` at
  the END of `toggle_project_collapse` (~2872) AND after the close-remap (~2902, after
  `remap_indices_after_remove`). (d) BOOT: at the RootView construction (app.rs:1056), init
  `collapsed_projects: collapsed_indices(&applied.collapsed, &<project roots of the just-built shell>)` instead
  of `HashSet::new()`.

**Regression Test Plan:**
| AC | Test (tabs.rs) | Proves |
|----|----------------|--------|
| REQ-001 | `collapsed_roots_cases`: `{0,2}`+`[a,b,c]`→`[a,c]`; `{0,5}`+`[a,b]`→`[a]` (5 skipped); `{}`→`[]` | indices→roots, stable order, out-of-range skip |
| REQ-001 | `collapsed_indices_cases`: `[a,c]`+`[a,b,c]`→`{0,2}`; `[x]`+`[a,b]`→`{}` (unknown skipped); round-trip via collapsed_roots | roots→indices, unknown skip, round-trip |
| REQ-001 | settings round-trip (settings.rs): set `CollapsedProjects`=["/a"] via a tempdir manager → `applied_from(...).collapsed == ["/a"]` (mirror the Recents test) | the setting persists/loads |
| REQ-002/003 | DRIVEN (control) | collapse a project → quit → relaunch → still collapsed (children hidden, chevron ▸) |

**Mutation/coverage:** `collapsed_roots`/`collapsed_indices` — the `contains` filter (both), the enumerate map;
RUN `cargo mutants --list -f tabs.rs` for the real set; cov/MSI 100. `persist_collapsed` mirrors `persist_recents`
(a `manager.set` — method call, likely no viable mutant beyond the body; covered by the round-trip). The app.rs
shim (persist_collapsed_state + the boot restore) is coverage-excluded.

**Risks:** (1) **the persist must be TRIGGERED** — `toggle_project_collapse` + the close path MUST call
`persist_collapsed_state` (grep + driven quit→relaunch; the #243 lesson). (2) boot-restore runs AFTER the shell/
projects exist (the RootView-construction insertion has the roots in scope). (3) ALL AppliedSettings fixtures
updated (compiler-enforced by the new field). (4) `p.root.display().to_string()` matches the format the
launcher/recents already persist (consistent root strings) — the roots round-trip as the same string.

**Phase 2 status: Design PASS — the setting + pure helpers + shim wiring; manifest + test matrix set.**

## Phase 3 — Implement

Applied the manifest (3 files):
- **settings.rs:** `CollapsedProjects: Vec<String>` define_setting (`rail.collapsed`); `AppliedSettings.collapsed`
  field; wired into `applied_defaults` + `applied_from`; all 3 AppliedSettings test fixtures got `collapsed:
  Vec::new()` (replace_all); `persist_collapsed` fn (mirror persist_recents).
- **tabs.rs:** pure `collapsed_roots` (iterate project_roots by index, keep those collapsed → stable order +
  out-of-range skip) + `collapsed_indices` (roots→indices, unknown-root skip), beside `remap_indices_after_remove`.
- **app.rs:** imported persist_collapsed + collapsed_indices/collapsed_roots; masked
  `persist_collapsed_state(&mut self)` shim (roots via collapsed_roots → persist_collapsed); CALLED it at the end
  of `toggle_project_collapse` (app.rs:2884) AND after the close-remap (app.rs:2932) — **the persist IS TRIGGERED
  on the mutation path** (grep-confirmed, the #243 lesson); BOOT restore = a `collapsed_projects` local computed
  before the `Self {}` literal (shell borrowed before it's moved) via `collapsed_indices(&applied.collapsed, &
  <project roots>)`, then the field is the shorthand.
- **Deviations:** the boot restore is a local-before-the-literal (not an in-literal expr) because `shell` moves
  into the struct at the `shell,` field — the local borrows it first. No functional change.
- **Build:** `cargo fmt` clean; `cargo check --all-targets -p marley` clean (the new field compiler-forced all
  fixtures — none missed); `clippy -D warnings` clean. The pure-fn tests + settings round-trip land at Phase 4.

**Phase 3 status: Implement PASS — compiles + clippy clean; the persist is triggered on toggle + close.**

## Phase 3.5 — Inspect (IN PROGRESS — critic spawned; spec HELD at Implement-PASS; properly in the inspect phase
so any fix stays in-code — the #241/#243 lesson)

Self-review so far:
- **[CLEAN, self] round-trip + edges** — `collapsed_roots({0,2},[a,b,c])`=`[a,c]` → `collapsed_indices([a,c],
  [a,b,c])`=`{0,2}` (round-trips). Out-of-range: `collapsed_roots({0,5},[a,b])`=`[a]` (iterates roots, so 5 never
  appears — structurally can't emit an OOR). Unknown root: `collapsed_indices([x],[a,b])`=`{}`. `collapsed_indices`
  enumerates `project_roots` so it can NEVER produce an index ≥ len (no OOR into collapsed_projects). ✓
- **[CLEAN, self] stable-key** — persist stores ROOTS (stable), restore maps roots→CURRENT indices, so a
  reorder/close between sessions restores the right projects (no raw-index aliasing — the #236/#245 point). ✓
- **[CLEAN, self] persist TRIGGERED (#243 lesson)** — `persist_collapsed_state()` CALLED at app.rs:2884
  (toggle) + 2932 (close-remap). Boot restore: the `collapsed_projects` local at app.rs:1045 borrows `shell`
  BEFORE it's moved into the struct at 1055; the field is the shorthand at 1067. Correct order. ✓
- **[CLEAN, self] settings wiring** — grep: EXACTLY 5 `AppliedSettings {` (applied_defaults:193, applied_from:226,
  fixtures 392/433/473) — ALL got `collapsed:` (cargo check green). ✓
- **[CLEAN, self] mutation** — 6 body-return mutants (collapsed_roots ×3, collapsed_indices ×3; the `.contains`
  filter is a method call → not mutated), all killable by the Phase-4 exact-value matrix. ✓
- **[EDGE, pending critic] duplicate roots** — if two projects shared the SAME root string, `collapsed_indices`
  would mark BOTH collapsed on restore (both match the saved root). Project roots are distinct repos in practice;
  cosmetic if not (an extra project collapsed). Assessing severity w/ the critic.

**The critic RETURNED (during Phase-4 test-writing) and CONCURS — its ONLY finding is the [LOW] duplicate-roots
edge I'd flagged (two projects sharing a root → both restore collapsed; cosmetic, inherent to root-keying) →
DOCUMENTED with an in-code caveat on `collapsed_indices` (its recommendation: document, don't block). It
independently verified: EVERY write to `collapsed_projects` (app.rs:1045/2881-2882/2931) is followed by
persistence or is the restore itself (no un-persisted mutation); the close ordering (remove-before-return) is
sound; the boot borrow-before-move is correct; ALL 5 AppliedSettings constructions got `collapsed:` (grep, no
other crate/cfg); the setting mirrors Recents byte-for-byte; and the `{0,2}` assertion is the required
`collapsed_indices→HashSet::new()` mutant-killer (my test has it). All other lenses CLEAN.

**Phase 3.5 status: Inspect PASS — critic CONCURRED; 1 LOW (dup-roots) documented; the persist-TRIGGER + boot-
order (the #243/#241 failure modes) both verified clean; all other lenses clean.**

## Phase 4 — Validate

**Tests:** `tabs::collapsed_roots_indices_cases` (indices→roots stable + out-of-range skip; roots→indices +
unknown skip; round-trip) + `settings::collapsed_setting_round_trips` (persist_collapsed → reload identity;
absent → empty). `cargo nextest run` → **2/2 PASS**; the 48/48 settings+tabs regression held.

**Driven capture (REQ-002/003, control) — the #243 lesson honoured (verify the persist is TRIGGERED + a driven
quit→relaunch):** collapsed the "Marley · main" project via its ▸/▾ chevron (a ~14px edge target — took 2 click
tries) → **settings.toml gained `[rail] collapsed = ["/Users/.../Marley"]`** (the persist IS triggered, keyed by
root) → quit → relaunch → **`245-restored-collapsed.png`: the rail shows ONLY "Marley · main" with a ▸ chevron,
all its tab rows HIDDEN — restored COLLAPSED**, dark. End-to-end confirmed.

**Gate:** `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov 100 + MSI 100 on
tabs.rs (collapsed_roots/collapsed_indices — 6 body mutants killed) + settings.rs (the CollapsedProjects
round-trip; persist_collapsed's Ok(()) mutant killed). The app.rs shim (persist_collapsed_state + boot restore)
is coverage-excluded.

**Phase 4 status: Validate PASS — 2 tests, driven confirms collapse persists + restores collapsed, gate green.**

## Complete (Phase 5)

- **Docs (§21):** CHANGELOG.md — `### Changed` entry above #243 ("Collapsed rail projects stay collapsed across
  a restart"). app_shell.md — an M14 #245 bullet (the `CollapsedProjects`/`rail.collapsed` setting, the pure
  `collapsed_roots`/`collapsed_indices` pair keyed by the STABLE root, the persist TRIGGERED on toggle + close,
  the boot restore, the dup-root LOW).
- **Knowledge:** `aar-submit b1f18e31` outcome=completed, effectiveness 5 (clean; the critic CONCURRED with a
  self-review that had already pre-checked the #243 persist-trigger + the #241 boot-order failure modes — only a
  documented LOW). `failure-record BF-claude-root-keyed-state-aliases-duplicate-roots` (validation/low) — the
  dup-root aliasing tradeoff of root-keying. Materialized `PR-claude-persist-verify-trigger-not-just-codec-001`
  (the #243 rule) — APPLIED proactively: grep-verified persist_collapsed_state on both mutation paths, then
  driven-verified `[rail] collapsed` landed in settings.toml before the relaunch.
- **Close:** forge #245 (`07560d41`) → done; TICKET-245 open→closed.
- **Archive:** spec status `Phase 5 — Complete PASS`; the spec+notes → `pipeline/completed/`.

**Phase 5 status: Complete PASS. Run `/commit` to deliver.**
