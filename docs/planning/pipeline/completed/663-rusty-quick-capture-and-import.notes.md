# Quick capture and import from the palette — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-663-rusty-quick-capture-and-import.md
- **Pipeline spec:** 663-rusty-quick-capture-and-import.spec.md

## Phase 1 — Plan (2026-10-06)
- **Request:** the Queue's top after #662, from Chad's 2026-10-06 "adding the last things missing".
- **Classification / tier:** feature, R-D3's Main.qml dialogs; Marley crates only.
- **Pre-flight:** green; no active pipeline; cargo idle; 75 GB free on the build disk.
- **Recall (§18.3):**
  - Nothing in the ledgers on capture or import. The rule on blocking IO in a handler holds already:
    every call to Rusty is a spawned task.
  - AD-662: after Marley's own vault writes, read the vault and the bookmarks again, since the
    service connection announces nothing.
  - #660's form (`rusty/follow_up.rs`): a modal with editors, `menu::Confirm` on Enter through a
    `… menu` key context, a refusal kept in the form.
  - The brain (consultation `d6e8d2a5a77d4229b049c50d3b749827`): nothing on this seam. A duplicate
    ask (`ca4159cb…`) was closed with `brain no-decision`.
- **Discovery:**
  - `rusty.rs`: `call` (533-561) races the request against `CALL_TIMEOUT` (5 s, :90) and words
    "took more than 5 s"; `call_tool` (564-582) strips Rusty's prefixes; `unavailable` (595);
    `reread_vault` (631). Zed's client defaults to 60 s (`context_server/src/client.rs:30`) and
    `protocol.request_with` (`protocol.rs:111-118`) takes a timeout.
  - `brain.rs`: `today` (601-623) calls `brain_daily_note`, rereads, reveals and opens kept.
  - `page.rs`: `init` (146-169) registers `OpenPage` on every workspace with the `unavailable`
    toast; `open_later` (173-190).
  - `Workspace::prompt_for_open_path` (`workspace.rs:3429`) falls back to Zed's own prompt when
    `use_system_path_prompts` is off; `DirectoryLister::Local(project, fs)` (`project.rs:1013`).
  - `ModalView::on_before_dismiss` (`modal_layer.rs:50-68`); `Toast::on_click`
    (`workspace.rs:795`).
  - The stand-in: `today_in_state` (247-255), `daily_note` (856-871, today from `time.strftime`),
    `write_page` (616-620), `VAULT_TOOLS` (1214-1290), the watcher's stamp covering the vault.
  - Rusty: `CaptureParams` (`target` is `daily` or `inbox`), `CaptureReceipt { slug, entry_id,
    created_page }`, the timeline line `- **DATE** (mcp) — TEXT` under `## Timeline`;
    `capture_url` answers the page either way, a failure as `status: failed` and `error` in its
    frontmatter (flattened into `frontmatter`), the answer marked `untrusted`; `ImportPlan` and
    `ImportReport` as in the spec.

### Design
- **`marley_rusty/src/capture.rs`** (Marley): `BRAIN_CAPTURE`, `SOURCE_CAPTURE`,
  `BRAIN_IMPORT_PLAN`, `BRAIN_IMPORT`; the deadlines (`CAPTURE_URL_DEADLINE` 45 s,
  `IMPORT_PLAN_DEADLINE` 60 s, `IMPORT_DEADLINE` 600 s); `CaptureTarget { Daily, Inbox }` with
  `as_str` and its form's title and placeholder; `capture_arguments(text, target)`;
  `CaptureReceipt` and `receipt_from_answer`; `SourcePage { slug, title, failed: Option<String> }`
  from the answer (`frontmatter.status` and `frontmatter.error`); `ImportPlan`, `ImportReport`
  (`Deserialize`, defaults) with `ImportPlan::summary()` and `details()` and
  `ImportReport::summary()`, worded as Rusty's app words them; `ImportPlan::brings_anything()`.
- **`rusty.rs`**: `call(server, tool, arguments, deadline, cx)` through `request_with(.., Some
  (deadline))` and the timer; `call_tool_within`; `call_tool` passes `CALL_TIMEOUT`.
