# gate:22: process spawns only in the listed adapter modules, held by a ratchet — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-541-spawn-in-adapters-ratchet.md
- **Pipeline spec:** 541-spawn-in-adapters-ratchet.spec.md

## Phase 1 — Plan
- **Request:** Chad asked on 2026-09-25 for every item decided that day to be specced; this is
  item 3 of report 07's list (`docs/orca_architecture/07-engineering-and-changelog.md` §3, with
  §A7), item 10 of the survey README. The brief for this draft: a gate step in `script/gates.sh`,
  or a semgrep rule for gate:20, that allows process spawns only in named adapter modules of the
  Marley crates, with Orca's five devices (an allowlist file, a failure on a stale entry, a pinned
  count, an anti-vacuity count, a planted violation the check must catch); find today's spawn
  sites.
- **Classification / tier:** chore, S to M. Gate tooling (`script/gates.sh`, two new files in
  `.config/`, the hook helper) and one small Rust move inside `marley_workbench`.
- **Recall (§18.3):**
  - L-claude-447-semgreps-rust-parser-and-its-file-list-001: semgrep's Rust parser trips on
    `&raw.payload` (rename the binding); semgrep reads only what git lists unless
    `--no-git-ignore`; `SEMGREP_ENABLE_VERSION_CHECK=0` and `--metrics=off` keep it off the
    network.
  - L-claude-447-smoke-a-gate-by-extracting-its-functions-001: smoke one gate step by extracting
    its function with awk and `eval`ing it with the variables it reads; plant faults in untracked
    files or backed-up tracked ones; compare `git diff | sha256sum` and the untracked list before
    and after. An untracked `.rs` no module declares also trips cargo-shear.
  - L-claude-443-run-the-gate-not-a-reconstruction-001: prove a gate change by running the step's
    own command and a real `script/gates.sh --diff`, never a command rebuilt by hand.
  - PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001: under `pipefail`, count with `grep -c`
    rather than end a pipeline in `grep -q`.
  - AD-claude-447-a-gate-run-names-its-mode-and-revokes-the-old-receipt-001: the receipt and its
    fingerprint, which the new files must join.
  - Brain: not consulted in this drafting pass, which was read-only; `/pipeline:plan` runs
    `brain_ask` at promotion.
