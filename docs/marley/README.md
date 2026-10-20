# Marley docs

Marley is the Zed fork (`Ignibyte/marley_ide`). Zed's own user manual lives in `docs/src/`
(mdBook). Everything else under `docs/` is Marley's: the forward plan, the workflow's
planning tree, and the design record carried over from the gpui-era app on 2026-09-18.

| Folder | What it holds |
|---|---|
| **`marley/`** | The fork's own record: [guide.md](guide.md) (everything Marley adds and how to use it, the user's reference), [walkthrough.md](walkthrough.md) (a hands-on tour of the whole IDE, stop by stop, with what each stop should show), [tutorial-outline.md](tutorial-outline.md) (the lesson-by-lesson tutorial, outlined), [workbench-shell.md](workbench-shell.md) (the plan for the Warp-style shell, the Marley layout and its rail, built first), [look-plan.md](look-plan.md) (the plan that builds the 2026-10-20 design canvas: the cat theme, the shrinking rail, project logos, tmux Sessions, the bottom row, forge, and every screen restyled, in seven waves), [three-prong-plan.md](three-prong-plan.md) (the forward plan), [rusty-in-marley.md](rusty-in-marley.md) (the plan that rebuilt Rusty's app as Marley's panels and tabs), [browser-handoff.md](browser-handoff.md) (the 2026-09-24 handoff that gathered the record on the embedded browser), [fleet-contract.md](fleet-contract.md) (the outputs Marley reads to show agents, their work and their hosts; a draft), [mutation-runs.md](mutation-runs.md) (the mutation pass, on this machine or on GitHub's runners), [zed-touchpoints.md](zed-touchpoints.md) (every change outside Marley-owned paths, read at each upstream merge), and `history/` (the gpui-era changelog). |
| **`marley/borrowing.md`** | The ongoing list of projects Marley was compared with (Zed, Warp, Orca, T3 Code, herdr and Hermes, MonoCode, plannotator, the ASD-STE100 skill): each survey, what came of it, and what is still open. |
| **`vendor/`** (at the root) | The upstream crates Marley changes and so carries, built through `[patch]` and kept out of Zed's workspace: `alacritty_terminal` at Zed's rev (#461). [`vendor/README.md`](../../vendor/README.md) records each copy's source, what is left out, Marley's hunks, and the re-sync steps. |
| **`planning/`** | The workflow's working tree (CONSTITUTION §3/§19): `tickets/` (open, closed, `BACKLOG.md`), `pipeline/` (`queued/`, `active/`, `completed/`, `parked/`, templates), `knowledge/` (the four append-only ledgers recall greps), `intake/`, `design-notes/`. History from the gpui era continues here unbroken. |
| **`marley_architecture/`** | The gpui-era architecture record: per-crate notes (the ones for the ported crates still describe them), the fleet control plane, the orchestration shell, the embedded-browser model, the old roadmap. Paths and crate names in these docs refer to the old app; read them as design, not as a map of this tree. The exceptions are the notes for crates written in the fork, [`marley_dcs`](../marley_architecture/marley_dcs.md), [`marley_rail`](../marley_architecture/marley_rail.md), [`marley_rusty`](../marley_architecture/marley_rusty.md) (#643), [`marley_sdk`](../marley_architecture/marley_sdk.md) (#607), [`marley_system_one`](../marley_architecture/marley_system_one.md) (#565) and [`marley_workbench`](../marley_architecture/marley_workbench.md), which describe this tree. |
| **`specs/`** | The EARS specifications of the gpui-era crates and the binding standards (`standards/quality-bar.spec.md`, superseded by CONSTITUTION §0 for the fork; `standards/seam-contracts.md`, still the owner map for the Marley crates' shared types). |
| **`warp_architecture/`** | The Warp deconstruction: the clean-room behavior source for the terminal, blocks and cockpit (CONSTITUTION §20). Observed captures go in `observed/`. |
| **`orca_architecture/`** | The 2026-09-25 survey of Orca (`stablyai/orca`, MIT), an agent IDE of worktrees, a browser, review and a phone companion: seven reports by area and a summary of what Marley takes from it. Its source is checked out at `/srv/stacks/orca-refs/orca` and, being MIT, may be read and adopted with its notice kept. |
| **`t3code_architecture/`** | The 2026-10-06 survey of T3 Code (`pingdotgg/t3code`, MIT), a chat front end that runs Claude Code, Codex and others headless with worktrees, checkpoints, a pull request client, a preview browser and phone apps: seven reports by area, a ranked list of what Marley lacks, and open questions. Being MIT, its source may be read and adopted with its notice kept. |
| **`zed_architecture/`** | Marley's deconstruction of Zed, written before the fork. Zed's source is now in this tree and is the authority; these notes remain a useful orientation map and the prior-art leg that names Zed's design. |
| **`decisions/`** | ADR-style decisions: licensing and ownership, the Chromium embed feasibility study. |
| **`tickets/`** | The gpui-era M0 ticket manifest, history. |

## The workflow, in one line

Plan → Code → Test → Complete: `/pipeline:plan` → `/pipeline:code` → `/pipeline:test` →
`/pipeline:complete`, governed by `CONSTITUTION.md`, enforced by `.claude/hooks/`, gated by
`script/gates.sh`; the Test phase is a visual check of the change, with no unit tests (§7). `/spec` drafts a sprint of Phase-1 specs into `planning/pipeline/queued/`.

## Standards

The bar is CONSTITUTION §0: Zed's own fmt and clippy for Zed's crates; the full Marley bar
(rustal's lint table, Zed's dylint lints and the meta-gates) for `crates/marley_*`. No gate
runs tests: a change is proven by its e2e visualization test, a scripted run of the real
Marley shot step by step (§7, `script/e2e.sh`), on hidden workspace 9 with keys only, or in a
headless sway of its own when the scenario clicks (#487). Prose follows the
`no-ai-slop` skill. Changes inside Zed's crates are additive and small so upstream merges
stay cheap.

## Running Marley

- `just build` makes the debug `marley` in the shared target directory
  (`/mnt/fast/target/debug/marley` on the dev box). The e2e scenarios run it.
- `just install` (#502) builds `marley` in the release profile and installs it under
  `~/.local` (`--prefix DIR` for another place): the binary `lib/marley/marley`, its launcher
  `bin/marley`, the desktop entry `share/applications/marley.desktop` (which claims `marley://`
  links, #762) and an icon, so the menu lists Marley. It also builds Marley's remote server for
  Linux x86_64 hosts and stages it under `~/.local/share/marley/remote_servers/` (#445), which
  needs `musl-gcc` and `llvm-objcopy`; `script/install-marley --remote-server-only` stages the
  server alone. Run it again after a pull; a running Marley keeps its old binary until it
  restarts. The launcher appends stderr to `~/.local/share/marley/logs/stderr.log` when stderr
  is not a terminal: on the `dev` channel a panic goes to stderr and nowhere else.
- Both builds are on the `dev` channel and share `~/.config/marley` and `~/.local/share/marley`.
  Since #513 a second Marley on the same data directory hands its paths to the one that runs,
  through `marley-dev.sock` in the data directory (#760), and exits; `--user-data-dir DIR` runs
  one beside it on a directory of its own. Marley's windows have the class
  `com.ignibyte.Marley` (#760).
- The guide's "Install and run" section has the details, and the walkthrough's Part 0 checks an
  install stop by stop.
