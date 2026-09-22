# TICKET-222 — Warp visual parity: command-palette & overlay styling

- **Forge ticket:** #222 (a25d9c71-3ee0-4c01-ae97-ae5f7f2bb418) (feature, M12.2, warp-parity, palette)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 456b5924-efd2-4101-b7c0-e553d69866de
- **Pipeline doc:** ../../pipeline/active/warp-palette-chips.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25 — the FINAL /work 195-222 ticket
- **Status:** closed

## Summary
Match Warp's command-palette content. #221 rounded/framed the palette card; this is the content — the
shortcut is rendered as plain space-joined text ("cmd b"), not Warp's bordered rounded keycap chips.
ui_components has `KeyboardShortcut::render` (a chip per key) but it's a bare surface bg and the palette
doesn't use it. Wire the palette to it, upgrade it to a real keycap (border + rounded + padding + gap +
muted text), and right-align the chips in the row. Reuses the pure `KeyboardShortcut::parse`/`keys`.
Clean-room — tokens only. Deferred: the backdrop scrim, the search-field styling, the selected highlight.

## Acceptance
The palette shortcuts render as right-aligned, bordered, rounded keycap chips (one per key); the pure
parse/keys/titles/highlight/filter unchanged. Full EARS (REQ-001..004) in the pipeline spec.
