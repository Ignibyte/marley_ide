# TICKET-362 — Auto-close: suppress `'`-pairing at lifetime-bound positions

- **Forge ticket:** #362 `bc01c12e-931f-4e92-a9da-44a82d603ee6` (feature, M22/editor/auto-close/346-followup/syntax)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `865d79f1-e854-4f82-bd8e-6781ec822db5`
- **Pipeline doc:** ../../pipeline/active/362-lifetime-bound-auto-close.spec.md
- **Source ticket:** the follow-up goal `/work 360,361,362,363,364` (the deferred half of #346)
- **Status:** closed

## Summary
#338's one-char `blocked_quote` guard suppresses `'`-pairing after `&`/`<`/identifier (`&'a`, `Foo<'a>`, `don't`)
but NOT at the bound positions `fn f<T: 'a>` / `fn f<'a>` — their `prev` is a space, identical to `let c = 'x'`,
so no one-char rule separates a lifetime from a char literal. #346 shipped the string/comment suppression via a
syntax `Context` but deferred this cut (the pre-insert tree has no `lifetime` node while typing). #362 adds a pure
`marley_syntax` ancestry probe: a Phase-1 spike confirmed the pre-insert tree DOES resolve a `type_parameters`
ancestor at the `<...>` declaration-site positions (the `<>` delimiters anchor the node even with an incomplete
inner bound), so a caret inside `type_parameters` typing `'` is a lifetime → suppress the pair. Threaded via a new
`Context::LifetimeBound` variant that suppresses only `'` (a non-`'` opener like `(` at `T: Fn(` still pairs).
The `where T: 'a` and `dyn Trait + 'a` positions do NOT resolve pre-insert (no structural anchor / broader node)
and stay deferred.

## Acceptance
Typing `'` inside a `<...>` type-parameter list (`fn f<'`, `fn f<T: '`, `struct S<T: '`) inserts a bare `'` (no
pair); typing `'` at a char-literal position (`let c = '`) still pairs `''`; a non-`'` opener at a bound still
pairs; the #346 string/comment suppression is unchanged.
