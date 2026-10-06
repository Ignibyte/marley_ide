# Test runs act on nothing of the user's — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-668-test-runs-act-on-nothing-of-the-users.md
- **Pipeline spec:** 668-test-runs-act-on-nothing-of-the-users.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #667, from the T3 Code survey (`docs/t3code_architecture/
  README.md`, "For Marley's own workflow").
- **Classification / tier:** chore, the e2e harness; no Rust.
- **Pre-flight:** green; no active pipeline; cargo idle.
- **Recall (§18.3):**
  - R-D8: scenarios never touch the user's brain or Rusty; the harness already turns Rusty off.
  - #633 to #653's lines in the harness: the user's features turned off in the copy, one Python
    block, the stand-ins named by variables that name no file until a scenario names its own.
  - The brain (consultation `0a579aa4be74459891fb9840a599502f`): nothing on this seam.
- **Discovery:**
  - `script/e2e.sh`: `config` and `data` from the XDG variables (`:57-58`), the scenario sourced at
    top level (`:81`) before the copy (`:627`), the Python block that turns features off
    (`:643-661`), `MARLEY_RUSTY_MCP` and `MARLEY_CODEX` exported after it, `setup` at `:737`.
  - `crates/settings_content/src/marley.rs`: `push`, `harness`, `embedded_harness`, `system_one`,
    `fleet`; `system_one.rs`: `MARLEY_SYSTEM_ONE_KEY`, `MARLEY_CLOUDFLARE_API_TOKEN`.
  - Scenarios that set their own: 535 (`marley.push`, its own helper), 548 and 565 (export the two
    variables in `setup`), 640 (`marley.harness`), 641 (`marley.fleet`), 651
    (`marley.system_one`).

### Design
- **`script/e2e.sh`**: in the Python block, `for key in ("push", "harness", "embedded_harness",
  "fleet", "system_one"): marley.pop(key, None)`, with a line in the comment above; after the block,
  `unset MARLEY_SYSTEM_ONE_KEY MARLEY_CLOUDFLARE_API_TOKEN` beside the two stand-in variables, with
  a line saying why.
- **The scenario**: at top level, `config` points at a scratch config under the shot folder with
  every one of the five settings and a harmless `"theme"`, and both variables are exported; its
  `setup` checks the run's copy and environment; `steps` takes one shot.
- **File manifest:** `script/e2e.sh`, `script/e2e/668-test-runs-act-on-nothing-of-the-users.sh`.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001, 004 | `setup` reads the run's copy of the settings | Checks: five gone, `theme` kept |
| REQ-002 | `setup` reads its own environment, which Marley inherits | Checks: neither variable set |
| REQ-003 | Run 535, 548, 565, 640, 641, 651 | Each passes its checks |
| — | `steps` | Shot `668-01-started` |

### Risks
- **A scenario that read the user's System One or fleet settings** without setting its own would
  now see defaults; the six that use them set their own, and the run of each shows it.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **`script/e2e.sh`**: the Python block that turns the user's features off now also drops
  `marley.push`, `marley.harness`, `marley.embedded_harness`, `marley.fleet` and
  `marley.system_one` from the copy; after it, `unset MARLEY_SYSTEM_ONE_KEY
  MARLEY_CLOUDFLARE_API_TOKEN`, before any scenario's `setup`. The comment above the block names
  what is left out and why.

### Deviations
- None from the design.

### Review
- bash -n and shellcheck on the harness pass; the scenarios that set these settings set them in
  `setup` or later, after the copy and the unset.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
### The scenario
`script/e2e/668-test-runs-act-on-nothing-of-the-users.sh`, run with `just e2e`. At top level it
points the harness's `config` at a scratch folder under the shots, whose `settings.json` holds a
comment, trailing commas, `"theme": "One Dark"` and made-up `marley.push`, `marley.harness`,
`marley.embedded_harness`, `marley.fleet` and `marley.system_one`, and exports made-up
`MARLEY_SYSTEM_ONE_KEY` and `MARLEY_CLOUDFLARE_API_TOKEN`. shellcheck and typos clean.

### Checks and shots
| Criterion | Proof | Result |
|---|---|---|
| REQ-001 | five checks: the run's copy leaves out each of the five | pass |
| REQ-004 | the run's copy keeps `theme` | pass |
| REQ-002 | the run holds neither variable (`setup` runs in the environment Marley inherits) | pass |
| — | `668-01-started`: Marley on the run's profile, the scratch repo open, a terminal at its path, One Dark | seen |
| REQ-003 | 535, 548, 565, 640, 641, 651, each with its own fakes | all exit 0 |

- **535** (18 checks pass): needs input, finished, failed (`rate_limit`), focused, and the
  "could not push" toast naming the scenario's own server on 127.0.0.1.
- **548** (9 checks pass): its own Cloudflare account and key from `setup`; the Calls tab lists
  only the run's calls; the refusal toast when no account is set.
- **565** (18 checks pass): answered, failed, masked, metadata only, slow, breaker open, budget,
  replay and off, each in its toast and the Calls tab.
- **640** (no checks; shots): the harness's questions reach Needs you (review's in 01 and 02,
  asker's joins in 04), so `marley.harness` from `setup` reaches Marley. Its Harness rows sit below
  the rail's Containers section, which lists the 13 containers running on this machine, and
  `BUILD_Y` now lands on `:8143 docker`. Run again on the harness at HEAD, with #668 taken out, the
  four shots are the same: pre-existing, not in scope. Filed as TICKET-669 (below).
- **641** (6 checks pass): connected, silent, the host's reason, input off, reattached, refused,
  rerun and ended.
- **651** (10 checks pass): command, file change, permissions, elicitation and input approvals
  through the inbox, and the server gone.

### Pre-existing — not in scope
- **The rail's Containers section reads the machine.** Every run's rail lists the containers
  running on the machine (`ports.rs` reads `/proc` and asks `docker ps`; `proc_root` is a test-only
  override), each row with a Stop that runs `docker stop`, so a click that misses could stop one of
  the user's real containers, and their number moves every row below them. 641's Fleet panel lists
  this machine's processes for its `lab` host (127.0.0.1), read only. Filed as TICKET-669.

### Focus
#668's run, on Hyprland: the user's window and workspace as they were. The six and 640's
second run, in headless sway: 0 Marley windows on Hyprland before and after, no rule added, sway
stopped with the run's Marley.

## Phase 4 — Complete (2026-10-06)
- **Documented:** `CHANGELOG.md` (Security: test runs leave the user's phone, harness, hosts and
  System One alone); the T3 Code survey's README marks its first workflow item done; the harness's
  own comment names what the copy leaves out. No Zed path touched.
- **Knowledge:** `L-claude-668-a-scenario-can-stand-in-for-the-users-settings-001`,
  `AD-claude-668-a-test-run-inherits-nothing-that-reaches-outside-it-001`.
- **Brain:** consultation `0a579aa4be74459891fb9840a599502f` closed with
  `decisions/marleys-test-runs-inherit-nothing-that-reaches-outside-them`.
- **Filed:** TICKET-669 (a run's rail lists the machine's containers), top of the Queue.
- **Ticket:** closed; the pair archived.
