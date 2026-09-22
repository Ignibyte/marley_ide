---
pipeline_id: 8ab30d81-d799-4f38-a8eb-0b639684b41f
ticket: forge#283 (401ba6b7-d6e6-4886-94e9-7db9a1e4adc6) · local docs/planning/tickets/open/TICKET-283-editor-tab-key-gate.md
aar_id: f836f1d7-a885-49bc-b0cd-975b9087051b
status: Phase 5 — Complete PASS
title: Editor tab — swallow modified Enter/Tab/Backspace instead of acting on the hidden prompt
type: bug
milestone: M17
references: []
---

## Title
A pre-existing #251-era leak (surfaced by the #276 routing critic F4): the editor key-router block
(`app.rs:6069`) is gated `editor && !platform && !control`, so a MODIFIED Enter/Tab/Backspace
(⌘Enter, ⌃Enter, ⌘Backspace, ⌃Tab) on an editor tab skips it and falls to the terminal handlers,
which resolve `focused_terminal` to a HIDDEN pane (#71) and act on its invisible prompt: ⌘Enter
SUBMITS it (catch-all `apply_key`, 6399), ⌘Backspace edits it, ⌃Enter/⌃Tab write raw bytes to the
hidden PTY (raw route, 6301). Swallow these on a non-terminal tab.

## Scope
### In
- A pure predicate — `swallow_hidden_prompt_key(active_tab_is_terminal: bool, key: &str) -> bool` —
  true when the tab is NOT a terminal AND the key is `enter`/`tab`/`backspace` (the keys that map to
  prompt submit/edit/raw regardless of modifiers).
- A one-line shim gate at the top of the terminal fallthrough (before the raw route + catch-all):
  if the predicate is true, `stop_propagation` + `return` (swallow — no hidden-prompt action).
  Mirrors the #278 `active_tab().grid().is_some()` gate.
- Headless asserts (`hidden_prompt_text`): on an editor tab, ⌘Enter/⌃Enter/⌘Backspace/⌃Tab leave the
  hidden prompt UNCHANGED; a terminal tab's Enter/Tab/Backspace stay byte-identical.

### Out (explicitly deferred)
- Other keys' fallthrough — an unbound chord (⌘X etc.) still propagates for the #267 platform text
  path; only enter/tab/backspace are swallowed.
- Making ⌘Enter DO something in the editor (e.g. insert a newline) — v1 is a no-op swallow.
- The terminal-tab prompt behavior (byte-identical — the gate is false there).
- Cockpit tabs' own key handling (unchanged; the predicate covers them via "not a terminal", but their
  Enter/Tab/Backspace were already unhandled leaks — this just stops the hidden-prompt corruption).

## Reference (§20)
N/A — Marley-specific input-routing correctness (a hidden-surface leak in Marley's own tab/pane model).
No reference-app behavior; clean-room. Mirrors the in-repo #278 `grid().is_some()` gate precedent.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — The decision is a PURE predicate (cov/MSI 100) in `input.rs` (next to `op_for_ctrl_key`); the
  app.rs gate is the masked shim (the input path is `mutants::skip` + coverage-excluded), proven by the
  headless flow.
- D2 — Gate on "the active tab is NOT a terminal" (`!active_tab().grid().is_some()`) so the fix covers
  editor AND cockpit tabs (both resolve `focused_terminal` to a hidden pane); the ticket's editor case is
  the headline, cockpit is the free correctness bonus.
- D3 — Swallow = `stop_propagation` + `return`, no state change / no `notify` (nothing happened).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---------|--------|
| REQ-001 | WHEN the active tab is an editor and the user presses ⌘Enter or ⌃Enter, the system shall NOT submit or write to the hidden terminal prompt | unit (predicate) + headless `hidden_prompt_text` unchanged |
| REQ-002 | WHEN the active tab is an editor and the user presses ⌘Backspace or ⌃Tab, the system shall NOT edit the hidden prompt / write raw bytes | unit + headless |
| REQ-003 | WHILE the active tab is a terminal, Enter/Tab/Backspace shall behave byte-identically to pre-#283 (submit/complete/edit the real prompt) | headless (terminal-tab flow green) |
| REQ-004 | WHEN a non-enter/tab/backspace key reaches the fallthrough on a non-terminal tab, the system shall keep its #267 fallthrough (not swallowed) | unit (predicate false for other keys) |
| REQ-005 | The pure predicate shall reach 100% line coverage and MSI 100 | gate |

## Phase Plan
- **P2 Design** — the predicate signature + module; the exact shim gate placement; the kill-list; the headless flow.
- **P3 Implement** — `input.rs` predicate + the app.rs gate.
- **P3.5 Inspect** — critics (does the gate over/under-swallow? terminal-tab regression? cockpit interaction?).
- **P4 Validate** — unit kill-list + headless editor-tab + terminal-tab flows; gate green.
- **P5 Complete** — CHANGELOG + editor.md routing note; AAR; close.