- **Discovery (opened and checked):**
  - Spawn calls in `crates/marley_*/src` on 2026-09-25 (a search for `Command::new`,
    `new_command`, `new_std_command`, `tty::new`, `std::process::`, `smol::process` and
    `tokio::process`, each hit opened):
    - `crates/marley_terminal/src/pty_os.rs:54`, `tty::new` in `spawn` (38), the PTY shim, a child
      module of `session` through `#[path]` (`session.rs` 32 and 33);
    - `crates/marley_browser/src/service.rs` 133, `util::command::new_command("systemd-run")` in
      `start` (132), and 177, `new_command("systemctl")` in `unit_state` (176);
    - `crates/marley_workbench/src/marley_workbench.rs` 291, `util::command::new_command(program)`
      in `run_program` (286; #515 and #516 are editing this file, so the lines move), called from
      `claude_plugin.rs` 215 and 226 and `voice.rs` 93;
    - `crates/marley_workbench/src/voice.rs` 126, `util::command::new_command(voxtype)` in
      `read_status` (125), with `Stdio` from `util::command` (16) and `kill_on_drop(true)`;
    - test code only: `crates/marley_workbench/src/claude_plugin_tests.rs:85`;
    - not spawns: `routing.rs:11` (`std::process::ExitStatus`, a type), `marley_mcp/src/transport.rs`
      (threads and a TCP listener), `pty_os.rs` 115 to 137 (`rustix` signals to the child).
  - `crates/util/src/command.rs`: `new_command` (16), `Command` over `smol::process::Command` (28),
    `new_std_command` re-exported from `gpui_util` (14; `crates/gpui_util/src/lib.rs` 22 and 31).
  - `script/gates.sh`: the gate list and numbering (20 to 27), `run_gate` (84), `marley_dirs`
    (293), `SEMGREP_PIN` and `semgrep_g` (313, 314 to 324), the run order (341 to 355, gate:21
    last), the summary and the receipt after it.
  - `.claude/hooks/lib-hook-helpers.sh`: `gate_state_hash` (49 to 54, the two `git ls-files` lists
    and the `grep -zE` filter at 54), `marley_owned_path` (214).
  - `.semgrep.yml`: `process-exit-in-library`, `command-injection-risk`; its header says semgrep's
    built-in ignores skip `tests/`.
  - `clippy.toml`: `disallowed-methods` on `std::process::Command::{spawn, output, status, stdin,
    stdout, stderr}`.
  - `CONSTITUTION.md`: §0's gate table and its "Process spawning is permitted" paragraph ("Keep
    spawns in adapter modules and validate inputs"), §14's bullet naming `marley_terminal`, the
    harness and Rusty clients and `marley_mcp::transport`, §15's list of gate-defining files, and
    the amendment rule (a commit touching only CONSTITUTION and the gate that enforces the rule).
  - `crates/marley_*` hold 76 Rust files, 67 outside `*_tests.rs` and `tests/` (late on
    2026-09-25, with #516's untracked `marley_mcp/src/redact.rs`; the count grows with the tree).
  - semgrep 1.156.0 is installed (`semgrep --version`).
  - Orca: `src/shared/child-process/child-process-import-boundary.test.ts` and its
    `__fixtures__/child-process-import-allowlist.txt` (182 lines with its comments);
    `src/shared/agent-status-legacy-ingress-ratchet.test.ts` (`PLANTED_MUTATION_BYPASSES`, one case
    per form, then all together).
- **Decisions:** D1 to D7 in the spec.

### Design
- **The move (commit 1).** `crates/marley_workbench/src/process.rs`, declared in
  `marley_workbench.rs`, with a module doc saying it is the workbench's one module that starts
  processes (§14, gate:22):
  - `pub(crate) async fn run_program(program: &Path, args: &[&OsStr]) -> anyhow::Result<()>`,
    moved, with its behavior and its error text unchanged.
  - `pub(crate) fn follow(program: &Path, args: &[&str]) -> anyhow::Result<util::command::Child>`:
    stdin null, stdout piped, stderr null, `kill_on_drop(true)`, the spawn `read_status` makes
    today. `read_status` takes the child's stdout and reads its lines as before.
  - `pub(crate) async fn output(program, args, dir, env) -> anyhow::Result<String>`: stdout of a
    finished program, its stderr in the error. `run_program` becomes a thin call of it, so the
    module holds two spawn calls (`output` and `follow`) and the pin is 5: one in `pty_os.rs`, two
    in `service.rs`, two here. The `git` and `gh` calls planned by sibling tickets (#509's
    snapshots, #510's `git config`, #511's merge, #531's `gh pr list`, per their queued specs of
    2026-09-25) use it and add no spawn call.
  - `claude_plugin.rs` and `voice.rs` call `crate::process::…`.
- **The rule (commit 2), `.config/spawn-sites.yml`.** One rule, `marley-spawn-site`, `languages:
  [rust]`, `severity: INFO`, a message naming the list, `paths: exclude: ["*_tests.rs", "tests/"]`,
  and a `pattern-either` of the twelve call forms in the spec. P2 confirms on the pinned engine
  that the short forms match after their imports; the planted self-test is what proves it on
  every run.
- **The list, `.config/spawn-sites.txt`.** A header comment (what the list is, that a new line
  needs the pin moved in the same change, that removing a line is the ratchet's direction), then
  the three adapter files, each followed by `# ` and what it starts.
- **gate:22, `spawn_sites_g`, in `script/gates.sh`,** with `SPAWN_SITES_PIN=5` and
  `SPAWN_SCAN_FLOOR` set at P2 a little under the count semgrep reports (about 60 against 67
  files today):
  1. The pinned-version check `semgrep_g` makes, shared rather than repeated.
  2. `semgrep --config .config/spawn-sites.yml --json --strict --metrics=off --no-git-ignore` over
     `marley_dirs`, with `SEMGREP_ENABLE_VERSION_CHECK=0`; a non-zero exit fails the step.
  3. `jq` reads `path:line` for each result and `.paths.scanned | length`.
  4. Offenders: results whose path is not a listed line (each printed); stale: listed lines with no
     result (each printed, "delete the line"); the count against the pin (both numbers printed);
     the scanned count against the floor. Counts use `grep -c`, never a quiet grep mid-pipeline.
  5. The self-test: `mktemp -d`, then a heredoc writes `crates/marley_planted/src/planted.rs` with
     the `use` lines for the short forms, one call per planted line, each ending `// planted`, and
     two decoys, a comment line naming `std::process::Command::new` and a string holding
     `util::command::new_command(...)`, each ending `// decoy`. The same rule scans that folder;
     the lines it reports must equal the lines `grep -n '// planted$'` lists, with no decoy line.
     The folder is removed by a trap.
  6. The step's exit code is the verdict; the summary line reads `gate:22 spawn sites`.
- **`run_gate "gate:22 spawn sites (adapters only)" spawn_sites_g`** after gate:21, and gate:22 in
  the header's list of numbers.
- **`.claude/hooks/lib-hook-helpers.sh`:** both files in `gate_state_hash`'s two `git ls-files`
  lists and in its filter (`|^\.config/spawn-sites\.(txt|yml)$`); both in `marley_owned_path`.
- **Docs in commit 2.** CONSTITUTION §0 (the table line; the spawn paragraph names the list), §14
  (the bullet names the list and gate:22 in place of the old module names), §15 (both files among
  the gate-defining files); `docs/marley/zed-touchpoints.md`'s owned list.
- **File manifest.** Marley-owned only: `crates/marley_workbench/src/process.rs` (new),
  `marley_workbench.rs`, `voice.rs`, `claude_plugin.rs`; `.config/spawn-sites.yml` and
  `.config/spawn-sites.txt` (new, owned once `marley_owned_path` names them); `script/gates.sh`;
  `.claude/hooks/lib-hook-helpers.sh`; `CONSTITUTION.md`; `docs/marley/zed-touchpoints.md` (its
  owned list, not a row). No Zed path.
- **Ledger rows at Complete.** An AD for the ratchet's shape (the engine, the files, the pin
  outside the list, the planted file written by the step); a lesson if the pinned semgrep's
  handling of imported short forms or of `paths.scanned` turns out different from its docs.

### E2E plan
| REQ | How | Evidence |
|---|---|---|
| REQ-001 | `script/gates.sh --diff` after both commits' changes | the summary's `PASS  gate:22 ...`, `GATE GREEN [diff]` |
| REQ-002 | smoke: an untracked `crates/marley_workbench/src/planted_spawn.rs` with one `util::command::new_command("true")`, the step extracted and run, then the file removed | exit 1 and the file:line printed; then exit 0 |
| REQ-003 | smoke: a line naming `crates/marley_workbench/src/rail.rs` added to the list, then removed | exit 1, "delete the line"; then 0 |
| REQ-004 | smokes: `SPAWN_SITES_PIN` set to 6, then 4, in the extracted function's environment | exit 1 each, both numbers printed |
| REQ-005 | smoke: the step run with `MARLEY_PKGS` holding only `marley_dcs` | exit 1, the scanned count and the floor printed |
| REQ-006 | smokes: the rule copied to a scratch config without the `tty::new` pattern, then with a regex pattern that matches the decoy string, and the self-test run against each | exit 1 each, naming the missing planted line or the decoy line |
| REQ-007 | smoke: an untracked `.rs` under a Marley crate that does not parse | exit 1 from `--strict` |
| REQ-008 | `gate_state_hash` printed, the list edited, printed again, restored | two different fingerprints |
| REQ-009 | `just e2e script/e2e/480-voice-input.sh` after the move | its four shots as #480's Test read them |

Each smoke ends with the tree as it was: `git diff | sha256sum` and the untracked list compared
before and after (L-claude-447). No UI scenario is new: the change has nothing new to see.

### Risks
- semgrep's matching of an imported short form (`tty::new` after `use alacritty_terminal::tty`,
  `new_command` after its `use`) is the part the drafting could not run. The planted file tests
  each form on every gate run, so a form the engine misses fails the gate at P2 instead of hiding.
- `paths.scanned` in semgrep's JSON: if 1.156.0 leaves it out or counts differently, P2 takes the
  floor from what the pinned engine prints and records it in the step's comment.
- The pin moves with every new spawn call. That is the friction the ratchet wants; the `git` and
  `gh` calls the sibling tickets plan go through `process::output` and leave it alone. If #531
  lands first, its `new_command("gh")` in `github.rs` is a third stray this ticket moves.
- A rule that matches too widely (a `Command::new` of a type that is not a process, such as a CLI
  parser's) would fail the gate on a false hit. No Marley crate has one today; the message tells
  the reader to narrow the rule rather than list the file.
