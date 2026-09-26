# Marley's own regression suite — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-517-regression-suite.md
- **Pipeline spec:** 517-regression-suite.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, on the Orca question 6 as explained to him (the app's own
  Playwright tests belong in its repository; Marley's own regression suite re-runs old scenarios
  and a golden set before every install, each run checking itself through Marley's MCP server):
  "ok yes ill defer to you on that".
- **Classification / tier:** chore; scripts only (`script/e2e.sh`, a new `script/regress`,
  `script/install-marley`, the `justfile`, checks added to golden scenarios); no Rust, so the
  gate is `--fast`.
- **Recall (§18.3):** the self-checking scenarios (#502, #512, #514, #516) show the pattern: a
  check helper that prints `check <name>: pass|FAIL` and returns 1, which `set -e` in the runner
  turns into a red run. L-claude-516's two lessons (read blocks before a second window opens
  under sway; fakes built at run time). TICKET-544: a rewrap scrambles blocks, so a golden check
  on blocks runs before any window opens. Brain: asked at promotion.
- **Discovery:** `script/e2e.sh` 60 to 61 (`MARLEY_BIN`, `binary`), 492 (the binary check), 552
  (the binary chosen); `script/e2e/browser-fixture.sh` (`mcp_agent`, its `terminal-read` since
  #516); `script/install-marley` 63 to 71 (the build, the staged copy, the rename); the `justfile`
  (`e2e`, `install`, `idle`). Scenarios with no checks today: 481, 484, 491, 492, 500, 515.

### Design
- **`E2E_BINARY`.** `script/e2e.sh` takes `MARLEY_BIN=${E2E_BINARY:-}` before the scenario is
  sourced, so a scenario's own `binary` still wins.
- **Checks per golden scenario** (each a `check_<what>` function at the end of `steps`, printing
  `check <what>: pass` or `FAIL <what>: <expected>, got <got>` and returning 1):
  - 491: `mcp_agent terminal-read 'seq 3'` answers `1 2 3`; `terminal_blocks` lists the four
    commands with exit codes 0, 1, 0, 0.
  - 484: the scenario's typed prefix and the suggested rest land as one command in
    `terminal_blocks`.
  - 481: the rich input's sent text is the block's command in `terminal_blocks` (its stand-in
    agent's `cat -v` echo).
  - 492: `browser_tabs` lists the page with its title; `browser_snapshot` holds the form's
    fields; after the typed and clicked steps the page's own result text is in the snapshot.
  - 500: sway's tree has the Browser tab's window title; `browser_tabs` lists a blank page.
  - 515: the settings file holds `"layout": "zed"` after the dropdown, and sway's tree shows the
    Zed layout's title.
  - 514 and 516 check themselves already.
- **`script/regress`.** Reads `script/e2e/golden` (or takes scenario paths), runs each through
  `script/e2e.sh` with `SHOT_DIR=$run/<scenario>` and `E2E_BINARY` passed through, times it, keeps
  its log, and prints `PASS 491-marley-mcp 74 s` or `FAIL 492-browser-tools 88 s: <the FAIL line>`;
  a verdict line; exit 1 on any failure. `$run` defaults to
  `${XDG_STATE_HOME:-~/.local/state}/marley/regress/<time>`, never the repository.
- **The installer.** After `cargo build --release`, and before `staged` is written:
  `E2E_BINARY=$built script/regress` unless `--skip-regress`; a red prints the summary and the
  run folder and exits 1 with nothing replaced.
- **File manifest:** `script/e2e.sh`, `script/regress` (new), `script/e2e/golden` (new),
  `script/install-marley`, `justfile`, and the six scenarios' checks; the owned-paths helper
  (`marley_owned_path`) gains `script/regress` if it is not covered by `script/*`.

### E2E plan
| REQ | Run | Proof |
|---|---|---|
| REQ-001 | `just regress` on the golden set, debug build | Eight PASS lines and the verdict |
| REQ-002 | `just regress script/e2e/491-marley-mcp.sh` with one expectation changed in a scratch copy | FAIL naming the check, exit 1 |
| REQ-003 | `just install --prefix $scratch` with the scratch copy of the failing scenario in the set | The prefix has no `lib/marley/marley` afterwards; the summary printed |
| REQ-004 | `just install --prefix $scratch --skip-regress` | The binary installed; the line that says the set was skipped |

### Risks
- Time: eight scenarios at one to two minutes each add ten to fifteen minutes to an install, on
  top of the release build. Recorded in the guide; `--skip-regress` is the way out.
- Flakes: a scenario that passes only most of the time turns every install red. A flaky scenario
  leaves the golden set until it is fixed, and the run log names the step.
- The golden scenarios run under sway with the runner's profile copy: Chad's session and settings
  are never touched.
