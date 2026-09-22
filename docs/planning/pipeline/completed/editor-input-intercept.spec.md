---
pipeline_id: 277fa58a-b83d-489b-aa6e-9c0c8ffca4a8
ticket: forge#251 (b97191e2-eff7-4db8-878a-90e7c1709fdc) · local docs/planning/tickets/open/TICKET-251-editor-input-intercept.md
aar_id: 9269d495-736b-41dc-8db0-466860c4cb4c
status: Phase 5 — Complete PASS
title: Editor focus + input intercept — type into the buffer, the caret moves (Enter⇒\n)
type: feature
milestone: M15
references: [forge#249, forge#250, forge#257, marley_editor, input::apply_key]
---

## Title
Make the editor tab CAPTURE keystrokes: when an editor tab is active, route a plain key to the active file's
`Buffer` + caret (reuse the tested `input::apply_key`, with the ONE editor divergence — Enter inserts `\n`, not
Submit) and RETURN, instead of leaking to a hidden terminal. The #250 caret starts MOVING + text inserts → the
#250 faithful render redraws it live. THE marquee interaction — the first WRITE path to the editor buffer.

## Scope
### In
- **A pure `input::apply_editor_key(&mut Buffer, &mut CharOffset, Key) -> KeyOutcome`** — the editor's key
  application. Enter ⇒ insert `\n` (multi-line editor), every other key delegates to `apply_key` (char insert,
  Backspace, Left, Right). Implemented as: map `Key::Enter → Key::Char('\n')`, else pass through to `apply_key`
  (`apply_key(Char('\n'))` already inserts a newline + advances — one tested path).
- **The `on_key_down` editor-routing branch** (app.rs, after the overlay guards, BEFORE the terminal routing):
  when the active tab is an editor AND a plain key (no ⌘/ctrl modifier), parse the `Keystroke → Key` (reuse the
  prompt's existing parse), call `apply_editor_key(surface.active_buffer_mut(), surface.active_caret_mut(), key)`,
  `cx.notify()`, and RETURN (don't leak to the terminal).
- **The capture contract:** the editor OWNS plain keys (printable + Backspace/Left/Right/Enter) — even an
  unhandled plain key (Up/Down for now) is captured (swallowed), NOT leaked. **⌘/ctrl chords fall through** to
  the keymap (⌘W/⌘P/⌘D still work). Overlays (finder/palette/renaming) capture keys FIRST (their existing
  early-returns are unchanged).

### Out (explicitly deferred)
- **Up/Down + Home/End + word-wise movement** (#257 keybinding parity — needs multi-line/line-aware movement;
  `movement.rs` is single-line today). #251 wires only Char + Backspace + Left/Right + Enter⇒`\n`.
- **Selection** (#255), **mouse click→caret** (#254), **save/dirty** (#252), **undo** (#253), **copy/paste**
  (#256), **caret blink** (polish).

## Reference (§20)
**Warp — focus routing between the command input and the panes/blocks (which surface owns the keystroke).**
Warp routes a keystroke to the FOCUSED surface: typing into the command input edits that input; the editing
FEEL Marley mirrors — a printable inserts at the caret + the caret advances one cell, Left/Right move one cell,
the caret tracks the insertion point (the `docs/warp_architecture/observed/250-warp-monospace-grid-caret.png`
grid+caret capture from #250). #251 gives Marley's editor tab that same "the focused surface owns the key"
routing (editor tab active ⇒ keys edit the file buffer, not a terminal). Clean-room: observe the routing
behavior; reuse `input::apply_key` (in-repo, permissive) + gpui; no Warp source read. The multi-line editor
(Enter⇒`\n`) is a Marley file-editor concern with no command-input analog — noted.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the pure seam is `apply_editor_key`** (input.rs), a thin wrapper over the tested `apply_key`: Enter →
  `Char('\n')`, else `apply_key(key)`. One divergence, maximal reuse; cov/MSI 100 (Enter inserts `\n`; a Char
  delegates + does NOT submit; Backspace/Left/Right delegate).
- **D2 — the editor branch guards on (active-tab-is-editor) && (no ⌘/ctrl modifier).** Plain keys → the editor
  (captured, handled-or-ignored, return). ⌘/ctrl chords → fall through to the keymap (shortcuts unaffected).
  Overlays already return first (unchanged) — the editor branch sits after them, before the terminal routing.
- **D3 — reuse the prompt's `Keystroke → Key` parse** (the existing on_key_down terminal path builds a `Key`;
  the editor branch reuses that parse, targeting `active_buffer_mut`/`active_caret_mut` instead of the terminal).
- **D4 — no new focus flag for v1.** "The editor tab is focused" == "an editor tab is active + no overlay" (the
  overlay early-returns + the active-tab check). A `focus: editor` status indicator is optional polish (design
  decides; likely a tiny status-bar tweak or deferred).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | When an editor tab is active, a plain character key shall insert at the caret + advance it (the render shows it). | pure unit + driven |
| REQ-002 | Backspace shall delete the char before the caret (no-op at offset 0). | pure unit |
| REQ-003 | Left/Right shall move the caret one char, clamped to `[0, len_chars]`. | pure unit |
| REQ-004 | Enter shall insert `\n` + advance the caret (NOT submit — the prompt's Enter⇒Submit is unchanged). | pure unit + driven |
| REQ-005 | Keys shall NOT leak to a terminal when an editor tab is active (plain keys captured); ⌘/ctrl chords shall still reach the keymap; and the terminal/prompt input (`apply_key`⇒Submit) shall be UNCHANGED when a terminal tab is active. | driven + review |

## Phase Plan
- **P2 Design** — confirm D1-D4; the `apply_editor_key` signature + the on_key_down branch placement (after the
  overlay guards, before the terminal route) + the exact modifier guard + the reused `Keystroke→Key` parse; the
  pure test matrix; `cargo mutants --list`. Confirm the §20 routing match.
- **P3 Implement** — `apply_editor_key` (input.rs) + the on_key_down editor branch (app.rs shim); `cargo check`.
- **P3.5 Inspect** — critics: the Enter⇒`\n` divergence is correct + the prompt's Submit is untouched; the
  capture contract (plain captured, ⌘/ctrl fall through, terminal unaffected); no overlay regression; caret math.
- **P4 Validate** — pure units cov/MSI 100 on `apply_editor_key`; DRIVEN (mac unlocked, app boots into an editor
  tab) — type a word → it appears + the caret advances; Left → caret back; Enter → newline; a terminal tab still
  types to the terminal. Gate green.
- **P5 Complete** — CHANGELOG + app_shell/editor doc; AAR; close #251; archive.
