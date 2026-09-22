# The Zed touchpoint ledger, enforced — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-436-zed-touchpoint-ledger.md
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
- **Promoted to active (2026-09-22, after the #443 baseline commit `f67d7b0`).** Seams
  re-verified against the committed tree: `run_gate` at `script/gates.sh:60`,
  `UPSTREAM_BASE_FALLBACK` at :73 and `upstream_base` at :74 unchanged; the static gate block
  moved to :385-395 (gate:12 grew the mask ban in #443); `normalize_path` at
  `lib-hook-helpers.sh:21`; `.claude/settings.json`'s PreToolUse `Write|Edit` runs only
  `enforce-phase-gate.sh`; CONSTITUTION §0 lists gates 1-14 (gate:15 retired, line 99), §14's
  upstream discipline is at :208, §21 point 2 at :347-351, and the amendment rule at :358
  wants the constitution in a commit with only its enforcers. `enforce-commit-gate.sh` needs a
  receipt only when `.rs` files change, so both W0 commits go through on a `--fast` green.
- **Recall added by #443.** `F-claude-443-d-pipefail-hooks-raced-sigpipe-again-001` (the pipefail
  class recurred in three hooks: no pipe into an early-exiting reader, anywhere);
  `L-claude-443-run-the-gate-not-a-reconstruction-001` (prove gate:16 through
  `script/gates.sh --fast` itself, not a hand-built copy). The gpui-era
  `PR-claude-diff-gate-mutation-tests-touched-shim-fn-001` still tells a reader to add
  `mutants::skip`, which gate:12 now bans; it gets a superseded note in this ticket's docs
  commit so W2 does not follow it. Brain consultation `2ef0d62346e94648bda185ffe1b08068`:
  nothing on this seam.
- **Phase 1 checklist (promotion):** promote the pair ✓ · re-verify seams ✓ · recall ✓ · ticket
  in-progress ✓ · backlog row removed ✓ · status PASS (autonomous) ✓.

## Phase 2 — Design
Drafted while #443's first FULL run finished; confirmed and completed after the baseline commit.

- **Approach.** One shared definition of the Marley-owned paths, two enforcers.
  - `.claude/hooks/lib-hook-helpers.sh` gains `ZED_LEDGER`, `marley_owned_path <path>` (a `case`
    over the owned globs), `zed_ledger_rows` (the backticked path opening each row of the
    ledger's Touchpoints table, parsed with the backtick built by `printf '\140'` so gate:11's
    shellcheck stays clean), `line_in_list <item> <list>` (pure bash membership, so no pipe
    feeds an early-exiting grep under pipefail) and `zed_ledger_check <base>`, which prints
    every changed path outside the owned set without a row and every row whose path no longer
    differs, and returns 1 if it printed either.
  - "Changed" is `git diff --name-only --no-renames <base> -- .` (base against the worktree,
    so staged, unstaged and deleted paths count, and a rename counts as both paths) plus
    `git ls-files --others --exclude-standard` (untracked, not ignored). The committed tree
    shows exactly the ledger's six rows, so gate:16 starts green.
  - `script/gates.sh` gains gate:16, a static gate: `zed_ledger_g() { zed_ledger_check
    "$(upstream_base)"; }`, run after gate:14 in every mode, `--fast` included. An empty base
    (no fetched `upstream` remote, no `MARLEY_UPSTREAM_BASE`, the fallback commit missing)
    fails closed with the reason. The header comment's gate list gains `16 zed-ledger`.
  - `.claude/hooks/enforce-zed-ledger.sh` (PreToolUse `Write|Edit`, always on) allows a path
    outside the repo, a Marley-owned path, a gitignored path and a path the ledger names, and
    blocks anything else with exit 2 and a message naming the ledger. A path holding a `..`
    segment never counts as owned (it could climb out of `crates/marley_x/`); it still passes
    when the ledger names it exactly, and is blocked otherwise. A relative path is checked as
    written, so it fails closed too.
  - `.claude/settings.json` runs the hook beside `enforce-phase-gate.sh` in the `Write|Edit`
    matcher.
  - `CONSTITUTION.md` (the amendment; text below).
- **§20.** `N/A` still holds: the fork's own merge hygiene; no Warp or Zed behavior is
  reimplemented.
- **The amendment text.**
  - §0 STATIC list, after gate:14: `gate:16 zed ledger     every changed path outside the
    Marley-owned set has its row in docs/marley/zed-touchpoints.md`.
  - §0 scoping bullets, after gate:14's: "`gate:16` compares the tree, untracked files
    included, with the upstream fork point (`upstream_base` in the gate: the merge-base with
    `upstream/main`, else `MARLEY_UPSTREAM_BASE`, else the recorded fork commit). It fails on
    a changed path outside the Marley-owned set that has no row in
    `docs/marley/zed-touchpoints.md`, and on a row whose path no longer differs. The owned set
    is `marley_owned_path` in `.claude/hooks/lib-hook-helpers.sh`, shared with the write hook
    (§14)."
  - §14 upstream discipline, a second bullet: "Every change outside the Marley-owned paths
    gets its row in `docs/marley/zed-touchpoints.md` in the same change (what changed, why,
    what to do at a merge), and a code hunk carries a `// Marley: <why>` comment.
    `enforce-zed-ledger.sh` blocks a Write or Edit to such a path until the row exists;
    gate:16 catches a change made any other way (§0)."
  - §21 point 2: "and a short note under `docs/marley/` for a change inside a Zed crate (which
    crate, what was added, why it is additive)" becomes "and, for a change outside the
    Marley-owned paths, its row in `docs/marley/zed-touchpoints.md` (what changed, why, what
    to do at a merge; gate:16 fails without it)".
