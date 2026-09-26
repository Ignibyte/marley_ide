# Warp once-over, 2026-09-25

Chad asked, with the sprint after the browser waves: "do a round at looking at warp see if
anything cool we dont know of we may want. dont try to stretch to find stuff just a good once
over." One agent read Warp's published docs (docs.warp.dev) against Marley's plan and queue. No
Warp source was opened (CONSTITUTION §20). Seven things came back that Marley has not planned.

| # | Warp feature | What it does | Why Marley might want it | Size | Source |
|---|---|---|---|---|---|
| 1 | Full Terminal Use | The agent attaches to a program already running (psql, gdb, a REPL, a dev server), reads the live screen, writes to the PTY and answers the program's prompts. A takeover control stops its writes until the user hands back; writes are approved on the first, on every one, or never. | Marley's terminal tools only read, and the plan's `terminal.run` starts commands rather than driving a running one. A screen read and a grant-gated type tool with a takeover key would give terminals the Browser tab's model: the agent acts where Chad watches. The Orca survey's receipted input (report 06, item 2) is the same seam. | M | docs.warp.dev/agents/capabilities/full-terminal-use/ |
| 2 | Secret Redaction | A recommended set of regexes plus the user's own finds keys, passwords, IPs and personal data and redacts them before anything reaches Warp's servers or a model; on screen they are struck through or starred, with reveal and copy in a tooltip. Off by default. | D15 redacts the browser's network and trace readers, but `terminal_read` hands agents raw output, `env` dumps included. | S for what agents read, M with masking on screen | docs.warp.dev/support-and-community/privacy-and-security/secret-redaction/ |
| 3 | Warpify subshells | In a nested shell, `docker exec` into bash, zsh or fish, `poetry shell` or `gcloud compute ssh`, Warp offers to run a setup script so blocks and the input editor work there; an rc line makes it automatic, and SSH has a wrapper. | Marley's hooks report a subshell flag, but blocks stop at the first `ssh` or `docker exec`, and much of the work on this box is SSH. The #474 nonce would have to travel with the bootstrap. | M | docs.warp.dev/terminal/warpify/subshells/ |
| 4 | Tab Configs | A TOML file per tab: directory, startup commands, pane splits, a shell per pane, colour, title, and parameters asked for at open (text, a branch, a repository). It opens from the + menu or a `warp://tab_config/<name>` link. | The rail's + opens one thing at a time. A project config could open the dev server, Claude Code and a Browser tab on the server's URL in one click. Orca's `defaultTabs` in `orca.yaml` is the same idea for new worktrees (docs/orca_architecture/05, item 6). | M | docs.warp.dev/terminal/windows/tab-configs/ |
| 5 | Block filtering and the sticky command header | A filter shows only the lines of a block that match text or a regex (case, invert, context lines) and deletes nothing. The sticky header pins a long block's command to the top of the pane while you scroll; a click jumps to the block's start. | Neither is in T1 to T7. Filtering in place needs T5's display-row map; an overlay does not. | S each, M to filter in place | docs.warp.dev/terminal/blocks/block-filtering/, docs.warp.dev/terminal/blocks/sticky-command-header/ |
| 6 | Runnable commands in the Markdown viewer | Shell code blocks get an icon that puts the command into the active terminal's input without running it; Ctrl-Up and Ctrl-Down select blocks, Ctrl-Enter inserts. | Zed's Markdown preview has nothing like it, and runbooks are mostly commands to copy. | S | docs.warp.dev/terminal/more-features/markdown-viewer/ |
| 7 | Vertical tabs with more metadata | Rows can show the branch's pull request, its status (through the GitHub CLI) and diff stats, with a hover card for the rest. | The rail has the layout and agent state. With parallel worktree agents coming (#510), PR state and +/- counts on each row show whose work is ready. | S | docs.warp.dev/terminal/windows/vertical-tabs/ |

Ruled out, because Marley has it, plans it or gets it elsewhere: rich input, notifications,
voice, attaching files, the agent bar and the vertical-tab look (#477 to #482, #468); Interactive
Code Review (the planned per-turn diffs and worktree review, though its one extra idea, line
comments batched and sent to the CLI agent in one pass, is worth keeping for #509 and #511);
worktrees; blocks as context (agents read blocks over MCP, #491); Remote Control (needs Warp's
cloud and a login, and Claude Code has its own); browser use, testing and recordings (#488 to
#499, with the Playwright export queued as #506); autosuggestions (#484); completions and the fzf
and atuin handoff (T3, T6); Warp Drive, workflows, notebooks and sharing (out of scope); Oz,
Factories and cloud agents (Warp's cloud; prong 2 is the local counterpart); synchronized inputs,
agent-prompt detection at the shell prompt, tab groups and pinned tabs (low value).

None of the seven is ticketed yet. Items 1 and 2 pair with the Orca survey's terminal identity
and receipted input; item 4 pairs with its repo-declared first tabs; item 7 with #510.

Chad took all seven the same evening ("1.) lose this idea lets do it 2.) love it need it …"). Item
2 shipped first, as #516, for what Marley's tools hand agents; the on-screen masking stays for
later.
