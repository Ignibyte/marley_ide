# A project's local HTML pages open in its Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-586-file-pages-in-the-browser-tab.md
- **Pipeline spec:** 586-file-pages-in-the-browser-tab.spec.md

## Phase 1 — Plan
- **Request:** #561's open question on `file://` pages from the project, answered by Chad on
  2026-09-28: "561 - marley browser". Filed as TICKET-586.
- **Classification / tier:** feature, S. Marley crates only (`marley_browser`,
  `marley_workbench`, `marley_mcp`'s registry text); scenarios 561 (updated) and 586 (new).
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (cargo idle); recall ✓; mint ✓; discovery
  (Explore, over `cec9958e4f`) ✓; the prior-art sweep (Orca's MIT source; a `cargo doc --open`
  run with a logging `BROWSER`) ✓; spec and design ✓.
- **Recall (§18.3):**
  - AD-claude-561-marley-exports-its-opener-as-browser-in-every-local-terminal-001: the opener
    speaks MCP with the bearer, `browser_open_url` opens Browser tabs only, by #503's rule, in the
    project holding the program's folder; "`file://` pages in a Browser tab (plan D15, left to
    Chad's open question)" was rejected pending Chad.
  - Plan D15 (`docs/marley/three-prong-plan.md:301-312`): "agent navigation takes only `http`
    and `https`", with the client's approval of each write call as a check an agent cannot skip.
  - #579: a `file://` link in a terminal keeps Zed's Ctrl+click (the editor).
  - The brain (consultation 54a29a659ec8422ba48be4cf3f4c50fe): nothing on this seam.
- **Discovery (the Explore report, 2026-09-28, over `cec9958e4f`):**
  - The opener, `crates/marley_workbench/bin/marley-open-url` (written to
    `<data_dir>/mcp/marley-open-url` at start, `mcp.rs:315-362`): no scheme filter; `tools/call`
    `browser_open_url {url, directory: getcwd()}`; `opened: true` or it execs `xdg-open` with
    `BROWSER` removed; five seconds in all.
  - `browser_tools::open_url` (`browser_tools.rs:319-370`): `address::agent_url` (337, the
    decline "not an http or https URL"), `links::browser_tab_url` (340, #503's `destination`,
    http and https only, reading `terminal_links`), `holding(directory)` (343; 183-200, the
    deepest local root), `window_of` (346), then `multi_workspace.activate` and `open_url_tab`
    (`browser.rs:8210-8251`: a tab on the same URL reused, else a new view in the active pane
    with the focus, `open_page_in`, `Page.navigate`).
  - `browser_open_url` is a `browser_write` row (`registry.rs:262-269`; schema 460-483, "The http
    or https URL the program opens.") listed to `Principal::Marley`; outside clients are refused.
  - No scheme check past `agent_url`; `chromium_args` has no file-access flags; #507 runs a
    Chromium per project, no browser contexts.
  - `browser_snapshot` reads page text unredacted; `browser_look` gives a screenshot.
  - Scenario 561's REQ-004 (lines 159-167) checks that `file://$E2E_WORK/doc.html` reaches the
    fake `xdg-open` with no new tab.
- **Observed** (2026-09-28): `cargo doc --open` with a logging `BROWSER` passed one argument, the
  absolute path `…/target/doc/cdoc/index.html`.
- **Decisions:** D1 to D5 in the spec.

### Design
- **Approach.**
  - *The reader (`crates/marley_browser/src/address.rs`, pure).* `pub fn local_page(text: &str,
    directory: &Path) -> Option<PathBuf>`: a `file://` URL through `Url::to_file_path`, else a
    path (absolute, or joined to `directory`); `Some` only for a name ending in `.html` or `.htm`
    (either case). The existence check is the caller's.
  - *The tool (`crates/marley_workbench/src/browser_tools.rs`, `open_url`).* Before `agent_url`:
    when `local_page` gives a path, the file's metadata is read off the main thread (a regular
    file, not a directory); declined with "not a local HTML page" otherwise, and with #503's
    reason while `terminal_links` is `system_browser`; then `holding`, `window_of` and
    `open_url_tab` with `Url::from_file_path`, as a local URL goes. Every other text keeps today's
    path through `agent_url` and `browser_tab_url`.
  - *The text (`crates/marley_mcp/src/registry.rs`).* The tool's description and its `url`
    field: an http or https URL on this machine, or a local HTML page, as a `file://` URL or a
    path.
- **File manifest.** Marley crates: `crates/marley_browser/src/address.rs`,
  `crates/marley_workbench/src/browser_tools.rs`, `crates/marley_mcp/src/registry.rs`. Test:
  `script/e2e/586-file-pages-in-the-browser-tab.sh` (new), `script/e2e/561-browser-env-opener.sh`
  (its `file://` step), `script/e2e/golden`. No Zed path, no ledger row.
- **Knowledge at Complete (expected).** An AD for local pages through the opener only (D1, D2);
  a lesson on `cargo doc --open` handing `BROWSER` a path.

### E2E plan
Scenario 586, `compositor sway`, sourcing `browser-fixture.sh` as 561 does: the HOME and fakes of
561 (`xdg-open`, `user-browser`, `gio`, `google-chrome`, `firefox` logging their arguments; the
check that the fake `xdg-open` leads the PATH), `offline_chromium`, `git init repo`,
`open_path repo`. Fixtures in `$E2E_WORK`: `pages/index.html` ("A local page", a link "Next" to
`other.html`), `pages/other.html` ("Another page"), `notes.txt`, `docs/`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the project's first terminal: `python3 -c 'import webbrowser; webbrowser.open("file://$E2E_WORK/pages/index.html")'` | `586-01-file-url`; `tabs`: one tab on the page, project `repo`, focused |
| REQ-002 | `sh -c '"$BROWSER" "$1"' opener "$E2E_WORK/pages/other.html"`, a program that hands its opener a path | `586-02-path`; `tabs` |
| REQ-003 | back on `index.html`'s tab, a click on "Next" | `586-03-link`: "Another page" |
| REQ-004 | `webbrowser.open` of `notes.txt`, `docs/` and `missing.html` | `586-04-declined`: no new tab; `xdg-open.log` holds the three |
| REQ-005 | `mcp_agent navigate file://…/pages/index.html`; `mcp_agent open-url file://…/notes.txt $E2E_WORK/repo` | `586-05-agent`; the answers: navigate refused ("http and https URLs only"), open-url `opened: false` |
| REQ-006 | `terminal_links: system_browser`, a new terminal, `webbrowser.open` of the page | `586-06-system`: no tab; `xdg-open.log` |
| REQ-007 | 561 with its `file://` step on `doc.txt`; the golden set; the gate | the logs |

Not reachable by a scenario: a real `cargo doc --open` (the scenario's HOME has no toolchain;
the observation above stands, and the path form is what REQ-002 drives).

### Risks
- An agent can open a local HTML page through `browser_open_url` and read it with
  `browser_snapshot`: bounded to `.html` and `.htm` files, which an agent could read with its own
  tools, and Claude Code asks before each call by default. A page's links to other local pages
  load (D5), so a page that links to a directory shows its listing; a rustdoc or coverage page
  does not.
- A page path through a symlink resolves to the file Chromium reads; the check reads the
  metadata of the path as given, following links.

## Phase 2 — Code
- **Checklist** (no task tool): the README marker present ✓; no Zed path ✓; `address.rs` ✓;
  `links.rs` ✓; `browser_tools.rs` ✓; `registry.rs` ✓; `just clippy marley_browser marley_mcp
  marley_workbench` ✓; `cargo fmt --check` ✓; the review ✓.
- **What was built.**
  - `crates/marley_browser/src/address.rs`: `local_page(text, directory)`: a `file:` URL through
    `Url::to_file_path`, a URL of any other scheme none, else the text as a path joined to the
    program's folder (an absolute path replaces it); `Some` only for a name ending in `.html` or
    `.htm`, either case.
  - `crates/marley_workbench/src/links.rs`: `pages_in_browser_tab(cx)`, true unless
    `terminal_links` is `system_browser`.
  - `crates/marley_workbench/src/browser_tools.rs`: `open_url` is async, run from `run` before the
    browser is up as before. A text `local_page` reads is checked with Zed's `Fs::metadata` off the
    main thread (a file, neither a folder nor a FIFO) and turned into a URL with
    `Url::from_file_path`; "not a local HTML page" otherwise, and "marley.terminal_links sends
    pages to the system browser" under `system_browser`. Any other text takes #561's path
    (`agent_url`, `browser_tab_url`) unchanged. Both end in `open_in_tab`, #561's tail (the
    project holding the folder, its window, the workspace shown, `open_url_tab`), with the
    answers from two small functions, `open_answer` and `declined`.
  - `crates/marley_mcp/src/registry.rs`: the tool's description and its `url` field name local
    HTML pages.
  - Scenario 561's `file://` step moves to `doc.txt`, a file that is no page; scenario 586 is new
    (written for Test).
