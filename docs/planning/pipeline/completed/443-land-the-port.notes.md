# Land the ported Marley tree behind a green gate — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-443-land-the-port.md
- **Pipeline spec:** 443-land-the-port.spec.md

## Phase 1 — Plan
- **Request:** Chad's goal (2026-09-22): "lets create the tickets and make it happen." No ticket
  of the workbench-shell sprint can commit until the uncommitted port lands; the first FULL run
  showed what stands between the tree and green.
- **Classification / tier:** chore; one Marley crate (`marley_mcp`, full bar), the gate script,
  and archived docs. No Zed crate.
- **Evidence from the first FULL run** (`script/gates.sh`, started 2026-09-22 15:14):
  static gates and coverage green (4,934 lines, 0 missed); mutation found 45 survivors by
  mutant 362 of 607, 44 in `transport.rs` and one in `tools.rs:71`. The `tools.rs:71` log
  (`mutants.out/log/crates__marley_mcp__src__tools.rs_line_71_col_5.log`) builds its own mutant
  and then runs nextest on a binary that warns about `transport.rs:53` (`unused variable: f`),
  the other worker's Debug mutant: both copies share `CARGO_TARGET_DIR`, and cargo gives
  `marley_mcp` one artifact hash (`marley_mcp-51cde4ca6951f9f4`) in both.
- **Recall (§18.3):** no ledger entry on the shared-target race. The gpui-era FULL ran two jobs
  on a shared target too (`lessons.md:191` mentions "the --jobs 2 fix"), so earlier FULL
  verdicts may carry the same noise. `prevention-rules.md:1696` (fixture builds inherit
  `CARGO_TARGET_DIR` in copies) does not apply: no ported Marley crate builds a fixture
  (checked: no `env!("CARGO")`, `CARGO_BIN_EXE` or `current_exe` in `crates/marley_*`).
  `BF-claude-guard-phrasing-creates-equivalent-mutant` and
  `BF-claude-dead-defensive-clamp-equivalent-mutant-001` apply to the `content_length > 0`
  guard (an equivalent mutant; remove the guard instead of testing around it). Brain
  consultation `6717f1a1122d41cb9e415a569eef71d1`: nothing on this seam.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit on
  `marley/workbench-shell` (2026-09-22). This harness has no `TaskCreate`/`TaskUpdate`, so each
  phase's checklist is kept here and ticked as it closes.
- **Phase 1 checklist:** parse request ✓ · intake (none) ✓ · classify ✓ · prior-art sweep ✓ ·
  ticket doc ✓ · spec ✓ · doc pair ✓ · present (autonomous) ✓.

## Phase 2 — Design
- **Approach.** Three independent fixes, then one gated commit.
  1. *Transport.* A `#[cfg(test)] mod tests` at the foot of `transport.rs` starts the real
     server (`spawn` with a fresh `Shared` and an mpsc effect channel) and talks raw HTTP/1.1
     over `std::net::TcpStream` with read timeouts, so every assertion is on the bytes a client
     receives. The tests live in-file because the private helpers (`now_epoch_ms`,
     `read_entropy`, `header_value`, `status_reason`) need direct unit tests and because gate:4
     counts in-lib tests only (`PR-claude-integration-only-coverage-fails-gate4-001`). The
     `if content_length > 0` guard goes: `read_exact` into an empty buffer is already a no-op,
     so `>=` on that guard is an equivalent mutant no test could kill.
  2. *Gate.* `mutation_g` in FULL mode runs `cargo mutants` under `env -u CARGO_TARGET_DIR`, so
     each copy builds in `<copy>/target`. Copy mode already isolates the sources; the shared
     target was the one shared resource, and cargo names a workspace path package's artifact by
     a hash that does not include the checkout path, so the two copies wrote the same file.
     DIFF mode keeps the shared warm target (it is in place with one job, no second writer).
     The FULL comment that said the shared target "serializes the builds" is corrected: it
     serialized compilation, not the build-then-test window a mutant needs.
  3. *Archive.* Done before the first run (the notes say the sections predate the rule).
  4. *Commit.* After Phase 5, `git add -A` stages the port, the plan, the queued sprint and this
     ticket; `script/gates.sh --diff` writes the receipt first. `mutants.out` and `.mcp.json`
     stay ignored.
