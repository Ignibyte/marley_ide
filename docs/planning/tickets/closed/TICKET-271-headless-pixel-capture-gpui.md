# TICKET-271 — Pixel capture for the headless lane — adopt render_to_image on a gpui upgrade

- **Ticket:** LOCAL #271 (chore, M-unset)
- **Tags:** testing, gpui, follow-up
- **Created:** 2026-07-12
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id f5d15b5c-1231-4b05-b350-9e7855c8540f)
- **Status:** closed

## Spike verdict (2026-08-14 — TICKET-424)

The #424 spike measured the get-it-early paths and decided **WAIT with a
watch** (memo + adoption recipe + traps in `tickets/closed/TICKET-424-…`;
watch = crates.io gpui max_version > 0.2.2). The environmental risk is
PRE-CLEARED: offscreen Metal + CoreText enumeration + glyph rasterization
all proven byte-identical over ssh (no WindowServer) — when the release
lands, adoption is a migration, not a gamble. Route the lane through
`HeadlessAppContext` (injected `MacTextSystem` + `current_headless_renderer`);
the `VisualTestAppContext` twin is in-session-only.

## Description

The #264 slice adopted gpui 0.2.2's VisualTestContext (headless INPUT+STATE — shipped as crates/marley_app/src/headless_drive.rs). The PIXEL half is impossible on 0.2.2: the test platform's draw(&Scene) is a no-op and render_to_image/capture_screenshot don't exist there (they're in the newer Zed tree — see docs/marley_architecture/marley_visual_harness.md "Future upgrade"). WHEN a gpui release ships render_to_image: (1) bump the dep; (2) add offscreen screenshot asserts to the headless lane; (3) migrate gate:15's screenshot-baseline half off the headed screencapture shim; (4) retire the frontmost/shadow-offset drive.swift hazards for capture (keep it only for true-OS-integration checks). Also from #264 inspect F4: the VIRGIN boot path drops its unused boot TerminalSession ON-THREAD at the end of new_in (the restored path reaps off-thread) — fold an off-thread reap into that arm while touching the boot ctor. Reference §20: N/A — own-dependency adoption.

## Resolution
Closed on 2026-09-28 by Chad's decision ("close both"). The pixel proof it asked for comes from the e2e scenarios now: `script/e2e.sh` runs the real Marley and each scenario's shots are read (CONSTITUTION §7), so no pixel capture inside a test is needed.
