# TICKET-225 — keycap chip inset fill (surface→background) — palette polish

- **Ticket:** LOCAL #225 (chore, M12.2)
- **Tags:** M12.2, visual, warp-parity, cleanup
- **Created:** 2026-07-09
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 3c04508c-5bf7-4413-9952-41dfdd60a70a)
- **Status:** closed

## Description

#222's `KeyboardShortcut::render` keycap uses `bg(colors.surface)` = the enclosing palette card's surface bg (ΔL 0), so only the 1px border outlines the chip; the muted key text (~5.1:1) carries it. The #222 inspect critic computed that `bg(colors.background)` (a darker inset fill) reads as a proper keycap (the classic inset look) and lifts chip-text contrast to ~6.6:1, with NO downside on the selected accent row (a darker chip only increases contrast there). Trivial one-token change (surface→background in ui_components render/keyboard_shortcut.rs), deferred from #222 (the surface chips already PASS — capture-confirmed legible; the phase-gate blocked a post-validate code change). Also updates the widgets-gallery bin's chip look. Low priority polish.
