# Marley docs

Marley is the Zed fork (`Ignibyte/marley_ide`). Zed's own user manual lives in `docs/src/`
(mdBook). Everything else under `docs/` is Marley's: the forward plan, the workflow's
planning tree, and the design record carried over from the gpui-era app on 2026-09-18.

| Folder | What it holds |
|---|---|
| **`marley/`** | The fork's own record: [workbench-shell.md](workbench-shell.md) (the Warp-style shell, first in line), [three-prong-plan.md](three-prong-plan.md) (the forward plan), [zed-touchpoints.md](zed-touchpoints.md) (every change outside Marley-owned paths, read at each upstream merge), and `history/` (the gpui-era changelog). |
| **`vendor/`** (at the root) | The upstream crates Marley changes and so carries, built through `[patch]` and kept out of Zed's workspace: `alacritty_terminal` at Zed's rev (#461). [`vendor/README.md`](../../vendor/README.md) records each copy's source, what is left out, Marley's hunks, and the re-sync steps. |
| **`planning/`** | The workflow's working tree (CONSTITUTION §3/§19): `tickets/` (open, closed, `BACKLOG.md`), `pipeline/` (`queued/`, `active/`, `completed/`, `parked/`, templates), `knowledge/` (the four append-only ledgers recall greps), `intake/`, `design-notes/`. History from the gpui era continues here unbroken. |
| **`marley_architecture/`** | The gpui-era architecture record: per-crate notes (the ones for the ported crates still describe them), the fleet control plane, the orchestration shell, the embedded-browser model, the old roadmap. Paths and crate names in these docs refer to the old app; read them as design, not as a map of this tree. The exceptions are the notes for crates written in the fork, [`marley_rail`](../marley_architecture/marley_rail.md) and [`marley_workbench`](../marley_architecture/marley_workbench.md), which describe this tree. |
| **`specs/`** | The EARS specifications of the gpui-era crates and the binding standards (`standards/quality-bar.spec.md`, superseded by CONSTITUTION §0 for the fork; `standards/seam-contracts.md`, still the owner map for the Marley crates' shared types). |
| **`warp_architecture/`** | The Warp deconstruction: the clean-room behavior source for the terminal, blocks and cockpit (CONSTITUTION §20). Observed captures go in `observed/`. |
| **`zed_architecture/`** | Marley's deconstruction of Zed, written before the fork. Zed's source is now in this tree and is the authority; these notes remain a useful orientation map and the prior-art leg that names Zed's design. |
| **`decisions/`** | ADR-style decisions: licensing and ownership, the Chromium embed feasibility study. |
| **`tickets/`** | The gpui-era M0 ticket manifest, history. |

## The workflow, in one line

Plan → Code → Test → Complete: `/pipeline:plan` → `/pipeline:code` → `/pipeline:test` →
`/pipeline:complete`, governed by `CONSTITUTION.md`, enforced by `.claude/hooks/`, gated by
`script/gates.sh`. `/spec` drafts a sprint of Phase-1 specs into `planning/pipeline/queued/`.

## Standards

The bar is CONSTITUTION §0: Zed's own gates for Zed's crates; the full Marley bar (rustal's
lint table, 100% line coverage and the meta-gates) for `crates/marley_*`. Mutation testing runs once at the end of a
sprint (`script/mutation.sh`), not per change. Prose follows the
`no-ai-slop` skill. Changes inside Zed's crates are additive and small so upstream merges
stay cheap.