- **Deviations from the design**: none.
- **The review**, against D2: `browser_navigate` and every other agent tool still go through
  `agent_url` (http and https); `browser_open_url` opens a local file only when its name is an
  HTML page's and `Fs` finds a file there, so a folder's listing, a FIFO and any other file are
  declined, and the opener hands them to `xdg-open` as before. A relative path joins the folder
  the opener sent; `from_file_path` refuses a relative one. The page loads with Chromium's usual
  file rules (no file-access flags), and a page's own links to other pages load (D5).
- **Clippy**: run 1, `open_url`'s `cx.update(links::pages_in_browser_tab)` did not fit
  `AsyncApp::update`'s `&mut App` (a closure now); run 2, `needless_pass_by_ref_mut` on `open_url`
  (`&AsyncApp`); run 3 clean; `cargo fmt --check` clean.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan ✓; 586 in the golden set, 561's step
  moved ✓; `just build` ✓; the scenario run and every shot read ✓; 561 alone ✓; the golden set ✓;
  `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/586-file-pages-in-the-browser-tab.sh` (compositor sway, on 561's
  fixture): the scenario's HOME with the opener as `BROWSER` and the fake `xdg-open` in its place
  otherwise; fakes for `xdg-open` (its log) and `gio`, `google-chrome`, `firefox` (the leak log,
  which stays empty); `offline_chromium`; `pages/index.html` ("A local page", a "Next" link to
  `other.html`), `pages/other.html` ("Another page"), `notes.txt`, a folder `docs/`; `repo` opened.
