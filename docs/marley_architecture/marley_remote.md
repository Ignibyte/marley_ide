# `marley_remote`

> Per-crate architecture note — **refreshed 2026-09-29 · current to #543.**
> Provenance: **`[Marley-original]`** (INVENT, `serde` for the saved-host setting; otherwise std-only).
> **The ssh-argv remote seam — no cloud, no secrets, no relay.** Marley's "remote" is deliberately thin:
> spawn the user's OWN `ssh` client in a terminal and let `ssh` own all security. This is NOT Warp's
> `remote_server` / isolation-platform lineage (those were SKIP'd), nor Zed's remote development, whose
> terminals end with their link.

A pure domain crate: it parses a target and produces an argv, and handles NO secrets (`ssh` owns keys,
`known_hosts`, auth and passwords). Its caller in the fork is `marley_workbench::remote` (#543); before
#543 nothing depended on it since the fork (the 2026-09-18 port entry: "Not yet wired into the app").

## Surface
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
- `saved_target(host, user, port) -> Option<SshTarget>` (#543) — one of Zed's saved SSH hosts
  (`ssh_connections`) through `parse_ssh_target`'s checks; a host with a `:` is taken as IPv6, and a
  port inside the host string is refused, since the entry has its own.
- `SessionName` (#543) — `marley-` and eight hex digits from `from_bits(u32)`, or `parse(&str)` for a
  name of `[a-z0-9-]` only (not leading `-`), so the name is plain text to a remote shell.
- `remote_terminal_command(&SshTarget, &SessionName) -> Vec<String>` (#543) — `ssh_command` with
  `-t`, then `tmux -L marley -f /dev/null new-session -A -s <name> -e MARLEY_REMOTE=1` and, each
  after a `\;` word, the server's options: `status off`, `prefix None`, `mouse on`,
  `allow-passthrough on`, and `terminal-features ,xterm-256color:RGB`. `-L marley` keeps the
  sessions on a tmux server of Marley's own, apart from the user's, and `-f /dev/null` keeps the
  user's `~/.tmux.conf` out; `-A` attaches the session when it is there, which is the reconnect.
  Every word after the destination is a fixed word, the session name or `\;`, which the remote
  shell turns into tmux's `;`.

## Security — two-part injection guard
An argv (`Vec<String>`, never a shell string) defeats SHELL injection. But that alone does NOT stop
OPTION-SMUGGLING: `ssh` re-parses any argument starting with `-` as a flag, so a destination like
`-oProxyCommand=evil` would become an option, not a host (the CVE-2017-1000117 class — a latent RCE). Two
guards, both applied: (1) `parse_ssh_target` REJECTS a leading-`-` host/user; (2) `ssh_command` inserts
`--` before the destination (defends even a hand-built `SshTarget` that skipped the parser). See
`AD`/`PR-claude-subprocess-positional-arg-guard-leading-dash-and-double-dash`.

## In the fork (#543)
- `marley: open remote terminal` (`marley_workbench::remote`) lists the saved hosts that pass
  `saved_target` and opens a Zed task terminal on `remote_terminal_command`. Zed's task tab shows
  running and ended; its Rerun runs the same argv in the same terminal, which attaches the same
  session, so a dropped link loses nothing on the host. A Zed task hands its arguments to the shell
  as shell text (`ShellBuilder::build_no_quote`), so the workbench quotes each word for the shell;
  this crate's argv stays the one a direct exec takes.
- Claude Code's hook frames from inside the session reach Marley through tmux's passthrough: Claude
  Code wraps a hook's `terminalSequence` for tmux itself when `TMUX` is set, the plugin's gate passes
  on `MARLEY_REMOTE=1`, and the workbench lets a remote terminal's frames in although its
  foreground program is ssh.
- Not wired: `RemoteStatus`, `remote_badge`, `RemoteHost` and `remote_palette_actions` have no caller
  in the fork. The task tab is the status, and the hosts are Zed's `ssh_connections`, not a
  `[[remote.hosts]]` TOML setting.
- Not yet: reattaching after a Marley restart (a task terminal is not restored) and ending a host's
  sessions from Marley; the embedded harness's remote entry replaces the tmux wrapper later.

## The gpui-era app (history)
#84 to #87 opened a remote pane from a typed target (⌘⇧O), badged it `⇄ host` or `✗ host`, kept it open
when its ssh exited, and listed `[[remote.hosts]]` as `connect:` palette entries, all in the gpui-era
`RootView`, which the fork replaced. The pure halves above are what carried over.
