# B1a: The address bar and navigation — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-490-browser-navigation.md
- **Pipeline spec:** 490-browser-navigation.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3; the chrome a person needs to browse in the tab.
- **Classification:** feature; `marley_workbench` (the toolbar, the dialog card, the key
  context) and `marley_browser` (history, loading and dialog events). No Zed path.
- **Recall (§18.3):**
  - #406's lesson: a page that fails can deliver nothing; the loading state ends on the
    frame's stop event, and a connection that closes shows in the tab as #488 draws it.
  - #481: an `Editor` inside a container that filters keys (the rich input) is the model for
    the address bar inside a tab that forwards keys to the page.
- **Discovery:** the probe's CDP protocol dump for the Page domain.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-24)
- **Pre-flight:** #489 committed (9a87647c68); no other active pipeline; cargo idle.
- **Seams re-verified:** `Editor::single_line(window, cx)`, `set_text`, `select_all(&SelectAll,
  …)`, `text(cx)`, `focus_handle(cx)`; the rich input's keymap pattern (`"MarleyRichInput >
  Editor"` binding Enter and Escape to Marley actions, which beats the editor's own and the
  keymap's context-less `menu::Confirm` and `menu::Cancel`); `ui::IconButton` with a tooltip
  and `on_click` (the agent bar); `IconName::ArrowLeft`, `ArrowRight`, `RotateCw`, `Close`.
  Zed's Linux keymap binds `ctrl-r` (open recent) and `f5` (debugger) in `Workspace` and
  `ctrl-l` in `Editor`; a `MarleyBrowser` binding is deeper than `Workspace`, so it wins while
  the tab has the focus. The workspace depends on `url` (its `form_urlencoded` encodes the
  search).
- **Recall:** #489's key path forwards every key the tab's `key_down` sees; with an editor
  inside the tab, its keys bubble through that listener too, so the page must get keys only
  while the tab itself holds the focus.

### Design
- **`marley_browser::address`** (new, pure): `url_for(text) -> Option<String>`: trimmed;
  empty is `None`; a known scheme (`http`, `https`, `file`, `about`, `data`, `chrome`,
  `view-source`) as typed; a host (no spaces; `localhost`, an IPv4 address, a bracketed IPv6
  address, or a name with a dot), with its port and path, gets `http://` when the host is
  loopback and `https://` otherwise; anything else is
  `https://duckduckgo.com/?q=<form-encoded text>`.
- **`Page`** gains `navigate`, `reload`, `stop_loading`, `history` (`Page.getNavigationHistory`
  into `NavigationHistory { current_index, entries }`), `go_to_history_entry`, and
  `answer_dialog(accept, prompt_text)`.