- **`rusty/capture.rs`** (new, Marley): `init` registers `CaptureToToday`, `CaptureToInbox`,
  `CaptureUrl` and `OpenToday` (`actions!(rusty, ..)`) on every workspace, each checking
  `unavailable`. `CaptureModal { kind: CaptureKind (Line(CaptureTarget) | Url), field, sending:
  Option<Task<()>>, refusal: Option<SharedString> }`, `ModalView`, key context `RustyCapture menu`;
  `menu::Confirm` sends, `menu::Cancel` dismisses. A line: `call_tool(BRAIN_CAPTURE)`; on `Ok` the
  toast with Open (`page::open_later`), `rusty::reread_vault`, dismiss. A URL:
  `call_tool_within(SOURCE_CAPTURE, CAPTURE_URL_DEADLINE)`; on `Ok` open the page kept, toast a
  failure's error, reread, dismiss. `open_today(workspace, window, cx)` for `OpenToday`, which the
  Brain view's `today` keeps its own reveal around.
- **`rusty/import.rs`** (new, Marley): `ImportVault` registered the same way; the prompt
  (`prompt_for_open_path`, directories only, `DirectoryLister::Local`), then `ImportModal { path,
  stage: Reading | Plan(ImportPlan) | Importing(ImportPlan) | Done(ImportReport) | Failed(String),
  task }`: the path, the stage's sentence, the details in a scrolled block, and Cancel or Close,
  Import, Open Report. `on_before_dismiss` refuses while `Importing`. Done rereads the vault and
  `favourites::read`.
- **The stand-in**: `brain_capture` (the page made when missing, the timeline line appended under
  `## Timeline`, the receipt), `source_capture` (`urllib` with a 20 s timeout, the title from
  `<title>`, the text with tags stripped, `sources/<slug>`; a failure recorded as Rusty does; an
  `ftp:` URL refused in Rusty's words), `brain_import_plan` and `brain_import` (the vault's pages
  and attachments, dot entries skipped, collisions, tags, unresolved links, bookmarks from
  `.obsidian/bookmarks.json`; the import copies, rewrites bare-name links to paths, adds the
  bookmarks and writes a report page under `inbox/`); `daily_note` takes today from
  `today_in_state`.
- **`guide/index.html`**: the Rusty article names capture, import and open today.
- **File manifest:** `crates/marley_rusty/src/{capture.rs, marley_rusty.rs}`, the stand-in,
  `crates/marley_workbench/src/rusty.rs`, `crates/marley_workbench/src/rusty/{capture.rs,
  import.rs, brain.rs}`, the guide page (all Marley); `script/e2e/663-rusty-quick-capture-and-
  import.sh` (Test). No Zed crate.

### Visual check plan
| Criterion | What the scenario does | Shot |
|---|---|---|
| REQ-001 | Palette: `rusty: capture to today` | `663-01-capture-form` |
| REQ-003 | Enter with nothing typed | `663-02-refused` |
| REQ-002 | Type a line, Enter | `663-03-captured` |
| REQ-004 | Palette: `rusty: open today` | `663-04-today` |
| REQ-002 | Capture to inbox, then the toast's Open | `663-05-inbox` |
| REQ-005 | `rusty: capture url`, the slow page, Enter, a shot at 2 s | `663-06-capturing` |
| REQ-006, 007 | Wait past 7 s | `663-07-source` |
| REQ-006 | The missing page | `663-08-fetch-failed` |
| REQ-003 | An `ftp:` URL | `663-09-url-refused` |
| REQ-008 | `rusty: import vault`, the fixture path typed in Zed's prompt | `663-10-folder` |
| REQ-008 | Enter in the prompt | `663-11-plan` |
| REQ-010 | Click Import | `663-12-imported` |
| REQ-011 | The Brain view open; click Open Report | `663-13-report` |
| REQ-009, 012 | Not shot | Review |

### Risks
- **The service connection over HTTP** may carry a transport timeout of its own; the scenario runs
  the embedded connection, where the deadline is the only one.
- **A long import** holds the form for up to 10 minutes; Rusty's import is one transaction, so a
  timeout leaves nothing behind and the form says so.
- **The source page's text is untrusted** (Rusty marks it); Marley renders it as any page, with no
  action taken on its content.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall, the brain; discovery.
- [x] Mint the pair; the BACKLOG row removed; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request ("continue on tickets until finished").