- **§20:** N/A still holds; the transport's behavior is unchanged, only proven.
- **File manifest.**
  - `crates/marley_mcp/src/transport.rs` (Marley crate, full bar): the test module, the guard
    removal, the module doc no longer calling the file masked.
  - `script/gates.sh` (gate tooling): FULL mutation isolation plus its comments.
  - `docs/planning/pipeline/completed/*.spec.md` (docs): the §20 notes, already applied.
  - `docs/planning/knowledge/failures.md`, `prevention-rules.md` (ledger): appended at inspect.
  - `CHANGELOG.md`: the entry at complete.
- **Regression test plan.**

  | Test (transport.rs `tests`) | Kills (first FULL's survivors) | REQ |
  |---|---|---|
  | `the_handle_names_a_loopback_url_and_redacts_its_bearer` | Debug `fmt`, `url` → ""/"xyzzy", `bearer` → ""/"xyzzy" | 010 |
  | `the_clock_and_the_entropy_are_the_real_ones` | `now_epoch_ms` → 0/1, `read_entropy` → None/zeros/ones | 010 |
  | `initialize_assigns_and_echoes_a_session` | `accept_loop`, `serve_connection` → Ok, the initialize `==`, `write_sse_response` → Ok, `read_http_request` → None/default, the header-loop `==`, `header_value` → None/"xyzzy"/"" | 003 |
  | `a_missing_or_wrong_bearer_or_a_foreign_origin_is_forbidden` | both `delete !`, `\|\|` → `&&`, `write_status` → Ok | 002 |
  | `a_request_without_a_session_is_bad_and_an_unknown_one_is_not_found` | `status_reason` arms 400/404 and → ""/"xyzzy" | 004 |
  | `a_live_session_gets_exactly_one_answer_and_no_new_session` | `delete !` on `wrote_response` | 004 |
  | `a_notification_only_post_is_accepted` | the 202 arm | 005 |
  | `delete_ends_the_session` | the Terminate arm | 006 |
  | `a_session_past_the_cap_is_refused` | the 503 arm | 007 |
  | `a_granted_surface_call_hands_the_focus_effect_to_the_app` | the effect send | 002 (dispatch path) |
  | `a_body_over_the_cap_is_refused_before_it_is_read`, `a_body_of_exactly_the_cap_is_read_whole` | `>` → `==`/`>=`/`<` on the cap, `<<` → `>>` | 008 |
  | `a_connection_that_sends_nothing_is_a_bad_request` | the empty request line `==` | 008 |
  | `the_stream_pushes_one_notification_per_change` | `serve_sse_stream` → Ok, `==` → `!=` in `wait_while`, `signal_change` → (), the POST/GET `==` | 009 |
  | `a_stream_the_client_hangs_up_frees_its_session` | `reap_session` → () | 009 |
  | `signal_change_bumps_the_version`, `header_value_matches_the_name_case_insensitively_and_trims`, `status_reason_names_the_gate_refusals`, `discovery_json_carries_the_url_and_the_bearer_header` | direct unit rows for the helpers and `discovery_json` → ""/"xyzzy" | 010 |
  | the directory check of the hook's logic over `completed/` | REQ-001 | 001 |
  | a scoped `cargo mutants` over `transport.rs` + `tools.rs` with isolated targets, `--jobs 2` | REQ-011 (no cross-worker binaries; `tools.rs:71` caught) | 011 |
  | `script/gates.sh --diff` | REQ-012 | 012 |

  Uncoverable: the IO-error arms of the transport (a loopback peer cannot make `local_addr` or
  a thread spawn fail), which is why the file stays on gate:4's documented exclude list.
- **Risks.** Loopback timing in the stream tests (bounded waits: 300 ms quiet windows, a 5 s
  poll for the hang-up reap). Each test leaks its server thread until the test process exits,
  which nextest's process-per-test model cleans up. An interrupted in-place DIFF run can leave a
  mutant in the tree; `git diff` before staging.
- **Phase 2 checklist:** approach ✓ · manifest ✓ · test plan ✓ · risks ✓ · present
  (autonomous) ✓.

## Phase 3 — Implement
- **Built.**
  - `crates/marley_mcp/src/transport.rs`: 20 loopback tests at the foot of the file (the
    Phase 2 table), the `content_length > 0` guard removed, and the module doc no longer calling
    the file masked.
  - `script/gates.sh`: FULL mutation runs `cargo mutants` under `env -u CARGO_TARGET_DIR`, so
    each copy builds in its own target; the mutation_g comments say why. shellcheck clean.
  - `crates/marley_terminal/tests/integration.rs`: five tests for the PTY shim's survivors the
    completed FULL run added (`spawn_runs_the_given_program_and_arguments`,
    `spawn_starts_in_the_given_directory`, `resize_reaches_the_program`,
    `teardown_kills_a_child_that_ignores_hangup`, `teardown_closes_the_pty_file_descriptors`),
    plus the `spawn_program`, `screen_text` and `pump_until_screen_shows` helpers.
- **The completed FULL run** (15:14 to 16:20): 13 of 14 gates green; mutation 517 caught / 53
  missed (MSI 90.7%), 14 timeouts counted caught. Survivors: 44 in `transport.rs`; `tools.rs:71`
  (race artifact, see Phase 1); `session.rs:230` and `:241` (`-> Default::default()`, which
  cannot compile because `TerminalSession` has no `Default`: race artifacts);
  `pty_os.rs:69` (`write -> Ok(1)`, which `child_exit_reports_exact_code` kills: an artifact);
  and five genuine `pty_os.rs` gaps: `delete field shell`, `delete field working_directory`,
  `set_winsize -> Ok(())`, `Drop -> ()`, `delete !` in `Drop`. The last three were the
  gpui-era masks' territory; each now has a test that fails under the mutant.
- **Deviations.**
  - The transport tests were drafted during Phase 1's investigation of the red run, before
    this phase formally opened. They are this ticket's product, so they are recorded here rather
    than deferred to Phase 4, which runs them.
  - The `marley_terminal` tests widen the scope named in Phase 2 (the PTY shim was not in the
    first survivor list at mutant 362). Same kind of fix, same crate family.
- **Checks.** `cargo check -p marley_mcp --tests` and `cargo check -p marley_terminal --tests`
  clean; `cargo fmt --all --check` clean.
- **Trap met.** The first `.rs` edit started this session's rust-analyzer plugin, whose
  whole-workspace `cargo check --all-targets` held the shared target's lock for ~10 minutes and
  stalled the running FULL gate (recorded in memory; a lesson at Phase 5).
- **Phase 3 checklist:** transport tests ✓ · guard removal ✓ · gate isolation ✓ · PTY shim
  tests ✓ · checks ✓.

## Inspect (Phase 3.5)
Two critics, run in parallel over the Phase 3 diff with the ledger's gate-integrity and
real-PTY entries as input: (1) test correctness and flakiness across the transport and PTY
tests, and whether each listed mutant has a test that fails under it; (2) the gate change, the
hooks and the archive notes. Both verified by running things (a fake cargo against the real
`mutation_g`, the real reference hook over a staged copy of all 409 specs, `ps` and
`/proc/<pid>/environ` on the box). Every finding below was re-checked by me before a verdict.

| # | Finding (critic) | Verdict | Fix |
|---|---|---|---|
| 1 | critical: DIFF mutation mutates no Marley crate. The root inherits Zed's `default-members = ["crates/zed"]` and cargo-mutants mutates only default members without `-p`, so the diff matched nothing, cargo-mutants said "No mutants to filter" and the gate printed a pass (2) | real: `--list --in-diff` gives 0 mutants without `-p`, 603 with it | `script/gates.sh` DIFF passes `-p` for every touched package and fails closed when the diff has crate lines but no package resolves |
| 2 | high: the reference hook false-blocks about half the time: `printf "$BLOB" \| grep -q` under pipefail, grep exits early, printf takes SIGPIPE (2) | real: 9 of 20 hook runs blocked a valid spec. A recurrence of `PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001` | here-strings in `enforce-warp-reference.sh` |
| 3 | medium: the same race makes `... \| grep -qE ... \|\| exit 0` allow: the commit gate let a 64 KB commit command through with no receipt 46 of 100 times (2) | real | here-strings in `enforce-commit-gate.sh` and `enforce-changelog.sh` too (11 sites in three hooks); shellcheck clean |
| 4 | high: the SIGHUP test's child outlives a test process that dies first: alacritty's `setsid` puts it outside nextest's process group, it ignores HUP and loops forever (1) | real: three such shells were alive, reparented to `systemd --user`, with `CARGO_MANIFEST_DIR` inside the isolated mutation run's copies | the three killed; the child is now `trap '' HUP; echo trap-set; exec sleep 30`, so it ends on its own and the ignored HUP survives the `exec` |
| 5 | medium: that test waited a fixed 300 ms for the trap; a HUP that beats it kills the shell, the SIGKILL path never runs, and `Drop -> ()` would pass (1) | real (a false pass is the bad direction for a mutation gate) | the test waits up to 5 s for `trap-set` on screen before hanging up |
| 6 | medium: the SSE tests gave data they expect only the 300 ms quiet window; separately `serve_sse_stream` wrote the head before reading the version, so a change in that gap was never pushed (1) | real, both | tests: `read_message` waits up to `REPLY_TIMEOUT` for a whole message, and `QUIET_WINDOW` is left to the "nothing more" checks. Server: the version is read before the head goes out, so the push is over-sent, never lost |
| 7 | medium: the shared target may still hold mutant builds from the pre-fix FULL runs (2) | real, already handled: `cargo clean -p` of the five Marley crates in Phase 3, and every mutation run since was isolated | the F entry records that pre-fix FULL verdicts were wrong in both directions |
| 8 | medium: spec #435 got a false "Not recorded" prior-art note under its real `### Prior art (three legs)` (2) | real: the annotation script's exact heading match missed the suffixed heading | the note deleted; the file is byte-identical to the gpui-era original |
| 9 | low: inputs nothing controls: `.cargo/mutants.toml`, `CARGO_MUTANTS_OUTPUT`, `CARGO_MUTANTS_MINIMUM_TEST_TIMEOUT`, `CARGO_BUILD_TARGET_DIR`, `build.build-dir`; the receipt fingerprint omits `.cargo/config.toml` and `.cargo/mutants.toml`; gate 12 does not look for `mutants::skip` (2) | real, all parts. I first meant to reject the gate 12 part because a skip needs the `mutants` crate; cargo-mutants' `attr_is_mutants_skip` (visit.rs:921) ignores the `cfg_attr` condition, so `#[cfg_attr(any(), mutants::skip)]` compiles without it and still masks | `--no-config`; the four variables unset and `TMPDIR` set to the scratch dir; both files in `gate_state_hash`; gate 12 bans `mutants::skip` in any form (checked against a masked file, an added line with spaces around `::`, and the real tree) |
| 10 | low: `delete !` in `if !reaped` is killed only on Linux (the fd test read `/proc/self/fd`) (1) | real | the fd test counts `/dev/fd`, which lists the process's descriptors on Linux and macOS |
| 11 | low: the fd test's warm-up comment claimed a descriptor the first spawn keeps; each `Pty` closes its own SIGCHLD pair (1) | real | comment reworded: the warm-up is a precaution against one-time process state |
| 12 | low: `read_http_request -> Ok(Some(Default::default()))` cannot compile (`HttpRequest` has no `Default`), so the Phase 2 table's credit to `initialize_assigns_and_echoes_a_session` is wrong; `write -> Ok(1)` is an artifact (1) | real: a race artifact, unviable, the same class as `session.rs:230/241` | recorded here; nothing to change in code |
| 13 | nit: the archive notes say "added on 2026-09-22", which holds only if the commit lands that day (2) | rejected: the date is the annotation's, which is fixed; rewording 289 files for a possible one-day slip buys nothing | none |
| 14 | my review: eight `let _ =` on fallible calls in the ported crates (`transport.rs:117/120/202`, `apply.rs:198/205/226`, `session.rs:562`, `pty_os.rs:118`), against Zed's `.rules` | real, out of scope: ported as they stood, none reached by this ticket's diff, and each fix adds a branch that needs a test | minted TICKET-444 (Queue, after #442) |

- **Checked and fine (both critics).** The guard removal (`read_exact` into an empty buffer
  returns at once); SIGPIPE on the server side (std sends with `MSG_NOSIGNAL`); `TerminalSession`
  is `Send`; `pwd -P` against `canonicalize`; `stty size` order; the env test's needle cannot
  come from an echoed line; the merged `spawn`; GNU `env -u` ordering and exit codes; exit codes
  0/2/3/4 reach the verdict; no other target sharing on the box (`CARGO_TARGET_DIR` comes only
  from `~/.bashrc`); every archive note sits where it should, and 408 specs diff clean against
  `/srv/stacks/marley` apart from the notes.
- **After the fixes.** `cargo nextest run -p marley_mcp -p marley_terminal`: 231 passed, the
  SIGHUP test in 2.04 s (the first deadline, then SIGKILL), and no test child left running.
  `cargo fmt --all --check` clean; shellcheck as gate 11 runs it, clean.
- **Box note.** Another session's cargo-mutants run (rustal, 826 mutants, copies under
  `/mnt/buildtmp` with their own targets) was live throughout. It does not use
  `/mnt/fast/target`, so this session's cargo runs did not share a target with it; it was left
  alone.
- **Phase 3.5 checklist:** spawn critics ✓ · review findings ✓ · fix confirmed ✓ · verify fixes ✓
  · write inspect ledger ✓.

## Phase 4 — Validate
- **Tests.** The design's rows were written in Phase 3 and hardened at inspect; none added
  here. `cargo nextest run -p marley_mcp -p marley_terminal` after the last source edit: 231
  passed (the SIGHUP test in 2.04 s). The gate's own test step: 271 passed across the five
  Marley crates, doctests included.
- **REQ-001.** The fixed reference hook, run 20 times over all 409 specs staged in a throwaway
  index (`scratchpad/req001/run-reference-hook.sh`; the real index untouched): 0 blocks.
- **First `script/gates.sh --diff` (16:21): red at gate:5, exit 1.** Reproduced with the exact
  argument vector plus `--list`: cargo-mutants 27.1.0 refuses `--jobs` beside `--in-place`.
  The inspect critic's `--list` proof had left both flags out, so it could not see this. Fixed
  in `script/gates.sh` (DIFF drops `--jobs`; in place is one job by construction); the listing
  then gave 603 mutants, exit 0. Recorded as `F-claude-443-g-diff-mutation-was-a-usage-error-001`.
- **Second `script/gates.sh --diff` (16:24:37 to 17:00:58): GATE GREEN [diff], 14 of 14.**
  Coverage: 4,934 lines, 0 missed (100%), `transport.rs` and `pty_os.rs` on the documented
  exclude list for their IO-error arms. Mutation over all five Marley crates in place: 555
  caught / 0 missed, MSI 100.0%, with 48 unviable. The 12 timeouts are hangs the mutants cause:
  the index and budget arithmetic in `dcs.rs` (`c_unescape`, `split_unescaped`,
  `find_unescaped`) and `session.rs` (`write_bytes`, `pump`), and `read -> Ok(1)`, which spins
  `pump`. The gate counts them as caught; 4 of the logs also show failing assertions.
- **Tree integrity.** The crates are untracked, so `git diff` could not show a leftover mutant:
  all five were diffed against a pre-gate snapshot and are identical. No test child or test
  binary was left running. The receipt (`5fe4e6b0…2446`) equals `gate_state_hash` recomputed
  after the run.
- **Live drive.** N/A: no UI surface (a library transport, a PTY shim's tests, gate tooling,
  hooks, archived docs); the spec's `## UI proof` is N/A.
- **Pre-existing failures.** None.
- **Phase 4 checklist:** tests (the Phase 2 table, all rows) ✓ · REQ-001 check ✓ · run gate ✓ ·
  tree integrity ✓.

## Phase 5 — Complete
- **Documentation (§21).** `CHANGELOG.md` gains a `### Fixed` section with three entries: the
  gate over the ported tree, the MCP transport's event stream, and the archive notes. The
  architecture record now matches what shipped. `docs/marley_architecture/terminal_blocks.md`
  has a fork note: the shim is unmasked, and six real-PTY tests kill its mutants.
  `orchestration-shell.md` and `crate-map.md` mark their "masked" cells as gpui-era.
  `docs/marley/three-prong-plan.md`'s Gates bullet said the old coverage and mutation gates
  were not carried over; it now says they came over with the port and hold the Marley crates to
  100%. `docs/marley/workbench-shell.md`'s Slices section gains a status line (the baseline
  landed; W0 to W6 are #436 to #442). No Zed crate changed, so the ledger
  (`docs/marley/zed-touchpoints.md`) is unchanged.
- **Knowledge (§19).**
  - Inspect appended `F-claude-443-c-diff-mutation-mutated-no-marley-crate-001`,
    `F-claude-443-d-pipefail-hooks-raced-sigpipe-again-001`,
    `F-claude-443-e-a-hup-ignoring-test-child-outlived-its-test-001`,
    `F-claude-443-f-the-sse-stream-read-its-version-after-the-head-001`,
    `PR-claude-a-workspace-mutation-run-names-its-packages-001` and
    `PR-claude-a-test-child-that-ignores-signals-ends-on-its-own-001`, beside Phase 3's
    `F-claude-443-a-…`, `F-claude-443-b-…` and their two PR rules.
  - Validate appended `F-claude-443-g-diff-mutation-was-a-usage-error-001`.
  - This phase appended `L-claude-443-the-editor-plugin-shares-the-target-001`,
    `L-claude-443-run-the-gate-not-a-reconstruction-001` and
    `AD-claude-443-mutation-topology-and-no-masks-001`.
- **Brain.** Consultation `6717f1a1122d41cb9e415a569eef71d1` closed with
  `brain decide`: `decisions/marley-fork-mutation-gate-diff-in-place-over-named-packages-full-copies-in-their-own-targets-no-masks`,
  follow-up due 2026-10-22 (does FULL's per-copy target cost too much time or disk?).
- **Follow-on work.** TICKET-444 (Queue, after #442): the eight `let _ =` discards in the
  ported crates.
- **Ticket.** TICKET-443 moved to `tickets/closed/`; it never had a `BACKLOG.md` row (minted
  fresh), and none is stale.
- **Phase 5 checklist:** CHANGELOG + architecture docs ✓ · capture knowledge ✓ · close ticket ✓
  · archive pipeline ✓.
