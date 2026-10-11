# Failed checks to an agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-718-failed-checks-to-an-agent.md
- **Pipeline spec:** 718-failed-checks-to-an-agent.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** MonoCode finding 5 (`docs/planning/intake/monocode-findings.md`): "#531 already
  polls `gh`: show the checks and "Send failures to agent"; the inbox (issue, start work, worktree
  agent) after." Chad, 2026-10-11, answering item 14 of the findings review: "14.) ok". This
  slice is the first half; the inbox is Out. It sits in the agents-together batch after the finish
  (Deliberate).
- **Classification:** feature, M.
- **Recall (§18.3):**
  - AD-claude-531: a project row's counts and pull request come from Marley's git and `gh`. `gh`
    runs with the branch as one argument and no shell, and the data stays in the rail's group
    entries, not `marley_rail`'s row model. The checks follow the same rule.
  - #531's Out deferred "CI checks on the chip (`statusCheckRollup`), review state, and a click that
    opens the PR". This ticket takes the checks.
  - #522 (D5 to D7): targets whose folder holds the files; the ready test; paste then Enter. This
    slice pastes without Enter, as #555 does.
  - #516: redaction for anything an agent reads.
  - L-claude-603: a scenario fakes a program for Marley through set-up's PATH, but `gh` is named by
    `MARLEY_GH`, since Marley takes its PATH from the login shell (`github.rs:13-15`).
  - The T3 survey's open question 1 (pull requests for agent work) does not bear on this slice:
    it reads checks Chad's own branches already have.
- **Discovery:**
  - `github.rs:61-109`: `pull_request` and `Listed`. The JSON list gains `statusCheckRollup` and
    `headRefOid`. A new `failed_log(folder, owner_repo, job)` runs `gh run view`.
  - `rail.rs`: `ProjectGit` (549-554) holds `pull_request`, which gains the checks;
    `pull_request_chip` (8475-8503) gains the mark, the tooltip line and a right-click menu
    (`right_click_menu`, as the rail's rows use).
  - `send_selection.rs:387-434` (`send_text`, `submit = false`) delivers.
  - `review_notes.rs:38-103` is the picker to copy: targets filtered by folder, readiness words, a
    copy row.
  - The data folder: through an `*_in(dir)` helper (§14), so a scenario's HOME holds the file.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again. A sketch follows.

### Design
- **Approach:**
  - **`github.rs`:**
    - `Check { name, workflow, outcome, link, job }` and `CheckOutcome { Failed, Pending, Passed,
      Cancelled }`, parsed from `statusCheckRollup`'s two shapes (`CheckRun` and `StatusContext`);
      `PullRequest` gains `checks` and `head`.
    - `failed_log(folder, owner_repo, job) -> anyhow::Result<String>`.
  - **`rail.rs`:**
    - the mark beside `#<n>` (Zed's `IconName::Check`, `XCircle` and a dot, in the theme's success,
      error and warning colours);
    - the tooltip line;
    - the chip's right-click menu while a check failed.
  - **`failed_checks.rs`** (new):
    - the action `SendFailedChecksToAgent` and its palette entry;
    - the picker, `agent_targets` in the project's folder, with Copy Prompt;
    - on pick, a background task fetches the logs, keeps their ends, redacts them and writes the
      file with mode 0600, then pastes the prompt through `send_text`;
    - errors show as a toast naming the step that failed.
- **File manifest (all Marley crate):** `github.rs`, `rail.rs`, `failed_checks.rs` (new),
  `marley_workbench.rs`; the guide; the scenario. No Zed path is touched, so no ledger row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | The repository on `feature`; the stand-in `gh` answers #42 with three checks | 718-01-chip (hover for the tooltip) |
| 003 | Right-click the chip | 718-02-menu |
| 004 | Choose the item | 718-03-picker |
| 005, 006, 007 | Pick the stand-in | 718-04-pasted; the file; the `gh` log |
| 001, 003 | Switch to `green` | 718-05-green |
| 008 | — | The review of the diff |

- **Not reached:** GitHub itself. The stand-in `gh` answers in `gh` 2.102.0's shapes, taken from
  `gh pr list --json statusCheckRollup` on a real repository at promotion and saved as the
  scenario's fixture.

### Risks
- `statusCheckRollup` can be long on repositories with many checks. The lookup keeps only name,
  workflow, outcome and link, and the tooltip names at most five of each kind, then "and N more".
- `gh run view --log-failed` may fail for a job whose logs have expired, or when there are more
  than 25 missing job logs. Such a check goes in by name and link.
- A check run from another CI app (not Actions) has a link Marley cannot fetch a log from. It goes
  in by name and link, as REQ-007 says.
