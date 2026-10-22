# shellcheck shell=bash
# #851's visual check: New Agent in Worktree as the Worktrees board draws it. A scratch repository
# on `main` with a commit and a `.worktreeinclude`, bypass for Claude Code in the project, and fake
# `claude` and `codex` CLIs that start nothing.
# - The project's +, New Agent in Worktree, Claude Code: the card with its badge, title, First
#   prompt, the branch chip, Also start, the facts and the footer (`851-01-prompt`, REQ-001).
# - Codex ticked: two branch chips and "Start 2 agents" (`851-02-two`, REQ-001).
# - Marley Day (`851-03-day`, REQ-001).
# Escape closes the card at the end, so no worktree is made.
compositor sway

STOP_AT=${STOP_AT:-0}
# The project's +, New Agent in Worktree in its menu, and Claude Code in that submenu, from the
# probe runs' shots.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
ENTRY_X=${ENTRY_X:-140}
ENTRY_Y=${ENTRY_Y:-279}
CLAUDE_X=${CLAUDE_X:-325}
CLAUDE_Y=${CLAUDE_Y:-282}
# Codex's box under Also start, once the card is open.
CODEX_X=${CODEX_X:-606}
CODEX_Y=${CODEX_Y:-300}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  for program in claude codex; do
    printf '#!/bin/sh\nexit 0\n' >"$bin/$program"
    chmod +x "$bin/$program"
  done
  export MARLEY_CLAUDE=$bin/claude MARLEY_CODEX=$bin/codex PATH="$bin:$PATH"
  git init -q -b main "$repo"
  printf '# repo\n' >"$repo/README.md"
  printf '.env\n.zed/settings.json\n' >"$repo/.worktreeinclude"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "Start"
  profile_setting marley.agent_permissions_by_project "{\"$repo\": {\"claude_code\": \"bypass\"}}"
  open_path "$repo"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== the project's +, New Agent in Worktree"
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  click "$ENTRY_X" "$ENTRY_Y"
  settle 2
  shot 851-00-menu
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Claude Code in a new worktree"
  click "$CLAUDE_X" "$CLAUDE_Y"
  settle 3
  type_text "Make the rail shrink to a strip of project logos."
  settle 1
  pointer_to 1500 950
  settle 1
  shot 851-01-prompt
  if [[ $STOP_AT == 2 ]]; then
    return
  fi

  echo "== Codex ticked"
  click "$CODEX_X" "$CODEX_Y"
  settle 1
  pointer_to 1500 950
  settle 1
  shot 851-02-two

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 851-03-day
  press "" Escape
  settle 1
}
