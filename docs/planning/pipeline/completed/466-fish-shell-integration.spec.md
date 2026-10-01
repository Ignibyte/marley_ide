---
pipeline_id: 79c17310-dd59-4df5-b31e-9338d71573df
ticket: docs/planning/tickets/open/TICKET-466-fish-shell-integration.md
status: Phase 4 — Complete PASS
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
- `for_program`'s `fish` arm: `XDG_DATA_DIRS` with Marley's folder first, then the user's own
  value or the system default (`/usr/local/share:/usr/share`) when it was unset, and the user's own
  value in `MARLEY_FISH_DATA_DIRS`; `for_program` takes the inherited `XDG_DATA_DIRS` as it takes
  `ZDOTDIR`, and the terminal's Marley hunk passes it (the terminal's environment, else Marley's).
- The script: it puts the user's `XDG_DATA_DIRS` back (or unsets it) and takes the nonce and the
  other Marley variables out of the environment before the user's files run; `init`, `history`
  and `bootstrapped` at the first prompt; `precmd` (exit, pwd) on `fish_prompt`, its exit from
  `fish_postexec`'s `$status`; `preexec` (the command without a leading space, from
  `fish_preexec`'s `$argv[1]`); `history;file=` with fish's history file (`$XDG_DATA_HOME` or
  `~/.local/share`, `fish/<$fish_history or fish>_history`).
- `marley_terminal::parse_history` reads fish's history (`- cmd: ` lines, with fish's `\n` and
  `\\` escapes), so the history file feeds #484's ghost text and #625's completions.
- `script/gates.sh` gate:11 checks the script with `fish --no-execute`, since shellcheck cannot
  read fish.

### Out (explicitly deferred)
- OSC 133 (fish 4 prints its own; Marley reads its DCS frames), Marley's `ssh` function and the
  `ssh_bootstrap` for fish on a remote host, and the leading-space history rule (#553), which fish
  already keeps: a line starting with a space never enters fish's history.

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
- **Re-verified at promotion (2026-10-01)** on this box's fish 4.9.2: a snippet in
  `<dir>/fish/vendor_conf.d/` with `<dir>` first on `XDG_DATA_DIRS` runs before `config.fish`, and
  a variable it erases is gone when `config.fish` runs; `fish_preexec` gets the line,
  `fish_postexec` the status (`echo hi` 0, `false` 1), and an empty Enter fires `fish_prompt`
  alone; `printf '\033Pq…\033\\\\'` in fish's single quotes prints `ESC P q … ESC \`. fish 4
  queries the terminal at start (`ESC P +q`, XTGETTCAP), which `marley_dcs` passes through, since
  `+` is not one of its selectors. The terminal's Marley hunk (`marley_shell_integration`) reads
  `ZDOTDIR` from the terminal's environment, else Marley's, and does the same for
  `XDG_DATA_DIRS`.

## UI proof
`script/e2e/466-fish-shell-integration.sh`: `terminal.shell` set to fish; `HOME`,
`XDG_CONFIG_HOME` and `XDG_DATA_HOME` in the scenario's folder (this box sets `XDG_CONFIG_HOME`, so
fish would read the user's own files otherwise); a `config.fish` that prints a marker and the
nonce's presence; a history file holding `cargo test --workspace`.
- `marker.png`: the marker printed, so the user's file ran;
- `block.png`: `echo hi` typed, a finished block with exit 0 and output `hi`, its gutter and pill;
- `failed.png`: `false`, a block with the failed pill;
- `nonce.png`: `printenv MARLEY_SHELL_NONCE; echo $status` printing `1`, and `echo
  $XDG_DATA_DIRS` printing the value Marley's own environment had;
- `history.png`: `cargo t` and Tab in the prompt editor, the history's `cargo test --workspace` in
  the completion menu (#625), from fish's history file; the grid's ghost text reads the same
  history, but fish draws its own suggestion in the grid, so the editor shows Marley's.

## Locked-In Decisions
- D1 — `vendor_conf.d` through `XDG_DATA_DIRS`, fish's own place for a vendor's startup code.
- D2 — The nonce and Marley's data-dir entry leave the environment before the user's files run,
  as bash's and zsh's scripts do.
- D3 — The user's `XDG_DATA_DIRS` travels in `MARLEY_FISH_DATA_DIRS`, as zsh's `ZDOTDIR` travels in
  `MARLEY_ZSH_ZDOTDIR`, so the script restores it exactly, or unsets it.
- D4 — The hooks are event handlers defined in the vendor snippet, so they come before any the
  user's `config.fish` defines, and Marley's `precmd` frame ends a block's output before another
  handler prints.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user types `echo hi` in a fish terminal, Marley shall show a finished block with exit 0 and output `hi`. | Shot `block.png` |
| REQ-002 | WHEN a command fails, its block shall carry the failed pill. | Shot `failed.png` |
| REQ-003 | WHEN fish starts, the user's `config.fish` shall still run, and the nonce shall be gone from the environment. | Shots `marker.png`, `nonce.png` |
| REQ-004 | WHEN fish starts, `XDG_DATA_DIRS` shall hold the value it would have held without Marley. | Shot `nonce.png` |
| REQ-005 | WHEN a typed line starts a command in fish's history file, Marley's completions shall offer it. | Shot `history.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the script, the arm, the install path, the gate's check; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
