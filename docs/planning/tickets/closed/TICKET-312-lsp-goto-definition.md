# TICKET-312 — LSP go-to-definition (F12) + a NavStack jump-back (⌃-)

- **Forge ticket:** #312 ff827668-3ded-4e51-9e0b-465184afbcca (feature, M20)
- **Owner:** fabd8254-8504-4118-9e8e-8827ad24687a
- **AAR:** 19a888c7-9045-451e-a00b-155378a34939
- **Pipeline doc:** ../../pipeline/active/312-lsp-goto-definition.spec.md
- **Source ticket:** forge #312 (M20 batch #308-317)
- **Status:** closed

## Summary
Press F12 on a symbol → land on its definition (across files), centered; press ⌃- → jump back. Multiple
results open a `path:line` picker. Built on the #311 general request/response path (its second
consumer): `RequestPurpose::Definition`, a pure `parse_definition_result` normalizing the three LSP
response shapes (`Location | Location[] | LocationLink[]`), a pure `NavStack` (push/pop, cap 50, dedupe),
and a DEFERRED center (the #273 open-then-scroll-in-one-frame trap). Quiet flash on not-found/not-ready.
**⌘-click-to-jump is DEFERRED** — ⌘-click is the shipped multi-cursor gesture (chad chose F12-only).

## Acceptance
F12 sends definition at the caret; the 3 response shapes normalize; a single result opens+jumps+centers;
multiple → a picker; a goto pushes the NavStack and ⌃- returns (cap 50, dedupe); no-definition/not-ready
→ a quiet flash. Full EARS in the pipeline spec.
