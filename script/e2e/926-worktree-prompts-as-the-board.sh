# shellcheck shell=bash
# #926's visual check: the worktree Merge and Remove prompts as the Worktrees board words them. A
# scratch repository on `main`; two worktrees made with New Agent in Worktree, as #589's are, since
# Remove acts only on a worktree Zed made: the first then gets two commits and stays clean, the
# second a commit `main` lacks and three changes not committed. A stand-in `claude`, first on the
# PATH, starts nothing. No agent CLI runs.
# - `926-01-merge`: the first worktree's Merge 2 commits into main…: where the branch goes, then
#   four ticked checks (REQ-001).
# - `926-04-day`: the same prompt in Marley Day (REQ-001).
# - `926-02-remove-stays`: the second's Remove…: "It has 3 changes not committed. They go with the
#   folder. The branch stays: main does not have its commits." and Remove Anyway (REQ-002).
# - `926-03-remove-goes`: after the merge, the first's Remove…: "The branch goes: main has its
#   commits." (REQ-002).
compositor sway

STOP_AT=${STOP_AT:-}
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-200}
# The project's +, New Agent in Worktree in its menu, and Claude Code in that submenu, as #851's.
PLUS_X=${PLUS_X:-236}
PLUS_Y=${PLUS_Y:-136}
ENTRY_X=${ENTRY_X:-140}
ENTRY_Y=${ENTRY_Y:-279}
CLAUDE_X=${CLAUDE_X:-325}
CLAUDE_Y=${CLAUDE_Y:-282}
# From the probe run's shots: the main checkout's row, then the two worktrees' rows.
ROW_X=${ROW_X:-120}
MENU_X=${MENU_X:-248}
MAIN_ROW_Y=${MAIN_ROW_Y:-180}
# A worktree's name is chosen at random and the rail lists them by name, each with its agent's
# terminal row under it.
UPPER_ROW_Y=${UPPER_ROW_Y:-234}
LOWER_ROW_Y=${LOWER_ROW_Y:-338}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

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
  cp "$E2E_WORK/gitconfig" "$home/.gitconfig"
  # A stand-in Claude Code that starts nothing and keeps its terminal open.
  printf '#!/bin/sh\necho "Claude Code (stand-in)"\nexec cat >/dev/null\n' >"$bin/claude"
  chmod +x "$bin/claude"
  export MARLEY_CLAUDE=$bin/claude PATH="$bin:$PATH"
  git init -q -b main "$repo"
  printf 'the first line\nthe second line\n' >"$repo/README"
  printf 'one\ntwo\nthree\n' >"$repo/notes.txt"
  printf 'a\n' >"$repo/list.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.agent_permissions_by_project "{\"$repo\": {\"claude_code\": \"bypass\"}}"
  open_path "$repo"
}

# Makes a worktree with New Agent in Worktree and Claude Code, its first prompt `$1`.
new_agent() {
  click "$PLUS_X" "$PLUS_Y"
  settle 2
  click "$ENTRY_X" "$ENTRY_Y"
  settle 2
  click "$CLAUDE_X" "$CLAUDE_Y"
  settle 3
  type_text "$1"
  press "" Return
  settle 10
}

# The linked worktrees' folders, one per line.
linked() {
  repo_git worktree list --porcelain | sed -n 's/^worktree //p' | grep -vx "$E2E_WORK/repo" || true
}

in_main() {
  click "$ROW_X" "$MAIN_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

row_menu() {
  click "$MENU_X" "$1" right
  settle 1
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  local first second first_branch
  settle 15
  # Trusts the scratch repository.
  press "" Return
  settle 4

  echo "== two worktrees from New Agent in Worktree"
  new_agent "Tidy the notes"
  first=$(linked)
  echo "first: $first"
  new_agent "Keep the list"
  second=$(linked | grep -vx "$first")
  echo "second: $second"
  [[ -n $first && -n $second ]] || { echo "the worktrees were not made"; return 1; }
  sed -i 's/^one$/one, tidied/' "$first/notes.txt"
  git -C "$first" commit -q -am "Tidy the notes"
  printf 'tidy\n' >"$first/tidy.txt"
  git -C "$first" add tidy.txt
  git -C "$first" commit -q -m "Add the tidy list"
  printf 'kept\n' >"$second/kept.txt"
  git -C "$second" add kept.txt
  git -C "$second" commit -q -m "Keep the list"
  printf 'changed\n' >>"$second/README"
  printf 'changed\n' >>"$second/notes.txt"
  printf 'changed\n' >>"$second/list.txt"
  repo_git worktree list
  local first_name second_name first_row second_row
  first_name=$(basename "$(dirname "$first")")
  second_name=$(basename "$(dirname "$second")")
  if [[ $(printf '%s\n%s\n' "$first_name" "$second_name" | LC_ALL=C sort | head -1) == "$first_name" ]]; then
    first_row=$UPPER_ROW_Y second_row=$LOWER_ROW_Y
  else
    first_row=$LOWER_ROW_Y second_row=$UPPER_ROW_Y
  fi
  in_main "git log --oneline --all --graph"
  settle 6
  away
  take 926-00-rail || return 0

  echo "== the first's Merge"
  row_menu "$first_row"
  press "" Home
  press "" Down
  press "" Return
  settle 3
  away
  take 926-01-merge || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 926-04-day
  profile_setting theme '"Marley Night"'
  settle 4

  echo "== Merge"
  press "" Return
  settle 5
  first_branch=$(git -C "$first" rev-parse --abbrev-ref HEAD)
  expect "main has the first's commits" repo_git merge-base --is-ancestor "$first_branch" main

  echo "== the second's Remove"
  row_menu "$second_row"
  press "" End
  press "" Return
  settle 3
  away
  take 926-02-remove-stays || return 0
  press "" Escape
  settle 2

  echo "== the first's Remove"
  row_menu "$first_row"
  press "" End
  press "" Return
  settle 3
  away
  shot 926-03-remove-goes
  press "" Escape
  settle 2
  expect "both worktrees are still there" test -d "$first" -a -d "$second"
}