- **The runs.** Run 1 passed REQ-001 and REQ-002 and missed the link, whose point was a guess
  (`586-01`'s shot put "Next" at 304, 219). Run 2, with the measured point, passed every check.
- **The shots** (run 2):
  - `586-01-file-url` (REQ-001): after Python's `webbrowser.open("file://…/pages/index.html")`, a
    Browser tab "A local page" in front, its address bar on the `file:///…/pages/index.html` URL,
    and its row under `repo` in the rail; the stand-in's `tabs`: `'A local page' at
    file://…/pages/index.html, project repo, focused`.
  - `586-02-path` (REQ-002): after `sh -c '"$BROWSER" "$1"' opener …/pages/other.html`, a second
    tab "Another page" in front; `tabs`: `'Another page' at file://…/pages/other.html, project
    repo, focused`.
  - `586-03-link` (REQ-003): the first page brought forward again by the opener (the tab on its
    URL reused), a click on "Next": that tab shows "Another page"; `tabs`: both tabs on
    `other.html`.
  - `586-04-declined` (REQ-004): the terminal after `notes.txt`, `docs/` and `missing.html`: still
    two Browser tabs; `xdg-open.log` holds the three `file://` URLs.
  - `586-05-agent` (REQ-005): the stand-in agent's `browser_navigate` to the page: `refused: …
    an agent may open http and https URLs only, not file:`; its `browser_open_url` of
    `notes.txt`: `{"opened": false, "reason": "not an http or https URL"}`.
  - `586-06-system` (REQ-006): `terminal_links` on `system_browser` and a new terminal, which has
    no opener: the page reached `xdg-open`, no new tab; the tool itself answered `{"opened":
    false, "reason": "marley.terminal_links sends pages to the system browser"}`; the leak log
    empty.
- **561 alone**: its 15 checks passed, the `file://` step on `doc.txt` reaching `xdg-open` with no
  tab.
- **Focus**: both scenarios ran in their own headless sway; `hyprland: 0 Marley windows before the
  run, 0 after`.
- **Not reachable by a scenario**: a real `cargo doc --open` inside the scenario (its HOME has no
  toolchain); `cargo doc --open` handing `BROWSER` a path was observed on this box (Phase 1), and
  REQ-002 drives the same form.
- **The golden set** with 586 added: 51 of 51 (586 in 87 s).
- **The gate**: `script/gates.sh --diff`: `GATE GREEN [diff]`, 15 gates PASS; the log is
  scratchpad `586/gate.log`.
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented**: `CHANGELOG.md` (Added: a project's local HTML pages open in its Browser tab);
  `docs/marley_architecture/marley_workbench.md` (the opener's `open_url`, async, and its page
  branch); `docs/marley_architecture/marley_browser.md` (`address::local_page`);
  `docs/marley_architecture/marley_mcp.md` (`browser_open_url`'s `url`). No Zed path changed.
  The guide has no section on #561's opener yet; the documentation pass after the queue writes
  it with this.
- **Knowledge**: AD-claude-586-local-html-pages-open-through-the-opener-only-001;
  L-claude-586-cargo-doc-open-hands-browser-a-path-001. No F-block: nothing broke.
- **Brain**: consultation 54a29a659ec8422ba48be4cf3f4c50fe closed with
  `decisions/a-projects-local-html-pages-open-through-the-opener-only`.
- **Closed**: TICKET-586 moved to `tickets/closed/`; its BACKLOG row went at promotion.
