# Workspace-close LSP host leak — Notes

- **Forge ticket:** #359 `d7981725-3b80-437c-8dae-0d2183d4bbf5`
- **AAR:** `cea6a711-e834-41b4-a5ae-8a707f0ec365`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-359-workspace-close-lsp-host-leak.md
- **Pipeline spec:** 359-workspace-close-lsp-host-leak.spec.md
- **pipeline_id:** 3428bb0b-7864-4193-9682-89bcb807d6f6 · on `a32fa7b`

## Phase 1 — Plan

**Request:** closing a workspace never drops its LSP host → a rust-analyzer (1–4 GB RSS) leaks for the process
lifetime. `close_project_at` cleans every other per-project map but not `lsp_hosts`. #321 follow-up (state-derived
creation widened the blast radius). A real bug with a clear, small fix.

**Classification / tier:** work pipeline, one shippable slice (a guarded `lsp_hosts.remove` in `close_project_at` +
its headless proof).

**Forge recall (§18.3):** no bulletins. `knowledge-context` (Plan) surfaced 13 nodes (3 ADRs, 3 distilled lessons,
5 prevention rules, 2 failures — LSP lifecycle + close-cleanup + the mock-clock pump). Logged to the AAR.

### ★ Recon (Explore agent, verified on `a32fa7b`) — the 3 design points settled

1. **ROOT-KEY IDENTITY — EXACT MATCH (D1).** The insert key is `active_root = view.shell.active_project().root.clone()`
   (app.rs:1412) → the RAW `Project.root: PathBuf` field (tabs.rs:168), passed verbatim to the `.insert(root.to_path_buf(), …)`
   at app.rs:4550. NO `.canonicalize()`/normalization touches the key (only the open-doc PATH is resolved via
   `resolve_under_root`, a containment test, never the key). `.root` is set once in `Project::new` and never
   reassigned. `LspHost::new` canonicalizes internally but the map key stays raw. ⟹ `removed.root` (the closed
   Project's raw `.root`) is byte-identical to the insert key → `lsp_hosts.remove(&removed.root)` hits it. (~40 host
   lookups across app.rs all key off `active_project().root.clone()`, none canonicalize — corroboration.)

2. **★ TWO PROJECTS, ONE ROOT — REACHABLE (D2 = LAST-ONE-OUT, not the naive one-liner).** `projects: Vec<Project<S>>`
   with no uniqueness; `Workspace::add_project` (tabs.rs:352) pushes unconditionally; `open_project_path` (app.rs:6210)
   builds a fresh `Project::new` and calls `add_project` with NO same-root check / no focus-existing (every entry
   point — the folder picker :6250, new-empty :6235, the launcher/recents row :8218 — funnels here). So picking the
   same directory twice yields TWO live `Project`s sharing one `LspHost`. An unconditional remove on the first close
   would tear down a server the surviving same-root project uses (it re-spawns on a later pump tick → a visible
   rust-analyzer restart + lost didOpen state). ⟹ remove ONLY when no surviving project shares the root:
   `if !self.shell.projects().iter().any(|p| p.root == removed.root) { self.lsp_hosts.remove(&removed.root); }`
   (`close_project` already removed it from the Vec; `Workspace::projects() -> &[Project<S>]` at tabs.rs:412), placed
   BEFORE the `std::thread::spawn(move || drop(removed))` at app.rs:6400 (so `removed.root` is read before the move).

