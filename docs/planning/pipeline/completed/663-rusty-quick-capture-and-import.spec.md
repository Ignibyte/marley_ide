---
pipeline_id: 1040557a-21bf-4494-8550-3ae266290b97
ticket: docs/planning/tickets/open/TICKET-663-rusty-quick-capture-and-import.md
status: Phase 4 — Complete PASS
title: "Quick capture and import from the palette"
type: feature
slice: Rusty in Marley, R-D3's Main.qml dialogs (docs/marley/rusty-in-marley.md)
references: [docs/marley/rusty-in-marley.md, docs/planning/pipeline/completed/644-brain-view-in-the-rail.spec.md, docs/planning/pipeline/completed/660-rusty-decision-follow-up.spec.md]
---

## Title
Quick capture and import from the palette. Rusty's app captures a line into today's daily note or
the inbox (`brain_capture { text, target, date }`, a timeline entry), keeps a URL as a source page
(`source_capture { url }`, fetched by Rusty within 20 s), and imports an Obsidian vault
(`brain_import_plan { path }`, then `brain_import { path }`). Marley does none of these, and its
5 s deadline on every call to Rusty is too short for the last three. This ticket adds the four
commands and `rusty: open today`, and lets a call name its own deadline.

## Scope
### In
- **Capture a line.** `rusty: capture to today` and `rusty: capture to inbox` open a small form
  over the workspace with one field. Enter sends the line to Rusty (`target` `daily` or `inbox`);
  the form closes and a toast says where it went, with Open. Rusty's refusal (an empty line) shows
  in the form, which keeps the text.
- **Capture a URL.** `rusty: capture url` opens the same form for a URL. While Rusty fetches, the
  form says Capturing…; then the source page opens in a kept tab. A fetch that failed still makes
  the page (Rusty records the failure on it), and a toast gives Rusty's error. A refused URL (not
  http or https) shows Rusty's words in the form.
- **Import a vault.** `rusty: import vault` asks for a folder (Zed's folder prompt, on this
  machine, since Rusty reads its own disk), then shows Rusty's plan: what comes in, what is
  skipped as already in the brain, the links that will not resolve, the bookmarks, with the full
  lists below. Import runs it; the form then gives Rusty's report with Open Report. The form
  cannot be closed while the import runs.
- **`rusty: open today`**: Rusty's daily note for today, made when missing, in a kept tab: what the
  Brain view's Today does, from the palette.
- **Deadlines.** A call to Rusty names its deadline: 5 s stays the default; a URL capture has 45 s,
  a plan 60 s, an import 10 minutes. Zed's MCP client gets the same deadline, so its own 60 s
  default does not cut an import short.
- **After a change.** A capture or an import reads the vault again for the Brain view (the service
  connection announces nothing); an import reads the bookmarks again (#662).
- **`marley_rusty::capture`** (the tools, the targets, the answers and their summary lines) and the
  stand-in's four tools.

### Out (explicitly deferred)
- **Appending to the page in front's timeline** (Rusty's "Timeline: Append an entry"); a later step.
- **A capture key outside Marley** (a desktop-wide hotkey); Rusty's own binding covers it.
- **Cancelling an import**; Rusty's tool has no cancel, and a failure part way leaves nothing behind.
- **Searching sources** apart from the brain search, where `type:source` already works.

