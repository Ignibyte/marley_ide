# TICKET-265 — B1 KeyContext: context-scoped keybindings (fix the ⌘D collision)

- **Forge ticket:** #265 73614d2c-09e9-46d9-9fb2-f3da97a08c94 (feature, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 6f75fb5a-6d6c-4584-bd60-c54282656727
- **Pipeline doc:** ../../pipeline/active/265-keycontext.spec.md
- **Source ticket:** sprint #29 "M16 — Cleanup + Editor Frontier" (forge)
- **Status:** closed

## Summary
Make Marley's keymap context-aware (the Zed/gpui KeyContext model as
Marley-original pure code in keymap.rs): bindings carry a context tag, the app
publishes a context stack from the focused surface, resolution is
deepest-match-then-order, and chord uniqueness becomes per-context. Fixes the
two live collisions — ⌘D (Terminal→new-terminal vs Editor→select-next-match,
the latter net-new single-selection v1: select word under caret, repeat jumps
to next occurrence with wrap) and ⌘F (Terminal-only find; an editor tab no
longer opens the terminal find bar) — and gives every future editor-only chord
a collision-free home.

## Acceptance
⌘D splits by surface (terminal: +pane; editor: word selection, repeat
advances, wrapping); ⌘F is terminal-scoped; deepest-match precedence + global
fallback proven in units; per-context uniqueness guard; all 41 existing chords
regress-free; driven capture proves the split live. Full EARS in the spec.
