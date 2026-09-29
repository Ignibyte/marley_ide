# Show the launch approval verbatim — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-592-launch-approval-verbatim.md
- **Pipeline spec:** 592-launch-approval-verbatim.spec.md

## Phase 1 — Plan
- **Request:** #527's after-the-fact visual check, shot 527-02-approve: the approval reads
  `terminal “dev server”: python3 -m http.server 8766 –bind 127.0.0.1 agent claude, split right
  browser, split down: http://127.0.0.1:8766/`, one run-on paragraph with an en dash and curly
  quotes. The text is right in `launch.rs` (`text()` joins the items with `\n`); the prompt
  renders it as Markdown.
- **Classification / tier:** bug, one Marley file, a trust gate's display.
- **Recall (§18.3):** nothing in `docs/planning/knowledge/` on Markdown in prompts. #527's spec
  names Zed's prompt on Linux (`ui_prompt.rs:22`) but not that it renders Markdown. The brain
  (consultation dc3807406dbc4e26b9138a51f686457c) had nothing on this seam, only unrelated
  follow-ups due.
- **Discovery:** `crates/marley_workbench/src/launch.rs` `run()`, the `window.prompt` call's
  detail. `crates/ui_prompt/src/ui_prompt.rs:47` wraps the detail in `Markdown::new`;
  `crates/markdown/src/parser.rs:17` enables smart punctuation. Of Marley's six
  `window.prompt` calls, only this one shows a repository's text.
- **Decisions:** D1 (the hash stays on the plain text), D2 (a backtick fence longer than any
  run inside).

### Design
- Approach: `fn verbatim(text: &str) -> String` in `launch.rs` returns the text inside a
  backtick fence of `max(longest run, 2) + 1` backticks; `run()` passes `Some(&verbatim(&text))`
  as the detail and keeps hashing `text`. A code block in Zed's Markdown is literal, keeps its
  line breaks and wraps (the prompt's style leaves `code_block_overflow_x_scroll` false).
- File manifest: `crates/marley_workbench/src/launch.rs` (Marley crate). No ledger row.

### Visual check plan
| REQ | The scenario and the shot |
|---|---|
| REQ-001 | 527's scenario, first run: 527-02-approve shows three lines, one per item. |
| REQ-002 | The same shot: `--bind` with two hyphens, `"dev server"` in straight quotes. An HTML comment or emphasis in a command rests on the review: a code block has no markup. |
| REQ-003 | Review of `verbatim()`: the fence outlasts any run inside. |
| REQ-004 | Review: the hash input is `text`, unchanged. |
| (the changed file) | 527-05-changed shows the new port's text the same way. |

### Risks
- A long command wraps inside the 320 px prompt; a wrapped line and a new item are told apart
  by the item's leading kind (`terminal`, `agent`, `browser`).
- The code block may show Zed's copy button on hover, which is harmless.

## Phase 2 — Code
- Built: `verbatim()` in `launch.rs`, and `run()` passes `Some(&verbatim(&text))` to the prompt.
  The approval key and its hash still take `text`. No deviation from the plan.
- Review of the diff: a text with no backticks gets a three-backtick fence; a run of two gets
  three; a run of three gets four, and CommonMark closes a fence only on a run at least as long
  as the one that opened it. A tilde line cannot close a backtick fence. A backtick is one
  byte, so `str::len` counts the run. No entity is touched; nothing new can fail.
- Gate: `just gate-diff`, GATE GREEN [diff], 16 passed, 0 failed (log in the scratchpad).

## Phase 3 — Test
- **The scenario:** `script/e2e/527-project-launch-configs.sh` (#527's visual check, `compositor
  sway`), with its dev server's command now ending in a shell comment that holds Markdown:
  `# <!-- hidden --> *kept*` and three backticks. The shell ignores the comment, so the server
  still runs.
  One run on the new debug build, exit 0; the focus report: 0 Marley windows on Hyprland before
  and after, the run's sway stopped.
- **527-02-approve** (cropped at 3x): "Run Dev?" over four lines: `terminal "dev server": python3
  -m http.server 8766` wrapping to `--bind 127.0.0.1 # <!-- hidden --> *kept*` and the three
  backticks, then
  `agent claude, split right`, then `browser, split down: http://127.0.0.1:8766/`. The items
  break where the text breaks (REQ-001); two hyphens, straight quotes, the HTML comment, the
  asterisks and the three backticks all shown as written (REQ-002, REQ-003).
- **527-05-changed:** "Dev changed since you approved it. Run it?" with the port-8767 text laid
  out the same way.
- **The rest, unchanged from #527's run:** 527-01 (Launch lists Dev), 527-03 (the dev server,
  the stand-in Claude split right with the focus, the Browser tab split down on "The dev
  server answers"), 527-04 (no question the second time), 527-06 (the disabled entry
  `.zed/marley.json: Dev: item 1: needs one of terminal, agent and browser`).
- **REQ-004** rests on the review: the hash still takes `text`. This run started from a fresh
  store, so it asked first, as it should.
- **Seen, not in scope:** a second run reuses the Browser tab already on the same URL (#503's
  rule in `open_url_tab`) and moves it into the new layout, so the first layout loses its tab;
  the second dev server fails because the first holds the port, which is the config's doing.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Fixed, #592); `docs/marley_architecture/marley_workbench.md`, the launch
  section names `verbatim`. No Zed path touched, so no ledger row.
- **Knowledge:** F-claude-592-the-launch-approval-was-rendered-as-markdown-001,
  PR-claude-a-repositorys-text-in-a-zed-prompt-goes-in-a-code-block-001. #527's completed notes
  gained its after-the-fact visual check, which found this.
- **Brain:** consultation dc3807406dbc4e26b9138a51f686457c closed with no decision (a bug fix;
  the rule lives in the ledger).
- **Closed:** TICKET-592 moved to `tickets/closed/`; it never had a BACKLOG row.
- **Commit:** with 527's scenario, the check that found the bug and shows the fix.
