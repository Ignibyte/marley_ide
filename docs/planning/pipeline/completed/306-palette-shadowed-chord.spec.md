---
pipeline_id: f18a0746-de52-4c26-8a5b-de5031d4988f
ticket: forge#306 (d5b09d10-581c-47d4-82c1-ad446dfec2d0) · local docs/planning/tickets/open/TICKET-306-palette-shadowed-chord.md
aar_id: 22741ab8-c3bd-4d66-8dee-d896a5d2a530
status: Phase 5 — Complete PASS
title: Palette hides a keycap chip whose chord a context-scoped row has shadowed
type: bug
milestone: M22
references:
  - crates/marley_app/src/palette.rs
  - crates/marley_app/src/keymap.rs
  - crates/marley_app/src/app.rs
---

## Title
The command palette renders each command's STATIC `binding` as keycap chips (app.rs ~16833) with no
resolution against the focused surface, so it advertises a chord even when a context-scoped keymap row
has shadowed it (⌘⇧L "Split Right" while an editor pane is focused resolves to `select-all-occurrences`,
#298, not a split). Fix: render a command's chip only when its chord still resolves to THAT command on
the active surface's context stack — a pure `palette::displayed_binding` composing the already-built
`keymap.action_for` + `action_for_command`, consumed at the render site with the live stack + keymap.

## Scope
### In
- `crates/marley_app/src/palette.rs` (pure, cov/MSI 100):
  - `pub fn displayed_binding<'a>(command: &'a Command, keymap: &Keymap, stack: &[KeyContext]) -> Option<&'a KeyBinding>`
    — `Some(chord)` iff `command.binding` is set AND `keymap.action_for(chord, stack) == Some(action_for_command(command.id))`;
    else `None`. Composes the two EXISTING pure fns; imports `Keymap`, `KeyContext`.
- `crates/marley_app/src/app.rs` (shim): the palette chip render (~16833) computes the live stack once
  (`self.shell.active_project().active_tab().key_context()` — the SAME source dispatch uses at app.rs:14503)
  and calls `displayed_binding(scored.command, &self.keymap, stack)` instead of `scored.command.binding.as_ref()`.

### Out (explicitly deferred)
- Filtering WHICH command rows appear by focused surface — the flat `cockpit_commands()` list is deliberate;
  this ticket scopes to the stale HINT, not row filtering.
- Re-labeling a shadowed chip to an alternative chord (a command has one static chord; shadowed → nothing
  else to show → re-label collapses to hide).
- Any change to `action_for`, `action_for_command`, `cockpit_commands`, or the keymap.

## Reference (§20)
Zed (the editor) and VS Code command palettes resolve an advertised keybinding against the active
when/KeyContext — they never surface a chord that won't fire on the focused surface. Marley matches that
BEHAVIOR. Behavior-level only; **no Zed/VS Code source read**. The resolution engine itself is
Marley-original — `keymap.rs`'s KeyContext model (#265 / editor-frontier B1, mined from Zed's vim *doc*
for behavior and reimplemented clean-room). #306 is a pure CONSUMER fix over that engine.

### Prior art
1. **OUR own code / permissive deps (highest-yield leg):** the resolver ALREADY EXISTS and is pure.
   `keymap.action_for(chord, stack)` (keymap.rs:542, #265) + `action_for_command(id)` (palette.rs:127)
   already compose to answer *"does this chord invoke this command on this surface?"* — the deepest-scoped
   binding wins, a global ranks 0, an off-surface scoped binding is ineligible. #306 needs **no new
   resolution logic** — a 3-line pure composer + a one-line render swap. gpui ships its own
   keymap/`KeyContext`/`KeyBindingContextPredicate`, but Marley already owns the equivalent, so no
   adoption is needed here — checked, no gpui seam to take.
2. **Behavior maps** (`docs/warp_architecture/`, `docs/zed_architecture/`): the palette is a cockpit/UX
   surface; stale-chip suppression is generic IDE hygiene. No Warp/Zed source consulted.
3. **Published material / protocol:** n/a.

## Locked-In Decisions
- **D1 — HIDE, not re-label.** A command's `binding` is its single static chord; if shadowed there is no
  alternative chord on that surface, so re-label collapses to hide. Hiding is the honest minimal fix.
- **D2 — Identity via the ACTION STRING.** Chip shows iff
  `keymap.action_for(binding, stack) == Some(action_for_command(command.id))`. The action string is the
  shared currency between the palette dispatch table and the keymap. The #204 invariant "every cockpit
  command resolves to a verb" (test `action_for_command_maps_every_row`) guarantees `action_for_command(id)`
  is `Some` for every bound row, so the predicate never spuriously hides an *unshadowed* chip.
- **D3 — CONSUMER-ONLY.** New pure `displayed_binding` composes the existing pure fns; nothing about the
  resolver, the command list, or which rows appear changes. Scoped to the HINT.
- **D4 — Live stack from the dispatch source.** The shim reads
  `self.shell.active_project().active_tab().key_context()` (the exact source app.rs:14503 dispatches
  against) once before the render loop, plus `self.keymap`; the pure seam only receives them.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a command's static chord resolves via `keymap.action_for(chord, stack)` to that command's OWN action on the active stack, `displayed_binding` shall return `Some(chord)`. | unit: Split-Right (⌘⇧L) on a `[Terminal]`/cockpit stack → `Some(⌘⇧L)` |
| REQ-002 | WHEN a command's static chord resolves to a DIFFERENT action on the active stack (a context-scoped row shadowed it), `displayed_binding` shall return `None`. | unit: Split-Right (⌘⇧L) on `[Editor]` → `None` (⌘⇧L = select-all-occurrences there) |
| REQ-003 | WHEN a command's static chord resolves to NO action on the active stack (unbound there), `displayed_binding` shall return `None`. | unit: an Editor-scoped-chord command viewed on `[Terminal]` → `None` |
| REQ-004 | WHEN a command has no static chord (`binding: None`), `displayed_binding` shall return `None` (unchanged — no chip). | unit: a `binding: None` command → `None` |

## Phase Plan
- **P2 Design** — the `displayed_binding` signature + render-site wiring (stack computed once before the
  loop; `&self.keymap`); the regression test plan (REQ-001..004 → unit tests in palette.rs; the render
  swap is mechanical — a pure predicate carries the behavior, so no new headless assertion is required
  unless cheap).
- **P3 Implement** — add `displayed_binding`; swap the render site.
- **P3.5 Inspect** — adversarial: action-string identity; the `binding: None` + `action_for_command == None`
  branches; confirm NO row-filtering / NO resolver change; unshadowed chips still show for all three stacks.
- **P4 Validate** — unit tests cov/MSI 100 on `displayed_binding`; `scripts/gates.sh` green.
- **P5 Complete** — archive, AAR capture, close #306.
