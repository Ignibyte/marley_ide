# Rustal's Rust quality gates on the Marley crates — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-447-rustal-quality-gates.md
- **Pipeline spec:** 447-rustal-quality-gates.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** Chad, 2026-09-22: "lets mimic the quality gates on rustal, clean up your mutation
  packages, recalibrate the workflow to make sense, clean up anything and make sure high quality
  rust items are here and lets continue working on the remaining tickets". The workflow and the
  mutation clean-up shipped first (`952f27ace4`, `dbad7c4c45`); this ticket is the gates.
- **Classification / tier:** chore, large. Marley-owned files, plus one Zed file (`clippy.toml`)
  with its ledger row. #439 was parked back in `queued/` so this pipeline could be the only
  active one; its re-verified seams are in its notes.
- **Recall (§18.3):**
  - `BF-clippy-toml-not-in-receipt-fingerprint-001`: every gate-defining config belongs in the
    fingerprint, so `.semgrep.yml`, `rustfmt.toml` and `.config/typos.toml` join it.
  - `PR-claude-gate-change-updates-quality-bar-row-001`: a gate change updates the gate table
    in the same change; in the fork that table is CONSTITUTION §0.
  - `BF-claude-approximated-the-gate-command-and-reported-green-001` and
    `L-claude-443-run-the-gate-not-a-reconstruction-001`: prove each new gate step with its
    exact command, `-D warnings` included.
  - `BF-claude-rustdoc-private-intra-doc-link`: the new doc comments must not link to private
    items.
  - Brain: consultation `58ac2b40cfaf465eaf2ddcef2dc961a6`, nothing on this seam.
- **Discovery.** The rustal gate map (an Explore sweep of `/srv/stacks/rustal`, this session)
  and a measurement with the table applied to all seven crates at warn level: **679 hits**, 233
  machine-applicable. By crate: marley_terminal 234, marley_mcp 217, marley_workbench 107,
  marley_fleet 82, marley_remote 16, marley_agent 14, marley_rail 9. By lint: `unwrap_used`
  126 and `expect_used` 107 (mostly test code, which the `clippy.toml` allowances clear),
  `must_use_candidate` 83, `too_long_first_doc_paragraph` 52, `unused_results` 46,
  `missing_const_for_fn` 32, `use_self` 27, `future_not_send` 26, then a long tail
  (`redundant_closure_for_method_calls`, `missing_errors_doc`, `doc_markdown`,
  `needless_pass_by_value`, `needless_pass_by_ref_mut`, `redundant_pub_crate`,
  `unreachable_pub`, casts, `missing_debug_implementations`, `missing_docs`). Starting points
  of the other checks: `cargo sort --check` fails on all seven manifests (only because
  `[lints]` sits above the dependency tables), `taplo fmt --check` passes, `typos` is clean
  repo-wide, the Marley Rust source has no TODO markers, and no doc comment holds an example.
  Semgrep 1.156.0 is installed, rustal's pin.

### Design
- **Lint tables.** One table, written seven times (cargo gives a crate either the workspace
  table or its own). A comment on each points at CONSTITUTION §14 and names the crate-specific
  allows. `marley_workbench` expects `future_not_send` to need an allow: gpui runs `!Send`
  futures on its foreground executor by design; the allow is kept only if the hits are all of
  that kind.
- **Fix order.** `clippy.toml` allowances, then `cargo clippy --fix` crate by crate for the
  machine-applicable suggestions, then the rest by lint:
  - `missing_docs`, `missing_errors_doc`, `missing_panics_doc`, `too_long_first_doc_paragraph`,
    `doc_markdown`: doc comments, no intra-doc links to private items;
  - `unused_results`: bind or act on the value where it means something; a builder's
    `&mut Self` return is chained or dropped explicitly;
  - `must_use_candidate`, `missing_const_for_fn`, `use_self`: mechanical;
  - `needless_pass_by_value`, `needless_pass_by_ref_mut`, casts: judged one by one, since each
    can change a signature or a value;
  - `unreachable_pub` against `redundant_pub_crate`: D3;
  - `missing_debug_implementations`: `#[derive(Debug)]`, or a manual `Debug` with
    `finish_non_exhaustive()` where a field has none (gpui subscriptions and tasks).
