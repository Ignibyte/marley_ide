# The marley:// scheme - Marley's own links, and zed:// links kept in Marley — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-762-the-marley-scheme.md
- **Pipeline spec:** 762-the-marley-scheme.spec.md

## Phase 1 — Plan (2026-10-11)
- **Request:** #445's split (Chad, 2026-10-11, "7.) yes"); its spec's Out item 3 drafted this
  slice. Taken from the Queue under the goal "continue until all tickets are finished".
- **Classification:** chore, packaging; Zed hunks in `zed` (two files), `editor`, `workspace`,
  `zed_actions`; Marley's `single_instance`, `links`; the install script. Size S to M.
- **Recall (§18.3):**
  - #749: verbs are `zed://marley/…` URLs on the single-instance socket; `URL_PREFIX` notes that
    #445 could move them to `marley://` there alone.
  - #513, #545, #760: the hand-off, its activation token (`zed://marley-activation-token/`, read
    before the parser), the socket's name.
  - #503 and #579: Marley's terminal URL hook; a fake `xdg-open` that logs, first on the PATH.
  - Brain: consultation 8e91e471a39a4489a4a5eed56620c3d7, nothing on this seam.
- **Discovery** (an Explore pass at `515bf548d6`):
  - `OpenRequest::parse` (`open_listener.rs:148-233`) matches `zed://` prefixes in its loop at
    170; `marley://` falls through to "unhandled url". Its sources: the socket listener, the CLI,
    `on_open_urls`, the first launch's arguments (`main.rs:921-952` through `parse_url_arg`
    1858-1873), `OpenZedUrl` (`zed.rs:1052-1057`) and the URL modal (`open_url_modal.rs:56-61`,
    which already keeps every `zed://` URL in the app).
  - `single_instance::url_for` (211-219) turns `marley://open` into `file://<cwd>/marley://open`;
    `parse_url_arg` into `file://marley://open`.
  - Clicked links all end in `cx.open_url`, and gpui has no hook for it: the editor's Open URL and
    Ctrl-click (`navigation.rs:1158-1165`, `1853-1862`, through `parse_zed_link`, channels only);
    `Workspace::open_url_or_file` (`workspace.rs:5302-5307`, its `_` arm), used by agent messages
    (`thread_view::open_link`), the markdown preview, hovers, diagnostics and notifications;
    Marley's terminal hook (`links.rs:105-140`, http and https only); the markdown default
    (`markdown.rs:2510-2514`) and OSC 8 (`terminal.rs:3489-3498`).
  - `script/install-marley`'s entry has no `MimeType` (its comment says why); #502's scenario
    runs `desktop-file-validate` on it. `xdg-open` is xdg-utils' (`/usr/bin/xdg-open`).

### Design
- **`crates/zed_actions/src/lib.rs` (Zed crate):** `pub fn opens_in_app(url: &str) -> bool`, a
  `zed://` or `marley://` URL.
- **`crates/editor/src/navigation.rs` (Zed crate):** both tests add `|| opens_in_app(&url)`.
- **`crates/workspace/src/workspace.rs` (Zed crate):** in `open_url_or_file`, `"zed" | "marley"`
  dispatch `OpenZedUrl`.
- **`crates/zed/src/zed/open_listener.rs` (Zed crate):** the loop's URL through
  `marley_workbench::single_instance::zed_url`.
- **`crates/zed/src/main.rs` (Zed crate):** `parse_url_arg` keeps `marley://`.
- **`crates/marley_workbench/src/single_instance.rs`:** `marley://` in `URL_SCHEMES`; `zed_url`.
- **`crates/marley_workbench/src/links.rs`:** the terminal hook takes an app link and dispatches
  `OpenZedUrl`.
- **`script/install-marley`:** `MimeType=x-scheme-handler/marley;` and its comment.
- **Ledger rows:** `zed_actions` and `editor/src/navigation.rs` new; `workspace.rs`,
  `open_listener.rs`, `main.rs` extended.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001 | `/usr/bin/xdg-open marley://settings` with a scratch entry | 762-01: the Settings window |
