---
pipeline_id: 5ff64b7a-90d7-41cd-9d21-be5fd42a3a34
ticket: docs/planning/tickets/closed/TICKET-767-file-choosers-downloads-and-take-over.md
status: Phase 4 — Complete PASS
title: File choosers, downloads and Take Over in the Browser tab
type: feature
slice: prong 3 (the Browser tab), the third slice of the T3 Code survey's item 10 (#756, #766 before it)
references:
  - docs/planning/pipeline/completed/756-the-browser-tools-agents-lack.notes.md
  - docs/planning/pipeline/completed/525-agent-drives-a-running-program.spec.md
  - docs/planning/pipeline/completed/495-select-picker.spec.md
  - docs/t3code_architecture/05-terminal-browser-and-capture.md
---

## Title
Headless Chromium draws no file chooser and saves no download where anyone sees it, and an agent
acting in a Browser tab cannot be stopped there as one typing in a terminal can. A page's file
chooser now opens Zed's path prompt for the person or waits for an agent's `browser_upload`,
downloads land in the Downloads folder with a notification, and Take Over (Ctrl-I) hands the page
to the person until Hand Back. Chad, 2026-10-11: "18.) build now", item 10's third slice.

## Scope
### In
- **File choosers.** Each page intercepts its file chooser (`Page.setInterceptFileChooserDialog`).
  - When the person's press opened it (within a second, as #495's select list tells), Zed's path
    prompt opens (files, several when the input takes several) and the files chosen are set on
    the input; cancelling sets nothing.
  - Otherwise (an agent's click) the chooser waits: a bar under the toolbar says "The page asks
    for a file" with Choose… (the path prompt) and Cancel, and `browser_upload` without `ref`
    answers it with the same checks as with a ref (open projects only, no secret file). A newer
    chooser replaces an unanswered one. `browser_tabs` and `browser_look` report it.
  - `browser_upload` with no `ref` and no chooser open is refused `no_chooser`.
- **Downloads.** At each browser's start Marley sends `Browser.setDownloadBehavior`
  (`allowAndName`, events on) into the Downloads folder: `$XDG_DOWNLOAD_DIR`, else the user dirs'
  download folder, else `~/Downloads`.
  - `Browser.downloadWillBegin` and `downloadProgress` are followed; a finished download is
    renamed from Chromium's id to its suggested name (with " (1)" and on when taken) and a
    notification names it with Open and Show in Folder. A cancelled one says so in the log.
  - `browser_tabs` lists each tab's downloads this session: name, path, state.
- **Take Over.** Once an agent has acted in a page, a bar under the toolbar names who acted and
  offers Take Over; Ctrl-I in the Browser tab does the same, and reaches the page as before when
  no agent has acted. While taken, the bar reads "You have control" with Hand Back, and the
  browser write tools refuse that tab with `taken_over` ("the user has taken over this tab; wait
  until they hand it back").
- **Docs:** the guide's Browser tab sections and the in-app guide; `marley_browser.md` and
  `marley_workbench.md`.

### Out (explicitly deferred)
- A downloads list or panel beyond the notification and `browser_tabs`.
- Choosing where a download goes, or asking before one starts.
- Drag and drop of files from the desktop into a page.
- A generation that fails an agent's write queued before a change of control: a browser write
  is one short action, checked when it starts.

## Reference (§20)
N/A — Marley-specific: the Browser tab is Marley's headless Chromium drawn in a Zed pane. The
behavior follows T3 Code's preview browser as the survey read it (report 05 §2.4 to §2.6: its file
chooser answered by a path prompt or a tool, its downloads, `ServerBrowser.ts:849-946`) and
Marley's own terminal take-over (#525), which the Browser tab's matches: the same key, the same
bar words, the same refusal.

### Prior art
- **The code we ship:**
  - `marley_browser::page::Page::attach` (`page.rs:460-492`, `Page.enable` at 484) for the
    interception; `set_file_input` (1237-1250, `DOM.setFileInputFiles`), which a chooser's
    `backendNodeId` fits.
  - `marley_browser::relay` (`chromium_message` 772-835): a browser-level command Marley sends
    goes on its browser session, so `Browser.download*` events come back with no `sessionId` and
    pass; the hub drops them today in `follow_observed`'s `_` arm.
  - `browser.rs`: `BrowserHub::start` (1098-1192, `pages_and_identity` at 1159) for the download
    behavior; `follow` (3705-3803) for session-less events; `follow_navigation` (3815-3912) for
    `Page.fileChooserOpened`; the pause bar (`pause_bar` 5208, `render_pause` 5217) as the bars'
    pattern; `PageState.agent` and `driven_by` for "an agent acted"; #495's `USER_PRESS` and
    `pressed_at` for whose press opened the chooser.
  - `terminal_drive.rs` (#525): `Drive.taken_over`, `toggle_control` (612-621, the key reaches the
    program when nothing was taken), `footer` (625-707), the refusal words (396-400);
    `keymap.json:29-34` binds Ctrl-I in `Terminal`.
  - Zed's `Workspace::prompt_for_open_path` (`workspace.rs:3443`), as `agent_bar::attach` uses it;
    `cx.open_with_system` and `cx.reveal_path` (`gpui/src/app.rs:1913-1918`); `dirs::download_dir`
    (a workspace dependency).
- **Behavior maps:** T3 survey report 05 §2.4 to §2.6 and §3; Orca report 03 §2.11.
- **Published material:** the Chrome DevTools Protocol: `Page.setInterceptFileChooserDialog`,
  `Page.fileChooserOpened`, `DOM.setFileInputFiles`, `Browser.setDownloadBehavior`,
  `Browser.downloadWillBegin`, `Browser.downloadProgress`.

## UI proof
`script/e2e/767-file-choosers-downloads-and-take-over.sh`, under `compositor sway`, with the
browser fixture and `use_system_path_prompts` off, `XDG_DOWNLOAD_DIR` in the run's folder. The
page holds a file input that lists the chosen names, and a link that downloads `report.csv`.
- `767-01-person-prompt`: the person clicks the page's Choose Files: Zed's path prompt is open.
- `767-02-person-chosen`: the path of `notes.txt` typed and Enter: the input lists it.
- `767-03-agent-chooser`: an agent clicks Choose Files: the bar "The page asks for a file".
- `767-04-agent-upload`: `browser_upload` with no `ref`: the input lists `more.txt`, the bar gone.
- `767-05-download`: an agent clicks the download link: the notification names `report.csv`
  with Open and Show in Folder; the file is in the run's Downloads folder (checked).
- `767-06-taken`: Ctrl-I in the tab after the agent acted: "You have control", Hand Back; an
  agent's click is refused `taken_over` (checked).
- `767-07-handed-back`: Hand Back clicked: the bar offers Take Over again, and an agent's click
  works (checked).

## Locked-In Decisions
- **D1:** a chooser the person opened goes straight to the path prompt; one an agent's click
  opened waits for `browser_upload`, so an agent never puts a prompt in front of the person.
- **D2:** downloads go to the Downloads folder (T3 survey 05 open question 3, its default), never
  a folder Marley invents; `XDG_DOWNLOAD_DIR` first, so a scenario keeps them in its own folder.
- **D3:** Take Over matches the terminal's (#525): Ctrl-I only after an agent acted, else the key
  reaches the page; the same bar words and refusal.
- **D4:** the take-over check happens when a write starts; no generation (Out).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the person's press opens a page's file chooser, the system shall open Zed's path prompt and set the files chosen on the input. | Shots 767-01, 767-02 |
| REQ-002 | WHEN an agent's action opens a file chooser, the system shall hold it with a bar and let `browser_upload` without `ref` answer it. | Shots 767-03, 767-04 |
| REQ-003 | WHEN a page's download finishes, the system shall save it in the Downloads folder under its suggested name and show a notification with Open and Show in Folder. | Shot 767-05 and the file (checked) |
| REQ-004 | WHEN the person takes over a tab an agent acted in, the system shall refuse the browser write tools there with `taken_over` until Hand Back. | Shot 767-06 and the reply (checked) |
| REQ-005 | WHEN the person hands the tab back, the system shall let agents act there again. | Shot 767-07 and the reply (checked) |
| REQ-006 | WHILE no agent has acted in a tab, Ctrl-I shall reach the page. | The review of the diff |
| REQ-007 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_browser` (`page.rs`: the interception, the download behavior);
  `marley_workbench` (`browser.rs`: chooser, downloads, control and their bars; `browser_tools.rs`
  and `browser_actions.rs`: `taken_over`, `browser_upload` on a chooser, `browser_tabs`;
  `marley_workbench.rs`: the action; `keymap.json`: Ctrl-I); the guides and notes; a review of
  the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
