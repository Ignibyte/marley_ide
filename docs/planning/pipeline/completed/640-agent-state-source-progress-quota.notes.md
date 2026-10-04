# A harness session's state source, progress and quota on its rail row — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-640-agent-state-source-progress-quota.md
- **Pipeline spec:** 640-agent-state-source-progress-quota.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-02)
- **Request:** rustal-harness's MREQ-005 to MREQ-007 (its `docs/planning/MARLEY_REQUESTS.md`,
  dated 2026-10-02) and its D164: Marley reads `state.source`, `progress.percent` and
  `progress.activity`, and `quota.KIND.percent_used` with `quota.KIND.resets_at_ms` as labels on
  the pinned `marley_fleet` session envelope, draws `detected` weaker, and never lets it drive
  approvals. Chad's words: the facts go "on each agent's row" (the lead's brief); on herdr, "for
  herdr we need to reach over to the Rustal Harness agent and we want anything related to that
  built there", which sent detection, progress and quota to the harness and left Marley the
  drawing; on the contract, "We would want to do what we have borrow ideas" (D164); and "Queue all
  three" for this batch (2026-10-02).
- **Classification / tier:** feature, prong 2 C1 (the harness's read side), S to M. Marley crates
  only (`marley_workbench`); no Zed crate, no setting, no new touchpoint row.
- **Recall (§18.3):**
  - AD-claude-534-the-harness-is-followed-by-polling-in-a-section-outside-the-rails-model-001: the
    section sits outside the rail's model and draws each `marley_fleet::Session` straight to a
    row; this ticket changes only that row and the inbox's source of harness entries.
  - F-claude-534-the-harnesss-routing-note-hid-the-options-001: a producer's annotation shown as
    content filled the inbox's line. The same care here: a label's value is checked before it is
    drawn (D6), and the end slot keeps the weaker mark visible when the line is cut.
  - AD-claude-566-the-stop-kind-is-rules-first-and-rides-on-the-seat-001: Marley already reads a
    seat's labels, and their keys are constants in the crate that reads them
    (`marley_agent::claude_events`), not in `marley_fleet` (D2).
  - AD-claude-607-the-fleet-contract-lives-in-marley-sdk-001: `marley_fleet` is the session
    envelope and `marley_sdk` owns `marley.work/v1`, so the Fleet panel's follow-up types go in
    `marley_sdk`, not here.
  - AD-claude-fleet-rail-quiet-no-transport-until-brain-001 and F-claude-609 (a drawing that
    trusted an optional field): show no invented state; a value that does not parse is left out.
  - L-claude-534-zeds-mcp-client-sees-no-server-exit-001: every call is bounded at 5 s, and an
    `isError` whose text starts `resync_required` re-seeds; the stand-in answers each call at once.
  - PR-claude-a-labels-values-decide-which-redaction-rule-owns-it-001: write down the values a
    label carries before a rule touches it; done for the account (an `@` hides it, D7).
  - The completed pipelines 534 (the section, its stale row, its real-harness scenario), 632 (the
    embedded runtime; `marley.harness` wins over it), 633 (a stand-in stdio MCP server in Python),
    607 to 611 (the Fleet panel and its stores).
  - Brain: `rusty-cli brain search` for the harness's state source and quota found nothing; the
    herdr pages under `research/agent-workspaces/` are research only. The Planner's `brain_ask` at
    promotion is owed.
- **Discovery (an Explore map of the rail, the Fleet panel and the helpers, and direct reads):**
  - `crates/marley_fleet/src/session.rs:73-76`: `labels`, a sorted `BTreeMap<String, String>`
    documented as uninterpreted; `:80` `capabilities` (MREQ-004). `reducer.rs:23-40` and
    `:169-180`: an `Upsert` carries and replaces `labels`; a `StateChange` leaves them.
  - `crates/marley_workbench/src/harness.rs`: `Harness` (108-121) with `seats` and `minute`;
    `seats()` (149-154); `follow` (408-442); `connected` (446-506), whose minute bump (492-504)
    runs only while some seat is `working`; `seed` (509-523) deserializes `fleet_snapshot`'s
    structured content into a `FleetSnapshot`; `answer` (579-598) maps an `isError` starting
    `resync_required` to a re-seed; `shown_prompt` (602-607); `now_ms` (617); `HarnessView`
    (645 on) shows output lines only.
  - `crates/marley_workbench/src/rail.rs`: `render_harness` (4031-4098), its section-wide `stale`
    (4051-4052) and its rows (4060-4063); `harness_row` (6121-6171): the state's word and dot
    colour (6128-6135), the question or `no update in N m` (6137-6145), `{line} · stale`
    (6146-6150), the dot and title muted when stale (6151-6158), `row_card` (6160-6169);
    `harness_entries` (6175-6202), called at 7139, the only path from a harness seat to the inbox;
    `RowLine` (7921-7937) with its `state` end slot, drawn `flex_none` (8019); `row_card` (7970
    on), whose height is `h_11` for one line and `rems(3.5)` for more (7981); tooltips by
    `Tooltip::text` (4092 on the section's header).
  - Every other `row_card` caller passes at most two lines (the terminal rows' subtitle and
    activity at 5030; the thread, browser and port rows one), so a three-line height touches only
    harness rows.
  - Harness seats reach only the section, the inbox and `HarnessView`: no desktop notification,
    push or attention order reads them (`Harness::seats` callers: `harness.rs` 496, 670, 679, 707;
    `rail.rs` 4060, 6176).
  - The rail's weaker idioms (Explore): stale as muted dot, muted title and `· stale` (above);
    `marley_rail::Reporting` (`marley_rail.rs:787-798`) changes order and a collapsed project's
    word, never a row's look; a `?` after a reading's guess (`marley_agent/src/stall.rs:263-299`,
    `stop_kind.rs:330-338`); a dashed chip for a model's inbox chip (`rail.rs:1847-1896`); a closed
    project's `Color::Disabled` and `opacity(0.5)` (`rail.rs:4341-4347`). No italic anywhere.
  - The Fleet panel: `read_providers` (`fleet.rs:449-512`) reads the pseudo provider, `mcp` and
    `http` stores of `marley.work/v1` (`fleet_providers.rs`) and the host collector; nothing there
    or in `marley_sdk` names `marley_fleet` or the harness. Its row (`render_agent`, `fleet.rs:
    790-885`) shows runtime, name, work item, phase and a state chip; tokens only in the snapshot
    (`fleet.rs:985-1000`). So harness sessions are not on it (Out).
  - Helpers: `marley_rail::waited_words` (`marley_rail.rs:466-473`: `now`, `3 m`, `1 h 5 m`);
    `fleet::how_long` (`fleet.rs:1207-1215`); `{:.0} %` for a percent (`fleet.rs:1150`);
    `ui::Indicator` (dot, `border_color`); `ui::ProgressBar` (`fleet::meter`, `fleet.rs:1177-1204`).
  - The harness, read not built: `docs/FLEET.md`'s label table (`kind`, `generation`, `reason`,
    `capability.NAME`, `ticket`, `phase`, `gate.NAME` and the rest; none of the three yet);
    MREQ-005's status "the harness sends no source yet"; TICKET-091 plans the reports; MREQ-007
    names no label for the account's identity.
- **Decisions:** D1 to D8 in the spec.

### Design
- **`marley_workbench::harness`** (Marley crate):
  - Label constants beside the module's other constants: `state.source`, `progress.percent`,
    `progress.activity`, the `quota.` prefix with the `.percent_used` and `.resets_at_ms` ends, and
    `quota.account`.
  - `pub(crate) struct Signals { source: Option<StateSource>, progress: Option<Progress>, quota:
    Vec<QuotaWindow>, account: Option<String> }` and `pub(crate) fn signals(labels:
    &BTreeMap<String, String>) -> Signals`, pure. `StateSource::{Protocol, Reported, Detected,
    Other(String)}` with `declared()`, true for the first two; an absent label is `None`,
    drawn and routed as declared. `Progress { percent: Option<u8>, activity: Option<String> }`,
    `None` when both are missing. `QuotaWindow { kind, percent_used: u8, resets_at_ms:
    Option<u64> }`: a KIND is the text between `quota.` and `.percent_used` (it may hold dots), a
    window with no valid percent is dropped, and the list is most used first, then by kind.
    `percent(&str) -> Option<u8>`: an `f64` parse, finite, 0 to 100, rounded. The account is kept
    only without an `@`.
  - The minute bump in `connected` also runs while some seat's shown reset lies ahead (REQ-013).
- **`rail.rs`** (Marley crate):
  - `harness_row` reads `signals(&seat.labels)`. Weaker when the source is present and not
    declared: the dot `Color::Muted` (as when stale), and the first line's end slot holds the
    source's word (`detected`, or the other word as sent, cut to 12 characters). The title keeps
    `Color::Default` unless the section is stale.
  - A progress line when sent: `{percent} %` and the activity's first line joined by ` · `, cut at
    the end. A quota line when a window is valid: text the most-used window's kind, end slot
    `{percent} % · resets in {words}` (or the percent alone without a reset), so the kind is cut
    before the numbers.
  - `until_words(ms)`: `marley_rail::waited_words` under a day, `{d} d {h} h` beyond, `now` for a
    reset past or under a minute away (`resets now`).
  - A tooltip when the seat carries any of the labels: the source's sentence (`State from the
    agent's protocol`, `State reported by the agent`, `State detected from its screen: shown, not
    acted on`, `State source "<word>": not known, not acted on`), each window's line most used
    first, and `Account: <name>`.
  - `harness_entries` skips a seat whose source is present and not declared.
  - `row_card`'s height by line count: `h_11` for one, `rems(3.5)` for two, `rems(4.5)` for three.
- **File manifest.** Marley: `crates/marley_workbench/src/harness.rs`,
  `crates/marley_workbench/src/rail.rs`; `script/e2e/640-agent-state-source-progress-quota.sh`
  (Test phase). Zed crates: none, so no `docs/marley/zed-touchpoints.md` row. `marley_fleet`,
  `marley_rail` and `marley_sdk` unchanged.
- **The stand-in** (written by the scenario, Test phase): `$E2E_WORK/bin/rh-stand-in`, Python,
  stdio, one JSON-RPC message per line as #633's. It answers `initialize` (tools capability), skips
  notifications, and on `tools/call`: `fleet_snapshot` gives `{instance_id, cursor, seats}` from
  `$E2E_WORK/fleet.json` (as `structuredContent` and as text); `fleet_events` gives `{events: [],
  next_cursor}` while the fixture is unchanged, and after a rewrite one `upsert` per seat with its
  labels, a `question_raised` for each waiting seat with a question, and the next cursor;
  `session_read` gives `{result: "accepted", value: {lines: ["stand-in"]}}`. The fixture's
  `last_event_ms` is the time it was written, and its resets are computed from then (95 m, 3 d 4 h).
  `marley.harness` is `{"command": "<bin>/rh-stand-in", "args": []}` through `profile_setting`,
  and `marley.no_update_after_minutes` is 0, so no working row turns quiet during the run.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-002, REQ-003 | the six sessions: watcher and asker detected, odd `guessed`, build protocol, review reported, plain no labels | `640-01-rows` |
| REQ-004 | asker (detected) waits on "Allow the edit?"; review (reported) on "Merge the branch?" | `640-01-rows` (asker's row), `640-02-inbox` (`Needs you 1`, review only) |
| REQ-006, REQ-007 | build `40 % · Running the tests`; review `75 % · Waiting for a merge decision` | `640-01-rows` |
| REQ-008 | build `five_hour` `62 % · resets in 1 h 35 m` | `640-01-rows` |
| REQ-009 | odd: percent 140, quota percent `lots`, a two-line activity: only the activity's first line | `640-01-rows` |
| REQ-010 | the pointer on build's row, settled for the tooltip | `640-03-tooltip` (source, `five_hour` then `seven_day`, `Account: work`) |
| REQ-005, REQ-011 | the fixture rewritten: build 80 % `Writing the summary`, `five_hour` 70 %; asker `reported`; settle 5 | `640-04-moved` (`Needs you 2`) |
| REQ-012, REQ-013 | review: the reading keeps only the named keys and drops an `@` account (odd carries `someone@example.com`, never shown); the minute bump | the diff |

The rows' places come from the first shot, as #534's did (`BUILD_Y`, `0` for the group shot
only). What no scenario can reach: a real harness sending these labels, since it sends none yet
(MREQ-005's status; TICKET-091 plans the reports). The stand-in speaks the read tools as `rh mcp`
does at the harness's state of 2026-10-02, and the Phase 3 entry says so.

### Risks
- MREQ-007 names no label for the account's identity; `quota.account` is this draft's reading.
  The promotion re-reads TICKET-091's report contract and `FLEET.md`'s table for the key the
  harness settles on, and for any change to the three names.
- A producer may send `42%` or `42.5`: the second is shown as 43, the first is left out (D6). If
  the harness's reports carry a unit, the promotion widens the parse rather than the drawing.
- An unknown source is drawn weaker even if it later proves declared (say a `hook`); Marley shows
  its word, and the next ticket that learns it adds it to `declared()`.
- Three lines make a harness row taller than any other rail row; six sessions must still fit the
  scenario's 1000 px with the inbox above them.
- The stand-in exercises both the seed and the `upsert` path, but not a real harness's event
  order; the review checks that a `state_change` keeps the labels (it does, `reducer.rs:184-189`).

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline, cargo idle, `/mnt/fast` 255G free.
- **Seams re-verified** at 6540353632: `harness.rs` `Harness` `:108-121` (with `minute` `:114`),
  `seats()` `:149-154`, `connected` `:446-506` (the minute bump `:492-504`), `seed` `:509`,
  `answer` `:579`, `shown_prompt` `:602`, `now_ms` `:617`; `rail.rs` `render_harness` `:4031`,
  `harness_row` `:6121-6171`, `harness_entries` `:6175` (called at `:7139`), `RowLine` `:7921`,
  `row_card` `:7970` (`rems(3.5)` for more than one line); `marley_fleet` `reducer.rs:169-184`
  (an `Upsert` replaces the labels, a `StateChange` keeps them); `marley_rail::waited_words`
  `:466-473`; `fleet::how_long` `fleet.rs:1207`; `one_line` `rail.rs:6615`.
- **The harness, re-read:** its seats now publish what the draft expected. `docs/AGENT_SEATS.md`
  and D174 confirm the three names and `quota.account` (a digest of the login's directory, never
  an email address, so D7's `@` rule only guards a producer that breaks the contract); the
  percent comes "to one decimal" (D6 rounds it to a whole number); at most eight windows. New
  since the draft: `state.source` `runtime`, which the harness sets on a seat it restarts while
  that seat reads `starting` (D173). It is the runtime's own knowledge, so D3 now counts
  `runtime` as declared, and the scenario gains a seventh session, `restart` (runtime,
  starting), drawn declared. The harness also publishes `usage.*` token labels; they belong to
  the Fleet panel follow-up (Out), as tokens on a row would crowd it.
- **Brain:** `brain ask` (consultation `4c74b4bebee441b3a1ec5ed583dcbf9c`) returned due
  follow-ups on other work only; `brain search` finds nothing on a state source. Nothing new.
- **The stand-in stays:** a real `rh` now sends these labels, but only for an agent that reports
  through `rh report` in a harness terminal; a scenario that drives that would test the harness,
  not Marley's drawing. The stand-in serves the label table above.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION.md §3, §7, §14, §18, §19, §20, the templates, and #633's pair for shape.
- [x] Read MREQ-005 to MREQ-007 in full, the harness's D164, TICKET-091, `HERDR.md` feature 9 and
      `FLEET.md`'s envelope; the design note `herdr-and-hermes-2026-10-02.md` with Chad's answers.
- [x] Recall (§18.3): the four ledgers grepped (harness, fleet, labels, stale) and the completed
      pipelines 534, 566, 607 to 611, 632, 633; a brain search.
- [x] Discovery: an Explore map (§18.2) of the rail's weaker idioms, the Fleet panel's sources and
      the helpers; direct reads of `harness.rs`, `harness_row`, `row_card`, `marley_fleet`.
- [x] Decided the surfaces: the Harness section's rows in this slice; the Fleet panel Out, since no
      harness session reaches it, with its follow-up named.
- [x] Reference (§20) and Prior art's three legs filled; Warp's source not read.
- [x] Spec, notes and ticket doc written; no other file touched, no cargo run.

## Phase 2 — Code (2026-10-04)
- **Built, to the manifest (Marley crate only, no Zed path, no ledger row):**
  - `harness.rs`: the label constants (`state.source`, `progress.percent`, `progress.activity`,
    the `quota.` prefix with `.percent_used` and `.resets_at_ms`, `quota.account`);
    `StateSource { Protocol, Reported, Runtime, Detected, Other }` with `declared()` (the first
    three) and `word()`; `Progress`, `QuotaWindow`, and `Signals::of(&labels)`, pure, which keeps a
    source only when non-empty, a percent only as a finite number from 0 to 100 (`percent_of`,
    rounded; the cast carries a justified `#[allow]`, as `marley_browser`'s roundings do), an
    activity's first trimmed line, each window whose percent parses (most used first, then by
    kind), and the account only without an `@`; `declared()` (no source is declared),
    `is_empty()`, `resets_after(now_ms)`. The minute bump in `connected` now also runs while some
    seat shows a reset ahead (REQ-013). The module doc says what the labels are.
  - `rail.rs`: `harness_row` reads `Signals`; a weaker state mutes the dot (the title keeps its
    colour); the first line's marks (the source's word, up to 12 characters, and `stale`) moved
    into `RowLine`'s end slot, so a long question is cut before them (before, `· stale` sat in the
    text and could be cut away); a progress line (`40 % · Running the tests`, either part alone);
    a quota line with the most-used window's kind as text and `62 % · resets in 1 h 35 m` in the
    end slot; a tooltip, `Tooltip::with_meta`, with the source's sentence as the title (the
    session's title when no source is sent) and each window, then the account, as the meta;
    `used_words` and `reset_words` (`resets now` under a minute or past, `waited_words` under a
    day, `N d N h` beyond). `harness_entries` skips a seat that is not declared. `row_card` is
    `rems(4.5)` tall for three lines under the title.
