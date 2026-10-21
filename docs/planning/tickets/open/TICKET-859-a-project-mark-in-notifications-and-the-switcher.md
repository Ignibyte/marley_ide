# TICKET-859 — A project's mark in its notifications and the switcher

- **Ticket:** LOCAL #859 (feature, the Marley look, W1.2 follow-up)
- **Owner:** 0e8e9f94-0ce3-4a15-aeeb-252b0ba66067
- **Pipeline doc:** (not planned yet)
- **Source ticket:** split from TICKET-831 while planning it (2026-10-20): the ProjectLogo board
  shows the mark on a project's desktop notifications and in the switcher, which #831 leaves out.
- **Boards:** ProjectLogo
- **Size:** M
- **Status:** open

## Summary

Desktop notifications carry no image today: `gpui::SystemNotification` has a tag, a title, a body
and actions, and the Linux side builds a `notify_rust` notification without an icon. Give the
notification an optional image (a file path, or the mark's pixels) in gpui and on each platform,
and pass a project's mark (#831's shared cache) from `notifications.rs`. The switcher's terminal
and thread rows name their project as text; draw the mark beside that name.

## Acceptance

A notification from a project with a logo shows it; the switcher's rows show their project's mark.
The visual check shoots the switcher in Marley Night and Marley Day, and the notification on the
scenario's private notification server (the hint it received).
