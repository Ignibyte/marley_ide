# TICKET-286 — Honor DECCKM (application cursor keys) — the alt-scroll fallback AND encode_key's arrows

- **Forge ticket:** #286 `bf9c4440-69b5-4249-9cbb-09bbae161d4a` (bug, M17)
- **Owner:** autonomous /goal run (sprint #30)
- **AAR:** `7daf1868-02b1-4827-8b7e-6ff41f3f83c6`
- **Pipeline doc:** ../../pipeline/active/286-decckm-app-cursor-keys.spec.md
- **Source ticket:** #280 inspect F2 (the M17 follow-up shelf)
- **Status:** closed

## Summary
Two consumers ignore DECCKM (application cursor-key mode, DECSET 1) project-wide.
(1) The #280 alternate-scroll wheel fallback (`mouse.rs`) always sends CSI A/B
(`ESC[A`/`ESC[B`); the xterm spec sends SS3 A/B (`ESC O A`/`ESC O B`) when
application cursor-key mode is active (vim's smkx, less — the common alt-screen
state). (2) `encode_key`'s cursor keys (`keys.rs`) have the same gap for REAL
unmodified arrow / Home / End presses. Most TUIs parse both forms so impact is
low, but spec-correct is one flag: a `MouseModes.app_cursor` field + a
`session.is_app_cursor()` accessor + an `encode_key(input, app_cursor)` param,
all fed from `TermMode::APP_CURSOR` via the session snapshot; a shared
`cursor_key_bytes(final, app_cursor)` helper the fallback and the unmodified
cursor-key branch both use. Fix BOTH together (a fallback-only fix would leave
arrows inconsistent). MODIFIED cursor keys stay CSI (`ESC[1;<param>X`) per xterm.

## Acceptance
An unmodified arrow / Home / End and the alt-scroll wheel fallback emit the SS3
form (`ESC O X`) when `TermMode::APP_CURSOR` is set and the legacy CSI form
(`ESC[X`) otherwise; a modified cursor key stays CSI in both modes. Full EARS in
the pipeline spec.
