---
pipeline_id: ba2763f1-0e0f-4548-8cd2-342984ffc09a
ticket: docs/planning/tickets/closed/TICKET-638-mutation-runs-on-githubs-runners.md
status: Phase 4 — Complete PASS
title: "Mutation runs on GitHub's runners"
type: chore
slice: the testing tooling (the end-of-product testing phase, CONSTITUTION §7)
references: [docs/planning/design-notes/mutation-run-2026-10.md]
---

## Title
The mutation pass, run by hand, locally or split over GitHub's free runners. #636's pass over the
nine pure cores took 1 h 27 min on the dev box with two workers; Chad, 2026-10-01: "I want to set
up the ability to run these on high cpu usage quick on the cloud because it takes hours to run
these". GitHub gives a public repository's Actions standard 4-vCPU runners at no cost, up to 20
jobs at once, and the fork is public; a run split into 16 shards takes 16 runners at once.

## Scope
### In
- `script/mutants run [--shard K/N] [--jobs J] [--packages "…"] [--output DIR]`: the pass on this
  machine or one shard of it (cargo-mutants' `--shard K/N --sharding round-robin`), as #636 ran
  it: copy mode, a target per copy (#443), `-p` for each package, `--no-config`, nextest; the nine
  pure cores unless packages are named.
- `script/mutants report DIR...`: one markdown report over one or more runs' or shards'
  outcomes: #636's table by crate (killed of viable), the runs that left no outcomes, and every
  missed mutant.
- `script/mutants cloud [--shards N] …`: pushes HEAD, with the run's settings in a commit on top,
  to a `mutants/<stamp>` branch of the origin; waits for the workflow's run, watches it, downloads
  its report into DIR, prints the table, and deletes the branch whatever happens.
- `script/mutants cloud-setup`: once Actions are on for the fork, turns off every workflow but
  Marley's, the 47 the fork carries from Zed.
- `.github/workflows/marley_mutants.yml`: on a push to `mutants/**`, a plan job reads the settings
  from the commit, a matrix runs the shards (each: the toolchain the repository pins,
  cargo-mutants 27.1.0 and nextest, `cargo fetch --locked`, `script/mutants run --shard`), and a
  report job merges them into the run's summary and an artifact.
- `docs/marley/mutation-runs.md`: how to run it, locally and on the runners, and the one switch
  that is the owner's (Actions on for the fork).
- `just mutants` and `just mutants-cloud`.

### Out (explicitly deferred)
- The two gpui crates (`marley_browser`, `marley_workbench`): each of their mutants builds most of
  Zed; they need a bigger machine or many more shards, timed first. `--packages` names them.
- A machine of our own in a cloud (RunPod holds the only key on the box; a provider is Chad's
  call, and a run on GitHub's runners costs nothing).
- Killing the survivors: the testing phase's (`intake/kill-mutation-survivors.md`).
- Turning Actions on for the fork: a switch in the repository's Actions tab only its owner sets.

## Reference (§20)
N/A — Marley-specific: the workflow's own tooling, with no Warp or Zed behavior to match. Zed's
CI (`.github/workflows/` upstream) is not reused: it builds and tests Zed, not a mutation pass.

### Prior art
- **Behavior maps:** none (tooling).
- **Published material:** cargo-mutants' book (sharding: `--shard k/n`, `--sharding
  round-robin|slice`, "Sharding" and "Continuous integration" chapters, which show the same
  matrix shape); GitHub's Actions limits for public repositories (standard runners, 20 concurrent
  jobs on the free plan); `taiki-e/install-action` for prebuilt cargo tools.
- **Code we already ship:** the retired `script/mutation.sh` (`git show bffb491dc9^:script/mutation.sh`:
  the flags, the cleared variables, the exit codes), #636's run (the same flags with `--jobs 2`
  and `--output`), its `outcomes.json` shape, and #636's design note's table.

## UI proof
N/A — no UI delta: tooling for the mutation pass, nothing in Marley changes. Its run is
`just shot`; the tooling's own checks are a local shard run, the report over #636's outcomes,
`actionlint`, `shellcheck`, and a cloud run against the fork as it is.

## Locked-In Decisions
- D1 — GitHub's runners over a rented machine: free for a public repository, no new account, 16
  runners at once; the run is public, as the code is.
- D2 — Round-robin shards, so `marley_terminal`'s 1,012 mutants spread over every runner.
- D3 — The run starts from a pushed branch, not `workflow_dispatch`: a dispatch needs the
  workflow on the default branch, `main`, which mirrors Zed's; a push runs the workflow the pushed
  commit holds. The settings ride in that commit's message.
- D4 — Zed's workflows are turned off through the API (`cloud-setup`), not edited: no Zed file
  changes, and a later merge cannot turn them on.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `script/mutants run --shard K/N --packages P` runs, the system shall mutate only that shard of P's mutants, in copies with targets of their own, and leave its outcomes in the output folder | local runs `--shard 0/2` and `1/2` of `marley_fleet`: their outcome counts |
| REQ-002 | WHEN shards' outcomes are reported together, the system shall count each mutant once | the two shards' report against `cargo mutants --list -p marley_fleet` |
| REQ-003 | WHEN `script/mutants report` reads #636's outcomes, the system shall print #636's table | the report's table against the design note's |
| REQ-004 | WHEN `script/mutants cloud` finds no run for its branch within two minutes, the system shall say Actions may be off and delete the branch it pushed | a cloud run against the fork as it is (Actions off); `git ls-remote` after |
| REQ-005 | The workflow shall be valid for GitHub Actions | `actionlint` exit 0 |
| REQ-006 | WHILE Actions are on for the fork, a cloud run shall run its shards at once and bring back one report | not reachable before the owner turns Actions on; recorded |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the touchpoint rows; `script/mutants`; the workflow; the recipes; the doc; review;
  `script/gates.sh --fast` (no Rust).
- **P3 Test** — `just shot`, then the checks in the UI proof.
- **P4 Complete** — CHANGELOG, the doc map, ledger, close, archive, commit.
