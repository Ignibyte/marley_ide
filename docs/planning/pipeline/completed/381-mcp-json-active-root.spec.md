---
pipeline_id: 1100a80c-0ecd-453d-a783-761708f8e8d9
ticket: forge#381 (81348947-e095-46fb-b060-b90c30549c1b) · local docs/planning/tickets/open/TICKET-381-mcp-json-active-root.md
aar_id: c11c5285-b9ce-4f23-b562-217a1355239d
status: Phase 5 — Complete PASS
title: .mcp.json resolved against process cwd — a Finder/`open` launch silently has no forge client
type: bug
milestone: M25
references:
  - crates/marley_app/src/app.rs
  - crates/marley_forge_client/src/lib.rs
  - crates/marley_app/src/fleet_live.rs
  - docs/planning/pipeline/completed/376-fleet-rail-livewire.spec.md
---

## Title
Fix the boot `.mcp.json` resolution so a Finder/`open` launch gets a forge client. Today the read is
cwd-relative and EARLY: `std::env::current_dir().ok().and_then(|cwd|
std::fs::read_to_string(cwd.join(".mcp.json")).ok())` (app.rs:1428-1430) runs in `new_in` BEFORE the
#163 shell restore (`restore_shell`, :1992) decides which project is actually active
(:2146-2153). Launched via `open target/Marley.app`, cwd is `/`, the read misses, and
`forge_client = None` — so the 2026-07-21 live QA run saw fleet dispatch-send (app.rs:6883-6896) and
question-pick (:6938-6950) both record "no forge client (.mcp.json)" on a machine whose repo HAS the
file. The same string feeds two consumers: the forge-client build (:1431-1434, stored :2358 — ⌘⇧F's
client, dead under Finder launches since #64, :3674) and #376's `endpoint_for_brain` bearer decision
(:2208) — so the live-wire subscription silently runs with an empty bearer too. This is the
`PR-claude-boot-decisions-key-the-restored-active-root-001` class (the #376 inspect-F2 sibling: a
boot decision keyed off a pre-restore value). Fix: resolve `.mcp.json` against the restored ACTIVE
project's root, at the same post-shell-restore placement #376 gave the subscription start
(:2192-2223), with the launch-cwd read kept as fallback so a dev `cargo run` never regresses. The
`marley_forge_client` loopback wall and bearer-iff-url-match invariants are PINNED, not edited.

## Scope
### In
- **A NEW pure resolution seam** (shape/home = D-OPEN-SEAM-SHAPE; cov/MSI 100): the candidate-order
  decision — active project root first, launch cwd as fallback (D2) — taking the candidates + an
  injected existence probe and returning the path to read (or `None`). Pure over paths; it never
  reads, logs, or holds file CONTENTS.
- **Moving the masked read+build** (app.rs:1428-1434) from its pre-restore site to the
  post-shell-restore placement, ABOVE the #376 fleet-subscription gate (:2202) so the single
  `mcp_json` string keeps feeding BOTH downstream consumers (`endpoint_for_brain` :2208 and the
  `forge_client` struct init :2358). Verified feasible: the only `mcp_json` uses are :1428/:1431
  (read/build) and :2208; the only `forge_client` use between build and struct init is the init
  itself — a straight move is clean.
- **Launcher-boot guard**: at the zero-project boot there is no active project (`active_project()`
  panics on an empty shell) — mirror the existing `project_count() > 0` guards (:2202, :2237); the
  resolution then has only the cwd candidate = today's behavior verbatim.
- **Regression pins:** the `marley_forge_client` lib.rs suites (loopback wall, bearer-iff-url-match,
  redacted Debug) green UNCHANGED; the #376/#372/#373 livewire + fleet suites; the #377/#378
  failure-record shapes ("no forge client (.mcp.json)") for the genuinely-missing case; the #247
  launcher boot.

### Out (explicitly deferred)
- **Re-resolving on an active-project SWITCH** — parity with #376's locked D-OPEN-RETARGET
  resolution (boot-resolved-only v1): no runtime re-target exists for the subscription either, and
  `sync_active_project` (:6789-6799) rebuilds only the files tree today. A future retarget slice
  moves the subscription AND this resolution together.
- **Any edit to `forge_endpoint_from` / `endpoint_for_brain` / `is_loopback_authority`**
  (lib.rs:124-178) — invariants pinned by REQ-004/005, not modified.
- **The missing/misconfigured-file diagnosis UX** — #384's ticket, queued BEHIND this one; it reads
  this ticket's outcome. This ticket keeps the degrade shape byte-identical to today (D3).
- **An upward directory walk** (git-style discovery from a nested root) — `.mcp.json` lives at the
  project root by convention; two candidates suffice for v1.
- **A settings field / env var for the `.mcp.json` path**; **watching or reloading** the file at
  runtime (settings load once at boot; no reload path exists).

## Reference (§20)
**N/A — Marley-specific boot wiring; no reference-app analog.** Which file the forge-sidecar client
is configured from is Marley's own `.mcp.json` convention (#64). The checked behavior map
`docs/warp_architecture/subsystems/06-platform-settings-infra.md` maps Warp's config as
app-home-keyed (user-visible TOML settings file, "path resolved in `warpui_extras`/`warp_core`"; a
`HomeDirectoryWatcher` for live reload) — nothing project-relative or cwd-keyed to observe or match.
`docs/zed_architecture/` carries no settings-resolution behavior map for this seam (crates/ has no
settings.md; project.md maps dependency edges only). Clean-room §20 untouched: no Warp (AGPL) / Zed
(GPL) source consulted — only our own maps.

### Prior art
1. **Behavior maps — checked, no owner.** `docs/warp_architecture/subsystems/06-platform-settings-infra.md`
   (app-home TOML + home watcher; no per-project config-resolution behavior described);
   `docs/zed_architecture/crates/` has no settings map to consult for this seam. Research only.
2. **Published — the platform pattern this bug violates.** macOS LaunchServices launches
   (Finder/`open`) start a GUI app with cwd `/`; process cwd is meaningless to a GUI app. The
   established convention for workspace-scoped config (VS Code `.vscode/`, JetBrains `.idea/`, git's
   own worktree discovery) resolves against the OPENED workspace/document root, never the process
   cwd. The fix direction is exactly this convention: key the read off the restored active project.
3. **OUR OWN tree — the highest-yield leg; the pattern already exists here three times.**
   - **App-scoped config is never cwd-keyed:** `marley_core::marley_config_dir()` (paths.rs:38,
     `~/.marley/config`) injected via the `new_in(config_dir)` seam (app.rs:1369-1394) so tests
     never touch the real home; `marley_settings` resolves its TOML under that injected dir (crate
     doc, lib.rs:1-6). The `.mcp.json` cwd read is the outlier.
   - **The post-restore correction family this fix joins:** (a) the files-tree/⌘P root re-derive
     when `active_root != project_root` (app.rs:2146-2153); (b) the #376 subscription start-gate,
     deliberately placed AFTER the restore because "the restored active project may differ from the
     launch cwd" (:2192-2199, inspect F2). This ticket is member (c).
   - **Adjacent-but-different file, recorded to prevent confusion:**
     `marley_core::marley_mcp_config_file_path()` = `<config>/mcp.json` (paths.rs:57-60, no leading
     dot) is the #371 marley_mcp server-registry home — NOT the per-project forge `.mcp.json`; it is
     not a resolution helper for this seam.
   - **Non-cwd resolution precedent in our own examples:** `check_forge.rs` resolves `.mcp.json`
     via `CARGO_MANIFEST_DIR` (examples/check_forge.rs:9) — compile-time repo-relative, deliberately
     not cwd.
   - **No shipped crate owns "resolve a file against a project root"** — it is a path join plus an
     existence probe; nothing to adopt beyond the placement precedent. Checked marley_core paths,
     marley_settings, marley_forge_client (whose doc pins "the caller reads the file — this stays
     pure", lib.rs:123): none owns this seam.

## Locked-In Decisions
- **D1 — Resolve ONCE at boot, at the #376 post-shell-restore placement, keyed to the restored
  ACTIVE project's root.** The restore is the only thing that KNOWS the root (the F2 lesson,
  `PR-claude-boot-decisions-key-the-restored-active-root-001`); parity with #376's D1 + its locked
  D-OPEN-RETARGET = boot-resolved-only v1 — stated explicitly so the two seams stay in step and a
  future retarget slice moves both together. **Rejected:** re-read per active-root switch (no such
  path exists for the subscription either; `sync_active_project` :6789-6799 rebuilds nothing
  forge-side); keeping the early site and predicting the root (prediction is what F2 caught).
- **D2 — Candidate order: ACTIVE ROOT first, launch cwd as fallback.** The active root's
  `.mcp.json` wins when present. When the active root has NO `.mcp.json`, fall back to today's cwd
  read — a dev `cargo run` from a repo root never regresses, including the edge where the restored
  active project differs from the launch-cwd repo. On a Finder launch cwd is `/` (no `.mcp.json`) —
  the fallback is inert. Launcher boot (zero projects) → cwd candidate only, today verbatim.
  **Rejected:** active-root-only (regresses the dev edge above — that launch has a working client
  today); cwd-first (perpetuates the bug class — the restored root must win when both exist).
- **D3 — Missing everywhere → the degrade shape is BYTE-IDENTICAL to today.** `forge_client = None`;
  ⌘⇧F (:3674) and the fleet writes record "no forge client (.mcp.json)" (:6888, :6944);
  `endpoint_for_brain(url, None)` still constructs a loopback brain endpoint with an EMPTY bearer
  (lib.rs:154-158) so the read-only subscription keeps working where it works today. NO new error
  surface, string, or UI in this ticket — the diagnosis UX is #384's, queued behind this and reading
  this outcome.
- **D4 — The security invariants are PINNED, not edited.** `forge_endpoint_from`'s loopback wall
  (lib.rs:124-141; `is_loopback_authority` :168-178 — the bearer never leaves loopback) and
  `endpoint_for_brain`'s bearer-iff-url-match (:149-163 — a credential is never sent to a service it
  wasn't issued for) ship UNCHANGED: the fix moves only WHICH FILE is read. The redacted-`Debug` /
  never-logged bearer contract extends to the new seam — the resolution fn handles PATHS, never
  contents; no new log/format of the string or bearer anywhere. These planning docs describe the
  file's SHAPE only (`mcpServers.forge.{url, headers.Authorization}`), never real contents.
- **D5 — ONE read, TWO consumers, moved TOGETHER.** The single `mcp_json` string keeps feeding both
  the forge-client build and the #376 bearer decision (:2208) so ⌘⇧F's client and the live-wire
  bearer can never diverge on source — today's property, preserved at the new placement (verified:
  no consumer exists between the current build site and the struct init :2358).

