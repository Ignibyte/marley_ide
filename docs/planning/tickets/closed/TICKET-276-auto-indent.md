# TICKET-276 — Auto-indent on Enter + Tab/⇧Tab indent-dedent

- **Forge ticket:** #276 2af791c8-03bb-45b6-93e2-3fa91e3443f8 (feature, M17)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 9cfbf20a-0cdd-49a2-85b5-428da6918264
- **Pipeline doc:** ../../pipeline/active/276-auto-indent.spec.md
- **Source ticket:** sprint #30 (forge)
- **Status:** closed

## Summary
The v1 indent layer: Enter clones leading whitespace (caret-clipped);
Tab/⇧Tab line indent/dedent with the selection rebased through the
#269 arithmetic; bare-caret Tab inserts to the next stop. Tab/⇧Tab
become router-handled editor keys (no more literal \t via the IME
fallback); the terminal's Tab completion + BackTab streaming untouched.

## Acceptance
Clone/clip exact; 3-line ops keep the same text selected; stop padding
column-aware; both surfaces route correctly; every op undo-recorded.
Full EARS in the spec.
