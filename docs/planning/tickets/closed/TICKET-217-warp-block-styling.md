# TICKET-217 — Warp visual parity: command-block styling (hover-reveal affordances)

- **Forge ticket:** #217 (2c09c9bc-f7b4-4a29-9cf8-2f6233b92ef0) (feature, M12.2, warp-parity)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 8ca6ef52-b09e-48b0-b885-f814a707bebe
- **Pipeline doc:** ../../pipeline/active/warp-block-styling.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25
- **Status:** closed

## Summary
Match Warp's command-block styling. Genuine delta: Marley's block affordances (⧉ copy / ↻ rerun) are
always visible; Warp reveals them on hover. Make Marley's affordances hidden by default, revealed on
block-header hover (gpui group-hover) — matching Warp + decluttering. Actions stay reachable via the
#175 right-click menu. The status glyph/command/separator already read Warp-like (unchanged).

## Acceptance
Affordances hidden at rest, revealed + clickable on hover; glyph/command/separator unchanged. Full
EARS (REQ-001..003) in the pipeline spec.
