# TICKET-218 — Warp visual parity: prompt input-row styling

- **Forge ticket:** #218 (6167542d-7df6-454c-9364-243637980ab9) (feature, M12.2, warp-parity, prompt)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 97bc81fe-e922-470f-8719-099afe411db2
- **Pipeline doc:** ../../pipeline/active/warp-prompt-styling.spec.md
- **Source ticket:** Warp-parity thread (#215); M12.2 sprint #25
- **Status:** closed

## Summary
Match Warp's prompt look. Genuine deltas: Marley's prompt is a filled gray rounded strip with a
bright accent `❯`, a bright cwd, and a thin 2px accent-bar caret; Warp has no strip (prompt on the
pane bg), a dim gray `❯`, a dim cwd breadcrumb, and a solid block cursor. Remove the strip fill/
rounding, dim the `❯`+cwd (git stays accent), and swap the caret bar for a block cursor
(reverse-video the caret grapheme) driven by a new pure split helper. Clean-room — no Warp assets.

## Acceptance
No input-strip box; muted `❯`+cwd (git accent); a block cursor mid-line & at EOL; the pure
caret-split helper covered exact-value. Full EARS (REQ-001..005) in the pipeline spec.
