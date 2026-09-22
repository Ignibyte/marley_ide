# TICKET-445 — Marley's own release identity: keyring, updater, app id, URL scheme

- **Ticket:** LOCAL #445 (chore, packaging)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (not specced)
- **Source ticket:** the #437 inspect ledger (`../../pipeline/completed/437-marley-app-identity.notes.md`)
- **Status:** open

## Summary
#437 gave the fork its own directories (`APP_NAME = "Marley"`) and binary, but some of Zed's
identity still depends on the release channel staying `dev`. On any other channel the fork would
take stock Zed's Secret Service items (the fixed `zed-github-account` label), run Zed's updater
(which would rsync stock Zed over `~/.local/zed.app` and restart as stock Zed), and use stock
Zed's app id `dev.zed.Zed`. `zed://` links opened from Marley already land in stock Zed, which owns
the scheme on this box, and the CLI's bundled uninstaller would delete stock Zed's files. A `paths`
test keeps `crates/zed/RELEASE_CHANNEL` at `dev` until this ticket lands. It waits until Marley
ships a package or needs a non-`dev` build.

The #437 live drive found one more thing a daily-driver Marley needs. A debug build reads its
assets (default settings, keymaps) from the checkout at run time and finds the checkout by a
`.git` above the executable or the working directory (`util::dev_repo_root`). This box builds
into `/mnt/fast/target`, outside the checkout, so a debug `marley` started from a launcher (whose
working directory is `$HOME`) panics with "dev asset loading requires running from within the
checkout". Started from inside the checkout it runs. A build for daily use needs the assets
embedded (a release build, or the `debug-embed` feature) or a launcher that starts it there.

## Acceptance
On every release channel, Marley uses its own keyring label, its own update source or a disabled
updater, its own app id and desktop entry, and handles `zed://` and `marley://` links itself; the
CLI is built with `no-bundled-uninstall`. After that the `RELEASE_CHANNEL` guard test can go.
