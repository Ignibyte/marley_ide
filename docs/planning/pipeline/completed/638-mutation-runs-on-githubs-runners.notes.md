# Mutation runs on GitHub's runners — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-638-mutation-runs-on-githubs-runners.md
- **Pipeline spec:** 638-mutation-runs-on-githubs-runners.spec.md

## Phase 1 — Plan (minted 2026-10-01)
- **Request:** Chad, 2026-10-01, with #637, #631 and #540: "for mutations we may run these soon
  but I want to set up the ability to run these on high cpu usage quick on the cloud because it
  takes hours to run these."
- **Classification / tier:** chore, tooling; no Rust. Two new files outside the Marley-owned set
  (`script/mutants`, `.github/workflows/marley_mutants.yml`), each with a touchpoint row.
- **Checklist:** pick ✓, pre-flight ✓ (no active pipeline; cargo busy with rustal-os's mutation
  pass in its own folders), recall ✓, mint ✓, prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - PR-claude-parallel-mutation-workers-get-their-own-target-001 and F-claude-443: copy mode,
    a target per copy; `CARGO_TARGET_DIR` cleared for the run.
  - PR-claude-a-workspace-mutation-run-names-its-packages-001: `-p` for every package, or
    cargo-mutants mutates `default-members` only.
  - L-claude-636: a run reads only the mutated crate's tests; read survivors per file.
  - Brain (`rusty-cli brain ask`, consultation 15c6cfffefa14c1e80576604c1d5fd8d): nothing on this
    seam.
- **Discovery (a research agent, 2026-10-01):** no cloud CLI on the box; RunPod is the only cloud
  key; `gh` is logged in with `repo` and `workflow`; the fork is public, Actions enabled at the
  repository level with 0 workflows registered and 0 runs ever, which is GitHub's default for a
  fork until its owner turns workflows on in the Actions tab; the Ignibyte organization is on the
  free plan (standard runners only). The nine pure cores need no gpui and no system package beyond
  a C compiler; `rust-toolchain.toml` pins 1.98.1, which rustup installs at the first cargo.
  cargo-mutants 27.1.0 has `--shard K/N` and `--sharding round-robin|slice`.

### Design
- **`script/mutants`** (bash, `set -uo pipefail`, from the repository root): `run`, `report`,
  `cloud`, `cloud-setup`, as the spec says. `run` keeps the retired script's flags and cleared
  variables, adds `--output`, `--shard … --sharding round-robin`. `report` is an embedded Python
  over every `mutants.out/outcomes.json` under the folders given. `cloud` makes a commit on HEAD's
  tree with `shards=`, `jobs=`, `packages=` lines (`git commit-tree`, nothing in the work tree
  changes), pushes it to `refs/heads/mutants/<stamp>`, sets a trap that deletes that branch on
  exit, finds the run by the workflow's name on the branch (a workflow that never ran has no id),
  `gh run watch`, `gh run download --name mutants-report`. `cloud-setup` lists the workflows
  through the API and disables each active one but Marley's.
- **The workflow:** `plan` (the settings, bounded: 1-64 shards, 1-8 jobs, package names of
  `[a-z0-9_ ]`), `shard` (matrix, `fail-fast: false`, 120 minutes), `report` (`if: always()`,
  merges what came back, writes the job summary). `permissions: contents: read`.
- **Recipes:** `mutants *args` → `script/mutants run {{args}}`, `mutants-cloud *args` →
  `script/mutants cloud {{args}}`.
- **Doc:** `docs/marley/mutation-runs.md`, linked from `docs/marley/README.md`.
- **File manifest:** Marley-owned: `justfile`, `docs/marley/mutation-runs.md`,
  `docs/marley/README.md`. Outside the owned set (rows first): `script/mutants`,
  `.github/workflows/marley_mutants.yml`.

