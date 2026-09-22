# .mcp.json resolved against the active root — Notes

- **Forge ticket:** #381 81348947-e095-46fb-b060-b90c30549c1b (bug, sprint #36 "M25 — App-Grade QA Hardening")
- **AAR:** c11c5285-b9ce-4f23-b562-217a1355239d
- **Local ticket doc:** docs/planning/tickets/open/TICKET-381-mcp-json-active-root.md
- **Pipeline spec:** 381-mcp-json-active-root.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request (provenance):** found in the 2026-07-21 live QA run (sprint #36 /spec batch): fleet
  question-pick and dispatch-send both surfaced "failed: no forge client (.mcp.json)" on a machine
  whose repo HAS `.mcp.json` — the app had been launched via `open target/Marley.app`, so
  `std::env::current_dir()` was `/` and the boot read missed. Pre-existing since #64 (⌘⇧F dead under
  Finder launches from day one); M24 raised the stakes — every fleet write (#377 answer, #378
  dispatch) and the #376 live-wire bearer source now silently degrade on the most common non-dev
  launch path. Fix direction (ticket): resolve against the restored ACTIVE project's root at the
  #376 post-shell-restore placement; loopback wall unchanged.
- **Classification / tier:** bug, ONE slice, small-medium. One new pure decision fn (candidate
  order) + a masked MOVE of the existing read+build inside the boot shim. Zero
  `marley_forge_client` edits — its invariants are the pins, not the surface.
