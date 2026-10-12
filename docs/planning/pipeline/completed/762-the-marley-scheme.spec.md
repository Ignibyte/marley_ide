---
pipeline_id: e2ae40b1-1c6e-4ebd-b738-9348bbf55af7
ticket: docs/planning/tickets/closed/TICKET-762-the-marley-scheme.md
status: Phase 4 — Complete PASS
title: The marley:// scheme - Marley's own links, and zed:// links kept in Marley
type: chore
slice: packaging, the release identity (TICKET-445), slice 4 of 5
references:
  - docs/planning/pipeline/completed/445-marley-release-identity.spec.md
  - docs/planning/pipeline/completed/760-marley-calls-itself-marley.spec.md
  - docs/planning/pipeline/completed/749-marley-commands.spec.md
---

## Title
Marley takes no `marley://` link: a launch with one opens it as a file, and the desktop knows no
handler for it. And a `zed://` link clicked inside Marley (in a buffer, a markdown preview, an
agent's message, a hover, a terminal) goes through `xdg-open` to whichever Zed the desktop names,
stock Zed if one is installed. This slice registers `marley://` with the desktop, reads it
wherever `zed://` is read, and keeps both kinds of link clicked inside Marley in Marley. The fourth
of #445's five slices (#445's spec, Out item 3).

## Scope
### In
- **The scheme read.** `marley://<rest>` means what `zed://<rest>` means: the open listener reads
  it as that (`open_listener.rs:170`), a second launch passes it on as a link
  (`single_instance::URL_SCHEMES`), and a first launch keeps it a link (`main.rs` `parse_url_arg`).
- **The desktop.** `marley.desktop` declares `MimeType=x-scheme-handler/marley;`, and only that,
  so `xdg-open marley://…` starts Marley, which hands the link to the running one.
- **Links clicked inside Marley.** A `zed://` or `marley://` link goes to Marley's own open
  listener (`zed_actions::OpenZedUrl`), not to `xdg-open`:
  - the editor, its Open URL action and a Ctrl-click (`editor/src/navigation.rs:1158`, `:1855`);
  - `Workspace::open_url_or_file` (`workspace.rs:5302-5307`), which agent messages, the markdown
    preview, hovers and diagnostics and notifications use;
  - a terminal's Ctrl-click, through Marley's terminal URL hook (`links.rs`).
- Docs: the guide (links and the scheme), CHANGELOG.

### Out (explicitly deferred)
- `#749`'s verbs keep `zed://marley/…` on the single-instance socket; they need no desktop
  scheme, and the socket is Marley's own since #760.
- A markdown element with no click handler of its own (`markdown.rs:2510-2514`) and an OSC 8
  terminal link (`terminal.rs:3489-3498`) still go to `xdg-open`; a `marley://` link there
  reaches Marley through the desktop.
- Registering Marley for `zed://` on the desktop: that would take stock Zed's links.
- The updater: #763.

## Reference (§20)
Upstream Zed, the `zed` and `editor` crates: `zed://` links are read by `OpenRequest::parse`, a
link the editor recognizes is dispatched as `OpenZedUrl` into the same listener
(`navigation.rs:1158-1165`), and `open_url_modal.rs:56-61` already keeps every `zed://` URL in the
app. Zed's own desktop entry declares `x-scheme-handler/zed` (`zed.desktop.in:16`). Marley keeps
the listener and the action, reads its own scheme as Zed's, and widens the editor's test to every
app link, as the URL modal does.

### Prior art
- **The code we ship:** `OpenRequest::parse` and its sources (`open_listener.rs:148-233`,
  `main.rs:489-497`, `:921-952`); `OpenZedUrl` and its handler (`zed_actions/src/lib.rs:23-29`,
  `zed.rs:1052-1057`); `open_url_modal.rs:56-61`; `client::parse_zed_link`, channels only
  (`client.rs:1949-1988`); `Workspace::open_url_or_file`; `single_instance::url_for`; Marley's
  terminal URL hook (`links.rs:105-140`); gpui's `App::open_url`, with no hook of its own.
- **Behavior maps:** none.
- **Published material:** the freedesktop desktop entry spec (`MimeType`,
  `x-scheme-handler/<scheme>`) and `xdg-mime`'s lookup.

## UI proof
`script/e2e/762-the-marley-scheme.sh`, under `compositor sway`. A fake `xdg-open` first on
Marley's PATH logs any call, so a link that leaves Marley is caught. The desktop half runs the real
`/usr/bin/xdg-open` with a scratch `XDG_DATA_HOME` and `XDG_CONFIG_HOME` holding a desktop entry
like the install's, whose `Exec` is the run's build on the run's profile.
- `762-01-desktop`: `xdg-open marley://settings` opens Marley's Settings window, through the
  desktop entry and the hand-off.
- `762-02-editor`: a Ctrl-click on `zed://settings` in a buffer opens the Settings window; the
  fake `xdg-open` logged nothing.
- `762-03-preview`: a click on a `marley://settings` link in the markdown preview opens it too;
  nothing logged.

## Locked-In Decisions
- **D1 — `marley://<rest>` reads as `zed://<rest>`.** One rewrite at the top of the parse loop
  gives every `zed://` form a `marley://` twin. Rejected: a second set of prefixes.
- **D2 — Only Marley's scheme on the desktop.** `x-scheme-handler/marley` and nothing else, as the
  entry has claimed nothing so far. Rejected: `x-scheme-handler/zed` (stock Zed's), an
  `xdg-mime default` call in the install (it writes the user's `mimeapps.list`, and the entry is
  the only one that names the scheme).
- **D3 — App links clicked inside stay inside.** `zed_actions::opens_in_app` names them; the three
  sites dispatch `OpenZedUrl`. Rejected: a hook in gpui's `open_url` (a library change for an app
  rule), registering Marley for `zed://`.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the desktop opens a `marley://` link, the system shall start Marley through its desktop entry, which hands the link to the running Marley, and act on it as on the `zed://` link of the same path. | Shot 762-01 |
| REQ-002 | WHEN a `zed://` link is Ctrl-clicked in a buffer, the system shall act on it in Marley and run no `xdg-open`. | Shot 762-02 and the log |
| REQ-003 | WHEN a `zed://` or `marley://` link is clicked in a markdown view (the preview, an agent's message), the system shall act on it in Marley and run no `xdg-open`. | Shot 762-03 and the log |
| REQ-004 | The installed desktop entry shall declare `MimeType=x-scheme-handler/marley;` and no other type. | The review; `desktop-file-validate` in #502's run |
| REQ-005 | The change shall pass `script/gates.sh --diff`. | The gate |
