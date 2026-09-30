# shellcheck shell=bash
# #598's e2e test: New Agent in Worktree sits directly under New Agent Thread in a project's +. A
# scratch git repository with a commit, a fake `claude` first on the PATH so the CLI list is never
# empty, and a `.zed/marley.json` with one launch config so the Launch header shows. A click on
# the project's + opens the menu, shot as `menu`: New Terminal, New Browser Tab, New Agent Thread,
# New Agent in Worktree, then the Agent CLIs header with its CLIs, then Launch with `Dev`
# (REQ-001, REQ-002). Down three times and Return open New Agent in Worktree's submenu, shot as
# `submenu`: its CLIs as before, and Claude Code where the worktree scenarios click it.
compositor sway

# The project's +, as 510's scenario finds it, and New Agent in Worktree in its menu.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-96}
WORKTREE_STEPS=${WORKTREE_STEPS:-3}
WORKTREE_ENTRY_X=${WORKTREE_ENTRY_X:-150}
WORKTREE_ENTRY_Y=${WORKTREE_ENTRY_Y:-193}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo/.zed"
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

  git init -q -b main "$repo"
  printf 'hello\n' >"$repo/README.md"
  cat >"$repo/.zed/marley.json" <<'JSON'
{ "launch": { "Dev": { "items": [{ "terminal": "" }] } } }
JSON
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "Start"

  printf '#!/bin/sh\nexec sleep 600\n' >"$bin/claude"
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== the project's + menu"
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  shot menu

  echo "== New Agent in Worktree's submenu, reached as the worktree scenarios reach it"
  local step
  for ((step = 0; step < WORKTREE_STEPS; step++)); do
    press "" Down
  done
  press "" Return
  settle 1
  pointer_to "$WORKTREE_ENTRY_X" "$WORKTREE_ENTRY_Y"
  settle 1
  shot submenu
  press "" Escape
  settle 1
}