- **File manifest** (gate-is-test: no `crates/*/src` file changes).
  - `.claude/hooks/lib-hook-helpers.sh` (modify): the five helpers above.
  - `.claude/hooks/enforce-zed-ledger.sh` (new): the write hook.
  - `.claude/settings.json` (modify): the hook in the `Write|Edit` matcher.
  - `script/gates.sh` (modify): `zed_ledger_g`, its `run_gate` line, the header's gate list.
  - `CONSTITUTION.md` (modify): §0, §14, §21 as above.
  - `docs/marley/zed-touchpoints.md` (modify): the enforcement paragraph says gate:16 and the
    hook are live and that `marley_owned_path` is the authority for the owned list.
  - `docs/planning/knowledge/prevention-rules.md` (modify): a superseded note on the
    gpui-era `PR-claude-diff-gate-mutation-tests-touched-shim-fn-001` (its advice is now a
    gate:12 failure).
  - `CHANGELOG.md`: the entry at Phase 5.
- **Regression test plan** (gate-is-test, §7: exit codes and negative smokes).

  | Check | How | REQ |
  |---|---|---|
  | An unlisted Zed change turns gate:16 red and names the path | in a throwaway `git worktree` carrying the W0 files: append a line to `docs/README.md`, `zed_ledger_check` → 1 naming it; add its row → 0 | 001 |
  | The same through the real gate | main tree: `printf >> docs/README.md` (a Bash edit, which the hook does not see), `script/gates.sh --fast` → gate:16 FAIL naming it; `git checkout -- docs/README.md`, rerun → GATE GREEN [fast] | 001, 003 |
  | A new untracked Zed file, a deleted Zed file and a renamed Zed file are changes | worktree: each → 1 naming the path(s) | 001 |
  | A gitignored file is not a change | worktree: a file under an ignored path → 0 | 001 |
  | A stale row turns gate:16 red | worktree: a row for an untouched path → 1 naming it | 002 |
  | An empty base fails closed | `zed_ledger_check ""` → 1 with the reason | 001 |
  | The committed tree is green | `script/gates.sh --fast` → gate:16 PASS (six rows), GATE GREEN [fast] | 003 |
  | The hook blocks an unlisted Zed path | crafted PreToolUse JSON: `crates/paths/src/paths.rs`, a relative `crates/zed/src/zed.rs`, `crates/marley_mcp/../zed/src/zed.rs`, `CLAUDE.md` (the symlink; edit `.rules` instead) → exit 2 each, stderr names the ledger | 004 |
  | The hook allows owned, listed, ignored and outside paths | `crates/marley_mcp/src/lib.rs`, `docs/marley/workbench-shell.md`, `README.md`, `.mcp.json`, a scratchpad path, and an input without `file_path` → exit 0 each | 005 |
  | One definition of the owned set | review: `gates.sh` and the hook both call `marley_owned_path` from `lib-hook-helpers.sh`; no second glob list | 006 |
  | The constitution names gate:16 and the ledger | grep §0, §14, §21 | 007 |
  | shellcheck | `script/gates.sh --fast` gate:11 PASS | 008 |

  Nothing is uncoverable. No live drive: no UI.
