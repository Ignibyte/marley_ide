# TICKET-515 — A Marley page in the Settings window

- **Ticket:** LOCAL #515 (feature, settings, cross-cutting)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/515-marley-settings-page.spec.md
- **Source ticket:** Chad, 2026-09-25: "WE need a marley settings pane" (with telemetry off by default and configurable, #514) and "6.) Configuration page for marley"
- **Status:** closed

## Summary
Marley's settings are scattered: its one setting of its own, `marley.layout`, is reachable only
in the settings file, and the rest sits among Zed's pages. Zed's Settings window gains a Marley
page, first in its list, that holds Marley's settings: the layout (a dropdown), and the telemetry
toggles under Privacy. `marley: open settings` opens the window on it. Later Marley settings (the
browser, agents, secret redaction) add their sections here.

## Acceptance
The Settings window lists a Marley page first; on it the layout dropdown switches the windows'
layout and the telemetry toggles write their settings; `marley: open settings` opens it.
