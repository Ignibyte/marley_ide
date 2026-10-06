# T3 Code survey 05: the terminal, the browser, devices and capture

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley). Line counts are non-test TypeScript.

## 1. Summary

T3 Code's terminal is a plain PTY drawer attached to each thread: node-pty on the environment
server, a `libghostty-vt` renderer on web and Android, and a server-side history of 5,000 lines or
8 MiB that is replayed when a client attaches or the server restarts. It has no blocks, no shell
integration, no prompt editor and no agent recognition; Marley's block terminal is well ahead on
every one of those. The browser is the richer half. Agent tabs run in a pinned Chrome for Testing
headless shell driven by Playwright on the server and streamed to any client, with 19 MCP tools,
a human-or-agent control lock, file choosers, downloads, popups as tabs, viewport presets, a
colour-scheme switch and MP4 recording with a gliding agent cursor. The desktop app adds an
in-page annotator that selects several elements, draws regions and freehand strokes, and live-edits
their CSS, sending the edits as `property: before → after`. SnapShots capture the active window
with its accessibility tree through a small Rust Wayland helper for Hyprland, and the Devices panel
streams iOS simulators and Android emulators to people and agents. Worth taking, in order: (1)
scrollback that survives a restart, replayed with query traffic stripped; (2) the browser tool
gaps Orca's report 03 already listed, now with T3's shapes; (3) file choosers and downloads in
the Browser tab; (4) a take-control lock on a Browser tab; (5) live style edits in an annotation;
(6) reviewed, verified edits to other tools' config files; (7) a tab recording as a video file.

## 2. Features

### 2.1 Terminals owned by the server, with bounded history

**What the user sees.** A terminal drawer under each thread (`terminal.toggle`, `terminal.split`,
`terminal.splitVertical`, `terminal.new`, `terminal.close` in
`packages/contracts/src/keybindings.ts:63-67`). Terminals belong to the thread and run in its
worktree. Closing the app or losing the connection leaves them running when the server runs as a
service; reconnecting from any client shows the same session with its history
(`docs/internals/terminal-runtime.md`). After a server restart the processes are gone but up to
5,000 lines of output come back (`docs/user/terminal.md:3`).

**How it works.**

- `apps/server/src/terminal/Manager.ts` (3,167 lines) owns every PTY through
  `NodePtyAdapter.ts`. The service is open, attachStream, write, resize, clear, restart, close,
  `closeIdle` and two subscriptions (`Manager.ts:150-263`). Sessions key on thread id plus
  terminal id, and up to 128 inactive sessions are retained (`Manager.ts:103`).
- History is a chunked ring, `BoundedTerminalHistory` (`Manager.ts:846`), capped at 5,000 lines
  and 8 MiB (`Manager.ts:96-97`). Eviction never splits a code point, and live output is never
  cut. It is written to `<logsDir>/<thread>[_<terminal>].log` 40 ms after the last change
  (`Manager.ts:99`, `1584-1590`), and restored by reading only the file's tail, skipping a split
  UTF-8 prefix (`Manager.ts:1775-1798`).
- Retained history drops request and response traffic before it is stored: device status and
  cursor reports (CSI `n`, `R`, `c`), DECRQM and DECRPM (`$p`, `$y`), XTVERSION (`>q`), the kitty
  keyboard query (`?u`), DECRQSS and XTGETTCAP DCS strings, and OSC 10/11/12 colour queries
  (`Manager.ts:1024-1066`). The reason given in the comment at `Manager.ts:1039`: replaying a stored
  query makes the terminal answer again, and the shell echoes the answer as junk at the prompt.
  The web renderer also detaches its PTY writer while it replays
  (`apps/web/src/terminal/ghostty/core.ts:332-338`).
- Whether a terminal is busy comes from the process table: the shell's children, polled from
  1 s backing off to 60 s, ignoring the forked copy an async prompt theme leaves
  (`Manager.ts:690-742`). `closeIdle` uses it when a thread settles or a setup script ends: idle
  shells close, a dev server stays (`Manager.ts:204-213`).
- `PORT`, `ELECTRON_RENDERER_PORT` and `ELECTRON_RUN_AS_NODE` are removed from the shell's
  environment (`Manager.ts:106`), AppImage variables are scrubbed (`Manager.ts:1256-1300`), and
  `COLORTERM=truecolor` is set when nothing else set it (`Manager.ts:1332`).

