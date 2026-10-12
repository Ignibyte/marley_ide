# The shared plugin with plan approval and the plan-questions skill — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-776-the-shared-plugin-with-plan-approval.md
- **Pipeline spec:** 776-the-shared-plugin-with-plan-approval.spec.md

## Phase 1 — Plan
- **Request:** #726's harness request (D1) and #728's (D5), sent as MREQ-012 and MREQ-013 on
  Chad's word of 2026-10-12; the harness shipped both as TICKET-119 (`5333d15`, pushed; 800 tests,
  88 audits). Its Done message: plugin 0.4.0, digest
  `b3b5ee99611a175712af94d53dabd907e9e34056380839e2f6c7476fede7e26a`, the skill at
  `skills/plan-questions/SKILL.md`; on 2.1.287 `ExitPlanMode` takes its `plan` from the session's
  plan file, and `approve` ends plan mode while `deny` keeps it.
- **Classification:** chore, #709's carry.
- **Recall:** #724's notes are the procedure (copy the files, `FILES` in the harness's order, the
  digest computed equals the harness's, the guides). The knowledge ledgers and the brain had
  nothing on this seam (consultation 9dfd1418a42840a8ad44dd7f60a6f55e).
- **Discovery:** since `cda7c24` only `plugin.json`, `register.js` and the new `SKILL.md` changed;
  the harness's `FILES` gained the skill last, and its `DIRECTORIES` the skill's folder, which
  Marley's installer derives from the paths. `claude-code-versions.json` is unchanged, so
  `CLAUDE_SHARED_PLUGIN`'s range stays.

### Design
- Copy the three files from `5333d15` with `git show`; add the 17th entry to `FILES`; the module
  doc names TICKET-119. `plan_review.rs`'s doc, `docs/marley/guide.md` and the in-app guide say
  0.4.0 and drop "at a revision that reads it".
- **File manifest:** `crates/marley_workbench/claude_shared_plugin/` (three files),
  `src/shared_plugin.rs`, `src/plan_review.rs` (doc), `guide/index.html`, `docs/marley/guide.md`,
  the architecture note, the scenario. All Marley-owned; no ledger row.

### Visual check plan
| REQ | The scenario | Shot |
|---|---|---|
| REQ-001 | `marley.claude_code_shared_plugin` and `marley.plan_review` on; the scenario lists the run's `claude-code/shared/<digest>` and reads the scripted API's trace for the skill's listing. | The trace check |
| REQ-002 | In the project's terminal: `unshare --user --map-root-user --net` the fixture API, then `env -i` the pinned Claude Code in plan mode, keeping `MARLEY_*`, `CLAUDE_CODE_PLUGIN_DIRS` and a scratch config that has finished onboarding. Type `PLAN_WRITE` (the model writes the plan file, one `:::question`), then `PLAN_SUBMIT` (the model calls `ExitPlanMode`). | `776-01-plan` |
| REQ-003 | Accept recommended on the card, Send Answers. | `776-02-answers` |
| REQ-004 | Type `PLAN_REWRITE` (the question settled), `PLAN_SUBMIT`; Approve in the tab. | `776-03-approved` |

Not reached: a real model (the scripted API stands in for it, as in the harness's suite).

### Risks
- `marley approve` runs inside the user namespace; Marley's agent socket is a path, which a network
  namespace does not hide, and the mapped user is the same on the outside. The first run tells.
- Claude Code's TUI takes typed text as #522's stand-in's did not: the keys go to a real prompt.

## Phase 2 — Code
- **Built:** `plugin.json` (0.4.0), `hooks/register.js` and `skills/plan-questions/SKILL.md` copied
  from rustal-harness `5333d15` with `git show`; the other fourteen compared byte for byte with the
  harness's and unchanged. `shared_plugin::FILES` has 17, the skill last as in the harness; a
  digest computed over them in that order is `b3b5ee99…e26a`, the harness's. The module docs of
  `shared_plugin.rs` and `plan_review.rs`; the guides (0.4.0, the skill, and no more "at a
  revision that reads it"); the architecture note; CHANGELOG (Changed).
- **Review:** the new files name no host, address or home path; `.config/typos.toml` already
  excludes the plugin's `skills/`; `claude-code-versions.json` is unchanged, so
  `CLAUDE_SHARED_PLUGIN`'s range stays.
- **Gate:** GATE GREEN [diff], 17 of 17, receipt written.

## Phase 3 — Test
- **Scenario:** `script/e2e/776-the-shared-plugin-with-plan-approval.sh` under `compositor sway`.
  The pinned Claude Code 2.1.287 (its path from the harness's `dependencies/claude-code.json`) runs
  in a new Marley terminal in plan mode, inside `unshare --user --map-root-user --net` with the
  harness's `scripts/fixtures/claude_api.py` serving scripted replies on loopback; `env -i` keeps
  only `MARLEY_*`, `CLAUDE_CODE_PLUGIN_DIRS` and `XDG_RUNTIME_DIR` of the terminal's variables, and
  a scratch `CLAUDE_CONFIG_DIR` has finished onboarding. `MARLEY_CLAUDE` names the pinned binary,
  so Marley's version check passes. The first pass ran to `776-01` and set the click points from
  it; `marley approve` reached Marley from inside the namespace at once.
- **The final run**, on the gated tree; every check passed:
  - REQ-001: the run's data directory holds `claude-code/shared/b3b5ee99…e26a`; the scripted API
    recorded the skill's listing ("Use in plan mode before you call ExitPlanMode") in a request.
  - **776-01-plan** (REQ-002): the Plan tab, "Keep the run records in the journal.", the card
    "Question 1 of 1 · Open · Which store keeps the run records?" with its context, SQLite
    (recommended) and Postgres, Other…, Accept recommended and Skip; "0 of 1 answered"; "Claude Code
    waits for your answer."; Needs you: "Claude Code · repo, Plan to review".
  - **776-02-answers** (REQ-003): the terminal: "Error: Permission to use ExitPlanMode denied by
    plugin rustal-harness: The user answered your questions. Revise the plan with these answers,
    then call ExitPlanMode again.", "Answers to your questions · 1 of 1 questions answered ·
    Question 1 … Answer: SQLite (your recommendation)"; the footer "plan mode on"; the API recorded
    the answers.
  - **776-03a-revised:** "Version 2 +1 −1 Changes", the question "Settled", SQLite ticked, Approve.
  - **776-03-approved** (REQ-004): "User approved Claude's plan · Plan saved to …/plans/….md" with
    the plan; the footer "manual mode on": plan mode ended; the API recorded "User has approved
    your plan."
- A tooltip of the terminal's tab covered part of 02 and 03 in the first full run; the scenario
  waits two seconds after moving the pointer away, before the gate.
- **Not reached:** a real model (the scripted API stands in, as in the harness's suite).

## Phase 4 — Complete
- **Documented:** CHANGELOG (Changed); `docs/marley/guide.md` and the in-app guide; the
  architecture note's shared-plugin section (seventeen files at `5333d15`, digest `b3b5ee99…e26a`).
- **Knowledge:** L-claude-776-a-real-claude-code-runs-in-a-marley-terminal-against-the-harnesss-scripted-api-001.
- **Brain:** consultation 9dfd1418a42840a8ad44dd7f60a6f55e decided.
- **Closed:** TICKET-776; the pair archived to `completed/`.
