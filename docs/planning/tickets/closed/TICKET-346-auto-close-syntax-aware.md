# TICKET-346 — Auto-close: syntax-aware suppression (+ the `'` lifetime-bound spike)

- **Forge ticket:** #346 `3cda6024-b130-4792-9f12-eb9ccfcb2506` (feature, M22/editor/auto-close, #338 follow-up)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `0300a225-e00a-4e80-94ce-03543bc9482f`
- **Pipeline doc:** ../../pipeline/completed/346-auto-close-syntax-aware.spec.md
- **Source:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #338 follow-up, was blocked on #315 — now shipped)
- **Status:** closed — shipped; string/comment suppression LOCAL-committed. The `'` lifetime-bound half deferred → forge #362.

## Summary

#338's auto-close pairs even inside strings/comments; typing `(` in a `"string"` or `// comment` still inserts
`()`. #346 makes pairing tree-aware: a pure `Context` threaded into `auto_close::pair_action` (kept pure — the
tree is probed by the caller, never handed to the fn), with `StringOrComment` → no pairing. A thin
`marley_syntax::node_kind_at` probe over the shipped `descendant_for_byte_range().kind()` pattern (#340) +
a Lang-free name-based classifier in the editor crate feed it; app.rs wires a throwaway `HighlightSession`
reparse into the Buffer's insert. `Code` context reproduces #338 byte-identical.

★ Recon correction: the ticket claimed the `'` lifetime-bound positions fall out of the same node-kind lookup.
They do NOT for the typing flow — the tree is parsed from the PRE-insert text, so the `'` being typed has no
`lifetime`/`char_literal` node yet. That case needs ancestry detection or speculative parsing and is a design
SPIKE → likely a follow-up; the string/comment suppression (which the tree cleanly answers) is the shippable
slice.

## Acceptance

Typing a pair char inside a string/comment inserts only the char (no pair); Code positions behave exactly as
#338 (the 26k sweep stays byte-identical); TypeOver is unaffected; the probe classifies string/comment vs code
per supported language. Full EARS in the pipeline spec. Verified by the pure sweep + parse-a-fixture units — no
live drive (chad at the machine). The lifetime-bound positions are spiked at design and, if deferred, filed as a
follow-up.
