# File choosers, downloads and Take Over in the Browser tab — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-767-file-choosers-downloads-and-take-over.md
- **Pipeline spec:** 767-file-choosers-downloads-and-take-over.spec.md

## Phase 1 — Plan (2026-10-11)
- **Request:** the third slice of the T3 Code survey's item 10, drafted in #756's notes ("Slice
  3") and split out as #767. Chad, 2026-10-11: "18.) build now".
- **Classification:** feature, prong 3. Marley crates only; no Zed path.
- **Recall (§18.3):**
  - #495: whose press opened something (`USER_PRESS`, one second, `pressed_at`); an agent's click
    keeps Marley's select list shut. The chooser follows the same rule.
  - #525: the terminal's take-over (`terminal_drive.rs`), F-claude-525 on the bar reading a
    dropped last write; the Browser tab copies the words and the refusal.
  - #756: `browser_upload`'s checks (canonical path, open projects, secret globs) and `acting`.
  - Brain: consultation f61ab97826fe4ca98ce319fda9ecbe1a, nothing on this seam.
- **Discovery (an Explore pass, 2026-10-11):** the spec's prior art cites it. Notable:
  - Each project's Chromium is its own process (#507); no browser contexts are used, so the
    default context's download behavior is per project.
  - The relay sends a session-less command on Marley's browser session, so its events come back
    with no `sessionId` and pass; `follow_observed`'s `_` arm drops them now.
  - A chooser's event arrives on the page's or an iframe's session; `page_of_event` maps both to
    the page, so the chooser keeps the session it came on for `DOM.setFileInputFiles`.
  - No `ctrl-i` is bound in `MarleyBrowser`; Marley never calls `open_with_system` or
    `reveal_path`; `marley_workbench` has no `dirs` dependency.

### Design
- **`marley_browser/src/page.rs`:** `attach` sends `Page.setInterceptFileChooserDialog {enabled:
  true}` after `Page.enable`; `pub async fn allow_downloads(connection, folder)` sends
  `Browser.setDownloadBehavior {behavior: "allowAndName", downloadPath, eventsEnabled: true}` with
  no session.
- **`browser.rs`:**
  - `PageState` gains `chooser: Option<Chooser { session, backend_node_id, multiple }>`,
    `taken_over: bool`, `downloads: Vec<Download>` (the newest 20).
  - The hub: `download_folder()`; at `start`, `allow_downloads`; `file_chooser_opened` (emits
    `BrowserEvent::ChooserOpened`), `answer_chooser(target, paths)`, `cancel_chooser`;
    `download_began` and `download_progress` (by `frameId` to the page; on `completed`, the rename
    off the main thread, then an app notification with Open and Show in Folder); `take_over`,
    `hand_back`, `is_taken`, `agent_acted`.
  - `follow` gains the session-less `Browser.downloadWillBegin` and `Browser.downloadProgress`;
    `follow_navigation` gains `Page.fileChooserOpened`.
  - `BrowserView`: on `ChooserOpened` for its page, the path prompt when its press is within
    `USER_PRESS`; a chooser bar (Choose…, Cancel) and a control bar (Take Over, or You have
    control and Hand Back) under the toolbar, beside the pause bar; the `TakeOverBrowser` action.
- **`browser_tools.rs`:** for `WRITES`, a `taken_over` refusal before `not_paused`; `not_paused`
  also refuses a taken tab (for `browser_navigate` and `browser_check_pick`); `browser_tabs` and
  `browser_look` add `file_chooser` and `downloads`.
- **`browser_actions.rs`:** `upload` with no `ref` answers the open chooser (`no_chooser` when
  none).
- **`marley_workbench.rs`:** `TakeOverBrowser` in the `actions!`; **`keymap.json`:** `ctrl-i` in
  `MarleyBrowser`; **`Cargo.toml`:** `dirs`.
- **File manifest:** `marley_browser` (`page.rs`); `marley_workbench` (`browser.rs`,
  `browser_tools.rs`, `browser_actions.rs`, `marley_workbench.rs`, `keymap.json`, `Cargo.toml`);
  the guides; two architecture notes; the scenario. No Zed path.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Clicks the page's Choose Files; types the path of `notes.txt`; Enter | 767-01-person-prompt, 767-02-person-chosen |
