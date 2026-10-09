---
status: promoted
created: 2026-10-09
ticket: TICKET-703 to TICKET-707
pipeline_spec: docs/planning/pipeline/completed/703-*, 704-*, 705-*, 706-*, 707-* (all shipped 2026-10-09)
---

# Full control of Zed through Marley's MCP server, with a security layer

- **Source:** Chad, 2026-10-09: "yes i would love to have full control over the zed ide. We do
  need to think about security across this too because if in theory we set up claude to have MCP
  access then if someone compromised someones machine and marley is running it provides an entry
  point into anything that marley connects so. however, that is the risk the user would need to
  accept in order to have fleet control over there system. We dont want restrictive approve every
  single thing that marley does sort of command otherwise it becomes uselss. But any sort of
  security would be ideal"

## What exists

Marley's MCP server (`marley_mcp`, `marley_workbench/src/mcp.rs`) serves terminals, ports,
the browser, docs, settings and keymap. Its guards today:
- Streamable HTTP on `127.0.0.1` only, a per-boot bearer in an owner-only (0600) endpoint file,
  an `Origin` check, and constant-time token compares.
- Outside programs get a named grant, read or read and write, from an explicit tool list
  (`clients.rs`, #524).
- Redaction of keys, tokens and passwords in everything agents read (#516).
- Terminal writes ask before an agent's first write (`agent_terminal_writes`, #525); `terminal_run`
  follows an allow and a deny list (#556); a consequential browser click pauses (#571); a settings
  or keymap change asks with Apply and Decline (#682, #686).

Missing: anything of Zed itself. No tool lists or reads open editors, edits a buffer, sees or
drives the Agent Panel's threads, or runs a palette command.

## Who we defend against

1. **Code already running as the user.** It can read the user's files and run commands without
   Marley. What Marley adds for it is reach: the fleet hosts, harness roots and Rusty that Marley
   connects to. Chad accepts that a user who wants fleet control takes this on. The defence is to
   keep that reach narrow: no tool ever returns a credential (SSH keys stay with the agent, Rusty's
   secrets behind its PIN, tokens redacted), and remote reach needs its own grant.
2. **Other local processes and web pages.** Already blocked: loopback only, the 0600 bearer, the
   `Origin` check.
3. **Prompt injection, the likely case.** An agent reads hostile text (a page, a file, an issue)
   and is steered into using Marley's tools. The defence is in the tiers below: what an injected
   agent can do without the user seeing it is limited, and everything it does is logged and can be
   stopped.
4. **The network.** Remote control goes through the relay, whose end-to-end encryption is an open
   decision (default: hardening).

## The tiers

Not "approve every action". Three tiers, set per area (editors, threads, actions):

| Tier | Examples | Default |
|---|---|---|
| **Read** | list editors, read a buffer, list and read threads, list actions | Allowed, logged. Reads are redacted. A file matching the secret globs (`.env*`, `*.pem`, `*.key`, `id_*`, `*credentials*`) is refused unless the user allows it |
| **Act** | open a file, edit a buffer (unsaved, one undo step), post into a thread, run a safe action | Ask once per agent session and project: "Claude Code in repo wants to edit files: Allow for this session / Always for this project / Deny". Free after that, and logged |
| **Sensitive** | answer another agent's permission prompt, run an action outside the safe list, save to disk | Ask every time. The user can lower it to ask-once |

Never allowed, whatever the settings say:
- an agent answering its own permission prompt, or one of a thread it started;
- changing `marley.agent_control` itself (settings changes already ask);
- quitting Marley or closing a window.

## The controls

- **One settings block,** `marley.agent_control`, with `editors`, `threads` and `actions`, each
  `off`, `ask_every`, `ask_first` or `allow`, plus `secret_globs`. Defaults as in the table.
- **An activity log.** None exists today (the server logs no call; System One's day log covers
  model-checked uses only). Every act and sensitive call: which agent, which tool, what it touched,
  when, allowed or refused. Shown in an Agent Activity card on Home and a tab of its own. An edit's
  row has Undo.
- **A kill switch.** `marley: stop agent control`, also a button on Home: every act and sensitive
  tool refuses until it is turned back on. Reads keep working.
- **Seen while it happens.** The acting agent's rail row shows a mark while it acts, as the
  browser's click pause does.
- **Outside clients.** Fleet peers and other programs get none of the new tools unless their grant
  names them (`clients.rs`'s explicit lists already work this way).

## Tickets, in order

1. **#703 The agent-control layer:** the tiers, `marley.agent_control`, the ask-once question, the
   activity log, the kill switch and the rail mark, proven on the existing terminal-write and
   settings tools first, so the new tools plug into it.
2. **#704 Editors, read and open:** `editor_list` (every open editor: path, project, dirty,
   language, cursor and selections), `editor_read` (a buffer's text by path or id, paged and
   redacted, the unsaved text included), `editor_open` (a file at a line, Act).
3. **#705 Editors, edit:** `editor_edit`, one undoable transaction in the open buffer
   (`Buffer::start_transaction`, `edit`, `end_transaction_with_source(Agent)`, as Zed's own
   agent edits), unsaved, Act; a review view of the change through `git_ui`'s `TextDiffView`;
   `editor_save`, Sensitive. Zed's inline Keep/Reject (`AgentDiff`) is not exported, so it stays
   out unless a later ticket takes the touchpoint.
4. **#706 Agent Panel threads:** `thread_list` (`ThreadMetadataStore` and the panels' live
   views, as the rail reads them), `thread_read` (`AcpThread::entries`, redacted), `thread_post`
   (Act; the message editor's `set_text` and `ThreadView::send`, so the panel's queue holds),
   `thread_answer` (another thread's pending permission, Sensitive, never its own). It keeps the
   panel's own gates: no answer to a sandbox escalation, and Allow refused while the request
   carries confusable Unicode.
5. **#707 Palette actions:** `action_run` by name with JSON arguments (`App::build_action`,
   then `FocusHandle::dispatch_action` on the target workspace). An explicit allowlist is Act
   (navigation, panels, splits, formatting, search); everything else is refused unless the user
   names it in `marley.agent_control.actions_allowed`. Upstream adds and renames actions at every
   merge, so a deny list would leak. Refused outright: quitting, closing or reloading windows,
   `zed::OpenBrowser` and `zed::OpenZedUrl`, tasks, debugger, REPL and notebook runs, every
   `terminal::SendText`/`SendKeystroke` (they skip `terminal_type`'s consent), every consent
   answer (`agent::AllowOnce` and kin, Marley's own Run/Allow actions), client grants, git
   pushes and destructive git, file deletes, extension and CLI installs, sign-in and sharing.

## Open for Chad (one at a time, after testing)

- Whether `editor_save` and `thread_answer` start at ask-every (the plan) or ask-once.
- Whether fleet peers may ever be granted the edit and thread tools.
