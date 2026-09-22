# TICKET-380 — App ignores ⌘Q — no Quit accelerator; only AppleEvent quit works

- **Forge ticket:** #380 f4d905aa-88dd-4339-aa8d-ef55273fb761 (bug, M25)
- **Owner:** unassigned (queued — claimed at /work)
- **AAR:** pending (opened at claim)
- **Pipeline doc:** ../../pipeline/queued/380-quit-accelerator.spec.md
- **Source ticket:** sprint #36 "M25 — App-Grade QA Hardening"
- **Status:** closed

## Summary
Found in the 2026-07-21 live QA run: pressing ⌘Q in the focused app does nothing — the window
stays open and the process keeps running its normal event loop (sampled healthy; NOT a teardown
hang). `osascript -e 'tell application "Marley" to quit'` exits promptly and cleanly (sessions
reaped, no orphans), so shutdown itself is correct — the gap is purely the missing Quit wiring:
Marley's boot (`marley_app::run`, app.rs:18944) never calls `cx.set_menus` and registers no
gpui action/binding for quit, so the app has no menu bar, no Quit item, and no ⌘Q key
equivalent. Fix: register the standard macOS app menu with a Quit item bound to a gpui `Quit`
action that calls `cx.quit()` — which converges with the already-proven AppleEvent teardown at
`[NSApp terminate:]` (RootView drop → #375 discovery-file removal, #376 subscription join).

## Acceptance
⌘Q in the focused app exits the process cleanly via the SAME teardown path as the AppleEvent
quit (RootView drop, discovery file removed, sessions reaped — never `process::exit`); the menu
bar shows the app menu with Quit ⌘Q; the AppleEvent path keeps working; in-app chords
(⌘W/⌘D/⌘⇧P…) are untouched. Full EARS criteria live in the pipeline spec.