| 002 | Ctrl-click `zed://settings` in `links.md` | 762-02; the fake `xdg-open` log empty |
| 003 | The markdown preview of `links.md`, a click on its `marley://settings` link | 762-03; the log empty |
| 004 | — | The review; #502's `desktop-file-validate` at the next install run |

### Risks
- **A `zed://` link meant for stock Zed** now opens in Marley when clicked inside Marley. That is
  the point of the ticket; outside Marley the desktop still sends it to stock Zed.
- **`OpenZedUrl` is a Workspace action:** a click whose dispatch path does not reach a workspace
  does nothing; every site here is inside one.

## Phase 2 — Code (2026-10-11)
- **Built:** the ledger rows first (`zed_actions` and `editor/src/navigation.rs` new; `main.rs`,
  `workspace.rs` and `open_listener.rs` extended). `zed_actions::opens_in_app`; the editor's two
  link tests; `open_url_or_file`'s arm for `zed` and `marley` URLs; `parse` reading each URL through
  `single_instance::zed_url`; `parse_url_arg` keeping `marley://`; `marley://` in `URL_SCHEMES`;
  Marley's terminal hook taking an app link; the entry's `MimeType=x-scheme-handler/marley;`. The
  guide's "`marley://` links (#762)", CHANGELOG.
- **Deviations from the design:** the entry's `Exec` unquoted unless the path holds a space, after
  the first Test run (F-claude-762-…).
- **The review:** `#749`'s `zed://marley/` verbs and the activation token keep their `zed://` forms;
  `zed_url` touches only a `marley://` prefix; the workspace arm dispatches from inside a
  workspace, where `OpenZedUrl` is registered.
- **Clippy:** `option_if_let_else` on `zed_url` (an early return, since `map_or` would move the
  string it borrows). Clean under the gate's scope.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/762-the-marley-scheme.sh`, under `compositor sway`, with #579's
  fake openers first on Marley's PATH and a desktop of the run's own for the real `xdg-open`.
  Exit 0, eleven checks passing. Shots read (the last run):
  - `762-01-desktop`: `xdg-open marley://settings` opened Marley's Settings window beside the
    workspace; no browser asked (REQ-001).
  - `762-02a-open-url` and `762-02-editor`: `links.md` with `zed://settings` on line 1; Open URL,
    then a Ctrl-click (cursor at 1:7), each opened the Settings window after the last one closed;
    nothing reached the fakes (REQ-002).
  - `762-03a-preview` and `762-03-preview`: the preview's "Open the settings" link (its hover says
    `marley://settings`) opened the Settings window; nothing reached the fakes (REQ-003).
- **Fixes on the way:** the first run's `xdg-open` fell back to Chromium, which started on the
  user's display with the link (stopped at once by its pid; its profile in the run's folder): the
  quoted `Exec` (F-claude-762-…), and the scenario's fallback fenced with `BROWSER` and a timeout
  (L-claude-762-…). A check that each Settings window closed, so every open is a new one.
- **What no scenario shows:** an agent's message link and a terminal Ctrl-click, which take the
  same `open_url_or_file` arm and the same `opens_in_app` test, read in the diff; the installed
  entry (REQ-004), at the next install (#502's `desktop-file-validate`).
- **Brain:** consultation 8e91e471a39a4489a4a5eed56620c3d7.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #762); the guide's "`marley://` links (#762)"; the five
  ledger rows describe what shipped.
- **Knowledge appended:**
  AD-claude-762-marley-reads-marley-as-zed-and-keeps-app-links-clicked-inside-001;
  F-claude-762-a-quoted-exec-defeated-xdg-opens-generic-mode-001;
  L-claude-762-xdg-opens-fallback-starts-a-real-browser-001.
- **Brain:** the consultation closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed. The next install
  writes the entry with the scheme and an unquoted `Exec`.