- **Forge recall (§18.3):** live forge calls deferred to /work promotion (Phase-1 draft is
  docs-only); recall from the standing record: `PR-claude-boot-decisions-key-the-restored-active-root-001`
  + `BF-claude-boot-root-resolved-before-restore-001` (both minted by #376's inspect F2 — the exact
  class this bug is; 376-fleet-rail-livewire.notes.md:284-285), the mutants-skip detach trap (5
  strikes; this change moves code near masked shims — re-run `--list` after),
  `PR-claude-trace-the-real-cargo-mutants-list` (run `--list -f` on the ACTUAL files at Validate),
  the never-logged-bearer contract (#64/#375), the #204/#205 env-blocked-capture fallback
  (mechanism + units when a driven capture is blocked).
- **Discovery (the sweep's load-bearing evidence — every cite read, not inferred):**
  - **The cwd-relative read, and WHERE it runs:** app.rs:1428-1430 — `let mcp_json =
    std::env::current_dir().ok().and_then(|cwd| std::fs::read_to_string(cwd.join(".mcp.json")).ok());`
    then :1431-1434 `forge_client = mcp_json.as_deref().and_then(forge_endpoint_from).map(ForgeClient::new)`.
    This sits EARLY in the masked `new_in` boot shim (:1380), before settings-driven work — the
    #163 shell restore that decides the real active project runs much later (`restore_shell`
    :1992; active-root correction for the files tree :2146-2153).
  - **Both consumers of the ONE string:** `mcp_json` occurs at :1428 (read), :1431 (build), and
    :2208 (`endpoint_for_brain(&url, mcp_json.as_deref())` — the #376 bearer decision inside the
    subscription gate). `forge_client` is built :1431-1434 and next touched at the struct init
    :2358. Nothing between build and init consumes either → the read+build MOVE to post-restore is
    mechanically clean (spec D5's feasibility evidence).
  - **The #376 placement precedent (the fix's landing zone):** app.rs:2192-2223 — the fleet
    subscription start, explicitly "resolved AFTER the shell restore above (the restored active
    project may differ from the launch cwd; inspect F2 caught the pre-restore placement subscribing
    to the wrong root on a Finder/restored-workspace boot)"; target = `shell.active_project().root`
    (:2205), guarded by `shell.project_count() > 0` (:2202; the recents fold repeats the guard
    :2237 — `active_project()` panics on an empty shell). The fix must land ABOVE :2202 so the
    bearer decision :2208 reads the retargeted string.
  - **The QA failure sites (the observable being fixed):** dispatch-send :6883-6896
    (`DispatchPhase::Failed("no forge client (.mcp.json)")` when `self.forge_client` is `None`);
    question-answer :6938-6950 (same string via `AnswerPhase::Failed`); ⌘⇧F :3674 (the #69
    re-fetch bails on `None` — dead under Finder launches since #64).
  - **The invariants that must NOT move (lib.rs read in full):** `forge_endpoint_from`
    lib.rs:124-141 — parses `mcpServers.forge.{url, headers.Authorization}` and REFUSES a
    non-loopback url up front ("the endpoint simply doesn't construct"); `endpoint_for_brain`
    :149-163 — same wall for the brain url, bearer sourced from `.mcp.json` IFF
    `endpoint.url == brain_url` else EMPTY ("a credential is never sent to a service it wasn't
    issued for"); `is_loopback_authority` :168-178 (localhost / 127.0.0.0/8 / ::1);
    `split_url` :182-188 (plain `http://` only). Redacted-Debug bearer contract in the crate doc
    (lib.rs:11). The crate stays pure over strings: "the caller reads the file — this stays pure"
    (:123).
  - **No per-switch re-read exists today (D1 parity evidence):** `sync_active_project`
    :6789-6799 re-walks files/tree/root only — forge_client is never rebuilt after boot; #376's
    D-OPEN-RETARGET locked boot-resolved-only v1. Keeping parity means one future slice retargets
    BOTH seams together.
  - **Cursor home unaffected (REQ-007 evidence):** `fleet_config_dir = config_dir.clone()` :1394
    (the marley config dir via `new_in`'s injected seam, app.rs:1369-1394) feeds `fleet_cursor_dir`
    at :2209/:2213 — orthogonal to `.mcp.json`; only the bearer INPUT string's source changes.
  - **Adjacent-but-different file (confusion guard):** `marley_core::marley_mcp_config_file_path()`
    = `<config>/mcp.json` (paths.rs:57-60, NO leading dot) — the #371 marley_mcp server-registry
    home, not the forge `.mcp.json`. Other in-tree `.mcp.json` readers: only the
    `check_forge.rs` example (resolves via `CARGO_MANIFEST_DIR`, examples/check_forge.rs:9 —
    deliberately non-cwd) — no other app-side reader to retarget.
  - **Behavior maps checked (§20 legs):** docs/warp_architecture/subsystems/06-platform-settings-infra.md
    — Warp config is app-home-keyed (user-visible TOML "path resolved in warpui_extras/warp_core";
    `HomeDirectoryWatcher` for live reload); nothing project-relative/cwd-keyed to observe.
    docs/zed_architecture/crates/ has no settings-resolution map (no settings.md; project.md maps
    dep edges only). Reference = N/A, Marley-specific boot wiring.
- **EARS drafted (7):** REQ-001 Finder-launch resolves against the restored active root (the QA
  repro, live-driven); REQ-002 dev `cargo run` whose cwd IS the active root byte-identical; REQ-003
  active-root-missing → cwd fallback (dev never regresses); REQ-004 loopback wall intact, lib.rs
  guards UNEDITED; REQ-005 bearer-iff-url-match intact + one-string-two-consumers + never
  logged; REQ-006 missing-everywhere/launcher-boot clean degrade (today's failure-record shape, no
  panic); REQ-007 subscription start-gate/target/cursor-home unaffected.
- **Decisions (reasoning):**
  - **D1 boot-once at the post-restore placement** — the restore is the only thing that knows the
    root (F2's lesson, the named PR class); #376 parity kept EXPLICIT (its D-OPEN-RETARGET locked
    boot-resolved-only v1) so a future retarget slice moves both seams together, never one.
  - **D2 active-root-first, cwd-fallback** — the fallback is what makes "a dev `cargo run` from the
    repo root must NOT regress" hold in full generality (including restored-active ≠ launch-cwd,
    which works today via cwd); it is inert on Finder launches (`/` holds no `.mcp.json`).
    Active-root-only was rejected for exactly that dev edge; cwd-first for perpetuating the class.
  - **D3 degrade byte-identical** — no new error surface; #384 (queued BEHIND this, ordering
    recorded on both tickets) owns the diagnosis UX and reads this ticket's outcome.
  - **D4 invariants pinned** — the fix moves WHICH file is read, never what the crate does with it;
    bearer/contents never logged; docs describe the file's SHAPE only (binding — no real
    bearer/contents ever quoted in planning docs).
  - **D5 one read, two consumers, moved together** — divergence between ⌘⇧F's client source and the
    live-wire bearer source would be a new bug class; the grep-verified consumer map makes the
    joint move clean.
  - **D-OPEN-FALLBACK-GRANULARITY** (missing-file-only vs any-`None` fallback; candidate
    missing-file-only so a misconfigured active-root file stays attributable — feeds #384) and
    **D-OPEN-SEAM-SHAPE** (pure candidate-order fn's home + signature; NOT marley_forge_client) left
    for Phase 2 with candidates named.
- **Forge ids:** ticket #381 `81348947-e095-46fb-b060-b90c30549c1b`; sprint #36 "M25 — App-Grade QA
  Hardening"; pipeline `1100a80c-0ecd-453d-a783-761708f8e8d9`; AAR
  `c11c5285-b9ce-4f23-b562-217a1355239d`.
- **Plan verification (Opus /work, 2026-07-22) — the D5 "move is clean" claim ATTACKED against source
  and CONFIRMED:**
  - `grep mcp_json crates/marley_app/src/app.rs` → EXACTLY 3 hits: :1428 (cwd read), :1431 (build),
    :2208 (`endpoint_for_brain(&url, mcp_json.as_deref())` bearer). No hidden consumer.
  - `grep forge_client` → the LOCAL var is used only at :1431 (build) and :2358 (struct init); every
    other hit is `self.forge_client` (the runtime FIELD: :3674 ⌘⇧F, :6883/:6938 fleet writes,
    :7350/:7375 — all post-boot). So between build and init the local is untouched.
  - Move target confirmed by reading the sites: the read/build (:1428-1434) moves DOWN to just above
    the #376 fleet gate (:2202), which already computes `shell.active_project().root` (:2205) behind
    `shell.project_count() > 0` (:2202). After the move: `mcp_json` is in scope for :2208 ✓ and
    `forge_client` for :2358 ✓, and the read now runs AFTER `restore_shell` so the active root is known.
  - **D-OPEN-SEAM-SHAPE narrowed (design confirms):** the pure resolution fn CANNOT live in app.rs —
    app.rs is the whole-file coverage exclude (gates.sh:222), and the spec requires the seam at cov/MSI
    100. So it needs a coverage-INCLUDED home: a NEW tiny module (e.g. `mcp_config.rs`), NOT app.rs, NOT
    fleet_live.rs (fleet-scoped). Design settles the exact name.
  - **D-OPEN-FALLBACK-GRANULARITY leaning missing-file-only:** the existence probe tests the FILE, so a
    present-but-unparseable active-root `.mcp.json` returns the active-root path (exists=true) → read →
    parse fails → `forge_client=None`, NOT a silent fall-through to a different repo's cwd file. That
    keeps a misconfigured active file attributable (feeds #384). Design confirms with a unit.
  - Verdict: plan solid; the move is genuinely a straight relocation + a pure candidate-order seam. No
    decisions revised.

## Phase 2 — Design

**Architecture / approach.** One NEW pure decision seam + a straight relocation of the existing masked
read+build. No new types, no forge-client change. §20 confirmed N/A (Marley-specific boot wiring — which
`.mcp.json` the forge sidecar client is built from is Marley's own #64 convention; the checked behavior
maps carry no project-relative config-resolution analog to match).

**The pure seam (D-OPEN-SEAM-SHAPE RESOLVED — a new module, because app.rs is coverage-excluded):**
`crates/marley_app/src/mcp_config.rs`:
```rust
pub fn mcp_json_path(
    active_root: Option<&Path>,
    cwd: Option<&Path>,
    exists: impl Fn(&Path) -> bool,
) -> Option<PathBuf>
```
Active-root candidate first (`active_root.map(|r| r.join(".mcp.json")).filter(|p| exists(p))`), else the
cwd candidate (same shape over `cwd`), else `None`. Pure over paths; injected existence probe; never
reads/logs/holds CONTENTS. Module doc distinguishes it from `marley_core::marley_mcp_config_file_path`
(`<config>/mcp.json`, no dot — the #371 marley_mcp server registry, a different file). Declared
`mod mcp_config;` in lib.rs (the private-mod-with-pub-fn pattern `fleet_live` uses; app.rs calls it as
`crate::mcp_config::mcp_json_path`, no re-export needed).

**D-OPEN-FALLBACK-GRANULARITY RESOLVED — missing-FILE-only.** `exists` probes the file (`is_file()` at
the call site, precise vs `exists()` which is true for a dir — the #205 `is_dir`/`exists` lesson
mirrored). A present-but-unparseable active-root `.mcp.json` → `mcp_json_path` returns the active path
(exists=true) → `read_to_string` succeeds → `forge_endpoint_from` parse fails → `forge_client = None`.
It does NOT silently fall through to a different repo's cwd file — a misconfigured active file stays
attributable (feeds #384). Pinned by a unit (T7).

**The move (D5 — read+build relocated, both consumers still fed):**
- DELETE app.rs:1428-1434 (the early cwd read + build).
- ADD, immediately above the #376 `fleet_incoming`/gate block (~:2200, AFTER `restore_shell`):
  ```rust
  // #381: resolve .mcp.json against the restored ACTIVE project root (fallback: launch cwd), AFTER
  // the shell restore — a Finder/open launch (cwd=/) still finds the active project's file. The one
  // string feeds BOTH the forge client and #376's bearer decision (:2208). D5; loopback wall pinned.
  let cwd = std::env::current_dir().ok();
  let active_root = (shell.project_count() > 0).then(|| shell.active_project().root.clone());
  let mcp_json = crate::mcp_config::mcp_json_path(active_root.as_deref(), cwd.as_deref(), |p| p.is_file())
      .and_then(|p| std::fs::read_to_string(p).ok());
  let forge_client = mcp_json.as_deref().and_then(forge_endpoint_from).map(ForgeClient::new);
  ```
  `mcp_json` is then in scope for `endpoint_for_brain(&url, mcp_json.as_deref())` (:2208) and
  `forge_client` for the struct init (:2358) — verified the only uses between are exactly those (Plan).
  `Project.root: PathBuf` (marley_project lib.rs:19) → `.clone()`→PathBuf → `.as_deref()`→`Option<&Path>`.
  The launcher-boot guard is intrinsic: zero projects → `active_root = None` → cwd-only candidate = today.

**File manifest:**
| File | Change |
|---|---|
| `crates/marley_app/src/mcp_config.rs` (NEW) | `pub fn mcp_json_path` + module doc + `#[cfg(test)] mod tests` (T1–T7). |
| `crates/marley_app/src/lib.rs` | `mod mcp_config;` (private mod, `fleet_live` pattern). |
| `crates/marley_app/src/app.rs` | DELETE the :1428-1434 early read+build; ADD the post-restore read+build above the :2200 fleet gate. Masked shim (coverage-excluded, `new_in` region). |

ZERO `marley_forge_client` edits (D4 — `forge_endpoint_from`/`endpoint_for_brain`/`is_loopback_authority`
lib.rs:124-178 pinned).

**Regression Test Plan.**
| Test (mcp_config.rs unit, injected `exists`) | Proves | AC |
|---|---|---|
| T1 active-root has `.mcp.json`, cwd empty → active path | active wins when only it has the file | REQ-001 |
| T2 active-root missing, cwd has it → cwd path | dev fallback | REQ-003 |
| T3 BOTH have it → active path (bug-class fix) | active must win over cwd | REQ-001/002 |
| T4 both missing → `None` | all-missing degrade | REQ-006 |
| T5 active-root `None` (launcher), cwd has it → cwd path | zero-project boot uses cwd only | REQ-006 |
| T6 active-root `None`, cwd `None` → `None` | no candidates, no panic | REQ-006 |
| T7 active present-but-(here modelled as exists=true) → active path, no cwd fall-through | missing-file-only granularity | (D-OPEN) |
| REQ-002 same-path: active_root == cwd → identical pick | dev `cargo run` byte-identical | REQ-002 |

- **Masked move (app.rs) — no unit (coverage-excluded); STRUCTURAL inspect** that the read is now below
  `restore_shell`, keyed to `active_root`, and both consumers (:2208, :2358) still read the one string;
  plus the existing suites green UNCHANGED: `marley_forge_client` loopback/bearer/redacted-Debug units
  (REQ-004/005), the #376/#372/#373 `fleet_livewire`+fleet suites (REQ-007), the #247 launcher boot,
  the #377/#378 "no forge client (.mcp.json)" failure-record shape (REQ-006).
- **Mutation:** run `cargo mutants --list -f crates/marley_app/src/mcp_config.rs` at validate for the
  REAL kill set (the `.filter`/`.map`/candidate-order predicates); the skip-detach trap habitat — the
  move is near masked shims, re-verify neighboring `#[mutants::skip]` via `--list`.
- **Live (REQ-001):** launch `open target/Marley.app` from a NON-repo cwd (e.g. `cd /tmp`), then trigger
  the forge-dependent affordance (`refresh_forge`, app.rs:3673 — the sprint re-fetch) and confirm it gets
  a client (pre-fix: `forge_client=None` → the guard returns early). Exact trigger chord read at validate.
  Machine-in-active-use fallback: units + structural mechanism carry REQ-001 (the resolution is
  unit-proven; the app wiring is a verbatim relocation of the shipped read+build, now sourced from the
  active root) — the #204/#205 env-blocked precedent; prefer an actual launch check if feasible.

**Risks / decisions:** the move is near the skip-detach trap habitat (5 prior strikes) — re-verify at
validate. The `then(|| ...root.clone())` allocates one PathBuf at boot (negligible). No `unwrap` on the
resolution path; the fn returns paths, never contents (D4).

## Phase 2 — Design (status)
Design PASS — seam shape + fallback granularity resolved with evidence; move manifest exact; test plan
per-REQ. Ready for Implement.

## Phase 3 — Implement
Built to the manifest:
- **NEW `crates/marley_app/src/mcp_config.rs`** — `pub fn mcp_json_path(active_root: Option<&Path>,
  cwd: Option<&Path>, exists: impl Fn(&Path)->bool) -> Option<PathBuf>`, implemented as
  `for base in [active_root, cwd].into_iter().flatten() { let path = base.join(".mcp.json"); if
  exists(&path) { return Some(path) } } None` — active-first, injected probe, first-existing-file wins.
  Module doc distinguishes it from `marley_core::marley_mcp_config_file_path` (#371). **No tests yet** —
  the T1–T7 units are written + run at Validate (the cov/MSI 100 surface; gate:4/5 + enforce-tests-ran).
- **`lib.rs`**: `mod mcp_config;` (between `lsp_host` and `mcp_host`, alphabetical; the `fleet_live`
  private-mod pattern — app.rs calls `crate::mcp_config::mcp_json_path`, no re-export needed).
- **`app.rs`**: the early cwd read+build (old :1424-1434) DELETED; the post-restore read+build ADDED
  above the #376 fleet gate (~:2195), calling `mcp_json_path(active_root.as_deref(), cwd.as_deref(),
  |path| path.is_file())`. `cwd` + `active_root` bound as locals;
  `active_root = (shell.project_count() > 0).then(|| shell.active_project().root.clone())` — the
  launcher-boot guard is intrinsic (zero projects → `None` → cwd-only). Post-edit grep confirms
  `mcp_json` in scope for the :2215 bearer and `forge_client` for the struct init; no borrow conflict.

**Deviation from design (1, cosmetic):** the fn is a `for`-loop over `[active_root, cwd]` rather than
the nested-closure sketch — identical behavior, avoids re-invoking a closure that borrows the injected
`exists`, reads cleaner. `cargo check -p marley` clean (only the pre-existing `block v0.1.6` dep note);
`cargo fmt --check` CLEAN. ZERO `marley_forge_client` edits (D4 pinned).

## Phase 3 — Implement (status)
Implement PASS — pure seam + the masked move compile fmt-clean; ready for Inspect.

## Phase 3.5 — Inspect
Two independent critics over the diff (mcp_config.rs + lib.rs + the app.rs move). **Verdict: PASS —
implementation CORRECT; 0 code fixes.** Lenses: (1) move + resolution correctness, (2) provenance +
security + state-integrity + the skip-detach trap.

**Confirmed correct (both critics, against source):**
- **Move behavior-preserving** — local `mcp_json` used only at build (:2197) + bearer (:2215); local
  `forge_client` only at build (:2202) + struct init (:2365); every other `forge_client` is
  `self.forge_client` (runtime field). Nothing in `[old-site, new-site]` referenced either local (grep
  zero hits 1426..2195). Both consumers sit below the new build site → in scope, read the new value.
- **Launcher-boot guard correct** — `active_project()` (tabs.rs:404) is a bare `self.projects[self.active]`
  index → panics on empty; `(project_count() > 0).then(|| active_project()…)` short-circuits so it's
  never called at zero projects. Matches the #376 block's identical guard right below.
- **Borrow clean** — `active_root` holds an OWNED `PathBuf` (`.clone()`); no borrow of `shell` survives;
  every later `shell` use is an immutable read before the struct move.
- **THE FOUNDATION (critic 1, load-bearing):** `active_project().root` is ABSOLUTE —
  `Project::discover_in` sets it from `current_dir()`/ancestors (marley_project lib.rs:36-48) and restore
  rehydrates the persisted absolute string gated on `root.is_dir()` (app.rs:1986). So
  `root.join(".mcp.json")` resolves independent of the process cwd `/` — the entire fix rests on this and
  it holds. (It's the same root value #376 already uses.)
- **`is_file()` correct** vs `exists()` — a dir named `.mcp.json` → false → fall-through; a present but
  unparseable regular file → true → returns the active path → parse fails → `None`, no cwd fall-through
  (missing-file-only, as designed). No unwrap/panic on the path.
- **Security/provenance CLEAN** — `git diff --stat -- crates/marley_forge_client/` EMPTY (loopback wall
  + bearer-iff-url-match reused verbatim, D4); no `println!/log/format!/dbg!` of `mcp_json`/bearer/endpoint
  on any added line; degrade byte-identical (all-missing → `None`; the failure-record sites :6895/:6951
  untouched); §20 clean (own code only); no secret/unsafe.
- **Skip-detach trap NOT triggered** — the diff inserts NO top-level fn into app.rs (both edits are
  statements inside `new_in`'s body, far from the 1379/1380 `#[mutants::skip]` boundary); the new fn lives
  in a separate file with no skip attr. `mcp_config.rs` is NOT in the gates.sh:222 coverage-ignore list →
  the seam IS gate-enforced (correct — the whole point of lifting it out of the excluded app.rs).

**Findings reviewed:**
- **[MED — critic 2] `mcp_json_path` has no units; both mutants (`None`, `Some(Default::default())`)
  survive → would fail cov:100 + MSI:100 IF shipped now.** Verdict: NOT an implement defect — the T1–T7
  units are the deferred VALIDATE deliverable (phase discipline: tests written+run at Phase 4; the design
  test plan already lists them). Acknowledged + will land at Validate; the critic even enumerated the
  cases (active-wins, both-missing→None, active-absent/cwd→cwd, both→active). The seam is not
  gate-enforced until then, which is exactly what Validate closes.
- **[LOW ×2 — critic 1] `is_file()` swallows a stat error on an unreadable PARENT dir** (pathological;
  the normal unreadable-file case still returns the active path; fall-through to cwd `/` is harmless →
  `None`) and **the both-exist precedence change** (that IS the fix, not a regression). Both REJECTED as
  no-fix.

**Fixes applied: none.** No `failure-record` (no real bug). No new prevention-rule — this ticket
INSTANTIATES the existing `PR-claude-boot-decisions-key-the-restored-active-root-001` (#376 F2); the
absolute-root foundation is captured here.

## Phase 3.5 — Inspect (status)
Inspect PASS — both critics converged on correct; the only MED is the deferred-tests gap that Validate
closes. Ready for Validate.

## Phase 4 — Validate
**Tests added: 7 `mcp_config.rs` units** (injected `exists` probe, no real FS) — active-wins (T1),
active-absent→cwd (T2, REQ-003), both→active (T3, the bug-class fix), both-missing→None (T4),
launcher active-None→cwd (T5, REQ-006), no-candidates→None (T6), dev same-root==cwd identical
(T7, REQ-002).

**Results:**
- `cargo nextest run -p marley mcp_config` → 7/7 pass; **full `-p marley` suite → 805 passed, 2 skipped**
  (the +7 are the new units; REQ-005 — nothing broke, incl. `fleet_livewire`/`headless_drive`).
- `cargo mutants --list -f crates/marley_app/src/mcp_config.rs` → exactly **2** viable mutants
  (`-> None`, `-> Some(Default::default())`), both killed (T1 expects `/proj/.mcp.json` ≠ `""`/`None`;
  T4 expects `None` ≠ `Some("")`). Gate:5 MSI 100 confirms.
- **`scripts/gates.sh --diff` → GATE GREEN [diff] 15/15** (coverage ≥100% incl. `mcp_config.rs` — the
  point of lifting the seam out of the coverage-excluded app.rs; MSI ≥100%). First run was RED on
  gate:1 rustfmt only (I'd added the `#[cfg(test)]` module after the earlier `cargo fmt`; rustfmt
  reflowed the multi-line `mcp_json_path(...)` test calls) → fixed at source with `cargo fmt`, re-ran
  green. Coverage/mutation were already green in the RED run (fmt is whitespace-only).

**LIVE (REQ-001 — proven END-TO-END on the running app, despite the active machine):**
- `open target/Marley.app` → the app's cwd is `/` (`lsof -d cwd` confirmed) — the EXACT Finder-launch
  bug condition. It **booted HEALTHY** (no panic) — validating the launcher guard + the
  `active_project()` borrow + the move in a real launch, not just units.
- The restored ACTIVE project is the Marley repo (`381-boot.png`: the Files tree shows
  `.cargo`/`.claude`/`Cargo.toml`), so `active_root` = the repo, which HAS `.mcp.json`.
- Opened the forge overlay via ⌘⇧F (toggle-forge; a terminal pane was focused so it was NOT
  editor-shadowed by #326) → the overlay shows **"loading sprint…"** (`381-forge.png`), NOT the
  "no forge client (.mcp.json)" degrade. That state appears ONLY when `refresh_forge` got a
  `Some(forge_client)` and spawned the fetch (`let Some(client) = self.forge_client.clone() else
  { return }`, app.rs:3681). So the app built the forge client from the ACTIVE ROOT's `.mcp.json`
  despite cwd `/` — **the fix, end-to-end**. Pre-#381 this same launch showed "no forge client"
  (the 2026-07-21 QA finding). Marley then quit cleanly (chad's machine left undisturbed).
- Harness note: the ⌘⇧P palette route to toggle-forge was fuzzy (no clean command match for "forge");
  the ⌘⇧F route with a terminal focused was the clean opener. (Also observed en route: the boot status
  bar reads "focus: terminal" while the Editor tab is highlighted — live evidence for #382, this
  sprint's next ticket.)

## Phase 4 — Validate (status)
Validate PASS — 7 units + suite green, 2 mutants killed, gate GREEN [diff]; REQ-001 proven live
(forge client built from the active root on a cwd=`/` launch). Ready for Complete.

## Phase 5 — Complete
- **CHANGELOG:** entry under `### Added` (the forge `.mcp.json` resolves against the restored active
  root; the Finder/`open` launch now builds a forge client; pure `mcp_config::mcp_json_path`).
- **Architecture docs:** `docs/marley_architecture/app_shell.md` — a boot note that the #64/#69 forge
  client resolves `.mcp.json` via `mcp_config::mcp_json_path` (active-root-first, cwd fallback,
  post-shell-restore), distinct from the #371 `<config>/mcp.json` server registry.
- **Forge knowledge:** `AD-claude-active-root-not-cwd-for-project-config-001` (8777d021 — app-scoped
  per-project config keys off the restored active root, never the process cwd; the pure resolver stays
  out of the coverage-excluded app.rs); `aar-submit` c11c5285 (completed, effectiveness 5, 1 novel
  finding). NO `failure-record` — inspect found no real bug (the one MED was the deferred-tests gap,
  closed at Validate). This INSTANTIATES `PR-claude-boot-decisions-key-the-restored-active-root-001`
  (#376 F2) — no new rule. `ticket-comment` #381 + a dependency-outcome `ticket-comment` onto **#384**
  (missing-file-only fallback → arm (c) "no forge client" is now the genuine-missing case; #381 fixed
  the common Finder-launch degrade so it's no longer a #384 trigger). `ticket-close` #381 → done.
- **Ticket doc** → `docs/planning/tickets/closed/`, status closed. **Pipeline** → `completed/`.

Complete PASS.
