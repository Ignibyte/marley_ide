---
pipeline_id: 1f97d1fe-a090-44de-8671-96c166770c3c
ticket: docs/planning/tickets/open/TICKET-503-terminal-urls-open-in-the-browser.md
status: Phase 4 — Complete PASS
title: "Local URLs in a terminal open in a Browser tab, and the terminal offers the one a dev server printed"
type: feature
slice: prong 3 with prong 1 (after wave 2), item 1 of the list after the browser waves; slice 1 of 2
references: [docs/orca_architecture/README.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/orca_architecture/05-terminal-and-workspace.md, docs/planning/pipeline/completed/500-browser-from-the-rail.spec.md, docs/planning/pipeline/completed/515-marley-settings-page.spec.md]
---

## Title
Where a URL clicked in a terminal opens becomes Marley's decision: a local URL opens in a Browser
tab of the terminal's project, where agents see the page, and anything else goes to the system
browser as in Zed. The terminal also offers the local URL a dev server printed, in its footer,
for as long as something listens on that port.

## Scope
### In
- **Routing.** One hook in Zed's terminal view: the `Event::Open` URL arm asks the global
  `MarleyTerminalUrl` first and calls `cx.open_url` only when Marley declines. Marley's side
  (`crates/marley_workbench/src/links.rs`, new) decides:
  - a local http or https URL (host `localhost`, a name under `.localhost`, `127.0.0.0/8`, `::1`,
    `0.0.0.0` or `::`) opens in a Browser tab of the terminal's workspace, with the focus, or
    brings forward the tab of that workspace already on the same URL;
  - `0.0.0.0` and `[::]` open as `127.0.0.1` and `[::1]`, port and path kept;
  - any other URL goes to the system browser;
  - Shift+Ctrl+click sends the URL to the destination Ctrl+click would not have chosen.
- **The setting.** `marley.terminal_links`: `local_in_browser_tab` (the default, the rule above),
  `all_in_browser_tab` (every http and https URL in a Browser tab) or `system_browser` (every URL
  in the system browser, as Zed does), with a Terminal section and dropdown on Marley's settings
  page (#515). Shift+Ctrl+click inverts whichever the setting says.
- **OSC 8.** A plain click on an OSC 8 link whose target is http or https reaches the same route.
  Today `Terminal::mouse_up` opens it with `cx.open_url` itself; it becomes the `FindHyperlink`
  event Ctrl+click already uses. Other schemes keep Zed's path.
- **SSH.** In a remote project's terminal, and while a terminal's foreground program is `ssh`,
  `mosh-client`, `autossh` or `et`, every URL goes to the system browser whatever the setting
  or the modifier, and nothing is offered: that `localhost` is another machine.
- **The offer.** Marley reads each local terminal's last 200 logical lines for local URLs while it
  prints, at most twice a second, and keeps them per terminal. A URL is offered only while a TCP
  socket listens on its port on a loopback or unspecified address (`/proc/net/tcp` and `tcp6`,
  read every 2 seconds off the main thread while any terminal holds a candidate). The newest live
  URL shows in the terminal's footer as a split button: its label (`localhost:5173`) opens the URL
  as Ctrl+click on a local URL does; its arrow opens a menu with Open in Browser Tab, Open in
  System Browser and Copy URL. The agent bar carries the button; a terminal with no agent shows a
  one-row strip holding the button alone, only while it has an offer.
- **The e2e runner.** `script/e2e.sh` gains `click_with <mods> <x> <y>`, a click while `wtype` holds
  the modifiers, for Ctrl+click and Shift+Ctrl+click under sway.

### Out (explicitly deferred)
- **Slice 2, its own ticket, minted at this ticket's Complete:** the popover on a link in the grid
  (a plain click offering Browser tab, system browser and Copy, Orca's `LinkActionPopover`, which
  is also what Warp's plain-click tooltip and right-click menu do), asking once which destination
  is the default, and stitching URLs a TUI wrapped at the edge or drew inside a box (Orca's
  `edge-wrapped-terminal-http-links.ts` and `hard-wrapped-terminal-http-links.ts`). They need a
  link-at-a-point accessor and a click path in Zed's `terminal.rs` that does not start a
  selection, and about 300 lines of stitching in `alacritty/hyperlinks.rs`: together they would
  double this slice's changes to Zed's terminal. Local dev-server URLs are short and rarely wrap,
  so this slice serves its purpose without them.
- Links clicked in the editor, the Markdown preview and the Agent Panel's messages (Orca routes
  those too). Terminals embedded in the Agent Panel are terminal views and take this route.
- A port list per project, port rows in the rail, attributing a listener to a project by its
  process's working directory, and Stop for a port: the ports-per-project ticket (Orca survey
  item 6), which this slice's listener check starts.
