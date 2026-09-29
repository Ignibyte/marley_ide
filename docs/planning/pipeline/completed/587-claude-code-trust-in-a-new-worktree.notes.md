# Claude Code's trust question in a new worktree, brought to the user — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-587-claude-code-trust-in-a-new-worktree.md
- **Pipeline spec:** 587-claude-code-trust-in-a-new-worktree.spec.md

## Phase 1 — Plan
- **Request:** Chad on 2026-09-28, on #510's check that the real Claude Code asks for trust in a
  new worktree: "this is true so we need to handle that with a popup if it does show the user can
  click or assume that since zed asks to trust the folder then we auto select it on for them via
  tmux or rustal harness". Filed as TICKET-587 the same day, at the top of the queue.
- **Classification / tier:** feature, M. Marley crates (`marley_agent`, `marley_workbench`) and
  the settings files #532 already touches (`settings_content`'s `marley.rs`, `default.json`,
  `settings_ui`'s `marley_page.rs` and `settings_ui.rs`), each with its ledger row.
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo
  busy with #560's release install, so no cargo ran and no crate or scenario was edited); recall
  ✓; mint ✓ (the pair from the templates, the backlog row removed, the ticket in progress);
  discovery (Explore, over `344303e7ab`) ✓; the prior-art sweep (a research agent: Claude Code's
  docs, CHANGELOG and issues; Orca's MIT source; Warp's docs; Zed's docs and the ACP adapter) ✓;
  spec and design ✓.
- **Recall (§18.3):**
  - AD-claude-510-a-worktree-agent-is-zeds-worktree-on-a-branch-of-its-own-001 rejected typing
    the prompt into the agent's TUI ("a trust dialog would take it") and writing trust into
    another tool's configuration; #510 put the prompt on the command line, so the question is
    what stands between the launch and the prompt.
  - L-claude-482 (lessons.md:2403-2411): live checks run in a folder Claude Code already trusts,
    "so no trust prompt and no project hooks"; #519's live check saw no SessionStart frame
    (519 notes, 322-327). Nothing records a frame while the question shows.
  - L-claude-560-zed-turns-a-repositorys-trust-on-without-an-event-001: `Repository::is_trusted`
    flips with no event, so Zed's trust is read from `TrustedWorktrees` here, not the repository.
  - The brain (consultation 437a1263a2c64bb99cf31058c3b7b213): nothing on this seam; it listed
    only follow-ups due on other projects.
- **What the prior-art sweep changed.** The ticket's premise came from #510's notes: that Claude
  Code asks in every new worktree. Its docs now say trust is keyed on the repository's main
  checkout ("In a worktree, it uses the main checkout's root"; issue #23109 closed on
  2026-08-17), so a worktree of a repository already trusted in Claude Code starts without the
  question, and the question comes for a repository never trusted there, the one case where
  its review of the repository's `.claude/` grants matters. Since 2.1.263 its focus starts on
  "No, exit", so Enter declines and quits. And Claude Code's docs tell its own agents "do not
  change trust settings on the user's behalf". So Chad's first way, a popup the user clicks, is
  the default, and his second, answering from Zed's trust, is a setting the user turns on (D4).
- **Discovery (the Explore report, 2026-09-28, over `344303e7ab`):**
  - #510's launch: `worktree_agents::create` (`worktree_agents.rs:348-427`) awaits Zed's create,
    which has already carried trust over (`maybe_propagate_worktree_trust`,
    `worktree_service.rs:643-692`) and added the workspace in the background (1409), then calls
    `agents::start_cli_with_prompt` (`agents.rs:281-326`), which opens a center terminal with
    `TerminalPanel::add_center_terminal`, runs the startup handshake and writes
    `marley_agent::launch_line` with `write_init_command_after_startup`, and returns `()`. Nothing
    keeps the terminal: the `WorktreeAgents` global's `creating` and `seed_skips` are gone before
    the agent starts.
  - The screen: `Terminal::last_n_non_empty_lines(n)` (`terminal.rs:2734-2737`) reads the live
    grid, joined wrapped rows, whatever the scroll, with no paint needed (the worktree's
    workspace is not painted); the stall watch reads it (`stall.rs:372-376`), and `links.rs`
    scans on each `Wakeup` with a 500 ms delay (262-318).
  - Events: the plugin's hooks reach Marley only as OSC 777 frames written through Claude Code's
    terminal (`event.py:136-149`, `notifications.rs:38-50`, `agent_events::on_frame`), and
    Claude Code holds hooks back until the folder is trusted; the inbox's terminal entries need a
    `Waiting` seat and carry no buttons (`seat_entry`, `answers: false`; `answer_inbox` ignores
    terminals).
  - Zed's trust: `TrustedWorktrees::can_trust(&store, id, cx)` (`trusted_worktrees.rs:459-554`)
    is the authority for a loaded worktree; `has_restricted_worktrees` reads "trusted" when
    nothing has checked the folder.
  - Keys: `Terminal::input` (`terminal.rs:2415-2425`) writes bytes and marks keyboard input;
    `paste` wraps text in bracketed paste, which a menu reading single keys misreads.
  - Buttons: `MessageNotification` with primary and secondary buttons
    (`workspace/src/notifications.rs:654-812`) and `show_app_notification` (1467); `Toast` with
    one button (`close_guard.rs:279-289`).
  - The setting pattern: #532's `claude_code_permissions`, from `settings_content`'s `marley.rs`
    to `default.json`, `MarleySettings`, `marley_page.rs` and a dropdown renderer in
    `settings_ui.rs`; `marley.*` keys are user-only.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.**
  - *The reader (`crates/marley_agent/src/trust.rs`, new, pure).* `pub struct TrustQuestion {
    folder: Option<String>, warnings: Vec<String>, options: Vec<TrustOption>, focus:
    Option<usize> }` with `TrustOption::{Trust, Exit}`, and `pub fn read(lines: &[String]) ->
    Option<TrustQuestion>`: the question when the lines hold a question line, a trust option, an
    exit option and the footer (D2). An option line is read past `❯`, spaces and a `1.`
    numbering; the focus is the option marked `❯`; the folder is the path line after `Accessing
    workspace:`, or the line after the question; the warnings are the lines that start with `⚠`.
    `answer_keys(&self) -> Option<Vec<u8>>`: Up (`\x1b[A`) or Down (`\x1b[B`) from the focus to
    the trust option, then `\r`; `None` without both options and a focus (D3). Marley's crates
    already hold the stall and risk rules as pure modules in `marley_agent`.
  - *The launch (`agents.rs`).* `start_cli_with_prompt` returns the launched terminal
    (`Task<Option<Launched>>`, `Launched { view: WeakEntity<TerminalView>, terminal:
    WeakEntity<Terminal> }`), still prompting its own errors, so its caller can watch it.
  - *The watch (`crates/marley_workbench/src/agent_trust.rs`, new).* A `TrustWatches` global of
    one `TrustWatch` entity per watched terminal, made by `worktree_agents::create` for a Claude
    Code launch: the terminal and its view, the worktree's workspace, the worktree's name, the
    launch time. It subscribes to the terminal's events; on output, at most twice a second (D6),
    it reads `last_n_non_empty_lines(24)` and `trust::read`. States: watching (until the
    question shows, for a minute after the launch), asking (the question on screen), done (the
    question gone, the minute over, or the terminal closed: the subscription dropped and the
    notification dismissed). Entering asking: under `follow_zed`, when `can_trust` finds the
    worktree's folder trusted, it answers at once and shows a toast in the source workspace;
    otherwise it shows the app notification (D5), with the question's warnings, Trust Folder
    when `answer_keys` gives keys, and Show Terminal. An answer reads the screen again, writes
    the keys with `Terminal::input` only while `trust::read` still finds the question, and
    checks a second and a half later; a question still there turns the notification into a note
    to answer it in the terminal, with Show Terminal alone (REQ-009). Show Terminal activates the
    worktree's workspace in the `MultiWorkspace` and the terminal's item, as the rail's
    `activate_terminal` does.
  - *The setting.* `ClaudeCodeWorktreeTrust { Ask, FollowZed }` in `settings_content`'s
    `marley.rs`, `claude_code_worktree_trust: Option<…>` in `MarleySettingsContent` (default
    `"ask"` in `default.json`), `MarleySettings::claude_code_worktree_trust`, a dropdown row in
    the Marley page's Agents section, and the dropdown renderer registered in `settings_ui.rs`.
- **File manifest.**
  - Marley crates: `crates/marley_agent/src/trust.rs` (new), `crates/marley_agent/src/marley_agent.rs`
    (the module); `crates/marley_workbench/src/agent_trust.rs` (new),
    `crates/marley_workbench/src/agents.rs` (the launch's return),
    `crates/marley_workbench/src/worktree_agents.rs` (the watch started),
    `crates/marley_workbench/src/marley_workbench.rs` (the module, the setting, `init`).
  - Zed crates, each with its ledger row extended before the edit:
    `crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
    `crates/settings_ui/src/marley_page.rs`, `crates/settings_ui/src/settings_ui.rs`.
  - Test: `script/e2e/587-claude-code-trust-in-a-new-worktree.sh`, `script/e2e/golden`.
- **Ledger rows.** The four Zed paths above already have rows (#460, #515, #503, #514, #532);
  each gains #587's words before its edit.
- **Knowledge at Complete (expected).** A lesson on Claude Code's trust keyed on the main
  checkout, and the focus on "No, exit"; an AD for bringing the question to the user rather
  than writing Claude Code's record; #510's AD and the guide corrected where they said Claude
  Code asks in every worktree.

### E2E plan
The scenario, `script/e2e/587-claude-code-trust-in-a-new-worktree.sh`, `compositor sway`. Setup:
#510's scratch repository and git config; a fake `claude` first on the terminal's PATH (#510's
guard stops the run before any launch unless the terminal's `claude` is the fake) that logs its
arguments and folder and, unless `$E2E_WORK/claude-trust.txt` names the repository's main
checkout, draws the question in the 2.1.263 form, "Accessing workspace:", the folder, "Quick
safety check: Is this a project you created or one you trust?", "⚠ This folder pre-approves 2
tool permissions", "❯ No, exit", "  Yes, I trust this folder", "Enter to confirm · Esc to
cancel", reads keys raw (Up and Down in both cursor modes, Enter, Escape), redraws the focus,
logs each key and the choice, and on the trust option writes the record, clears the screen and
prints its prompt; on "No, exit" it exits. The coordinates come from the first run's shots.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-002 | trust the repository in Zed; `+`, New Agent in Worktree, Claude Code, a prompt, Enter; settle 8 on the main checkout's workspace | `587-01-asked`: the notification, the worktree's name and folder, the ⚠ line, Trust Folder and Show Terminal |
| REQ-004 | click Show Terminal | `587-02-shown`: the worktree's workspace, the agent's terminal with the question |
| REQ-003 | click Trust Folder in the notification; settle 3 | `587-03-trusted`: no notification, the fake past the question with its prompt; the log: Down, Enter, `yes` |
| REQ-005 | remove the record; a second worktree agent; once the notification shows, click the agent's row in the rail, press Down and Enter in the terminal; settle 3 | `587-04-answered-in-terminal`: no notification; the log: the scenario's Down and Enter, and no key from Marley |
| REQ-006 | set `claude_code_worktree_trust` to `follow_zed`; remove the record; a third worktree agent; settle 8 | `587-05-followed-zed`: no notification, the toast; the log: Down, Enter, `yes`, with no click |
| REQ-007 | keep the record; a fourth worktree agent; settle 8 | `587-06-no-question`: no notification; the log: the launch, no key |
| REQ-008, REQ-009 | review: `can_trust` before an answer without a click; `answer_keys` `None` without both options; no keys after a question that stayed | review |
| REQ-010 | the gate and the golden set | the gate's log; `just regress` |

Not reachable by a scenario: the real Claude Code's screen (the real `claude` never runs here;
the fake draws the published 2.1.263 form, and the first real run is Chad's check at the end of
the queue, in a repository never trusted in Claude Code); a restricted project, whose own
worktree create differs (#510's flow waits on Zed's own trust modal), so REQ-008 is reviewed.

### Risks
- The question's words change again: the reader misses it, and the agent waits as it does
  today, answered in its terminal. The reader takes each published form; a miss costs nothing.
- The arrows: Marley sends the normal-mode forms (`\x1b[A`, `\x1b[B`); a TUI in application
  cursor mode may want `\x1bOA`; Claude Code's input parser takes both forms as far as its
  published issues show, and the fake accepts both. The answer is checked afterwards, and a
  question still on the screen gets no more keys (REQ-009).
- `Terminal::input` marks keyboard input, which lets the terminal's tab close when its shell
  exits, as after any key the user types.
- Several worktree agents at once each get their own notification, one per terminal.
- A worktree agent whose Claude Code takes over a minute to draw anything is not watched after
  the minute: the question then waits in its terminal, as today.

## Phase 2 — Code
- **Checklist** (no task tool): the README marker present ✓; the four Zed paths' ledger rows
  extended before their edits ✓; `marley_agent`'s `trust.rs` ✓; `agents.rs` ✓;
  `agent_trust.rs` ✓; `worktree_agents.rs` ✓; `marley_workbench.rs` ✓; the setting in
  `marley.rs`, `default.json`, `marley_page.rs` and `settings_ui.rs` ✓; `just clippy marley_agent
  marley_workbench settings_content settings_ui` ✓, after #560's release install had ended;
  `cargo fmt --check` on the four crates ✓; the review ✓.
- **What was built.**
  - `crates/marley_agent/src/trust.rs` (new, pure): `TrustQuestion { folder, warnings, options,
    focus }` and `TrustOption::{Trust, Exit}`; `read(lines)` finds the footer (`Enter to
    confirm`) among the last three lines, the first question line (`Do you trust the files in
    this folder?`, `Is this a project you created or one you trust?`, `Accessing workspace:`)
    within 24 lines above it, and the options between, each read past `❯`, spaces and a `1.`
    numbering (`Yes, I trust this folder` or `Yes, proceed`; `No, exit`), with the focus on the
    option marked `❯`, the folder the first path line after the question line, and the warnings
    the lines that start with `⚠`; a question needs both options. `answer_keys()` gives Up or
    Down (`\x1b[A`, `\x1b[B`) from the focus to the trust option, then `\r`, and nothing without
    a focus.
  - `crates/marley_workbench/src/agents.rs`: `start_cli_with_prompt` returns
    `Task<Option<WeakEntity<Terminal>>>`, the terminal once the command is written, through
    `prompt_err`, so its errors still reach a prompt; `start_cli` detaches it.
  - `crates/marley_workbench/src/worktree_agents.rs`: `create` awaits the launch and, for Claude
    Code, starts the watch with the terminal, the worktree's workspace, the workspace that started
    it and the worktree's name.
  - `crates/marley_workbench/src/agent_trust.rs` (new): the `TrustWatches` global, one
    `TrustWatch` entity per terminal. It subscribes to the terminal's `Wakeup` and reads
    `last_n_non_empty_lines(24)` half a second after output, one read at a time; it ends after a
    minute while the question has not shown, when the question leaves the screen, or when the
    terminal is released (`observe_release`). When the question shows, under `follow_zed` and
    with `TrustedWorktrees::can_trust` true for the worktree's first folder, it sends the keys and
    shows a toast in the starting workspace; otherwise an app notification (`show_app_notification`,
    one id per terminal) with the headline, the warnings in the warning color, Trust Folder when
    `answer_keys` gives keys, and Show Terminal. An answer reads the screen again first, sends the
    keys once (`Terminal::input`), and a second and a half later either finds the question gone or
    shows a second notification, with an id of its own, to answer it in the terminal, with Show
    Terminal alone. Show Terminal activates the worktree's workspace in the window's
    `MultiWorkspace` and the terminal's item, found among the workspace's `TerminalView`s.
  - The setting: `ClaudeCodeWorktreeTrust { Ask, FollowZed }` and
    `claude_code_worktree_trust` in `settings_content`'s `marley.rs`, `"ask"` in `default.json`,
    `MarleySettings::claude_code_worktree_trust`, the Marley page's Worktree Trust Question
    dropdown (the Agents section now ten items), and its renderer in `settings_ui.rs`.
- **Deviations from the design.**
  - The launch returns the terminal alone, not a `Launched` with its view: Show Terminal finds
    the view among the worktree's workspace's items when it is clicked, so nothing holds a view
    that may close.
  - No `init`: the watch starts from `worktree_agents::create`, and nothing else needs one.
  - The "still asking" notification has an id of its own: a button of Zed's `MessageNotification`
    dismisses its notification after its handler runs, deferred, which would have dismissed a
    replacement under the same id.
- **The review**, against each REQ:
  - REQ-001, REQ-002: the fake draws at once, its output wakes a read half a second later, and the
    notification shows in every workspace of the window, the headline and the warnings from the
    screen.
  - REQ-003: `answer_keys` counts from the `❯` option to the trust option, so "No, exit" first
    takes one Down; the keys go only while `read` still finds the question.
  - REQ-004: the button's own dismissal takes the notification; the workspace and the item are
    activated as the rail's `activate_terminal` does.
  - REQ-005, REQ-007: a question that leaves ends the watch with the notification dismissed and no
    key; a question that never comes ends it after the minute.
  - REQ-006, REQ-008: the automatic answer needs both the setting and `can_trust`, read when the
    question shows; the default is to ask.
  - REQ-009: no Trust Folder without both options and a focus; after its keys, a question still
    on the screen gets no more (`answered: true`) and a note to answer it in the terminal.
  - Re-entrancy: the buttons' handlers update the watch, the `MultiWorkspace` and the worktree's
    workspace from the notification's own listener, inside no update of theirs; `finish` defers
    the watch's removal from the global.
  - The keys: the arrows and Enter alone, computed from the screen; nothing typed is text, so
    bracketed paste is not involved.
  - Provenance: Orca's recognition and its refusal to type blindly, from the report and its MIT
    source's behavior, reimplemented; Claude Code's forms from its published issues; no Zed
    function body copied; nothing of Warp.
- **Clippy**: run 1 found `option_if_let_else` in `trust.rs`'s option reader; run 2
  `too_long_first_doc_paragraph` on `start_cli_with_prompt` and `needless_pass_by_ref_mut` on
  `read_soon`; each fixed at the source. Run 3, `cargo clippy -p marley_agent -p marley_workbench
  -p settings_content -p settings_ui --all-targets -- -D warnings`, is clean; `cargo fmt` wrapped
  `agent_trust.rs`'s gpui import, and `--check` is clean.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario per the E2E plan ✓; 587 in the golden set ✓; `just
  build` ✓; the scenario run and every shot read ✓; the golden set ✓; `gates.sh --diff` ✓.
- **The scenario**, `script/e2e/587-claude-code-trust-in-a-new-worktree.sh` (compositor sway, no
  Chromium): #510's scratch repository on `main`, trusted in Zed at the start; a fake `claude`
  (Python, first on the terminal's PATH, behind #510's guard) that, unless its record names the
  repository's main checkout (as Claude Code keys its trust), draws the question in the 2.1.263
  form ("Accessing workspace:", the folder, "Quick safety check: Is this a project you created or
  one you trust?", "⚠ This folder pre-approves 2 tool permissions", "❯ No, exit", "  Yes, I trust
  this folder", "Enter to confirm · Esc to cancel"), reads keys raw (both cursor modes' arrows,
  Enter, Escape), and logs each key, its choice, its folder, its root and the record it read,
  with its pid; the flag file `answer-yes` stands in for a user answering at the terminal. Five
  worktree agents from the project's `+`: Show Terminal and then keys in the terminal; Trust
  Folder; answered through the flag; `follow_zed`; and one the fake's record already trusts. The
  run log prints each launch's keys and choice, the record before the fifth, and the fake's
  whole log.
- **The runs.**
  - Run 1: the notification showed as planned, but Show Terminal's point was a guess and missed;
    the keys went to the main checkout's terminal. The buttons were measured from its shot: the
    notification is anchored to the bottom right, so they sit at y 932 whatever its height.
  - Run 2 found two product bugs (F-587 in the knowledge): the watch read an answered question as
    still showing, from the scrollback, so Trust Folder's check showed "still asking" for
    answered agents, and the notification of a question answered elsewhere never went; and the
    fifth launch was asked, which run 3's log traced to the scenario itself.
  - The first bug, fixed: `last_n_non_empty_lines` skips blank rows and reaches into the
    scrollback on a sparse screen, where the fake's cleared question stayed; the question now
    counts as showing only while its footer is on a row of the visible screen
    (`Terminal::with_renderable_cells`, `trust::footer_on_screen`), and its text still comes from
    the joined lines, so a wrapped path stays whole.
  - Run 5, with the fake logging its root and record, showed every launch reading an empty record
    while the file held the root: the scenario's `sed` template replaced only the first
    `@RECORD@` of the line that names it twice (no `g`), so `os.path.exists` checked a file named
    `@RECORD@`. Fixed with `g` on each substitution. Run 5 also showed the second product bug:
    `follow_zed`'s toast was gone by its shot, since Zed shows a toast as a notification under its
    id and the toast used the question's id, which the question's end dismissed; it has an id of
    its own now, and stays until it is closed.
  - Run 6 passed every check with every shot as planned.
- **The shots** (run 6, read one by one):
  - `587-01-asked` (REQ-001, REQ-002): the main checkout's workspace in front; the rail lists
    `bronze-vole` with its terminal, "Claude Code · waiting"; at the bottom right, "Claude Code in
    bronze-vole is asking whether to trust /…/worktrees/repo/bronze-vole/repo.", "⚠ This folder
    pre-approves 2 tool permissions" in the warning color, and Trust Folder and Show Terminal.
  - `587-02-shown` (REQ-004): after Show Terminal, the worktree's workspace (`repo / bronze-vole /
    agent/bronze-vole` in the title), the agent's terminal focused and its rail row selected, the
    question on its screen with "❯ No, exit" first; no notification. The run log: the scenario's
    Down and Enter reached that terminal (`launch 1: keys ['down', 'enter'], choice yes`).
  - `587-03-trusted` (REQ-003): after Trust Folder, no notification; `elder-lotus` listed. The run
    log: `launch 2: keys ['down', 'enter'], choice yes`, the keys Marley sent from "No, exit".
  - `587-04-answered-elsewhere` (REQ-005): the flag answered the third, and its notification went
    by itself; the run log: `launch 3: keys [], choice yes, flag True`.
  - `587-05-followed-zed` (REQ-006): with `follow_zed`, no question notification for the fourth;
    the toast "Claude Code in sandy-peony asked whether to trust /…/sandy-peony/repo; Marley
    answered yes, since Zed trusts it." with its close button; the run log: `launch 4: keys
    ['down', 'enter'], choice yes`, with no click.
  - `587-06-no-question` (REQ-007): the fifth, `civil-sequoia`, listed, with no notification of
    its own (the fourth's toast stays until closed); the run log: `launch 5: keys [], choice
    none`.
- **Focus**: the scenario ran in its own headless sway; the runner reported `hyprland: 0 Marley
  windows before the run, 0 after`, no rule added.
- **Not reachable by a scenario**: the real Claude Code's screen, since the real `claude` never
  runs here (the fake draws the published 2.1.263 form; the first real run is Chad's check at the
  end of the queue, in a repository never trusted in Claude Code); a project Zed does not trust
  (REQ-008, reviewed: `can_trust` before any answer without a click); a question without both
  options, and one that stays after Marley's keys (REQ-009, reviewed).
- **The gate**: run 1 was red at gate:2: `too_long_first_doc_paragraph` on `trust::footer_on_screen`, which came with the scrollback fix after the Code phase's clippy; the doc comment split (no code change, so the scenario's runs stand). Run 2: `GATE GREEN [diff]`, 15 gates PASS (rustfmt, clippy on every target, cargo-audit, cargo-deny, cargo-shear, gitleaks, shellcheck, no-suppressions, source-bans, docs, zed-ledger, manifests, typos, semgrep, dylint); the logs are scratchpad `587/gate.log` and `587/gate2.log`.
- **The golden set** with 587 added: 50 of 50 (`just regress`), 587 among them (109 s).
- **Verdict**: PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented**: `CHANGELOG.md` (Added: Claude Code's trust question in a new worktree, brought
  to you); `docs/marley/workbench-shell.md` (W4's record gains #587); `docs/marley/tutorial-outline.md`
  (a row, shipped); `docs/marley/guide.md` ("Worktree agents": the paragraph that said Claude Code
  asks in every new worktree corrected, and the question's notification, its two buttons and the
  setting); `docs/marley_architecture/marley_workbench.md` ("Claude Code's trust question in a new
  worktree"); `docs/marley_architecture/marley_agent.md` (`trust.rs`, and `agent_trust` among the
  consumers). The four Zed paths' ledger rows (`settings_content`'s `marley.rs`,
  `settings_ui`'s `marley_page.rs` and `settings_ui.rs`, `default.json`) were extended before
  their edits and describe what shipped.
- **Knowledge**: F-claude-587-the-trust-watch-read-an-answered-question-from-the-scrollback-001
  and PR-claude-what-a-terminal-shows-now-is-read-from-its-visible-rows-001;
  F-claude-587-the-answers-toast-shared-the-questions-notification-id-001;
  L-claude-587-claude-code-keys-its-trust-on-the-main-checkout-001,
  L-claude-587-a-zed-toast-is-a-notification-under-its-id-001,
  L-claude-587-a-sed-template-needs-g-for-a-placeholder-twice-on-a-line-001;
  AD-claude-587-marley-brings-claude-codes-trust-question-to-the-user-001.
- **Brain**: consultation 437a1263a2c64bb99cf31058c3b7b213 closed with
  `decisions/marley-brings-claude-codes-trust-question-in-a-new-worktree-to-the-user` (follow-up
  2026-10-28).
- **For Chad, at the end of the queue** (with #535's phone check): a worktree agent in a
  repository never trusted in Claude Code, with the real `claude`: the notification, Trust
  Folder, and the agent running its first prompt.
- **Closed**: TICKET-587 moved to `tickets/closed/`; its BACKLOG row went at promotion.
