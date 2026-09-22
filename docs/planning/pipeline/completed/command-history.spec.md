---
pipeline_id: b38c33a4-0079-4c0e-ad03-6d80bd71edc9
ticket: forge#29 (f6447fdf-d9cf-4543-b67e-79b6aa7230d9) · local docs/planning/tickets/open/TICKET-029-command-history.md
aar_id: c9c222c0-06fa-499b-8bd4-f545b0da2753
status: Phase 5 — Complete PASS
title: command history — ↑/↓ recalls previous commands
type: feature
milestone: M1.D
references:
  - crates/marley_app/src/workspace.rs (PaneState — gains the per-pane history field)
  - crates/marley_app/src/app.rs (the ↑/↓ routing + on_submit record + detach-on-edit)
  - docs/specs/SPEC-app-shell.spec.md (gains the history clause)
---

## Title
Add per-pane command history. Today ↑/↓ at the prompt do nothing (`Key::Other` → Ignored). Add a
PURE `CommandHistory` — a bounded ring of submitted commands + a navigation cursor with a
draft-stash — held per-pane on `PaneState`; the shim routes ↑/↓ (only when the command palette is
CLOSED — the palette owns ↑/↓ while open) to recall previous commands into the prompt buffer, like
a real shell.

## Scope
### In
- NEW pure `CommandHistory` (in input.rs or a new history.rs — design call):
  - `record(cmd)` — push the command (already trimmed non-empty by `submit_line`) UNLESS it equals
    the most-recent entry (dedup-last); evict the oldest past a capacity bound; reset navigation.
  - `prev(draft) -> Option<&str>` — step toward older: on the FIRST prev from the live line, stash
    `draft`; return the entry at the new cursor; clamp at the oldest (return the oldest, no wrap).
  - `next() -> Option<&str>` — step toward newer: return the next entry; at the newest, return the
    stashed draft and detach; clamp (Nothing when already at the live line).
  - `detach()` — leave navigation (a fresh edit began); a later prev re-stashes the current line.
- `crates/marley_app/src/workspace.rs` — `PaneState` gains `history: CommandHistory` (default-empty
  in `PaneState::new`); per-pane, travels with the pane on split/close.
- `crates/marley_app/src/app.rs` (shim) — in the palette-CLOSED key path: `"up"`/`"down"` →
  `history.prev/next` → replace the focused pane's `buffer` (set to the recalled string) + caret to
  end; an EDIT key (Char/Backspace/DeleteForward — not a motion) calls `history.detach()`;
  `on_submit` calls `history.record(line)` on the pane before running.
- SPEC-app-shell: the history clause (record/prev/next/draft/detach) + AC/Test-Plan/
  Mutation-Targets. CHANGELOG + arch doc.

### Out (explicitly deferred)
- Cross-session persistence (history to disk — a later settings/M2 concern), reverse-i-search
  (Ctrl-R), shared-across-panes history, dedup of non-adjacent duplicates. Raw-mode key handling
  is seq-6 (in raw mode ↑/↓ stream to the PTY, not history — the mode switch is seq-6's job).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — PURE `CommandHistory` state machine (cov 100/MSI 100); the shim only routes keys + applies
  the recalled string to the Buffer/caret. Per-pane on `PaneState`.
- D2 — Dedup only the IMMEDIATELY-PREVIOUS entry (like zsh's default `hist_ignore_dups`), not a
  global dedup — cheap + matches shell behavior.
- D3 — Bounded ring (a capacity const, e.g. 1000) — evict oldest on overflow. In-memory only.
- D4 — `prev` stashes the live draft on FIRST step so `next` past the newest restores it; clamp at
  both ends (no wrap). Editing detaches (the edited line becomes the new live draft).
- D5 — ↑/↓ recall ONLY when the palette is closed (the palette owns ↑/↓ while open — M1.C seq-4);
  the shim checks `palette_open` first (it already does for the palette path).
- D6 — Detach is driven by EDIT keys (Char/Backspace/DeleteForward), NOT motions (Left/Right/…) —
  the shim knows the dispatched `Key`; motions during a recall keep the recalled line navigable.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `record(cmd)` is called, `CommandHistory` shall append `cmd` unless it equals the most-recent entry (no consecutive duplicate), shall evict the oldest entry when over capacity, and shall reset the navigation cursor to the live line. | unit tests (append, dedup-last, capacity eviction, cursor reset) |
| REQ-002 | WHEN `prev(draft)` is called from the live line, the system shall stash `draft` and return the most-recent entry; repeated `prev` shall walk toward older entries and clamp at the oldest (returning it, no wrap). | unit tests (first prev stashes + returns latest; walk; clamp at oldest) |
| REQ-003 | WHEN `next()` is called while navigating, the system shall return the next-newer entry; stepping past the newest shall return the stashed draft and detach; calling `next()` at the live line shall return `None`. | unit tests (walk newer; draft restore at the top; None at live line) |
| REQ-004 | WHEN `detach()` is called (an edit began), the system shall leave navigation so that a subsequent `prev` re-stashes the then-current line. | unit tests (recall → detach → prev re-stashes the edited line) |
| REQ-005 | WHEN `scripts/gates.sh --diff` runs, every gate shall be GREEN with coverage 100%/MSI 100% on `CommandHistory`. | gate exit 0 + receipt |

## Phase Plan
- **P2 Design** — the exact `CommandHistory` shape (Vec + cursor enum/index + Option<draft>), the
  prev/next/record/detach signatures + return types (owned vs borrowed), the capacity const, the
  shim wiring (buffer-replace helper), the SPEC clause + mutation targets.
- **P3 Implement** — history + PaneState field + app.rs wiring + spec + CHANGELOG.
- **P3.5 Inspect** — critics: the cursor/clamp arithmetic, draft-stash edges, dedup, detach.
- **P4 Validate** — the unit tests + gate GREEN [--diff].
- **P5 Complete** — docs, AAR, archive, close #29.