- More than one offer per terminal: only the newest live URL is offered.
- Labelled hosts per worktree (Orca's `*.orca.localhost` proxy) and remote dev servers reached
  through the SSH host (report 03 item 15).

## Reference (§20)
Upstream Zed's terminal (`crates/terminal`, `crates/terminal_view`) finds URLs and OSC 8 links,
underlines them while Ctrl is held and opens them on Ctrl+click through
`Event::Open(MaybeNavigationTarget::Url)` and `cx.open_url`. Marley keeps the detection, the hover
and the gestures, and changes only where a URL opens. Warp's published docs
(docs.warp.dev/terminal/more-features/files-and-links, read 2026-09-25): on Linux Ctrl+click opens
a link, a plain click shows an "Open Link" tooltip, a right-click copies the URL, web links open in
the default browser, and OSC 8 links are followed. Marley matches Ctrl+click and OSC 8 now and
leaves the plain-click and copy gestures on grid links to slice 2. Orca (report 03 §2.3, report
05 §2.4): a link-routing setting with an inverting modifier, printed URLs offered only while their
port listens, `0.0.0.0` opened as loopback, SSH panes kept on the system browser; Marley takes
each of these.

### Prior art
- **Behavior maps and reports.** `docs/orca_architecture/README.md` (the #503 row of "What it
  changes in the queued sprint"), report 03 §2.3 and item 5, report 05 §2.4 and item 2, and the
  Orca files they cite, read at `1c2cf120e3`: `src/renderer/src/lib/http-link-routing.ts`
  (`resolveModifierRouting`: with the inverting option on, Shift flips whichever way the setting
  points); `src/main/ports/advertised-url-parsing.ts` (output scanned at line breaks, controls
  stripped, candidates matched by ``https?://[^\s<>"'`]+`` and checked with `new URL`);
  `src/main/ports/local-workspace-platform-port-scanner.ts` (`parseProcNetTcp`, state `0A`);
  `src/shared/localhost-worktree-labels.ts` (`connectableLoopbackHost`: `0.0.0.0` to
  `127.0.0.1`, `::` to `::1`); `src/renderer/src/components/link-actions/LinkActionPopover.tsx`
  and the two wrapped-link files (slice 2). `docs/zed_architecture/subsystems/
  08-terminal-tasks-fusion.md` §7 maps Zed's hyperlink detection (OSC 8, then the regex
  searches).
- **Published material.** Warp's files-and-links page (above); the OSC 8 hyperlink escape
  (`ESC ] 8 ; ; URI ST`); `proc(5)` on `/proc/net/tcp` (hex addresses, `st` 0A for LISTEN).
- **Code we already ship.** `crates/terminal/src/alacritty/hyperlinks.rs` `find_from_grid_point`
  reads an OSC 8 target before the text (line 98) and runs `URL_REGEX` between
  `line_search_left` and `line_search_right` (line 125), so a soft-wrapped URL already joins;
  `Terminal::process_hyperlink` turns a match into `Event::Open`; `Terminal::mouse_up` opens an
  OSC 8 cell on a plain click with `cx.open_url(link.uri())` (line 2968);
  `Terminal::last_n_non_empty_lines` (logical lines, wraps joined; line 2624);
  `Terminal::foreground_process_command_name` (from argv, so `python3 ./ssh` reads as `ssh`);
  `Project::is_local`; gpui's `Window::modifiers`; gpui_linux's `open_uri_internal`, which runs
  the `open` crate's `xdg-open`, so a scenario fakes the system browser on the PATH. The
  **`procfs-core` 0.18** crate, already built on Linux for `crates/crashes` through
  `minidump-writer`, parses `/proc/net/tcp` and `tcp6` (`net::TcpNetEntries`, `TcpState::Listen`,
  IPv6 word order handled); Marley adopts it for the listener check instead of parsing hex by
  hand. Marley's own: `browser::new_tab` and `open_page_in` (a tab on a URL), `address::host_kind`
  (the loopback rules), `agent_bar::render` behind `MarleyTerminalFooter` (AD-claude-477),
  `ui::SplitButton` and `ui::PopoverMenu`, the `url` and `regex` crates. Checked and not used:
  `sysinfo` (processes, not sockets); Zed's private `URL_REGEX` (the offer's scan needs only
  http and https).

## UI proof
UI-AFFECTING (a click in the terminal opens a Browser tab; the footer shows the offer; a setting).
`script/e2e/503-terminal-urls-open-in-the-browser.sh` (`compositor sway`, offline Chromium, a fake
`xdg-open` first on Marley's PATH that logs its argument instead of opening a browser). The
terminal runs a fake dev server that listens and prints its URL, a URL nothing listens on,
`python3 -m http.server 0 --bind 0.0.0.0`, an OSC 8 link, a stand-in agent that prints a URL, and
a stand-in `ssh`. Shots: `503-01-offer`, `503-02-browser-tab`, `503-03-same-tab`, `503-04-inverse`,
`503-05-unspecified-offer`, `503-06-unspecified-tab`, `503-07-offer-opens`, `503-08-offer-menu`,
`503-09-offer-gone`, `503-10-osc8`, `503-11-agent-bar`, `503-12-ssh`, `503-13-settings`,
`503-14-system-default`; the run log holds the fake `xdg-open` calls and the clipboard read after
Copy URL.

## Locked-In Decisions
- D1 — Marley decides the destination through one hook in Zed's terminal view, and Zed still
  detects the links: the `Event::Open` URL arm consults `MarleyTerminalUrl` (the view's context,
  the URL, the window) and falls back to `cx.open_url`. The hook reuses `MarleyFooterContext`
  (the view, terminal, project, workspace and focus handle), so the touch adds no context type.
- D2 — A local URL is one whose host is loopback or unspecified: `localhost`, a name under
  `.localhost`, `127.0.0.0/8`, `::1`, `0.0.0.0`, `::`. LAN addresses, `.test` names and the
  machine's own name are not local. The unspecified hosts open as the loopback of their family,
  as Orca's `connectableLoopbackHost` does: `0.0.0.0` says where a server listens, not an address
  to connect to.
- D3 — Ctrl+click follows `marley.terminal_links` (default `local_in_browser_tab`) and
  Shift+Ctrl+click takes the other destination for that click. The modifiers are read from the
  window when the event arrives: Zed's `Event::Open` carries none, and adding them would change an
  upstream event that every match reads.
- D4 — SSH wins over the setting and the modifier: a remote project's terminal, or one whose
  foreground program is `ssh`, `mosh-client`, `autossh` or `et`, sends every URL to the system
  browser and is never scanned for offers.
- D5 — A Browser tab of the terminal's workspace already on the clicked URL is brought forward
  instead of opening a second one; otherwise a new tab opens after the active one, with the focus,
  as `browser::new_tab` opens one.
- D6 — The offer is read from the grid, not the byte stream: the terminal's last 200 logical lines
  (Zed's `last_n_non_empty_lines`, where escapes are already gone and soft wraps joined), scanned
  on its output at most every 500 ms and once more after it goes quiet. A URL is kept per port,
  the newest printed wins, and it shows as printed except that `0.0.0.0` and `[::]` show and open
  as loopback.
- D7 — A URL is offered only while a socket listens on its port on a loopback or unspecified
  address, checked from `/proc/net/tcp` and `tcp6` through `procfs-core` every 2 seconds on the
  background executor while any terminal holds a candidate, and not at all when none does.
- D8 — The footer carries the offer: a chip in the agent bar, and under a terminal with no agent
  a one-row strip that shows only while there is an offer. This widens AD-claude-477 ("a bar
  while the foreground is a CLI agent") by that one case; a terminal with nothing to offer draws
  no footer, as before.
- D9 — OSC 8: a plain click on a link whose target is http or https queues the `FindHyperlink`
  event Ctrl+click already queues, instead of calling `cx.open_url` in the terminal model, so both
  gestures reach D1's hook. Other schemes keep Zed's direct open.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user Ctrl+clicks a local http or https URL in a local terminal, Marley shall open it in a Browser tab of the terminal's project, with the focus. | Shot `503-02-browser-tab` |
| REQ-002 | WHEN the terminal's project already has a Browser tab on the clicked URL, Marley shall bring that tab forward instead of opening another. | Shot `503-03-same-tab` |
| REQ-003 | WHEN the clicked URL's host is `0.0.0.0` or `[::]`, the Browser tab shall load it at `127.0.0.1` or `[::1]` with the same port and path. | Shot `503-06-unspecified-tab` |
| REQ-004 | WHEN the user Ctrl+clicks a URL that is not local, Marley shall open it in the system browser. | The run log's fake `xdg-open` line; no new tab in the next shot |
| REQ-005 | WHEN the user Shift+Ctrl+clicks a URL, Marley shall open it in the destination Ctrl+click would not have chosen. | Shot `503-04-inverse` and the run log; shot `503-14-system-default` |
| REQ-006 | WHERE `marley.terminal_links` is `system_browser`, Ctrl+click shall open every URL in the system browser. | The run log's `xdg-open` line; shot `503-14-system-default` |
| REQ-007 | WHEN the user clicks an OSC 8 link whose target is a local http URL, Marley shall open the target, not the label, in a Browser tab. | Shot `503-10-osc8` |
| REQ-008 | WHILE a terminal's foreground program is an SSH client, Marley shall open every URL clicked in it in the system browser and offer none. | Shot `503-12-ssh`; the run log |
| REQ-009 | WHEN a program in a local terminal prints a local URL and a socket listens on its port, the terminal's footer shall offer that URL, under a plain terminal and in an agent's bar. | Shots `503-01-offer`, `503-05-unspecified-offer`, `503-11-agent-bar` |
| REQ-010 | WHILE nothing listens on a printed URL's port, the footer shall not offer it, a listener that closes taking its offer with it within five seconds. | Shots `503-01-offer` (the dead URL not offered), `503-09-offer-gone` |
| REQ-011 | WHEN the user clicks the offer's label, Marley shall open its URL as a Ctrl+click on the URL would. | Shot `503-07-offer-opens` |
| REQ-012 | WHEN the user opens the offer's menu, it shall list Open in Browser Tab, Open in System Browser and Copy URL. | Shot `503-08-offer-menu` |
| REQ-013 | WHEN the user chooses Copy URL, the offered URL shall be on the clipboard. | The run log's clipboard read |
| REQ-014 | WHEN the user opens Marley's settings page, its Terminal section shall show Terminal Links with its three choices and the one in force. | Shot `503-13-settings` |

## Phase Plan
- **P1 Plan** — promote this pair, re-verify the seams against HEAD, confirm that #515's page
  takes a second dropdown the way it takes the layout's, and that `procfs-core`'s version matches
  the one `minidump-writer` builds.
- **P2 Code** — the terminal view hook and the OSC 8 click in Zed's crates, with their rows in
  `docs/marley/zed-touchpoints.md` written first; `links.rs`, the address and port helpers, the
  setting and its section, the footer's offer, `click_with`; fmt and clippy clean; a review of the
  diff (entity re-entrancy in the hook, the SSH rule, the upstream discipline).
- **P3 Test** — write and run the scenario and read every shot; rerun #500's scenario (the agent
  bar it shoots gains nothing unless an offer is live, and must look as before);
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the prong's slice status and the per-crate notes (§21), the
  touchpoint rows checked against what shipped, ledger capture (§19), mint slice 2's ticket, close,
  archive, commit.