3. **TEST LANE — needs `tick_pump` (mock clock; the #349 lesson).** Hosts are created on the PUMP (#321
   state-derived), and `tick_pump` (headless_drive.rs:76 = `advance_clock(40ms)` + `run_until_parked`) is REQUIRED —
   its own doc warns a `run_until_parked`-only drive leaves negative assertions VACUOUSLY passing. The model is
   `restored_editor_tab_spawns_lsp_headless` (headless_drive.rs:8460): `write_cargo_fixture` (a `Cargo.toml` — the
   `root.join("Cargo.toml").is_file()` gate at app.rs:4538 — + a `.rs`) + `seed_stub_language_server` (rust → a
   nonexistent binary → the host is CREATED but no real process spawns, hermetic) + seed + boot + `tick_pump` →
   assert `lsp_host_exists_for_test(root)`. A host is created only for the ACTIVE project's root when it has an open
   `.rs` under it — so the 2-host test must make EACH project active + open its `.rs` + `tick_pump`. A 2-project
   single-shell seed does NOT exist yet — build it by extending `seed_one_project`'s blob (a 2nd `{root}\t0\tT=t`
   line) or the codec `projects: vec![…]`.

**Decisions recorded:** D1-REMOVE-KEY (`removed.root`, exact), D2-LAST-ONE-OUT (2-projects-1-root reachable → guard),
D3-DROP-UNTOUCHED, D4-PROOF-VIA-MAP + tick_pump. See the spec.

**The load-bearing risk (for inspect):** the last-one-out guard — an unconditional remove would be a NEW bug (drop a
shared host). The guard + the same-root test (REQ-LAST-ONE-OUT) is the proof it's correct.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**Approach.** The leak fix is a guarded `lsp_hosts.remove` in `close_project_at`, with the last-one-out PREDICATE
extracted to a `Workspace` method so its logic is cov/MSI 100 (app.rs is coverage-excluded; tabs.rs is NOT). §14
holds (no panic — the predicate is total); §20 = N/A (Marley's own workspace/LSP lifecycle; `LspHost::drop`'s
shutdown handshake is the LSP spec, already implemented + untouched).

**★ D5 — the predicate home (a recon-driven refinement of D2).** tabs.rs is coverage-INCLUDED (the gate:4
`--ignore-filename-regex` excludes only app.rs / lsp_host.rs / etc., NOT tabs.rs) and is full of cov'd `Workspace`
methods. So the last-one-out predicate lives there as `Workspace::has_project_with_root(&self, root: &Path) ->
bool { self.projects.iter().any(|p| p.root == root) }` — cov/MSI 100 via a unit (kills the `any`/`==` mutants
directly), and the call site reads naturally: `if !self.shell.has_project_with_root(&removed.root) {
self.lsp_hosts.remove(&removed.root); }`.

