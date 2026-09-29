# TICKET-587 — Claude Code's trust question in a new worktree, answered for the user

- **Ticket:** LOCAL #587 (feature, prong 2: worktree agents, a follow-up of #510)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (to be drafted into `pipeline/queued/`)
- **Source ticket:** #510's check for Chad (`../../pipeline/completed/510-worktree-agents.notes.md`, "Not reachable by a scenario"), answered by Chad on 2026-09-28: "this is true so we need to handle that with a popup if it does show the user can click or assume that since zed asks to trust the folder then we auto select it on for them via tmux or rustal harness".
- **Status:** open

## Summary
Claude Code saves its folder trust per folder, so the first launch in each new worktree #510
makes stops on its "trust the files in this folder" question, and the first prompt waits on the
command line until the user answers it in the terminal. Zed has already asked the user to trust
the repository, and its worktree service carries that trust to the new worktree. This slice
answers Claude Code's question from Zed's trust: while Zed trusts the worktree's folder, Marley
accepts it for the user in the agent's own terminal, which it owns (no tmux needed; the rustal
harness takes this over once it runs Claude Code sessions); when Zed does not trust it, or Marley
cannot tell the question is showing, the question shows in Marley as a prompt the user can click,
and the click answers it in the terminal. Plan decides how the question is recognized (the
terminal's screen, the plugin's events, or Claude Code's own record of trusted folders, which
#510's AD rejected writing to) and whether the automatic answer is a setting.

## Acceptance
Starting Claude Code through New Agent in Worktree in a folder Zed trusts runs its first prompt
with no trust question left for the user to answer; in a folder Zed does not trust, Marley shows
the question with a button, and pressing it lets the prompt run.
