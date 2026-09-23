# just for Marley's workflow — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-471-just-for-the-workflow.md
- **Pipeline spec:** 471-just-for-the-workflow.spec.md

## Phase 1 — Plan (2026-09-23)
- **Request (Chad, verbatim):** "when given a chance we should install just and use it in
  replacement (or augment) the workflow where needed."
- **Found:** `just` was not installed; Arch's `extra` has 1.58.0; sudo is passwordless on the
  box. `enforce-tests-ran.sh` finds a run by `cargo (test|nextest|llvm-cov)` or
  `script/gates.sh` at a command position, so a `just` recipe needs adding there.
  `enforce-commit-gate.sh` reads the receipt, which `gates.sh` writes however it is started.
- **Design:** the recipes above; `script/live-shot.sh` from the scratch `rail-shot.sh`, with
  its paths from the environment (`CARGO_TARGET_DIR`, `XDG_*`, `TMPDIR`) and its profile copy
  in a `mktemp` directory it removes.
- Brain: not consulted; tooling that wraps existing commands, no design decision beyond D1.

## Phase 2 — Code (2026-09-23)
- **Installed:** `sudo pacman -S --needed just`: 1.58.0 from `extra`.
- **Built.**
  - `justfile`: `default` (the list), `idle`, `gate-diff`, `gate-fast`, `gate-full`, `build`,
    `test`, `clippy`, `fmt` (these three over named crates through `prepend("-p ", crates)`) and
    `shot`; every cargo recipe depends on `idle`; `set shell` to `bash -euo pipefail -c`.
  - `script/live-shot.sh`: the scratch `rail-shot.sh` generalized: paths from `CARGO_TARGET_DIR`,
    `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `TMPDIR`; the profile copy in a `mktemp` directory
    removed on exit; the Hyprland signature found with `find` (shellcheck's SC2012 on `ls`); a
    clear failure when the binary is missing, a Marley window is already open, or none maps in
    90 seconds.
  - `enforce-tests-ran.sh` counts `just gate-diff|gate-fast|gate-full` and `just test` at a
    command position.
  - `lib-hook-helpers.sh`: `justfile` and `script/live-shot.sh` join the Marley-owned paths,
    as `script/gates.sh` and `script/mutation.sh` are, and the ledger's list says so;
    `enforce-zed-ledger.sh` had refused both as Zed's paths.
  - gate:11 lints `script/live-shot.sh`; CONSTITUTION's tools and gate:11 row, and the Test
    phase's steps, name the recipes.
- **Deviation:** the Marley-owned list grew; the plan had not foreseen that the root and
  `script/` are Zed's.
- **Review:** `shot`'s doc is one line because `just` shows only the last comment line before a
  recipe; the capture refuses to start a second Marley, which the dev channel would allow.

## Phase 3 — Test (2026-09-23)
- **REQ-001.** `just --list` names all ten recipes, each with its line.
- **REQ-002.** `just idle` started while another project's `cargo test` (toolchain 1.94.0) ran on
  the box, and returned 665 s later, when that run ended and no cargo was left.
- **REQ-003.** `enforce-tests-ran.sh` fed a transcript whose `/pipeline:test` was followed by one
  Bash command: allowed for `just gate-diff`, `just gate-fast`, `just test marley_rail
  marley_workbench`, `cd … && just gate-full` and `script/gates.sh --diff`; blocked for
  `just build`, `just --list`, `echo just gate-diff`, `just gate-diffx` and `just testing`.
- **REQ-004.** `just shot just-471` wrote the PNG (the rail and a terminal at its prompt), left
  no `profile.*` directory, closed Marley, and the reload restored the window rules.
- **gate:11** (`shellcheck -S info -e SC1091` over its list, the new script included): clean.
- **Gate:** `just gate-diff`: `GATE GREEN [diff]`, 20 passed, the receipt matching the tree.
  With no crate touched, gate:4 skips cleanly.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Added); CONSTITUTION (the tools line, gate:11's row); the Test
  phase's steps (`.claude/commands/pipeline/test.md`); the ledger's Marley-owned list.
- **Ledger (§19).** None: the recipes wrap commands the ledger already records.
- **Ticket** closed; the pipeline archived; one commit.
