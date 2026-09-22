# TICKET-200 — Inline history autosuggestion (ghost text) at the prompt

- **Forge ticket:** #200 (841e188f-5de7-47a0-93a1-c36647b9b243) (feature, M12.2, prompt/autosuggest/history)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** b01def64-0cb9-46f4-a4ee-f1183baf5fec
- **Pipeline doc:** ../../pipeline/active/warp-ghost-text.spec.md
- **Source ticket:** M12.2 sprint #25 (the polish batch)
- **Status:** closed

## Summary
As the user types at the prompt, show a dimmed "ghost" completion drawn from command history (fish/Warp
style); → accepts it. The passive inline hint, distinct from #183 Tab's active popup. Pure
`suggest(prefix, history) -> Option<suffix>` (most-recent entry strictly starting with the prefix → its
remainder; None on empty/no-match/exact), cov/MSI 100; the shim paints the muted suffix after the block
cursor ONLY at end-of-line + suppressed while the tab popup is open, and → inserts it. Display-only until
accepted. Clean-room — muted token.

## Acceptance
A muted ghost appears at EOL for a matching prefix; → accepts it; no ghost on empty/no-match/exact or while
the tab popup is open; the pure `suggest` is exact-value tested. Full EARS (REQ-001..005) in the spec.
