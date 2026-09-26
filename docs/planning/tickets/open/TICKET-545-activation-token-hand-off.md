# TICKET-545 — A second launch hands its activation token to the running Marley

- **Ticket:** LOCAL #545 (feature, the Marley app: startup, after #513)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** (none yet)
- **Source ticket:** found in #513's Test, 2026-09-26: the running Marley asked to come forward,
  and sway did not act on it
- **Status:** open

## Summary
Since #513 a second launch hands its paths to the running Marley, which asks the compositor to
bring a window forward. gpui asks for that activation token itself, from the window it activates
and with its last pointer press, so a compositor that checks tokens against the seat's focus
(sway) declines it while another app or window has the focus; Omarchy's Hyprland focuses anyway
(`focus_on_activate`). The Wayland way is for the launcher's token to travel: a desktop launcher
starts the second launch with `XDG_ACTIVATION_TOKEN`, made for that click. The second launch
would send it with its hand-off (a query on `zed://open` and on each `file://` URL, or a datagram
of its own), and the running Marley would activate its window with that token instead of asking
for one. That needs a gpui call to activate with a given token, and the listener to pass the
token through.

## Acceptance
Started from a launcher that provides an activation token, a second Marley brings the running
Marley's window forward under a compositor that checks tokens (sway in the e2e run), and the
Wayland trace shows the launcher's token in the `activate` request.
