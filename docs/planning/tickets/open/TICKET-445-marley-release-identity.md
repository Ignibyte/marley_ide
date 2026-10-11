# TICKET-445 — Marley's own release identity: keyring, updater, app id, URL scheme

- **Ticket:** LOCAL #445 (feature, packaging: the release identity, slice 1 of 5)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** [445-marley-release-identity.spec.md](../../pipeline/queued/445-marley-release-identity.spec.md)
- **Source ticket:** the #437 inspect ledger (`../../pipeline/completed/437-marley-app-identity.notes.md`).
  Chad, 2026-10-11, answering "Marley's own identity: its own app id, keyring name, `marley://`
  links and updater … it also unblocks building Marley's remote server, so SSH projects work":
  "7.) yes". Plan: [finish-line shelf](../../design-notes/finish-line-2026-10-11.md) (row 7, the
  release step) and the first recommendation (M1) of
  [harness-and-zed-remote-2026-10-11.md](../../design-notes/harness-and-zed-remote-2026-10-11.md).
- **Status:** open (held since 2026-10-01: "wait on 445 that way if there is any bugs we wont
  have to cut multiple releases"; lifted 2026-10-11: built last of the build-now items, as the
  release step)

## Summary
Marley ships on Zed's `dev` channel, and on `dev` Zed's client can neither download a remote
server nor check one, so Marley opens no SSH project and no dev container: a release build stops
with "ZED_BUILD_REMOTE_SERVER is not set and no remote server exists at …"
(`crates/remote/src/transport/ssh.rs:1077-1086`, `docker.rs:278-287`). This ticket's slice gives
Marley a server of its own. `script/install-marley` builds `remote_server` for Linux x86_64 from
the same tree and stages it where Zed caches downloaded servers. On `dev` the client names the
host's server by Marley's commit, uploads the staged one when the host lacks it, and replaces a
server whose `version` answers another commit.

The rest of the identity is four more slices, each its own ticket, in this order: the app id and
the channel-keyed data; Marley's own keyring label; the `marley://` scheme and the desktop entry;
the updater. The queued spec's Out section describes each.

Found while specifying, against this ticket's first summary:
- The directories are Marley's own since #437 (`APP_NAME = "Marley"`). What a move off `dev` would
  shift are the channel-keyed pieces inside them: the database `db/0-dev`, the socket
  `zed-dev.sock`, the `"dev"` block of the settings, and the dev credentials file. Staying on
  `dev` and changing what Marley's build calls itself moves none of them.
- The keyring is already shared on `dev`. The System One layer calls the platform's credential
  store directly (`crates/marley_workbench/src/system_one.rs:501`, `546`, `560`), so its keys sit
  in the Secret Service under stock Zed's fixed label `zed-github-account`
  (`crates/gpui_linux/src/linux/platform.rs:57`).
- The debug-asset need is settled: since #502 the daily Marley is the installed release build,
  which embeds its assets.

## Acceptance
With nothing staged on the host, `just install` and then opening a folder on an SSH host opens
the project: the host gets `~/.zed_server/zed-remote-server-dev-<commit>`, and a server there from
another commit is replaced. Without a staged server the connection fails with a message that
names the install step.