## Reference (§20)
N/A — Marley-specific: Rusty's own app is the reference for the behaviour (`crates/rusty-app/qml/
Main.qml:148-220`, `:519`, `:552-556`, `:710-715` in Rusty's repository at `eb1ab51`): a prompt for
each capture, a URL dialog that opens the page it made and notes a failed capture, and an import
that picks a folder, shows the plan's sentence and lists, enables Import only when something
comes in, and gives the report. Zed's own modals, editor, buttons, toasts and folder prompt draw it.

### Prior art
- **Behaviour maps:** `docs/t3code_architecture/` and `docs/orca_architecture/` hold no capture into
  a notes vault; `docs/zed_architecture/` none for an import.
- **Published material:** Rusty's tools (`crates/rusty-mcp/src/main.rs:1094-1113`, `:1430-1475`;
  `CaptureParams` `:556-565`, `ImportParams` `:681-684`, `SourceCaptureParams` `:688-691`), its
  store (`crates/rusty-core/src/brain/mod.rs`: `capture` `:1622-1655` answering `CaptureReceipt`,
  `capture_url` and `capture_fetched` `:1938-2030` recording `status: failed` and `error`;
  `brain/import.rs:23-58` `ImportPlan` and `ImportReport`; `brain/sources.rs:47` the 20 s fetch).
- **The code we ship:** `Workspace::prompt_for_open_path` and `DirectoryLister::Local` (as
  `agent_bar.rs:58-79` uses the prompt); `context_server::protocol::request_with`'s timeout;
  `ModalView::on_before_dismiss`; the follow-up form's modal, editors and keys (#660,
  `rusty/follow_up.rs`); the Brain view's Today (`rusty/brain.rs`); Zed's `Toast::on_click`. No
  crate we build owns capture or import.

## UI proof
`script/e2e/663-rusty-quick-capture-and-import.sh` (`compositor sway`: a click on a toast's Open
and on the forms' buttons). Setup: the stand-in over a scratch vault with the day fixed by its
`today` file; a fixture web server on 127.0.0.1 with a page that answers after 7 s and a missing
one; a scratch Obsidian vault (two folders, three pages, an attachment, a page whose slug is
already in the brain, an unresolved link, `.obsidian/bookmarks.json` with one file bookmark); the
run's settings with `use_system_path_prompts` off, so Zed's own folder prompt shows.
`MARLEY_RUSTY_MCP` names the stand-in, never the user's Rusty (R-D8). Shots:
- `663-01-capture-form`: `rusty: capture to today`: the form, its title and field.
- `663-02-refused`: Enter with nothing typed: Rusty's "Nothing to capture" in the form.
- `663-03-captured`: a line typed, Enter: the form gone, the toast naming today's note, with Open.
- `663-04-today`: `rusty: open today`: today's note with the line under its timeline.
- `663-05-inbox`: `rusty: capture to inbox`, a line, Enter, the toast's Open: the inbox page with it.
- `663-06-capturing`: `rusty: capture url` with the slow page: Capturing… in the form.
- `663-07-source`: after 7 s, more than the old 5 s: the source page in a kept tab.
- `663-08-fetch-failed`: the missing page: its source page saying the capture failed, and the toast.
- `663-09-url-refused`: an `ftp:` URL: Rusty's refusal in the form.
- `663-10-folder`: `rusty: import vault`: Zed's folder prompt, the fixture vault typed.
- `663-11-plan`: the plan: the sentence and the lists, Import enabled.
- `663-12-imported`: Import: the report's sentence, Open Report and Close.
- `663-13-report`: Open Report: the report page; the Brain view lists the imported folders.

## Locked-In Decisions
- D1 — **A call names its deadline**: `rusty::call_tool_within(tool, arguments, deadline, cx)`,
  `call_tool` at 5 s as before; the deadline goes to Zed's client through `request_with` as well as
  to Marley's own timer, whose message names the seconds.
- D2 — **One form for the three captures** (`rusty/capture.rs`), one field each, Enter sends,
  Escape closes; while a capture runs the form says Capturing… and a second Enter does nothing.
- D3 — **A line's capture closes the form and toasts** "Captured to `<slug>`" with Open; a URL's
  capture opens the source page kept and focused, and toasts Rusty's `error` when the page records
  a failed fetch.
- D4 — **The import's folder comes from `DirectoryLister::Local`**: Rusty reads this machine's disk,
  so a remote project's machine would be the wrong one.
- D5 — **The import form follows Rusty's app**: the plan's sentence and its lists (pages,
  attachments, collisions, unresolved links, tags, bookmarks, bookmarks not carried), Import only
  when a page or an attachment comes in, the report's sentence, Open Report; `on_before_dismiss`
  holds the form while the import runs.
- D6 — **`rusty: open today` shares the Brain view's Today path** (`brain_daily_note`, opened kept
  and focused); the Brain view's Today keeps its reveal in the tree.
- D7 — **The stand-in answers as Rusty does**: `brain_capture`'s `CaptureReceipt` and timeline line,
  `source_capture` fetching with a 20 s cap and recording a failure on the page, and the import's
  plan and report shapes; today's date from its `today` file for the daily note as for the capture.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rusty: capture to today` or `rusty: capture to inbox` runs while Rusty is connected, the system shall open a form with one field titled for its target. | Shot `663-01-capture-form` |
| REQ-002 | WHEN Enter is pressed in a line's form, the system shall send the line to Rusty for its target, close the form on success, and show a toast naming the page with Open. | Shots `663-03-captured`, `663-05-inbox` |
| REQ-003 | IF Rusty refuses a capture, THEN the form shall show Rusty's words and keep what was typed. | Shots `663-02-refused`, `663-09-url-refused` |
| REQ-004 | WHEN `rusty: open today` runs, the system shall open today's daily note in a kept tab. | Shot `663-04-today` |
| REQ-005 | WHILE a URL's capture runs, the form shall say Capturing…. | Shot `663-06-capturing` |
| REQ-006 | WHEN Rusty answers a URL's capture, the system shall open the source page in a kept tab and, IF the page records a failed fetch, show Rusty's error in a toast. | Shots `663-07-source`, `663-08-fetch-failed` |
| REQ-007 | WHEN a call to Rusty names a deadline, the system shall wait that long before giving up. | Shot `663-07-source` (7 s); the stand-in's log |
| REQ-008 | WHEN `rusty: import vault` runs, the system shall ask for a folder on this machine and then show Rusty's plan for it. | Shots `663-10-folder`, `663-11-plan` |
| REQ-009 | WHILE the plan brings in no page and no attachment, Import shall be disabled. | Review (the button's `disabled`), a refused or empty plan |
| REQ-010 | WHEN Import is chosen, the system shall run Rusty's import, hold the form open until it answers, and show the report with Open Report. | Shot `663-12-imported` |
| REQ-011 | WHEN a capture or an import succeeds, the Brain view shall show the vault as it now is. | Shot `663-13-report` |
| REQ-012 | WHILE Rusty is off or not connected, the commands shall say why in a toast and open nothing. | Review (`rusty::unavailable`, #661's filter hides them while off) |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `marley_rusty::capture`; `call_tool_within`; `rusty/capture.rs` (the line and URL
  form, open today); `rusty/import.rs` (the folder, the plan, the import); the Brain view's Today on
  the shared path; the stand-in's four tools; the guide page; a review; `script/gates.sh --diff`.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG, the plan, the architecture notes, the guide and the walkthrough,
  ledger capture, the brain decision, close, archive, commit.
