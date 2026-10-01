# Shell integration for fish — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-466-fish-shell-integration.md
- **Pipeline spec:** 466-fish-shell-integration.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`); Chad, 2026-09-30, on installing fish: "Yes, install fish".
- **Batch:** wave 3, third batch (#628 to #630, and #466).
- **Recall (§18.3):**
  - #463 and #465 left fish, OSC 133 and remote bootstraps for later; the scripts send DCS frames.
  - fish 4.9.2 installed on 2026-09-30 (`omarchy pkg add fish`), recorded in the ops handbook.
  - No gate runs the PTY tests since #483; #475 is deferred, so the scenario is the proof.
- **Discovery:** one Explore sweep (2026-09-30) over the scripts, `install_in`, `for_program` and
  the tests; the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-10-01)
- **Recall (§18.3):**
  - AD-claude-465: a shell's own startup mechanism hands the user's files back (`ZDOTDIR`); the
    hooks go in at the first prompt so Marley's precmd runs first. fish's equivalent is
    `vendor_conf.d` through `XDG_DATA_DIRS`, and event handlers defined there come first.
  - L-claude-463-proving-a-shell-script-before-wiring-it-001: prove the script under the real
    shell on a PTY first. Done for the events (`script -qfc 'fish -i'`), with one wrinkle: fish 4
    waits for answers to its terminal queries (DA1, a cursor report), which `script` never gives,
    so the run had to feed them.
  - L-claude-465-zsh-reads-an-unquoted-replacement-by-context-001: quoting differs by shell; the
    fish script's escapes were checked by `od`.
  - Brain (consultation 7f636b2988ba44e482eb087fe56146b0): nothing on this seam.

### Design
- **`marley.fish`** (new, `crates/marley_terminal/shell_integration/`): restores the user's
  `XDG_DATA_DIRS` from `MARLEY_FISH_DATA_DIRS` or unsets it; moves `MARLEY_SHELL_NONCE` into a
  global and erases it, and erases `MARLEY_SSH_COMMAND` and `MARLEY_AGENT_HISTORY`. In an
  interactive fish, `__marley_quote` escapes a value as `marley_dcs` reads it (backslash first,
  then `;`, tab, CR and ESC per line, the lines joined by a literal `\n`); `fish_postexec` keeps
  `$status`; `fish_prompt` prints `init` (`$fish_pid`), `history` and `bootstrapped` once, then
  `precmd` with the kept status and `$PWD`; `fish_preexec` prints `preexec` with the line less a
  leading space.
- **`shell_integration.rs`:** `FISH_INTEGRATION`, `FISH_DIR` (`fish/vendor_conf.d`), `FISH_FILE`,
  `FISH_DATA_DIRS_VARIABLE` (`MARLEY_FISH_DATA_DIRS`); `install_in` writes the script;
  `for_program` gains `user_data_dirs: Option<&str>` and a `fish` arm (env only, no arguments);
  the module doc, `shown_arguments` and the tests pass `None`.
- **`suggest.rs`:** `parse_history` reads a fish history (its first line `- cmd: `), each `- cmd: `
  line's command unescaped.
- **`crates/terminal/src/terminal.rs`** (Zed, Marley's own hunk): `marley_shell_integration` reads
  the inherited `XDG_DATA_DIRS` as it reads `ZDOTDIR` and passes it.
- **`script/gates.sh`:** gate:11 also runs `fish --no-execute` on the script.

### File manifest
- Marley: `crates/marley_terminal/shell_integration/marley.fish` (new),
  `crates/marley_terminal/src/shell_integration.rs`, `crates/marley_terminal/src/suggest.rs`;
  `script/gates.sh`; `script/e2e/466-fish-shell-integration.sh`.
- Zed: `crates/terminal/src/terminal.rs` (the row widens first).

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-003 | fish starts; `config.fish` prints `config ran` and whether the nonce was set | `marker.png` |
| REQ-001 | `echo hi` | `block.png` |
| REQ-002 | `false` | `failed.png` |
| REQ-003, REQ-004 | `printenv MARLEY_SHELL_NONCE; echo $status`, `echo $XDG_DATA_DIRS` | `nonce.png` |
| REQ-005 | `cargo t` typed | `ghost.png` |
Prompt editor: #627 docks the editor at fish's prompt like bash's; the scenario types in it, so it
also shows fish's prompt is seen.

### Risks and decisions
- fish 4's startup queries need a terminal that answers them; Zed's terminal answers DA1 and the
  cursor report, so fish starts without its timeout. The scenario proves it.
- A user `fish_prompt` handler defined later runs after Marley's; one defined in a vendor snippet
  sorted before `marley.fish` runs first. Rare; the frame then follows that handler's output.

## Phase 2 — Code (2026-10-01)
- **Built:** `marley.fish`; `FISH_INTEGRATION`, `FISH_DIR`, `FISH_FILE`, `FISH_DATA_DIRS_VARIABLE`,
  `install_in` writing the snippet, `for_program`'s `user_data_dirs` and `fish` arm;
  `parse_history` reading fish's history (`fish_unescape`); the terminal hunk passing the
  inherited `XDG_DATA_DIRS`; gate:11 running `fish --no-execute` on the script.
- **Proved before the Rust** (L-claude-463): the script on a PTY under fish 4.9.2, feeding fish's
  terminal queries their answers: `init`, `history`, `bootstrapped`, `precmd` with `exit=0`, then
  `exit=1` after ` false` (its leading space trimmed in `preexec`), a value with `;` and `\`
  escaped as `marley_dcs` reads it; `config.fish` ran with the nonce gone and `XDG_DATA_DIRS` the
  user's.
- **Deviation:** the test of programs with no integration listed `fish`; it lists `nu` now. No
  test was added (§7).
- **Review:** the user's `XDG_DATA_DIRS` is read like `ZDOTDIR`, from the terminal's environment,
  else Marley's; an empty value is put back as it was. A comment in `gates.sh` that began with
  "shellcheck" read as a directive and broke the parse; it is reworded.
- **Clippy found:** two identical match arms in `fish_unescape`, merged.
- **Gate:** GREEN, 17 PASS.

## Phase 3 — Test (2026-10-01)
- **Scenario:** `script/e2e/466-fish-shell-integration.sh` (sway): `terminal.shell` fish, `HOME`,
  `XDG_CONFIG_HOME` and `XDG_DATA_HOME` in the scenario's folder; one run, green.
- **Deviation:** REQ-005 is shown through the prompt editor's completions rather than the grid's
  ghost text: fish draws its own suggestion in the grid, so a ghost there would not tell whose it
  was; the scenario also turns fish's off. The completion menu is Marley's alone.
- **Shots:**
  - `marker` (REQ-003): `config.fish ran; the nonce is gone` above fish's `$ ` prompt; the tab and
    the rail read `repo — fish`; the prompt editor docked at fish's prompt, so `precmd` was seen.
  - `block` (REQ-001): `$ echo hi` and `hi` in a block with its gutter and check pill; the rail's
    `echo hi · done · 0 s`.
  - `failed` (REQ-002): `$ false` with the red gutter and the `exit 1` pill; the rail's
    `false · exit 1`.
  - `nonce` (REQ-003, REQ-004): `printenv MARLEY_SHELL_NONCE; echo nonce status $status` printed
    `nonce status 1`; `echo data dirs $XDG_DATA_DIRS` printed `/usr/local/share:/usr/share`, this
    box's own value, with Marley's folder gone.
  - `history` (REQ-005): `cargo t` and Tab: the menu lists `cargo test --workspace`, which only
    fish's history file holds.
- **Seen, not in scope:** `history.png` also shows #557's hint after `cargo t`: `cargo` is not on
  the search path of the Marley the harness starts, so the rules read the word as English. #557's
  rule, unchanged here.

## Phase 4 — Complete (2026-10-01)
- **Documented:** `CHANGELOG.md`; the guide (the shell integration section, the no-blocks entry,
  the waiting list); `terminal_blocks.md`; the plan's T0 row; the ledger row of
  `crates/terminal/src/terminal.rs`.
- **Knowledge:** L-claude-466-fish-4-waits-for-its-terminal-queries-001,
  L-claude-466-a-bash-comment-that-starts-with-shellcheck-is-a-directive-001,
  AD-claude-466-fish-loads-marleys-hooks-as-a-vendor-snippet-through-xdg-data-dirs-001. No bug
  reached the Test phase.
- **Brain:** the consultation closed with `brain decide`.
