# Orca survey 03: the embedded browser, Design Mode, computer use, the emulator

Read at Orca commit `1c2cf120e3`. Paths are relative to `/srv/stacks/orca-refs/orca` unless they
start with `crates/`, which are Marley's (`/srv/stacks/marley_ide`).

## 1. Summary

Orca's browser is an Electron `<webview>` per tab; profiles are Electron persistent partitions.
Agents drive the same tabs through the `orca` CLI over a Unix socket; the app runs the bundled
`agent-browser` (Vercel) against a fake DevTools endpoint backed by `webContents.debugger`.
"Design Mode" is Grab (a click puts the element's context on the clipboard), Annotate
(element-tied comments pasted as one prompt into an idle agent's terminal) and Markup (draw on a
frozen screenshot, PNG to the clipboard). No recording, test generation or before/after check.
Worth taking: (1) the richer pick payload (bounded HTML, computed styles, React component chain
and source, redaction), which Marley's pick lacks; (2) ports attributed to a worktree, for
#503/#504; (3) storage keyed on durable identity: CDP contexts alone lose logins (#507); (4)
agent commands scoped to the caller's worktree; (5) sends only to an idle agent.

## 2. Features

### 2.1 Browser engine and embedding

**What the user sees.** A Browser tab inside a worktree's tab groups (splits allowed) with an
address bar (history, fuzzy completion, search fallback), back/forward, reload (right-click for
hard reload), find in page, a downloads shelf, zoom, viewport presets, an Open DevTools button,
Open in Default Browser, and a profile/cookie menu. `Cmd/Ctrl+T` opens a tab scoped to the
worktree; switching worktrees restores that worktree's tabs and scroll positions.

**Underneath.**
- The renderer mounts one Electron `<webview>` per page with `partition=<profile partition>`
  (`src/renderer/src/components/browser-pane/host-guest/attach-browser-page-webview.ts`,
  `host-guest/webview-registry.ts`). Main checks every attach in `will-attach-webview`
  against an allowlist of partitions and URL schemes, fail-closed
  (`src/main/window/main-window-webview-security.ts`,
  `src/main/browser/browser-session-registry.ts` `isAllowedPartition`), then registers the
  guest's WebContents with `BrowserManager` (`src/main/browser/browser-manager*.ts`).
- An Electron `<webview>` destroys its guest when its DOM node moves. So every browser pane of a
  worktree renders in one overlay layer and follows its tab group's body through CSS
  `position-anchor`; moving a tab between splits only swaps the anchor name
  (`assemble-chrome/BrowserPaneOverlayLayer.tsx`).
- Headless `orca serve` has no renderer, so pages are hidden offscreen `BrowserWindow`s behind
  the same interface (`src/main/browser/browser-backend.ts`, `offscreen-browser-backend.ts`;
  on Linux it needs Xvfb because Electron's `--headless` segfaults). The daemon can also run an
  external Chromium with its own profile through agent-browser
  (`src/main/orcad/external-chromium-browser-session.ts`).
- `target=_blank` links and unnamed popups become Orca tabs with the opener's profile, at most 4
  per 2 s so one hostile click cannot plant a hundred tabs
  (`src/main/browser/browser-page-initiated-tab-budget.ts`).
- Local HTTPS dev servers: a "proceed" is offered only for loopback hosts (`localhost`,
  `*.localhost`, 127.0.0.0/8, `::1`) and is pinned to the leaf certificate's SHA-256
  (`browser-certificate-trust-controller.ts`, `browser-certificate-identity.ts`,
  `src/shared/browser-url.ts` `isEligibleLocalCertificateHost`).
- Viewport presets copy Chrome DevTools' list (Mobile S 320×568 to Desktop 1920×1080; phones
  and tablet at DPR 2 with `mobile: true`), applied as CDP device metrics
  (`src/shared/browser-viewport-presets.ts`, `browser-manager-viewport.ts`).
- Favicons come from Electron's `page-favicon-updated`. Rules worth copying: `data:,` means "no
  icon", take the first http(s) or `data:image/` entry, drop the icon when the page leaves its
  origin, keep it while the origin is still unknown
  (`describe-page/browser-favicon-url.ts`, `src/renderer/src/components/browser-favicon.tsx`,
  which shows a spinner while loading).
- Browser tabs are not rows in the left sidebar. They are found through the Cmd+J palette
  (favicon, title, URL match, "Current Tab" badge;
  `src/renderer/src/components/worktree-jump-palette-browser-simulator-rows.tsx`).

**Good and bad.** A real Chromium view gives native painting, IME, drag and drop, WebAuthn and
DevTools for free. The price is a long list of Electron workarounds in the automation path:
`Page.captureScreenshot` hangs on webview guests (rerouted to `capturePage`),
`Page.printToPDF` is missing, CDP subscriptions lapse after cross-process swaps, and
`Input.insertText` needs native focus, which steals the foreground (`src/main/browser/cdp-ws-proxy.ts`).
`src/main/browser` is 529 files: 41.5k lines of source and 54k of tests.

**Size.** A subsystem: about 41k lines in main and 24k in the renderer pane.

**Marley today.** Has the core by another route: headless Chromium as a systemd user unit, its
screencast drawn in a Zed tab with an address bar, popups through `Target.targetCreated` and
`openerId` (`crates/marley_browser/src/service.rs`, `page.rs`,
`crates/marley_workbench/src/browser.rs`). Lacks find in page, downloads, viewport presets,
DevTools, favicons and a local-HTTPS proceed. It sets no user agent, so pages see
`HeadlessChrome/…`.

### 2.2 Streamed pages and remote workspaces

**What the user sees.** For a workspace on a paired remote Orca server, new pages render on the
desktop by default while HTTP, WebSocket, DNS and loopback traffic go through the remote host; a
toolbar indicator names the host. A setting switches new pages to "Server (streamed)". For SSH
workspaces, a toggle routes browser traffic through the SSH host
(`docs/site/content/docs/browser/overview.mdx`).

**Underneath.**
- Streamed pages are Marley's architecture: `Page.startScreencast` on the server, frames sent
  in a binary envelope (16-byte header, JSON metadata: device size, scale, scroll, offsetTop)
  and drawn in the renderer as images, with input sent back as RPCs
  (`src/main/browser/browser-screencast-stream.ts`, `src/shared/browser-screencast-protocol.ts`,
  `src/renderer/src/components/browser-pane/stream-remote/*`, 4.4k lines).
- Lessons in that code: ACK each frame only when it was sent, so Chromium stops producing frames
  under back-pressure; keep the newest throttled frame and flush it after the interval, because
  a static page's last frame may be the only one (`browser-screencast-frame-pacer.ts`); capture
  a snapshot frame 250 ms after navigation, because a static page can finish loading without a
  screencast frame (`browser-screencast-snapshot-capture.ts`); the session that saw a JS dialog
  is the only one that can answer it, so a stopping stream dismisses it first; wheel events are
  coalesced to one per animation frame with one in flight (`use-remote-browser-page-wheel.ts`).
- SSH egress: a loopback SOCKS5 server per SSH target whose dials go through the SSH connection;
  the page's partition proxies at it, DNS is resolved remotely, a dial while disconnected fails
  instead of falling back to a direct connection, and WebRTC IP handling is restricted
  (`local-ssh-browser-route.ts`, `remote-browser-socks-server.ts`, `browser-route-webrtc-policy.ts`).
- Storage per remote host: route partitions are derived from the execution host's durable
  identity. The comment in `browser-execution-host-storage-identity.ts` records the bug: an
  earlier version hashed per-boot values (runtime start time, SSH connection generation) into
  the partition name, minting a fresh partition, and so losing cookies and localStorage, on
  every restart or reconnect.

**Good and bad.** Careful and heavily fenced; the placement, tunnel and route code is 16.7k
lines. It exists because Orca pairs desktops, servers and phones.

**Marley today.** The Browser tab is a local screencast; there is no remote-host browsing yet
(`marley_remote` only builds an `ssh` argv).

### 2.3 Link routing, ports and localhost labels

**What the user sees.** A setting decides whether http(s) links clicked in the terminal,
Markdown and the editor open in Orca's browser or the system browser, with a nested toggle that
makes `Shift+Ctrl+click` (`⇧⌘-click`) invert that for one click. Terminal links offer both
destinations in a popover ("Orca Browser" / "System Browser"). Each worktree card in the
sidebar shows a plug icon with its live ports; each port row has Open in Browser, Copy and Stop.
A Ports panel lists all of them. An optional setting opens a worktree's ports as
`http://<project>-<worktree>.orca.localhost:<proxy-port>/…`.

**Underneath.**
- Routing: `src/renderer/src/lib/http-link-routing.ts` (`openHttpLink`,
  `resolveModifierRouting`), `http-link-destinations.ts`.
- Ports: on Linux, listening sockets from `/proc/net/tcp` and `/proc/net/tcp6`, each socket's
  inode mapped to a pid by reading the `socket:[inode]` links under every `/proc/<pid>/fd`, then
  the pid's `comm`, `cmdline` and `cwd` (`lsof` on macOS, `netstat` on Windows). The listener's
  cwd, or failing that its command line, is matched to the deepest worktree path
  (`src/main/ports/local-workspace-platform-port-scanner.ts`,
  `local-workspace-port-attribution.ts`); a scan that times out backs off from one to five
  minutes (`workspace-port-scan-timeout-backoff.ts`). 0.0.0.0 and `::` binds are opened as
  127.0.0.1 and `::1` (`src/shared/localhost-worktree-labels.ts` `connectableLoopbackHost`,
  `src/renderer/src/lib/workspace-port-urls.ts`).
- Printed URLs: `AdvertisedUrlWatcher` buffers each PTY's output and scans only at line breaks
  (a URL or an escape sequence can straddle two writes), strips OSC/CSI, replaces cursor moves
  with a guard character so skipped cells cannot fuse two words into a URL, drops trailing
  punctuation, validates with `new URL`, and caches the URL per (worktree, port). An entry is
  validated against the live listener and evicted when the listener disappears, so the dev
  server's own host name (`myapp.test:5173`) wins over the numeric bind
  (`src/main/ports/advertised-url-parsing.ts`, `advertised-url-watcher.ts`,
  `advertised-url-reconciliation.ts`).
- Labels: a reverse proxy on 127.0.0.1 maps `<label>.orca.localhost` to the worktree's port,
  including WebSocket upgrades, headers and bodies untouched except `Host`
  (`src/main/localhost-worktree-label-proxy.ts`). The setting's stated reason is telling tabs
  apart; the side effect is a distinct origin per worktree, so cookies stop colliding (cookies
  ignore ports: `localhost:3000` and `localhost:5173` share a jar).

**Good and bad.** The port attribution works for servers started anywhere, not only in an Orca
terminal. The label proxy changes the origin, which will break OAuth redirect URIs and CORS
allowlists registered for `localhost:<port>`; it is opt-in for that reason.

**Size.** Ports 2.4k lines, routing and labels 0.8k.

**Marley today.** Terminal URLs open the system browser. No port list. Queued as #503.

### 2.4 Design Mode, part 1: Grab (the element picker)

**What the user sees.** "Grab page element" in the toolbar, or `Mod+C` when nothing is selected
and focus is not in an editable field (keybinding `browser.grabElement`,
`src/shared/keybindings/definitions-core-3.ts`; guard in
`src/main/browser/browser-guest-grab-shortcuts.ts`). The cursor becomes a crosshair; a white box
and a label (`button role=… "Save" 120x32`) follow the element under the pointer. A left click
copies the element's context to the clipboard as text and re-arms for another pick. A right
click opens a menu: Copy Contents (C), Copy Screenshot (S), Cancel. While armed, bare C or S
copies the hovered element's text or screenshot without clicking. Esc cancels.

**Underneath.**
- The picker runs in the page's main world through `webContents.executeJavaScript`
  (`src/main/browser/grab-guest-*.ts`). `arm` installs a full-viewport host at z-index 2³¹−1 with
  a closed shadow root; it catches all pointer events, and hit-tests with
  `document.elementFromPoint` after briefly turning its own pointer events off.
  `awaitClick` returns a promise that resolves with the payload; clicks are consumed in the
  page because Electron's `before-input-event` never sees mouse events on guests
  (`browser-grab-session-controller.ts`). One pick per tab at a time, 120 s timeout, cancelled
  on main-frame navigation.
- The payload (`src/shared/browser-grab-types.ts`, built in `grab-guest-react-script.ts`):
  - page: URL with query and fragment stripped (only http, https, file, about:blank), title,
    viewport, scroll, DPR, time;
  - target: tag; a unique CSS selector built from id, up to two "stable" classes (it skips
    `css-xxxx` and 12+ character mixed-case hashes) and `:nth-of-type`; a readable path and a
    full DOM path; classes; up to six sibling descriptions; the selected text; whether a
    fixed or sticky ancestor exists; the React component chain (up to six names from
    `__reactFiber$…` walking `.return` 35 levels, skipping Provider/Router/Boundary-style
    names); the source `file:line:col` from the fiber's `_debugSource`, with webpack,
    turbopack and Next.js layer prefixes cleaned; a 200-character text snippet; outerHTML with
    `<script>` removed, up to 4,096 characters; allowlisted attributes plus `aria-*`; role and
    accessible name; the box in viewport and page coordinates; 16 computed styles (display,
    position, width, height, margin, padding, color, background, border, radius, font family,
    size, weight, line height, text align, z-index);
  - context: up to ten sibling texts and ten ancestors.
- Redaction, done in the page and again in main (`src/main/browser/browser-grab-payload.ts`):
  any attribute or metadata value containing `access_token`, `api_key`, `client_secret`,
  `session_id`, `csrf`, `secret`, `password` and the like becomes `[redacted]`; URL attributes
  lose their query strings.
- Crop: main hides its own overlay and the annotation badges, `capturePage()`s the guest,
  derives the scale as bitmap width ÷ `innerWidth` (right on mixed-DPI setups), crops to the
  viewport box, and drops the PNG if it is over 2 MB (`browser-grab-screenshot.ts`).
- Clipboard text: `formatGrabPayloadAsText` in `annotate/GrabConfirmationSheet.tsx`
  ("Attached browser context from …", element, accessible name, role, selector, source,
  React, size, text, nearby context, a few styles, HTML, paths).

**Good and bad.** The payload is well chosen for "fix this element" and the budgets keep it
bounded. Weaknesses:
- `elementFromPoint` in the top document cannot pick inside iframes or shadow roots; Chromium's
  inspect mode, which Marley uses, reaches into shadow roots and same-origin frames (Marley's
  `pick.rs` walks both).
- Main-world injection is visible to the page. The code tears down any pre-existing
  `window.__orcaGrab` before arming and re-validates everything in main, but it had to work
  around Zone.js replacing `Promise`.
- `_debugSource` exists only in React 18 and older dev builds; React 19 removed it, so the
  source field is empty on React 19 apps such as Next.js 15.
- The crop covers the viewport only, so an element partly off screen is cut or missing.
- The docs and the code disagree. `docs/site/content/docs/browser/design-mode.mdx` says the
  capture "ships into the active agent terminal as one attachment" and includes "the source
  file/line if a dev-mode source map is available". In the code the pick goes to the clipboard,
  source maps are not used, and the "Attach to AI" sheet (`GrabConfirmationSheet`'s default
  export) is mounted nowhere.

**Size.** About 2.3k lines in main and shared, plus a share of the renderer's 4.4k in `annotate/`.

**Marley today.** Has a stronger picker in some respects and a weaker bundle in others. Marley
uses Chromium's inspect mode, walks to the nearest interactive ancestor, ranks locators (test id,
id, text, CSS path, marked unique where it can tell), lists what would block a click, maps listeners
through source maps and opens the file, crops the page, stages picks in a tray with a caption,
and types a reference line into the last terminal for the agent to read through `browser_pick`
(`crates/marley_browser/src/pick.rs`, `browser.rs` `send_pick`, `pick_line`). The brief says
Marley's pick sends the element's HTML and computed CSS; `PickBundle` has neither (tag, role,
name, text, locators, listeners, blockers, box, crop), and nothing in `marley_browser` or
`marley_workbench` calls `DOM.getOuterHTML` or reads computed styles. Marley also has no React
(or other framework) component resolution. That matters because React 17+ attaches event
handlers to the root container, so a React button's listeners map to react-dom's code, not the
component that rendered it.

