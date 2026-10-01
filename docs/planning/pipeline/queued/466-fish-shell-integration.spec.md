---
pipeline_id: 79c17310-dd59-4df5-b31e-9338d71573df
ticket: docs/planning/tickets/open/TICKET-466-fish-shell-integration.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Shell integration for fish"
type: feature
slice: prong 1 T0c, the third shell
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/463-bash-shell-integration.spec.md, docs/planning/pipeline/completed/465-zsh-shell-integration.spec.md]
---

## Title
fish gets Marley's shell integration as bash (#463) and zsh (#465) have it: commands become blocks,
with the prompt's folder, the exit status, the history file and the nonce. fish 4.9.2 is on the
dev box since 2026-09-30.

## Scope
### In
- `crates/marley_terminal/shell_integration/marley.fish`, embedded with `include_str!` and written
  by `install_in` to `<data dir>/shell_integration/fish/vendor_conf.d/marley.fish`.
- `for_program`'s `fish` arm: `XDG_DATA_DIRS` with Marley's folder first (and the system default,
  `/usr/local/share:/usr/share`, when it was unset), and the nonce.
- The script: it takes Marley's entry out of `XDG_DATA_DIRS` and the nonce out of the environment
  before the user's files run; `init` and `bootstrapped` at the first prompt; `precmd` (exit,
  pwd) on `fish_prompt`, its exit from `fish_postexec`'s `$status`; `preexec` (the command, from
  `fish_preexec`'s `$argv[1]`); `history;file=` with fish's history file.
- `script/gates.sh` gate:11 checks the script with `fish --no-execute`, since shellcheck cannot
  read fish.

### Out (explicitly deferred)
- OSC 133, the `ssh_bootstrap` for fish on a remote host, and #484's history parser reading fish's
  YAML history (the terminal's own verified commands still feed the ghost text).

## Reference (§20)
- **Upstream Zed:** the terminal's shell spawn (`crates/terminal/src/terminal.rs`), where
  `for_program` is called, kept.
- **Warp (behavior):** a shell's hooks frame each command as a block
  (`docs/warp_architecture/subsystems/03-terminal-session-core.md`).

### Prior art
- **Behavior maps:** the note above; plan D5.
- **Published material:** fish's documentation: `fish_preexec`, `fish_postexec` and `fish_prompt`
  events, and `vendor_conf.d` under `XDG_DATA_DIRS`.
- **Code we already ship:** `marley.bash` and `marley.zsh` and their frames; `install_in` and
  `for_program` (`shell_integration.rs`); the PTY tests for bash and zsh (`terminal.rs`,
  `finished_block_of`), which no gate runs (#483, #475).

## UI proof
`script/e2e/466-fish-shell-integration.sh`: `terminal.shell` set to fish; a `config.fish` that
prints a marker.
- `marker.png`: the marker printed, so the user's file ran;
- `block.png`: `echo hi` typed, a finished block with exit 0 and output `hi`, its gutter and pill;
- `failed.png`: `false`, a block with the failed pill;
- `nonce.png`: `printenv MARLEY_SHELL_NONCE; echo $status` printing `1`.

## Locked-In Decisions
- D1 — `vendor_conf.d` through `XDG_DATA_DIRS`, fish's own place for a vendor's startup code.
- D2 — The nonce and Marley's data-dir entry leave the environment before the user's files run,
  as bash's and zsh's scripts do.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user types `echo hi` in a fish terminal, Marley shall show a finished block with exit 0 and output `hi`. | Shot `block.png` |
| REQ-002 | WHEN a command fails, its block shall carry the failed pill. | Shot `failed.png` |
| REQ-003 | WHEN fish starts, the user's `config.fish` shall still run, and the nonce shall be gone from the environment. | Shots `marker.png`, `nonce.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the script, the arm, the install path, the gate's check; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
