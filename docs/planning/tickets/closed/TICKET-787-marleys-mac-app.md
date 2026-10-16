# TICKET-787 — Marley's Mac app and its install

- **Ticket:** LOCAL #787 (chore, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [787-marleys-mac-app.spec.md](../../pipeline/completed/787-marleys-mac-app.spec.md)
- **Source ticket:** the same move as #786 (Chad, 2026-10-16, "lets make it happen").
- **Status:** closed

## Summary
`script/bundle-mac` still builds Zed's `zed` binary and signs with Zed's identity, and the dev
bundle metadata in `crates/zed/Cargo.toml` still says `dev.zed.Zed-Dev`, "Zed Dev", the `zed`
URL scheme and Zed's document types. `script/install-marley` is Linux-only (desktop entry, musl
remote server, GNU-only flags). A Mac install gets a `Marley.app` named Marley with the id
`com.ignibyte.Marley` (#760), the `marley` URL scheme (#762) and no claimed document types, signed
ad hoc for the local machine; and an install path that builds into a target directory of the
user's choosing with capped jobs (a 16 GB machine), puts the app in place, leaves a running Marley
alone, and stages the Linux remote server for the same commit, fetched from the machine that
built it, so remote projects on a Linux host keep working.

## Acceptance
On the Mac, the install leaves a `Marley.app` that starts Marley, shows Marley as its name and
id, and opens a remote folder on the Linux box with the server staged for its commit.

## After close
REQ-004 (a remote folder on the Linux box opened from the Mac) waits on the Mac's Local
Network permission for Marley, which only the user grants.