- **Risks and decisions.**
  - A Bash edit (`sed -i`, a heredoc) is invisible to the hook by design. gate:16 catches it
    whenever a gate runs, and `/commit` runs one for every commit. The commit hook itself
    requires a receipt only for `.rs` changes, so a commit made outside `/commit` could still
    carry an unrecorded non-Rust Zed change. A commit-time ledger check would close that; it
    is left as a ratchet for when a miss shows up (the spec keeps it out of scope).
  - The hook goes live when `.claude/settings.json` is saved. From then on every Write or Edit
    to a Zed path in any session needs its row first, W1's rename included. That is the point
    of the ticket.
  - Rows are exact paths (D3). A slice that adds many files outside the owned set adds a row
    for each; a directory row can come later if that turns tedious.
  - `upstream/main` is not fetched on this box, so the base is the recorded fork commit
    `78648aaf7d` until an upstream merge moves it (the merge checklist in the ledger already
    says to move both).
- **Commits.** The amendment rule wants `CONSTITUTION.md` in a commit with only its enforcers:
  commit one is `CONSTITUTION.md`, `lib-hook-helpers.sh`, the hook, `settings.json` and
  `gates.sh`; commit two is the ledger prose, the knowledge note, the CHANGELOG entry and the
  pipeline paperwork.
- **Phase 2 checklist:** approach ✓ · §20 confirmed ✓ · manifest ✓ · test plan ✓ · risks ✓ ·
  present (autonomous) ✓.

## Phase 3 — Implement
- **Built** (the manifest, in the order that keeps the wiring from pointing at a missing file).
  - `.claude/hooks/lib-hook-helpers.sh`: `ZED_LEDGER`, `marley_owned_path`, `zed_ledger_rows`,
    `line_in_list`, `zed_ledger_check`, appended after the transcript helpers.
  - `.claude/hooks/enforce-zed-ledger.sh` (new, executable): outside-repo, owned, ignored and
    listed paths pass; anything else exits 2. A path with a `..` segment skips both the owned
    and the ignored checks and passes only on an exact row.
  - `.claude/settings.json`: the hook in the `Write|Edit` matcher after `enforce-phase-gate.sh`
    (timeout 10 s); the `_comment` names it.
  - `script/gates.sh`: `zed_ledger_g` after gate:14's function, `run_gate "gate:16 zed-ledger"`
    after gate:14's line, the header's gate list.
  - `CONSTITUTION.md`: §0's list line and scoping bullet, §14's second upstream-discipline
    bullet, §21 point 2, worded as the design fixed them.
  - `docs/marley/zed-touchpoints.md`: the enforcement paragraph (live; `marley_owned_path` is
    the authority; how a row is recognized).
  - `docs/planning/knowledge/prevention-rules.md`: the superseded note on
    `PR-claude-diff-gate-mutation-tests-touched-shim-fn-001`.
- **Deviations.**
  - The `..` handling is stricter than the design's sentence: the design skipped only the owned
    check for a `..` path, but the ignored check has the same hole (`target/../crates/...` can
    read as a path under the ignored `target/`), so both are skipped.
  - `gates.sh`'s coverage comment still said `transport.rs` is "verified on the live wire";
    since #443 its loopback tests drive it, and the comment now says why the file stays on
    the exclude list. A comment only, in a file this ticket already edits.
