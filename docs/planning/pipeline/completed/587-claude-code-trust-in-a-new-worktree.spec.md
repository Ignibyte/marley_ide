---
pipeline_id: 43fcc733-bda1-4fad-a4ad-7b5e983f00b6
ticket: docs/planning/tickets/closed/TICKET-587-claude-code-trust-in-a-new-worktree.md
status: Phase 4 — Complete PASS
title: "Claude Code's trust question in a new worktree, brought to the user"
type: feature
slice: prong 2, worktree agents, a follow-up of #510
references: [docs/planning/pipeline/completed/510-worktree-agents.spec.md, docs/orca_architecture/01-agents-and-sessions.md, docs/orca_architecture/06-cli-automations-skills.md, docs/orca_architecture/02-worktrees-and-review.md]
---

## Title
When Claude Code, started by New Agent in Worktree (#510), stops on its folder-trust question,
Marley brings the question to the user wherever they are in the window: a notification that
names the worktree and the folder, carries the question's warnings, and answers it with one
click, or shows the agent's terminal. The worktree's workspace opens in the background, so
today the question waits unseen in a terminal nobody is looking at. A setting, off by default,
lets Marley answer it by itself when Zed trusts the folder.

## Scope
### In
- **The watch.** For each Claude Code terminal New Agent in Worktree starts, Marley reads the
  last lines of the terminal's screen as it draws (`Terminal::last_n_non_empty_lines`, as the
  stall watch does), for at most a minute after the launch, and recognizes Claude Code's trust
  question: a question line, a trust option and an exit option, and the `Enter to confirm`
  footer, in any of the published forms (D2). It stops watching once the question has come and
  gone, the minute passes, or the terminal closes.
- **The notification,** shown in every workspace of the window while the question is on the
  screen: "Claude Code in `<worktree>` is asking whether to trust `<folder>`", the question's
  warning lines (such as a folder that pre-approves tool permissions), and two buttons, Trust
  Folder and Show Terminal. It goes when the question leaves the screen, however it was answered.
- **Trust Folder** reads the screen again, and while the same question is still there, moves
  Claude Code's focus to the trust option, wherever the focus is, and confirms it (D3). A
  question still on the screen a moment later turns the notification into a note to answer it in
  the terminal.
- **Show Terminal** shows the worktree's workspace and focuses the agent's terminal.
- **The setting** `marley.claude_code_worktree_trust`: `"ask"` (the default) or `"follow_zed"`,
  under which Marley answers the question itself when Zed trusts the worktree's folder, and says
  so in a toast; when Zed does not trust it, the notification shows as under `"ask"` (D4). On
  the Marley settings page, in the Agents section. User settings only.
- The guide's #510 paragraph corrected: Claude Code keys its trust on the repository's main
  checkout, so a worktree of a repository already trusted there starts without the question.
- `script/e2e/587-claude-code-trust-in-a-new-worktree.sh`.

### Out (explicitly deferred)
- Writing Claude Code's own trust record (`~/.claude.json`, `hasTrustDialogAccepted`): the docs
  call it a file Claude Code "writes for itself" and name the key only as a user's hand edit,
  and concurrent writes to it have been lost (D1).
- The trust questions of the other agent CLIs in a worktree (Codex, Gemini CLI, OpenCode), and
  Claude Code started from the `+` in a project's main checkout, whose terminal the user is
  looking at: later slices, on the same watch.
- A Claude Code run without its interactive dialog (`-p`, the SDK, ACP): it runs a repository's
  hooks and MCP servers with no trust gate at all.
- The inbox (#508): its terminal entries have no answer buttons and need the plugin's events,
  which Claude Code holds back until the folder is trusted.

## Reference (§20)
Orca's Claude path (MIT, `/srv/stacks/orca-refs/orca` at `1c2cf120`; the maps
`docs/orca_architecture/01-agents-and-sessions.md:74-81, 583` and
`06-cli-automations-skills.md:229-233`): Orca puts the first prompt on Claude Code's command
line, recognizes a trust question in the terminal's last non-blank lines
(`src/main/runtime/terminal-wait-detection.ts:244-258`, `agent-trust-workspace`), and refuses to
type into it ("Pasting anyway would answer whatever question is on screen",
`agent-launch-terminal-prompt.ts:60-66`), leaving the answer to the user. Marley keeps the
recognition and the refusal to type blindly, and adds what Orca lacks: the question carried to
wherever the user is, and an answer the user gives with one click, which picks the trust option
by name rather than pressing Enter. Upstream Zed: its worktree service carries Zed's trust to a
new worktree (`maybe_propagate_worktree_trust`, `crates/git_ui_core`), which the setting reads;
Zed's Agent Panel runs Claude Code through the SDK, where the dialog never shows. Warp: N/A,
nothing published on a CLI agent's trust question (docs.warp.dev, searched 2026-09-28).

