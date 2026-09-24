# A new terminal opens at the last terminal's size — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-485-starship-echo-lands-off-the-line.md
- **Pipeline spec:** 485-starship-echo-lands-off-the-line.spec.md

## Phase 1 — Plan (2026-09-23)
- **Found by** #483's first e2e runs: with the user's starship prompt on a long path, only the
  first typed character showed, the cursor sat about 18 columns left of the line's end, and bash
  still ran the whole command; Ctrl-L redrew the line right.
- **Recall.** The brain (consultation 2ff4dd85d4334ee0b71ca277ab8e1bdc) returned only unrelated
  follow-ups; nothing on this seam in the ledger.
- **Probes** (e2e scenarios in the scratchpad, and bash in a plain pty):
  - A plain `$ `, a two-line `\n$ `, a lone `❯ ` and a 140-column plain prompt all echoed right
    in Marley.
  - Bash with the user's `.bashrc` (mise, starship, zoxide, fzf, `bind -f inputrc`), alone and
    with Marley's `--rcfile`, in a plain pty sized 321 × 65 before it started: every key echoed
    as one byte.
  - The user's shell under `script` inside Marley: the header recorded the PTY's first size,
    100 × 6; after the first prompt came readline's SIGWINCH redraw (`\r\e[K\r\e[A\e[K\r` and
    the prompt), and then, for each key, `\r`, the whole prompt, 25, 24, 23… backspaces and the
    key: readline drawing each key against a layout made for 100 columns.
  - An external command at the end of the rcfile, so `checkwinsize` rereads the size, changed
    nothing: the resize came after readline had begun the line.
  - A second terminal opened from the palette was misdrawn too (in the hidden workspace,
    where frames come slowly).
- **Root cause.** `TerminalBuilder::new` opens every PTY at `TerminalBounds::default()`, 100 ×
  6, and the view resizes it at its first layout. When the shell reaches its first prompt first,
  readline lays the prompt out for 100 columns; after the resize, `redraw_prompt` redraws a
  multi-line prompt's last line and restores that old layout, and every key after is drawn
  against it. It takes a multi-line prompt (starship's `add_newline`), escapes marked invisible,
  a last line wider than 100 columns, and readline set up before the resize (the rc's `bind`).
- **Design.** Remember the size `set_size` last gave a PTY terminal (a process-wide
  `parking_lot::Mutex`), and open the grid, the PTY and `last_content` at it. The first
  terminals of a launch still open at the debug size: keeping the size across launches is
  TICKET-486.

### E2E plan
| REQ | Shot |
|---|---|
| 001 | `485-02-second-terminal`: `echo second` whole on the prompt line, cursor after it |
| 002 | `485-01-first-terminal`: still misdrawn (the limit) |
| 003 | `just gate-diff` |

## Phase 2 — Code (2026-09-23)
- **Built** (`crates/terminal/src/terminal.rs`, a `// Marley:` comment on each hunk; its ledger
  row extended): `MARLEY_LAST_BOUNDS`; `set_size` stores the normalized bounds for a PTY
  terminal; `TerminalBuilder::new`'s future reads them once as `marley_bounds` for `new_term`,
  `open_pty` and `last_content`, so the view's first `set_size` at the same size resizes nothing.
- **Review.** Display-only terminals (the agent panel's cards) never set the size. A terminal
  opened in a pane of another size still starts at the last one and is resized as before, so
  the gain is for terminals opened where the last one was laid out, which is the common case.

## Phase 3 — Test (2026-09-23)
- **Before the fix** (`485-long-prompt.sh` on the build without it): both terminals showed
  `probe ❯ e`, the cursor among the dashes; after Enter, `second` printed: bash had every key.
- **After** (`SHOT_DIR=<scratchpad>/e2e485c just e2e script/e2e/485-long-prompt.sh`; focus
  report: "the user's window and workspace are as they were"):
  - `485-01-first-terminal`: still `probe ❯ e` with the cursor among the dashes: the first
    terminal of the launch opened before any view had a size (REQ-002, TICKET-486).
  - `485-02-second-terminal`: `probe ❯ echo second`, the cursor after it (REQ-001).
  - `485-03-second-ran`: `second` printed and a new prompt.
- **Gate.** The first `just gate-diff` was red on gate:1 alone (the `new_term` call fits one line
  now); after `cargo fmt -p terminal`, `GATE GREEN [diff]`, receipt matching.

## Phase 4 — Complete (2026-09-23)
- **Docs.** CHANGELOG (Fixed: typing on a long prompt in a new terminal); the
  `crates/terminal/src/terminal.rs` row in `docs/marley/zed-touchpoints.md` now names the
  last-size hunks and when to drop them. TICKET-486 carries the limit.
- **Knowledge.** F-claude-485-a-shell-started-at-the-debug-size-misdrew-its-first-long-prompt-001,
  L-claude-485-log-the-bytes-a-shell-writes-inside-marley-with-script-001.
- **Brain.** Consultation 2ff4dd85d4334ee0b71ca277ab8e1bdc closed with
  `a-new-marley-terminal-opens-at-the-last-terminals-size`, follow-up by 2026-10-07.
