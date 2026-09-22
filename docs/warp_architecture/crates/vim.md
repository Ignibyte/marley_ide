# vim

> Per-crate reference (Marley round 2). Crate dir: `crates/vim`. Marley is forked from Warp (warpdotdev/warp).

| Field | Value |
| --- | --- |
| Subsystem | [editor-and-text](../subsystems/02-editor-and-text.md) |
| License | AGPL v3 (workspace `license = "AGPL-3.0-only"`; no per-crate LICENSE marker) |
| Internal deps | 3 |
| Used by | 2 |

## Purpose

`vim` implements Warp's Vim modal-editing engine: a **finite-state automaton**
that consumes keystrokes, tracks modal state (Normal/Insert/Visual/Replace),
accumulates pending commands (counts, operators, registers, text objects), and
emits semantic **`VimEvent`s** describing what the editor view must do. It also
provides the text-traversal primitives Vim motions need — word/paragraph
iteration, find-char, matching-bracket, and text-object boundary resolution —
generically over any buffer implementing `warp_core`'s `TextBuffer`.

It exists to keep Vim *semantics* (parsing `d2aw`, `ci"`, `f x ;`, named
registers, dot-repeat) cleanly separated from *editing mechanics* (the actual
buffer mutation, done by the consumer that implements `VimHandler`). The crate is
the keymap-to-intent layer; the editor crate is the intent-to-mutation layer.

## Key types, modules & public API

Crate root (`src/lib.rs`) re-exports motion helpers and the `vim` + `register`
modules:

- **`vim::VimFSA`** — the state machine. Holds `mode: VimMode`, `showcmd`,
  pending action/operand counts, pending visual object, `last_find_motion`,
  active `register`, and `dot_repeat_event`. Feeds on keystrokes and produces
  `VimEvent`s.
- **`vim::VimMode`** — `Normal`, `Insert` (default), `Visual(MotionType)`,
  `Replace`.
- Motion/command vocabulary (all in `vim.rs`): `Direction` (`Forward`/`Backward`,
  with `opposite()`), `WordMotion` (`WordBound`, `WordType`), `LineMotion`,
  `CharacterMotion`, `FirstNonWhitespaceMotion`, `FindCharMotion` +
  `FindCharDestination`, `VimMotion`, `MotionType`, `VimTextObject` +
  `TextObjectType` / `TextObjectInclusion`, `QuoteType`, `BracketChar` /
  `BracketType` / `BracketEnd` (with `complements()`), `InsertPosition`,
  `ModeTransition`.
- **`vim::VimEvent`** + **`vim::VimEventType`** — the emitted intents; operators
  via `VimOperator` and `VimOperand`.
- **`vim::VimModel`** — a `warpui_core` `Entity`-backed model wrapping the FSA:
  `new()`, `state() -> VimState`, `typed_character(c, ctx)`,
  `keypress(&Keystroke, ctx)`, `force_insert_mode(ctx)`, `interrupt(ctx)`.
  `VimState<'a>` is the borrowed snapshot (`mode`, …).
- **`vim::VimHandler`** (trait) — **the integration seam**. The consuming editor
  implements it; the FSA calls back into it: `insert_char`, `navigate_char`,
  `navigate_word`, `navigate_line`, `first_nonwhitespace_motion`, `find_char`,
  `navigate_paragraph`, `operation(operator, count, operand, register, replacement, ctx)`,
  `replace_char`, `toggle_case`, `search`, `cycle_search`,
  `search_word_at_cursor`, `ex_command`, `keyword_prg`, `visual_operator`, …
  Plus `vim::VimSubscriber` for observers.