- **Gates.** `script/gates.sh`: gate:13's SAFETY and transmute patterns; gate:14's rustdoc
  warning check and Rust TODO scan; gates 17 to 20; explicit modes; BLOCKED heavy gates after a
  static red; the receipt removed at start and written only when the start and end
  fingerprints match. `.claude/hooks/lib-hook-helpers.sh`: the fingerprint's path list. New
  `.semgrep.yml`, Marley-owned (added to `marley_owned_path`).
- **Docs.** CONSTITUTION §0's table, scope notes and tools; §14's lint rule; the Test and Plan
  commands name `--diff` and `--full`; the ledger row for `clippy.toml`.
- **File manifest.**
  - Marley: `crates/marley_{agent,fleet,mcp,rail,remote,terminal,workbench}/Cargo.toml` and
    their `src/**` and `tests/**` (the fixes); `script/gates.sh`; `.semgrep.yml` (new);
    `.claude/hooks/lib-hook-helpers.sh`; `.claude/commands/pipeline/{plan,test}.md`;
    `CONSTITUTION.md`; `CHANGELOG.md`.
  - Zed: `clippy.toml` (two keys, a ledger row in `docs/marley/zed-touchpoints.md`).

### Test plan
| REQ | Proof |
|---|---|
| 001, 002 | gate:2 and gate:12 green over the seven crates |
| 003 | smoke: a temporary file with `unsafe` under a previous-line `// SAFETY:` passes gate:13; one with a bare `transmute(` fails it |
| 004 | smokes: a temporary `// TODO: x` in a Marley `.rs` fails gate:14; a broken intra-doc link fails it through the `warning:` check |
| 005 | smoke: a Marley manifest with an unsorted dependency fails gate:17 |
| 006 | smoke: a temporary misspelling fails gate:18 |
| 007 | smoke: a temporary Marley crate whose library has no tests fails gate:19 (or the jq check fed a listing with an empty suite) |
| 008 | smoke: a temporary `std::process::exit(1)` in a Marley library fails gate:20 |
| 009 | smoke: `script/gates.sh` with no argument and with `--bogus` exits 2 and prints the usage line |
| 010 | smoke: a FULL run with a static red reports gates 4 and 6 BLOCKED |
| 011 | smoke: a stale receipt is gone once a run starts; a run whose tree changes mid-run writes none |
| 012 | smoke: editing each named file changes `gate_state_hash` |
| 013 | the manifests carry no `doctest = false`, and gate:3 prints the doctest runs |
| 014 | `cargo nextest run` over the seven crates and gate:4 at 100% |
| 015 | `script/gates.sh --diff` green |

### Risks
- Lint fixes in the ported crates can change behavior (casts, signature changes). Each is
  judged on its own; the crates' tests and 100% coverage must hold.
- `must_use_candidate` and `unreachable_pub` touch public APIs across crate boundaries; the
  callers are all Marley crates, fixed in the same change.
- The DIFF gate will cover all seven crates, since each manifest changes.

