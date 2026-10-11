# TICKET-749 — `marley <command>`

- **Ticket:** LOCAL #749 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [749-marley-commands.spec.md](../../pipeline/queued/749-marley-commands.spec.md)
- **Source ticket:** Chad, 2026-10-11: "we should think about commands we can run like marley
  <command> where it can open things inside marley. for example i could cd /whatever/folder and do
  marley agent-thread --provider=claude and it would pop open a new agent thread", and "yes to all
  of these". Folds in #721 (a quick prompt). Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** open

## Summary
`marley` is the app itself, and a second launch hands its paths to the running Marley as URLs
(#513). Verbs ride the same hand-off as `zed://marley/…` URLs, read before Zed's own argument
parser, which would refuse their flags: `agent-thread [--provider=NAME] [--folder=PATH]`,
`agent-cli [--provider=NAME]`, `terminal [PATH]`, `talk marley|rusty`, `threads`, `home`,
`rusty [SCREEN]`, `panel NAME`, `browser URL` and `help`. Each works in the folder it runs from (or
the one named) and lands in the group whose project holds that folder, else Default. `--provider`
takes short names (`claude`, `codex`, `zed`, `marley`, `rusty`) or an agent server's id, and
defaults to the agent last started. `help` and argument errors are answered by the launch; what
only the app can judge shows there as a toast. A Marley not yet running starts, restores its
session, then runs the verb; Zed would otherwise run the request in place of the restore. `marley`
typed in a Marley terminal reaches that Marley, also on a data directory of its own. A Hyprland key
bound to `marley agent-thread` is #721's quick prompt; the guide shows the binding.

## Acceptance
From a shell in a project's folder, `marley agent-thread --provider=claude` opens a Claude Code
thread tab in that project, working in that folder; `marley help` lists the verbs; with no Marley
running, the same command starts Marley with its session and the thread.
