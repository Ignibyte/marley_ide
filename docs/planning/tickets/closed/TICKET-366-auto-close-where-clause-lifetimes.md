# TICKET-366 — Auto-close: suppress '-pairing at where-clause + dyn-bound lifetime positions (the #362 deferred half)

- **Ticket:** LOCAL #366 (feature, M22)
- **Tags:** M22, editor, auto-close, syntax, 362-followup
- **Created:** 2026-07-19
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 38c21f44-4c3c-47f4-b8e3-eac7546b5493)
- **Status:** closed (2026-08-11 — pipeline 366-lifetime-where-dyn-auto-close, `4e610839-3f3d-4dbc-9ad5-b969c71f71c3`; both deferred halves shipped via `marley_syntax::speculative_lifetime_at`, the speculative post-insert parse — the ticket's option (b), which the Phase-1 spike proved clean rather than fiddly; the option (a) backward scan died to the substrate)

## Description

#362 shipped the `<...>` type-parameter half of the lifetime-bound `'`-suppression via a `type_parameters` ancestry probe + `Context::LifetimeBound`. Two lifetime-bound position classes remain deferred because the PRE-insert tree has no `<>` structural anchor for them (a Phase-1 spike on #362 confirmed):

- **`where T: 'a`** (and `where T: Send + 'a`): the incomplete `where T: ` doesn't parse to a `where_clause`/`trait_bounds` node — the spike showed the caret falls to `function_item`, NO bound ancestry. So `in_type_parameters` returns false and the `'` still pairs `''`.
- **`dyn Trait + 'a` / `Ref<'a, T>`**: these resolve to `type_arguments`/`generic_type` ancestry (a type-USAGE `<>`), NOT `type_parameters`. `type_arguments` is broader (any `Vec<i32>` usage) — adding it risks a const-generic-block char-literal false-positive (`Foo<{ 'x' }>`), so #362 kept v1 to the zero-false-positive `type_parameters` core.

## The work
- `where`: needs a DIFFERENT signal than ancestry (the pre-insert tree can't help). Options: (a) a backward text scan from the caret — nearest non-space is `:` or `+` inside a `where` region → bound; (b) a speculative post-insert parse. Both fiddly; weigh vs the low value (where-clause lifetimes are rarer; suppression is the safe-direction miss — the pair just isn't withheld, harmless).
- `dyn`/`type_arguments`: extend the probe to also accept a `type_arguments` ancestor, BUT guard against the const-generic-block char (`{ 'x' }` inside `<>`) — check the immediate context isn't a block/const_block.

Lower priority than #362's core: the un-suppressed `''` at these positions is annoying, not wrong (you delete the extra `'` or type over it). References: #362 (the type_parameters half + the spike findings), #346 (the string/comment Context), #338 (blocked_quote). M22, area editor/auto-close/syntax.
