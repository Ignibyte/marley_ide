# Rusty's Server, Secrets and Skills as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-940-rusty-server-and-forms-as-the-board.md
- **Pipeline spec:** 940-rusty-server-and-forms-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11, Rusty's boards).
- **Classification / tier:** feature, M. Marley crates only (`marley_workbench`, `marley_rusty`).
  The ticket's list was L; MoreTabs and MoreForms are split off (D6), as the list allowed.
- **Recall (§18.3):**
  - #894's scenario: the wrapper that fails while a flag file exists, and the backoff's timing
    (each try up to 5 s, waits doubling from 1 s); its Try Now place came from the first run.
  - #666's scenario: a keymap entry `zed::OpenSettingsAt {"path": "marley.rusty.server"}` opens
    the Server page, which the settings search does not list; `semantic.json` and
    `settings.json` in the stand-in's state.
  - #876's scenario: skills in `$(store)/.claude/skills` and `$(store)/staging`, scripts beside
    them, made-up secrets.
  - L-claude-565 (a scenario reaches the user's own keyring): the Secrets PIN is the stand-in's,
    held in its state folder, never the system keyring.
  - F-claude-831 (a title-bar view read the workspace inside its update) and PR-claude-831: the
    once-a-second tick is a task on the view, which reads only the `Rusty` global.
  - No failure names `rusty.rs`'s page or `skills_tab.rs`.
- **Discovery:** `crates/marley_workbench/src/rusty.rs` (`Rusty` global fields, `keep`,
  `connected`, `RustyServerView::render`, `settings_section`, `known_row`);
  `crates/marley_rusty/src/settings.rs` (`KnownSetting`, `KNOWN`, `SemanticStatus`);
  `crates/marley_workbench/src/rusty/secrets_tab.rs` (the unlock's lifetime, the locked and
  unlocked blocks); `crates/marley_workbench/src/rusty/skills_tab.rs` (the list's read, rows,
  scans, the New skill form).
- **Decisions:** D1–D6 in the spec.

### Design
- `Rusty` gains `answered_at: Option<Instant>` (set in `connected` after start and after each
  ping) and `retry_at: Option<Instant>` (set in `keep` before the wait, cleared after it).
  Accessors `answered_at()` and `retry_at()`; the page computes seconds from `Instant::now()`.
- `RustyServerView` gets a `_tick: Task<()>` that notifies once a second (the background
  executor's timer) while the view lives; the Settings window drops the view when the page
  closes.
- The status card: a `v_flex` with `border_1`, `rounded_md`, `p_3` and four `fact(label, value)`
  rows (a muted label column of fixed width, the value). State dot: `Indicator::dot()` in success
  or error. Program: the path from `via` and the version from `server`; `State::Connected` keeps
  both strings today, so the card splits them for the board (or `State::Connected` gains named
  fields `program` and `transport` if that reads better).
- `render_dropped` (Rusty's tabs) appends " Trying again in N s." to the reason when `retry_at`
  is set; the tabs that show it tick with the same one-second task while dropped.
- Dropped: the reason (error colour), "Trying again in N s" from `retry_at`, and
  `Button::new(…, "Try Now")` calling `try_now(cx)`.
- Brain search: `SemanticStatus::total()` = `stats.pages + stale`, and `progress()` as a share; a
  bar drawn like System One's Today bar (`system_one_calls.rs:275-288`), `text_accent` on
  `element_background`.
- `KnownSetting` gains `name: &'static str`; `known_row` draws `name` (default colour), `about`
  (muted), `key` (mono, XSmall), and the field at the right with the fallback as its muted
  placeholder (the field already uses the fallback as placeholder).
- Secrets: the tab keeps `locks_at: Option<Instant>` from the unlock; the unlocked line formats
  minutes (rounded up) or seconds; a one-second tick while unlocked (the tab already locks on
  expiry, so it may have a timer to reuse).
- Skills: after `SKILL_LIST`, the tab calls `SKILL_SCAN` for each pending skill (in sequence, in
  the same task) and keeps `scans: HashMap<String, ScanResult>`; `render_skill_row` draws the
  second line and the buttons (they call the detail pane's `approve`, `approve_anyway` and
  `reject` methods with the skill's name). `render_script_row` adds `Button` "Run" for an active
  script, calling the same function the pane's Run calls.
- File manifest (all Marley crates): `crates/marley_workbench/src/rusty.rs`,
  `crates/marley_rusty/src/settings.rs`, `crates/marley_workbench/src/rusty/secrets_tab.rs`,
  `crates/marley_workbench/src/rusty/skills_tab.rs`. No Zed path.

### Visual check plan
- REQ-001, REQ-002, REQ-004, REQ-005: open the Server page with #666's key; `940-01-server`;
  `940-08-day` after Marley Day.
- REQ-003: create the flag file and kill the stand-in (`pkill -f "$E2E_WORK/bin/rusty-mcp"`, a
  pattern naming the run's own folder, never a bare name); wait for the drop; `940-02-dropped`;
  remove the flag, click Try Now; `940-03-back`.
- REQ-011: while still dropped, `rusty: open memory`; `940-09-dropped-tab`; then the flag goes
  and Try Now as above.
- REQ-007, REQ-006: `rusty: open secrets`; `940-04-locked`; type the stand-in's PIN, Enter;
  `940-05-unlocked`.
- REQ-008, REQ-009: `rusty: open skills`; `940-06-skills`.
- REQ-010: click New Skill; `940-07-new-skill`; Escape.
- Not reachable: a real Rusty's version string and a real embedding run; the stand-in's
  `semantic.json` stands in.

### Risks
- A one-second tick on a Settings sub-page keeps the window drawing; limit it to while the page
  is shown and the state needs it (connected or down).
- Killing the stand-in in a run: `pgrep -f`/`pkill -f` patterns can match the scenario's own
  shell (memory: "`pgrep -f` wait loops match themselves"); match the run's folder path.
- Scanning every staged skill on each read costs one call per staged skill; the store holds few.
  If the list is read often (each announcement), keep scans by name and content hash and skip
  unchanged ones.

## Phase 2 — Code
- **Built (all Marley crates):**
  - `rusty.rs`: `Rusty.answered_at` and `retry_in`, with `keep`'s wait counting down each second
    (in place of a tick on every Rusty tab). `RustyServerView` became a struct with a one-second
    `_tick`. The page has `status_card`, `fact`, `semantic_block`, `known_row` with the names, and
    `setting_field` split from `setting_row`; `render_dropped` appends the countdown; the embedding
    provider's row carries its name and key.
  - `marley_rusty/src/settings.rs`: `KnownSetting.name`, `SemanticStatus::total` and
    `provider_words`.
  - `rusty/secrets_tab.rs`: `locks_at`, the one-second expiry loop, `unlocked_words` and the
    locked card.
  - `rusty/skills_tab.rs`: `scans`, filled in the read task before `take_read` (which drops that
    task); `render_staged`; Run on an active script's row; the New Skill words.
- **Deviations:**
  - The countdown lives in the global, not in per-view ticks, so every screen counts the same.
  - A staged row whose scan failed says "Scan: not run".
  - The Program value puts the server's name and version before the path, so a long path is cut
    and the version is not (found in Test).
- **Gate:** GREEN the first time, and again on the final tree.

## Phase 3 — Test
- **Scenario:** `script/e2e/940-rusty-server-and-forms-as-the-board.sh` (sway).
  - It uses #894's wrapper over the stand-in, with #666's settings and semantic status, #876's
    staged `quick-shell` (its backtick found) and active `release-notes` with `notes-draft.sh`,
    two made-up secrets, and the made-up PIN in the stand-in's `pin`.
  - To drop the connection, the scenario creates the flag and ends only its own stand-in, matched
    by `RUSTY_STAND_IN_STATE` in `/proc/<pid>/environ`.
- **Reds, fixed:**
  - The Program path ran past the card (now truncated, the version first).
  - "within 5 s Trying again" had no full stop between them (`render_dropped` adds one).
- **Reds, the scenario's own, fixed:**
  - `rusty: open memory` is refused while Rusty is down, so the dropped card is shown on the open
    Secrets tab.
  - The loss shows only at the next ping, so the scenario waits 12 s after ending the stand-in.
  - A Try Now clicked during a 5 s try waits for the next one, so it settles 12 s.
- **Shots:**
  - `940-01-server`: the header line; the card reads Connected · checked 4 s ago, embedded ·
    stdio, rusty-mcp stand-in · path, and on · as "rusty". Then "42 of 45 pages embedded · ollama
    · nomic-embed-text", the bar, "3 wait their turn", and the named rows (Brain vault, Notes,
    Embedding provider…), each with its key in mono and its default muted in the field.
  - `940-04-locked`: the lock card, the PIN rule, a field labelled PIN, and Unlock.
  - `940-05-unlocked`: "Unlocked · locks in 5 m", Lock, Change PIN.
  - `940-06-skills`: "quick-shell · staged by an agent", "Scan: contains a dynamic command-injection
    marker…" in the warning colour, Reject, Approve Anyway; "$ notes-draft … active" with Run.
  - `940-07-new-skill`: "Stage it for approval" and Make.
  - `940-08-day`, `940-08-day-server`: Skills and the page in Marley Day.
  - `940-09-dropped-tab`: Secrets' card "rusty-mcp did not answer within 5 s. Trying again in
    2 s." with Try Now.
  - `940-02-dropped`: Not connected, the reason in red, "Trying again in 1 s", Try Now, and the
    backoff line.
  - `940-03-back`: after Try Now, Connected · checked 0 s ago.

## Phase 4 — Complete
- **Documented (§21):** `CHANGELOG.md`, `docs/marley/look-plan.md`, and
  `docs/marley_architecture/marley_workbench.md` (the #940 entry under Rusty). No Zed path was
  touched.
- **Knowledge (§19):** L-claude-940-a-scenario-sees-rusty-drop-at-the-next-ping-001. No `F-`
  block: the two look reds were caught and fixed in the same ticket's Test.
- **Brain:** consultation e1ffef5917e64ce49732d0e65aa3ec5d closed with `brain no-decision`.
- **Ticket:** closed.
