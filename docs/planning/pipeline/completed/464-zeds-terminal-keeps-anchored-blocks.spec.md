---
pipeline_id: 93ea0724-9957-47db-9f7a-56591d638c2a
ticket: docs/planning/tickets/closed/TICKET-464-zeds-terminal-keeps-anchored-blocks.md
status: Phase 4 — Complete PASS
title: Zed's terminal keeps anchored blocks
type: feature
slice: prong 1, T0b (Zed's half; the event loop's half was #462)
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/462-shell-hooks-from-the-event-loop.spec.md]
---

## Title
The vendored event loop reports each Marley shell hook with its absolute line (#462), and Zed's
terminal ignores it. Now the terminal decodes each hook and keeps the terminal's commands as
blocks: ranges of its one scrollback, anchored by absolute line (the plan's D2). Each block has
its command, state, exit code and prompt, and its output is read from the grid while the lines
are still held.

## Scope
### In
- **The model** (`marley_terminal`, a new pure module `anchored.rs`): `AnchoredBlocks::apply(hook,
  line)` and `AnchoredBlock`:
  - `InitShell` registers the shell and forgets a staged prompt;
  - `Precmd` finishes the running block at its line with its exit code, and stages the next
    prompt's metadata and line;
  - `Preexec` opens a running block whose output starts at its line, with the staged prompt,
    and finishes any block still running, without an exit code;
  - `Bootstrapped` changes nothing here;
  - a `Preexec` or `Precmd` before any `InitShell` is refused, as the gpui era's model refused
    it.
- **Zed's `Terminal`** (`crates/terminal`):
  - `process_event` decodes each `ShellHook` (`marley_terminal::decode_frame`) and applies it
    at `position.absolute_line()`, skipping hooks that arrive on the alternate screen;
  - `blocks()`;
  - `block_output(&block)`: the block's lines from the grid, through the eviction counter,
    while they are held; `None` once they are gone.
- **Exports:** `marley_terminal` re-exports `marley_dcs::RawDcs`, and the root
  `[workspace.dependencies]` gains `marley_terminal`.

### Out (explicitly deferred)
- Drawing blocks (T1).
- The shell scripts (#463); the tests print the frames themselves.
- Reflow on resize (the plan's D2 weak spot).
- Retiring `marley_terminal`'s own PTY engine (`session.rs`, `pty_os.rs`).

## Reference (§20)
N/A — Marley-specific: the plan's D2 block model over Zed's terminal. No Warp code is read; the
behavior map records only that blocks come from shell hooks
(`docs/warp_architecture/subsystems/00-overview.md:44`).

### Prior art
- **Behavior maps:** as above.
- **Published material:** none.
- **Code we already ship.**
  - `marley_terminal`'s `SessionModel::apply_hook` (`apply.rs:100-145`), whose transitions
    this model keeps, less the gpui era's run tags and copied output.
  - alacritty's `Term::bounds_to_string`, which joins lines and drops the final newline.
  - #462's `HookPosition::absolute_line` and `Grid::evicted_lines`.
  - Zed's PTY test helpers `build_test_terminal_with_arguments` and the polling pattern of
    `assert_content_eventually`.

## UI proof
N/A — no UI delta: blocks are kept and queryable, and T1 draws them. The acceptance tests run a
real shell on a real PTY.

## Locked-In Decisions
- D1 — The anchored model is new and pure, beside the gpui era's `SessionModel`, which copies
  output for its own engine.
- D2 — A block is `prompt_line`, `output_start` and `output_end`, absolute lines from the hooks.
  Its output text is read from the grid on demand, never stored.
- D3 — Hooks on the alternate screen are skipped: that grid keeps no history, and its lines are
  not the scrollback's.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a shell prints Marley's hook frames around `echo hi`, the terminal shall hold a Finished block with exit 0 whose output is "hi" | PTY test |
| REQ-002 | WHEN one write carries the whole command, frames and output together, the same block shall result | PTY test |
| REQ-003 | The model shall apply each hook as the scope lists, refusing hooks before `InitShell` | unit tests (`marley_terminal`) |
| REQ-004 | WHEN a block's lines have left the scrollback, `block_output` shall return `None` | unit test of the mapping over a `Term` with a small history (task terminals keep the maximum history) |
| REQ-005 | The diff gate shall be green | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — `anchored.rs`, the exports, Zed's `Terminal` (ledger rows first); fmt and
  clippy.
- **P3 Test** — REQ-001 to REQ-005, with negative checks.
- **P4 Complete** — CHANGELOG and docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