### 2.5 Design Mode, part 2: annotations and sending feedback to an agent

**What the user sees.** "Annotate page element" arms the same picker. A click opens a card at
the element: a comment (up to 2,000 characters) and an intent, Change or Question;
`Cmd/Ctrl+Enter` adds it. Numbered blue badges stay on the annotated elements as the page
scrolls. A tray lists the annotations (edit, delete, copy all, clear) and has "Send feedback
to an agent". That highlights the worktree's running agents in the sidebar; agents that cannot
take input are disabled with a reason ("Agent needs permission", "Agent status is stale",
"Terminal is no longer available"). Picking one pastes the feedback into its terminal, submits
it, and removes the delivered annotations from the tray.

**Underneath.**
- Shape: `BrowserPageAnnotation { id, browserPageId, comment, intent, priority, createdAt,
  payload }`, where the payload is the Grab payload minus its screenshot
  (`src/shared/browser-grab-types.ts`). The types also hold intents `fix` and `approve` and a
  priority, which the UI no longer shows. At most 20 per page; a 21st silently drops the oldest
  (`store/slices/browser/browser-page-metadata-actions.ts`).
- Badges: a script in isolated world 1207 draws them in a closed shadow root and reports scroll
  offsets back as `console.debug` lines carrying a random token prefix, which the renderer picks
  out of the webview's `console-message` events to place its own popovers
  (`src/shared/browser-annotation-viewport-bridge.ts`,
  `src/main/browser/browser-manager-viewport.ts`,
  `host-guest/browser-page-webview-navigation-handlers.ts`).
