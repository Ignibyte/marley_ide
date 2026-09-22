---
pipeline_id: 4edd63e4-e93b-4acf-b623-75f924ccdbae
ticket: forge#265 (73614d2c-09e9-46d9-9fb2-f3da97a08c94) · local docs/planning/tickets/open/TICKET-265-keycontext.md
aar_id: 6f75fb5a-6d6c-4584-bd60-c54282656727
status: Phase 5 — Complete PASS
title: B1 — KeyContext: context-scoped keybindings (fix the ⌘D collision)
type: feature
milestone: M16
references:
  - docs/zed_architecture/subsystems/09-vim-keymap-contexts.md
  - crates/marley_app/src/keymap.rs
---

## Title
The editor frontier's first step (roadmap B1): make the keymap context-aware so
one chord can mean different things on different focused surfaces. Today the
flat table binds ⌘D→new-terminal globally (#197) and a `chords_unique` guard
forbids the very duplicate the editor needs (⌘D→select-next-match); ⌘F is a
second LIVE context-blind collision (hardcoded before the table at app.rs:4451
— it opens the terminal scrollback find bar even on an editor tab). Adopt the
KeyContext model — bindings carry a context predicate, the app publishes a
context stack from the focused surface, resolution is deepest-match-then-order
— as Marley-original pure code in the existing `keymap.rs` seam.

## Scope
### In
- **Pure keymap context model** (`crates/marley_app/src/keymap.rs`,
  Marley-original, gpui-free, cov/MSI 100): a context tag per binding
  (None = global), a caller-supplied context stack, and
  `action_for(chord, stack)` resolving by deepest-matching context then
  insertion order; `chords_unique` relaxes to **per-context** uniqueness
  (disjoint contexts may share a chord; a same-context duplicate is still
  rejected).
- **Context-stack builder**: a pure function mapping the app's live focus
  facts (active tab is editor / focused pane is terminal / etc.) to the stack
  the shim passes in (testable without gpui).
- **⌘D split**: Terminal context → "new-terminal" (unchanged behavior); Editor
  context → "select-next-match" (NEW).
- **Editor select-next-match v1** (`crates/editor`, pure): first press selects
  the word under the caret; repeat moves the (single) selection to the next
  occurrence of the selected text, wrapping past EOF; no match → no-op.
  (Multi-cursor add-next is B5; this is the single-selection subset.)
- **⌘F migration**: the hardcoded pre-table ⌘F becomes a Terminal-context
  binding "open-find" — an editor tab no longer opens the terminal find bar
  (falls through unbound; editor-find is a future feature).
- **App shim wiring** (app.rs): build the stack at key-time, pass it to
  `action_for`, dispatch "select-next-match" to the editor surface, delete the
  hardcoded ⌘F arm.
- Keymap test updates (signature + per-context uniqueness + roster count) and
  new editor unit tests; driven visual proof of the ⌘D split.

### Out (explicitly deferred)
- gpui-native adoption (per-surface FocusHandle + `.key_context()` +
  `actions!`/`on_action`) — Marley has ONE root FocusHandle; the focus-tree
  rewire is its own later step (#267 adds EntityInputHandler; full dispatch
  migration later).
- The full predicate grammar (`>`, `||`, `==`) — design picks the minimal
  subset that expresses surface identity; grammar growth arrives with the
  declarative keymap (roadmap step 3).
- NoAction/Unbind + multi-stroke chords (roadmap step 2); declarative TOML
  keymap (step 3); vim modal layer (step 4).
- Multi-cursor ⌘D (B5 — add-next-occurrence as a second cursor).
- Editor find (⌘F-in-editor), and migrating the other hardcoded pre-table
  chords (⌘⇧D diff, ⌘⇧C git, ⌘C/⌘X/⌘V, ⌘-arrows) into the table — follow-up
  ticket; they are context-CORRECT today via is_some() gates, just not
  table-declared.
- Palette per-context chord chips (the ⌘D chip stays labeled for its primary
  Terminal context in v1).

## Reference (§20)
**Zed (the editor reference — same-gpui-stack)** — behavior matched: the
KeyContext-scoped keybinding model (one keymap, many surfaces; a binding is
eligible where its context predicate matches the focused surface's context
stack; deepest match wins, then load order) and the editor's ⌘D
select-word/next-occurrence idiom, per
`docs/zed_architecture/subsystems/09-vim-keymap-contexts.md` (§A, §D — a
behavior-level transcription; the ⌘D worked example is the exact collision
this ticket fixes). Clean-room: the dispatch MECHANISM is gpui's
(Apache-2.0, adoptable freely); Marley implements the model as its OWN pure
code in `keymap.rs` per §A.7 of that doc — no GPL `vim`/`keymap_file.rs`
source is read or translated; the select-next-match behavior is the public
editor idiom reimplemented against Marley's own Buffer/Selection types.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- D1 — The context model is **Marley-original pure code in keymap.rs** (no
  gpui types in the pure seam — keeps the module gpui-free and cov/MSI-100
  like today), matching the gpui model's semantics: deepest-context match
  wins, insertion order breaks ties, a context-less binding is global and is
  shadowed by any context match.