**Good.** The query stripping is a small, exact list with a reason per entry, and the replay
protection is applied twice. History is bounded in both lines and bytes, so one long unterminated
line cannot defeat the cap.
**Bad.** Busy-ness from `ps` polling is a guess next to shell integration, and the terminal knows
nothing about commands, exit codes or agents.
**Size.** About 3,600 lines on the server.
**Marley today: has part.** Marley's terminals carry far more (The block terminal: blocks, exit
codes, the prompt editor, autosuggestions, blocks over ssh; Remote terminals (#543, #641) in a
tmux session that survives a dropped link). A terminal comes back after a restart in its old folder
(#577) and keeps its id (#575), and Claude Code resumes its session (#540), but the scrollback and
the blocks above the prompt are gone: Zed persists a terminal's folder, not its output. Processes
that outlive Marley are the harness's job (The harness's sessions, #632).

### 2.2 The `libghostty-vt` renderer

**What the user sees.** The same terminal emulation on web, desktop and the Android app, with a
bundled symbols-only Nerd Font so prompt glyphs draw without a local install.

**How it works.** `native/libghostty-vt/` pins Ghostty's VT library (MIT, revision in
`native/libghostty-vt/VERSION`). Web compiles it to WebAssembly, shared once per browser tab, with
a 112-byte trampoline for PTY replies; `core.ts` turns the C ABI into render snapshots,
`renderer.ts` paints Canvas 2D, `surface.ts` handles input, IME, selection and links
(`apps/web/src/terminal/ghostty/README.md`). About 4,000 lines.