## Phase 2 — Code
- **Built.**
  - The lint table in the seven Marley manifests (`[lints] workspace = true` and
    `doctest = false` gone), `clippy.toml`'s test allowances with their ledger row. The count
    went 679 → 0, and `cargo clippy -p <the seven> -p zed --all-targets --all-features --
    -D warnings` is clean at deny level.
  - The fixes, by kind: doc sections and summaries; `#[must_use]`, `const fn`, `Self`;
    signatures that borrow what they only read (`transport`'s accept loop, connection and
    stream, `snapshot_changed`, `tool_result`, `register_sidebar`, `Rail::new`,
    `TerminalSession::spawn`); casts through `try_from` with a saturating fallback, and element
    ids from `EntityId` and `u64` directly; a manual `Debug` for `TerminalSession` and for
    `Rail`, each with a test; module visibility (`jsonrpc` and `marley_workbench_tests` are
    `pub`, `pty_os` is a child of `session`, `KeptSidebar` is public and re-exported,
    `TerminalFactory` private); map literals in place of discarded `insert` results in tests.
  - TICKET-444's eight discards: transport's thread spawn (logged at warn), connection IO
    error and effect send (debug); `apply_hook` in the pump (debug, plus the new test
    `pump_drops_a_hook_that_arrives_before_init_shell`); the seed helper's three hooks
    propagate with `?`, so `seed_finished_block_for_test` returns `Result`; the reap signal
    matches ESRCH and logs any other failure.
  - `script/gates.sh`: explicit modes with a usage error (exit 2) before any gate; the receipt
    removed at start and written from the start fingerprint only when the end one matches (a
    `receipt:` step fails the run otherwise); heavy gates BLOCKED after a static red; gate:13's
    previous-line SAFETY and bare `transmute`; gate:14's `warning:` check and Rust TODO scan;
    gates 17 to 20. New `.semgrep.yml`. `lib-hook-helpers.sh`: the fingerprint's paths and
    filter, `.semgrep.yml` in `marley_owned_path`. The hook messages name `--diff`.
  - Docs: CONSTITUTION §0 (modes, table, retired numbers, known scope, receipt, tools), §14 (the
    lint table and the visibility rule), §15 (the fingerprint and the revocation); the Test
    command names `--full`; the ledger's owned-path prose gains `script/mutation.sh` (a gap
    older than this ticket) and `.semgrep.yml`.
- **Deviations from the plan.**
  - `let_underscore_must_use = "deny"` joined the table: Zed's `.rules` bans `let _ =` on a
    fallible call and the lint enforces it. That made #444's discards compile errors, so #444
    is folded in (spec: REQ-016) and closes with this ticket.
  - Three cargo-group lints are allowed with a comment. `cargo_common_metadata`,
    `negative_feature_names` and `redundant_feature_names` read every manifest in the
    workspace, so they report Zed's packages (`html_to_markdown`'s metadata) and Zed's
    `test-support` convention. The counting script missed them: their diagnostics carry no
    source span. The deny-level run found them.
  - `marley_workbench` allows `unused_results`, with a comment: gpui's registration calls
    return `&mut App` for chaining and `.log_err()` (the `.rules` idiom) returns an `Option`.
    Its other hits were fixed where the fix read better anyway (the `actions!` chain).
  - `derive_partial_eq_without_eq` fired inside gpui's `actions!` expansion; the macro's
    per-action attribute slot takes `#[derive(Eq)]`, so there is no allow.
  - `pty_os` became `session::pty_os` through `#[path = "pty_os.rs"]`, which keeps the path the
    coverage exclusion, CONSTITUTION §0 and the history cite; `rail.rs` already uses `#[path]`
    for its tests.
  - `raw` → `frame` in the pump's DCS arm: semgrep 1.156's Rust parser reads `&raw.payload` as
    the start of a raw borrow and fails, and `--strict` makes that a red.
  - gate:20 passes `--no-git-ignore` (semgrep otherwise scans only what git lists, so a new
    untracked file would go unscanned), `--metrics=off` and `SEMGREP_ENABLE_VERSION_CHECK=0`
    (no network call from the gate).
  - REQ-007 and REQ-008 now say what rustal's rules do: every suite but a binary's must hold a
    test, and `command-injection-risk` flags a `Command::new` whose program is not a literal.
  - The POST lock in `transport` drops its guard explicitly after `handle_message` rather than
    carrying an allow for `significant_drop_tightening`.
