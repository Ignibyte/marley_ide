# The golden regression run green — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-635-the-golden-regression-green.md
- **Pipeline spec:** 635-the-golden-regression-green.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** wave 5 (Chad, 2026-09-30).
- **Recall (§18.3):** AD-claude-517; since 2026-09-29 no regression ran per ticket; #627 made the
  prompt editor the default input and #630 made the block headers the default look, both of which
  golden scenarios that type at a prompt or read its rows will meet.

## Phase 1 — Plan (promoted 2026-10-01)
- **Classification / tier:** chore; scenarios, and any code a regression exposes.
- **Recall (§18.3):** the last full runs were 2026-09-29 03:17 and 04:53 (52 scenarios each; the
  set has 53 now), before the per-ticket regression stopped; `just install` runs the set only with
  `--regress` since then. #634 just changed the workbench's focus checks (`holds_focus`), its
  blocking reads and two test seams; #627 (the prompt editor at every prompt) and #630 (block
  headers on) changed what a scenario that types at a prompt or reads its rows meets. The brain was asked only at Complete (an omission here); it
  listed follow-ups due and nothing on the question.
- **Discovery:** `script/regress` runs each scenario under `COMPOSITOR=sway` with a 900 s limit,
  logs and shots in `~/.local/state/marley/regress/<stamp>/` (the five newest kept), and prints
  PASS or FAIL with the first line that says why.

### Design
- **Approach:** the run on the current debug build (`just regress`, in the background, its
  summary kept); a triage table, one row per failure, each read from its log and shots: stale
  (the scenario follows the shipped UI, naming the ticket), regression (code fixed), flaky (the
  scenario waits on its condition). Each fix is checked by rerunning its scenario alone
  (`just regress <scenario>`), then the whole set again.
- **File manifest:** decided by the failures: `script/e2e/*.sh` (scenarios), Marley crates for a
  regression; a Zed path only with its row first.

### Visual check plan
| REQ | What runs | What shows it |
|---|---|---|
| REQ-001 | the whole set again after the fixes | its verdict: all passed |
| REQ-002 | the first run | the triage table, one row per failure |
| §7 | each fixed scenario's shots | read for what the fix changed |

### Risks
- An hour or more per run; `/mnt/fast` has 38G free (no build is needed unless a fix is in code).
- A scenario that depends on the box (docker, sshd, Chromium, a fake provider's port) can fail for
  the box, which is named, not hidden.

## Phase 2 — Code
### First run (2026-10-01, `20261001-040437`): 53 run, 45 passed, 8 failed

| Scenario | Class | What failed | Cause | Fix |
|---|---|---|---|---|
| 484-autosuggestions | stale (#627) | → took no suggestion; `ech` ran | Since #627 the prompt editor takes the line at every prompt and shows no history suggestion (#627 deferred #484's ghost text in the editor to a follow-up that was never minted). | The scenario turns `marley.prompt_editor` off with a new shared helper, `setting`, in `script/e2e.sh`. The missing follow-up becomes a ticket. |
| 516-secret-redaction-for-agents | stale (#542, #557, #627) | the toggle click missed | Three Marley page items came in above Redact Secrets for Agents. | The toggle's measured place (y 830). |
| 535-phone-push-notifications | stale (#538) | the OSC 777 banner missing | A project's five seconds without a second banner: of one burst's two, the first shows. | The check expects the first. |
| 503-terminal-urls-open-in-the-browser | stale (#526) | nothing printed after `ssh e2e-host` | Marley's `ssh` asks `ssh -G` first; the fake printed and slept 600 s on it. | The fake refuses `-G`, which sends the function to the plain ssh. |
| 532-agent-permission-modes | stale (#510) | the menu's highlight on New Agent in Worktree | The + menu gained New Agent in Worktree above the agent CLIs. | One more step to each CLI. |
| 511-review-and-merge-a-worktree | stale (#589) | Remove's refusal in place of Merge's prompt | Remove… is the menu's last entry since #589; `merge` pressed End. The three earlier refusal checks had passed on Remove's refusal, not Merge's. | `merge` takes the second entry. |
| 575-terminal-id-across-restore | **bug** | the quit never happened | See "The bell" below: the quit asked to save all changes and waited. | `rich_input::send` |
| 577-restored-terminal-keeps-its-folder | **bug** | the right terminal restored in `repo` | ctrl-shift-w from the prompt editor raised the window's "save all changes" dialog; the third terminal never closed and was restored over beta's. See both bugs below. | `rich_input::send`; the keymap |

- **The bell (#627).** To send a line, the prompt editor wrote Ctrl-U first, to clear readline's
  line. On an empty line readline rings the bell at Ctrl-U, so every command sent from the editor
  rang it; the bell sets the view's `has_bell`, which Zed reads as dirty, and the key that would
  clear it (a key typed in the terminal) never comes while the editor has the keys. So after one
  command each terminal showed the dirty dot, a close asked "Do you want to save all changes",
  and a quit waited on that dialog. Fixed in `send`: no Ctrl-U while nothing was typed at the
  shell's prompt (the anchored model's `input_start` is unset), and the view's bell clears on
  send, as upstream clears it on any input from the view.
- **Ctrl-Shift-W (#627).** In gpui an unscoped binding matches at the deepest context, so from
  the prompt editor (`… > Terminal > MarleyRichInput > Editor`) Zed's unscoped
  `ctrl-shift-w: workspace::CloseWindow` outranks the Terminal context's
  `pane::CloseActiveItem`, and the key closed the window, not the terminal. Fixed in Marley's
  keymap: `MarleyRichInput > Editor` binds it to `pane::CloseActiveItem`. The other Terminal keys
  an unscoped binding outranks from the editor (arrows, Enter, Escape, paging, Ctrl-O, Ctrl-Q)
  have the editor's own meanings there, as they should.
- Seen with a probe scenario in the scratchpad (not kept): before, the dialog and then, with the
  bell fixed, the whole window closing; after both, the second terminal closed, nothing dirty.

### The failures again, alone (`20261001-052302`, then 535 alone)
- Seven passed. 535 passed its first fix and failed a later check the first run never reached:
  "the desktop banner still shows" counted banners holding `repo finished`, and since #538 a
  banner's title is the event's line, `repo: Claude finished`; stale (#538), the match updated.
  Alone again: PASS (5 such banners before the step, 6 after).
- Also minted: TICKET-637, #484's history suggestions in the prompt editor, the follow-up #627
  deferred and nobody minted (queued after #636).

### Phase 2 summary
- **Built:** the scenario updates above; `setting` in `script/e2e.sh`; in `rich_input::send`, no
  Ctrl-U on an empty line and the bell cleared on send; in Marley's keymap, Ctrl-Shift-W in
  `MarleyRichInput > Editor`.
- **Review:** each scenario change names the ticket that changed the UI; 511's fix also makes its
  three refusal checks test Merge's refusals again. The `send` change keeps Ctrl-U wherever the
  line may hold text (typed into the terminal since the prompt, or unknown), so a command never
  lands after an old line.
- **Gate:** RED once at gate:2 (clippy: a redundant closure, `TerminalView::clear_bell` passed
  directly), then `GATE GREEN [diff]`, 17 passed.
