---
pipeline_id: 9e712e8c-111f-4eaa-a597-77e3d718a2a2
ticket: docs/planning/tickets/open/TICKET-435-run-block-rerun-jump-to-failure.md
status: Phase 5 — Complete PASS
title: Run blocks — per-block rerun + block-scoped jump-to-failure (M33 Phase C, thread 2)
type: feature
milestone: M33
references:
  - docs/planning/design-notes/m33-tail-and-wedge-shelf.md
  - docs/planning/pipeline/queued/434-runnables-gutter-run-block.spec.md
---

## Title
The wedge's payoff loop — **DEPENDS on #434** (queued; its run-Block identity
is this ticket's substrate and must ship first). A #434 run Block carries its
runnable identity, so (a) a per-block RERUN affordance re-spawns the SAME
resolved command as a FRESH run Block (identity carried forward — not a raw
command-string replay), and (b) a FAILED run block gets one-keystroke
block-scoped jump-to-failure: the first failing `file:line[:col]` ref in THAT
block's output opens in the editor (NavStack pushed — ⌃- returns), and
repeating the keystroke cycles through the block's refs. Edit → rerun → jump
becomes one tight keyboard loop. Note the shipped base this builds on: #175
already gives EVERY finished block a string-replay ↻ affordance and #213 a
menu-only, first-ref-only Jump to Failure — the delta here is identity-true
rerun for run blocks, plus keystroke + cycling + NavStack for the jump.

## Scope
### In
- Run-block identity CONSUMPTION: the #434 identity (resolved command +
  origin file/symbol, per the #434 spec) is readable off a run Block for the
  lifetime of the session (survives scrollback; not restart — D1).
