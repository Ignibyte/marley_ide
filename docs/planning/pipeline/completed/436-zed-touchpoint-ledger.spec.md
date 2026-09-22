---
pipeline_id: d2a4f467-26e9-4f96-a4bd-0b2cb1403962
ticket: docs/planning/tickets/closed/TICKET-436-zed-touchpoint-ledger.md
status: Phase 5 — Complete PASS
title: The Zed touchpoint ledger, enforced (gate:16 + a write hook)
type: chore
slice: workbench shell W0
references: [docs/marley/workbench-shell.md, docs/marley/zed-touchpoints.md, docs/planning/design-notes/workbench-shell-shelf.md]
---

## Title
Make Chad's rule mechanical: every path where the fork differs from upstream Zed outside
Marley-owned paths has a row in `docs/marley/zed-touchpoints.md`. The gate fails without the
row, a hook blocks the write before it happens, and the constitution names the ledger.

## Scope
### In
- `.claude/hooks/lib-hook-helpers.sh`: one shared definition of the Marley-owned path set
  (`marley_owned_path <path>`) and the ledger lookup (`zed_ledger_names <path>`), sourced by
  both the gate and the hook.
- `script/gates.sh`: gate:16 `zed-ledger`, a static gate (runs in FAST, DIFF and FULL). It
  lists the paths that differ from `upstream_base` (tracked changes plus untracked,
  non-ignored files), drops Marley-owned paths, and fails on any remaining path the ledger's
  Touchpoints table does not name, and on any row whose path no longer differs.
- `.claude/hooks/enforce-zed-ledger.sh` (PreToolUse, `Write|Edit`): blocks a write to a repo
  path outside the Marley-owned set when the ledger does not name it; wired in
  `.claude/settings.json`.
- `.claude/hooks/enforce-commit-gate.sh`: the same check at every `git commit`, Rust or not
  (added at inspect: the receipt covers neither the ledger nor a non-Rust Zed file).
- `CONSTITUTION.md`: §0 lists gate:16; §14's upstream discipline names the ledger and the
  hook; §21 point 2 replaces "a short note under `docs/marley/`" with the ledger row. Its own
  commit, per the amendment rule.
- `docs/marley/zed-touchpoints.md`: the enforcement paragraph says the gate and hook are live
  and that `lib-hook-helpers.sh` is the authority for the owned set.

### Out (explicitly deferred)
- Checking that each code hunk carries its `// Marley:` comment (a review item; a later
  ratchet if misses show up).
- Edits made by Bash (`sed -i`, heredocs) are not seen by the hook; gate:16 catches them.

## Reference (§20)
N/A — Marley-specific: this is the fork's own merge-hygiene tooling. Neither Warp nor Zed has
a behavior it reimplements; upstream Zed has no ledger of downstream divergence.

### Prior art
- **Behavior maps:** none apply (no Warp or Zed behavior involved).
- **Published material:** long-lived forks keep their divergence as an explicit list: a
  patch series (Debian's quilt `debian/patches/series`) or a per-file patch directory
  (Brave keeps its Chromium changes as `patches/*.patch`). The ledger is the same idea kept
  as a table, because Marley's changes are tiny additive hunks rather than patch files.
- **Code we already ship:** `script/gates.sh` already has `upstream_base()` (merge-base with
  `upstream/main`, fallback `78648aaf7d`) and the `run_gate` harness; `lib-hook-helpers.sh`
  has `normalize_path` and the sourcing pattern every hook uses;
  `enforce-phase-gate.sh` shows the path-classification shape (`case` globs). git owns the
  path listing (`git diff --name-only <base>`, `git ls-files --others --exclude-standard`),
  so no new tool is needed. Zed's `script/` has nothing for downstream forks (checked).

## UI proof
N/A — no UI delta: a gate, a hook and constitution text; nothing the app renders or reads.

## Locked-In Decisions
- D1 — The owned-path set is defined once, in `lib-hook-helpers.sh`, and both enforcers
  source it; the ledger's prose list is documentation of that function.
- D2 — gate:16 is static: it costs one `git diff` and runs in every mode, including `--fast`.
- D3 — The ledger's rows are recognized by a backticked path in the first column of the
  Touchpoints table; paths elsewhere in the doc do not count as rows.
- D4 — The hook is always on (like the commit hooks), not scoped to pipeline sessions,
  because the rule applies to every change.
- D5 — The constitution change ships in a commit that touches only `CONSTITUTION.md` and its
  enforcers (the amendment rule).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a path outside the Marley-owned set differs from the upstream base and the ledger has no row for it, gate:16 shall fail and print that path | negative smoke: append a comment to an untouched Zed file, run `script/gates.sh --fast`, red naming the path; revert, green |
| REQ-002 | WHEN a ledger row names a path that does not differ from the upstream base, gate:16 shall fail and print that row's path | negative smoke: add a row for an untouched path, red; remove it, green |
| REQ-003 | WHEN every differing non-owned path has a row and every row's path differs, gate:16 shall pass | `script/gates.sh --fast` exit 0 on the current tree |
| REQ-004 | WHEN a Write or Edit targets a repo path outside the Marley-owned set that the ledger does not name, the hook shall exit 2 with a message naming the ledger | hook smoke: crafted PreToolUse JSON on stdin, exit code 2 |
| REQ-005 | WHEN a Write or Edit targets a Marley-owned path, a path the ledger names, or a path outside the repo, the hook shall exit 0 | hook smoke: three crafted inputs, exit 0 each |
| REQ-006 | The Marley-owned set shall have exactly one definition, shared by gate:16 and the hook | review: both source `marley_owned_path` from `lib-hook-helpers.sh` |
| REQ-007 | CONSTITUTION §0 shall list gate:16, and §14 and §21 shall name `docs/marley/zed-touchpoints.md` | grep the three sections |
| REQ-008 | gate:11 (shellcheck) shall pass over the new hook and the gate changes | `script/gates.sh --fast` gate:11 PASS |
| REQ-009 | WHEN a `git commit` is attempted while a changed path outside the Marley-owned set has no ledger row, `enforce-commit-gate.sh` shall block it and name the path, whether or not the commit carries Rust | hook smoke in a throwaway worktree: a non-Rust Zed edit, exit 2; with its row, exit 0 |
| REQ-010 | The write hook shall judge a path by the checkout the file lives in, whatever the session's working directory, and a linked worktree by its own ledger | hook smoke from `/`, the scratchpad, a worktree and the repo; a row in a worktree's ledger allows that worktree only |

## Phase Plan
- **P2 Design** — the helper signatures, the gate's diff listing (tracked and untracked, rename
  handling), the table parse, the hook's path normalization; the smoke list.
- **P3 Implement** — helper, gate:16, hook, settings wiring, constitution text, ledger prose.
- **P3.5 Inspect** — independent critics vs the diff (correctness of path matching, pipefail
  traps per `PR-claude-no-quiet-grep-tail-in-pipefail-hooks-001`, bypasses); fix the real
  findings.
- **P4 Validate** — the negative smokes (gate-is-test change, §7), `script/gates.sh --fast`
  green.
- **P5 Complete** — CHANGELOG, ledger capture, close the ticket, archive.