- Prompt: `formatBrowserAnnotationsAsMarkdown` (`annotate/browser-annotation-output.ts`) writes
  `## Design Feedback: <path>`, the URL, the tab id and the viewport, then per annotation the
  element label (React component plus tag and name), intent, selector, readable path, source,
  React chain, bounds, classes, text, nearby text and elements, non-default computed styles,
  full DOM path, the HTML in a fence sized to its backticks, and the feedback.
- Targets: `deriveRunningAgentSendTargets` (`src/renderer/src/lib/running-agent-targets.ts`)
  reads the agent status that hooks and live terminal titles report per pane.
- Delivery (`src/renderer/src/lib/active-agent-note-send.ts`,
  `active-agent-note-send-delivery.ts`): check the agent is "sendable"; send the prompt as one
  bracketed paste (`ESC[200~ … ESC[201~`, sanitized) with `requireAgentStatus: 'sendable'` so
  the runtime refuses if the agent just hit a permission prompt; wait 50 ms; check again; send
  Enter as a separate write. A failure after the paste reports "partial-submit-failed" rather
  than "not sent". With no explicit target, it first waits up to 8 s for the TUI to go idle.
- Storage: renderer memory only (`browserAnnotationsByPageId`). It is emptied at startup
  (`store/slices/browser/browser-hydration-actions.ts`) and when the page's URL changes
  (`browser-page-state-actions.ts`).

