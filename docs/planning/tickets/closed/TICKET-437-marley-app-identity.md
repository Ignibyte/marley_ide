# TICKET-437 — Marley's own app identity

- **Ticket:** LOCAL #437 (chore, workbench shell W1)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/437-marley-app-identity.spec.md
- **Source ticket:** ../../design-notes/workbench-shell-shelf.md · ../../../marley/workbench-shell.md
- **Status:** closed

## Summary
The fork still calls itself Zed, so it shares ~/.config/zed and Zed's database with the stock Zed installed at ~/.local/zed.app. Both apps would overwrite each other's window and sidebar state. Zed supports forks renaming themselves through APP_NAME in crates/paths/src/paths.rs, with a compile-time check that the binary name matches. This ticket renames the app and binary to Marley and copies Chad's current Zed settings into Marley's config directory once.

## Acceptance
cargo run starts a binary named marley that reads ~/.config/marley and writes ~/.local/share/marley, with Chad's settings in place and the stock Zed config untouched. Full EARS in the queued spec.
