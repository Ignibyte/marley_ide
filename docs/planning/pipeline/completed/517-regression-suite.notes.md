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
- **Recall (§18.3):**
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: a rerun where every step
    "passed" while the scenario clicked the wrong button. Machine checks are what catch that.
  - The self-checking scenarios (#502, #512, #513, #514, #516): a check prints `check <what>:
    pass` or `FAIL` and returns 1, which the runner's `set -e` turns into a red run.
  - L-claude-516-a-second-window-in-sway-narrows-the-terminal-under-test-001 and TICKET-544:
    checks on blocks run before another window opens.
  - Brain (consultation 4ebe8ab965924d078bed4442b03d606c): nothing on this seam.
- **Discovery:** `script/e2e.sh` 60 to 61 (`MARLEY_BIN`, `binary`), 71 (`COMPOSITOR` from the
  environment when the scenario names none), 492 (the binary check), 552 (the binary chosen);
  `script/e2e/browser-fixture.sh` (`mcp_agent`: `terminals`, `terminal-read`, `tabs`, `snapshot`,
  `console`); `script/install-marley` 63 to 71 (the release build, the staged copy, the rename);
  the `justfile` (`e2e`, `install`). #481 and #484 name no compositor, so they run on Chad's
  Hyprland, where the runner refuses while a Marley window is open.

### Design
- **`E2E_BINARY`.** `script/e2e.sh` sets `MARLEY_BIN=${E2E_BINARY:-}` where it now sets it empty,
  so a scenario's own `binary` still wins, and a runner can aim every scenario at one build.
- **The golden set** (`script/e2e/golden`, one scenario per line with what it guards):
  `491-marley-mcp`, `484-autosuggestions`, `481-rich-input`, `492-browser-tools`,
  `500-browser-from-the-rail`, `515-marley-settings-page`, `516-secret-redaction-for-agents`,
  `513-one-marley-per-data-dir`.
- **The checks it adds**, each a line `check <what>: pass` or `check <what>: FAIL` from an
  `expect` helper that returns 1:
  - 491: the harness's client lists the terminal's blocks with `echo hi` exit 0, `false` exit 1,
    `seq 3` exit 0, and reads `seq 3`'s output as `1 2 3`; the endpoint file's mode is 600; the
    watcher saw `tools/list_changed` at the quit; with no Marley, `tools` lists none.
  - 484: `terminal-read` finds the block `echo hello world` (history's suggestion taken with →)
    and `echo from this session`.
  - 481: the stand-in agent's block holds what the rich input sent (`hello rich input`, `second
    line`) and what was typed after (`typed after`).
  - 492: the navigate answer names the page's title; the snapshot holds the form's fields; the
    console lists `hello from the page`; the typed and clicked steps change the page's result
    text; the two refused navigations are refused.
  - 500: `tabs` lists the site's page in a Browser tab.
  - 515: the settings file holds `"layout": "zed"` after the dropdown.
  - 516 and 513 check themselves already.
- **`script/regress`** (new) and `just regress [scenario...]`: runs each scenario of the golden
  set (or those named) through `script/e2e.sh` one after another with `COMPOSITOR=sway`, so every
  run is headless and none touches Chad's session, `E2E_BINARY` passed through, a 15-minute
  timeout each, and `SHOT_DIR` at `$run/<scenario>` under
  `${XDG_STATE_HOME:-~/.local/state}/marley/regress/<time>` (the five newest runs kept). It
  prints `PASS <scenario> <seconds> s` or `FAIL <scenario> <seconds> s: <the first FAIL line>`
  and a verdict, and exits 1 on any failure.
- **The installer.** After the release build and before anything is staged, `E2E_BINARY=$built
  script/regress`, unless `--skip-regress`; a red prints where the run's logs are and exits 1
  with nothing replaced.
- **File manifest:** `script/e2e.sh`, `script/regress` (new), `script/e2e/golden` (new),
  `script/install-marley`, `justfile`, and the checks in the six scenarios; `marley_owned_path`
  already covers `script/*`. No Rust, so the gate is `--fast`.

### E2E plan
| REQ | Run | Proof |
|---|---|---|
| REQ-001 | `just regress` on the golden set, debug build | One PASS line per scenario and the verdict |
| REQ-002 | `just regress script/e2e/491-marley-mcp.sh` on a scratch copy with one expectation changed | FAIL naming the check, exit 1 |
| REQ-003 | `script/install-marley --prefix $scratch` with the scratch copy of the failing scenario in the set (`REGRESS_GOLDEN=<file>`) | `$scratch/lib/marley/marley` absent afterwards; the summary printed |
| REQ-004 | `script/install-marley --prefix $scratch --skip-regress` | The binary installed; the line saying the set was skipped |

### Risks
- Time: eight scenarios at one to two minutes each add ten to fifteen minutes to an install, on
  top of the release build. Recorded in the guide; `--skip-regress` is the way out.
- Flakes: a scenario that passes only most of the time turns every install red. A flaky scenario
  leaves the golden set until it is fixed, and the run log names the step.
- The golden scenarios run under sway with the runner's profile copy: Chad's session and settings
  are never touched.

## Phase 2 — Code
- **Built:**
  - `script/e2e.sh`: `MARLEY_BIN=${E2E_BINARY:-}` (a scenario's `binary` still wins); two helpers
    for scenarios, `expect <what> <command...>` (prints `check <what>: pass` or `FAIL`, and a FAIL
    ends the run) and `holds <file> <text>...` (each text as a fixed string); the header documents
    both and `E2E_BINARY`. #513's scenario drops its own `expect` for the runner's.
  - `script/e2e/golden`: the eight scenarios, each with what it guards.
  - `script/regress` (new): the golden set, the scenarios named, or `REGRESS_GOLDEN`'s list, each
    through `script/e2e.sh` with `COMPOSITOR=sway`, `E2E_BINARY` passed through and a 900-second
    timeout; `SHOT_DIR` per scenario under `${XDG_STATE_HOME:-~/.local/state}/marley/regress/<time>`
    (five newest kept); `PASS <name> <s> s` or `FAIL <name> <s> s: <why>` and a verdict; exit 1 on
    any failure. `just regress [scenario...]` runs it after `idle`, since a cargo build beside it
    would slow the scenarios' timing.
  - `script/install-marley`: `--skip-regress`; after the release build and before anything is
    staged, `E2E_BINARY=$built script/regress`, and a red installs nothing. Its header no longer
    says a second Marley hangs (#513).
  - The checks: 491 (the harness's client: the blocks' exit codes, `seq 3` read back as `1 2 3`,
    the endpoint file's mode, `list_changed` at the quit, no tools with no Marley); 484 (the
    suggestion taken with → ran as `echo hello world`, and `echo from this session` ran); 481 (the
    stand-in agent's block holds `claude got:` for both rich-input lines and the typed one); 492
    (navigate names the title; the snapshot names the form's and the frame's fields; the console
    holds the page's messages; a full snapshot after the typing and the click holds "Signed in as
    agent@example.com." and the frame's echo; both refused navigations; no tool evaluates
    script); 500 (`tabs` lists the page opened from the rail); 515 (the settings file holds
    `"layout": "zed"`). The stand-in agent's `snapshot` takes `full`.
  - `script/regress` joins the owned paths (`marley_owned_path`) and the gate's shellcheck list.
- **Deviation, and a trap:** the installer was edited while `just install` was running from it;
  bash reads a script as it executes, so the edit could have shifted what the running install read
  after its `cargo build`. The committed file was restored within the minute, while the build still
  ran, and the edit is re-applied once the install ends.
- **Review:** `regress` reads the failing line from the scenario's own log (its first `FAIL`, a
  missing window or a build message), and a timeout is named as one; the check helpers return 1
  only through `expect`, so a helper's failure inside a condition never trips `set -e` early; the
  pruning removes only directories under the runs folder. No Rust changed.

## Phase 3 — Test
- **REQ-001, the golden set** (`script/regress` against the debug build; `just regress` was
  waiting on its `idle` for another project's single-job release build, which the runner does
  not need, so the script ran directly: the recipe adds only that wait):
  `PASS 491-marley-mcp 45 s`, `PASS 484-autosuggestions 26 s`, `PASS 481-rich-input 34 s`,
  `PASS 492-browser-tools 29 s`, `PASS 500-browser-from-the-rail 36 s`,
  `PASS 515-marley-settings-page 28 s`, `PASS 516-secret-redaction-for-agents 42 s`,
  `PASS 513-one-marley-per-data-dir 71 s`, "regress: all 8 passed": about five minutes. Every
  check line in the eight logs reads `pass`, thirty in all (491: five; 484: two; 481: one; 492:
  six; 500: one; 513: seven; 515: one; 516: five with their counts), so each scenario reached its
  checks rather than exiting early.
- **Shots read from the run:** #481 and #484 ran under sway for the first time (they name no
  compositor, and `regress` sets `COMPOSITOR=sway`). `481-04-two-lines`: the stand-in as "Claude
  Code · working" in the rail, the agent bar (Claude Code, +, pencil, microphone, "Connect Claude
  Code to Marley", folder, branch) and the rich input above it with both lines. `484-01-ghost`:
  `ech` with "o hello world" dimmed after the cursor.
- **REQ-002, a negative smoke:** a scratch copy of #491 expecting `'false': exit 7`, run through
  `script/regress`: `FAIL 491-broken 33 s: check the blocks' exit codes: FAIL`, "1 of 1 failed",
  exit 1. The count line then read "1 scenarios"; it now reads "1 to run".
- **REQ-003:** `REGRESS_GOLDEN=<the broken list> script/install-marley --prefix <scratch>`: the
  release build (already built, 0.66 s) ran the broken scenario, `FAIL 491-broken 34 s`,
  "install-marley: the golden set failed against the new build, so nothing was installed", exit
  1, and the scratch prefix did not exist afterwards. The release binary runs the scenarios as the
  debug one does.
- **REQ-004:** `script/install-marley --prefix <scratch> --skip-regress`: "the golden set skipped
  (--skip-regress)", then `bin/marley`, `lib/marley/marley`, the desktop entry and the icon in the
  scratch prefix, exit 0; the scratch prefix was removed after.
- **Gate:** `just gate-fast`: `GATE GREEN [fast]` (shellcheck over `script/regress` among the
  rest). No Rust changed, so no receipt is needed.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Added: Marley's own regression suite); `docs/marley/guide.md` (the
  install section and the developers' table: `just regress`, `--skip-regress`); the runner's
  header (`E2E_BINARY`, `expect`, `holds`), `script/regress`'s own header, the golden list's
  comments, the browser fixture's header (`snapshot [full]`). No Marley crate changed, so no
  per-crate note; no Zed path changed. CONSTITUTION §7 gains the golden set in a commit of its
  own, as its amendment rule asks.
- **Knowledge:** L-claude-517-a-running-bash-script-reads-its-file-as-it-goes-001,
  AD-claude-517-the-golden-set-gates-the-install-001.
- **Brain:** consultation 4ebe8ab965924d078bed4442b03d606c closed with
  `decisions/marleys-golden-set-of-self-checking-e2e-scenarios-gates-every-install`, follow-up by
  2026-10-10.
- **Ticket:** closed; the pair archived to `completed/`.