## Phase 2 — Code (2026-10-06)
### Built
- **`marley_rusty/src/capture.rs`**: the four tool names; the deadlines (45 s, 60 s, 600 s);
  `CaptureTarget` (`as_str`, `title`, `arguments`); `CaptureReceipt` and `receipt_from_answer`;
  `source_arguments`; `SourcePage` and `source_page_from_answer` (the slug, and Rusty's `error`
  when `frontmatter.status` is `failed`); `import_arguments`; `ImportPlan` (`brings_anything`,
  `summary`, `details`) and `ImportReport` (`summary`), worded as Rusty's app words them, with
  `plan_from_answer` and `report_from_answer`. The plan's bookmarks are #662's `Bookmark`.
- **`rusty.rs`**: `call_within` and `call_tool_within` carry a deadline to Zed's client
  (`request_with`) and to Marley's own timer, whose message names the seconds; `call` and
  `call_tool` keep 5 s.
- **`rusty/capture.rs`**: `CaptureToToday`, `CaptureToInbox`, `CaptureUrl` and `OpenToday` on every
  workspace behind `ready` (the `unavailable` toast); `CaptureForm` (one field, Enter through
  `menu::Confirm` in `RustyCapture menu`, Capturing… on the button while a call runs, Rusty's
  refusal in the error colour); a line's toast with Open; a URL's page opened kept, its failure
  toasted; `open_today` over `brain_daily_note`.
- **`rusty/import.rs`**: `ImportVault`; the folder through `prompt_for_open_path` with
  `DirectoryLister::Local`; `ImportForm` with `Stage` (reading, plan, importing, done, failed): the
  path, the sentence, the lists in a scrolled block, Cancel and Import (enabled when something
  comes in), Importing…, Open Report and Close; `on_before_dismiss` holds it while importing; a
  report rereads the vault and the bookmarks.
- **The stand-in**: `brain_capture` (the page made when missing, the line under `## Timeline`),
  `source_capture` (a 20 s fetch, the title and paragraphs of an HTML page, a failure recorded as
  `status: failed` and `error` with "Capture failed: …" as the body, an `ftp:` URL refused),
  `brain_import_plan` and `brain_import` (pages and attachments, dot entries skipped, collisions
  skipped, tags, unresolved links, `.obsidian/bookmarks.json`, bare names rewritten to paths, a
  report page under `inbox/`). Its slow tools answer on a thread of their own, as Rusty answers
  every request, so Marley's 5 s ping goes on during a 7 s fetch. `daily_note` takes today from
  `today_in_state`. A smoke run over stdio passed every tool, the ping included.
