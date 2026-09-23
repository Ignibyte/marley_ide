---
pipeline_id: c3b25f04-bc54-4b03-8088-f35102d9f4fc
ticket: docs/planning/tickets/closed/TICKET-462-shell-hooks-from-the-event-loop.md
status: Phase 4 — Complete PASS
title: Shell hooks from alacritty's event loop
type: feature
slice: prong 1, T0b (the event loop's half; Zed's half is #464)
references: [docs/marley/three-prong-plan.md, vendor/README.md, docs/planning/knowledge/prevention-rules.md]
---

## Title
Marley's shell hooks travel in the PTY stream as DCS frames (`ESC P <selector> <payload> ESC
\`), and vte drops every DCS unread. The vendored event loop now takes Marley's frames out of
each read before the parser sees it. At each complete frame it reports the hook with the grid
position where it fell.

## Scope
### In
- **`marley_dcs`**, a new leaf crate (MIT OR Apache-2.0, no dependencies): Marley's DCS
  scanner, moved from `marley_terminal::dcs` and made public, with three fixes to how it treats
  bytes that are not Marley's:
  - a DCS whose selector is not Marley's (`h`, `p`, `q`) passes through as it arrived;
  - CAN and SUB, which cancel a control string, pass the partial frame through, so the parser
    cancels it as it always did;
  - a payload past a size cap passes through, so a stray `ESC P` cannot hide the rest of the
    output.
- **`marley_terminal`** keeps its decoder and takes the scanner from `marley_dcs`.
- **The vendored `alacritty_terminal`**, every hunk marked `Marley:` and listed in
  `vendor/README.md`:
  - a new `src/marley_hooks.rs`: `ShellHook { final_byte, payload, position }`, `HookPosition`
    (lines evicted so far, history size, cursor line and column, alt screen) and
    `advance_with_hooks`, which parses each passthrough run and, at each hook, snapshots the
    position and sends `Event::ShellHook`, in stream order;
  - `event_loop.rs`: the scanner in `State`, and `advance_with_hooks` in place of
    `parser.advance` in `pty_read`;
  - `event.rs`: the `ShellHook` variant and its `Debug` arm;
  - `grid/mod.rs`: `evicted_lines`, a monotonic count of lines dropped off the top of the grid
    (`scroll_up` with full history, `clear_history`, `update_history`), with
    `#[serde(default)]`;
  - `Cargo.toml`: the `marley_dcs` path dependency;
  - `Cargo.lock`, committed, for the copy's standalone tests.
- **Zed's `terminal` crate**: a `TerminalBackendEvent::ShellHook` mirror, since the conversion
  from alacritty's events is exhaustive, and a `process_event` arm that ignores it until #464.
- **The gate**: gate:3 runs each vendored crate's own tests (`cargo test --locked
  --manifest-path vendor/<crate>/Cargo.toml`).

### Out (explicitly deferred)
- Decoding the hooks in Zed and the anchored `BlockList`: #464.
- The shell scripts: #463.
- 8-bit C1 introducers (`0x90`, `0x9c`); Marley's hooks are 7-bit.
- Positions across a reflowing resize (the plan's D2 weak spot).

## Reference (§20)
N/A — Marley-specific plumbing: the plan's D1 design, in a crate Marley carries. The hook frame
format is Marley's own (`marley_terminal::dcs`, clean-room from the gpui era). No Warp code is
read, and alacritty and vte are Apache-2.0 upstream code we adopt.

### Prior art
- **Behavior maps:** `docs/warp_architecture/subsystems/00-overview.md:44` records only the
  behavior (per-command blocks from shell hooks).
- **Published material:** ECMA-48 control strings: DCS, ST, and CAN/SUB cancelling a string.
- **Code we already ship.**
  - `marley_terminal::dcs`'s scanner, with its ordered `Passthrough | Hook` stream
    (`PR-claude-ordered-events-coalesced-stream-hooks-001`).
  - vte 0.15 `ansi.rs`, which drops DCS (`hook`, `put` and `unhook` only log, `:1311-1326`).
  - alacritty's `Grid::scroll_up` history rotation (`grid/mod.rs:252-307`).

## UI proof
N/A — no UI delta: Zed ignores the new event until #464, so nothing on screen changes. The
unit tests over a real `Term` prove the event and its positions.

## Locked-In Decisions
- D1 — The scanner lives in a leaf crate so both `marley_terminal` and the vendored
  `alacritty_terminal` use one copy, under Marley's full gates.
- D2 — Only Marley's selectors are taken out of the stream. Every other byte, including other
  DCS, cancelled frames and oversize payloads, reaches the parser unchanged, so vte behaves
  exactly as before for everything that is not a Marley hook.
- D3 — Positions are raw (`evicted_lines`, `history_size`, cursor line and column, alt screen);
  the absolute line `evicted + history + cursor line` is a method, not a stored field. #464
  maps it back to a grid line.
- D4 — The upstream diff stays small: one new file, a field and one call in `event_loop.rs`, a
  variant, a counter, a dependency.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN one read carries `[preexec]`, an output line and `[precmd]`, the event loop shall emit the two hooks in that order, with positions before and after the output line | unit test over a real `Term` |
| REQ-002 | WHEN a hook frame is split across reads, the event loop shall emit it once, with the position where it completed | unit test |
| REQ-003 | WHEN a DCS is not Marley's, is cancelled by CAN or SUB, or overflows the cap, the scanner shall pass its bytes through unchanged | unit tests (`marley_dcs`) |
| REQ-004 | WHEN lines leave the top of the grid, `evicted_lines` shall grow by their number, so absolute lines stay stable | unit tests (`grid`) |
| REQ-005 | Zed's terminal and `marley_terminal` shall behave as before | `cargo nextest run -p terminal -p marley_terminal` |
| REQ-006 | The diff gate shall be green, running the vendored crate's own tests | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — `marley_dcs`, the `marley_terminal` switch, the vendored hunks, the Zed
  mirror, the gate step; fmt and clippy.
- **P3 Test** — REQ-001 to REQ-006, with negative checks.
- **P4 Complete** — CHANGELOG and docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