- A per-block RERUN affordance on finished run Blocks (header chrome and/or
  block context menu — the #175/#217 hover-action grammar) that re-spawns
  through the #434 spawn path as a fresh tail Block carrying the same
  identity; #40 idle-guarded like the shipped `rerun_block`.
- One-keystroke block-scoped jump-to-failure (keymap-routed; chord is a
  Phase 2 pick — Terminal-context F8 is free, the #290 convention): target =
  the relevant FAILED run block, refs derived fresh from its CURRENT output
  (the #196/#289 link fold — `parse_trace_frames`/`first_failure_ref`
  recipe), first press opens ref 1, repeat cycles with wrap (#290 shape);
  every jump places the caret AND pushes the NavStack explicitly (#330).
- Jump state lifecycle: the cycle cursor is keyed to the target block and
  discarded when the target changes; a PASSING rerun leaves no failed target
  → the keystroke fails closed (flash, no side effects), i.e. jump state
  clears on green.
- Failed-but-refless run block: keystroke flashes and changes nothing.
- Non-run blocks: byte-identical #175/#213 behavior (no regression).

### Out (explicitly deferred)
- Rerun-with-edited-args (any prompt/pre-fill UI; rerun is verbatim v1).
- Cross-block failure aggregation — failed refs entering `problem_rows` /
  ⌘⇧M is **#433's lane**, not this ticket's.
- Watch mode (auto-rerun on file change) and any run-on-save policy.
- Rerun affinity/supersede policy beyond "fresh block at the tail" (the Zed
  `deferred_tasks`/reuse analog — a recorded follow-up).
- Any change to #434's discovery/spawn machinery itself.

## Reference (§20)
**Warp + Zed** (behavior maps; research only — clean-room, no Warp (AGPL) or
Zed (GPL) source read or translated). Warp: per-block rerun EXISTS in Warp's
block model — `docs/warp_architecture/subsystems/03-terminal-session-core.md`
records "Command Blocks (per-command grid, exit code, cwd, duration,
**rerun**)" as a Matched capability row; Warp has NO jump-to-failure (no
editor to jump into — the doc's fusion-wedge paragraph states this is
exactly where Marley wins). Zed:
`docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` — task rerun
is a TAB-level hover button (`rerun_button`, terminal_view.rs:1082)
dispatching `Rerun { task_id, reevaluate_context, … }` against the task
inventory's stored `SpawnInTerminal`; jump-to-failure is only the GENERIC
terminal path-hyperlink, never task-scoped. The doc's §8 fusion map rows —
"Rerun = re-spawn the Block's stored spec (metadata rides the Block)" and
"jump-to-failure: parse THIS Block's output, block-scoped" — are literally
this ticket's brief, written as Marley's win precisely because Zed has no
block to scope either action to. Marley deviation kept: our affordances are
BLOCK chrome (the #217 hover grammar + block menu), not tab chrome.

### Prior art (three legs)
1. **Behavior maps** — the two §20 docs above: Warp proves per-block rerun
   is the established block-terminal grammar (capability row, Matched by
   `Block::rerun_command` R30); Zed proves the task-identity rerun model
   (stored spec, re-resolve option) and its §6 gap analysis + §8 table give
   the block-scoped jump design almost verbatim. Both consulted as maps
   only.
2. **Published material** — none likely, and honestly: none found. No spec
   or protocol governs per-block rerun or block-scoped failure navigation;
   the nearest public product behavior (VS Code's shell-integration command
   decorations offering "rerun command") is observed behavior, not a
   published contract. Nothing normative to cite.
3. **Permissive deps** — none owns this seam, honestly: the block substrate
   is Marley's OWN (`terminal_blocks` — Block/BlockList, the DCS hook
   protocol), the ref fold is our own `links.rs` (#196/#212/#289/#291), and
   `alacritty_terminal` (permissive) models grids, not blocks. This ticket
   is integration of shipped Marley infra; no new dependency is warranted.

## React-first (parity)
UI-AFFECTING — **zone A** surface (the MARLEY-PARITY.md zone map's "Terminal
blocks | 03, 23 | components/TerminalView.tsx" row): run-block header
affordances and the jump affordance are visible terminal chrome inside the
frozen shell. Zone A means Marley-authoritative for SHIPPED chrome — but
these affordances are NEW chrome with no Marley truth yet, so the pipeline's
React-first rule still applies as a minimal POC mirror, honestly scoped: the
POC's `components/TerminalView.tsx` already models command blocks, the
block context menu, and a failed-only "Jump to Failure" row (its comment at
:110 pins presence-on-failure), so extend THAT file with the run-block
rerun/jump affordance states (idle/hover/failed variants; mock identity),
visually verify at localhost:5173 (screenshot + READ the PNG), then port
1:1. Once shipped in Rust, the Marley render resumes authority for this row
and the POC must match it (the zone A discipline). No zone B surface is
touched; keystroke behavior (jump/cycle) is Rust-side and needs no POC
counterpart beyond the affordance chrome.

## Locked-In Decisions
- D1 — **Identity is transient, session-scoped**: where it lives is a
  Phase 2 pick — Block metadata (a `terminal_blocks` field/API, correlated
  at the Preexec open, apply.rs:66) or an app-side map keyed by `BlockId` —
  but it is BOUNDED: it must survive scrollback (any block still in the
  `BlockList` keeps its identity) and must NOT survive restart (transient,
  the #427 documented-drop precedent — no persistence, no serialization).
  Design weighs both options and records the pick + correlation story.
- D2 — **Jumps push the NavStack EXPLICITLY** (#330): `open_file_at` /
  `open_and_place_caret` do NOT push — the jump captures the origin
  `NavLoc` BEFORE and pushes AFTER a successful open, the goto-definition
  shape. Every press of the cycle pushes (each jump is returnable).
- D3 — **Refs re-derive from the block's CURRENT output — stateless**
  (the #331/#327 stale-latch class): no cached ref list; each keystroke
  derives from `output_text()` via the shipped fold. The ONLY state is the
  cycle cursor, keyed by the target block's `BlockId` and discarded when
  the target changes or resolves; no failed target → fail closed.
- D4 — **Rerun carries identity through the #434 spawn path** — a fresh
  tail Block with the SAME identity attached (so the fresh block is itself
  a run block: rerun/jump affordances, #433-ready), never a bare
  `write_command` string replay that would mint an identity-less block.
  #40 idle-guarded exactly like `rerun_block` (app.rs:8958).
- D5 — **One resolution root discipline** (#289 lesson): jump refs resolve
  against the SAME field that builds stored open paths (`project_root`) —
  never a parallel root field. Block-cwd-aware resolution (PromptInfo.pwd)
  is a design option but must obey the same-field comparison rule.
- D6 — **Target selection is derived at press time**: the keystroke
  resolves its failed run block when pressed (no armed pointer); v1 scope
  is the active workspace terminal — if design widens to workspace-wide,
  it MUST iterate all terminal grids (PR-1345), not `terminal_grid_index`.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a finished #434 run Block's RERUN affordance is activated and the session is idle, the system shall spawn the same resolved command through the #434 spawn path as a FRESH run Block at the tail — new `BlockId`, same runnable identity attached, the original block unchanged in scrollback. | headless drive |
| REQ-002 | WHEN the jump-to-failure keystroke is pressed and a FAILED run block is the target, the system shall derive the block's `file:line[:col]` refs from its CURRENT output, open the FIRST ref in the editor with the caret placed, and push the NavStack origin explicitly (⌃- returns to the pre-jump location). | headless drive |
| REQ-003 | WHEN the keystroke is pressed again with the same target, the system shall jump to the NEXT ref in the block's refs, wrapping past the last back to the first (the #290 convention), each jump placing the caret and pushing the NavStack. | unit (pure cycle fn) + drive |
| REQ-004 | WHEN a rerun of a failed run block's identity finishes with exit 0, the jump keystroke shall no longer target the superseded failure — with no failed run block remaining it shall flash and change nothing, and the old cycle cursor shall be discarded (jump state clears on green). | headless drive |
| REQ-005 | WHEN the keystroke targets a FAILED run block whose output contains NO file:line refs, the system shall flash and change nothing (fail closed — no open, no NavStack push). | unit + drive |
| REQ-006 | WHILE a block is not a #434 run Block, the run-block affordances shall be absent and the shipped #175/#213 block actions (hover ↻/⧉, menu Rerun / Jump to Failure) shall behave byte-identically to before this ticket. | drive + existing suite green |
| REQ-007 | WHILE the gate runs, the new pure logic (identity carry/lookup, ref-cycle, target selection) shall hold cov 100 / MSI 100 with no suppressions (new files `git add -N`-staged before the diff gate — PR-claude-new-file-mutants). | gate exit |

## Phase Plan
- **P2 Design** — READ the promoted #434 spec/design first (identity shape +
  spawn seam are its contract); pick D1's residence (Block metadata vs
  app-side map) with the Preexec correlation story; the jump chord + target
  rule (D6); the ref-derivation fn (`first_failure_ref` vs
  `parse_trace_frames` for the cycle list); affordance chrome per the #217
  grammar; file manifest + regression test plan.
- **P3 Implement** — POC first (TerminalView.tsx affordance states,
  screenshot, read), then Rust per manifest: pure seams (cycle fn, target
  selection, identity carry) before the app-shell shims.
- **P3.5 Inspect** — independent critics vs the diff; fix real findings.
- **P4 Validate** — write + RUN the planned tests; live drive of the full
  edit → rerun → jump loop + the parity pair; gate green (scoped mutants
  locally — full sweeps ride the dev-box lane).
- **P5 Complete** — docs (§21), parity sync, ledger capture (§19), close the
  ticket, archive the pair.
