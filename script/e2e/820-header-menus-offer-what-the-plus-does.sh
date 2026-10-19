# shellcheck shell=bash
# #820's visual check: a header's right-click offers what its + does. `repo` is a git repository,
# so its + offers New Agent in Worktree.
# - `repo`'s header: New Terminal, New Browser Tab, New Agent…, Reopen Thread…, New Agent in
#   Worktree, then its own entries (`820-01-project-menu`, REQ-001).
# - Default's header: the same first entries, then its line (`820-02-default-menu`, REQ-002).
# - The rail's empty space: New Group…, New Terminal, New Browser Tab, New Agent…, Reopen
#   Thread…, no Agent CLIs list (`820-03-empty-menu`, REQ-003).
# - New Terminal from `repo`'s right-click: a second terminal row under `repo`
#   (`820-04-terminal-from-menu`, REQ-004).
compositor sway

# The rail's points on the 1600 x 1000 output, from the first run's shots: Default's and `repo`'s
# headers, the empty space below the rows, and New Terminal in `repo`'s menu.
HEADER_X=${HEADER_X:-60}
DEFAULT_Y=${DEFAULT_Y:-88}
REPO_Y=${REPO_Y:-136}
EMPTY_Y=${EMPTY_Y:-700}
NEW_TERMINAL_X=${NEW_TERMINAL_X:-110}
NEW_TERMINAL_Y=${NEW_TERMINAL_Y:-158}

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b menus "$E2E_WORK/repo"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== repo's header"
  click "$HEADER_X" "$REPO_Y" right
  settle 1
  shot 820-01-project-menu
  press "" Escape
  settle 1

  echo "== Default's header"
  click "$HEADER_X" "$DEFAULT_Y" right
  settle 1
  shot 820-02-default-menu
  press "" Escape
  settle 1

  echo "== the empty space"
  click "$HEADER_X" "$EMPTY_Y" right
  settle 1
  shot 820-03-empty-menu
  press "" Escape
  settle 1

  echo "== New Terminal from repo's right-click"
  click "$HEADER_X" "$REPO_Y" right
  settle 1
  click "$NEW_TERMINAL_X" "$NEW_TERMINAL_Y"
  settle 3
  shot 820-04-terminal-from-menu
}
