---
pipeline_id: 8bd21b15-5caa-42ee-9dc8-d07069ab8d64
ticket: forge#87 (4bdd76e9-296a-42af-a34f-7a9b125fa516) · local docs/planning/tickets/open/TICKET-087-remote-hosts.md
aar_id: 08963f50-f4b5-4d26-be9a-d6fd53b43b20
status: Phase 5 — Complete PASS
title: known-hosts config + a 'connect to…' palette action
type: feature
milestone: M3.A
references:
  - crates/marley_remote/src/lib.rs (PURE: RemoteHost, RemoteAction, remote_palette_actions)
  - crates/marley_app/src/settings.rs (define_setting! RemoteHosts)
  - crates/marley_app/src/app.rs (SHIM: load hosts, open_remote_target, connect commands + palette dispatch)
---

## Title
Save named ssh hosts in the TOML settings; the command palette (⌘⇧P) lists a "connect: {name}" entry per
host that opens a remote pane (#84). Closes M3.A.

## Scope
### In
- PURE (`marley_remote`, cov/MSI 100): `RemoteHost { name, target }` (serde); `RemoteAction { label,
  target: SshTarget }`; `remote_palette_actions(&[RemoteHost]) -> Vec<RemoteAction>` (validate each target
  via `parse_ssh_target`; DROP the invalid; label "connect: {name} → {target}").
- SETTINGS (`marley_app/settings.rs`): `define_setting!(pub RemoteHosts: Vec<RemoteHost> = Vec::new(),
  "remote.hosts")`; boot-load into `RootView.remote_actions` (tolerant — a load error → empty).
- SHIM (app.rs, masked): extract `open_remote_target(&mut self, SshTarget) -> bool` (the #84 spawn/split/
  tag/flash) reused by cmd-shift-o AND the palette; connect Commands `CommandId(1000 + i)`; the palette
  Enter opens the selected host's pane.

### Out
- A settings EDITOR UI (config is TOML-file only, per M1.B). Editing/adding hosts in-app. Grouping/folders.
  Reconnect. Host key management (ssh owns it, #83/#84).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — connect commands live in a high CommandId range (`CONNECT_BASE = 1000`, far above the static ids
  0/1/2/4/5); the palette Enter tries `action_for_command` first, else maps `id - CONNECT_BASE` → the
  loaded `remote_actions` (a defensive `.get`, never `[]`).
- D2 — `open_remote_target` is extracted from the #84 arm so cmd-shift-o + the palette share ONE open path
  (preserves #84 spawn / #85 tag / #86 Connected / #77 flash). cmd-shift-o still clears the compose prompt
  on success; the palette has no prompt to clear.
- D3 — an invalid host `target` is DROPPED from the actions (not a panic), mirroring the settings tolerant
  load; the #83 guards (leading-dash etc.) apply.
- D4 — `RemoteHost` gains serde (the workspace-standard `serde` derive dep) so it's a `SettingsValue`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `remote_palette_actions(hosts)` runs, each host with a valid target shall yield a `RemoteAction { "connect: {name} → {target}", parsed }`. | unit |
| REQ-002 | WHEN a host's target is invalid (empty / leading-dash / bare IPv6), that host shall be DROPPED (no panic). | unit |
| REQ-003 | WHEN the `remote.hosts` setting is absent/malformed, the load shall yield an empty host list (no clobber). | unit |
| REQ-004 (visual) | WHEN a host is configured, ⌘⇧P shall list a "connect: …" entry that opens that host's remote pane. | self-test (may be env-blocked — see #86) |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure surface; the shim masked. | gate |

## Phase Plan
- **P2** — RemoteHost/RemoteAction/remote_palette_actions; the RemoteHosts setting + boot-load; the
  open_remote_target extraction + connect-command build + palette dispatch; mutation targets; test plan.
- **P3** — implement (marley_remote + serde dep + settings.rs + app.rs).
- **P3.5** — 1 critic: remote_palette_actions MSI; the settings round-trip; the connect-id range (no
  collision, defensive get); open_remote_target preserves #84/#85/#86.
- **P4** — the pure + settings tests (cov/MSI 100) + the SELF-TEST (a `localhost` host → ⌘⇧P → connect →
  a pane; may be env-blocked per #86 — then unit-verify + state it) + gate GREEN.
- **P5** — docs, AAR, archive, close #87; **M3.A COMPLETE**.
