# TICKET-471 — `just` for Marley's workflow

- **Ticket:** LOCAL #471 (chore, the workflow)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/471-just-for-the-workflow.spec.md
- **Source ticket:** Chad, 2026-09-23: "when given a chance we should install just and use it in
  replacement (or augment) the workflow where needed"
- **Status:** closed

## Summary
Install `just` on the dev box (Arch's `extra/just`, 1.58.0) and give the repository a `justfile`
whose recipes run the commands the pipeline repeats: the gate in each mode, building the debug
`marley`, a crate's tests, clippy over the touched crates, and the live capture on a hidden
workspace. `script/gates.sh` stays the gate and its receipt: the recipes call it, and the hooks
that read the transcript for a real gate or test run (`enforce-tests-ran.sh`) learn the recipes'
names. The pipeline skills name the recipes where they now spell the commands out.

## Acceptance
`just --list` names each recipe with a one-line description; each recipe runs the command it
wraps, one cargo at a time; a Test phase that runs the gate through `just` satisfies the hooks
as a direct run does; the diff gate is green.