**Good and bad.** Good: annotations are tied to DOM elements, not screen regions; the batch
prompt carries enough to act on; the send path respects agent state. Bad: agents cannot list or
read annotations through the CLI; they get pasted text or nothing. A page navigation or a
restart silently drops unsent feedback. Pasting 4 KB of HTML per annotation fills the agent's
context fast.

**Size.** About 2.3k lines of renderer code plus the shared send path.

**Marley today.** Has annotations of another kind: boxes with a note drawn over the page by the
user or by an agent (`browser_annotate`), readable by agents through `browser_annotations`
(`browser.rs` `Annotation`). They carry no element and no intent, and there is no batch send.
Picks go to "the terminal the user focused last" with no check of the agent's state
(`browser.rs` `send_pick`).

### 2.6 Design Mode, part 3: Markup

**What the user sees.** A pen button captures the page and freezes it under a drawing layer:
pen, highlighter, arrow, rectangle, ellipse, text; seven colors, three widths, five font sizes;
undo and redo. Copy composites a PNG and puts it on the clipboard ("Draw on the page, then copy
the markup to paste into your agent.").

**Underneath.** Shapes are vectors in CSS viewport coordinates, rendered live on a canvas and
again at output scale for the PNG, within the clipboard handler's byte and pixel limits
(`annotate/markup-drawing-model.ts`, `markup-screenshot-compose.ts`, `useMarkupMode.ts`,
`MarkupOverlay.tsx`, `MarkupToolbar.tsx`). It works on streamed pages too, from the last frame.

**Size.** About 2k lines.

**Marley today.** No freehand markup; annotation boxes cover part of the use.

### 2.7 The fix-and-verify loop

**What the user sees.** `docs/site/content/docs/recipes/design-mode-fix.mdx`: grab, describe
the change, let the agent edit, let hot reload refresh, "click the element again to verify".

**Underneath.** Nothing. No re-find, no second crop, no comparison.

**Marley today.** Same; queued as #505.

### 2.8 Browser profiles and isolation

**What the user sees.** Settings → Browser → Profiles: add a named profile. The toolbar picks
the profile for a tab; agent commands inherit the tab's profile. A tab can be cloned into
another profile. Deleting a profile clears its storage.

**Underneath.**
- `BrowserSessionProfile { id, scope: default | isolated | imported, partition, label, source }`
  (`src/shared/browser-workspace-types.ts`). The default partition is `persist:orca-browser`
  (per Orca user profile); the others are derived deterministically from the profile id so the
  allowlist rebuilds at startup (`src/main/browser/browser-session-registry.ts`,
  `src/shared/orca-profiles.ts`). Profile metadata, the recorded import source and pending
  staged imports are JSON in userData (`browser-session-meta-store.ts`).
- `persist:` partitions keep cookies, localStorage, IndexedDB, service workers and cache on disk.
- New tabs take a per-host default profile, else the global default
  (`src/renderer/src/store/slices/browser/browser-tab-actions.ts`). There is no per-worktree
  default: every worktree's tabs share the default profile's cookies unless the user switches.
  "Per-worktree browser" in the docs means per-worktree tab lists, not per-worktree storage.
- CLI: `orca tab profile list|create --scope isolated|imported|delete|set|show|use-default|clone`,
  `orca cookie get|set|delete`, `orca storage local|session get|set|clear`.
- Browser identity: "Cleaned" strips the Orca and Electron tokens from the user agent to look
  like Chrome; on `accounts.google.com` and `accounts.youtube.com` only, it presents a Firefox
  140 user agent and strips Chromium's `sec-ch-ua*` client hints, because Google refuses
  embedded browsers and flags an inconsistent UA (`browser-google-auth-ua.ts`,
  `browser-session-ua.ts`, `browser-identity-mode-store.ts`). "Native" keeps Electron's UA for
  sites that reject the cleaned one. A change needs a restart.

**Good and bad.** Simple model, persistent storage, isolation on request. The docs promise
seeding a profile "with cookies and a viewport size"; the profile type has no viewport.

**Size.** Session registry and partition policies about 2k lines; identity 0.6k.

**Marley today.** One Chromium profile under Marley's data directory for every tab
(`service.rs`, `--user-data-dir`, `--password-store=basic`). Queued as #507.

### 2.9 Cookie import and Google sign-in

**What the user sees.** Import cookies into a profile from Chrome, Chromium, Arc, Edge, Comet,
Helium, Firefox, Safari, or a JSON file (Settings or the toolbar). Only the imported domains'
cookies are replaced. Google cookies are skipped, and the menus say "Google logins aren't
imported. Sign in to Google directly in Orca." (`src/renderer/src/components/BrowserCookieImportDisclosure.tsx`).

**Underneath.**
- Chromium on Linux: the key is the "Safe Storage" password from `secret-tool lookup` (v11); when
  no keyring is available Chromium writes v10 with the fixed password `peanuts`, which Orca also
  tries (`src/main/browser/browser-cookie-key.ts`). macOS reads the Keychain; Windows unwraps the
  DPAPI key through PowerShell over stdin, and v20 "app-bound" cookies (Chrome 140+) cannot be
  decrypted and are counted. Values are AES-128-CBC (GCM on Windows), and Chromium 127+/schema
  24 values carry a 32-byte SHA-256(host) prefix that must be stripped
  (`browser-cookie-decryption.ts`).
- Writes go two ways: live writes through CDP, because Electron's `cookies.set` silently drops
  CHIPS `partitionKey`; and a staged copy of the partition's SQLite, merged before
  `session.fromPartition` at the next cold start, because CookieMonster rejects some valid bytes
  (`browser-cookie-import-pipeline.ts`, `browser-cookie-chromium-*.ts`,
  `browser-session-cookie-staging.ts`). Replacement, writes and rollback run under one
  per-partition lock.
- `google.com` is excluded: Google binds a session to the browser that created it, and
  transplanted cookies are flagged and expire within about an hour
  (`browser-cookie-import-policy.ts`, `browser-google-auth-ua.ts`).

**Good and bad.** Thorough about data loss (rollback, domain-scoped replace, honest skip counts),
but 4.8k lines of source and 9.3k of tests for a convenience feature, with a history of
fixes (STA-4097, STA-4300, STA-4601 and STA-4797 in the comments).

**Marley today.** None.

### 2.10 WebAuthn security keys

HID permission is granted only for devices on the FIDO usage page (0xF1D0) and only on https or
localhost origins; Electron's `select-webauthn-account` opens an account picker in the renderer
instead of cancelling silently. Platform passkeys are not supported
(`src/main/browser/browser-webauthn-access.ts`, `browser-webauthn-account-picker.ts`,
`src/renderer/src/components/browser-webauthn-account-dialog.tsx`; about 260 lines).
**Marley today:** none; headless Chromium shows no security-key UI.

### 2.11 The agent-facing browser: the `orca` CLI and agent-browser

**What agents see.** Eighty browser commands (`src/cli/specs/browser-basic.ts`,
`browser-advanced.ts`): `snapshot`, `screenshot`, `full-screenshot`, `pdf`, `click`, `dblclick`,
`fill`, `type`, `inserttext`, `select`, `check`, `uncheck`, `focus`, `clear`, `select-all`,
`keypress`, `hover`, `drag`, `upload`, `scroll`, `scrollintoview`, `goto`, `back`, `forward`,
`reload`, `eval`, `wait` (selector, text, URL, load state, JS condition, hidden or visible),
`get` (text, html, value, url, title, count, box), `is` (visible, enabled, checked), `find`
(semantic locator plus action), `mouse move|down|up|wheel`, `tab list|show|current|switch|create|close`,
the profile commands, `cookie`, `storage`, `viewport`, `geolocation`,
`set device|offline|headers|credentials|media`, `intercept enable|disable|list`,
`capture start|stop`, `console`, `network`, `clipboard read|write`, `dialog accept|dismiss`,
`download`, `highlight`, `open-url`, and `exec --command "<any agent-browser command>"`. The loop
the skill teaches is snapshot, act on an `@eN` ref, snapshot again
(`skill-guides/orca-cli/references/browser.md`; `skills/orca-cli/SKILL.md` is a stub that runs
`orca skills get orca-cli` to load the guide matching the installed build).

**Targeting.** Commands default to the worktree the CLI's cwd is in and that worktree's active
tab (`src/cli/selectors.ts` `getBrowserCommandTarget`); `--page <browserPageId>` pins a tab for
concurrent work; `--worktree all` widens it. Agents in parallel worktrees therefore drive their
own tabs without saying so.

**Transport.** The CLI reads the runtime metadata file in userData (endpoints and an auth
token), connects to a Unix socket or named pipe, and sends newline-delimited JSON RPC such as
`browser.click` (`src/cli/runtime/transport.ts`, `metadata.ts`;
`src/main/runtime/rpc/methods/browser-core.ts`).

**In the app.**
- `AgentBrowserBridge` (`src/main/browser/agent-browser-bridge*.ts`, created in
  `src/main/startup/main-process-ready-runtime.ts`) keeps one session per tab named
  `orca-tab-<pageId>` and a queue per tab.
- Each session owns a `CdpWsProxy`: an HTTP and WebSocket server on 127.0.0.1 at a random port
  that answers Chrome's discovery endpoints (`cdp-target-discovery.ts`) and forwards CDP to
  Electron's `webContents.debugger` (`cdp-debugger-channel.ts`), patching what Electron guests
  get wrong (screenshots, PDF, navigate and reload with lifecycle priming, focus replay for
  `DOM.focus` and `Input.insertText`).
- A command runs `agent-browser --session orca-tab-<id> --cdp <port> <args> --json` via
  `execFile`. The agent-browser client forks a daemon per session that outlives it, so Orca sets
  a 10-minute idle timeout, a private socket directory `/tmp/orca-ab-<hash>`, and sweeps orphans
  at startup (`agent-browser-process-environment.ts`, `agent-browser-orphan-sweep.ts`).
- Some commands skip agent-browser: `goto` calls `loadURL` and treats a page's unload veto as a
  failure; `fill` sets the value through one `eval --stdin` because agent-browser's text insertion
  loses focus in Electron guests.
- Errors carry codes agents can branch on: `browser_stale_ref`, `browser_tab_not_found`,
  `browser_no_tab`, `browser_host_unavailable`.
- The snapshot is agent-browser's accessibility tree with `@eN` refs. An older in-house engine
  is still in the tree and tested, but `CdpBridge` is no longer constructed at startup
  (`cdp-bridge.ts`, `snapshot-engine.ts`, `snapshot-ax-tree-walk.ts`,
  `snapshot-cursor-interactive-elements.ts`, `cdp-ref-resolution.ts`). Three ideas from it are
  worth keeping: elements that are clickable but have no ARIA role (cursor:pointer, onclick,
  tabindex, contenteditable) become refs with role `clickable`; duplicate role and name pairs get
  a `(2nd)` suffix; a ref whose node was re-rendered is recovered by role, name and occurrence
  before giving up. Its line format was `[@e3] button "Submit"`, indented.

**Good and bad.** Broad and consistent, with version-matched guides and recovery codes. Not an
MCP server: Orca relies on shell access plus skills. Agents cannot read Grab picks or
annotations. `orca console --limit` and `orca network --limit` are accepted and then ignored
(`agent-browser-bridge-state-commands.ts`, `consoleLog(_limit, …)`). Bundling a third-party CLI
with its own daemon lifecycle cost Orca idle timers, socket-directory ownership rules and an
orphan sweep.

**Size.** Bridge, proxy and snapshot code about 8k lines of source and 7.4k of tests; CLI specs and
handlers about 1.7k.

**Marley today.** Has the MCP family (tabs, navigate, look, snapshot with refs, click, type,
press, scroll, console, network, picks, annotations, recordings;
`crates/marley_workbench/src/browser_tools.rs`). Lacks wait, eval, hover, select, check, drag,
upload, forward and reload, emulation, dialogs for agents, cookies and storage, full-page
screenshots, PDF, and interception. A stale ref gets "take a new snapshot" with no recovery.
Tools that name no tab act on "the page whose tab the user focused last", in any project
(`page_of`), and the MCP server has no notion of which project a caller belongs to.

### 2.12 Console, network, HAR, interception, emulation

All through agent-browser: `console`, `network requests`, `network har start|stop` (the CLI's
`capture start|stop`), `network route` patterns for interception (continue or block per request
is a TODO), device, offline, headers, HTTP credentials, color scheme and reduced motion,
geolocation. Interception patterns are re-applied after an idle daemon is replaced. There is no
console or network panel of Orca's own; DevTools covers that. **Marley today:** keeps console and
network per tab with secrets hidden (`crates/marley_browser/src/observe.rs`) and the flight
recorder's last minute (`recorder.rs`); no HAR, interception or emulation.

### 2.13 Computer use

**What agents do.** `orca computer permissions|capabilities|list-apps|list-windows|get-app-state|click|set-value|type-text|press-key|hotkey|paste-text|scroll|drag|perform-secondary-action`
(14 commands), in a snapshot, act, snapshot loop on numbered elements
(`docs/site/content/docs/cli/computer-use.mdx`, `skill-guides/computer-use.md`).

**Underneath.**
- A Node sidecar (`src/main/computer/sidecar-entry.ts`) talks to one provider per platform: a
  Swift helper app on macOS ("Orca Computer Use", which holds its own Accessibility and Screen
  Recording grants) over a Unix socket (`native/computer-use-macos`, 6.9k lines); a
  PowerShell UI Automation script on Windows (`native/computer-use-windows/runtime.ps1`); a
  Python AT-SPI script on Linux that reads one JSON operation file and prints one JSON answer
  (`native/computer-use-linux/runtime.py`, 1.15k lines; driven by `desktop-script-*.ts`).
- Linux specifics: the tree comes from AT-SPI (at most 1,200 nodes, depth 64), written as
  tab-indented lines `index role name, Value: …, Secondary Actions: …` with empty containers
  elided and plain-text subtrees summarized. Clicks on elements use AT-SPI actions (click,
  press, activate) first. Coordinates and typing use `Atspi.generate_mouse_event` and
  `generate_keyboard_event`. Screenshots use the Gdk root window, and hotkeys and modified
  clicks use `xdotool`; the helper reports both unavailable when `XDG_SESSION_TYPE=wayland`.
  On Hyprland, then, it reads trees and triggers AT-SPI actions, with no pixels and no chords;
  synthetic pointer input likely reaches XWayland windows only.
- Safety: password managers blocked by name (1Password, Bitwarden, Dashlane, LastPass, NordPass,
  Proton Pass); nodes that look secure (password, PIN, one-time code, AT-SPI PROTECTED) are
  redacted; secrets go through stdin, though on Linux and Windows they still pass through a
  short-lived operation file; every action reports whether it was verified, unasserted or blind
  synthetic input, and the skill says never to report an unverified action as done; the skill
  forbids sending, submitting, buying or deleting unless asked. There is no per-action approval.

**Size.** 4.5k lines of TypeScript plus the three native helpers (about 9.4k lines).

**Marley today.** None. Chad's `dev-box-desktop` skill already drives Hyprland with `grim` and
`hyprctl`.

### 2.14 The emulator pane

**What the user sees.** A pane (or a floating tab) with a device frame, touch and keyboard input,
hardware buttons and rotation; one active device per worktree.

**Underneath.** iOS (macOS only): a bundled `serve-sim` helper streams MJPEG and drives private
simulator APIs; Orca copies it out of the app bundle and strips the quarantine attribute so its
injected dylib loads (`src/main/emulator/serve-sim-*.ts`, `backends/ios-emulator-backend.ts`).
Android: the SDK's `adb`, `emulator` and `avdmanager`; video from a pushed scrcpy 2.4 server (h264
over an adb-forwarded abstract socket) decoded in the renderer with WebCodecs' `VideoDecoder`;
input through `adb shell input`; the tree from `uiautomator dump`
(`src/main/emulator/android/*`, `src/renderer/src/components/emulator-pane/use-emulator-video-stream.ts`).
CLI: `orca emulator list|devices|attach|tap|gesture|type|button|rotate|ax|install|launch|permissions|logcat|exec|kill|shutdown`,
coordinates normalized to 0..1 (`skill-guides/orca-emulator.md`, `orca-emulator-android.md`).

