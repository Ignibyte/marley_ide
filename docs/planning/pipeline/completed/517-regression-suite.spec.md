---
pipeline_id: c4b4b8fc-4627-4ffd-bddd-428f9be34434
ticket: docs/planning/tickets/open/TICKET-517-regression-suite.md
status: Phase 4 — Complete PASS
title: "Marley's own regression suite: a golden set that checks itself before an install"
type: chore
slice: the workflow's e2e layer (CONSTITUTION §7) and the installer (#502)
references: [docs/orca_architecture/07-engineering-and-changelog.md, docs/planning/pipeline/completed/502-installed-release-build.spec.md, docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md]
---

## Title
A named golden set of e2e scenarios, each ending in machine checks through Marley's MCP server,
run together by `just regress` against a given binary, and run by `just install` against the
release build before it replaces the installed one.

## Scope
### In
- `script/e2e/golden` (new): the golden set, one scenario per line with a comment on what it
  guards. The first set: `491-marley-mcp` (the MCP server and the terminal tools),
  `484-autosuggestions` (blocks and the prompt), `481-rich-input` (the agent bar), `492-browser-tools`
  (the Browser tab driven through MCP), `500-browser-from-the-rail` (the rail's + and a Browser
  tab), `515-marley-settings-page`, `516-secret-redaction-for-agents` and
  `513-one-marley-per-data-dir` (startup).
- Machine checks where a golden scenario has none: each ends by reading Marley's state through
  the stand-in agent (`mcp_agent`: `terminal_list`, `terminal_blocks`, `browser_tabs`,
  `browser_snapshot`) or sway's tree, and fails with a line naming what it expected. The shots
  stay; a passing run needs no one to read them.
- `script/e2e.sh`: `E2E_BINARY=<path>` in the environment names the binary, as a scenario's
  `binary` does, so a runner can aim every scenario at one build.
- `script/regress` (new) and `just regress [scenario...]`: runs the golden set (or the named
  scenarios, or the list `REGRESS_GOLDEN` names) one after another through `script/e2e.sh`, every
  one in a headless sway (`COMPOSITOR=sway`, so none touches the user's session), shots under
  one run folder, and prints one line per scenario (pass or fail, seconds, the failing check) and
  a verdict; exit 1 on any failure.
- `script/install-marley`: after the release build and before anything is replaced, runs the
  golden set against the built binary; on a red it installs nothing, prints the summary and
  where the shots are, and exits 1. `--skip-regress` installs without it and says so.

### Out (explicitly deferred)
- Choosing scenarios by the files a change touches (a table from paths to scenarios) and running
  them in `script/gates.sh --diff`: slice 2, once the golden set's run time is known.
- Comparing shots against saved ones: the shots are for a human reading a failure.
- A nightly run of every scenario.

## Reference (§20)
N/A — Marley-specific: Marley's own test workflow, with no Warp or Zed behavior to match. The
idea of a small golden set re-run on every change comes from Orca's engineering practice
(docs/orca_architecture/07, its golden e2e specs and the gate that runs them); Marley runs its
own scenarios, on the real app, and checks them through its own MCP server.

### Prior art
- **The code we already ship.** `script/e2e.sh` (the runner: `compositor sway`, `binary`, the
  profile copy, `SHOT_DIR`), the browser fixture's `mcp_agent` stand-in (the plugin's bridge, the
  run's endpoint file), the self-checking scenarios #502, #512, #514 and #516 (a check that
  fails the run with `return 1` under `set -e`), `script/install-marley` (the staged binary and
  the rename), the gate's receipt (`gate_state_hash`) as the model of a per-tree record.
- **Reports.** docs/orca_architecture/07: Orca's golden e2e set and its ratchet gates.
- **Published material.** none needed: bash, `swaymsg -t get_tree`, MCP's `tools/call`.

## UI proof
N/A — no UI delta: a runner and checks inside existing scenarios; Marley's windows do not change.
The Test phase runs `just regress` on the golden set (every scenario must pass), runs it once with
a deliberately wrong expectation (it must fail and name the check), and runs `just install
--prefix <scratch>` on a tree where a golden check fails (nothing installed) and where all pass.

## Locked-In Decisions
- D1 — The install tests what it installs: the golden set runs against the release binary it
  just built, not the debug build.
- D2 — Checks go through Marley's own MCP server where they can, so a regression run also tests
  the tools agents use.
- D3 — A red install changes nothing: the installed binary, launcher and menu entry stay.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `just regress` runs, it shall run each scenario in `script/e2e/golden` against the named binary and print one line per scenario and a verdict, exiting 1 when any failed. | The run's output |
| REQ-002 | WHEN a golden scenario's machine check does not hold, the scenario shall exit non-zero with a line naming the check. | A negative smoke: one check's expectation changed for the run |
| REQ-003 | WHEN `just install` runs, it shall run the golden set against the new release binary before replacing anything, and shall install nothing when a scenario failed. | `just install --prefix <scratch>`: the scratch prefix unchanged after a red |
| REQ-004 | WHERE `--skip-regress` is given, `just install` shall install without running the set and shall print that it did. | `just install --prefix <scratch> --skip-regress` |

## Phase Plan
- **P1 Plan** — this spec; the design and the golden set's checks in the notes.
- **P2 Code** — `E2E_BINARY`, `script/regress`, the recipe, the installer's step, the checks.
- **P3 Test** — the runs above; `just gate-fast` (no Rust).
- **P4 Complete** — CONSTITUTION §7 and the guide (the golden set and `just regress`), knowledge,
  close, archive, commit.
