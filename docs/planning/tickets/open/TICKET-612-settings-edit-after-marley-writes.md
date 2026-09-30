# TICKET-612 — A hand edit to settings.json after Marley writes it does not reload

- **Ticket:** LOCAL #612 (bug, workbench shell)
- **Owner:** abb571b4-bf75-463f-889a-b17f7d32f71e
- **Pipeline doc:** none yet
- **Source ticket:** found in #607's Test phase
- **Status:** open

## Summary
Once Marley has written `settings.json` itself (the layout switch does, as do Zed's own writers
such as the theme picker), a later edit of the file by hand or by another program does not
reload until Marley restarts. Seen in #607's Test phase: an e2e probe changed `ui_font_size`
before and after `marley: use zed layout` then `marley: use marley layout`; the first change
applied and the second did not. The log shows `notify::inotify` "unable to remove watch
descriptor" at the moment of Marley's write. The writer replaces the file (a new inode), and the
settings watcher (`settings::watch_config_file` over `fs::fs_watcher`) seems to keep watching the
old one. Start by checking whether upstream Zed has the same behavior on Linux and whether a
newer upstream `fs_watcher` fixes it.

## Acceptance
WHEN Marley has written `settings.json` and the file is then edited outside Marley, the edit
shall take effect without a restart (a scenario: a layout round trip, then an edit to
`ui_font_size`, then a shot showing the new size).