**Size.** 4.8k lines in main, 4.5k in the renderer.

**Marley today.** None.

### 2.15 Recording, visual diffs, Playwright

None in the product. Playwright is Orca's own end-to-end runner, through Stably's
`@stablyai/playwright-test` wrapper launching Electron (`tests/playwright.config.ts`,
`tests/e2e/browser-*.spec.ts`). **Marley today:** has the flight recorder; #506 is queued.

### 2.16 HTML document preview

Local HTML and documents open in the pane under `orca-preview://` in a non-persistent session
whose Content-Security-Policy blocks all outbound requests. The comment's reasoning: previewed
documents are agent-written, so any request they can make is an exfiltration channel
(`src/main/browser/doc-preview-protocol.ts`). "Share as artifact" uploads a self-contained file
to Orca's service. **Marley today:** Zed's Markdown preview; no sandboxed HTML preview.

## 3. Bring to Marley

Ranked by value to the pick, fix and check loop and to running several projects at once.

1. **A fuller pick bundle: HTML, computed styles, framework component and source.** Not queued,
   but #505 needs it.
   - *Why:* `PickBundle` lacks the two things the agent most needs to change a style, and for
     React apps its source-mapped listeners point into react-dom.
   - *Lands:* `crates/marley_browser/src/pick.rs`: add outerHTML with scripts stripped and a
     4 KB cap, Orca's 16 computed properties, the selected text and a few sibling texts to the
     `DESCRIBE` pass; a second function reads `__reactFiber$…`, walks `.return` up to 35 levels
     for component names (Orca's skip list in `grab-guest-react-script.ts`), and takes the
     source from `_debugSource` (React 18 and older) or from the first non-React frame of the
     fiber's `_debugStack` (React 19), mapped through `source_map.rs`. Add Orca's redaction:
     secret-looking attribute values and URL query strings. `browser_pick` returns the new
     fields; the pick line stays short.
   - *Size:* S to M.
   - *Hard:* check the React 19 `_debugStack` frames against a real dev build. Vue
     (`__vueParentComponent.type.__file`) and Svelte (`__svelte_meta.loc`) are cheap extras.

2. **#505 pick, fix, check (queued).** Orca has only "click again", so the lessons are about
   what to capture and how to compare.
   - Record at pick time what the check compares: page box, the computed-style subset, text, a
     hash of the HTML.
   - Re-find in this order: test id, id, role and name from the accessibility tree, text, CSS
     path. Keep hashed classes out of every CSS locator (Orca skips `css-*` and 12+ character
     mixed-case hashes): CSS-in-JS class hashes change exactly when the agent edits styles. When
     several nodes match, take the one nearest the old box; when none does, say so instead of
     cropping something else.
   - Crop both times under the same conditions: inspect highlight off, same margin and scale,
     element scrolled into view or `captureBeyondViewport`. Orca's viewport-only crop fails for
     elements off screen; don't copy it.
   - Report "what changed" as text: `padding: 8px → 16px`, box moved or resized, text changed.
     That is what the ticket's tool should return next to the new crop.
   - Trigger from a Check button and from the tool. An automatic check after hot reload can
     key off the `[vite] hot updated` console line Marley already keeps.
   - *Size:* M.

3. **#507 a browser context per project (queued).** Orca teaches two things.
   - Logins live in more than cookies. Electron `persist:` partitions keep cookies,
     localStorage, IndexedDB and service workers. CDP's `Target.createBrowserContext` contexts
     are off the record: Marley could save and restore their cookies with
     `Storage.getCookies`/`Storage.setCookies` and a `browserContextId`, but localStorage and
     IndexedDB, where Supabase and Firebase keep their auth tokens, die with the context. With
     contexts alone, "a project's login survives a restart" fails for those apps. The
     alternative is one Chromium unit per project with its own `--user-data-dir` under Marley's
     data directory; `service.rs` already names units by a hash of the profile path, so a
     second profile is a second unit. Stop a project's unit when the project closes.
   - Key the storage on a durable project identity (the root path's hash, or an id Marley
     stores), never on anything per-boot; Orca's remote-host partitions lost cookies on every
     reconnect until it fixed exactly that.
   - Also include: a "clear this project's browser data" action; a decision on what worktree
     projects (#510) get (a copy of the parent project's profile at creation, or a clean one;
     Orca shares one default profile across worktrees and makes isolation opt-in); and item 4.
   - *Size:* M to L.
   - *Hard:* memory per Chromium (a few hundred MB each); moving the current single profile to
     the first project; Google sign-in (item 10).

4. **Scope agent browser tools to the caller's project.** Not queued; belongs with #507 and #510.
   - *Why:* Orca resolves the worktree from the CLI's cwd and defaults every command to that
     worktree's active tab. Marley's `page_of` takes the last-focused tab in any project, so an
     agent in project A can drive project B's page (and, after #507, B's logins).
   - *Lands:* the stdio bridge sends its cwd (or a `MARLEY_PROJECT` variable that Marley's
     terminals set) when the session opens; the hub maps it to a project; unnamed-tab tools use
     that project's last tab and new tabs open there.
   - *Size:* S to M.