- **Checks.** `bash -n` and shellcheck (`-S info -e SC1091`, as gate:11 runs it) clean over every
  hook and the gate; `jq empty .claude/settings.json` clean; `zed_ledger_check 78648aaf7d…` on
  the real tree: "every touchpoint recorded (6 rows)", exit 0; a quick hook smoke blocks
  `crates/paths/src/paths.rs` (2) and passes `crates/marley_mcp/src/lib.rs` and `README.md` (0).
  No `.rs` changed, so no cargo ran.
- **Phase 3 checklist:** helpers ✓ · hook ✓ · wiring ✓ · gate:16 ✓ · constitution ✓ · ledger
  prose ✓ · superseded note ✓ · checks ✓.

## Inspect (Phase 3.5)
Two critics, run in parallel with the pipefail and gate-integrity ledger entries as input:
(A) the write hook's path logic and bypasses, and (B) gate:16's correctness, the pipefail
traps and whether the docs tell the truth. Both reproduced every finding with commands in
throwaway worktrees, removed afterwards; neither ran cargo. I re-checked each one before a
verdict.

| # | Finding (critic) | Verdict | Fix |
|---|---|---|---|
| 1 | high: the hook compared the path with the git root of the session's working directory, so from `/`, a scratchpad, another repo or a linked worktree every Zed write passed; Claude Code runs hooks in the session's current directory, which follows `cd` and EnterWorktree (A, B) | real | the hook resolves the file's own checkout: `git -C <nearest existing dir> rev-parse --show-toplevel/--show-prefix`, and gates it only when its `--git-common-dir` is this repository's, against that checkout's own ledger |
| 2 | medium: nothing checked the ledger at commit time: a commit without `.rs` needs no receipt, the receipt fingerprints neither the ledger nor a non-Rust Zed file, and gate:16 read the work tree while a commit ships the index; three docs claimed gate:16 "catches a change made any other way" (B) | real | `enforce-commit-gate.sh` runs `zed_ledger_check` at every `git commit`, before its `.rs` early exit; the check now lists the index as well as the work tree and untracked files; the docs say what runs when |
| 3 | medium: git quoted non-ASCII, `"` and tab names, so the gate reported the quoted form missing and the real row stale, with no way to fix it (A, B) | real | `-z` on every git listing (`diff`, `diff --cached`, `ls-files`, `ls-tree`) |
| 4 | medium: `.claude/worktrees/<name>/` with a `/` in the name broke `normalize_path`'s one-segment strip, so owned and listed paths in such a worktree were blocked (A) | real | the hook no longer uses `normalize_path` (finding 1's resolution) |
| 5 | medium: the §21 amendment never reached `.claude/commands/pipeline/complete.md`, which still asked for "a short note under `docs/marley/`", and §21 called the row a Phase-5 step while the hook demands it before the Phase-3 write (B) | real | `complete.md` step 1(b) and §21: Phase 5 confirms the row still describes what shipped; the row is written before the change (§14) |
| 6 | low: a symlinked directory inside an owned path let a Write reach a Zed file (A) | real | git reports the physical directory, so the path resolves to the real file (smoke: `crates/marley_mcp/zassets -> ../../assets` blocks) |
| 7 | low: writes under `.git/` were blocked (A) | real | inside a `.git` directory there is no work tree, so the hook exits 0 |
| 8 | low: nothing kept the owned set disjoint from upstream, so a future upstream `CHANGELOG.md` or `docs/specs/` would be exempt silently (A, B) | real | gate:16 reports any file of the upstream base that `marley_owned_path` claims (4,314 base paths, about 90 ms). A glob widened over paths upstream does not have (a new file inside a Zed crate) still passes; changing the owned set is an edit to an enforcer, which the amendment rule governs |
| 9 | low: `.claude/settings.json`'s `_comment` said every gate arms on pipeline intent, but the commit, changelog, reference and ledger hooks are always on (B) | real, pre-existing and sharpened by this ticket | comment corrected |
| 10 | nit: duplicate rows and rows for owned paths passed, and a renamed `## Touchpoints` heading read as "every path missing" (A, B) | real | reported by name: duplicates, owned rows, and a missing section |
| 11 | nit: `check-ignore` read a leading `:/` as pathspec magic (A) | real | `./$REL` |
| 12 | nit: a newline in a path could pose as two adjacent rows (A) | real | such a path is blocked |
| 13 | nit: `zed_ledger_check` relied on its caller's pipefail; `gates.sh`'s fork-point comment no longer said who uses it; §0 said an invalid `MARLEY_UPSTREAM_BASE` falls back, where it fails closed; ledger rule 4 did not say to revert the hunk before removing its row (B) | real | the listings run in their own `set -o pipefail` subshells; `upstream_base` moved into `lib-hook-helpers.sh` (one definition for the gate and the commit hook); §0 and rule 4 reworded |
| 14 | nit: commit one would carry `gates.sh`'s `transport.rs` coverage comment, which is not part of the amendment; the amendment rule says "hook" where `gates.sh` is a gate (B) | real | the comment hunk goes in commit two; the rule now says "any hook or gate that enforces the changed rule" |
| 15 | pre-existing: `normalize_path`'s `\+` is a GNU sed extension, so on macOS the worktree prefix is never stripped and the phase gate misreads worktree paths (A) | real | `\{1,\}`, the POSIX interval, the same on GNU and BSD sed |
| 16 | the `Write|Edit` matcher misses MultiEdit and NotebookEdit; `jq` missing exits 0 (A) | not new: the phase gate has the same matcher and every hook the same `jq` rule; NotebookEdit sends `notebook_path` and the tree has no notebooks | none |
| 17 | parser edges: a `###` table inside Touchpoints counts as rows; an indented row is not a row (A) | accepted: both fail closed (a stale row, or a path reported missing), and D3 defines a row as a backticked path opening the first column | none |
| 18 | my review: the design said the hook goes live when `settings.json` is saved; I doubted it, thinking Claude Code snapshots hooks at startup | tested: a real Edit to `docs/README.md` in this session was blocked with the ledger message, and the file is unchanged. The design was right | none |

