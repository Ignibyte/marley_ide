# TICKET-944 — Remote folders, layouts and About as the board draws them

- **Ticket:** LOCAL #944 (feature, the Marley look, opening and connecting)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [944-open-and-connect-as-the-board.spec.md](../../pipeline/completed/944-open-and-connect-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23
  (docs/marley/design-review-2026-10-23/G-system.md, board OpenConnect)
- **Boards:** OpenConnect
- **Size:** M
- **Status:** closed

## Summary

Three dialogs on the OpenConnect board differ from Marley's. Opening a remote folder uses Zed's
Remote Projects modal as it is: no field takes an `ssh://` address, a host has no status dot, and
the upload line says "Uploading remote development server" with no size. The layout choice is a
plain setting row, where the board draws two picture cards. About is Zed's window with the cat line
added: no build time, no line saying how Marley updates, and its Copy button copies the version,
not the install command. This ticket brings the three to the board. The upload's running count
("18 of 31 MB") needs a progress report the SSH upload does not give, so it goes to a follow-up.

## Acceptance

The remote dialog takes an `ssh://` address, shows a dot per host and says what it uploads and how
big it is; Settings shows the two layouts as picture cards; About shows the build line, the update
line, Copy the command and Open the log. Shot in Marley Night and Marley Day.