| 002 | `browser_click` on Choose Files; `browser_upload` with no `ref` | 767-03-agent-chooser, 767-04-agent-upload |
| 003 | `browser_click` on the download link; lists the run's Downloads folder | 767-05-download |
| 004 | Clicks the page's empty area, Ctrl-I; `browser_click` refused | 767-06-taken |
| 005 | Clicks Hand Back; `browser_click` works | 767-07-handed-back |
| 006 | — | The review |
| 007 | — | The gate |

### Risks
- **Ctrl-I with nothing taken:** the action propagates, as the terminal's does, so the page's own
  Ctrl-I (italics in an editor) still works; the review checks the key reaches `key_down`.
- **Another relay client** (Playwright over CDP) may set its own download behavior on the same
  browser; Marley's is set again at each start.
- **A download from a subframe:** its `frameId` is not a page's; it is matched to the page whose
  iframes hold that frame, else to the focused tab.

## Phase 2 — Code (2026-10-11)
- **Built:** `marley_browser/page.rs`: the chooser interception in `attach`, `allow_downloads`.
  `browser.rs`: `Chooser`, `Download`, `download_folder`, `unique_in`, the page state (`chooser`,
  `taken_over`, `last_agent`, `downloads`), the hub's chooser, download and control methods,
  `allow_downloads` at start, `follow_download`, the `Page.fileChooserOpened` arm, the view's
  `choose_files`, `chooser_bar`, `control_bar`, `take_over` and `render_message` (moved out of
  `render` for length). `browser_tools.rs`: `TAKEN_OVER`, the `taken_over` refusal, `page_files`
  in `browser_tabs` and `browser_look`. `browser_actions.rs`: `answer_chooser` for `upload`
  with no `ref`. `registry.rs`: `upload` needs only `paths`. `marley_workbench.rs`:
  `TakeOverBrowser`; `keymap.json`: Ctrl-I; `Cargo.toml`: `dirs`.
- **Deviations from the design:** `last_agent` (from Test, F-claude-767-…) instead of the Agent
  chip's record; the chooser bar shows while the person's path prompt is open too (one state,
  cleared by the answer or the cancel).
- **The review:** the download name is cut to its file name, so a page cannot write outside the
  folder; the rename and the folder's making run off the main thread; the chooser's session is
  the event's, so an iframe's input is filled in its own session; `choose_files` updates the
  workspace from the view's listener, after the hub's update has ended.
- **Clippy found and fixed:** `maybe_infinite_iter` (`unique_in` bounded), `too_many_lines`
  (three: `start`, `follow`, `render`), `needless_pass_by_ref_mut`; a private `DirectoryLister`
  re-export. `cargo fmt`, then clippy again, clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/767-file-choosers-downloads-and-take-over.sh`, under `compositor
  sway`, exit 0, every check passing. Shots read:
  - `767-01-person-prompt`: Zed's path prompt on the repository's folder (`.git`, `more.txt`,
    `notes.txt`), the chooser bar behind it.
  - `767-02-person-chosen`: "notes.txt" in the input and "Files: notes.txt"; the control bar
    ("An agent acted in this page", Take Over) from the agent's navigate.
  - `767-03-agent-chooser`: the agent's click on Choose File: the bar "The page asks for a file"
    with Cancel and Choose…, the control bar under it.
  - `767-04-agent-upload`: "Files: more.txt", the chooser bar gone; `no_chooser` refused after.
  - `767-05-download`: the notification "Downloaded report.csv" with Open and Show in Folder;
    `report.csv` (19 bytes) in the run's Downloads folder.
  - `767-06-taken`: "You have control: agents' actions in this page are refused", Hand Back; the
    agent's click refused `taken_over`.
  - `767-07-handed-back`: "An agent acted in this page", Take Over; the agent's click worked
    ("Pings: 1").
- **Fixes on the way:** F-claude-767-… (`last_agent`); Choose File recalibrated below the control
  bar (L-claude-767-…).
- **What no scenario shows:** Ctrl-I reaching the page when no agent acted (REQ-006): the action
  propagates as the terminal's does; checked in review.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #767); the guide (a new "Files, downloads and taking
  over", the limits, the `browser_upload` row) and the in-app guide's Chromium card and tool row;
  `marley_browser.md` and `marley_workbench.md`.
- **Knowledge appended:**
  AD-claude-767-the-browser-tab-answers-choosers-saves-downloads-and-can-be-taken-over-001;
  F-claude-767-take-over-read-the-agent-chips-five-second-record-001;
  L-claude-767-the-control-bar-moves-the-page-after-an-agents-first-action-001.
- **Brain:** consultation f61ab97826fe4ca98ce319fda9ecbe1a closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed.