5. **#503 localhost URLs open a Browser tab (queued).** Include:
   - Orca's routing: one setting for where links open, a one-click inverse modifier, and a small
     menu with both destinations, rather than a fixed rule.
   - Ports as well as printed URLs, found the way Orca does on Linux: listening sockets from
     `/proc/net/tcp` and `tcp6`, the socket inode mapped to a pid through `/proc/<pid>/fd`, and
     the pid's cwd matched to the deepest project root. That finds a dev server whose URL
     scrolled away or that was started outside Marley, and it is the same data worktree agents'
     port offsets (#510) will want.
   - Printed URLs parsed per terminal at line breaks, after stripping OSC and CSI, with cursor
     moves replaced by a guard character, trailing punctuation dropped, kept per (project, port),
     and dropped when the port stops listening; prefer the printed host name over the numeric
     bind.
   - 0.0.0.0 and `[::]` opened as 127.0.0.1 and `[::1]`.
   - The block terminal can offer a block's printed URL in its footer; the rail can list the
     project's live ports (feeds #504).
   - *Size:* M.

6. **Annotations tied to elements, sent as one batch.** Not queued.
   - *Why:* Orca's annotation is a pick plus a comment plus an intent. Marley's box and note
     leave the agent to work out which element is meant.
   - *Lands:* `browser.rs` annotations: when a box is drawn around one element (or a click lands
     on one), attach its pick bundle; add an intent (change or question); "Send all" types one
     reference line per annotation, in Marley's pick-line style, for the agent to read through
     `browser_annotations`, instead of pasting Orca's full HTML into the prompt.
   - Keep unsent annotations across navigation and restart; Orca's loss of them is a defect.
   - *Size:* S to M.

7. **Send picks and feedback only to an idle agent chosen in the rail.** Not queued; sits next
   to the approvals inbox (#508).
   - *Why:* `send_pick` pastes into the last-focused terminal whatever its agent is doing.
   - *Lands:* the rail highlights the project's agent rows that can take input and disables
     the ones waiting for permission, with the reason. Sending is a bracketed paste, a re-check
     of the agent's state, then Enter as a separate write; a failure after the paste is reported
     as "pasted, not submitted".
   - *Size:* S.

8. **#504 Browser tabs as rail rows (queued).** Orca has no sidebar rows for tabs (a palette and
   port rows only). Include: a favicon with Orca's rules from section 2.1 (headless Chromium
   gives no favicon event, so read `link[rel~=icon]` after load and fall back to
   `/favicon.ico`); a spinner while loading; host and port; counts of picks and annotations; a
   marker when an agent acted on the page, from Marley's own tool calls; and rows for live
   ports with no tab (from #503), whose click opens one.
   - *Size:* S to M.

9. **#506 a recording turned into a Playwright test (queued).** Nothing in Orca to copy.
   What carries over: the snapshot's ref model (role, name, occurrence) maps directly to
   `page.getByRole(role, { name, exact: true }).nth(n)`, and Orca's hashed-class filter keeps
   CSS fallbacks stable. Include:
   - compute each click or typing target's locators in the page at event time, before the DOM
     changes;
   - prefer `getByTestId`, then `getByRole`, `getByLabel` or `getByPlaceholder`, `getByText`,
     then `locator(css)`;
   - write password fields as `process.env.<NAME>`, never the typed value;
   - `await expect(page).toHaveURL(…)` after each navigation;
   - `baseURL` from the recording's origin so the test runs against the dev server.
   - *Size:* M.

10. **Drop "HeadlessChrome" from the user agent.** Not queued; check first.
    - *Why:* Marley's Chromium announces `HeadlessChrome/…`, which bot checks such as
      Cloudflare's and Google's sign-in commonly reject. Orca's identity work shows the UA and
      the `sec-ch-ua` client hints must agree, or the mismatch is itself flagged.
    - *Lands:* `--user-agent` in `service.rs`, or `Emulation.setUserAgentOverride` with
      `userAgentMetadata` per page.
    - *Size:* S. *Hard:* Google may still refuse sign-in in a CDP-driven browser.

11. **Fill the agent tool gaps.** Not queued. From Orca's CLI: `browser_wait` (text, URL,
    selector, network idle, JS condition), `browser_eval`, hover, select option, check, drag,
    upload, forward and reload, viewport and media emulation (dark mode, reduced motion),
    dialog accept and dismiss, full-page screenshot, cookies and storage for the project's
    context. From Orca's older snapshot engine: refs for clickable elements that have no ARIA
    role, a `(2nd)` suffix for duplicate names, and recovery of a re-rendered ref by role, name
    and occurrence (Marley answers "take a new snapshot", `browser_tools.rs`).
    - *Size:* S each, M together.

