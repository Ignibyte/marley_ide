# shellcheck shell=bash
# #891's visual check (a probe so far): the branch picker as the SearchMore board draws it. A
# scratch repository on `feature/one`, with `main` a commit ahead of it, `agent/rail-strip`
# checked out in a worktree, and a remote `origin` holding `main`.
# - `git: branch`: each branch's dot, its name, and its note: current, behind, worktree, remote
#   (`891-01-branches`, REQ-001).
# - Marley Day (`891-02-day`, REQ-001). Escape closes the picker; nothing switches.
compositor sway

STOP_AT=${STOP_AT:-0}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo origin=$E2E_WORK/origin.git
  mkdir -p "$home" "$repo"
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
  git -C "$repo" add -A
  git -C "$repo" commit -q -m "Start"
  git init -q --bare "$origin"
  git -C "$repo" remote add origin "$origin"
  git -C "$repo" push -q origin main
  git -C "$repo" fetch -q origin
  git -C "$repo" branch feature/one
  printf 'more\n' >>"$repo/README.md"
  git -C "$repo" commit -q -am "More on main"
  git -C "$repo" worktree add -q "$E2E_WORK/strip" -b agent/rail-strip
  git -C "$repo" checkout -q feature/one
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

  echo "== the branch picker"
  palette "git: branch" 3
  pointer_to 1500 950
  settle 1
  shot 891-01-branches
  if [[ $STOP_AT == 1 ]]; then
    return
  fi

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  shot 891-02-day
  press "" Escape
  settle 1
}
