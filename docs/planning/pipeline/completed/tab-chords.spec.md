---
pipeline_id: 01191ecd-a671-4a73-a183-1ff4415b5a92
ticket: forge#170 (0cb3582e-7424-47a2-8c84-fd6246efa639) · local docs/planning/tickets/open/TICKET-170-tab-chords.md
aar_id: 04f3bfca-cde2-4b88-bf97-be78e601eeab
status: Phase 5 — Complete PASS
title: M10 — tab chords (⌘T new, ⌘[ prev, ⌘1–⌘9 jump)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/tabs.rs (PURE: prev_index + tests)
  - crates/marley_app/src/keymap.rs (the 11 bindings + test asserts)
  - crates/marley_app/src/app.rs (SHIM: the 3 dispatch arms)
---

## Title
Keyboard tab management — ⌘T opens a new terminal tab, ⌘[ cycles back (⌘] already cycles forward),
⌘1–⌘9 jump straight to a tab.

## Scope
### In
- PURE: `tabs::prev_index(current, len)` (wrap `0 → len-1`; `len==0 → 0`).
- keymap: ⌘T → `new-tab`; ⌘[ → `prev-tab`; ⌘1..⌘9 → `switch-tab-1..9` (collision-checked free).
- SHIM dispatch: `new-tab` → `new_terminal_pane`; `prev-tab` mirrors the `next-tab` arm with `prev_index`;
  `switch-tab-N` → `switch_tab(N-1)` (out-of-range = the guard's no-op).

### Out
- Reordering tabs; ⌘0 (Warp uses it for "last tab" — later if wanted); per-project chords.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `prev_index` shall wrap 0 to `len-1`, step interior indices down, and yield 0 for `len == 0`. | unit |
| REQ-002 | The keymap shall bind ⌘T, ⌘[, and ⌘1/⌘9 to their actions with no collisions. | unit (binding test) |
| REQ-003 (visual) | ⌘T shall add "terminal 2" (active, in the rail); ⌘[ shall move the highlight back; ⌘2 shall jump to the second tab. | driven capture |
| REQ-004 | gate GREEN; prev_index at cov/MSI 100. | gate |

## Phase Plan
P2 folded (mirror next-tab). P3 implement. P3.5 self-review (the chords + arms mirror inspected patterns;
gate:5 covers prev_index). P4 tests + driven + gate. P5 docs/AAR/archive.