### Prior art
- **Published material (Claude Code, read 2026-09-28).** The trust dialog accepts a folder
  before Claude Code loads its configuration; acceptance is saved per project; "In a worktree,
  it uses the main checkout's root" (code.claude.com/docs/en/permissions, "Project allow rules
  and workspace trust"; issue #23109 closed 2026-08-17: "git worktrees of an already-trusted repo
  don't prompt again"). It shows in interactive sessions only; `-p` and the SDK never show it
  (security, hooks#workspace-trust). An interactive session holds back hooks from every settings
  file until the folder is trusted (hooks#workspace-trust). The record is `~/.claude.json`,
  `projects["<root>"].hasTrustDialogAccepted`, which the docs name only as a fix a user makes by
  hand (permissions, mcp, errors), in a file Claude Code "writes for itself" (settings);
  concurrent writers have lost it (CHANGELOG 2.1.259; issues #92908, #97888). The screen's text
  has changed: "Do you trust the files in this folder? … ❯ 1. Yes, proceed / 2. No, exit / Enter
  to confirm · Esc to exit" (2.0.27, #10409); "Accessing workspace: … Quick safety check: Is this
  a project you created or one you trust? … ❯ 1. Yes, I trust this folder / 2. No, exit / Enter
  to confirm · Esc to cancel" (#40002); "❯ No, exit / Yes, I trust this folder", with the focus
  on "No, exit" so Enter declines (2.1.263, #92911; 2.1.270, #94277). The screen can carry
  "⚠ This folder pre-approves N tool permissions…" (CHANGELOG 2.1.218 names the root the grant
  covers). No flag, setting or environment variable pre-answers it; `--dangerously-skip-permissions`
  and `IS_DEMO` do not grant trust. This box runs Claude Code 2.1.284 (its link's target, read,
  not run).
- **Behavior maps and source (Orca, MIT).** As in the Reference; Orca pre-writes trust files for
  four other CLIs (`src/main/agent-trust-presets.ts:9-22`, "the only documented bypass") and not
  for Claude Code; it types into a trust question only in a hidden probe in an empty folder of its
  own (`src/main/rate-limits/claude-pty.ts:282-286`).
- **The code we already ship.** `Terminal::last_n_non_empty_lines` (`crates/terminal`, the live
  grid whatever the scroll, with no paint needed; the stall watch reads it at
  `crates/marley_workbench/src/stall.rs:372-376`, and `links.rs` scans on each output with a 500
  ms delay); `Terminal::input` for keys; `TrustedWorktrees::can_trust` and
  `has_restricted_worktrees` (`crates/project/src/trusted_worktrees.rs`) for Zed's trust, which
  `maybe_propagate_worktree_trust` has settled before #510's create resolves;
  `MessageNotification` with a primary and a secondary button and `show_app_notification`
  (`crates/workspace/src/notifications.rs`); `Toast::on_click` (`close_guard.rs:279-289`);
  `TerminalPanel::add_center_terminal` and #510's `agents::start_cli_with_prompt`. Does a crate
  we build own this seam? No: Zed's terminal gives the screen and the input, Zed's trust store
  gives the trust, and nothing reads another program's dialog; the owner is Marley's, beside the
  stall rules in `marley_agent`.

## UI proof
UI-AFFECTING (a notification with two buttons, a toast, a setting). `script/e2e/587-claude-code-trust-in-a-new-worktree.sh`
(`compositor sway`: it clicks the notification's buttons and the `+`'s submenu, as #510's does).
Setup: #510's scratch repository on `main`; a fake `claude` first on the terminal's PATH (Python,
with #510's guard that stops the run unless the terminal's `claude` is the fake) that logs its
arguments, and unless the scenario's own trust record names the repository's main checkout,
draws the trust question in the 2.1.263 form with the focus on "No, exit" and a "⚠ This folder
pre-approves 2 tool permissions" line, reads keys raw (the arrows in both cursor modes, Enter,
Escape), logs each key and the choice, and on "Yes, I trust this folder" records the trust,
clears the screen and prints its prompt argument; on "No, exit" it exits. Shots:
`587-01-asked` (the notification over the main checkout's workspace, with the worktree's name,
the folder, the warning line and both buttons); `587-02-shown` (Show Terminal: the worktree's
workspace and the agent's terminal with the question); `587-03-trusted` (Trust Folder: the
notification gone, the fake past the question with its prompt; the log: Down, then Enter, the
choice "yes"); `587-04-answered-in-terminal` (a second worktree agent after the record is reset,
answered with keys in its terminal: the notification gone, and the log holds only the
scenario's keys); `587-05-followed-zed` (`follow_zed` set, a third worktree agent: no
notification, the toast, the choice "yes" with no click); `587-06-no-question` (the record kept,
a fourth worktree agent: the fake asks nothing, no notification, nothing sent).

## Locked-In Decisions
- D1: Marley never writes Claude Code's trust record. The docs name `hasTrustDialogAccepted`
  only as a user's hand edit, in a file Claude Code writes for itself, and concurrent writers
  have lost it; #510's AD rejected it for the same reason. The question stays Claude Code's, and
  Marley only brings it to the user and types the user's answer.
- D2: The question is recognized on the screen, in the last lines `last_n_non_empty_lines`
  gives, by its parts: a question line (`Do you trust the files in this folder?`, or `Is this a
  project you created or one you trust?`, or `Accessing workspace:`), a trust option (`Yes,
  proceed` or `Yes, I trust this folder`), an exit option (`No, exit`) and the footer (`Enter to
  confirm`). Claude Code sends no hook event while the question shows (hooks wait for trust), so
  the screen is the only signal. The watch runs only for terminals New Agent in Worktree started,
  for their first minute: nothing else is read.
- D3: An answer picks the trust option by name: the keys move the focus from the option marked
  `❯` to the trust option (Up or Down), then Enter, and only while the screen still shows the
  same question, read again just before. Since 2.1.263 the focus starts on "No, exit", so Enter
  alone would decline and quit. A question Marley cannot parse down to both options gets no keys,
  and its notification offers Show Terminal alone.
- D4: The default is to ask. `follow_zed` answers only when Zed trusts the worktree's folder
  (`TrustedWorktrees::can_trust` on the worktree's workspace, read when the question shows): the
  question exists to review what a repository's `.claude/` grants, and Claude Code's docs say
  not to change trust for the user, so answering without a click is the user's own setting,
  never Marley's default.
- D5: The notification is an app notification (`show_app_notification`), one per agent terminal,
  so it shows in the displayed workspace, whichever it is, and goes from all of them at once.
- D6: The watch reads the screen at most twice a second, on the terminal's output, as the
  served-URL scan does; a question that never comes costs a minute of reads of a few lines.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Claude Code that New Agent in Worktree started shows its trust question, the system shall show a notification naming the worktree and the folder, with Trust Folder and Show Terminal, in the workspace the user is on, within five seconds. | Shot `587-01-asked` |
| REQ-002 | The notification shall carry the question's warning lines as the terminal shows them. | Shot `587-01-asked` |
| REQ-003 | WHEN the user presses Trust Folder, the system shall choose the trust option, whichever option has the focus, and the notification shall go. | Shot `587-03-trusted`; the fake's log: Down, Enter, `yes` |
| REQ-004 | WHEN the user presses Show Terminal, the system shall show the worktree's workspace with the agent's terminal focused, and the notification shall go. | Shot `587-02-shown`; the fake's log: the scenario's Down and Enter |
| REQ-005 | WHEN the question leaves the screen without Trust Folder, the notification shall go, and the system shall send the terminal nothing. | Shot `587-04-answered-in-terminal`; the fake's log holds the scenario's keys alone |
| REQ-006 | WHERE `marley.claude_code_worktree_trust` is `follow_zed` and Zed trusts the worktree's folder, the system shall choose the trust option without a notification and say so in a toast. | Shot `587-05-followed-zed`; the fake's log: `yes`, no click |
| REQ-007 | WHERE Claude Code starts without the question, the system shall show nothing and send the terminal nothing. | Shot `587-06-no-question`; the fake's log |
| REQ-008 | WHERE Zed does not trust the worktree's folder, the system shall not answer the question by itself, whatever the setting. | Review of the diff (the scenario's repository is trusted; a restricted project changes #510's own flow) |
| REQ-009 | WHEN the screen does not give both options, or still shows the question after Marley's keys, the system shall send no Enter, or no more keys, and shall offer Show Terminal. | Review of the diff |
| REQ-010 | The diff gate shall be green, and the golden set shall pass. | `script/gates.sh --diff`; `just regress` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes.
- **P2 Code:** the question's reader in `marley_agent`; the launch returning its terminal; the
  watch, the notification and the answer in `marley_workbench`; the setting and its page row;
  fmt and clippy clean; a review of the diff against each REQ and each key Marley sends.
- **P3 Test:** write and run the scenario and read every shot; rerun #510's scenario; the golden
  set; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; the guide's worktree paragraph corrected; the architecture notes;
  the ledger capture; close the ticket, archive, commit.