**D-OPEN — RESOLVED (Phase 2, with evidence):**
- **D-OPEN-FALLBACK-GRANULARITY → missing-FILE-only.** The injected existence probe tests the FILE
  (`is_file()` at the call site). A present-but-unparseable active-root `.mcp.json` returns the active
  path → read succeeds → `forge_endpoint_from` parse fails → `forge_client = None`; it is NOT masked by
  a different repo's cwd file. A misconfigured active file stays attributable (feeds #384). Pinned by a
  unit. **Rejected:** fall-through on any `None` (would silently hide a broken active-root file behind
  an unrelated repo's config).
- **D-OPEN-SEAM-SHAPE → a new module `crates/marley_app/src/mcp_config.rs`.** `pub fn mcp_json_path(
  active_root: Option<&Path>, cwd: Option<&Path>, exists: impl Fn(&Path)->bool) -> Option<PathBuf>`.
  It CANNOT live in app.rs (the whole-file coverage exclude, gates.sh:222 — the spec requires cov/MSI
  100 on the seam); `fleet_live.rs` is fleet-scoped (wrong home); `marley_forge_client` stays pure over
  strings (lib.rs:123). `mod mcp_config;` in lib.rs (the `fleet_live` private-mod pattern). Module doc
  distinguishes it from `marley_core::marley_mcp_config_file_path` (the #371 `<config>/mcp.json`).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the app is launched with a process cwd outside any project (the Finder/`open target/Marley.app` path — cwd `/`) and the restored ACTIVE project's root contains `.mcp.json`, the app shall build the forge client from the ACTIVE root's file — fleet question-pick and dispatch-send reach the wire, never recording "no forge client (.mcp.json)". | live-drive: the QA repro (`open` the bundle from a non-repo cwd; fleet rail → pick + dispatch; the failure records absent); pure units: resolution picks the active root when cwd holds nothing |
| REQ-002 | WHEN launched from a dev shell (`cargo run`) whose cwd IS the restored active project's root, the app shall behave byte-identically to today — same file resolved, same client, same bearer decision. | pure unit: both candidates name the same path → identical pick; regression: the existing forge-client/#69 ⌘⇧F suites green unchanged |
| REQ-003 | WHEN the active project's root has NO `.mcp.json` but the launch cwd does (a dev launch with a different restored active project), the app shall fall back to the cwd read — today's dev behavior never regresses. | pure units on the candidate order (active missing → cwd; both missing → `None`); the D-OPEN-FALLBACK-GRANULARITY pick pinned by its own unit in Phase 2's test plan |
| REQ-004 | The app shall construct NO endpoint for a non-loopback url regardless of WHICH candidate resolved the file — `forge_endpoint_from` / `endpoint_for_brain` / `is_loopback_authority` (lib.rs:124-178) ship UNEDITED. | §18.1 inspect: the diff contains no `marley_forge_client` guard edits; the existing loopback suites green unchanged; unit: a non-loopback fixture fed through the moved path yields no client |
| REQ-005 | The fleet subscription's bearer decision shall keep receiving the SAME single string the forge client was built from — the bearer rides IFF the `.mcp.json` forge url matches the brain url, else empty (lib.rs:149-163) — and no path shall log or format the bearer or the file contents. | the existing `endpoint_for_brain` units green unchanged; inspect checkpoint: ONE read site, both consumers downstream (D5); negative sweep for any new log/format of the string/bearer |
| REQ-006 | WHEN no candidate holds `.mcp.json` — or the boot is the zero-project launcher, which has no active root — the app shall degrade EXACTLY as today: `forge_client = None`, the fleet writes record "no forge client (.mcp.json)" (:6888, :6944), no panic at the launcher state. | pure unit: all-missing → `None`; no active root → cwd-only candidate; the #247 launcher-boot + #377/#378 failure-record suites green unchanged |
| REQ-007 | The fleet subscription's start-gate, target, and cursor home shall be unaffected — `subscription_target` (:2203-2206) and `fleet_cursor_dir` keyed off the marley config dir (:1394, :2209) unchanged; only the bearer INPUT string's source moves. | the #376/#372/#373 livewire + fleet suites green unchanged; §18.1 inspect on the boot diff (a placement-plus-source change around :2192-2223, nothing else) |

## Floors (constitution)
The pure resolution seam at **cov/MSI 100** (candidate order, existence-probe injection — no real
filesystem in units). MASKED: the moved read+build glue inside the already-masked `new_in` boot shim
(app.rs is coverage-excluded). Typed handling, no `unwrap` on the resolution path; the fn returns
paths, never contents. At Validate run `cargo mutants --list -f` on the ACTUAL touched files (the
syntactic-form lesson) and re-verify neighboring `#[mutants::skip]` bindings — this change MOVES
code inside/near masked shims, the skip-detach trap's exact habitat (5 prior strikes).

## Phase Plan
- **P2 Design** — settle D-OPEN-FALLBACK-GRANULARITY + D-OPEN-SEAM-SHAPE with evidence; the exact
  move manifest (where :1428-1434 lands relative to the :2202 fleet gate; the launcher-boot guard);
  re-grep at design time that no consumer of `mcp_json`/`forge_client` slipped in between the old
  and new sites; per-REQ test plan incl. the live-drive repro script (launch via `open` from a
  non-repo cwd) and its env-blocked fallback (mechanism + units, the #204/#205 precedent).
- **P3 Implement** — the pure seam first (candidate-order fn + units), then the masked move; ZERO
  `marley_forge_client` edits.
- **P3.5 Inspect** — adversarial: does ANY path still read cwd first when an active root exists? do
  both consumers still read the one string? any new log/format of bearer or contents? does the
  launcher boot panic on `active_project()`? skip-detach re-verify via `--list`; provenance (§20).
- **P4 Validate** — write + RUN the units and regressions per REQ; `cargo mutants --list -f` on the
  actual touched files before claiming the kill set; gate green (`--diff`); the live-drive QA repro
  (or a documented env-block with mechanism + units carrying it).
- **P5 Complete** — CHANGELOG; ticket-comment the outcome onto #384 (which reads it); AAR capture;
  archive; close #381.