### Visual check plan
- `just shot` (the hook's run; nothing in Marley changes).
- REQ-001, REQ-002: `script/mutants run --shard 0/2 --packages marley_fleet --output <scratch>/a`
  and `1/2` into `b`; `script/mutants report a b` totals against
  `cargo mutants --list -p marley_fleet`.
- REQ-003: `script/mutants report` over #636's outcomes (its scratchpad) against the design note.
- REQ-004: `script/mutants cloud --shards 2 --packages marley_fleet` now; then `git ls-remote
  origin 'refs/heads/mutants/*'` is empty.
- REQ-005: `actionlint .github/workflows/marley_mutants.yml`.
- REQ-006: not reachable until Actions are on for the fork (the owner's switch).

### Risks
- **Zed's workflows.** Turning Actions on for the fork turns on Zed's 47 workflows too; until
  `cloud-setup` runs, a push to the fork can start them. The doc puts the two steps together.
- **The public run.** The workflow's logs and the report are public, as the code is.
- **A runner's disk.** Two copies of the tree and their targets for the nine crates fit a
  standard runner's free space; the gpui crates may not.

## Phase 2 — Code (2026-10-01)
- **Built:** `script/mutants` (`run`, `report`, `cloud`, `cloud-setup`);
  `.github/workflows/marley_mutants.yml` (`plan`, `shard`, `report`); `just mutants` and
  `just mutants-cloud`; `docs/marley/mutation-runs.md`, linked from `docs/marley/README.md`; the
  two touchpoint rows.
- **Deviations:**
  - Outcomes and reports default to `~/.cache/marley-mutants` (`run-<stamp>`, `cloud-<stamp>`),
    beside the copies, not the working folder: the review found the first draft wrote
    `mutants-run/` into the repository, which no ignore rule covers.
  - `cloud` finds its run by the workflow's name on the branch, not by the file name: a workflow
    that has never run has no id to ask by.
  - `report` lists the runs that left no outcomes instead of failing on them, so a shard that died
    leaves the rest's report.
- **Review:** `run` keeps the retired script's cleared variables and exit codes, with `-p` for
  each package (PR-claude-a-workspace-mutation-run-names-its-packages-001) and copies in their own
  targets (PR-claude-parallel-mutation-workers-get-their-own-target-001); `cloud` changes nothing
  in the work tree (`git commit-tree` on HEAD's tree) and deletes its branch from an `EXIT` trap,
  so a failure or Ctrl-C leaves none; the workflow bounds what it reads from the commit (1-64
  shards, 1-8 jobs, package names of `[a-z0-9_ ]`), so a message cannot inject a command, and runs
  with `contents: read`. `shellcheck`, `actionlint` and `typos` clean.
- **Gate:** `script/gates.sh --fast` GATE GREEN (16 of 16; no Rust, so no receipt).

## Phase 3 — Test (2026-10-01)
- **`just shot 638-mutants`** (the hook's run): Marley started on Hyprland's hidden workspace and
  drew its window, the trust question over the scratch project; the user's window and workspace
  as they were. Nothing in Marley changed, and nothing it needs broke.
- **REQ-001:** `script/mutants run --shard 0/2 --packages marley_fleet` and `--shard 1/2`, each
  into its own folder in the scratchpad: 19 mutants each (17 caught and 2 unviable, 16 and 3),
  both baselines green, exit 0, copies under `~/.cache/marley-mutants`.
- **REQ-002:** `cargo mutants --list --no-config -p marley_fleet` lists 38; the two shards' 19 and
  19 share none and together are the 38; `script/mutants report` over both prints
  `marley_fleet | 38 | 33 | 0 | 0 | 5 | 100.0%`, #636's row.
- **REQ-003:** `script/mutants report` over #636's outcomes prints the design note's table row for
  row (2,658 mutants, 961 caught, 12 timeouts, 1,448 missed, 237 unviable, 40.2%) and lists the
  1,448 missed.
- **REQ-004:** `script/mutants cloud --shards 2 --packages marley_fleet` against the fork as it is:
  it pushed `mutants/20261001-184836`, found no run in two minutes, said so and exited 1 in
  2 min 15 s; `git ls-remote origin 'refs/heads/mutants/*'` then listed nothing, the trap had
  deleted the branch. The message now points at `docs/marley/mutation-runs.md`; shellcheck and the
  fast gate ran again after that edit (GATE GREEN).
- **REQ-005:** `actionlint .github/workflows/marley_mutants.yml` exit 0.
- **REQ-006: not reachable yet.** A run on the runners needs Actions on for the fork, a switch in
  its Actions tab only the owner sets (GitHub keeps a fork's workflows off until then); recorded
  in the doc's first-run steps for Chad.

## Phase 4 — Complete (2026-10-01)
- **Docs (§21):** `CHANGELOG.md` (Added); `docs/marley/mutation-runs.md` (new) and its link in
  `docs/marley/README.md`; `docs/planning/design-notes/mutation-run-2026-10.md` points at the new
  commands. Touchpoint rows checked: `script/mutants` and `.github/workflows/marley_mutants.yml`
  describe what shipped. No slice of the plan owns the testing tooling.
- **Ledger:** AD-claude-638-the-mutation-pass-runs-in-shards-on-githubs-free-runners-001,
  L-claude-638-a-forks-actions-stay-off-until-its-owner-turns-them-on-001. No bug.
- **Brain:** `brain decide` on consultation 15c6cfffefa14c1e80576604c1d5fd8d
  (`decisions/marleys-mutation-pass-runs-in-shards-on-githubs-free-runners`).
- **Ticket:** closed; archived to `completed/`; committed on `marley/workbench-shell`. The first
  run on the runners waits on Chad turning Actions on for the fork (the doc's two steps).