- **Deviations:** (1) the scenario was written and run in this phase, before the gate, as #642's
  was: its first run placed build's row (y 460) so `BUILD_Y` is set before the receipt binds the
  file, and its second run took all four shots (read in Phase 3). (2) The tooltip names the
  windows as the row does (`five_hour 62 % · resets in …`), not "% used", one wording in both
  places through `used_words`. (3) The scenario's terminal gets a HOME of its own with `PS1='$ '`,
  so the user's prompt is not drawn.
- **Review of the diff** against REQ-001 to REQ-013: a source absent is declared (today's
  harness, #534's rows unchanged: REQ-003); `runtime` declared (D3 at promotion); the inbox's only
  harness path is `harness_entries`, now gated (REQ-004), and an `Upsert` replaces the labels, so
  a seat turning declared enters the inbox on the next refresh (REQ-005); a `StateChange` keeps
  the labels (`reducer.rs:184`); no `quota.` key but the window pair and the account is read, and
  an `@` account is dropped in `Signals::of` (REQ-012); the countdown redraws each minute while a
  reset is ahead (REQ-013). No entity is read or updated during an update: `Signals::of` is pure,
  and the tooltip closure owns its strings. Provenance: Marley's own rail idioms; herdr and Orca
  read only through the design note's maps. No defect found.
- **Checks:** `cargo clippy -p marley_workbench --all-targets -- -D warnings`: three rounds. The
  first failed to compile (a local `percent` shadowed the function `percent`, renamed
  `percent_of`), the second had four lints (`is_empty` could be `const`; a `format!` collected
  into a `String`, now a fold; two `match`es on `resets_at_ms`, now `used_words`'s
  `map_or_else`), the third clean. `just build` and two scenario runs green.
- **Gate, run 1:** RED on gate:14 alone: the module doc linked `[`Signals`]`, a `pub(crate)`
  item, from a public module's doc (`rustdoc::private_intra_doc_links`). Now plain code text.
  The other 16 passed. Run 2 follows.
- **Gate, run 2:** `GATE GREEN [diff]`, 17 passed, the receipt written.

## Phase 3 — Test (2026-10-04)
- **Build and run:** `just build` on the gated tree, then `just e2e
  script/e2e/640-agent-state-source-progress-quota.sh` (`compositor sway`), exit 0. The two runs
  in the Code phase (one for build's place, one with every shot) showed the same. Focus report:
  "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and did not reload
  it"; the run's sway stopped with its Marley. Every shot is Marley's window; none deleted.
- **The shots, read** (the Harness section cropped and enlarged three times for 01):
  - `640-01-rows` (REQ-001 to REQ-003, REQ-006 to REQ-009): HARNESS connected, seven rows.
    build: a blue dot, `working`, `40 % · Running the tests`, `five_hour 62 % · resets in 1 h 35 m`
    (three lines; its `seven_day` 31 % is not the most used; its `usage.input_tokens` not shown).
    review: a yellow dot, `Merge the branch?`, `75 % · Waiting for a merge decision`. watcher: a
    muted dot, `working · detected`. asker: a muted dot, `Allow the edit? · detected`. plain: `idle`,
    as #534 draws it. odd: a muted dot, `working · guessed`, `Reading the logs` (the activity's
    first line; the percent 140, the quota `lots` and the `@` account left out). restart: `starting`,
    no source word (runtime declared). Every title in its normal colour.
  - `640-02-inbox` (REQ-004): "Needs you 1": review · Harness, `Merge the branch? (yes, no)`;
    asker's question is not there.
  - `640-03-tooltip` (REQ-010): the pointer on build's row: "State from the agent's protocol",
    then `five_hour 62 % · resets in 1 h 35 m`, `seven_day 31 % · resets in 3 d 4 h`, `Account:
    work`, most used first.
  - `640-04-moved` (REQ-005, REQ-011): five seconds after the fixture was rewritten: build `80 % ·
    Writing the summary`, `five_hour 70 % · resets in 1 h 35 m`; asker, now `reported`, with a
    yellow dot and no word, and "Needs you 2" with `asker · Harness`, `Allow the edit? (allow,
    deny)`.
  - REQ-012 and REQ-013: the review (Phase 2): only each window's pair and the account are read;
    odd's `someone@example.com` is never drawn (01, and odd's tooltip would name only its source);
    the minute bump runs while a reset is ahead.
