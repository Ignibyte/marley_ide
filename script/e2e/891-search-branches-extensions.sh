# shellcheck shell=bash
# #891's visual check: project search's count, the branch picker's dots and notes, and the
# extensions' cards, as the SearchMore board draws them. A scratch repository on `feature/one`;
# `main` one commit behind `origin/main`; `release` only on `origin`; `agent/rail-strip` checked
# out in a worktree; three files that hold `rail_order`.
# - `pane: deploy search`, `rail_order`: "N matches in 3 files" in the toolbar (`891-01-search`,
#   REQ-001).
# - `git: branch`: each branch's dot and note: current, 1 behind, worktree, remote
#   (`891-02-branches`, REQ-002), and in Marley Day (`891-04-day`, REQ-002).
# - `zed: extensions`: the cards with their tiles, the installed one (the profile's HTML) saying
#   Installed in green (`891-03-extensions`, REQ-003).
compositor sway

STOP_AT=${STOP_AT:-}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo origin=$E2E_WORK/origin.git
  mkdir -p "$home" "$repo/src"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  git init -q -b main "$repo"
  printf '# repo\n' >"$repo/README.md"
  printf 'let order = rail_order();\nfn rail_order() {}\n' >"$repo/src/rail.rs"
  printf 'rail_order = "window"\n' >"$repo/src/settings.toml"
  printf 'The rail_order setting.\n' >"$repo/src/guide.md"
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "Start"
  git init -q --bare "$origin"
  git -C "$repo" remote add origin "$origin"
  git -C "$repo" push -q -u origin main
  git -C "$repo" branch -q feature/one
  printf 'more\n' >>"$repo/README.md"
  git -C "$repo" commit -q -am "More on main"
  git -C "$repo" push -q origin main main:release
  git -C "$repo" worktree add -q "$E2E_WORK/strip" -b agent/rail-strip
  git -C "$repo" checkout -q feature/one
  git -C "$repo" branch -q -f main main~1
  git -C "$repo" fetch -q origin
  profile_setting marley.tmux_hosts '[]'
  open_path "$repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 2
  type_text "$1"
  settle 3
  press "" Return
  settle "${2:-2}"
}

steps() {
  settle 15
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== project search"
  palette "pane: deploy search" 3
  type_text "rail_order"
  press "" Return
  settle 4
  pointer_to 1500 950
  settle 1
  take 891-01-search || return 0
  press "" Escape
  settle 1

  echo "== the branch picker"
  palette "git: branch" 3
  pointer_to 1500 950
  settle 1
  take 891-02-branches || return 0
  press "" Escape
  settle 1

  echo "== the extensions"
  palette "zed: extensions" 6
  pointer_to 1500 950
  settle 1
  take 891-03-extensions || return 0

  echo "== Marley Day, the branch picker"
  profile_setting theme '"Marley Day"'
  settle 4
  palette "git: branch" 3
  pointer_to 1500 950
  settle 1
  take 891-04-day || return 0
  press "" Escape
}
