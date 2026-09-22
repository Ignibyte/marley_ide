---
pipeline_id: fba7410b-e496-4c09-b3ec-af943abcf046
ticket: forge#197 (59f70ac0-a0bc-438f-8a84-9a25344e1090) · local docs/planning/tickets/open/TICKET-197-split-keybindings.md
aar_id: da6cfe79-3922-4077-9b77-844dc2c3e128
status: Phase 5 — Complete PASS
title: Split keybindings — ⌘-chords for Split Right / Split Down
type: feature
milestone: M12.2
references: []
---

## Title
Give the tile-split — **Split Right** (`split_focused_pane(Horizontal)`) and **Split
Down** (`Vertical`) — keyboard chords and command-palette entries. Today those two
actions are reachable ONLY through the right-click context menu (app.rs:2297-2298);
there is no keybinding for either direction.

## Scope
### In
- `keymap.rs` — two new bindings: a Split-Right chord + a Split-Down chord → two new
  action names (`"split-right"` / `"split-down"`).
- `app.rs dispatch_action` — two new arms dispatching to the existing
  `split_focused_pane(PaneAxis::Horizontal)` / `(PaneAxis::Vertical)`.
- The command palette — Split Right + Split Down entries (so they're discoverable +
  invocable without a mouse).
- A PURE keymap collision guard (no two bindings share a chord) — proves the new
  chords don't clash + guards future additions.

### Out (explicitly deferred)
- The split algebra + `split_focused_pane` (already exist — M1.B / #155 / #166).
- The right-click context menu (already has Split Right / Split Down — #166).
- Changing ⌘D's behavior (it stays `new_terminal_pane`).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- **D1** — The tile-split is `split_focused_pane(axis)`: Right = `Horizontal`, Down =
  `Vertical`. #197 only adds chord + palette entry points that dispatch to it.
- **D2 — ⌘D is a MISNOMER, not a split.** `keymap.rs:78` binds ⌘D → action
  `"split-pane"`, but `dispatch_action` (app.rs:2879) runs `new_terminal_pane()` (#135,
  "same as +") — a NEW TERMINAL, not a tile-split. So ⌘D is NOT one of the split
  chords. Design may RENAME the `"split-pane"` action → `"new-terminal"` (behavior
  unchanged) for honesty IF a grep shows only keymap + dispatch reference it — a
  decluttering, decided in Phase 2.
- **D3** — Chord choice is a Phase 2 decision. Candidates: Split-Right = ⌘⇧D (free;
  shift-variant of the ⌘D new-terminal); Split-Down = a free chord (e.g. ⌘⇧E is taken
  by fleet — pick from the free set). AVOID ⌘⇧-arrow (the #131 bug the keymap test pins
  as `None` — must stay unbound). Design picks + records + a collision test.
- **D4** — Add a PURE collision guard on `Keymap` (e.g. `has_unique_chords()` or a test
  that no chord maps twice) — cov/MSI 100.
- **D5** — Autonomous through commit (chad away, pre-approved the M12.2 queue); hold the
  push (offer-first).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the Split-Right chord is pressed with a terminal pane focused, the system shall split it into a right-neighbor pane (Horizontal). | keymap resolution unit test (chord → `"split-right"`) + driven capture |
| REQ-002 | WHEN the Split-Down chord is pressed, the system shall split the focused pane into a below-neighbor pane (Vertical). | keymap unit test (chord → `"split-down"`) + driven capture |
| REQ-003 | No two keymap bindings shall share the same chord (the new split chords do not collide with any shipped binding). | PURE collision-guard unit test over `default_bindings()` |
| REQ-004 | The command palette shall list "Split Right" and "Split Down" as invocable commands. | palette command-list unit test + driven (palette → Split Down → pane splits) |

## Phase Plan
- **P2 Design** — pick the two chords (collision-checked); confirm the `split-pane`→
  `new-terminal` rename scope (grep); the file manifest (keymap + dispatch + palette +
  the collision guard); the regression test table.
- **P3 Implement** — bindings + dispatch arms + palette entries + the guard.
- **P3.5 Inspect** — critics: collision correctness, the rename's blast radius, does the
  chord reach a NON-terminal-focused pane safely.
- **P4 Validate** — keymap + collision + palette tests; driven capture (press each chord
  → split right / down).
- **P5 Complete** — CHANGELOG + app_shell keymap doc; AAR; close.