- **Not reached:** a real rustal-harness sending these labels for a reporting agent (the stand-in
  serves the label table of its `AGENT_SEATS.md` and D174); the countdown crossing a minute while
  watched (the review covers the bump).
- **Pre-existing — not in scope:** none.

## Phase 4 — Complete (2026-10-04)
- **Documented (§21):** `CHANGELOG.md` Added ("A harness session's source, progress and quota on
  its row"); `docs/marley_architecture/marley_workbench.md` (the harness section: `Signals`, the
  row's lines and marks, the tooltip, the inbox's rule, the bump, `row_card`'s height);
  `docs/marley/three-prong-plan.md` (the C1 row); `docs/marley/guide.md` (the Harness section);
  `docs/marley/fleet-contract.md` (Later: the Fleet panel's three optional fields). No Zed path,
  so no `zed-touchpoints.md` row. The in-app guide and the walkthrough have no harness section.
- **Knowledge (§19):**
  `AD-claude-640-a-harness-sessions-labels-are-drawn-and-only-declared-states-act-001`;
  `L-claude-640-a-private-item-linked-from-a-public-doc-fails-only-the-docs-gate-001`,
  `L-claude-640-a-stand-in-harness-serves-the-fleet-tools-from-a-fixture-001`. No `F-…`.
- **Brain:** `brain decide` on consultation `4c74b4bebee441b3a1ec5ed583dcbf9c` (follow-up
  2026-10-25).
- **Closed:** the ticket to `tickets/closed/`, its link at `completed/`; no BACKLOG row left; the
  pair archived.