- **Verification of the fixes.** `scratchpad/w0-smoke/run-ledger-smokes.sh`: 57 checks, 0
  failures, covering every row above that has behavior (the ledger check in a throwaway
  worktree; the commit hook with a commit command built at run time; the write hook from `/`,
  the scratchpad, the worktree and the repo). shellcheck at gate:11's settings and `bash -n`
  clean over every hook and the gate.
- **Phase 3.5 checklist:** spawn critics ✓ · review findings ✓ · fix confirmed ✓ · verify fixes ✓
  · write inspect ledger ✓.

## Phase 4 — Validate
Gate-is-test (§7): no `.rs` changed, so the proof is exit codes and negative smokes.
- **Smokes** (`scratchpad/w0-smoke/run-ledger-smokes.sh`, rerun after the last hook-library edit):
  57 checks, 0 failures, in a throwaway worktree that was removed afterwards (`git worktree
  list` shows only the main tree).
- **Negative through the real gate (17:43:02 to 17:43:39).** A Bash append to `docs/README.md`
  (the write hook never sees it), then `script/gates.sh --fast`: gate:16 FAIL with
  "changed outside the Marley-owned paths with no row: docs/README.md", the other eleven
  static gates PASS, GATE RED, exit 1. The file was restored with `git checkout`.
- **Green (17:43:45 to 17:44:20).** `script/gates.sh --fast` on the clean tree: 12 of 12, gate:16
  "every touchpoint recorded (6 rows)", 271 tests passed, GATE GREEN [fast]. W0 carries no
  `.rs`, so no receipt is needed for either commit.
- **Live harness.** An Edit to `docs/README.md` in this session was blocked by
  `enforce-zed-ledger.sh` with the ledger message (the file is unchanged): the hook runs in the
  real Claude Code harness, not only on crafted JSON.