- **`register`** module — `valid_register_name(c) -> bool` and
  `BLACK_HOLE_REGISTER: char = '_'` (Vim's `"_` register).
- Buffer-traversal helpers (generic over `T: TextBuffer`, returning
  `string-offset` `CharOffset`s):
  - `vim_word_iterator_from_offset` (+ `WordHeadsVim`, `WordTailsVim` iterators).
  - `find_next_paragraph_end`, `find_previous_paragraph_start`.
  - `vim_find_char_on_line`.
  - `vim_find_matching_bracket`.
  - `text_objects::*` — word/quote/block/paragraph text-object resolution
    (`src/text_objects/{word,quote,block,paragraph}.rs`).

## Depends on (internal)

- [string-offset](./string-offset.md) — typed `CharOffset`/byte-offset
  arithmetic; all the motion iterators return and operate on these offsets.
- [warp_core](./warp_core.md) — provides the `TextBuffer` abstraction the
  motions traverse, plus `safe_info` logging.
- [warpui_core](./warpui_core.md) — the UI runtime: `Entity`, `ModelContext`,
  `ModelHandle`, `ViewContext`, and `keymap::Keystroke`. `VimModel`/`VimHandler`
  are built on these, so the engine is bound to Warp's entity/view framework.

## Used by (internal dependents)

- [warp](./warp.md) — wires Vim mode into the app.
- [warp_editor](./warp_editor.md) — implements `VimHandler` against real editor
  buffers and drives the FSA.

Total: 2 dependents.

## Related crates

- [warp_editor](./warp_editor.md) — the primary consumer / `VimHandler` impl;
  read alongside this to see motions become buffer edits.
- [warp_core](./warp_core.md) — owns `TextBuffer`, the substrate the motions run
  on.
- [string-offset](./string-offset.md) — the offset types threaded through every
  traversal helper.

## Marley relevance

**Classify: KEEP.**

Self-contained modal-editing logic with no auth, no network, and no user-facing
Warp branding — keep verbatim. It is only coupled to Warp via `warpui_core`'s
entity/view types (`Entity`, `ViewContext`, `Keystroke`), so its fate is tied to
that framework rather than to anything Marley wants to strip.

- Goal (1) *expand the UI surface with a custom panel*: if a new Marley panel
  embeds an editable buffer, it gets Vim keybindings "for free" by implementing
  `VimHandler` and feeding keystrokes to a `VimModel` — no changes to this crate
  needed.
- Goals (2) session spawn/write/read, (3) de-auth/login stub, (4) de-Warp
  rebrand: **N/A here.** No session, auth, or brand surface lives in `vim`. A
  package rename to e.g. `marley_vim` is cosmetic and low-value (only 2
  dependents) — defer to a coordinated mass rename, do not touch in isolation.

Net: a leave-it-alone crate. The only realistic Marley change is *extending* it
(new motions / commands) if the custom panel wants richer editing — but that is
opt-in, not required for the four core goals.

## Notes / gotchas

- **`VimHandler` is the contract, not the implementation.** This crate decides
  *what* should happen; it never mutates a buffer itself. All editing side
  effects are delegated to the consumer's `VimHandler` impl, parameterized by
  `warpui_core::ViewContext<Self>`. Test/port work must mock that context.
- **Default mode is `Insert`, not `Normal`** (`VimMode` derives `Default` on the
  `Insert` arm) — appropriate for a terminal where typing is the common case;
  surprising if you expect Vim-classic Normal-on-start.
- **`vim.rs` is large (~2060 lines)** and holds the bulk of the command-parsing
  logic; the motion submodules are small and generic.
- Traversal helpers are generic over `T: TextBuffer + ?Sized` and use
  `string-offset` `CharOffset`s — mind char-vs-byte offset distinctions when
  integrating.
- Pulls a direct third-party dep `unindent = "0.2.4"` (used in tests) outside the
  workspace dependency table; otherwise deps are workspace-pinned.
- Dot-repeat (`.`) and register state live inside `VimFSA`
  (`dot_repeat_event`, `register`); resetting/interrupting must go through
  `VimFSA`/`VimModel::interrupt` to keep them consistent.
