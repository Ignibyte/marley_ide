# The Zed touchpoint ledger, enforced — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-436-zed-touchpoint-ledger.md
- **Pipeline spec:** 436-zed-touchpoint-ledger.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** Chad, 2026-09-22: "Our crates should attempt to not go into zed as much as
  possible and if we do then we need to record it for future code changes." Goal set the same
  day: "lets create the tickets and make it happen."
- **Classification / tier:** chore, gate-is-test (no `.rs`); verified by exit codes and
  negative smokes (§7).
- **Recall (§18.3):** `PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001` (count with
  `grep -c`, never end a pipefail pipeline in `grep -q`); `PR-claude-detection-tracks-runner-001`
  (keep detectors and the prescribed command in lockstep);
  `PR-claude-receipt-binds-gate-definition-001` and
  `PR-claude-gate-defining-files-in-receipt-fingerprint-001` (a gate edit invalidates the
  receipt, so W0 lands after the baseline commit); `PR-claude-gate-verdict-owns-the-exit-code-never-pipe-001`.
  Brain: `decisions/marleys-shell-is-a-marley-layout-beside-zeds-built-as-a-workspacesidebar-in-a-marley-crate`.
- **Discovery:** `script/gates.sh` (`upstream_base` at :74, `run_gate` at :60, the static block
  at :352-362); `.claude/hooks/lib-hook-helpers.sh` (`normalize_path`); `.claude/settings.json`
  (PreToolUse `Write|Edit` already runs `enforce-phase-gate.sh`); the ledger's current rows
  (six paths from the port).
- **Human confirmation:** Chad's goal authorizes autonomous execution of this sprint through
  commit on `marley/workbench-shell` (2026-09-22). This harness has no `TaskCreate`, so each
  phase's checklist is kept in these notes.

## Phase 2 — Design (drafted while #443's first FULL run finished)
- **Approach.** One shared definition, two enforcers.
  - `lib-hook-helpers.sh` gains `ZED_LEDGER`, `marley_owned_path`, `zed_ledger_rows` (the
    backticked path opening each row of the ledger's Touchpoints table, parsed without a quoted
    backtick so gate:11's shellcheck stays clean), `line_in_list` (pure bash membership, no
    pipe for an early-exiting grep to break under pipefail) and `zed_ledger_check <base>`.
  - `script/gates.sh` gains gate:16 as a static gate: `zed_ledger_g() { zed_ledger_check
    "$(upstream_base)"; }`, run after gate:14 in every mode.
  - `.claude/hooks/enforce-zed-ledger.sh` (PreToolUse `Write|Edit`) allows a path outside the
    repo, a Marley-owned path, a gitignored path, or a path the ledger names, and blocks
    anything else with exit 2.
  - `.claude/settings.json` runs the hook beside `enforce-phase-gate.sh`.
  - `CONSTITUTION.md` §0 lists gate:16; §14's upstream discipline and §21 point 2 name the
    ledger.
- **Drafts:** `scratchpad/w0/ledger-helpers.sh` and `scratchpad/w0/enforce-zed-ledger.sh`,
  shellcheck-clean at `-S info`; the check passes on the port's tree (six rows) and the hook
  blocks `crates/paths/src/paths.rs` and `crates/settings_content/src/marley.rs` while
  allowing `README.md`, a Marley crate, `docs/marley/`, `.mcp.json` and a path outside the
  repo.
- **Test plan (gate-is-test, §7):** negative smokes in a throwaway `git worktree`, so the main
  tree is never dirtied: an unlisted Zed edit (red, then green once the row exists), a stale
  row (red), a new untracked Zed file (red), a deleted Zed file (red); hook smokes on crafted
  PreToolUse JSON for each allow and block case; `script/gates.sh --fast` green on the real
  tree.
- **Commits.** The amendment rule wants `CONSTITUTION.md` in a commit with only its enforcers:
  commit one is the constitution, the helper, the hook, the settings wiring and `gates.sh`;
  commit two is the ledger prose, the CHANGELOG entry and the pipeline paperwork.