**Marley today: has.** Marley draws with its own `alacritty_terminal` copy in GPUI (#461). Nothing
here is worth a switch.

### 2.3 Terminal excerpts as composer context

**What the user sees.** Select lines in a terminal and add them to the message being written; the
chip reads "Terminal 1 lines 3-4" and opens the captured text (`docs/user/composer.md`).

**How it works.** `apps/web/src/lib/terminalContext.ts` records terminal id, label, line range and
text, and formats a `t3-context://` reference in the prompt.

**Marley today: has most of it.** Send to Agent types a whole block at another agent's prompt,
as Markdown or as a `terminal_read` reference when long, with secrets hidden (#555), and Ctrl+>
sends an editor selection (#549). A selection of some lines of a block, rather than the whole
block, has no route of its own; copying and pasting it works.

### 2.4 The preview browser: two hosts, one broker

**What the user sees.** A Browser tab in the right panel of a thread, a floating mini preview over
the chat when an agent opens a page (`ThreadPreviewMiniPlayer.tsx`), an agent cursor that glides to
each target and pulses before a click, and on a phone a picture-in-picture window
(`docs/user/remote-access.md:175-201`). Agent tabs have storage of their own; you press **Take
control** before typing into one and **Release control** to give it back (`remote-access.md:188`).

**How it works.**

- Two hosts implement the same operations. The desktop app drives an Electron `<webview>` and
  exposes its debugger to the server as a one-page CDP endpoint, so the server's Playwright drives
  it like any page (`apps/desktop/src/preview/CdpRelay.ts:1-10`). The server runs its own
  headless browser for standalone and remote environments (`apps/server/src/preview/ServerBrowser.ts`,
  2,202 lines) and streams it with `Page.startScreencast` at 2x scale
  (`ServerBrowser.ts:1943-1989`).
- `apps/server/src/mcp/PreviewAutomationBroker.ts` (705 lines) routes each agent call to a host.
  The server's headless browser registers as *preferred*, so new agent work goes there before any
  desktop (`PreviewAutomationBroker.ts:56-63`), and a lease pins one provider session to one host
  for as long as that connection lives (`:96-106`).
- The browser is one pinned Chrome for Testing headless shell, version 154.0.8037.92, downloaded
  once (about 120 MB) and checked against a SHA-256 per platform
  (`apps/server/src/preview/PreviewBrowser.ts:24-40`). HTML render previews use the same binary
  (`docs/user/html-renders.md:9`). It never uses a browser the user installed.
- Contexts: "Persistent human profiles keep their storage; isolated agents share a browser, never a
  context" (`ServerBrowserContexts.ts:20`). An agent's server tab cannot be driven by another agent
  session (`preview/tools.ts:73-80`).
- Control is `SessionControl` (`apps/server/src/preview/SessionControl.ts`, 135 lines). An agent
  action is refused with "A human controls this tab" while a viewer holds control (`:76-86`).
  Taking or releasing control bumps an epoch, so an agent action queued before the change fails
  with "Browser control changed. Refresh the snapshot before trying again." (`:3-12`, `:59-74`).
  Viewport and appearance changes queue behind running actions without needing control
  (`:92-98`).
- Popups become tabs and keep `window.opener`, so sign-in popups can report back
  (`ServerBrowser.ts:823-824`).

**Good.** One Playwright engine for both hosts, and the epoch check is a simple, correct answer to
a human and an agent acting on one page at once.
**Bad.** Two hosts mean every feature is written twice or marked "server browser tabs only", which
several tool descriptions say.
**Size.** About 5,500 lines on the server, 9,600 in the desktop app, 9,200 in the web client.
**Marley today: has most of it.** The Browser tab is its own headless Chromium per project with a
profile per project (#507), tabs saved with the workspace (#494), popups as tabs (Tabs), dialogs
and select lists drawn by Marley (Select lists and dialogs), and an Agent chip naming each action
(What agents can do in the tab). It lacks the control lock: you and an agent can act on the same
page at once, and only the click consequence holds a write for you (#571). It also lacks a view of
an agent-driven page from inside the agent's terminal; the rail marks a page an agent acted in
while no tab showed it (#504).

### 2.5 The agent's browser tools

**What the user sees.** Nothing directly; agents get 19 tools in the `preview` toolkit and two in
`previewControls` (`apps/server/src/mcp/toolkits/preview/tools.ts`,
`apps/server/src/mcp/toolkits/previewControls/tools.ts`). Each carries MCP annotations: read-only
and idempotent for `preview_status`, `preview_snapshot` and `preview_wait_for`, destructive and
open-world for actions that change page state (`preview/tools.ts:46-57`).

**How it works, tool by tool against Marley's** (Marley's list is in Marley's MCP server, The tools,
plus `browser_check_pick` #505, `browser_draft_test` #506 and `browser_open_url` #561):

| T3 tool (line in `preview/tools.ts`) | Marley | Note |
|---|---|---|
| `preview_status` (59) | `browser_tabs`, `browser_look` | T3 adds the control owner and a pending dialog or file chooser |
| `preview_open` (73), `t3_preview_list`, `t3_preview_close` | `browser_navigate` with `new_tab`, `browser_tabs` | Marley has no close tool |
| `preview_navigate` (97) | `browser_navigate`, `browser_open_url` | T3 also takes `{kind:'environment-port',port}` |
| `preview_snapshot` (137) | `browser_snapshot`, `browser_look`, `browser_console`, `browser_network` | T3 returns Playwright's AI aria snapshot with refs, console, network, action history and a PNG in one call, text capped near 20 KB with the omissions listed; `save=true` writes the PNG for the agent's reply |
| `preview_click` (162) | `browser_click` | T3 takes a Playwright locator, legacy CSS, or x and y |
| `preview_type` (173), `preview_press` (231), `preview_scroll` (242) | `browser_type`, `browser_press`, `browser_scroll` | equivalent |
| `preview_hover` (184) | none | |
| `preview_select` (195) | none | Marley keeps a native select shut on an agent's click (Select lists and dialogs) |
| `preview_drag` (209) | none | |
| `preview_upload` (220) | none | answers an open file chooser, or sets an `<input type=file>` by locator |
| `preview_dialog` (86) | none | Marley's dialog card is answered by you only |
| `preview_resize` (108) | none | fill, freeform, or 17 device presets (`packages/shared/src/previewViewport.ts`), without changing the user agent |
| `preview_set_appearance` (121) | none | `prefers-color-scheme` dark, light or system |
| `preview_wait_for` (276) | none | waits until every locator, selector, text and URL condition matches |
| `preview_evaluate` (265) | none, by rule | Marley runs no script an agent supplies (Grants and what keeps an agent in check) |
| `preview_recording_start`, `_stop` (287, 298) | `browser_recordings`, `browser_recording` | T3 makes a video file up to 50 MiB; Marley keeps the last minute as frames and a timeline (#499) |
| none | `browser_pick`, `browser_picks`, `browser_check_pick`, `browser_annotate`, `browser_annotations`, `browser_find`, `browser_draft_test`, `browser_back` | Marley's own |

The implementations are thin Playwright calls in `apps/server/src/preview/ServerBrowserPage.ts`
(640 lines): `snapshot` uses `page.ariaSnapshot({ mode: "ai", boxes: true })` (`:198`), and
`hover`, `select`, `drag`, `setInputFiles`, `evaluate` and `waitFor` follow (`:323-480`).

**Good.** The file-chooser flow is complete for agents: a click on an upload control shows up in
`preview_status`, and `preview_upload` answers it with paths on the environment or an empty list to
cancel (`ServerBrowser.ts:905-958`). Error text tells the agent its next step ("No file picker is
open. Click the page's upload control first, or pass the file input's locator.").
**Bad.** `preview_evaluate` hands an agent arbitrary script in a page that may hold the user's
logins on the desktop host.
**Marley today: has part.** Marley's reading side is as good or better (picks with source
locations, `browser_check_pick`, redaction, the flight recorder, `browser_find`). The acting side
lacks hover, select, drag, upload, dialog answers, viewport and colour-scheme emulation and
waits. Orca's report 03 listed the same gaps as its item 11, and none has been built. Marley's
tools also carry no MCP annotations (`crates/marley_mcp/src/registry.rs` sets none of
`readOnlyHint`, `destructiveHint`, `idempotentHint`, `openWorldHint`).

### 2.6 File choosers, downloads and the human side of the browser

**What the user sees.** On the server browser, a page's file picker opens your own device's picker,
a finished download is offered for you to save, and text the page copies lands on your clipboard
(`remote-access.md:192-197`). Downloads stay on the host until the tab closes. Device presets and a
rotate control sit in a device toolbar (`apps/web/src/browser/BrowserDeviceToolbar.tsx`), and the
viewport can be dragged to a size (`BrowserViewportResizeHandles.tsx`).

**How it works.** `ServerBrowser.ts:817-824` subscribes to `dialog`, `download`, `filechooser` and
`popup` on each page. A download is saved under `server-browser/downloads/<tab>/<id>` and offered
only to the viewer who holds control; agents read it from status (`:849-892`). A file chooser is
offered to the controlling viewer with its `accept` filter and `multiple` flag, a newer picker
replaces an unanswered one, and the answer is `chooser.setFiles` (`:895-946`).

**Marley today: lacks.** Limits of the Browser tab: "Marley draws dialogs and select lists, but not
file choosers, downloads, context menus or the page's cursor shape." An upload form in a project's
app cannot be tested by hand in a Browser tab, and a download goes nowhere you can find.

### 2.7 Element picking and the annotator

**What the user sees.** In the desktop app, pick an element and it becomes a composer chip with its
HTML preview, styles, selector, React component name and source frame. The annotator adds three
tools over the page: select (several elements, Shift to add), marquee regions and freehand
strokes, with a comment and a style panel of text, colour, border and sizing controls that change
the selected elements live. Submitting sends one annotation chip with the elements, regions,
strokes and each style edit as the property, its old value and its new one.

**How it works.** The picker and annotator are one preload script,
`apps/desktop/src/preview/PickPreload.ts`. Style edits are recorded per element and property, the
first computed value kept as `previousValue`, and applied with `!important` so the page shows them
at once (`PickPreload.ts:597-730`, the setter at `700-718`); the baseline inline style is put back
when an element leaves the selection (`:653-668`). The submitted shape, with `elements`,
`regions`, `strokes` and `styleChanges`, is validated in
`apps/desktop/src/preview/PickedElementPayload.ts:70-140`.

**Good.** A style edit sent as `padding: 8px → 16px` is an instruction the agent can apply without
guessing what "a bit more room" means.
**Bad.** The page is edited in place with `!important`, so what the agent sees in a later snapshot
includes your edits until the page reloads.
**Marley today: has part.** Pick, the fuller pick with HTML, styles, components and source (#518),
pick-then-check (#505) and annotation boxes with notes (Annotations) are all there, and Marley's
pick is richer than T3's. Marley has no multi-element selection, no freehand markup (Orca's report
03 item 14, unbuilt) and no live style edits.

### 2.8 Recording a tab as video

**What the user sees.** A record button on a tab; agents start and stop recordings with tools and
get an evidence file's path. Key presses and mouse presses can be drawn into the video
(`apps/desktop/src/preview/RecordingInput.ts`), and the agent's cursor is drawn gliding to its
target (`ServerBrowser.ts:94-96`, `1383-1392`).

**How it works.** The client records with `MediaRecorder`, preferring MP4 (H.264) and falling back
to WebM VP9 or VP8 (`apps/web/src/browser/browserRecording.ts:246-268`); the server side records
the screencast as JPEG frames (`ServerBrowser.ts:92`, `1230-1250`). `preview_recording_stop`
transfers the file once, up to 50 MiB, into the agent's environment (`preview/tools.ts:298-308`).

**Marley today: has part.** The flight recorder keeps the last minute as frames, the timeline and
the agent's actions, and agents read it (#499), and a failed Playwright run saves one (#523). There
is no video file to attach to a pull request or play outside Marley.

### 2.9 Ports and local servers

**What the user sees.** A card per local dev server in the preview's empty state, and
`environment-port` targets for agents.

**How it works.** `apps/server/src/preview/PortScanner.ts` (667 lines) parses
`lsof -iTCP -sTCP:LISTEN -P -n -F pcn`, then publishes a port only after a bounded HTTP(S) probe
finds an HTML document or a redirect to one (`PortScanner.ts:1-17`). `registerTerminalProcesses`
ties listeners to the thread and terminal whose process tree owns them (`:56-62`).

**Marley today: has, and more.** Port rows per project from `/proc/net/tcp`, with systemd units,
containers, Stop, Restart and Show Logs (The rail, #521, #603, #614, #615), and the footer offer for
a URL a terminal printed (#503). Marley lists every listener, a database's included, and says
which project owns it but not which terminal started it.

### 2.10 Browser profiles and cookie import

**What the user sees.** Settings → Integrations → Browser profiles → Add profile → Import from a
browser (`docs/user/browser-import.md:7-10`): a one-time copy of another browser's cookies into a
T3 profile.

**How it works.** `apps/desktop/src/preview/BrowserImport/` (about 6,000 lines) reads Chrome, Edge,
Brave, Vivaldi, Opera, Arc, Helium, Safari and Firefox stores (`Sources.ts:119-205`). On Linux the
cookie key comes from libsecret through a 60-line C helper that looks the key up by attributes
only, so items older keyrings wrote without a schema name still match, and maps a cancelled unlock
to its own exit code (`native/browser-secret/main.c`); `v10` records fall back to the `peanuts`
passphrase (`ChromiumKeys.ts:10-38`, `310-315`). There is no entry for plain Chromium, the browser
Omarchy ships.

**Marley today: lacks.** Orca's report 03 asked the same question (its open question 3). T3 adds
nothing new for a Chromium on Omarchy.

### 2.11 SnapShots: the active window, with its accessibility tree

**What the user sees.** A global key captures the window you are working in and attaches it to the
current draft with the app's name, title and icon and, when the app exposes it, its controls and
text with their positions (`docs/user/snap-shot.md`). Captures that are not yet attached survive a
quit (`snap-shot.md:32`). On Hyprland and Omarchy, setup installs a helper, shows the exact line it
will add to your own Hyprland config, writes only that line with a backup and reloads Hyprland, and
tells you to bind in your own config rather than Omarchy's defaults (`snap-shot.md:83`).

**How it works.**

- `native/hyprland-snap-shot` is a Rust binary (about 1,400 lines, MIT, `wayland-client` 0.31 and
  `smithay-client-toolkit` 0.19). `capture` reads the active window over Hyprland IPC, maps its
  foreign-toplevel handle to the full 64-bit window address with
  `hyprland-toplevel-mapping-v1`, exports the frame with `hyprland-toplevel-export-v1` into a shm
  buffer and writes a PNG, then rereads the window list and returns metadata only if the window is
  still the same one at the same geometry (`src/main.rs:16-38`, `src/capture.rs`). `feedback.rs`
  draws the capture flash.
- The binding is `bind = CTRL SHIFT, 2, global, <app-id>:<action>` through Hyprland's
  GlobalShortcuts portal, written to `bindings.conf` (or the Lua form) under `~/.config/hypr`
  (`apps/desktop/src/snapShot/HyprlandSnapShot.ts:21-46`).
- The accessibility tree is read through AT-SPI with `@crowecawcaw/xa11y`, only in forked workers
  with a 3 s deadline (`SnapShotAccessibility.ts:17`). Matching is by PID plus a unique title and
  size, since AT-SPI reports `(0, 0)` positions on Wayland; positions are kept only when the root
  agrees with the compositor frame (`docs/internals/linux-snap-shot.md`).
- Config edits: setup never reads the file until you press Review changes; Save re-reads the file
  and refuses if its bytes, inode, mode or path changed since the preview, stages a temp file,
  writes a backup beside the original and renames into place
  (`apps/desktop/src/snapShot/CaptureShortcutConfig.ts:225-297`).

**Good.** The config-edit protocol is the most careful code in this area: consent to read, an exact
diff, a re-check before writing, a backup, an atomic rename.
**Bad.** The feature spans about 6,700 lines of desktop code and four desktop-specific backends for
a screenshot with text.
**Marley today: has part.** Omarchy's own screenshot keys put an image on the clipboard or in a
file, Claude Code takes a pasted image, and Marley sends a dropped or attached image as its raw path
(#479, #536). What Marley lacks is the accessibility text and a one-key route from another window
to the agent's prompt. Marley's own config writes are plain: `crates/marley_workbench/src/
agent_notify.rs:292-302` rewrites Codex's `config.toml` with `std::fs::write`, with no backup and no
check that the file is unchanged since it was read.

### 2.12 The Devices panel

**What the user sees.** A live iOS Simulator or Android Emulator in the right panel or floating over
the chat, interactive by mouse and keyboard, with Home, Back, rotate and power, a 3D view for
foldables, and a Tools drawer for dark mode, text size, accessibility settings, location and app
permissions (`docs/user/devices.md`). Devices can run on an SSH host (`devices.md:89`).

**How it works.** The server installs `expo-device-hub` (streaming) and `agent-device` (driving) at
pinned versions after consent, keeps the hub on loopback behind an allowlisting proxy because the
hub's own routes include a shell-exec channel, and runs settings changes itself through `simctl`
and `adb` (`docs/internals/devices.md`). Agents get four tools, `device_list`, `device_open`,
`device_screenshot` and `device_close` (`apps/server/src/mcp/toolkits/device/tools.ts:28-82`), and
drive through the `agent-device` CLI, which a shim prepends to the provider's PATH and which refuses
to run without the `--config` and `--session` that `device_open` returned
(`apps/server/src/device/AgentDeviceShim.ts`). How to drive a device is returned by
`device_open` instead of sitting in an always-loaded prompt. About 8,000 lines.

**Marley today: lacks.** Chad does no mobile work now (Orca's report 03, open question 5), and the
iOS half needs macOS.

### 2.13 Voice input

**What the user sees.** On iPhones with iOS 26, a microphone in the composer that transcribes on
the device into the draft (`docs/user/composer.md`). There is no desktop or web voice input.

**How it works.** A shared controller records the draft's owner and revision before recording and
drops a late transcript if the draft changed (`docs/internals/voice-input.md`).

**Marley today: has.** Dictation through Voxtype, typed where the focus is, with the agent bar's
microphone (The microphone, #480, #642).

## 3. Bring to Marley

1. **Scrollback that survives a restart.** *Why.* Chad restarts Marley every time he installs a new
   build; each restart empties every terminal except what a resumed Claude Code redraws. T3 keeps
   each terminal's raw output, capped at 5,000 lines and 8 MiB, written 40 ms after the last change,
   and replays it on attach with request and response sequences removed, so old queries do not
   make the shell print junk (`Manager.ts:96-99`, `846-930`, `1024-1066`, `1775-1798`). Marley
   would replay the tail into the restored terminal's grid, dimmed, before the new shell's first
   prompt, with the PTY write side disconnected during the replay. *Seam.* The vendored
   `alacritty_terminal` (#461) and `crates/marley_terminal` for the log and the sanitizer; Zed's
   terminal persistence for the file per terminal id (#575 already keeps ids). *Size.* M to L.
   *Hard.* Blocks: the replayed shell reports carry the old nonce, so they would read as
   unverified; persist the block records (command, exit, folder, times) beside the log and redraw
   pills from them, never from replayed bytes. Secrets: the log holds whatever was printed, so it
   needs mode 0600 and a setting to turn it off.

2. **The browser tool gaps, with T3's shapes.** *Why.* Orca's report 03 item 11 listed these and
   none was built; T3 now shows a complete, tested set. `browser_wait_for` (locator, text, URL),
   `browser_hover`, `browser_select` (native select by value or label), `browser_drag`,
   `browser_upload` (answer the open file chooser, or set an input's files by locator),
   `browser_dialog` (accept or dismiss, with prompt text, for a dialog the card shows),
   `browser_resize` (fill, a size, or a device preset, user agent unchanged) and
   `browser_set_appearance` (`prefers-color-scheme`). Add MCP annotations to every Marley tool at
   the same time: read-only and idempotent for the readers, destructive for writes.
   *Seam.* `crates/marley_mcp/src/registry.rs` and the browser tools in `crates/marley_workbench`;
   the acting tools fall under the `browser.write` grant and the click-consequence pause.
   *Size.* S each, M together. *Hard.* Upload paths must be absolute and inside a project or a
   folder the user allowed; select and drag through CDP need the same care the click path took.

3. **File choosers and downloads in the Browser tab.** *Why.* The guide lists both as limits. A
   page's picker should open Zed's own path prompt and answer with `DOM.setFileInputFiles`
   (intercepted with `Page.setInterceptFileChooserDialog`); a download should land in a folder
   with a toast offering Open and Show in Folder. T3's rules are worth copying: a newer picker
   replaces an unanswered one, the controlling person gets the offer and agents read it from
   status, and an agent answers with item 2's upload tool (`ServerBrowser.ts:849-946`).
   *Seam.* `crates/marley_browser` (CDP) and `crates/marley_workbench/src/browser.rs` (the card
   and toast). *Size.* S to M. *Hard.* Headless Chromium needs `Browser.setDownloadBehavior` per
   context.

4. **Take control of a Browser tab.** *Why.* Marley's terminal has Take Over and Hand Back (#525);
   its Browser tab has nothing like it, so a click of yours and an agent's queued click can
   interleave. T3's `SessionControl` refuses agent actions while you hold control and fails any
   action queued before a change of control with a message telling the agent to look again
   (`SessionControl.ts:58-98`). Ctrl-I in a Browser tab, or a Take Over button beside the Agent
   chip, would do the same, and the refusal would name the user. *Seam.* The browser write path
   in `crates/marley_workbench` and the tools' refusal text in `crates/marley_mcp`. *Size.* S.
   *Hard.* Deciding whether ordinary input takes control by itself; start with the explicit key.

5. **Live style edits in an annotation.** *Why.* For UI work, "make this padding 16px" said by
   changing it on the page and sending `padding: 8px → 16px` is clearer than a note on a box. T3's
   annotator also selects several elements, draws regions and freehand strokes
   (`PickPreload.ts:597-730`). Marley would add a small style panel to annotate mode for the
   selected element, apply edits through CDP (`CSS.setStyleTexts` or an inline style), and put
   them in `browser_annotations` with the element's locators from the pick model.
   *Seam.* Annotate mode in `crates/marley_workbench/src/browser.rs`, the annotation record in
   `crates/marley_mcp`. *Size.* M. *Hard.* Undoing the edits when the annotation is sent or
   dropped, so later snapshots show the real page; T3 leaves them in place.

6. **Reviewed, verified edits to other tools' config files.** *Why.* Marley writes Codex's
   `config.toml`, OpenCode's plugin folder and Claude Code's plugin marketplace on a click (#552,
   The Claude Code plugin), with a plain write (`agent_notify.rs:292-302`). T3's protocol for the
   Hyprland binding: show the exact before and after, re-read before writing and refuse if bytes,
   inode, mode or path changed, stage a temp file with the original's mode, keep a backup, rename
   into place (`CaptureShortcutConfig.ts:225-297`). *Seam.* A small helper in
   `crates/marley_workbench` used by `agent_notify.rs` and every later config write. *Size.* S.
   *Hard.* None worth naming.

7. **A tab recording as a video file.** *Why.* Evidence for a pull request or a bug report that
   plays anywhere. Marley's flight recorder already holds frames and a timeline; an Export as
   Video on a recording, and an agent tool that records between start and stop, would encode them
   (ffmpeg on the PATH, or GStreamer) with the agent's cursor and key presses drawn in, as T3's
   `RecordingInput.ts` and agent cursor do. *Seam.* `crates/marley_browser` recordings.
   *Size.* S to M. *Hard.* Chromium sends frames only on change, so the encoder needs timestamps
   from the timeline, not a fixed rate.

8. **Port rows that know which serve pages and which terminal started them.** *Why.* T3 offers a
   port only after an HTTP(S) probe finds HTML or a redirect to it (`PortScanner.ts:1-17`), and
   ties each listener to the terminal whose process tree owns it (`:56-62`). Marley's rows could
   mark the ones that serve a page (Open shows only there) and name the terminal, with a click to
   it. *Seam.* The port scan behind `ports_list` (#521). *Size.* S. *Hard.* A probe must never
   send anything but a plain `GET /` with a short deadline.

9. **A live view of the page an agent drives, over its terminal.** *Why.* While Claude Code drives
   a Browser tab in another pane, you watch the terminal and see nothing of the page. T3 floats a
   mini preview over the chat when an agent opens a page, with the agent's cursor
   (`ThreadPreviewMiniPlayer.tsx`, `AgentBrowserCursor.tsx`). Marley holds the screencast frames
   already; a small overlay in the agent's terminal pane, shown while that agent acts and gone
   five seconds after, would do it. *Seam.* `crates/marley_workbench`. *Size.* M. *Hard.* Knowing
   which terminal's agent made the call, which #520's terminal identity answers.

10. **The active window into the agent's prompt.** *Why.* Lower value on Omarchy, whose screenshot
    keys and Claude Code's image paste cover the image. What T3 adds is the window's accessibility
    text, so an agent can read a native dialog's message or a settings window's values. The Rust
    helper `native/hyprland-snap-shot` is MIT and close to what Marley would write; a Marley
    command bound in Hyprland would capture the active window and type its path, title and text
    file at the focused agent's prompt. *Seam.* A small binary or a module in
    `crates/marley_workbench`; the binding stays in Chad's own Hyprland config. *Size.* S to M.
    *Hard.* AT-SPI matching on Wayland, which T3's notes describe in detail.

## 4. Skip

- The `libghostty-vt` renderer and its WebAssembly build: Marley draws with `alacritty_terminal`
  in GPUI and gains nothing from a second emulator.
- Server-owned PTYs streamed to any client: process survival is the harness's job (#632) and
  phone access is report 04's.
- Busy detection by polling the process table: Marley's shell integration knows each command's
  start and end.
- `preview_evaluate`: Marley's rule that no tool runs an agent's script stays, unless Chad says
  otherwise (question 2).
- Isolated agent contexts: Marley chose one profile per project shared by you and your agents
  (#507), so an agent can act on a page you are signed in to. That choice still fits a single
  user's machine.
- A pinned Chrome for Testing download: Marley runs Arch's Chromium. Revisit only if a Chromium
  update breaks the Browser tab.
- The Electron `<webview>` host and its CDP relay: Marley has one headless Chromium and draws its
  screencast.
- Cookie import: nothing for Omarchy's Chromium, and Orca's open question 3 still stands.
- The Devices panel: no mobile work, and the iOS half needs macOS. Keep its one lesson: return a
  tool family's how-to from its open call rather than an always-loaded prompt.
- SnapShot backends for GNOME, KDE and Niri, the capture sound and the fly-in animation.
- Voice input: Marley's Voxtype dictation already covers the desktop; T3's is iPhone only.

## 5. Open questions

1. **Restored scrollback.** How much, and how drawn? *Default: T3's caps (5,000 lines, 8 MiB),
   replayed dimmed above the new prompt, blocks redrawn from saved records, on unless turned off.*
2. **`browser_evaluate`.** Allow an agent's script in a page behind its own grant, off by default?
   *Default: no; the rule in Grants and what keeps an agent in check stands.*
3. **Downloads.** Into the XDG download folder, or a folder per project under Marley's data?
   *Default: the XDG download folder, with a toast naming the file.*
4. **Taking control.** Only by a key and a button, or also whenever you type into a page an agent
   acted on in the last few seconds? *Default: the key and the button only.*
5. **Window capture.** Wanted at all, given Omarchy's screenshot keys? *Default: not built.*
