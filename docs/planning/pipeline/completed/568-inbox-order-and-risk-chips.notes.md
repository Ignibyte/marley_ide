# The approvals inbox: needs-you order and risk chips — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-568-inbox-order-and-risk-chips.md
- **Pipeline spec:** 568-inbox-order-and-risk-chips.spec.md

## Phase 1 — Plan
- **Request:** the Jev note's use 3 (2026-09-25): "an urgency score; nouls for destroys,
  outside the project, sends data out, credentials, rewrites history, installs, claims
  approval", which "orders #508's inbox and marks entries; approves nothing", recommendation 5
  ("Inbox order and risk chips on #508, with auto mode kept as the approver"). Chad approved the
  use on 2026-09-26 with the rules quoted in #565's D1 to D4 and kept Claude Code's reviewer as
  the approver.
- **Classification / tier:** feature, prong 2 (attention). Marley crates: `marley_agent`,
  `marley_rail`, `marley_system_one`, `marley_workbench`. One Zed path with a row:
  `marley_page.rs`. Depends on #508 and #565. Size S to M, as the note sizes it.
- **Recall (§18.3):**
  - #508's queued notes: `InboxEntry { key, project, agent, ask, waited, answers }`; the rail
    keeps when each entry was first seen; the entries are gathered in `build_snapshot` and the
    answer runs from a click into `ThreadView`, deferred out of the rail's update
    (PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001). The ask's task is
    spawned the same way and its answer lands in the rail's own state.
  - PR-claude-state-another-entity-reads-is-kept-outside-render-001: the chips and levels are
    computed at refresh and read in render, never computed there.
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001: the ask and the question text
    are masked before they become state; a marker already in the preview is a credentials chip.
  - PR (prevention-rules.md:537): a security invariant is enforced in code: D5's "no path from
    a reading to an answer" is reviewed as such in Code, and the layer offers no verb for it.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: chips widen an entry's
    row; #508's scenario clicks its buttons by coordinates, so Test reruns it with the use off
    and reads its log.
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery** (2026-09-26, at the working tree of `ca70b6488d`; #508 and #565 are queued, so
  their seams are their specs' designs, re-read at promotion as built):
  - #508's spec and notes: the entries (Agent Panel: `ConversationView::pending_tool_call`, the
    thread's `tool_call` with `label`, `tool_name`, `raw_input`; terminal: #519's waiting seats
    with `Question.prompt` and the `tool` label), the order by first-seen, Allow and Deny through
    `ThreadView::authorize_tool_call`, the pick guard; the section's rows from a function in
    `marley_rail`; `508-approvals-inbox.sh`'s fixtures (the stand-in ACP agent that requests a
    permission per prompt and logs the answer; the fake `claude` with a `PermissionRequest`).
  - `crates/marley_agent/src/claude_events.rs`: the `PermissionRequest` arm (303-313: the wait's
    prompt is `Permission for <Tool: preview>`), `tool_line` (460-467), `TOOL_LABEL` (37).
  - `crates/acp_thread/src/acp_thread.rs:947` (`ToolCall`: `label`, `kind`, `raw_input`,
    `tool_name`, per #508's discovery), `:1417` (`WaitingForConfirmation`).
  - `crates/marley_workbench/src/rail.rs`: `refresh` (309), `build_snapshot` (1805), the rows'
    card (1371); `crates/marley_rail/src/marley_rail.rs`: `Row` (253), `rail_rows` (500),
    `has_attention` (636) (#521's discovery lines).
  - `crates/marley_mcp/src/redact.rs:32-97` (the built-in kinds and their patterns; the
    classifier reuses the credential shapes by name, not by copying the regexes: it reads the
    marker).
  - `crates/marley_fleet/src/attention.rs:74` (`attention`: a stable ranking with a clock).
  - `crates/marley_agent/src/marley_agent.rs:76` (`agent_kind_of`).
  - #565's spec: `UseSpec`, `StateBuilder`, `ask` with a dedupe subject, `Mode`, `OutcomeRow`,
    `system_one_setting` in its scenario, the Decisions view's "would add" reading in shadow.
- **Decisions:** D1 to D11 in the spec.

### Promotion (2026-09-28)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #508's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, at `6b0bbee4f5`) ✓; the prior-art sweep ✓; spec and design updated ✓.
  Three Zed paths will change (`marley.rs`, `marley_page.rs`, `default.json`), each with its
  ledger row widened before its edit.
- **Recall, added:** PR-claude-a-state-fact-holds-only-what-code-computed-001 (the working
  directory the draft sent as a fact is a path, so it goes); F-claude-508-a-held-click-reached-the-inbox-only-with-another-refresh-001
  and PR-claude-a-state-another-view-lists-is-announced-by-an-event-001 (the pause's class rides
  the same `PageStatusChanged`); L-claude-565-jevs-answer-shape-as-recorded-001 (a score answers a
  fractional expected level, a confidence and probabilities by level); L-claude-566-a-use-whose-questions-vary-needs-a-set-per-shape-001
  (one static set serves every entry here, since the questions do not vary). The brain
  (consultation 3ef7d5d46eb143cc87b29cd06479ce39): nothing on this seam beyond #508's and #565's
  decisions.
- **What the code says now** (the Explore report, 2026-09-28):
  - #508's `InboxEntry { key, kind, agent, project, ask, waited, answers }` derives `PartialEq` and
    `Eq`, and the rail redraws only when the snapshot changes, so a chip keeps its probability
    as whole permille, not a float. The order is `note_inbox`'s stable sort by first seen, in the
    workbench. `inbox_entries` and `build_snapshot` take `&App`, `note_inbox` a shared
    `&Context`, and `system_one::ask` needs `&mut App`: only `Rail::refresh` can start an ask.
  - A thread entry's `ask` is the call's label on one line, and an Edit's label is Markdown with
    its punctuation escaped (`MarkdownEscaped`). The full `ToolCall` (kind, `tool_name`,
    `raw_input`, `locations`) is in hand in `thread_entry` and kept nowhere after. Zed's
    terminal tool sends `{"command", "cd", …}` and titles the call with the command; `edit_file`
    sends `{"path", "edits"}`; `delete_path` is of kind `delete`; `ToolKind` is non-exhaustive.
  - A terminal seat keeps its wait as `Permission for <Tool: preview>` (`claude_events.rs:423-443`),
    the tool line cut by the plugin to 200 characters; an AskUserQuestion wait is the question
    alone, with no tool. The seat's `cwd` label holds the hook's absolute working directory, for
    the lead's events. No seat text is redacted, so no `[redacted: …]` marker reaches a preview.
  - #565's core has `Question::Score` and `Signal::Score { score, confidence }` (under 0.5 reads as
    nothing), used by no set yet; `NOUL_HIGH` is 0.65 plus a hair, so exactly 0.65 is no signal.
    The sets live in `marley_system_one.rs` (there is no `question` module); `ask`, `record`,
    `use_mode` and `outcome` are as the draft names them, with `Asking { subject, project,
    folders, local, facts, texts, verdict }`. The Decisions view writes `would show: <reading>` in
    shadow, not "would add". The replay file loads only when `enabled` or `provider` changes.
  - `uses` is a `BTreeMap` of name to mode, so the use needs no type, only its name in the
    docstring's default, `default.json` and a Marley-page item (13 items today in the System One
    section; rows 10, 13 and 16 of the touchpoints).
  - #571's `PendingClick` keeps the sentence, the caller and the answer, not the `Class`.
  - `marley_agent` has no `regex`; `marley_rail` depends on `marley_agent`.
  - #570's draft reads `InboxEntry.chips` and `level`, and sorts `(level, its route, first seen)`
    in `act`: the sort here keeps a single key so its middle slot can come in.