- **Review of the diff** (behavior hidden in lint fixes): the rewrites preserve behavior
  (`map_or_else` arms checked against the `match` they replaced, `result_response`'s key
  order under `preserve_order`, the hex lookup against `{:02x}`, `as_chunks::<2>` against
  `chunks_exact(2)`). The deliberate changes are the log lines above, `eprintln!` → `log::warn!`
  in the reap give-up, casts that saturate where `as` wrapped (unreachable for u16 grid
  sizes and u128 millis), and the seed helper's `Result`. One ignored signal is left as it
  was: `Workspace::activate_item`'s `bool` in the rail's terminal click, which the gpui
  crate's `unused_results` allow now covers.
- **Checks.** clippy deny-level clean over the seven and `zed`; `cargo nextest run` over the
  seven: 308 passed; `script/gates.sh --fast`: 15 green, gate:12 red on the integration
  test's reason sitting above its `#![allow]`, now moved onto the line; usage smokes exit 2.

## Phase 3 — Test
- **Checklist** (no `TaskCreate` in this harness; one row per test-plan row plus the gate):
  - [x] REQ-001, 002 — gate:2 and gate:12 green (the `--diff` run below).
  - [x] REQ-003 to 008, 011 (the mid-run half), 012, 016 — the smoke harness, below.
  - [x] REQ-009 — usage: no argument, `--bogus`, `--full --diff` and a bare `full` each exit 2
    with the usage line and run no gate (no `scope:` line).
  - [x] REQ-010, 011 — a real `--diff` run with a planted misspelling and a stale receipt.
  - [x] REQ-013 — no `doctest` key in any Marley manifest; gate:3 prints `Doc-tests` for all
    seven crates (none holds an example yet, so each runs 0).
  - [x] REQ-014 — 308 tests in the seven crates; gate:4 at 100.00% of 5,880 lines.
  - [x] REQ-015 — `script/gates.sh --diff`: GATE GREEN [diff].
- **Tests added** (Code phase, run here): `debug_names_the_session_by_its_id`,
  `pump_drops_a_hook_that_arrives_before_init_shell` (marley_terminal),
  `the_rail_debugs_as_its_width_and_rows` (marley_workbench); `spawn_real_pty_succeeds` now
  asserts the grid size and that nothing runs, in place of a discarded id.
- **The smoke harness** (`scratchpad/gate-smoke.sh`) extracts each gate function from
  `script/gates.sh`, so the shipped code is what runs, plants a fault, checks the exit status
  and removes the fault; the tree was byte-identical afterwards. 42 of 42 as expected:
  - gate:13: SAFETY on the line above and on the line itself pass; none, or two lines up,
    fails; bare `transmute(`, `transmute::<` and `mem::transmute` fail; `transmuted` passes; a
    Zed crate's added `unsafe` passes with SAFETY above and fails without.
  - gate:14: the tree passes; a `TODO:` and a `FIXME(` in a Marley `.rs` fail; an unused
    manifest key fails through the `warning:` check. That smoke caught a hole first: with
    `cargo doc --quiet`, cargo prints none of its own warnings, so the check could not see
    them. `--quiet` is gone from gate:14. (The plan's "broken intra-doc link" row proves the
    older `-D warnings` half, not the new check, so the manifest key replaced it.)
  - gate:17: an unsorted dependency fails (cargo-sort); a misformatted key fails (taplo).
  - gate:18: a planted `teh` fails.
  - gate:19: the real listing passes; the same listing with `marley_agent`'s suite emptied
    fails; an empty binary suite passes; an empty, non-JSON or suite-less listing fails.
  - gate:20: `std::process::exit` in an untracked library file fails (so `--no-git-ignore`
    works); `Command::new(program)` fails; `Command::new("ls")` passes.
  - receipt: an unchanged tree passes; a new non-Rust file under `crates/marley_*` mid-run
    fails; restored, it passes; no start fingerprint fails. Editing `rustfmt.toml`,
    `.config/typos.toml`, `.semgrep.yml` or a Marley manifest changes the fingerprint and
    restoring it restores the hash; editing `docs/marley/README.md` does not change it.
  - REQ-016: `let _ = "1".parse::<u8>();` in `marley_agent` fails clippy with
    `let_underscore_must_use`; restored, clippy passes.