- D2 — One slice ships mechanism + both named collisions: ⌘D (Terminal vs
  Editor) and ⌘F (Terminal-only). No other chords change behavior.
- D3 — Editor ⌘D v1 is single-selection: select word under caret → next
  occurrence (wrap). Implemented as pure `crates/editor` fns over
  Buffer/Selection; the app shim only routes the action.
- D4 — `chords_unique` becomes per-context (`chords_unique_scoped`): same
  chord + same context = rejected; same chord + disjoint contexts = legal.
  The debug_assert in `default_bindings()` stays, upgraded to the scoped
  guard.
- D5 — Overlays keep first refusal ahead of the table (existing ladder,
  unchanged); the context stack describes the POST-overlay surface only
  (Workspace + Terminal|Editor). Overlay contexts come later if ever needed.
- D6 — The palette "New Terminal" ⌘D chip stays (truthful in its Terminal
  context); noted as a known v1 simplification.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN ⌘D is pressed with a Terminal surface focused (no overlay), the system shall dispatch "new-terminal" (pane count +1), exactly as today. | keymap unit (Terminal stack → "new-terminal") + driven capture |
| REQ-002 | WHEN ⌘D is pressed with the editor tab active and no selection matching the caret word, the system shall select the word under the caret; a repeat ⌘D shall move the selection to the next occurrence of the selected text, wrapping past end-of-buffer; with no occurrence/word it shall be a no-op. | pure editor unit tests (select_next_match) + driven capture (visual_acceptance) |
| REQ-003 | The keymap shall resolve a chord bound under multiple disjoint contexts by the caller's context stack: the binding whose context matches deepest wins; a context-less binding shall apply only when no context-bound binding for the chord matches. | keymap unit tests |
| REQ-004 | WHEN ⌘F is pressed with a Terminal surface focused, the system shall open the scrollback find bar; WHEN the editor tab is active, ⌘F shall NOT open the terminal find bar. | keymap unit + app-shim behavior; driven capture |
| REQ-005 | The uniqueness guard shall reject two bindings sharing both chord AND context, and shall accept the same chord under disjoint contexts. | unit tests on the scoped guard |
| REQ-006 | Every pre-existing chord in the table shall resolve to its current action on every surface it works on today (no regressions from the signature change). | full existing-chord roster test updated + green |
| REQ-007 (visual_acceptance) | The live app shall demonstrate the split: on a terminal tab ⌘D adds a pane; on an editor tab ⌘D visibly selects the caret word (and a second press moves the highlight) with pane count unchanged. | driven self-test capture, PNG read + asserted |

## Phase Plan
- **P2 Design** — the context type + predicate subset, exact keymap API,
  stack-builder inputs, select_next_match algorithm + edge cases, app.rs
  wiring points (4451 removal, 4603 stack pass, action dispatch), test plan
  incl. cargo-mutants --list on the new fns.
- **P3 Implement** — keymap.rs model + guard; editor select_next_match;
  app.rs shim wiring; palette untouched.
- **P3.5 Inspect** — critics: correctness (precedence/wrap edges),
  regression (all 41 chords), provenance (no GPL-derived code), simplification.
- **P4 Validate** — unit+mutation on the pure fns; nextest workspace; driven
  capture of REQ-007; gate --diff green.
- **P5 Complete** — CHANGELOG, arch docs (app_shell keymap section + editor.md),
  AAR, archive, close #265.
