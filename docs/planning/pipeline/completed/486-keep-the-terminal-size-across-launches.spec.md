---
pipeline_id: dd59ee01-3aee-4415-93b6-63424e46452e
ticket: docs/planning/tickets/open/TICKET-486-keep-the-terminal-size-across-launches.md
status: Phase 4 — Complete PASS
title: "The first terminals of a launch open at the last session's size"
type: bug
slice: prong 1, T0 (#485's known limit)
references: [docs/planning/pipeline/completed/485-starship-echo-lands-off-the-line.spec.md]
---

## Title
Since #485 a terminal opens at the size the last terminal view laid out, so its shell lays its
first prompt out for the width it will have. A launch's first terminals open before any view
has a size, at Zed's 100 × 6 debug size, and a multi-line prompt wider than 100 columns is
misdrawn until the next prompt. The last size is now kept across launches.

## Scope
### In
- `crates/terminal/src/terminal.rs`: `marley_last_bounds()` and `marley_seed_last_bounds(bounds)`
  over #485's `MARLEY_LAST_BOUNDS`; a seed fills it only while no view has set it this launch.
- `crates/marley_workbench/src/terminal_size.rs`: at `init`, the size kept in Zed's key-value
  store (scope `marley-terminal-size`) seeds the slot; at quit, the slot's size is written back.
- `script/e2e/486-keep-the-terminal-size-across-launches.sh`.

### Out (explicitly deferred)
- A window restored at another size than the last session's: its first terminals open at the
  kept size and are resized as they were before #485.
- A crash, which writes nothing: the size kept by the last clean quit stays.

## Reference (§20)
N/A — Marley-specific: #485's slot is Marley's; upstream Zed opens every terminal at its debug
size. Zed's own key-value store (`db::kvp`) keeps the value, as `shortcut_note.rs` does.

### Prior art
- **Code we already ship.** #485's `MARLEY_LAST_BOUNDS` and `set_size`
  (`crates/terminal/src/terminal.rs`); `TerminalBounds` already derives `Serialize` and
  `Deserialize`; `KeyValueStore::scoped` (`crates/db/src/kvp.rs`), read synchronously and written
  on quit as `shortcut_note.rs` and `clients.rs`'s `on_app_quit` do. The store's global is set at
  the start of `app.run`, before `initialize_workspace` calls `marley_workbench::init`, and
  before any workspace is restored.

## UI proof
`script/e2e/486-keep-the-terminal-size-across-launches.sh` (#485's two-line prompt wider than 100
columns): the first launch's first terminal is misdrawn (`486-01-first-launch`, #485's limit, no
size kept yet); after a quit and a relaunch on the same data directory, the first terminal echoes
the typed line whole (`486-02-relaunched`).

## Locked-In Decisions
- D1 — The size is written at quit, not at each resize: `set_size` runs on every layout, and the
  quit's size is the one the next launch's window will most likely have.
- D2 — The kept size only seeds the slot; the first `set_size` of this launch takes over.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts after a quit, its first terminal shall open at the size the last session ended with. | `486-02-relaunched`: `echo again` whole on the prompt line |
| REQ-002 | WHEN no size was kept, a launch's first terminals shall open as before. | `486-01-first-launch` |

## Phase Plan
- **P1 Plan** — this spec, the design in the notes.
- **P2 Code** — the ledger row first; the two functions and the module; clippy; the gate.
- **P3 Test** — the scenario, both shots read.
- **P4 Complete** — CHANGELOG, docs, ledger, close, archive, commit.