12. **Local HTTPS "proceed" for loopback hosts, pinned to the certificate.** Not queued. Offer it
    for loopback hosts only, as Orca does, and remember the leaf certificate's fingerprint so a
    different certificate on the same host asks again.
    - *Size:* S.

13. **Page chrome gaps.** Not queued: find in page, downloads, viewport presets (Orca's list
    matches Chrome DevTools'), zoom, DevTools. For DevTools, the debugging port should serve
    the frontend at `/devtools/inspector.html?ws=…`, which a Browser tab could show; check that
    Arch's Chromium ships it and that it is usable through the screencast.
    - *Size:* S each.

14. **Markup on a frozen frame.** Not queued, lower value: arrows, boxes, freehand and text over
    the current frame, sent to the agent as an image. Marley already holds the frame in gpui.
    - *Size:* S to M.

15. **Remote project pages through the project's SSH host.** Later, with `marley_remote`. Orca's
    pattern: a loopback SOCKS5 server whose dials go through the SSH connection, DNS resolved
    remotely, no fallback to direct connections, WebRTC restricted. Marley could give a remote
    project's context `proxyServer` (`Target.createBrowserContext`) or its unit
    `--proxy-server=socks5://127.0.0.1:<port>` fed by `ssh -D`.
    - *Size:* M to L.

## 4. Skip

- The `<webview>` embedding and its workarounds (anchored overlay layer, focus replay, rerouted
  screenshots): Marley draws a screencast and has none of those problems.
- Bundling agent-browser and `orca exec`: a per-tab daemon that needed idle timers, socket rules
  and an orphan sweep; Marley's tools already speak CDP.
- The full cookie importer (Safari binarycookies, Windows DPAPI and app-bound keys, staged SQLite
  images, CHIPS rollbacks): if Chad wants import, take only Linux Chromium (`secret-tool` v11,
  `peanuts` v10) into `Storage.setCookies`, and skip `google.com` as Orca does.
- Client-hosted versus server-streamed placement and the network tunnel (16.7k lines): Marley has
  one local Chromium.
- Browser identity modes: they exist to hide Electron, and Marley's Chromium needs only item 10.
  Keep the Firefox UA on Google's sign-in hosts in mind as the fallback if item 10 is not enough.
- The WebAuthn account picker: Electron-specific; headless Chromium has no security-key UI.
- The mobile driver overlay: Marley has no phone client.
- Computer use: the Linux provider assumes X11 for pixels and hotkeys, and Chad's desktop skills
  already cover Hyprland.
- The emulator pane: the iOS half needs macOS; the Android half (scrcpy, WebCodecs, adb) is a
  product of its own.
- Grab's copy to clipboard: Marley's reference line plus a tool read keeps the agent's context
  smaller than pasting 4 KB of HTML.
- HTML preview and artifact sharing: tied to Orca's account service.

## 5. Open questions for Chad

1. For #507: one Chromium per open project (logins stored in localStorage survive, a few hundred
   MB each), or one Chromium with contexts and a cookie jar Marley keeps (lighter, but those
   logins are lost at restart)?
2. Should a worktree project (#510) start with a copy of its parent project's browser profile,
   or clean?
3. Is cookie import from your own Chromium wanted? It depends on its keyring: a `secret-tool`
   entry for "Chromium Safe Storage", or `--password-store=basic`.
4. For #503: route only local URLs into Marley, as the ticket says, or every http(s) link, with
   a setting like Orca's?
5. Any iOS or Android work coming that would justify an emulator pane?
6. Computer use inside Marley, or leave it to your desktop skills?