- **The guide page**: a Capture and import article; the page picker's line now names favourites
  first (#662 had left it saying the recent pages come first).

### Deviations
- D6: `rusty: open today` is its own small function in `capture.rs`; the Brain view's Today keeps
  its own, which also reveals the page in the tree. The behaviour is the same.
- The capture form closes on Escape while a capture runs; Rusty still finishes it, and the Brain
  view shows it on Rusty's announcement. Only the import's form holds itself open.

### Review
- Clippy: `missing_const_for_fn` (`brings_anything`), `derive_partial_eq_without_eq` on the
  actions (the crate's idiom, `#[derive(Eq)]` on each, as `tasks_tab.rs` does),
  `unused_self` (`send_line` and `send_url` are associated functions), `clone_on_ref_ptr`
  (`Arc::clone` for the lister's `Fs`), needless `&mut` on `open_today`, `ImportForm::new` and
  `open_report`.
- Re-entrancy: `open_today`'s failure toasts through the workspace it is updating, not through a
  second update of it; the forms' toasts update the workspace from their own spawned tasks; every
  page opens through `page::open_later`, which defers.

### Gate
`just gate-diff`: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-06)
Scenario `script/e2e/663-rusty-quick-capture-and-import.sh` under `compositor sway`: the stand-in
over a scratch vault (`home`, `notes/old`) with its day fixed at 2026-10-06; a fixture web server
on 127.0.0.1 at a free port (`/slow` answers after 7 s, the rest 404), stopped in `teardown`; a
scratch Obsidian vault (`projects/atlas` tagged `idea` linking `[[plan]]` and `[[nowhere]]`,
`projects/plan`, `notes/old` colliding with the brain's, `assets/map.png`, `.obsidian/
bookmarks.json` with a file and a graph bookmark); `use_system_path_prompts` off. Three runs;
every check passed in each.

### What each shot shows (third run)
- `663-01-capture-form` (REQ-001): "Capture to Today's Note", the empty field, the hint, Capture.
- `663-02-refused` (REQ-003): Enter with nothing typed: Rusty's "Nothing to capture" in red.
- `663-03-captured` (REQ-002): the form gone; the toast "Captured to daily/2026-10-06" with Open.
- `663-04-today` (REQ-004): `rusty: open today`: the 2026-10-06 tab, the line under Timeline.
- `663-05-inbox` (REQ-002): a line to the inbox, the toast's Open: the Inbox tab with the line.
- `663-06-capturing` (REQ-005): the URL in the field and Capturing… on the disabled button.
- `663-07-source` (REQ-006, 007): after the 7 s answer, past the old 5 s: "Notes on Slow Pages" in
  a kept tab with its url, site, captured and kind, and the page's two paragraphs.
- `663-08-fetch-failed` (REQ-006): the missing page: the `127.0.0.1` source page with `status:
  failed`, `error: HTTP 404 Not Found` and "Capture failed: …", and the toast with Rusty's error.
- `663-09-url-refused` (REQ-003): `ftp://example.com/file` kept in the field, Rusty's "only http and
  https URLs are captured" under it.
- `663-10-folder` (REQ-008): Zed's own folder prompt with `obsidian` selected; the Brain view
  already lists daily, inbox and sources from the captures.
- `663-11-plan` (REQ-008): the path ending `…/obsidian`; "Will bring in 2 pages in 2 folders, 1
  attachment, 2 tags and 1 bookmark; 1 collision skipped; 1 link unresolved. …"; the lists; Cancel
  and Import.
- `663-12-imported` (REQ-010): "Imported 2 pages and 1 attachment; 1 link rewritten to vault paths;
  1 bookmark added. The report is inbox/import-obsidian-2026-10-06."; Open Report and Close; the
  rail shows assets and projects, and Atlas under Favourites.
- `663-13-report` (REQ-011): the report page in a kept tab, its long path wrapped, its outline
  beside it; the Brain view as after the import.
- REQ-009 and REQ-012 by review: Import is `disabled(!plan.brings_anything())` and `confirm` checks
  the same; each command goes through `ready` (the `unavailable` toast), and #661's filter hides
  the `rusty` namespace while Rusty is off.

### Fixes, each rebuilt and run again
- **The confirmation toast stayed** (first run): "Captured to …" sat in the corner until closed.
  It now hides itself after Zed's 5 s, as Zed's own confirmations do; a failure's toast stays.
- **The import's path was cut at its end** (first run, `663-11`): the folder's name was the part
  hidden. The label now truncates from its start.
- **A long line widened the Page tab** (second run, `663-13`): the report's path in code pushed
  the body past the tab and the outline column and the rows' × off it. The body is `flex_1` in a
  row with no `min_w_0`, so it could not shrink below its widest line; it now shrinks and clips.
  This was in the Page tab since #645, shown by the report page.
- **The scenario's clicks**: the toast's Open at (1171, 933) and Open Report at (1011, 512); the
  first run's guesses missed both (the second closed the form by clicking outside it).

### Seen, not in scope
- The rail's CONTAINERS rows show the machine's own Docker ports (#521's port rows); shots never
  go in the repository.
- "Installing html extension…" in the status bar of the second run: Zed's extension suggestions,
  not this change.

The gate runs on the final tree at Complete, after the fixes and the docs.

## Phase 4 — Complete (2026-10-06)
- **Docs:** `CHANGELOG.md` (Added; Fixed for the Page tab's width); `docs/marley/rusty-in-marley.md`
  (the Main.qml dialogs row, the batch line); `docs/marley/guide.md` (Capture and import);
  `docs/marley/walkthrough.md` (stop 2.15e, the commands in Appendix B);
  `docs/marley_architecture/marley_rusty.md` (`capture`, the stand-in's four tools and its thread);
  `docs/marley_architecture/marley_workbench.md` (`call_within`, the Page tab's body, the Capture
  and import section). No Zed crate touched, so no zed-touchpoints row.
- **Knowledge:** F-claude-663-a-long-line-widened-the-brain-page-tab-001,
  L-claude-663-the-stand-in-answers-a-slow-tool-on-its-own-thread-001,
  L-claude-663-actions-in-marley-crates-derive-eq-001,
  AD-claude-663-a-call-to-rusty-names-its-deadline-001.
- **Brain:** `brain decide` on consultation `d6e8d2a5a77d4229b049c50d3b749827`, follow up by
  2026-11-06: `decisions/a-call-from-marley-to-rusty-names-its-own-deadline`.
- **Closed:** the ticket in `tickets/closed/`; this pair in `completed/`.
