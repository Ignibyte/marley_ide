# Local URLs in a terminal open in a Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-503-terminal-urls-open-in-the-browser.md
- **Pipeline spec:** 503-terminal-urls-open-in-the-browser.spec.md

## Phase 1 — Plan (drafted overnight, 2026-09-25)
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 1 of the list after the browser
  waves: a local URL clicked in a terminal opens a Browser tab, and the terminal offers the URL a
  dev server printed. The Orca survey of the same day adds the details this spec folds in
  (`docs/orca_architecture/README.md`, the #503 row; report 03 §2.3 and item 5; report 05 §2.4
  and item 2): link routing with a default and an inverting modifier, a small popover (Browser
  tab, system browser, Copy), wrapped-URL stitching and OSC 8, a printed URL offered only while
  its port listens, `0.0.0.0` opened as loopback, SSH-terminal URLs left to the system browser.
- **Classification / tier:** feature, prong 3 with prong 1. Two small touches in Zed's terminal
  crates, one Zed settings field and its page section, the rest in Marley's crates. Too big with
  the grid popover and the stitching, so it is split (below); this spec is slice 1.
- **Recall (§18.3):**
  - AD-claude-477: everything under a terminal goes through `MarleyTerminalFooter`, and the bar
    shows only for a CLI agent. D8 widens that for a live offer and nothing else.
  - L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001: Zed's foreground
    process is refreshed on output, so the stand-in `ssh` and agent print before they wait.
  - L-claude-480-an-e2e-fake-acts-out-the-program-001: fakes act the program out (the dev server
    listens for real; the stand-in `ssh` prints and waits).
  - L-claude-489-each-wtype-resets-gpuis-compose-001: each `wtype` brings a keyboard of its own;
    `click_with` holds its modifiers in one `wtype` for the whole click.
  - L-claude-493-zed-gives-a-lost-focus-to-the-panes-front-item-001: a tab added with the focus
    takes it, which is what a clicked link wants; the reuse case activates the old tab instead.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001: the offer's menu starts on
    Open in Browser Tab; the scenario counts Downs from there.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: the agent bar gains a chip,
    so #500's scenario, which clicks the bar's plugin chip by coordinates, reruns at Test.
  - Brain: not consulted in this drafting run (the overnight brief is read-only); promotion runs
    `brain_ask` before locking D1 to D9.
