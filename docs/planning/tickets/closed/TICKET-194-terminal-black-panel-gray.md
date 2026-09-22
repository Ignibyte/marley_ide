# TICKET-194 — Terminal-black pane background + gray panel/dock surfaces

- **Forge ticket:** #194 (1469c76c-b4ac-4072-9d92-57ca7fe96a07) (feature, M12.2)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 209075eb-b1d7-4713-80b6-94078239581f
- **Pipeline doc:** ../../pipeline/active/terminal-black-panel-gray.spec.md
- **Source ticket:** M12.2 "Terminal fidelity & cockpit UX" (sprint #25) — chad Warp-look direction (color half)
- **Status:** closed

## Summary
Recalibrate the dark theme so the terminal pane reads near-black and the
panels/docks/tab-bar/footer read a distinct, slightly lighter gray (chad: "terminal
area black, panel the slight gray"). Keep foreground text + the #31 ANSI palette
legible on the darker background. Clean-room (§20) — observe Warp's look, copy no
assets/hex. **Taste-gated: plan+design propose the exact shades; chad approves them
before implement.**

## Acceptance
Terminal pane near-black + distinctly darker than the gray panels; text + ANSI legible
(WCAG contrast guard); light theme unchanged; values Marley-original. Full EARS
(REQ-001..005) in the pipeline spec.