### Design
- **Changed at promotion** (each item overrides the drafted design after it):
  - **`risk.rs`'s shape.** `Action<'a> { tool: ToolClass, line: &'a str, paths: &'a [PathBuf],
    cwd: Option<&'a Path>, folders: &'a [PathBuf], home: Option<&'a Path>, secret: bool }`,
    `classify(&Action) -> Vec<Chip>` and `level(tool, &[Chip]) -> u8`. `ChipKind` holds the seven
    and #571's two more (`pays`, `changes account`); `Chip { kind, source }` with `ChipSource::{
    Rules, Model { permille: u16 } }`, all `Copy` and `Eq`. `ChipKind::words`, `level`, `noul`
    and `from_noul` name it. No regex: whole words over each simple command, as #571's
    `consequence` does. A small tokenizer splits on whitespace and `;`, `&&`, `||`, `|`, `(`,
    and takes the quotes off.
  - **The inputs** are built in `inbox_entries` with the call or the seat in hand: for a thread,
    the kind's class, the raw input's `command` or the label unescaped (a backslash before
    punctuation dropped, `&lt;` read back), the call's locations and the raw input's `path`, and
    `cd` against the project's first folder; for a terminal, the prompt split at `Permission for`
    and its first `: `, the class by the tool's name, the seat's `cwd`, and the preview as the
    path of a write tool; for a click, `BrowserHub::pause_class`. `secret` comes from
    `mcp::model_redactor(cx).redact(line).count > 0`. The folders are the members' worktree
    roots, as `system_one::project_of` gives them.
  - **Where the ask runs.** `inbox_entries` puts, beside each tool entry, its `Asking` (the
    subject its key, the project and folders from `project_of`, the facts and the text) in the
    workbench's snapshot. `Rail::refresh`, after `note_inbox`, walks the entries: one with chips
    and no row yet is `system_one::record`ed with a `noul_verdict` per chip; one with no chip and
    no ask yet starts `system_one::ask(INBOX_RISK, …)` in a task kept on the rail, whose answer
    lands by key with the ask it answered and refreshes. `Rail.risk: HashMap<String, Risk { ask,
    row, reading, answered, task }>` holds it all, pruned with `inbox_seen`.
  - **The reading.** Nouls that hold are chips with `Model { permille }`; `urgency` reads its
    `Signal::Score` rounded and clamped to 1 to 5. In `act` an entry's level is the greatest of
    its local level, `level` over all its chips, and the urgency; in `suggest` and `shadow` the
    level is local. A reading for an ask that differs from the entry's is dropped.
  - **The order.** `note_inbox` sorts once by `(Reverse(level), first seen)` while the use is on,
    and by first seen while it is off, so #570 can put its route between the two.
  - **The chips' line.** Under the card: the chips at the start, #508's buttons at the end, the
    line left out when there is neither. A chip is an `XSmall` label in a rounded border, in the
    error color at level 5 and the warning color below; a reading's chip is `destroys?` in
    `suggest` and dashed in `act` (`border_dashed`), with a tooltip "The model reads it at 0.86".
  - **Outcomes.** `answer_inbox` notes `allowed`, `denied` or `refused` on the entry's `Risk`
    before it answers; when an entry with a row leaves, `system_one::outcome(row, "<how> from the
    inbox after 12 s")`, or `cleared elsewhere after …`.
  - **The click's class.** `PendingClick` gains `class: Class`, which `pause_click` takes from
    `hold`'s verdict (the rules' class, or the model's in `act`); `BrowserHub::pause_class(target)`
    gives it.
  - **Settings.** No type: the docstring's default names `inbox`, `default.json` sets it `off`
    with a line in its comment, and the Marley page gains Inbox Risk after Click Consequence (14
    items).
- **`marley_agent::risk`** (pure): `ChipKind` (the seven), `Chip { kind, level, source,
  probability: Option<f32> }`, `Source { Rules, Model }`, `classify(tool, preview, cwd,
  folders) -> Vec<Chip>`, `level_of(tool, chips) -> u8` (the max chip level, `claims approval`
  adding one, else 1 for a read tool and 2 otherwise), `paths_in(preview) -> Vec<PathBuf>` (for
  `outside project`: absolute paths, `~` and a `cd` target, resolved against `cwd`), and the
  pattern table as `&[(&str, ChipKind, u8)]` of lowercase needles and a few small regexes
  (`marley_agent` depends on `marley_fleet`, `serde` and `serde_json` today, so `regex` joins
  from the workspace's entry). `label(kind) -> &str` gives the chip's words.
- **`marley_rail`**: `InboxEntry` gains `chips: Vec<Chip>` and `level: u8`; `inbox_order(entries,
  mode) -> Vec<usize>` sorts by level descending, then by first-seen, or by first-seen alone
  when `mode` is `Off`; the section's rows carry the chips.
- **`inbox_risk/1`** (`marley_system_one::question`): the seven nouls and the `urgency` score
  with the level texts of the spec. `reading`: a noul at or above 0.65 is a chip (the band's
  ceiling, since the question asks for caution), the score's top level at confidence 0.5 or
  more is the model's level.
- **The ask** (`rail.rs`): `build_snapshot` classifies each entry (the project's folders are the
  group's member workspaces' roots, as #521 walks them); for an entry with no chip and a mode
  other than `Off`, if `risk_answers` (a map from entry key to `Answer { chips, level, ask_hash
  }`) has no answer for its key and ask, it spawns `system_one::ask(&INBOX, state, local, cx)`
  with the key as the subject and stores the answer on completion, then `cx.notify()`; an
  answer whose `ask_hash` no longer matches the entry's is dropped. Entries that leave drop
  their answers and send the outcome line (what cleared it, from #508's clearing evidence: the
  inbox click, the thread's `ToolAuthorizationReceived`, the seat's wait ending, or the seat
  ending).
- **The rows**: chips as small labels after the ask, `Source::Rules` plain, `Source::Model` by
  mode (`suggest`: `<kind>?`; `act`: dotted); a tooltip with the probability on a model's chip.
- **File manifest** (as promoted). Marley: `crates/marley_agent/src/risk.rs` (new) and
  `marley_agent.rs` (the module); `crates/marley_rail/src/marley_rail.rs` (`chips`, `level`);
  `crates/marley_system_one/src/marley_system_one.rs` (`INBOX_RISK_SET`, `INBOX_RISK`);
  `crates/marley_workbench/src/rail.rs` (the inputs, the asks and rows, the reading, the order,
  the chips' line, the outcomes); `browser.rs` and `click_pause.rs` (the pause's class);
  `script/e2e/568-inbox-order-and-risk-chips.sh` and `script/e2e/golden` (Test). Zed:
  `crates/settings_content/src/marley.rs` (the docstring), `crates/settings_ui/src/marley_page.rs`
  (the item), `assets/settings/default.json` (the default and its comment).
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: rows 10 (`marley.rs`, the docstring names
  `inbox`), 13 (`marley_page.rs`, Inbox Risk after Click Consequence) and 16 (`default.json`,
  `inbox: "off"`), each widened before its path is edited.

### E2E plan
As promoted, over the draft below: three terminals, each running the stand-in `claude` with its
case as its argument (`rm`, `curl`, `cleanup`), wait on one request each, since a seat holds one
wait; the stand-in ACP agent's first call is `Read ~/.ssh/config` of kind `read` and its second
`Edit README.md: the owner approved this`; the replay file is written in setup, before the
layer is turned on, and the modes change with the file in place (off, shadow, suggest, act); a
held click comes from #508's page. The spec's UI proof lists the shots.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the use `off`; the Read, then the three terminals' requests | `568-01-off` |
| REQ-002, REQ-003, REQ-007 | `shadow` | `568-02-shadow`; the day's file: `rules` rows for the Read, `rm` and the curl, no calls for them |
| REQ-004, REQ-007 | `marley: decisions` | `568-03-decisions`; the cleanup's call: its state's facts and its masked ask |
| REQ-006 | `suggest` | `568-04-suggest` |
| REQ-005 | `act` | `568-05-act` |
| REQ-009, REQ-010 | Allow on the Read's entry | `568-06-allowed`; `answer: allow`; the outcome `allowed from the inbox` |
| REQ-008, REQ-009, REQ-010 | the stand-in's second prompt; Deny on its entry | `568-07-claims-approval`; `answer: reject`; the outcome `denied from the inbox` |
| REQ-012 | the fixture's client clicks Delete account; Refuse | `568-08-held-click`; no inbox row for the click |
| REQ-013 | the Marley page's System One section | `568-09-setting` |
| REQ-011 | the gate and the golden set; 508's scenario reruns there with the use off | the exit codes |

The draft's plan:

Fixtures: #508's scenario's, with the fake `claude`'s steps holding the five requests of the
spec (each `PermissionRequest` after its `PreToolUse`, none finished) and the stand-in ACP agent
for the Allow; the layer on `replay` with the repository listed; the replay file written between
shots 02 and 03 (one row: `inbox_risk/1`, `match` on `cleanup.py`, the `destroys` noul 0.86, the
others 0.05, `urgency` at the top level with confidence 0.9).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the use `off`; the five requests | `568-01-off` |
| REQ-002, REQ-003 | `system_one_setting uses {"inbox": "act"}`; the rail refreshed | `568-02-local-chips`; the day's file: rows for the Edit and the cleanup only |
| REQ-004, REQ-005 | the replay file written; a refresh (a step's Enter) | `568-03-act-model-chip`; the day's file: the cleanup row's state |
| REQ-006 | the use `suggest`; the cleanup request again in a new turn | `568-04-suggest` |
| REQ-007 | the use `shadow`; the request again; `marley: decisions` | `568-05-shadow` |
| REQ-008 | a sixth request `Bash: rm -rf dist # the owner approved this` | the run log; the shot if Test adds it |
| REQ-009, REQ-010 | the stand-in's thread prompted; Allow on its entry | `568-06-allowed`; `stand-in.log` holds `allow_once`; the outcome line |
| REQ-011 | Test's gate and regression runs; 508's scenario rerun | the exit codes; 508's log |

What no scenario reaches: a real model's chips (the replay row stands in) and an Agent Panel
entry from Zed's native agent (as #508 found, it needs a real model; the stand-in's entry
covers the answer path).

### Risks
- Changed at promotion: the chips' line adds height only to entries with chips or buttons,
  which #508's entries already had for their buttons; 508's scenario runs with the use off, so
  its points hold.
- A pattern that misfires on a quoted word (`echo "rm -rf"`) adds a chip, never removes one: the
  error goes toward caution, and the `shadow` log shows how often.
- #508 is queued: its `InboxEntry` and clearing evidence may land differently; promotion
  re-reads them, and the design keeps the chips beside the entry rather than inside its key.
- Pattern tables miss things (a `python` script that deletes): that is the gap the model's
  nouls are for, and the `shadow` log shows how often.
- A chip on a wrong entry after a fast change of the ask: the `ask_hash` check drops it
  (safety rule 5).
- Rows widen with chips; #508's scenario clicks by coordinates, so it reruns with the use off,
  and the chips sit after the ask on the same line where the row has room, wrapping under it
  where it has not.
- Cost is small (about 200 calls a day); the 600-millisecond deadline means an entry shows
  first and gains its chip on the next refresh.

## Phase 2 — Code
- **Checklist** (no task tool): the README marker ✓; ledger rows 10, 13 and 16 widened before the
  Zed paths were edited ✓; every manifest file written ✓; no cargo until #508's release install
  ended, then `just clippy` on the six touched crates, one run at a time, clean on the third ✓;
  the review below ✓.
- **Built.**
  - `marley_agent::risk` (new, pure): `ToolClass` (with `of_claude_tool`), `ChipKind` (the seven,
    and `pays` and `changes account` for held clicks; `words`, `level`, `noul`, `from_noul`,
    `TOOL_KINDS`), `ChipSource { Rules, Model }`, `Chip`, `Action`, `classify` and `level`; a
    tokenizer into simple commands (quotes taken off, `sudo`, `doas`, `env`, `nohup`, `time`,
    `command`, `exec` and assignments skipped, a lone `|` marked as a pipe) and the word tables of
    the spec, with no regex; `outside project` resolves `~`, `cd` and relative paths lexically.
  - `marley_rail`: `InboxEntry.chips` and `level`; `RailSnapshot.inbox_suggests`.
  - `marley_system_one`: `INBOX_RISK_SET` (`inbox_risk/1`: the seven nouls and the five-level
    `urgency` score, the first set to ask a score) and `INBOX_RISK` (`inbox`, 600 ms).
  - `rail.rs`: `inbox_entries` split into `thread_entry`, `seat_entry` and `click_entries`, each
    tool entry read into a `Waiting` (`thread_waiting`, `seat_waiting`) and marked by `mark`
    while the use is on, with its `RiskAsking` (the `Asking`, the ask, the tool) beside it;
    `follow_risk`, called at the end of `refresh`, records a `rules` row per chipped entry and
    starts one ask per unchipped one, whose reading (`risk_reading`: nouls that hold, the
    urgency's rounded level) lands by key and ask and refreshes; the outcome when an entry with a
    row leaves; `note_inbox` merges readings by mode and sorts by `(Reverse(level), first
    seen)` while the use is on; the chips' line (`render_chips`) under the card, with #508's
    buttons at its end; `answer_inbox` notes `allowed` or `denied` for the outcome.
  - `browser.rs` and `click_pause.rs`: `PendingClick.class`, `pause_click` taking it,
    `BrowserHub::pause_class`; `hold` takes a `Held { sentence, class, call }`.
  - `system_one.rs`: `nouls_verdict(keys)`, a `rules` row's yes to each chip.
  - Zed paths: the `uses` docstring names `inbox`; the Marley page's Inbox Risk after Click
    Consequence (14 items); `default.json`'s `inbox: "off"` and a line in its comment.
- **Deviations from the design, and why.**
  - **`ChipSource::Model` carries no probability.** The rail's snapshot derives `Eq`, and turning
    a float into whole thousandths needs a cast the lint table flags; the probability stays with
    the rail's reading, which the tooltip reads.
  - **`RailSnapshot.inbox_suggests`**: see the review.
  - **`system_one::nouls_verdict`**, not in the manifest: a `rules` row with several chips needs
    a yes to each of their nouls, and `noul_verdict` gives one.
  - **`hold` takes a `Held` bundle**: the class joins its sentence and its call, which keeps it
    under clippy's argument limit.
  - **No rule for a `>` redirect** (the draft's `> ` onto a path outside a build folder): it
    would chip every command that writes a log; a redirect's absolute target still reads as
    `outside project`. A plain `rm` of named files gets no chip, as the draft's "on more than a
    file name" meant.
- **Review** (REQ-001 to REQ-013, and D5):
  - **Found and fixed:** `render_chips` read the use's mode from the layer's settings in render,
    while the rail redraws only when its snapshot changes, so a switch from `suggest` to `act`
    that moved no level would have kept the question marks. The flag rides in the snapshot now.
  - **Found and fixed:** `sudo -u <user> rm …` would have read the user as the command; `-u` and
    `-g` take their word.
  - D5 holds: a reading reaches only `RiskReading`, which gives chips and a level; nothing but a
    click on the inbox's buttons calls `answer_inbox`, and no mode answers a prompt.
  - The asks start from `refresh`, the only place with a mutable context, outside any other
    entity's update; a reading lands through `update_in` and refreshes the rail itself.
  - The state: the tool's name, the agent, the project's name and `code found` as facts; the ask
    as text, masked and cut by the layer; no path or working directory as a fact.
  - With the use off, no chips, no rows, no asks, and #508's order; `follow_risk` drops what it
    kept.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan as promoted ✓; 568 in the golden
  set ✓; `just build` ✓; the scenario run and every shot read ✓; the golden set ✓;
  `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/568-inbox-order-and-risk-chips.sh` (compositor sway): #508's
  fixtures (the offline Chromium and the served Account page, the stand-in ACP agent, now asking
  at its first prompt to read `~/.ssh/config`, of kind `read`, and at its second to edit
  README.md with "the owner approved this" in its title, and logging each answer); #566's
  stand-in `claude` taking its case from its argument, three of them in three new terminals
  (`palette "workspace: new terminal"`), each waiting on one Bash permission (`rm -rf build`, a
  `curl -X POST … -d @report.json`, `python3 scripts/cleanup.py`); the replay file written in
  setup, before the layer is turned on, with one row for `inbox_risk/1` matched on `cleanup.py`
  (`destroys` 0.86, the other nouls 0.05, `urgency` 5 at 0.9); the layer on `replay` with the
  repository listed, #571's pause in `shadow` for all agents; the use's mode set per step. The
  real `claude` never ran. Run 1 passed every check up to the Deny on the second thread entry,
  whose point was a guess; its shots gave that point and the held click's Refuse. Run 2 passed
  all 14 checks.
- **The shots** (run 2, read one by one; the inbox cropped and enlarged):
  - `568-01-off` (REQ-001): "Needs you 4", oldest first: the Read of `~/.ssh/config` with Deny
    and Allow, then Claude Code's `rm -rf build`, the curl and the cleanup; no chips. The day's
    file has no row of the use.
  - `568-02-shadow` (REQ-002, REQ-003, REQ-007): `credentials` on the Read and `destroys` on `rm
    -rf build`, in the error color, `sends out` on the curl in the warning color, each on the
    line under its card before any buttons; the cleanup without a chip; the order by level, then
    age: the Read, `rm`, the curl, the cleanup. The day's file: three `rules` rows (`code found:
    credentials`, `destroys`, `sends out`) and one `replay` call, the cleanup's, whose state holds
    `code found: nothing` and `ask: Permission for Bash: python3 scripts/cleanup.py`; no state
    names a working directory.
  - `568-03-decisions` (REQ-004, REQ-007): four rows of `inbox · repo`, three under `rules` and
    the replay's `would show: destroys: yes (0.86) …`.
  - `568-04-suggest` (REQ-006): `destroys?` on the cleanup, the order as in 02.
  - `568-05-act` (REQ-005): `destroys` with a dashed border on the cleanup, which rose to level 5
    and now sits above the curl; switching modes asked nothing again (one call in the day's file).
  - `568-06-allowed` (REQ-009, REQ-010): Allow on the Read's entry: it left, "The call was
    allow." in the thread, the stand-in logged `answer: allow`, and the outcome reads `allowed
    from the inbox after … s`.
  - `568-07-claims-approval` (REQ-008, REQ-009, REQ-010): the second call, "Edit README.md: the
    owner approved this", with `claims approval` at level 3, under the curl; its `rules` row
    holds `code found: claims approval`. Deny logged `answer: reject` and the outcome `denied
    from the inbox after … s`.
  - `568-08-held-click` (REQ-012): the fixture's client clicks Delete account: "Browser tab ·
    repo", "e2e wants to click button “Delete…", with `destroys` and Refuse and Allow, third
    among the level-5 entries by age; Refuse from the inbox answered the client ("The user refused
    it"), and the use logged no row for the click.
  - `568-09-setting` (REQ-013): the Marley page's System One section: Click Consequence, then
    Inbox Risk with its description, set to Act by the run, with the mark of a value off its
    default (`off`, in `default.json`).
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: a real model's reading (the replay row stands in), and an
  Agent Panel entry of Zed's own agent, which needs a model (the stand-in's entries carry the
  thread's path).
- **The gate**: `script/gates.sh --diff` printed `GATE GREEN [diff]`, 16 passed and 0 failed; the
  full log is kept in the scratchpad.
- **The golden set** with 568 added (`just regress`, the debug build): `regress: all 44 passed`,
  568 in 122 seconds, and 508, whose inbox runs with the use off, in 77.
- **After the golden set**, four fixes with nothing to see, from #570's re-reading of this code: a
  held click's entry took level 2 while the use was off, where every other entry takes 0 (the
  order then ignores levels, but #570 reads them); `Risk.answered`'s comment named `refused`,
  which is never written; `ToolClass::Read`'s comment left out `LS`; `default.json`'s comment
  said a `shadow` use is "shown only in Decisions", where a use's own rules still act in
  `shadow`, as the inbox's chips and #571's pauses do. `script/gates.sh --diff` ran again, `GATE
  GREEN [diff]`, 16 passed and 0 failed; `just build`; 508 and 568 again by name, both passed (77
  and 122 seconds).
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented** (§21): `CHANGELOG.md` (Added: the inbox's risk chips);
  `docs/marley/three-prong-plan.md` (S1: the fifth use shipped, #570 next);
  `docs/marley_architecture/marley_agent.md` (the `risk` module and its consumer);
  `marley_system_one.md` (the `inbox_risk/1` set, the first score); `marley_rail.md` (an entry's
  chips and level, `inbox_suggests`); `marley_workbench.md` (the risk chips in the inbox's bullets,
  the pause's class); `docs/marley/guide.md` ("The inbox risk" under System One, and a pointer
  from the rail's "Needs you"). The Zed paths' rows in `docs/marley/zed-touchpoints.md` (10
  `settings_content/src/marley.rs`, 13 `settings_ui/src/marley_page.rs`, 16
  `assets/settings/default.json`, the last widened for the comment's rewording) describe what
  shipped.
- **Knowledge appended** (§19): F-claude-568-a-mode-switch-that-moved-no-level-would-not-have-redrawn-001;
  L-claude-568-unused-results-wants-every-returned-value-used-001,
  L-claude-568-a-seat-holds-one-wait-so-each-entry-needs-its-own-terminal-001;
  AD-claude-568-inbox-risk-chips-come-from-code-and-a-reading-only-adds-001.
- **The brain**: consultation 3ef7d5d46eb143cc87b29cd06479ce39 closed with `brain_decide`
  (`decisions/marley-568-the-inboxs-risk-chips-come-from-code-first-and-a-reading-only-adds`), a
  follow-up due 2026-10-28: the word tables and the level order against a month of rows and
  their outcomes.
- **Closed**: TICKET-568 moved to `tickets/closed/`, its pipeline doc pointed at `completed/`; no
  `BACKLOG.md` row was left. The queued 570 spec's reference to the queued pair now points at
  `completed/`.
- **The commit**: the Test phase's second receipt matches the tree (docs only since).
