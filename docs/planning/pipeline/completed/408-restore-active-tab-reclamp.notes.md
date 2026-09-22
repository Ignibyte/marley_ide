# Shell-codec index hygiene on drop — Notes

- **Forge ticket:** #408 767ae35d-8c5e-4608-896b-241a6f99996e
- **AAR:** 4c2f4f92-88bc-4d9d-8558-47c424b43585
- **Local ticket doc:** docs/planning/tickets/open/TICKET-408-restore-active-tab-reclamp.md
- **Pipeline spec:** 408-restore-active-tab-reclamp.spec.md

## Phase 1 — Plan

- **Request:** `/work` on the next ticket → forge #408 (the M29 #403 inspect critic's
  deferred filing): shell-codec restore drops tabs without reclamping `active_tab`; the
  `serialize_shell` blob is the one writer input with no framing guard.
- **Classification / tier:** single work-pipeline slice, chore, milestone M29 wake.
  Two crates' worth of surface in ONE crate (`marley_app`): the codec module
  (`grid_layout.rs`) + the restore consumer (`app.rs`). No UI delta (React-first N/A).
- **How this became "the next ticket":** open-ticket queue + `pipeline/queued/` were
  empty; both pre-authored shelves (IDE-MVP, integrity-five) fully shipped; M29's
  5-ticket train (#402–#406) complete. Forge held the two fresh follow-ups; #407
  (FULL-audit debt) mandates a complete whole-workspace mutation run first — the
  overnight-audit shape (the 2b9eab9 DIFF-first decision), wrong for an interactive
  slice — so #408, the newest deliberately-filed, unblocked, well-scoped chore, is
  next. `ticket-next` was probed and is FIFO (returned the Windows-blocked #6); not
  treated as a priority signal.
- **Forge recall (§18.3):** forge was DOWN at session start (postgres stale
  `postmaster.pid` after the Aug 7 reboot + the LaunchAgents' TCC failure on
  /Volumes/Offload + `start-all.sh` building the tool-less OSS binary — restored via
  the installed `~/.local/bin` rlm pair; #405/#406 statuses synced done while there).
  Bulletins: none. Top recall hits:
  - **Prevention rule (codec layers):** the SHELL codec reserves ONLY `\t \n \r \x1f`;
    the grid-leaf layer reserves different bytes — identify WHICH layer serializes an
    artifact before choosing its encoding. Directly governs the blob-guard hazard set
    (REQ-005: `breaks_framing` + `\x1f`, NOT the grid alphabet).
  - **AD (#394 lineage):** registry residents' lifecycle (pinned vs dropped) dictates
    the resolve-door contract — the restore loop's Cockpit/Browser arms acquire
    differently from Terminal/Code; the reclamp must not disturb the acquire pairing
    (the app.rs:2294 LOAD-BEARING comment).
- **Discovery (verified against live code this session — the ticket's :2229/:2294
  anchors were stale):**
  - `breaks_framing` grid_layout.rs:261 (`\t \n \r`); `reclamp_active` :269
    (unit-tested :1139; shared at FILE level by `serialize_code_paths` :291 and the
    V= restore arm app.rs:2334).
  - `serialize_shell` :341 — root guarded :344; V= paths guarded via
    `serialize_code_paths` :284 (framing + `\x1f`); **blob pushed verbatim :387 — the
    asymmetry**; `active_project` written verbatim :342 while :344-346 drops projects
    → **serialize-side project drift** (NEW find at plan, added to scope).
  - Restore consumer app.rs:2220-2379 — vanished root `continue` :2222; `T=` push
    conditional on spawn success :2259; `V=` push conditional on any-readable :2332;
    Cockpit :2281 / Browser :2302 push unconditionally; `active_tab.min(last)` :2360;
    `active_project.min(last)` :2379 → **restore-side tab AND project drift**.
  - `restore_shell` doc :419 pins "clamping is the caller's job" → D3.
- **Prior-art sweep:** in-repo `reclamp_active` owns the seam (adoption, not
  invention); Zed behavior map 07 (SerializableItem registry) is the §20 reference;
  no permissive dep owns persistence-index reclamping (gpui/ropey/regex/
  alacritty_terminal/tree-sitter checked). Recorded in the spec.
- **Decisions:** D1 (one primitive), D2 (writer guards), D3 (caller-side restore fix),
  D4 (pure-seam testability; app.rs killable) — locked in the spec. OPEN for Phase 2:
  D-BLOB-GUARD (drop-tab `continue` vs `debug_assert!` vs both) and the drive-test
  strategy for the spawn-fail `T=` arm.

## Phase 2 — Design

### Approach — one primitive, four sites, one guard
All four drift sites route through the existing `reclamp_active` (D1). The serialize
side becomes a **pre-scan writer**: compute the surviving original indices FIRST (projects:
`!breaks_framing(root)`; tabs: Terminal → blob hazard-free (NEW guard), Cockpit/Browser →
always, Code → `serialize_code_paths(..).is_some()`), write the RECLAMPED active index,
then emit exactly the survivors. The restore side collects surviving original indices as
it pushes (tab loop: 4 arms; project loop: at the `restored` push) and replaces both
`.min(last)` bounds with `reclamp_active`. Empty-survivor edge: write/derive `0` (the
restored side never calls reclamp on empty by construction — the `Some(first_tab)` /
`Some(w)` arms guarantee non-empty).

**D-BLOB-GUARD (locked):** write-time DROP of the Terminal tab (`continue`, joining the
V=-none-survived arm) with the hazard set `\t \n \r` (`breaks_framing`) **plus `\x1f`**
(the in-entry rider separator — a `\x1f`-bearing blob would inflate the #399 part-count
disambiguation). NO `debug_assert!`: it would panic the REQ-005 test in the debug
profile — the drop path must be EXERCISABLE (§7 demands the test run it; gate:5 must
kill guard mutants), and uniform D2 (root drops, paths drop, hazard-blob drops) needs no
second mechanism. A hazard blob is bug-only territory today (the grid alphabet + 
`breaks_grid_framing` on embedded cwds exclude every hazard byte) — the guard is the
robustness backstop the ticket asked for, not a reachable-in-production branch.

**Design discovery (spec amended, REQ-007):** the serialize-side TAB drift already
exists TODAY via the V= arm — `serialize_shell` writes `project.active_tab` verbatim
(:350) then `continue`s on a no-representable-path Code tab (:400). The blob guard would
have added a second instance. Both are covered by the same pre-scan reclamp.

**§20 confirmed:** the plan's Zed reference stands (behavior-level focus stability under
degraded restore; the behavior map is research). The mechanism is Marley's own #243
recorded semantics ("first survivor at-or-after, else last") — no source read, no wall
contact. §14: all new logic is pure, total on hostile input, no unwrap/expect on input
paths, no new IO, single crate.

### File manifest
1. `crates/marley_app/src/grid_layout.rs` — `serialize_shell` becomes the pre-scan
   writer (surviving projects/tabs → reclamped actives → survivor emission); the blob
   hazard predicate (named, doc'd — `breaks_framing(blob) || blob.contains('\u{1f}')`);
   unit tests for REQ-004/005/006/007 + empty-survivor edges in the existing
   `#[cfg(test)]` mod.
2. `crates/marley_app/src/app.rs` — the restore loop: `surviving_tabs: Vec<usize>`
   pushed in all four arms + `reclamp_active` replacing `.min(last)` at the
   `switch_tab` site (:2360); `surviving_projects: Vec<usize>` + `reclamp_active`
   replacing `.min(last)` at the `switch_project` site (:2379); one
   `debug_assert_eq!(surviving_tabs.len(), tabs.len())` tripwire (kills push-deletion
   mutants on EVERY restoring drive, not just the crafted one).
3. `crates/marley_app/src/headless_drive.rs` — a `seed_shell_layout(dir, &ShellLayout)`
   helper (generalizes the #321 helper — always through `serialize_shell`, never a hand
   blob) + three `#[gpui::test]` drives (T1/T2/T3 below). NOTE (inspect flag): the
   `seed_project_with_editor_tab` doc comment (:68-74) still describes the pre-#395
   force-seed guard — stale; correct it if the helper is touched, else leave for a docs
   pass.

### Regression test plan
Discriminator rule (novel, recorded for the AAR): place the expected survivor strictly
BEFORE the last survivor — whenever the expected target IS the last one, `min(last)`
coincides with `reclamp_active` and the test proves nothing.

| T | REQ | Kind | Lane | Shape |
|---|-----|------|------|-------|
| T1 | REQ-001 | drive | headless_drive | Seed via `serialize_shell`: tabs `[T, V(missing file), C, B]`, `active_tab=2` (the Cockpit, orig 2). V drops on restore → survivors `[0,2,3]` → active must be the COCKPIT (position 1). The old `min(2,3)=2` picks B. One scenario exercises all four arms' survivor pushes: deleting any push shifts count AND active kind. Assert `tab_count()==3` + active-tab kind probes. |
| T2 | REQ-002 | drive | headless_drive | Same seed with the V file READABLE → nothing drops → `active_tab==2` exactly, `tab_count()==4` (identity; kills always-0/always-last reclamp mutants + REQ-006's restore half). |
| T3 | REQ-003 | drive | headless_drive | Seed 4 projects `[A ok, B vanished-root, C ok, D ok]`, `active_project=2` (C). Survivors `[0,2,3]` → active project must be C (position 1); old `min(2,2)=2` picks D. Assert active project root == C. |
| T4 | REQ-004 | unit | grid_layout tests | ShellLayout 4 projects, root[1] carries `\t`, `active_project=2` → `restore_shell(serialize_shell(..))` → 3 projects, `active_project==1` (C). Old writer emitted 2 → D. |
| T5 | REQ-005 | unit | grid_layout tests | Terminal blobs `"t\tX"` and `"t\u{1f}X"` seeded among honest siblings → serialize → round-trip: the hazard tab is ABSENT, no forged entry (the `\t` case would mint a phantom `X` entry), sibling entries intact, actives reclamped. Guard-removal mutants die here. |
| T6 | REQ-006 | unit | grid_layout tests | A hazard-free two-project layout (every tab kind present, titles + pane names) serializes to the EXACT pre-change golden string (inline literal captured from today's writer) and round-trips. Pins byte-identity when nothing drops. |
| T7 | REQ-007 | unit | grid_layout tests | Tabs `[T, V(all paths hazardous), T, T]`, `active_tab=2` → serialize writes active `1` → round-trip `active_tab==1` (2nd T). Old writer wrote 2 → 3rd T. |
| T8 | edges | unit | grid_layout tests | ALL projects hazardous → output `"0"`; all tabs of a project dropped → `root\t0` line restoring an empty project; actives beyond every survivor → last survivor (the `unwrap_or` arm). |

Coverage posture: `grid_layout.rs` new code at cov/MSI 100 (no exclusions). `app.rs` is
coverage-excluded (standing gate posture) but mutated: its new lines are killed by
T1–T3 (+ the tripwire amplifier). No ACCEPTED-UNTESTABLE additions. The spawn-fail
`T=` arm gets NO dedicated drive: `cwd_or_root` validates candidates (a bogus cwd falls
back to root — unforceable headlessly without breaking the boot PTY `.expect`), and the
arm contains NO new branch — its only new line is the survivor push, killed by T1's
kind/count asserts through the SHARED reclamp seam.

### Risks
- **Behavior change is the fix itself**: a degraded restore now focuses the mapped
  survivor; the no-drop path is pinned byte-identical + index-identical (T2/T6).
- Serialize bytes change ONLY for already-degraded layouts (hazard root / all-hazard
  V= / hazard blob) — states whose old bytes encoded a WRONG active. Recorded, accepted.
- `reclamp_active`'s non-empty precondition: honored by construction at every new call
  site (pre-scan writers guard with the empty→0 arm; restore arms sit inside non-empty
  matches). T8 pins the writer edge.

## Phase 3 — Implement

**React-first: N/A** (per the spec section — no UI delta; recorded, not skipped).
Recall: done at plan this session (knowledge-search/explain + the codec prevention
rule); the seams were read live in Phase 1/2 — no fresh code-find needed.

Built exactly to the manifest, no deviations:
- `grid_layout.rs` — `blob_breaks_entry_framing` (framing + `\x1f`, doc'd as the
  write-time backstop); `tab_survives_shell_entry` (the pre-scan predicate, doc-bound
  to MUST-agree with the emission arms); `serialize_shell` pre-scans surviving
  projects/tabs, writes both actives through `reclamp_active` (empty → 0), emission
  loop unchanged except the Terminal arm's hazard `continue`.
- `app.rs` — `surviving_projects`/`surviving_tabs` collected at the exact push sites
  (4 tab arms + the project push); both `.min(last)` bounds replaced with
  `crate::grid_layout::reclamp_active` (the :2334 fully-qualified idiom); the
  `debug_assert_eq!` tripwire ties survivor count to tab count (kills push-deletion
  mutants on every restoring drive).
- `cargo fmt` + `cargo check -p marley` green (the `block v0.1.6` future-incompat
  note is pre-existing upstream, untouched).
- Tests intentionally NOT written here (Phase 4 owns T1–T8); `headless_drive.rs`
  untouched this phase.

## Phase 3.5 — Inspect

Three parallel critics (correctness · data/state integrity · simplification+provenance)
over the 2-file diff; lead-reviewed skeptically. Ledger:

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | low | Writer's survivor model missed the READER's unconditional empty-root drop (`restore_shell` skips `root:""`): a `root:""` project counted as a survivor, so the reclamped `active_project` indexed a line the parse removes — the #408 drift class one level removed. Found independently by BOTH correctness and state critics; unreachable from the live writer (roots come from opened `is_dir` directories) but violates the diff's own "survivors = what reaches the RESTORED state" claim. | **REAL — fixed** | Pre-scan filter gains `&& !p.root.is_empty()` (one clause; emission iterates the survivor list, so writer+reclamp agree by construction). Comment cites the reader-agreement rule. REQ-004 extended. |
| F2 | low/info | REQ-006's "byte-identical" is violated for OUT-OF-RANGE saved actives (old wrote verbatim `7`, new clamps to last survivor) — unreachable from the live writer (`switch_*`/`adjust_active` keep indices in-range; verified by the critic against tabs.rs), and post-restore behavior is identical. | **REAL as spec-wording bug — spec fixed, no code change** | REQ-006 re-worded: in-range actives byte-identical; degenerate actives deliberately clamped (recorded hygiene change). Phase 4 goldens pin in-range actives. |
| F3 | minor | `blob_breaks_entry_framing`'s body duplicated the pre-existing #243 inline filter in `serialize_code_paths` (`breaks_framing ‖ contains \x1f`) — the hazard-byte set stated in two places. | **REAL — fixed** | Renamed payload-generic `breaks_entry_framing(s)`, doc covers both payload kinds (V= path, T= blob); `serialize_code_paths` now calls it. ONE place owns the set. |
| F4 | nit | Retained doc sentence still enumerated the pre-#205 blob alphabet (`t/f/c/g H/V : ,`, missing `=` + cwd bytes). | **REAL — fixed** | Enumeration corrected in place. |
| F5 | note | The `is_empty → 0` guards are behaviorally redundant (`reclamp_active(&[], n)` is already total, returns 0) — could widen the fn's contract instead. | **REJECTED — keep** | The explicit arm states the "restore ignores the index" decision at the site and honors the fn's documented non-empty contract; no mutation-surface cost (no operator targets the guard). |
| F6 | minor | Dual drop-decision (pre-scan predicate + emission arms) with `serialize_code_paths` computed twice per Code tab; a single-pass scratch-String shape exists. | **REJECTED — keep as shipped** | The critic's own analysis: persist is event-driven (not per-frame) and existing per-persist allocations dwarf the double call; the refactor would restructure ~10 push sites and risks dead arms (MSI hazard). Drift risk bounded by exhaustive matches + shared predicates. Phase 4 adds the agreement-killing tests (hazard-before-active for each kind). |

Cross-cutting intel for Phase 4 (from the critics, verified):
- **`RootView::new_in` is `#[cfg_attr(test, mutants::skip)]` (app.rs:1545) — the entire
  restore loop is EXCLUDED from mutation.** The MSI burden for #408 lands 100% on
  grid_layout.rs unit tests; the T1–T3 drives are BEHAVIORAL proof of the restore REQs,
  not mutant-killers. The existing serialize-drop test only drops index 0 (where old and
  new writers agree) — the new tests MUST place drops before the active index.
- Double-reclamp composition PROVEN exact by the state critic (serialize reclamp ∘
  restore reclamp = joint reclamp, incl. past-all fallback); all survivor vectors are
  ascending by construction.
- Bonus behavior the guard buys (state critic): a hand-mangled multi-`\x1f` `T=` entry
  parsed leniently used to be RE-EMITTED as a forged frame on the next save — the guard
  now drops it with a correct reclamp (the laundering hole closes). T5 covers this path.
- Provenance: PASS — every new shape has a named in-repo ancestor (#243 kept/orig
  pattern, #163 D2 stance, the app.rs readable/orig idiom); nothing structurally foreign.

Post-fix `cargo fmt` + `cargo check -p marley`: green.

## Phase 4 — Validate

**Tests added (the Phase 2 plan, T-numbers preserved):**
- T1 `restore_reclamps_active_tab_over_a_dropped_editor_tab` (headless_drive) — the
  four-kind fixture `[T, V(missing file), C, B]`, active=2: restore drops the V, focus
  lands on the Cockpit at position 1 (the old `.min(last)` kept 2 = the Browser). All
  four arms contribute survivors in one scenario.
- T2 `restore_keeps_active_tab_when_nothing_drops` (headless_drive) — identity: file
  readable → 4 tabs, active exactly 2.
- T3 `restore_reclamps_active_project_over_a_vanished_root` (headless_drive) — 4
  projects, root[1] vanished, active=2 → focus follows /c to position 1 (old bound → /d).
- T4 `serialize_reclamps_active_project_over_dropped_roots` (grid_layout) — REQ-004,
  parameterized over BOTH drop causes: framing root AND the inspect-F1 empty root.
- T5 `serialize_drops_hazard_blob_tabs` (grid_layout) — REQ-005, three hazard bytes
  (`\t`, `\x1f`, `\n`); asserts no truncated-terminal/forged-title shapes and the
  reclamped active; Cockpit+Browser sit before the active for the predicate-flip kills.
- T6 = the PRE-EXISTING `shell_codec_round_trips` exact-bytes golden (in-range actives,
  all four kinds) — REQ-006's byte-identity proof, unchanged and still green.
- T7 `serialize_reclamps_active_tab_over_dropped_code_tab` (grid_layout) — REQ-007.
- T8 `serialize_reclamp_edge_cases` (grid_layout) — empty-survivor → `"0"` at both
  levels (exact wire asserted), past-all actives → last survivor at both levels.
- Fixture: `seed_shell_layout` + `reclamp_fixture` helpers in headless_drive (always
  through `serialize_shell`, the #321 no-hand-blob rule). Every discriminating case
  places the expected survivor strictly BEFORE the last (the P2 rule).

**Runs (transcript is truth, §15):**
- `cargo nextest run --workspace`: **2151 run, 2151 passed, 5 skipped** (the 5 are the
  standing headed-lane ignores). `cargo test --workspace --doc`: ok. Targeted codec
  set first: 18/18.
- `scripts/gates.sh --diff`: **GATE GREEN [diff] — 15 passed, 0 failed** (summary in
  transcript). gate:4 coverage ≥100% lines PASS; gate:5 mutation: **15 mutants on the
  diff, 15 caught, MSI 100** (mutants.out/outcomes.json: CaughtMutant 15 + the
  baseline Success); gate:6 miri PASS; gate:15 visual/AX PASS (no UI surface touched;
  harness suite 158/158). Receipt written: `.git/ignibyte-gate-receipt` =
  `1a62ebe6764d319d38640696f1713c6394a89e60`.
- Live-app drive: N/A per the spec's React-first section — no drawn pixel changes; the
  restore behavior is proven by the T1–T3 headless drives (the app's own RootView
  booted on seeded config), not deferred to manual verification.

**Pre-existing failures: none.** Nothing skipped, no baseline, no suppression, no floor
touched. The `block v0.1.6` future-incompat warning is pre-existing upstream (gpui dep
chain), untouched.

## Phase 5 — Complete

- **CHANGELOG.md** — a `### Fixed` entry (first section under Unreleased): the four-site
  reclamp, the blob guard + the laundering hole it closes, the byte-identity pin, and —
  because it is the batch's transferable insight — the discriminator rule spelled out.
- **Architecture docs** — `app_shell.md`'s #163 entry: the "dropped tab shifts the saved
  active index" KNOWN LIMIT is retired via an *(Coda — M29 #408)* recording the as-built
  mechanism (pre-scan writer, survivor mirror of the reader's drops, the shared
  `breaks_entry_framing`, drive-proven restore wiring). Parity sync: N/A (non-UI).
- **Forge capture (§18.3/§19)** — AAR 4c2f4f92 submitted (outcome completed, score 5/5;
  surfaced-used: the codec-layers prevention rule + the #243 primitive; materialized:
  F-claude-writer-survivor-model-missed-reader-drop-001,
  PR-claude-survivor-model-must-mirror-reader-drops-001). A second rule recorded from
  the test-design lesson: PR-claude-place-drop-discriminators-before-the-last-survivor-001.
- **Ticket closed** — local doc → `tickets/closed/` (status closed); forge #408
  `ticket-close` final_status=done.
- **Session ops note (outside the pipeline, recorded for the record):** forge was DOWN
  at session start — postgres had a stale `postmaster.pid` from the Aug 7 reboot (PID
  reused by a FaceTime service; lockfile removed after triple verification), the forge
  LaunchAgents fail on launchd-TCC ("Operation not permitted" on /Volumes/Offload —
  needs a GUI grant or a local-disk script home), and `start-all.sh` builds the
  tool-less OSS binary (`.env` pins the real `~/.local/bin` rlm pair, which
  `run-mcp.sh`/`run-sched.sh` honor — those were used). #405/#406 forge statuses were
  synced done (shipped while forge was down); M29's five train tickets all read done —
  sprint closure left to Chad.
- Archived to `docs/planning/pipeline/completed/`.
