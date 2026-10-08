# A notice for the manager's reports — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-688-a-notice-for-the-managers-reports.md
- **Pipeline spec:** 688-a-notice-for-the-managers-reports.spec.md

## Phase 1 — Plan
- **Request:** the cron's last Marley item, unblocked by harness TICKET-106 and 111 (closed
  2026-10-08).
- **Recall:**
  - #694's Manager entry.
  - #534's polling.
  - #538's banners.
  - #508's inbox.
  - PR-687: the scenario types into nothing.
- **Probe (2026-10-08, the harness's built `rh`):**
  - `rh new mgr -- script`, whose script runs `rh report --source fixture --seq 1 idle`.
  - Then `rh manager agent/<ws>`.
  - Then, inside that terminal, `rh mcp --grant agent` with `thread_post {kind: report, text}`
    answers `accepted` with `{id, sequence}`.
  - `rh thread read --after 0` then gives
    `{events: [{change: {author, kind, text, id, session, created_ms}, kind: "thread_record", sequence}], next_cursor}`.
- **Design:**
  - **`harness.rs`:**
    - `Harness.thread_cursor: Option<u64>` and `Harness.unread: Vec<ManagerRecord {id, kind, line}>`.
    - In `connected`, after the fleet's events, `follow_thread` runs while a session holds
      `role: manager` and `writes_on`. With no cursor yet it pages to the end silently; after
      that it reads and returns the new manager records.
    - `cx.update` then runs `manager_posted(records, cx)`. When the Manager thread is not in
      front, that pushes to `unread` and posts a notification (tag `marley-manager-<id>`).
    - In-front checks each second clear `unread`.
    - `manager_in_front(cx)`: the active window's `MultiWorkspace` workspace, the Agent Panel
      visible, and `selected_agent` being `Custom { id: "Manager" }`.
  - **`rail.rs`:**
    - `manager_entries` lists each unread record as an `InboxKind::Harness` entry: agent
      "Manager", project "Harness", the kind and the first line.
    - `InboxTarget::Manager` opens the newest thread whose agent is Manager, or starts one.
  - **Manifest:** `harness.rs`, `rail.rs`, the scenario, and the docs.
- **Visual check plan:** see the spec's UI proof. The scenario drives the post by touching a
  file the fixture waits for.

## Phase 2 — Code
- **Built:**
  - `harness.rs`:
    - `ManagerRecord` with `kind_word`, plus `Harness.thread_cursor` and `unread`.
    - `has_manager`, now shared with #694, and `manager_in_front`.
    - `ThreadPage`, `read_thread` (at most `THREAD_PAGES` pages), `follow_thread` (run after
      each fleet poll) and `manager_posted`.
  - `rail.rs`: `InboxTarget::Manager`, `open_manager_thread` and `manager_entries`.
- **Review:**
  - The first read sets the cursor and raises nothing (REQ-003).
  - On a resync the cursor resets, and the next read is quiet again.
  - A thread error never ends the fleet's connection; it is logged at debug.
  - Unread records clear on the first pass that finds the thread in front.
- **Gate:** `scratchpad/688-gate-1.log`: **GATE GREEN [diff]**, after `cargo check` flagged
  three unneeded path prefixes in `rail.rs`.

## Phase 3 — Test
- **Scenario:** `script/e2e/688-a-notice-for-the-managers-reports.sh`: #538's private bus with a
  banner log, the harness's built `rh`, and the fixture manager terminal.
  - Run 1 measured the entry.
  - Run 2 passed 2 of 2.
  - Run 3, with a third shot added, passed 2 of 2 (`scratchpad/688-e2e-3.log`).
- **Shots** (run 3):
  - `688-01-needs-you` (REQ-001, REQ-002):
    - "Needs you 1", the entry "Manager · H…" with "Report: The ga…";
    - the Harness section, mgr idle;
    - the banner log: `Marley|Manager: Report|The gate passed on TICKET-post1`, the first line
      only.
  - `688-02-in-front`: the click opened "New Manager Thread" (Manager · idle in the rail), and
    Needs you is gone.
  - `688-03-streamed`: after the second report, the thread shows "Report: The gate passed on
    TICKET-post2" and "the rest of the report". The banner log still has one Manager line.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

- **Gate after Test:** shellcheck (SC2034) found an unused `local pid` in the scenario's teardown, which is now gone. **GATE GREEN [diff]** (`scratchpad/688-gate-3.log`); the teardown change alters no step.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md` (#688).
  - The guide's manager passage.
  - `marley_workbench.md`.
  - The plan's TICKET-688 line, marked done.
- **Knowledge:** `AD-claude-688-the-managers-records-are-polled-and-raised-while-its-thread-is-not-in-front-001`.
- **Ticket:** TICKET-688 closed.