| REQ | Proof |
|---|---|
| 001 | smokes: an unlisted edit, a new file, a deletion, a rename (both paths) and a staged-only change are red and named; the real gate's gate:16 FAIL above |
| 002 | smoke: a row for the untouched `docs/AGENTS.md` is red and named |
| 003 | the green `--fast` run, gate:16 PASS with 6 rows |
| 004 | smokes: seven blocked forms (absolute, relative, two `..` forms, the `CLAUDE.md` symlink, `:/` magic, a newline) plus the live Edit probe |
| 005 | smokes: owned, listed, ignored (`.mcp.json`), outside (a scratchpad file, another repo), inside `.git/`, a new Marley crate, and an input without `file_path`, all exit 0 |
| 006 | the owned globs appear once (`lib-hook-helpers.sh:212`); `gates.sh`, the write hook and the commit hook call the shared functions |
| 007 | CONSTITUTION §0 lines 42 and 100-108, §14 lines 221-228, §21 line 365-368 |
| 008 | gate:11 PASS in both `--fast` runs |
| 009 | smokes: the commit hook refuses a non-Rust Zed change without a row (exit 2, path named), allows it with the row, allows a clean tree |
| 010 | smokes: blocked from `/`, the scratchpad, the worktree and the repo; a row in the worktree's own ledger allows that worktree's file while the main checkout still blocks |

- **Live drive.** N/A: no UI (hooks, a gate, constitution text).
- **Pre-existing failures.** None.
- **Phase 4 checklist:** smokes ✓ · negative gate ✓ · green gate ✓ · live harness probe ✓ · REQ
  table ✓.

## Phase 5 — Complete
- **Documentation (§21).** `CHANGELOG.md`: an `### Added` entry (the ledger, enforced at the
  write, in gate:16 and at every commit; `upstream_base` shared) and a `### Fixed` entry
  (`normalize_path` on macOS). `docs/marley/workbench-shell.md`'s status line says W0 shipped.
  The ledger's own prose (`docs/marley/zed-touchpoints.md`) was updated in Phase 3 and at
  inspect. No Marley crate changed, so no per-crate note; no path outside the owned set
  changed, so the six rows still describe what shipped (gate:16 green, 6 rows).
  `three-prong-plan.md` needs no change: the shell comes before the prongs and its slices are
  tracked in `workbench-shell.md`.
- **Knowledge (§19).** Inspect appended `F-claude-436-a-a-path-hook-judged-files-by-the-sessions-directory-001`,
  `F-claude-436-b-a-gate-only-rule-was-not-checked-at-commit-001`,
  `PR-claude-a-path-hook-resolves-the-files-own-repo-001` and
  `PR-claude-a-rule-the-gate-checks-is-also-checked-at-commit-001`, plus the superseded note on
  `PR-claude-diff-gate-mutation-tests-touched-shim-fn-001`. This phase appended
  `AD-claude-436-one-owned-set-three-enforcers-001`,
  `L-claude-436-a-new-hook-is-live-in-the-same-session-001` and
  `L-claude-436-no-taskcreate-means-no-mid-phase-stop-001`.
- **Brain.** Consultation `2ef0d62346e94648bda185ffe1b08068` closed with `brain decide`:
  `decisions/marley-forks-zed-touchpoint-ledger-one-owned-set-enforced-at-the-write-the-gate-and-the-commit`,
  follow-up due 2026-10-22 (the first upstream merge: did the ledger make it cheap?).
- **Commits.** Commit one, the amendment: `CONSTITUTION.md` with its enforcers
  (`lib-hook-helpers.sh`, `enforce-zed-ledger.sh`, `enforce-commit-gate.sh`, `settings.json`,
  `script/gates.sh` without its `transport.rs` comment hunk, and `complete.md`). Commit two:
  the rest (the comment hunk, the ledger prose, the knowledge, the CHANGELOG, the paperwork).
- **Ticket.** TICKET-436 closed; its backlog row left at promotion.
- **Phase 5 checklist:** CHANGELOG + architecture docs ✓ · capture knowledge ✓ · close ticket ✓
  · archive pipeline ✓.