- **The hub** follows `Page.frameStartedLoading` and `frameStoppedLoading` for the main frame
  (its id is the target's), `Page.frameNavigated` for the main frame and
  `navigatedWithinDocument` (each refreshes the history, which sets whether back and forward
  can go), and `Page.javascriptDialogOpening` and `javascriptDialogClosed`. It gains
  `navigate`, `back`, `forward`, `reload`, `stop` and `answer_dialog`, and emits
  `DialogOpened` so the tab can take the focus for it.
- **The tab** gets a toolbar: back, forward, and reload or stop as `IconButton`s, disabled when
  they cannot act, and the address bar, a single-line `Editor` inside a `MarleyAddressBar` key
  context; under it a thin accent bar while loading. The address bar shows the page's URL
  whenever it does not have the focus. `key_down` forwards keys only while the tab's own focus
  handle is focused. The JavaScript dialog is a card over the page: the message, a single-line
  editor with the default for `prompt`, OK and Cancel; Enter answers OK and Escape Cancel.
- **Actions** (in `marley_workbench`'s `actions!`): `FocusAddressBar`, `GoToAddress`,
  `RestoreAddress`, `BrowserBack`, `BrowserForward`, `BrowserReload`, `AnswerDialog`,
  `DismissDialog`. **Keymap:** `MarleyBrowser`: ctrl-l, alt-left, alt-right, ctrl-r, f5;
  `MarleyAddressBar > Editor`: enter, escape; `MarleyBrowserDialog`: enter, escape (and the
  same in `MarleyBrowserDialog > Editor`).
- **Manifest:** `crates/marley_browser/{Cargo.toml (url), src/address.rs, src/page.rs,
  src/marley_browser.rs}`; `crates/marley_workbench/{src/browser.rs, src/marley_workbench.rs,
  keymap.json}`; `script/e2e/browser-fixture.sh` (a server with a slow path);
  `script/e2e/490-browser-navigation.sh`. No Zed path.

### E2E plan
| REQ | Step (`compositor sway`) | Shot |
|---|---|---|
| REQ-001, 007 | Ctrl+L in the tab, type page A's URL, Enter | `490-01-typed-url` |
| REQ-003 | click page A's link to B | `490-02-link` |
| REQ-003 | Alt+Left | `490-03-back` |
| REQ-002, 003, 004 | the forward button, then Ctrl+L, `127.0.0.1:<port>/b.html` with no scheme, Enter, then Ctrl+R (B counts its loads) | `490-04-forward-reload` |
| REQ-005 | Ctrl+L, the slow path, Enter; shot while it loads; the stop button | `490-05-loading` |
| REQ-002, 007 | Ctrl+L, "marley browser test", Enter; Escape afterwards restores the URL | `490-06-search` |
| REQ-006 | the dialogs page: alert → OK with Enter; prompt → type, Enter | `490-07-alert`, `490-08-prompt-answered` |
| REQ-008 | the stand-in agent navigates to page A | `490-09-agent-navigated` |

The search result page needs the network; the shot proves the address bar's URL, whatever the
page shows.

### Risks
- A `MarleyBrowser` binding takes its key from the page too (Ctrl+R, F5, Alt+arrows): the
  browser's own meaning, as in any browser.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:**
  - `marley_browser::address::url_for`: the three rules (a known scheme as typed; a host with
    its port and path over `http` for loopback, `https` otherwise; else a search at
    `duckduckgo.com`). IPv4 and bracketed IPv6 go through `std::net`; a name needs a dot, labels
    of letters, digits and inner hyphens, and a last label that is not a number (`3.14` is a
    search). `url` joins the crate for `form_urlencoded`.
  - `Page`: `navigate` (Chromium's `errorText` becomes a `CdpError::Protocol`), `reload`,
    `stop_loading`, `history` into `NavigationHistory` (`entry_at(offset)`),
    `go_to_history_entry`, `answer_dialog`; the types `JavaScriptDialog` (with `origin()`, the
    host and port a card names as the asker) and `DialogKind`.
  - The hub: `pending_url`, `loading`, `history`, `dialog`; `address()`, `can_go`, `navigate`,
    `go`, `reload`, `stop`, `answer_dialog`; the events it follows, in `follow_navigation`
    (main-frame `frameNavigated`, `navigatedWithinDocument`, `frameStartedLoading` and
    `frameStoppedLoading`, `javascriptDialogOpening` and `javascriptDialogClosed`); the history
    read at attach. `BrowserEvent` (`PageInfoChanged`, `DialogOpened`, `DialogClosed`) replaces
    the `PageInfoChanged` struct.
  - The tab: the toolbar (back, forward, reload or stop as `IconButton`s with tooltips; the
    address bar in `MarleyAddressBar`), the loading bar, the dialog card, the focus rules, the
    eight actions, and `key_down` forwarding only while the page itself has the focus.
  - `keymap.json`: `MarleyBrowser` (ctrl-l, alt-left, alt-right, ctrl-r, f5),
    `MarleyAddressBar > Editor` (enter, escape), `MarleyBrowserDialog` and
    `MarleyBrowserDialog > Editor` (enter, escape).
  - `script/e2e/browser-fixture.sh`: `serve_site` runs a small Python server (`serve.py`) with a
    `/slow` path (three seconds; checked by hand: 200 after 3.0 s); `offline_chromium`.
- **Deviations from the design, and why:**
  - The address bar shows where a navigation the tab asked for goes (`pending_url`) until it
    commits or ends, as Chrome's omnibox does; without it, Enter would flash the old URL for
    the whole of a slow load.
  - `navigate`, `go` and `reload` answer an open dialog with Cancel first: Chrome closes a
    page's dialog when the page is left, and a page blocked in a dialog may not take a
    navigation.
  - The loading bar is drawn over the toolbar's lower edge, absolutely, so a load never
    changes the page's viewport (a row that came and went would resize the page twice a load).
  - The dialog card is Zed's `ui::AlertModal` (key context, focus, actions, custom footer), over
    an occluding layer so the page takes no mouse input while it waits. The card names its
    asker (`127.0.0.1:8000 says`), as Chrome does; `beforeunload` reads "Leave this page?" with
    Leave and Cancel.
  - A dialog takes the focus only from inside the tab; when the tab takes the focus later, the
    page's focus is handed to the dialog.
  - `offline_chromium` (a wrapper that starts Chromium with
    `--host-resolver-rules="MAP * ~NOTFOUND, EXCLUDE localhost"`): the plan let the search step
    reach the network; now the search URL fails in the page and nothing leaves the machine.
  - Stop has a button and no action or key: the spec names none.
- **Review of the diff:**
  - Re-entrancy: the view's handlers read the hub and update only the editors; the hub's emits
    are delivered after its update ends. `show_address` runs from focus and hub events, never
    inside the editor's own update.
  - Keys: the page gets keys only while its own focus handle is focused (`is_focused`), and its
    input handler is registered only then (gpui's `handle_input`), so the address bar and the
    prompt field keep their keys and text.
  - Logging: no URL is logged; a failed navigation logs Chromium's error text only.
  - Security: a typed `javascript:` URL is not a known scheme and becomes a search.
  - Found: the toolbar moves the page down, so #489's scenario's `PAGE_Y` is stale; the Test
    phase measures the new origin and updates it. Open: whether `Page.navigate` answers at the
    commit (the loading shot shows it: an early answer would clear the pending URL at once).
- **Checks:** `cargo check`, `cargo fmt`, and `cargo clippy -p marley_browser -p
  marley_workbench --all-targets -- -D warnings` clean (clippy's `useless_let_if_seq`,
  `too_many_lines`, `trivially_copy_pass_by_ref`, `needless_pass_by_ref_mut`, `doc_markdown` and
  `too_long_first_doc_paragraph` fixed at the source). No Zed path touched; `Cargo.lock`'s row
  covers the new `url` entry.
- **Checklist (no TaskCreate in this harness):** address ✓, page ✓, hub ✓, view ✓, actions ✓,
  keymap ✓, fixture ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/490-browser-navigation.sh` (`compositor sway`, 1600x1000). The
  fixture site (`serve_site`, with `/slow`) serves A, B and the dialogs page; `offline_chromium`
  starts Chromium with host-resolver rules, so the search fails in the page and nothing leaves
  the machine. Shots beyond the plan's nine: `490-00-address-selected`, `490-03b-forward`,
  `490-04a-host`, `490-05b-stopped`, `490-06b-restored`, `490-08a-prompt`.
- **First run: red.** The resolver rule `MAP * ~NOTFOUND, EXCLUDE localhost` mapped the IP
  literal 127.0.0.1 too, and every page failed with `ERR_NAME_NOT_RESOLVED`. Fixed in the
  fixture (`EXCLUDE 127.0.0.1`). That run showed the address bar keeping the typed URL
  over Chromium's error page, since the `unreachableUrl` change had gone in before the build.
- **Second run: every shot read.**
  - `490-00-address-selected` (REQ-007): Ctrl+L put the focus in the address bar with
    `about:blank` selected; back and forward disabled at the start of the history, reload
    enabled.
  - `490-01-typed-url` (REQ-001): page A; the address bar `http://127.0.0.1:36261/a.html`; the
    tab "Page A"; back enabled, forward disabled.
  - `490-02-link` (REQ-003, REQ-008): the link's click reached B ("Page B, load 1: navigate");
    the address bar followed the page's own navigation.
  - `490-03-back` (REQ-003): Alt+Left went back to A; forward enabled.
  - `490-03b-forward` (REQ-003): the forward button went to B, restored from the back-forward
    cache ("load 1: back-forward cache"); the button's tooltip reads "Forward Alt-Right";
    forward disabled again at the end of the history.
  - `490-04a-host` (REQ-002): `127.0.0.1:PORT/b.html`, typed with no scheme, loaded as
    `http://127.0.0.1:36261/b.html` ("load 2: navigate").
  - `490-04-forward-reload` (REQ-004): after Ctrl+R and F5, "Page B, load 4: reload".
  - `490-06b-restored` (REQ-007): after `before`, Ctrl+L, `not a destination` and Escape, the
    address bar shows B's URL again and the page took ` after`: its field reads
    "before after".
  - `490-05-loading` (REQ-005): the address bar shows `/slow` while it loads, reload is a stop
    button, and a 2-pixel accent bar runs under the toolbar (rows 106 and 107 are `#74ADE8`;
    the page still starts at row 109, so the bar did not move it). `Page.navigate` does answer at
    the commit: the pending URL held for the whole wait.
  - `490-05b-stopped` (REQ-005): the stop button ended the load; reload is back, the bar gone
    (rows 106 and 107 are the toolbar's gray again), the address bar shows B's URL, and the log
    has `Page.navigate failed: net::ERR_ABORTED`, with no URL.
  - `490-06-search` (REQ-002): the address bar shows
    `https://duckduckgo.com/?q=marley+browser+test`, the tab `duckduckgo.com`, over Chromium's
    error page (offline, by design).
  - `490-07-alert` (REQ-006): the card "127.0.0.1:36261 says / Hello from the page." with OK,
    centered over the page.
  - `490-08a-prompt` (REQ-006): the prompt's card with its field, "the default" selected, and
    Cancel and OK; the page's log shows `alert: answered` and `confirm: false` (Enter answered
    the alert, Escape the confirm).
  - `490-08-prompt-answered` (REQ-006): `prompt: typed in Marley`.
  - `490-09-agent-navigated` (REQ-008): the stand-in agent's navigation shows in the address bar
    (`/a.html`) and the tab ("Page A").
- **Regressions:** the toolbar moves the page 42 pixels down, so `489-browser-input.sh`'s
  `PAGE_Y` is now 109. Its rerun: typed and edited text, the composed é, the single, double and
  right clicks, the iframe's field, paste and copy (`the clipboard after Ctrl+C: copy`), the
  Super chord filtered, `scrollY 500` after five detents; latency 37 inputs, median 22.5 ms, 95th
  percentile 75.7 ms (the median is #489's; the tail moved with the box's load and one run).
  `488-no-chromium.sh`: the failure is centered under a toolbar whose buttons are disabled and
  whose address bar shows its placeholder.
- **Focus report:** every run was in a headless sway; the Hyprland check printed "0 Marley
  windows before the run, 0 after; the run added no rule and did not reload it".
- **Pre-existing, not in scope:** the log's `Received keymap format NoKeymap` lines (each
  `wtype` brings a keymap), the Vulkan loader messages, and the llama.cpp provider's 404.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  REQ-005 ✓, REQ-006 ✓, REQ-007 ✓, REQ-008 ✓, regressions ✓, gate (below).
- **Gate:** `just gate-diff`, scope the nine Marley crates, touched `marley_browser` and
  `marley_workbench`: gate:1 rustfmt, gate:2 clippy (every target), gate:7 cargo-audit, gate:8
  cargo-deny, gate:9 cargo-shear, gate:10 gitleaks, gate:11 shellcheck, gate:12
  no-suppressions, gate:13 source-bans, gate:14 docs, gate:16 zed-ledger, gate:17 manifests,
  gate:18 typos, gate:20 semgrep, gate:21 dylint, and the receipt: 16 passed, 0 failed,
  `GATE GREEN [diff]`; the receipt matches the tree.
- **Verdict:** every criterion has its shot, read; the regressions hold; the gate is green.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (Browsing in the Browser tab); `marley_browser.md` (navigation
  and dialogs: the address rules, `Page`'s calls, `JavaScriptDialog`, the error page's URL);
  `marley_workbench.md` (the Browser tab's section, now #488 to #490: navigation and dialogs);
  `three-prong-plan.md` (B1a shipped). No Zed path was touched; `Cargo.lock`'s row covers the
  new `url` entry.
- **Knowledge appended:**
  `F-claude-490-the-offline-resolver-rule-mapped-the-loopback-address-001`,
  `L-claude-490-page-navigate-answers-at-the-commit-001`,
  `L-claude-490-a-failed-load-commits-chromiums-error-page-001`,
  `L-claude-490-a-view-that-forwards-keys-sees-its-editors-keys-001`,
  `AD-claude-490-the-browser-tab-draws-its-own-chrome-and-dialogs-001`. The resolver rule's
  failure gets no prevention rule: the lesson is the fixture's own comment.
- **Brain:** consultation `0e16015f0fba4851a52797130d60fb18`, asked at Complete (Phase 1 had
  not asked; nothing on the seam came back), closed by
  `decisions/marleys-browser-tab-draws-its-own-navigation-chrome-and-javascript-dialogs`,
  follow-up by 2026-10-24.
- **Checklist (no TaskCreate in this harness):** document ✓, capture knowledge ✓, close the
  ticket ✓, archive ✓, commit (below).
