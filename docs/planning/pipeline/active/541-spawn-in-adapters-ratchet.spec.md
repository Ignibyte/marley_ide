---
pipeline_id: 9c54373e-8167-4219-95fe-bdf7562af598
ticket: docs/planning/tickets/open/TICKET-541-spawn-in-adapters-ratchet.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "gate:22: process spawns only in the listed adapter modules, held by a ratchet"
type: chore
slice: cross-cutting (the gate; CONSTITUTION §0 and §14), from the Orca survey's report 07
references: [docs/planning/pipeline/completed/447-rustal-quality-gates.spec.md, docs/planning/pipeline/completed/448-zed-dylint-lints.spec.md, docs/orca_architecture/07-engineering-and-changelog.md]
---

## Title
`script/gates.sh` gains gate:22, which lets a Marley crate start a child process only in the
adapter modules that `.config/spawn-sites.txt` names. A semgrep rule finds the spawn calls, so
comments and strings never count, and the step holds the list with the five devices Orca's
ratchet tests use. The two spawn calls that sit outside an adapter today move into one first.

## Scope
### In
- **The move.** `run_program` (in `crates/marley_workbench/src/marley_workbench.rs`, at line 286
  on 2026-09-25; #515 and #516 are editing that file, so the line moves) and the spawn in
  `voice::read_status` (`crates/marley_workbench/src/voice.rs` 125 to 133) move into
  `crates/marley_workbench/src/process.rs`, the workbench's one module that starts processes. They
  run the same programs as before: `claude plugin` commands (#482), `voxtype record toggle` and
  `voxtype status --follow` (#480). `run_program`'s callers (`claude_plugin.rs` 215 and 226,
  `voice.rs` 93) call it there, and `read_status` keeps reading the follower's lines. A spawn
  call that a sibling ticket adds outside an adapter before this one lands (#509, #510, #511 and
  #531 plan `git` or `gh` calls) moves the same way; P2 recounts the sites.
- **The rule.** `.config/spawn-sites.yml`, one semgrep rule, `marley-spawn-site`, matching each way
  the Marley crates can start a process: `std::process::Command::new`,
  `smol::process::Command::new`, `tokio::process::Command::new`, `util::command::new_command`,
  `util::command::new_std_command`, `util::command::Command::new`, `gpui_util::new_std_command`
  and `alacritty_terminal::tty::new`, and the short forms an import leaves (`Command::new`,
  `new_command`, `new_std_command`, `tty::new`). Test files (`*_tests.rs`, `tests/`) are left out,
  as Orca leaves tests out.
- **The list.** `.config/spawn-sites.txt`: one adapter file per line, each with a comment saying
  what it starts. At the end of the move: `crates/marley_terminal/src/pty_os.rs` (the PTY),
  `crates/marley_browser/src/service.rs` (Chromium's user unit) and
  `crates/marley_workbench/src/process.rs` (the workbench's programs).
- **gate:22 in `script/gates.sh`**, after gate:21, on semgrep's pinned 1.156.0:
  1. The scan: the rule over the Marley crates with `--json --strict --metrics=off
     --no-git-ignore`; semgrep must exit 0.
  2. A finding in a file the list does not name fails the gate, printing the file and line and the
     modules to move the spawn into.
  3. A listed file with no finding, or no longer there, fails the gate with "delete the line".
  4. The number of findings must equal `SPAWN_SITES_PIN`, a literal in `gates.sh`, in both
     directions: above it, a new spawn needs a reviewed change of the pin; below it, the pin must
     come down so the ground taken is kept.
  5. semgrep must report at least `SPAWN_SCAN_FLOOR` scanned files, so a broken target list
     cannot pass empty.
  6. The planted self-test: the step writes a file holding each spawn form on its own line, plus a
     comment and a string that name the forms, into a scratch folder, scans it with the same rule,
     and requires exactly the planted lines to be found.
- **The bookkeeping.** Both new files join the receipt's fingerprint (`gate_state_hash`) and the
  Marley-owned set (`marley_owned_path` and the ledger's list of it). CONSTITUTION §0 gains
  gate:22's line in its table and names the list in its spawn paragraph; §14's spawn bullet names
  the list; §15's list of gate-defining files names both files.

### Out (explicitly deferred)
- Sockets, which §14 keeps in adapter modules too: the same step with a second rule, later.
- Spawns inside Zed's crates, and spawns Marley asks Zed's crates for (a terminal from
  `Project::create_terminal_shell`, a task, gpui's `open_url`): there Zed's crates are the
  adapters.
- Report 07's other candidate rules: durable writes only, no gpui in the core of `marley_mcp` and
  `marley_browser`, every per-terminal map cleared on close.
- A dylint lint in place of the semgrep rule.

## Reference (§20)
N/A for Warp and upstream Zed: a Marley gate, with no reference app behavior. Orca (MIT, read at
`1c2cf120e3`) for the devices: `src/shared/child-process/child-process-import-boundary.test.ts`
keeps its allowlist as data (`__fixtures__/child-process-import-allowlist.txt`, which says it only
shrinks), fails on a stale entry ("delete the line"), pins the real count as a literal (152) in
both directions because a bound by the list's own length lets a swap through, requires more than
500 scanned files so a broken root cannot pass, ignores comment lines, and exempts test files.
`src/shared/agent-status-legacy-ingress-ratchet.test.ts` plants each forbidden form in its own
case and all of them together, so no form hides another.

### Prior art
- **Behavior maps.** `docs/orca_architecture/07-engineering-and-changelog.md` §A7 (the six shapes
  of architecture tests; "Marley today: has the shape, not the devices"; "§14's 'keep spawns in
  adapter modules' is not checked by anything"), §A12, and §3 item 3 (rule 1: spawn only in the
  adapters; "grep over Rust is brittle, so semgrep or a dylint lint is the better engine").
- **Published material.** semgrep's JSON output (`results` with `path` and `start.line`,
  `paths.scanned`, `errors`) and `--strict`, on the version gate:20 pins.
- **Code we already ship.**
  - gate:20 (`semgrep_g`, `script/gates.sh` 313 to 324) runs `.semgrep.yml` with `--error`, which
    fails on any finding, so a rule that finds every allowed spawn cannot live there; it gets its
    own config and step. `.semgrep.yml`'s `command-injection-risk` checks what a
    `std::process::Command` is built from, and `clippy.toml`'s `disallowed-methods` bans the
    blocking `std::process::Command` calls in favor of `smol`'s; neither says where a spawn may
    sit. gates 12 and 13 (`no_suppr_g`, `source_bans_g`) are grep meta-gates with no list, pin,
    floor or self-test.
  - The receipt's fingerprint (`gate_state_hash`, `.claude/hooks/lib-hook-helpers.sh` 49 to 54)
    lists its paths and keeps only `^crates/marley_|\.(rs|sh|toml|lock)$|^\.semgrep\.yml$`, so a
    new `.txt` and `.yml` need both the list and the filter. `marley_owned_path` (214) is the owned
    set that gate:16 and the write hook share.
  - Today's spawn sites, from a search of `crates/marley_*` for `Command::new`, `new_command`,
    `new_std_command`, `std::process`, `smol::process` and `tty::new`, each opened:
    `crates/marley_terminal/src/pty_os.rs:54` (`tty::new`), `crates/marley_browser/src/service.rs`
    133 (`systemd-run`) and 177 (`systemctl`), `crates/marley_workbench/src/marley_workbench.rs:291`
    (`run_program`), `crates/marley_workbench/src/voice.rs:126` (`voxtype status --follow`). A test
    spawn sits at `crates/marley_workbench/src/claude_plugin_tests.rs:85`. `marley_mcp::transport`,
    which §14 names as an adapter, starts threads and binds a socket but starts no process.

## UI proof
N/A — no UI delta: a gate step, its rule and its list, and two spawn calls moved into an adapter
module without a change in what they run. The e2e run is #480's scenario, which drives both moved
calls (the microphone's `voxtype record toggle` and the status follower), in place of `just shot`
alone. The gate is proven by its exit codes and negative smokes (§7): each device made to fail
once, then restored.

## Locked-In Decisions
- D1: semgrep, not grep. The rule parses Rust, so a comment or a string naming `Command::new` is
  no spawn, and the short forms left by imports are caught. It is pinned already (gate:20).
- D2: The rule and the list live in `.config/` (`spawn-sites.yml`, `spawn-sites.txt`), apart from
  `.semgrep.yml`, which gate:20 runs with `--error`.
- D3: The pin and the floor are literals in `script/gates.sh`, a different file from the list, so
  a new spawn changes the gate script, which the receipt binds.
- D4: The step writes the planted file from a heredoc in `gates.sh` on every run, so the
  self-test cannot drift from the gate it tests.
- D5: The two stray spawns move into `crates/marley_workbench/src/process.rs` instead of joining
  the list where they are: the list names adapter modules only. `process.rs` also offers an
  output helper (stdout of a finished program, stderr in the error), so the `git` and `gh` calls
  that #509, #510, #511 and #531 plan from the workbench add no spawn call and leave the pin
  alone. A spawn that fits no helper adds its call to an adapter and moves the pin in the same
  change.
- D6: gate:22 is a new number; retired numbers are never reused. semgrep must exit 0 on both scans,
  and the step's own exit code carries the comparison of semgrep's JSON with the list; no
  human-readable output is grepped (§0).
- D7: Two commits, as the amendment rule of CONSTITUTION wants: first the move of the two spawns
  (Rust, with its CHANGELOG entry); then the gate step, its two files, the hook helper, the ledger's
  owned list and the CONSTITUTION lines together.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `script/gates.sh` runs on the tree after the move, gate:22 shall pass. | The gate's summary line for gate:22; `script/gates.sh --diff` green |
| REQ-002 | WHEN a Marley crate starts a process in a file the list does not name, gate:22 shall fail and print that file and line. | Negative smoke: an untracked copy of a module with a spawn added |
| REQ-003 | WHEN a listed file starts no process, or is gone, gate:22 shall fail and name the line to delete. | Negative smoke: a line naming a file without a spawn |
| REQ-004 | WHEN the number of spawn calls differs from the pin, in either direction, gate:22 shall fail and print both numbers. | Negative smokes: the pin one above and one below the count |
| REQ-005 | WHEN semgrep reports fewer scanned files than the floor, gate:22 shall fail. | Negative smoke: the step run with one small crate as its whole target |
| REQ-006 | WHEN the rule misses a planted form, or matches the planted comment or string, gate:22 shall fail and name that line. | Negative smokes: one pattern removed from the rule; a pattern that matches the decoy string |
| REQ-007 | WHEN semgrep cannot parse a scanned file, gate:22 shall fail. | Negative smoke: an untracked file that does not parse, under `--strict` |
| REQ-008 | WHEN the list or the rule changes after a green `--diff` run, the receipt shall no longer match the tree. | `gate_state_hash` printed before and after an edit to each file |
| REQ-009 | WHEN Voice input and the Claude Code plugin's install run after the move, they shall start the same programs as before. | #480's scenario, its shots as its own Test recorded them; review of the moved `claude plugin` calls |

## Phase Plan
- **P1 Plan:** promote the pair, recall, consult the brain, confirm the design in the notes.
- **P2 Code:** the move into `process.rs` (fmt and clippy clean, a review of the diff); then the
  rule, the list, gate:22, the fingerprint and owned-set lines, and the CONSTITUTION and ledger
  lines; shellcheck clean on `gates.sh`.
- **P3 Test:** #480's scenario and its shots; each negative smoke, extracted and run as
  L-claude-447-smoke-a-gate-by-extracting-its-functions-001 describes, then restored; a real
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG (the move and gate:22); CONSTITUTION §0, §14 and §15; the ledger;
  close, archive, the two commits.