**★ `Workspace::close_project` does NOT guard the last project** (tabs.rs:360 — only an `idx >= len` range check,
then `remove`). The never-empties invariant is enforced elsewhere (the rail only shows × when applicable /
`close_project_at`'s callers), NOT in the tabs-level close. So T2 (the same-root guard) needs NO 3rd project:
close ONE of two same-root projects → the surviving sibling keeps the shared host — that IS the last-one-out
proof. (The "drop when it IS the last of a root" arm is covered by T1's distinct-roots close + the predicate unit.)

**The pieces (exact):**
1. **`Workspace::has_project_with_root` (tabs.rs, pub, cov/MSI 100 — D5):** `self.projects.iter().any(|p| p.root ==
   root)`. + a `#[cfg(test)]` unit next to the other Workspace tests.
2. **`close_project_at` guard (app.rs:6381, mutants::skip):** inside the `if let Ok(removed) = shell.close_project(idx)`,
   AFTER the existing map cleanup and BEFORE `std::thread::spawn(move || drop(removed))`: `if
   !self.shell.has_project_with_root(&removed.root) { self.lsp_hosts.remove(&removed.root); }`. `removed.root` is
   borrowed (shared) by both `has_project_with_root` and `remove`, released before `drop(removed)` moves it;
   `self.shell` (shared) then `self.lsp_hosts` (mut) are different fields, sequential — compiles without a clone
   (confirm at implement). The removed `LspHost` value drops in place → `Drop` fires shutdown+exit+reap. The
   `Project` has NO `LspHost` (it lives in `lsp_hosts`), so `drop(removed)` doesn't double-drop it.
3. **`close_project_at_for_test` hook (app.rs, `#[cfg(test)] pub(crate)`):** `close_project_at` is a private `fn`
   (rail-click only); a headless test in the sibling module can't call it → add `pub(crate) fn
   close_project_at_for_test(&mut self, idx: usize) { self.close_project_at(idx) }` (mirrors the other `_for_test`
   hooks).
4. **`seed_two_projects` helper (headless_drive.rs):** the blob route — `format!("0\n{a}\t0\tT=t\n{b}\t0\tT=t")`
   (active idx 0; two project lines, terminal tabs). Same-root variant passes the same root twice. (Confirmed
   `seed_one_project` is `"0\n{root}\t0\tT=t"`; a 2nd line = a 2nd project.)

**File manifest:**
| File | Change |
|---|---|
| `crates/marley_app/src/tabs.rs` | ADD `Workspace::has_project_with_root(&self, root: &Path) -> bool` + a `#[cfg(test)]` unit. |
| `crates/marley_app/src/app.rs` | the guarded `lsp_hosts.remove` in `close_project_at` + a `#[cfg(test)]` `close_project_at_for_test` hook. |
| `crates/marley_app/src/headless_drive.rs` | `seed_two_projects` helper + T1 (distinct-roots drop+survive) + T2 (same-root guard). |

**Regression Test Plan:**
| # | Test | Kind / home | Proves |
|---|------|-------------|--------|
| T1 | `close_project_drops_its_lsp_host_headless` | headless, app | REQ-CLOSE-DROPS-HOST + REQ-OTHER-SURVIVES — 2 projects at DISTINCT roots (a, b); make each active + `open_file_in_viewer` its `.rs` + `tick_pump` → both hosts exist; `close_project_at_for_test(idx_a)` → `!lsp_host_exists_for_test(a)` AND `lsp_host_exists_for_test(b)` |
| T2 | `close_one_of_two_same_root_projects_keeps_the_shared_host_headless` | headless, app | REQ-LAST-ONE-OUT — 2 projects at the SAME root r; one host (shared); `close_project_at_for_test(0)` → `lsp_host_exists_for_test(r)` STILL true (the surviving sibling keeps it — the guard prevents the shared-host kill) |
| T3 | `has_project_with_root_*` | pure unit, tabs.rs (cov/MSI 100) | the predicate — `[a,b]` has `a`/`b` (true), lacks `c` (false); empty has nothing (false) — kills the `any`/`==` mutants |

The `close_project_at` arm is `mutants::skip`/cov-excluded → T1/T2 (headless drives) carry the WIRING; the pure
`has_project_with_root` is cov/MSI 100 in tabs.rs via T3. ★ Host creation is on the PUMP → the drives MUST
`tick_pump` (mock clock — the #349 lesson; `run_until_parked` alone leaves negatives VACUOUS), with
`write_cargo_fixture` (the `Cargo.toml` gate) + `seed_stub_language_server` (hermetic — no real process) per root.
NO live synthetic drive (headless = cargo test, safe).

**Risks / decisions:** (a) ★ the last-one-out guard is the load-bearing correctness (unconditional remove = a NEW
bug killing a shared host; T2 + T3 prove the guard). (b) borrow ordering (shared `removed.root` before the move;
different `self` fields — no clone needed, confirm at implement). (c) the 2-host test sequence (activate +
open_file_in_viewer + tick per project; `shell.switch_project(idx)` switches active). (d) the tick_pump
vacuous-pass trap. (e) Drop untouched — no double-drop (host not in Project). §14; §20 = N/A.

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest, 3 files. `cargo fmt --all`; `cargo check -p marley --all-targets` CLEAN; `cargo clippy
-p marley --all-targets -D warnings` rc 0 (only the pre-existing `block v0.1.6` warning). Diff = exactly tabs.rs +
app.rs + headless_drive.rs.

- **crates/marley_app/src/tabs.rs** — ADDED `Workspace::has_project_with_root(&self, root: &Path) -> bool {
  self.projects.iter().any(|p| p.root.as_path() == root) }` (after `projects()`, doc'd) + a `#[cfg(test)]` unit
  `has_project_with_root_matches_any_open_root` (a Workspace with `/a` + `/b` → true/true, `/c` → false, + a
  same-root `/a`×2 → still true). Used `.as_path()` on the `PathBuf` field for an unambiguous `&Path == &Path`
  compare.
- **crates/marley_app/src/app.rs** — ADDED the last-one-out guard in `close_project_at` (after the
  `agents`/`remotes`/`last_agent` cleanup, before `std::thread::spawn(move || drop(removed))`): `if
  !self.shell.has_project_with_root(&removed.root) { self.lsp_hosts.remove(&removed.root); }`. The borrows compiled
  without a clone (shared `removed.root` for both calls; `self.shell` shared then `self.lsp_hosts` mut — different
  fields, sequential). ADDED the `#[cfg(test)] pub(crate) fn close_project_at_for_test(&mut self, idx)` hook next to
  `lsp_host_exists_for_test`.
- **crates/marley_app/src/headless_drive.rs** — ADDED `seed_two_projects(dir, root_a, root_b)` (the
  `"0\n{a}\t0\tT=t\n{b}\t0\tT=t"` blob, same-root allowed) + two `#[gpui::test]` drives:
  `close_project_drops_its_lsp_host_headless` (T1 — distinct roots: activate + `open_file_in_viewer` + `tick_pump`
  each → both hosts; `close_project_at_for_test(0)` → `!exists(root_a)` AND `exists(root_b)`) and
  `close_one_of_two_same_root_projects_keeps_the_shared_host_headless` (T2 — same root, one shared host; close one
  → `exists(root)` still true). Modeled on `gesture_open_spawns_on_next_tick_headless`; `tick_pump` (mock clock).

**Deviations from design:** none. The predicate uses `.as_path()` (unambiguous `&Path` compare) rather than a raw
`PathBuf == &Path`. The borrow needed no `.clone()`. The 2-host test uses `shell.switch_project(1)` to make each
project active before opening its `.rs` (a host is created only for the active project's root). ⚠️ Phase 4 RUNS the
drives + confirms the exact host-creation sequence (the pump-tick timing is the #349 mock-clock lesson — if a host
doesn't appear on the first tick, Phase 4 adjusts the tick count; the map assertions are the AC).

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

**2 independent critics** (general-purpose), parallel — scaled to the cross-cutting change (a guard + a pure
predicate + 2 headless drives). Both verified the tree untouched. **Verdict: the fix is CORRECT and robust** — both
load-bearing concerns (last-one-out correctness, does-the-remove-hit) are CLEAN, with a strong underlying reason.

- **Critic 1 — last-one-out + key + Drop/borrow + predicate:** CLEAN on all (a)–(d). ★ the decisive property: the
  map key equality and the guard equality are the SAME relation (`Path` Hash/Eq) on the SAME raw `.root` field, so
  a guard "miss" is never a wrong-drop (Path-unequal roots have SEPARATE hosts) and a real shared host (Path-equal
  keys) is always caught — even the trailing-slash case is consistent. `close_project` removes BEFORE returning
  (survivors-only). `removed.root` is Path-equal to the insert key → remove HITS (leak actually plugged). Project
  has no LspHost field (no double-drop). tabs.rs is cov-included → the predicate unit is the cov/MSI-100 surface
  and kills the viable set.
- **Critic 2 — tests non-vacuous + 2-host sequence + seed + provenance:** CLEAN. Both drives are REAL gates (T1's
  `a0 && b0` pre-close + T2's exists-before-close — the post-close assertions are meaningful only because these
  fire); host_b IS created (switch_project sets active=1 → the pump keys root_b; open_file_in_viewer targets the
  active project; file_b under root_b passes the wants_host gate); `tick_pump` used throughout (no bare
  `run_until_parked` — the vacuous-pass trap avoided); the seed blob is a valid 2-project extension; hermetic (stub
  → resolve_binary None → no real process).

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| F1 | LOW | **The `LspHost::drop` doc (lsp_host.rs:727-732) is now STALE/FALSE** — it claims "reached on app quit ONLY", "`lsp_hosts` is insert-only", and "#321 filed the teardown as a follow-up". After #359 every clause is false, in the exact file the next LSP-lifecycle maintainer opens (the #337-F5 "a doc must not claim what the code doesn't keep" class). | **FIXED at source.** Rewrote it: Drop is now reached on app quit AND workspace close (via `close_project_at`, last-one-out); the pre-#359 leak + #321's widening noted as history; the SIGKILL orphan edge + the last-editor-tab-close deferral kept. lsp_host.rs is cov-excluded + doc-only → no cov/MSI impact. |
| F2 | LOW/info | T1's host_b creation depends on the fixtures using ABSOLUTE paths (`switch_project` alone leaves `project_root` at root_a; `resolve_under_root` ignores the base for an absolute doc path). Fail-safe (a relative fixture → `b0=false` trips the `a0 && b0` gate, never a silent pass). | **FIXED — documented.** Added a comment in T1 stating the absolute-path invariant + its fail-safe nature (the critic's recommendation — record the load-bearing invariant). |
| — | (info) | Host `Drop` runs synchronously on the UI thread (Project drops off-thread). Accepted — `shutdown()` queues + `ServerHandle::drop` SIGKILL+reap is near-instant. | **ACCEPTED — no action** (intentional asymmetry; near-instant). |

**Result: 0 CRITICAL/HIGH/MEDIUM. The fix is CORRECT (both critics, with the map-key==guard-key proof). 2 LOW fixed
(the stale Drop doc — a real doc-accuracy catch in the LSP file; the absolute-path invariant comment).** cargo check
clean after the fixes. Lenses: last-one-out, the key, Drop/borrow, the predicate cov/MSI, test non-vacuity, the
2-host sequence, the seed, provenance.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests RUN (written in Phase 3; Phase 4 confirms + gates):**

| # | Test | Result |
|---|------|--------|
| T1 | `close_project_drops_its_lsp_host_headless` | **PASS** — ★ the non-vacuous `a0 && b0` gate PASSED (both hosts created by the switch+open+tick sequence — the 2-host setup worked first try), then close A → `!lsp_host_exists_for_test(root_a)` AND `lsp_host_exists_for_test(root_b)`. REQ-CLOSE-DROPS-HOST + REQ-OTHER-SURVIVES proven. |
| T2 | `close_one_of_two_same_root_projects_keeps_the_shared_host_headless` | **PASS** — shared host asserted before close, then close one → `lsp_host_exists_for_test(r)` STILL true. REQ-LAST-ONE-OUT proven (the guard prevents the shared-host kill). |
| T3 | `has_project_with_root_matches_any_open_root` | **PASS** — the pure predicate unit (cov/MSI 100 in tabs.rs). |

**Runs (`CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley -E 'test(close_project_drops_its_lsp_host) + test(close_one_of_two_same_root) +
  test(has_project_with_root)'` → **3 passed, 0 failed** — first run, no drive-sequence iteration needed.
- `cargo nextest run -p marley` → **732 passed, 2 skipped** (T1/T2/T3 + no regression; was 729, +3 — the existing
  #321 LSP drives stay green).

**NO LIVE SYNTHETIC DRIVE — stated.** chad may be at the machine; the headless drives are `cargo nextest` through
the REAL `close_project_at` arm (the actual leak-fix path), so they ARE the exact proof (the map-entry drop ⟹ the
`LspHost` was dropped ⟹ `Drop` fired — a real-process death assertion would need the #320 fake-server lane, out of
scope). A live drive is off-limits AND unnecessary.

**FULL `--diff` gate → `GATE GREEN [diff]` — 15/15 on the first run.** Receipt
`ca113aa780187a4d356e37e5c5f8e4452fa5b64c`. gate:4 coverage 100% (`has_project_with_root` tested; app.rs /
lsp_host.rs excluded), gate:5 MSI 100 (the predicate's viable mutants killed by T3; the app.rs guard is
`mutants::skip`; lsp_host.rs is doc-only), gate:14 docs PASS (the `LspHost::drop` doc rewrite is accurate; plain
backticks). No pre-existing failures; the `search_open…` flake did not recur.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Complete (Phase 5)

**Docs (§21).**
- **CHANGELOG.md** — a `### Fixed` entry: closing a workspace now shuts down its rust-analyzer instead of
  leaking it for the process lifetime (the insert-only map; #321's blast-radius widening; the last-one-out fix
  gated on the pure `Workspace::has_project_with_root`; the map-key == guard-key structural guarantee; the
  last-editor-tab-close drop is a separate #321 deferral).
- **docs/marley_architecture/app_shell.md** — a new `#359` bullet after the `#162` project-close entry:
  `close_project_at` now drops the closed project's LSP host last-one-out so `LspHost::drop` reaps rust-analyzer;
  keyed by the raw project root (= the insert key); two projects can share a root and thus one host.
- `LspHost::drop`'s doc in lsp_host.rs was corrected at INSPECT (F1) — no restore needed.

**Capture (forge wired).**
- `aar-submit` cea6a711, outcome completed, effectiveness 4 (1 novel finding, 13 verdicts). Headline lessons:
  (a) ★ the fix was a GUARD, not a one-liner — the ticket sketched `lsp_hosts.remove(&removed.root)`, but the
  recon found two open projects CAN share a root (no dedup in `add_project`/`open_project_path`), sharing one
  host, so an unconditional remove would be a NEW bug (a killed shared host) → last-one-out. A "trivial" bug fix
  can hide a reachability question. (b) ★ map-key == guard-key: a stale/wrong drop is structurally impossible
  because the HashMap key equality and the guard equality are the SAME `Path` relation on the SAME raw field.
  (c) ★ the STALE-DOC catch (Critic 1, F1): the fix falsified `LspHost::drop`'s doc ("app-quit ONLY" + "#321
  filed teardown as a follow-up"), in the exact file the next maintainer opens — a fix must re-check the docs it
  invalidates. (d) cov/MSI move: extract the last-one-out predicate to a cov-INCLUDED module
  (`Workspace::has_project_with_root` in tabs.rs) so its logic is unit-tested (cov/MSI 100), not left inline in
  the cov-excluded app.rs. (e) the #349 mock-clock lesson reused (hosts are pump-created → `tick_pump`, not
  `run_until_parked` — the tests passed first try because the sequence honored it).
- `prevention-rule-record` **PR-claude-guard-last-one-out-shareable-resource-001** (e331b13c): before tearing
  down a resource on one owner's close, check whether it can be shared and guard last-one-out if so; derive the
  teardown guard's key and the container's key from the same field + relation so the agreement is structural.
- `failure-record`: none SHIPPED — the leak was a PRE-EXISTING bug this ticket fixed; the 2 inspect findings
  were LOWs (a stale doc + a test comment), caught + fixed in-phase.
- No follow-up ticket: the last-editor-tab-close host drop is already the tracked #321 deferral.

**Close + archive.** forge #359 → done; TICKET-359 → closed/ (`status: closed`); the spec/notes pair archived
active/ → completed/; spec `status: Phase 5 — Complete PASS`.

**Status: Phase 5 — Complete PASS.** Run `/commit` to deliver (LOCAL — push un-OK'd).
