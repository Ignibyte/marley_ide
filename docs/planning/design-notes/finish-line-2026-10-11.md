# The finish line: what is left from the surveys (2026-10-11)

Chad, 2026-10-11: "lets talk about the remaining things we found that were comparable things from
like t3 and other ide. I want to decide on those now because we are pretty close to a finished
product". Every idea still open from T3 Code (`docs/t3code_architecture/`), MonoCode
(`docs/planning/intake/monocode-findings.md`) and plannotator
(`docs/planning/intake/plannotator-findings.md`) was merged into one list, each with a call: Now
(before the finish), Later, or Skip. His answers are quoted per row. Then: "Lets spec the rest of
these out".

## Now: built after the tabs batch (#743 to #749)

| # | Idea | Chad | Ticket |
|---|---|---|---|
| 1 | Restarts that lose nothing: Codex resumes, a cut-off turn gets Continue, restored scrollback | "this is nice" | #750, #751 |
| 2 | A usage limit as a state, Continue at Reset, a usage meter | "yes" | #715 |
| 3 | Codex gets what Claude Code gets: push, answers from Needs you, turn checkpoints | "yes" | #752, #753 |
| 4 | Old CLI sessions in Threads, reopened with `--resume` | "yes" | #754 |
| 6 | LSP tools for every agent | "yes" | #712 |
| 7 | Marley's own identity, and its own remote server | "yes" | #445 (last, the release step) |
| 8 | The rail cleanup, Codex's config written safely, a token per terminal, Chromium survives a crashed page | "yes to all" | #713, #755 |
| 17 | One page for every CLI's MCP servers, and CLI version checks | "build now if we can" | #720 |
| 18 | The browser tools agents lack | "build now" | #756 |
| 5 | Undo a turn, after Codex's turn checkpoints | "meh not sure. convince me of this or not", then, after the case for and against, "ok we can build it" | #714 |
| 9 | Follow Omarchy's theme: one template and a setting, off by default; Omarchy only, since macOS and Windows already follow the system's light and dark | "if this is easy to do and will be across mac, windows, and omarchy then lets do it. if not i dont want to maintain this. if its just here is an omarchy theme then maybe", then "lets go with 9 as well" | #759 |
| — | A remote harness's manager as a thread, and every gap around it | "ok we can build it. once things are spec and then add the things for the missing manager." | #758 |

## Later: specced now, built after the finish

| # | Idea | Chad | Ticket |
|---|---|---|---|
| 10 | Plan review in a tab, versions, question cards | "yes" | #726 to #728 |
| 11 | Notes on Markdown or the last reply, findings as diff notes, viewed marks, a feedback template, guided review | "yes" | #729 to #733 |
| 12 | A second opinion and a hand-off between agents | "yes, with a star new thread with context would be cool" | #717 |
| 13 | Agents that start agents; one prompt to several agents | "yes as long as its tracking and not lost" | #716 |
| 14 | Failed checks sent to an agent, then a GitHub inbox | "ok" | #718 |
| 15 | Scheduled runs | "harness yes" | #719 moves to rustal-harness; Marley shows the runs |
| 16 | Claude Code's Remote Control switch | "yes and shelve" | #757, shelved |

Asked the same day: "is there a way to do remote agent threads. meaning if manager lives on a
remote server and we dont want to steer them through the cli. Using or not using our harness
(preferrable our harness)". #742 already opens a remote seat as a thread through the harness; #758
(in the Now table) adds a Manager entry per harness, seat threads from remote windows, and a
by-hand check on a real host.

## Folded and skipped

- **19, a quick prompt (#721):** folded into `marley <command>` (#749): a Hyprland key bound to
  `marley agent-thread` does the same. Chad: "yes".
- **20, skipped:** permission-mode presets (each CLI keeps its own prompts), long pastes sent as
  files, subagent rows, extra port-row detail, pages agents draw, branch names from the first
  prompt, and a `marley: triage` command. Chad: "yes".
