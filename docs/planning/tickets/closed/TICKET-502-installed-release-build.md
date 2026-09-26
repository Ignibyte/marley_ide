# TICKET-502 — An installed release build of Marley

- **Ticket:** LOCAL #502 (chore, packaging, cross-cutting)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/502-installed-release-build.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 8 of the list after the browser waves)
- **Status:** closed

## Summary
Chad runs the debug `marley` out of the shared target directory, so a `cargo clean` or a build that fails half way takes his editor with it, and every debug build is slow to use. A script builds `marley` in release and installs a copy to `~/.local/bin/marley` with a desktop entry, so the launcher starts it; running the script again after a pull reinstalls it. The `dev` channel stays, so the installed build shares Marley's settings, database and sessions with the debug one.

## Acceptance
`just install` builds the release `marley`, installs it to `~/.local/bin/marley` and writes a desktop entry the launcher lists; the installed binary starts in the Marley layout.