- **Discovery:**
  - `crates/terminal_view/src/terminal_view.rs`: `MarleyFooterContext` (lines 129 to 136) and the
    global `MarleyTerminalFooter` (138 to 145); `subscribe_for_terminal_events` (1150), whose
    `Event::Open` arm (1260 to 1269) calls `cx.open_url(url)` for `MaybeNavigationTarget::Url`
    and `open_path_like_target` for paths, the latter already updating the workspace from this
    callback; the footer call in `render` (1395 to 1408); `deploy_context_menu` (549 to 621),
    which slice 2's grid menu would extend.
  - `crates/terminal/src/terminal.rs`: `Event` (728 to 742) and `MaybeNavigationTarget` (754 to
    762); `process_hyperlink` (2033), which emits `Event::Open` when `open`;
    `schedule_find_hyperlink` (2697), which looks for links only while Ctrl is held; `mouse_down`
    (2828), where a Ctrl+left press on a link keeps `mouse_down_hyperlink`, Shift+Ctrl included;
    `mouse_up` (2909), where the same link under the release queues `ProcessHyperlink(.., true)`,
    and, with no press on a link, a plain click on an OSC 8 cell calls `cx.open_url(link.uri())`
    (2968) while a Ctrl+click queues `FindHyperlink(position, true)` (2970); `mouse_up` takes
    `&Context`, so it can queue but not emit. `last_n_non_empty_lines` (2624, through
    `alacritty.rs`'s `last_non_empty_lines` at 1004: logical lines joined by `logical_line_for_row`,
    trailing spaces trimmed); `working_directory` (3059, `None` for a remote terminal);
    `foreground_process_command_name` (3071) and `foreground_process_command_from_argv` (3615,
    which reads the script name after `python3`, so a Python fake named `ssh` reads as `ssh`).
  - `crates/terminal/src/alacritty/hyperlinks.rs`: `URL_REGEX` (22); `find_from_grid_point` (91),
    the OSC 8 target first (98), then the regex between `line_search_left` and
    `line_search_right` (125), which follow soft wraps only.
  - `crates/gpui_linux/src/linux/platform.rs`: `open_url` (447) to `open_uri_internal` (817),
    which runs `open::commands` (`xdg-open` first) found on the PATH.
  - `crates/marley_workbench/src/agent_bar.rs`: `init` sets the footer (33); `contents` (94)
    returns `None` without an agent (`agent_in(terminal)?`), so `render` (153) draws nothing for
    a dev server's terminal today.
  - `crates/marley_workbench/src/browser.rs`: `new_tab` (5744) opens a blank page after the
    active item with the focus through `open_page_in` (2960) and `new_page` (2174), which waits
    for the hub to show its pages; `track_terminals` (5409) is the pattern for following every
    `TerminalView` (`observe_new`); `init` (5385).
  - `crates/marley_browser/src/address.rs`: `host_kind` (60) already knows loopback hosts for the
    address bar.
  - `crates/settings_content/src/marley.rs` (`MarleySettingsContent { layout }`, `MarleyLayout`
    with `strum::VariantArray` and `VariantNames` for #515's dropdown);
    `crates/settings_ui/src/marley_page.rs` (#515: sections as functions chained in
    `marley_page`); `crates/settings_ui/src/settings_ui.rs` line 559 (the dropdown renderer
    registered per enum); `crates/marley_workbench/src/marley_workbench.rs` (`MarleySettings`,
    165; `init`, 232).
  - `crates/project/src/project.rs` `is_local` (3069); gpui `Window::modifiers` (window.rs 3243).
  - `procfs-core` 0.18 in the cargo registry: `net.rs` `TcpNetEntries` (235) through
    `FromBufReadSI` with a `SystemInfo`, `ExplicitSystemInfo` in `lib.rs` (273); built today for
    `crates/crashes` (`minidumper` to `minidump-writer`, `default-features = false`).
  - `crates/ui/src/components/button/split_button.rs` (`SplitButton::new(left, right)`).
  - `script/e2e.sh`: `click` (446), `press` (338), `hold_keyboard` (203);
    `script/e2e/500-browser-from-the-rail.sh` (a stand-in agent by `exec -a claude`).
- **Decisions:** D1 to D9 in the spec.

### The split
- **Slice 1 (this spec):** routing with the setting and the inverting modifier, `0.0.0.0`, OSC 8
  plain clicks, SSH, the offer with its menu.
- **Slice 2 (a new ticket at Complete):** the popover on a link in the grid (plain click: Browser
  tab, system browser, Copy link, which for an OSC 8 link copies the hidden target), asking once
  on the first click which destination is the default, and stitching URLs a TUI wrapped at the
  right edge or drew in a box frame (`│ ┃ ║`). Its Zed surface: a public link-at-a-point on
  `Terminal` (today `find_hyperlink_at_point` is private), a plain-click path in `mouse_up` that
  opens the popover when the press and release land on one link with no drag, and the stitching
  in `alacritty/hyperlinks.rs` beside `find_from_grid_point`, with the hover range across rows.
  Orca's two stitching files are 331 lines, and its popover 197.

### Design
- **Approach.**
  1. *The hook (Zed).* In `terminal_view.rs`, beside `MarleyTerminalFooter`: `pub struct
     MarleyTerminalUrl(pub Arc<dyn Fn(&MarleyFooterContext, &str, &mut Window, &mut App) ->
     bool>)` and its `Global`. The `Event::Open` URL arm builds the context from the view's own
     fields and calls the hook; `false` or no hook runs `cx.open_url(url)`. The hook must not
     update the terminal view (it is being updated), so Marley only reads the terminal there and
     opens the tab through the workspace, as `open_path_like_target` already does.
  2. *The OSC 8 click (Zed).* In `Terminal::mouse_up`, the plain-click OSC 8 branch queues
     `InternalEvent::FindHyperlink(position, true)` when the link's URI starts `http://` or
     `https://`, and keeps `cx.open_url(link.uri())` otherwise. `find_from_grid_point` reads the
     OSC 8 target first, so the event carries the target.
  3. *The decision (Marley, `links.rs`).* `route(context, url, modifiers, settings) ->
     Destination`: SSH first (D4: `project.is_local()` false, or the foreground command among
     `ssh`, `mosh-client`, `autossh`, `et`); then `address::local_url(url)` (D2); then the setting
     and `modifiers.shift` (D3). A Browser-tab destination calls `browser::open_url_tab(workspace,
     url, window, cx)`, new beside `new_tab`: the workspace's `BrowserView` on the same URL is
     activated (D5), else a new view is added to the active pane with the focus and
     `open_page_in` loads the URL. The hook returns `true` for either destination it handled and
     opens the system browser itself with `cx.open_url` for the other, so Zed's fallback runs only
     when Marley has no opinion (a non-http URL).
  4. *Local URLs (Marley, `marley_browser::address`).* `local_url(text) -> Option<LocalUrl>`
     (the printed text, the URL to open with the unspecified host replaced, the `host:port` label,
     the port with the scheme's default), and `printed_local_urls(line) -> Vec<LocalUrl>`:
     candidates by ``https?://[^\s<>"'`]+``, trailing `.,;:!?` and unbalanced closing brackets or
     quotes dropped, parsed by `url::Url`, local ones kept.
  5. *Listeners (Marley, `marley_browser::ports`, new).* `listening_ports_in(dir) ->
     io::Result<BTreeSet<u16>>` reads `dir/tcp` and `dir/tcp6` through `procfs-core`, keeping
     `TcpState::Listen` sockets whose local address is loopback or unspecified (IPv4-mapped
     loopback included). Production passes `/proc/net`; the directory argument is §14's `*_in`
     rule. It runs on the background executor only.
  6. *The offer (Marley, `links.rs`).* A global `ServedUrls`: per terminal view (weak handle and
     entity id) its candidates by port, the newest first, and the listening set. `init` follows
     every `TerminalView` (`observe_new`, as `track_terminals` does) and its `terminal::Event::
     Wakeup`; a scan runs at most every 500 ms per terminal and once 500 ms after the last wakeup,
     skips SSH terminals, and reads `last_n_non_empty_lines(200)`. While any terminal holds a
     candidate, one background task reads the listeners every 2 seconds; when the live set of a
     terminal changes, Marley notifies that view so its footer redraws. `offer(context, cx) ->
     Option<LocalUrl>` gives the newest live one.
  7. *The footer (Marley, `agent_bar.rs`).* `render` asks `links::offer` and, with an agent,
     adds the offer's `SplitButton` to the bar's right side before the folder chip; without an
     agent but with an offer it draws a one-row strip (the bar's border and padding) holding the
     button; with neither it draws nothing, as today. The button's left part opens the URL through
     the same route as a Ctrl+click; its right part is a `PopoverMenu` with Open in Browser Tab,
     Open in System Browser and Copy URL (`cx.write_to_clipboard`).
  8. *The setting.* `MarleyTerminalLinks { LocalInBrowserTab (default), AllInBrowserTab,
     SystemBrowser }` in `settings_content/src/marley.rs` with the strum derives #515 gave
     `MarleyLayout`; `terminal_links` on `MarleySettingsContent` and on `MarleySettings`; a
     `terminal_section()` ("Terminal", "Terminal Links": "Where a URL Ctrl+clicked in a terminal
     opens; Shift+Ctrl+click opens it in the other place") on the Marley page; the dropdown
     renderer beside `MarleyLayout`'s.
  9. *The runner.* `click_with <mods> <x> <y>` in `script/e2e.sh`: under sway, one `wtype -M
     <mod>... -s 1500 -m <mod>...` in the background, a short settle, `click`, then `wait`.
- **File manifest.**
  - Zed crates: `crates/terminal_view/src/terminal_view.rs` (the global, the URL arm);
    `crates/terminal/src/terminal.rs` (the OSC 8 branch of `mouse_up`);
    `crates/settings_content/src/marley.rs` (the field and the enum);
    `crates/settings_ui/src/marley_page.rs` (the Terminal section);
    `crates/settings_ui/src/settings_ui.rs` (the dropdown renderer); root `Cargo.toml`
    (`procfs-core = { version = "0.18", default-features = false }` in `[workspace.dependencies]`).
  - Marley crates: `crates/marley_browser/src/address.rs`, `crates/marley_browser/src/ports.rs`
    (new), `crates/marley_browser/src/marley_browser.rs` (the module), `crates/marley_browser/
    Cargo.toml` (`procfs-core` under `[target.'cfg(target_os = "linux")'.dependencies]`);
    `crates/marley_workbench/src/links.rs` (new), `agent_bar.rs`, `browser.rs` (`open_url_tab`),
    `marley_workbench.rs` (the module, `links::init`, `MarleySettings.terminal_links`).
  - The e2e runner: `script/e2e.sh` (`click_with`), and the scenario
    `script/e2e/503-terminal-urls-open-in-the-browser.sh` (Test). Docs at Complete: `CHANGELOG.md`,
    `docs/marley/three-prong-plan.md`, `docs/marley_architecture/marley_browser.md` and
    `marley_workbench.md`.
- **Ledger rows (`docs/marley/zed-touchpoints.md`, written before the code, §14).**
  - `crates/terminal_view/src/terminal_view.rs`: the row gains `MarleyTerminalUrl` and the URL arm
    that asks it before `cx.open_url` (#503). Merge: keep the global and the arm's fallback; if
    upstream reshapes `Event::Open`, carry the question to wherever the URL is opened.
  - `crates/terminal/src/terminal.rs`: the row gains the plain-click OSC 8 branch of `mouse_up`
    queuing `FindHyperlink` for http and https targets (#503). Merge: keep the branch; if upstream
    routes OSC 8 clicks through `Event::Open` itself, drop it.
  - `crates/settings_content/src/marley.rs`, `crates/settings_ui/src/marley_page.rs`,
    `crates/settings_ui/src/settings_ui.rs`: their #515 rows gain `terminal_links`,
    `MarleyTerminalLinks`, the Terminal section and its dropdown renderer.
  - `Cargo.toml`: the row gains `procfs-core` in `[workspace.dependencies]`, kept at the version
    `minidump-writer` builds.
- **Knowledge at Complete (expected).** An AD for the route through `MarleyTerminalUrl` and the
  offer's listener rule; lessons from whatever the run teaches (the modifiers read at event time,
  `click_with` under sway).

### E2E plan
Setup: `offline_chromium`; a site served by `serve_site` (a dev page, `osc8.html`); a fake
`xdg-open` first on Marley's PATH (checked with `command -v` before the launch) that appends its
argument to `$E2E_WORK/xdg-open.log`; on the terminal's PATH a Python `devserver` that binds
`127.0.0.1:0`, serves a page titled "Dev page", prints Vite's lines (`➜  Local:
http://localhost:<port>/`) and serves until Ctrl+C; the scratch repository opened. Coordinates of
printed text are measured on the first run, as #496's are.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-009, REQ-010 | `echo http://localhost:9/nothing` (nothing listens on 9), then `devserver`; settle 4 | `503-01-offer`: the strip under the plain terminal offers `localhost:<port>` and not `localhost:9` |
| REQ-001 | `click_with CTRL` on the printed URL | `503-02-browser-tab`: a Browser tab of the scratch project on "Dev page", focused |
| REQ-002 | back to the terminal's tab; `click_with CTRL` on the same URL | `503-03-same-tab`: the same Browser tab in front, one Browser tab in the tab bar |
| REQ-005 | back to the terminal; `click_with "SHIFT CTRL"` on the URL | `503-04-inverse`: the terminal still in front, no second tab; the log's `xdg-open` line holds the URL |
| REQ-004 | `echo https://example.com/docs`; `click_with CTRL` on it | the log's `xdg-open` line; no new tab in `503-05` |
| REQ-009 | Ctrl+C; `python3 -m http.server 0 --bind 0.0.0.0` | `503-05-unspecified-offer`: the offer reads `127.0.0.1:<port>` |
| REQ-003 | `click_with CTRL` on the printed `http://0.0.0.0:<port>/` | `503-06-unspecified-tab`: the address bar reads `http://127.0.0.1:<port>/` |
| REQ-011 | close that tab; back to the terminal; click the offer's label | `503-07-offer-opens`: a Browser tab on `http://127.0.0.1:<port>/` |
| REQ-012 | back to the terminal; click the offer's arrow | `503-08-offer-menu`: Open in Browser Tab, Open in System Browser, Copy URL |
| REQ-013 | Down twice, Enter (Copy URL); `wl-paste` on the sway's display; the menu again, Down, Enter (Open in System Browser) | the log: the clipboard holds the URL; the `xdg-open` line |
| REQ-010 | Ctrl+C; settle 5 | `503-09-offer-gone`: no strip |
| REQ-007 | `printf` an OSC 8 link to `osc8.html` with the label "Open the OSC 8 page"; a plain click on the label | `503-10-osc8`: a Browser tab on `osc8.html` |
| REQ-009 | a new terminal: `(exec -a claude bash -c 'echo "Dev server: http://localhost:<site port>/"; sleep 600')` | `503-11-agent-bar`: the agent bar carries the offer |
| REQ-008 | a new terminal: a Python fake `ssh` that prints `http://localhost:<site port>/` and sleeps; `click_with CTRL` on it | `503-12-ssh`: no strip although the port listens here; the log's `xdg-open` line, no new tab |
| REQ-014 | quit; `"marley": {"terminal_links": "system_browser"}` written into the profile copy's settings; launch; `marley: open settings` | `503-13-settings`: the Terminal section's dropdown reads System Browser |
| REQ-006, REQ-005 | `devserver`; `click_with CTRL` on the URL, then `click_with "SHIFT CTRL"` | the log's `xdg-open` line for the first; `503-14-system-default`: a Browser tab for the second |

Not reachable by a scenario: a remote project's terminal (Zed's remote server over SSH), which the
review checks through `project.is_local()`; the real system browser, faked by `xdg-open`.

### Risks
- The modifiers are read when the event is handled, a frame after the release; a Shift let go
  inside that frame opens the default destination.
- The offer reads the last 200 logical lines at most twice a second, so a URL printed and pushed
  past 200 lines inside half a second is missed. A dev server prints its URL first and then
  waits, so this bites only very chatty starts.
- The strip under a plain terminal takes a row from the grid while an offer is live and resizes
  the PTY, as the agent bar does (AD-claude-477); a full-screen program redraws once.
- `procfs-core` as a direct dependency must stay at the version `minidump-writer` builds, or the
  tree builds two copies; cargo-shear needs the new workspace entry used.
- The OSC 8 change sits in upstream's busiest terminal file, and `mouse_up` has two link paths;
  the review checks that Ctrl+click on an OSC 8 cell still goes through `mouse_down_hyperlink`
  and is not queued twice.
- Under sway, `click_with` depends on the seat carrying the `wtype` keyboard's modifiers into the
  pointer's click. If gpui sees no modifier, the helper moves the hold into one `wtype` that
  presses, waits for the click and releases, and the Test notes say which worked.
- #507 later gives each project its own Chromium; `open_url_tab` takes the workspace, so #507 can
  route it to that project's browser without a change here.

## Promotion (2026-09-26, at `b71069c8a8`)
- **Seams re-read**, with where the lines are now:
  - `terminal_view.rs`: `MarleyFooterContext` is at 130, `MarleyTerminalFooter` at 141, the
    `Event::Open` URL arm at 1284 and the footer call at 1419-1430.
  - `terminal.rs`: `process_hyperlink` is at 2086, `last_n_non_empty_lines` at 2677 (over
    `alacritty.rs`'s `last_non_empty_lines` at 1024, the grid that shows) and `mouse_up` at 2962,
    with the OSC 8 `cx.open_url(link.uri())` at 3021 and the Ctrl `FindHyperlink` at 3024.
    `working_directory` is at 3112, `foreground_process_command_name` at 3124 and
    `foreground_process_command_from_argv` at 3668, which still reads the script after `python3`.
  - `browser.rs`: `new_page` is at 2224, `live_views` at 2744, `new_view` at 2817, `open_page_in`
    at 3063, `init` at 5513, `track_terminals` at 5537 and `new_tab` at 5872.
  - `agent_bar.rs`: `init` is at 33, `contents` at 93 and `render` at 149.
  - `address.rs`: `host_kind` is at 65, still private.
  - Settings: `settings_ui.rs` registers the `MarleyLayout` dropdown at 559; `marley_page.rs` has
    `layout_section`, `agents_section`, `push_section` and `privacy_section`;
    `settings_content/src/marley.rs` is as cited.
  - `project.rs`: `is_local` is at 3069.
  - `gpui_linux`: `open_uri_internal` (817) tries `open::commands` in order and stops at the
    first that exits 0.
  - `Cargo.lock` has `procfs-core` 0.18.0.
- **What changed since the draft:**
  - #574 gave `new_page` a `place_in: Option<WeakEntity<Workspace>>` and added
    `tab_workspaces`. `open_url_tab` passes `None` as `open_page_in` does: the view it adds
    claims the page, as `new_tab`'s does.
  - #549's `L-claude-549-wtype-sends-a-shifted-binding-by-its-symbol-001`: `click_with` sends
    modifier names (`ctrl`, `shift`), which wtype's `-M` takes.
- **A correction to the design, from the re-read:** the hook runs inside the terminal view's
  subscription, so the view is being updated. Adding a tab to its pane calls `deactivated` on
  the pane's active item, which is this terminal view, and that would update it again and panic.
  Opening a tab or activating one must not happen in the hook. The hook decides the destination
  there, returns `true`, and defers the tab (`window.defer`) until the view's update has ended.
  The system browser (`cx.open_url`) needs no deferral.
- **The fake `xdg-open` is a guard, not only a fixture:** without it, a scenario's Ctrl+click on
  a non-local URL would run the real `xdg-open` and open the user's own browser. The setup puts
  it first on the PATH Marley inherits and checks `command -v xdg-open` before the launch; the
  scenario stops if that is not the fake.
- **Recall added:** the brain decision `decisions/marleys-browser-tools-act-in-the-callers-project`
  (#574): a tab opened for a terminal's click opens in that terminal's workspace, as D5 has it.
- **Brain consultation d7131167e33d4f27934917a47d20fa57:** nothing on link routing.

## Phase 2 — Code
- **Built:**
  - *Zed crates.* `terminal_view.rs`: `MarleyTerminalUrl`, and the `Event::Open` URL arm asks it
    with the view's `MarleyFooterContext` before `cx.open_url`. `terminal.rs`: `mouse_up`'s plain
    OSC 8 click queues `FindHyperlink(position, true)` for an http or https target (by the
    scheme, case-insensitive). `settings_content/src/marley.rs`: `terminal_links` and
    `MarleyTerminalLinks`, the default first so the dropdown's fallback matches it.
    `settings_ui`: the Terminal section after Agents and the dropdown renderer.
    `assets/settings/default.json`: `terminal_links` with a comment. Root `Cargo.toml`:
    `procfs-core`.
  - *Marley crates.*
    - `marley_browser::address`: `LocalUrl`, `local_url` and `printed_local_urls`, with the
      trailing punctuation and unopened brackets trimmed.
    - `marley_browser::ports::listening_ports_in(dir)`: both TCP tables through `procfs-core`, an
      error only when neither reads.
    - `marley_workbench::links`: the hook `open_clicked`, `destination`, `open` (the Browser tab
      through `window.defer`), `over_ssh`, `ServedUrls` and `Printed` (each port's newest print,
      16 kept), the scan on `Wakeup` (one per 500 ms, the task kept per view and dropped with it),
      `watch_ports` and `take_ports` (every 2 s while any terminal holds a URL), `offer` and
      `offer_button` (a `SplitButton` whose `PopoverMenu` opens upward).
    - `agent_bar::render`: the offer in the bar before the folder chip, or alone in a one-row
      strip.
    - `browser::open_url_tab`: an existing tab of the workspace whose address is the same URL is
      activated, or a new view goes in the active pane with the focus and `open_page_in` loads
      the URL.
    - `MarleySettings.terminal_links`.
  - *The runner.* `script/e2e.sh`'s `click_with <mods> <x> <y>`: one `wtype` presses the
    modifiers, holds them for 1.5 s while `click` runs, and releases them.
- **Deviations from the plan, and why:**
  - The Browser tab opens in `window.defer` (the promotion's correction): a tab added inside the
    hook would deactivate the terminal view while it is being updated.
  - `procfs-core` is an ordinary dependency of `marley_browser`, not a Linux-only one. The crate
    only parses text, so it builds anywhere, and elsewhere the missing `/proc/net` files read as
    no listeners.
  - `url` is a new dependency of `marley_workbench` (the scheme check and the URL comparison in
    `open_url_tab`).
  - `default.json` gains `terminal_links`, as each Marley setting since #516 has; the ledger row
    for that file says so.
  - Printed URLs accumulate across scans (16 kept per terminal) instead of being re-read from the
    last 200 lines alone. A dev server that logs requests pushes its URL past 200 lines, and
    its offer would go while it still listens. A kept URL whose port closes stops being offered
    anyway.
  - Over SSH the URL opens as printed. Loopback is not rewritten there, since that host is
    another machine.
- **Review of the diff:**
  - Re-entrancy: the hook and the menu read the terminal and the project and defer the tab. The
    scan runs in its own task's update of the view and reads the workspace. `take_ports` updates
    each view from the watch task. Nothing updates an entity that is already being updated.
  - Every place the terminal opens a URL was checked: only the hook's fallback and the OSC 8
    branch for other schemes call `cx.open_url`.
  - `schedule_scan` reads the global with `try_global`, since `default_global` would notify its
    observers on every wakeup.
  - The Terminal section sits after Agents, so the Layout dropdown (#515) and the redaction
    toggle (#516), which those scenarios click by coordinates, do not move.
  - Provenance: Orca's behaviours (MIT) are reimplemented, and `open_url_tab` follows Marley's own
    `new_tab`.
  - fmt clean. `cargo clippy -p marley_browser -p marley_workbench -p terminal -p terminal_view
    -p settings_content -p settings_ui --all-targets -- -D warnings` is clean after fixing its
    first findings: a long first doc paragraph, an inverted `!=`, two `&mut` parameters that only
    needed `&`, and a missing `#[must_use]`.

## Phase 3 — Test
- **The scenario:** `script/e2e/503-terminal-urls-open-in-the-browser.sh` (`compositor sway`,
  offline Chromium).
  - The fake `xdg-open` is first on the PATH Marley starts with and is checked with `command -v`
    before the launch. It logs its argument and always exits 0: gpui tries the desktop portal
    only after every `open` command fails, and the portal would reach the user's own browser.
  - Fakes on the terminal's PATH, each waiting after it prints: `links` (a docs URL and
    `localhost:9`), `devserver` (127.0.0.1 at a free port, Vite's lines), `osc8` (an OSC 8 link
    to the site's page) and `ssh` (Python, so its foreground reads as `ssh`). Beside them,
    `python3 -m http.server 0 --bind 0.0.0.0`, and a stand-in Claude Code made with `exec -a
    claude`.
  - 15 `expect` checks, on the `xdg-open` log, `mcp_agent tabs` and `wl-paste` on the sway's
    display.
- **Measuring:** the first two runs used a copy in the scratchpad whose `expect` reported and
  carried on.
  - The first run placed the offer's arrow on its label.
  - The second clicked the OSC 8 label one row off: nothing was offered then, so there was no
    strip.
  - The coordinates are now the defaults at the file's top.
  - The `Public` icon drew a broadcast glyph and became `ToolWeb`, the Browser tab's globe.
- **Green, the debug build:** all 15 checks pass.
- **Red, the installed build (`93b87ee807`, before the change):** `check a Browser tab of the
  project on the dev server, focused: FAIL`. The local URL went to the system browser, and the
  old `503-01` shows no strip under the dev server.
- **Shots, read** (all Marley's headless sway window, kept in the scratchpad):
  - `503-00-links`: the docs and `localhost:9` URLs printed, no strip. The Ctrl+click on the docs
    URL reached the fake `xdg-open` (REQ-004).
  - `503-01-offer`: the strip under the plain terminal offers `localhost:<port>` and not
    `localhost:9` (REQ-009, REQ-010).
  - `503-02-browser-tab`: a "Dev page" Browser tab in the terminal's pane at
    `http://localhost:<port>/`, in front; `tabs` says `project repo, focused` (REQ-001).
  - `503-03-same-tab`: after a second Ctrl+click, the one Dev page tab in front and no other
    (REQ-002).
  - `503-04-inverse`: after Shift+Ctrl+click, the terminal still in front and no tab; the log
    holds the URL (REQ-005).
  - `503-07-offer-opens`: the offer's label opened a Dev page tab (REQ-011).
  - `503-08-offer-menu`: the menu opens upward from the chevron with Open in Browser Tab
    (selected), Open in System Browser and Copy URL (REQ-012). Copy URL put the URL on the
    clipboard, and Open in System Browser reached the log (REQ-013).
  - `503-09-offer-gone`: after Ctrl+C, the prompt with no strip, within the 5 s settle (REQ-010).
  - `503-05-unspecified-offer`: `Serving HTTP on 0.0.0.0 port N (http://0.0.0.0:N/)` offered as
    `127.0.0.1:N`, the parenthesis trimmed. The dev server's closed URL above it is not offered.
  - `503-06-unspecified-tab`: the address bar reads `http://127.0.0.1:N/` (REQ-003).
  - `503-10-osc8`: an OSC 8 page tab at `…/osc8.html` reading "Reached by an OSC 8 link.",
    which is the target and not the label (REQ-007).
  - `503-11-agent-bar`: the stand-in Claude Code's bar carries the offer before the folder chip
    (REQ-009). The install chip shows and was not clicked.
  - `503-12-ssh`: the stand-in `ssh` printed a URL whose port listens here, and there is no
    strip. The Ctrl+click on it reached the log with no tab (REQ-008).
  - `503-14-system-default`: with `system_browser`, Ctrl+click reached the log, and
    Shift+Ctrl+click opened a Dev page tab (REQ-006, REQ-005).
  - `503-13-settings`: the Marley page's Terminal section after Agents, Terminal Links with the
    reset arrow, and the dropdown reading System Browser (REQ-014).
- **Not reached by a scenario:** a remote project's terminal (Zed's remote server over SSH). The
  review checked that path through `Project::is_local`. The stand-in `ssh` covers the other half
  of D4.
- **The golden set, with 503 added (`just regress`, the debug build):** all 22 pass.
  - #500 (the agent bar it shoots gains no chip without a live offer) passes.
  - #515 and #516 click the Layout dropdown and the redaction toggle by coordinates; the Terminal
    section sits below both, and both pass.
  - 503 itself took 145 s.
- **The focus report:** each sway run stopped with its Marley, pointer and keyboard. The
  Hyprland check found no Marley window before or after the run, and nothing was added or
  reloaded.
- **The gate:** `script/gates.sh --diff` gave `GATE GREEN [diff]`, 16 passed and 0 failed,
  cargo-deny and cargo-shear included for `procfs-core` and `url`. The receipt matches the tree.
- **Verdict:** PASS. REQ-001 to REQ-014 are each shown by a shot and, where a machine check
  reaches it, by an `expect`.

## Phase 4 — Complete
- **Documented:**
  - `CHANGELOG.md`, under Added: "Local URLs in a terminal open in a Browser tab".
  - `docs/marley/three-prong-plan.md`: prong 3's slice table gains B6a (#503, shipped).
  - `docs/marley_architecture/marley_browser.md`: `local_url` and `printed_local_urls` under
    Navigation, and a Listening ports section.
  - `docs/marley_architecture/marley_workbench.md`: a Terminal links and the offered URL
    section, and the agent bar's strip.
  - The touchpoint rows were checked against what shipped: `terminal_view.rs`, `terminal.rs`,
    `settings_content/src/marley.rs`, `settings_ui/src/marley_page.rs`,
    `settings_ui/src/settings_ui.rs`, `assets/settings/default.json` and `Cargo.toml`.
- **Knowledge appended:**
  - `F-claude-503-a-tab-opened-inside-a-terminal-views-event-would-update-the-view-again-001`
  - `PR-claude-defer-a-pane-change-out-of-an-items-own-event-001`
  - `L-claude-503-gpui-falls-back-to-the-desktop-portal-when-every-open-command-fails-001`
  - `L-claude-503-a-scenario-cannot-redefine-the-runners-helpers-001`
  - `AD-claude-503-marley-routes-a-terminals-urls-and-offers-a-listening-dev-server-001`
- **Brain:** consultation `d7131167e33d4f27934917a47d20fa57` was closed by `brain decide`
  (`decisions/a-local-url-clicked-in-a-marley-terminal-opens-in-a-browser-tab-of-its-project-and-a-listening-dev-servers-url-is-offered`,
  follow-up 2026-10-26).
- **Slice 2 minted:** TICKET-579 (the popover on a plain-clicked link, the default asked once,
  wrapped URLs joined), queued after #561.
- **Closed:** TICKET-503 moved to `tickets/closed/`, and its BACKLOG row went at promotion.