- **REQ-010/011, the real run.** A stale receipt written at 22:15:06 was gone when the run
  printed its `scope:` line. The run then went red at gate:18 (the plant) and gate:9
  (cargo-shear flags the planted file as unlinked from any module), reported
  `BLOCKED gate:4` and `BLOCKED gate:6`, exited 1 after 49 s, and left no receipt.
- **The gate.** `script/gates.sh --diff`, scope: the seven Marley crates plus `zed`; 19 passed,
  0 failed, `GATE GREEN [diff]` after 99 s; gate:3 400 tests passed and 308 in the coverage
  run; gate:4 TOTAL 5,880 lines, 0 missed, 100.00%; the receipt matches `gate_state_hash`.
- **Live drive: N/A, confirmed.** The rail's render changes are signatures (`&self` dropped,
  `&mut` to `&`) and element ids now built from `EntityId` and `u64` without a cast. gpui builds
  `ElementId::NamedInteger(name, id)` from the same `u64` through the `EntityId`, `usize` and
  `u64` conversions alike (`crates/gpui/src/window.rs:7471-7493`), so the element tree and the
  hit targets are the ones W2 shipped, and the 26 driven tests render and click those rows,
  menus and disclosures. Nothing on screen can differ, and a headless drive borrows one of
  Chad's workspaces (L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001).
- **Pre-existing — not in scope:** upstream Zed's
  `#[ignore = "This test has timing issues across platforms."]` at `crates/zed/src/zed.rs:3614`
  is the one skipped test in gate:3.

## Phase 4 — Complete
- **Checklist** (no `TaskCreate` in this harness): [x] document · [x] capture knowledge ·
  [x] close the tickets · [x] archive · [x] commit.
- **Documented.**
  - `CHANGELOG.md`: Added (rustal's quality gates), Changed (the gate's explicit modes and
    receipt), Fixed (#444's dropped errors).
  - `docs/marley/README.md`: the Standards line names the lint table.
  - `docs/marley_architecture/marley_workbench.md`: the crate's two lint allows and why.
  - `docs/marley_architecture/terminal_blocks.md`: the shim as `session`'s child, the reap
    signal's logging, and the stale "mutation gate" wording now reads as the end-of-sprint
    run.
  - Rows for the paths outside the owned set, checked against what shipped: `clippy.toml`
    (added in Code), `crates/zed/src/zed.rs` (the call passes a reference now; the row names
    the call, not its argument) and `Cargo.lock` ("the Marley crates and their dependencies"
    covers `log`).
  - The slice sits outside the prongs, so `three-prong-plan.md` has no row for it.
- **Knowledge appended.**
  - `F-claude-447-a-warning-check-read-quiet-cargo-output-001`
  - `PR-claude-a-new-gate-check-is-proven-red-by-a-planted-fault-001`
  - `L-claude-447-clippys-cargo-group-reads-the-whole-workspace-001`
  - `L-claude-447-semgreps-rust-parser-and-its-file-list-001`
  - `L-claude-447-gpui-code-under-rustals-lints-001`
  - `L-claude-447-smoke-a-gate-by-extracting-its-functions-001`
  - `AD-claude-447-the-marley-crates-carry-rustals-lint-table-001`
  - `AD-claude-447-a-gate-run-names-its-mode-and-revokes-the-old-receipt-001`
- **Brain.** Consultation `58ac2b40cfaf465eaf2ddcef2dc961a6` closed with
  `decisions/the-marley-crates-carry-rustals-lint-table-and-gate-checks`, follow-up by
  2026-10-20: check the gpui crate's two allows against W3 to W6, and pick up TICKET-448.
- **Tickets.**
  - #447 and #444 moved to `tickets/closed/`, both pointing at this archive.
  - #444's backlog row is gone.
  - TICKET-448 (Zed's dylint lints, the Out item) is minted and queued after W6.
