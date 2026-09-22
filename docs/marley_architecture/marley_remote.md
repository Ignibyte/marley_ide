# `marley_remote`

> Per-crate architecture note — **round 4 refresh · 2026-07-12 · current to M15.**
> Provenance: **`[Marley-original]`** (INVENT, `serde` for the saved-host setting; otherwise std-only).
> **The ssh-argv remote seam — no cloud, no secrets, no relay.** Marley's "remote" is deliberately thin:
> spawn the user's OWN `ssh` client as a terminal pane and let `ssh` own all security. This is NOT Warp's
> `remote_server` / isolation-platform lineage (those were SKIP'd); a future clean REIMPLEMENT of a
> remote-dev relay/transport (on `russh`) is the M5 `remote-connection-seam` intake, separate from this.

The remote (ssh) seam — a pure domain crate (like [`marley_agent`](./marley_agent.md); a third peer,
`marley_forge_client`, was deleted at #411 — the scrap-forge rip). A remote pane is just a normal `TerminalSession`
running `ssh`; `ssh` owns ALL security — keys, `known_hosts`, auth, passwords — and this crate handles NO
secrets. It only parses a typed target and produces an argv.

## Surface (M3.A #83/#86/#87, SHIPPED)
- `SshTarget { user: Option<String>, host: String, port: Option<u16> }`.
- `parse_ssh_target(&str) -> Option<SshTarget>` — parse `[user@]host[:port]`, including bracketed IPv6
  (`[::1]:22`). Returns `None` on empty / empty-user / empty-or-bad-or-overflow port / empty-host / a bare
  (unbracketed) IPv6 (bracket it as `[::1]`) / **a leading-`-` host or user**.
- `ssh_command(&SshTarget) -> Vec<String>` — the argv `["ssh", ("-p", port)?, "--", [user@]host]`.
- `RemoteStatus { Connected, Disconnected }` + `remote_status_from(exited) -> RemoteStatus` +
  `remote_status_glyph(status) -> &'static str` (`⇄` / `✗`).
- `remote_badge(host: &str, status: RemoteStatus) -> String` — `"{glyph} {host}"`.
- `RemoteHost { name, target }` (serde) — one `[[remote.hosts]]` TOML entry; `RemoteAction { label, target }`;
  `remote_palette_actions(&[RemoteHost]) -> Vec<RemoteAction>` (drops invalid targets, labels
  `"connect: {name} → {target}"`).

## Security — two-part injection guard
An argv (`Vec<String>`, never a shell string) defeats SHELL injection. But that alone does NOT stop
OPTION-SMUGGLING: `ssh` re-parses any argument starting with `-` as a flag, so a destination like
`-oProxyCommand=evil` would become an option, not a host (the CVE-2017-1000117 class — a latent RCE). Two
guards, both applied: (1) `parse_ssh_target` REJECTS a leading-`-` host/user; (2) `ssh_command` inserts
`--` before the destination (defends even a hand-built `SshTarget` that skipped the parser). See
`AD`/`PR-claude-subprocess-positional-arg-guard-leading-dash-and-double-dash`.

## Open a remote pane (#84, SHIPPED)
`SessionOptions` gained an `args: Vec<String>` field (the PTY spawn already accepted args — it was
hardcoded empty), so a session can run any program. ⌘⇧O reads the composed prompt line → `parse_ssh_target`
→ `ssh_command` → a `spawn_remote_session` splits a pane running the ssh argv (`shell = "ssh"`, PATH-
resolved; `args = ["-p", p?, "--", dest]`). A remote pane IS a normal `TerminalSession`, so blocks/input/
badges all reuse. The local zsh spawn passes `args: []` (unchanged).

## Remote-pane badge (#85, SHIPPED)
The app tags each remote pane in `RootView.remotes: HashMap<PaneId, Remote { host, status }>` — inserted
when ⌘⇧O opens the pane, removed at both close paths (the pump auto-close + close-pane, beside the `agents`
cleanup). `remote_badge` renders as a top-LEFT accent pill (the agent badge is top-right; a pane is agent
XOR remote). PaneId is monotonic (never recycled), so a tag can never badge the wrong pane.

## Connection status (#86, SHIPPED)
`RemoteStatus`/`remote_status_from`/`remote_status_glyph` are pure. When the #67 pump sees a remote pane's
session exit, it branches BEFORE the auto-close: flips the pane to `Disconnected` + flashes
"disconnected: {host}" + KEEPS the pane (never auto-closes a remote — a visibly-dead remote beats a
vanishing pane). The flip fires exactly once (the transition guard makes the per-frame `Err(Disconnected)`
re-entry idempotent; `Err` doesn't set `dirty`, so no repaint churn).

## Saved hosts + the connect palette (#87, SHIPPED)
`RemoteHost { name, target }` is one `[[remote.hosts]]` TOML setting entry; `remote_palette_actions`
validates each target (drops the invalid, never a panic) and labels `"connect: {name} → {target}"` (pure,
cov/MSI 100). The app loads the setting at boot, builds a palette `Command` per action in a
`CONNECT_BASE (1000+)` id range, and on Enter maps `id - CONNECT_BASE` → the action's target → the shared
`open_remote_target` (extracted from #84 — used by ⌘⇧O + the palette). The settings load is fully tolerant:
an absent/malformed `remote.hosts` → the empty default, never a boot crash.

## M3.A — The Remote Seam: COMPLETE
#83 parse/command · #84 open pane · #85 ⇄ badge · #86 connect/disconnect status · #87 saved hosts +
palette. Marley opens remote panes by spawning the user's ssh (argv-safe, no secrets); `ssh` owns all
security. A cloud/relay remote-dev transport (M5) is a *separate* future clean REIMPLEMENT, not this seam.
