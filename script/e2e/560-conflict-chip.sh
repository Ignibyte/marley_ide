# shellcheck shell=bash
# #560's e2e test: the drift chip on worktree rows. A scratch repository on `main` with two
# worktrees made the way #510 makes them, each on `agent/<name>` from `main` with its base in the
# config: `ok` changes the first line of `notes.txt`, `clash` the first line of `README`. Up to
# date, neither row has a chip (REQ-005). Two commits on `main`, one to README's first line and
# one to the end of `notes.txt`: `ok` reads `2 behind` (REQ-001) and `clash` `1 conflict` in the
# warning color (REQ-002), whose tooltip names the base, its commit and `README` (REQ-003). Each
# worktree merging `main` loses its chip (REQ-004, REQ-005). With `merge-tree --write-tree`
# refused, one more commit on `main` gives both rows `1 behind` and one line in Marley's log
# (REQ-007). The git the chip runs is a wrapper, named by `MARLEY_GIT`, that logs its arguments;
# the refs, the indexes and the checkouts are compared before and after a run, and no fetch runs
# (REQ-006).
compositor sway

# The rows and the clash row's chip, from the first run's shots.
FIRST_ROW_Y=${FIRST_ROW_Y:-180}
SECOND_ROW_Y=${SECOND_ROW_Y:-224}
CHIP_X=${CHIP_X:-222}
MAIN_ROW_X=${MAIN_ROW_X:-120}
MAIN_ROW_Y=${MAIN_ROW_Y:-136}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

# What a drift run must leave as it was: the branches, each checkout's status against its
# `HEAD`, the worktrees and the index files.
git_state() {
  local checkout
  repo_git for-each-ref --format='%(refname) %(objectname)' refs/heads
  repo_git worktree list --porcelain
  for checkout in "$E2E_WORK/repo" "$E2E_WORK/ok" "$E2E_WORK/clash"; do
    echo "status of ${checkout##*/}:"
    git -C "$checkout" status --porcelain --untracked-files=no
  done
  sha256sum "$E2E_WORK/repo/.git/index" "$E2E_WORK"/repo/.git/worktrees/*/index |
    sed "s|$E2E_WORK/||"
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo real_git
  mkdir -p "$home" "$repo"
  cat >"$home/.bashrc" <<'RC'
PS1='$ '
RC
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  # The terminal's commits find the identity whether or not the variables reach its shell.
  cp "$E2E_WORK/gitconfig" "$home/.gitconfig"

  git init -q -b main "$repo"
  printf 'the first line\nthe second line\n' >"$repo/README"
  printf 'one\ntwo\nthree\nfour\nfive\n' >"$repo/notes.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  # As #510 makes a worktree agent's: a new branch from the base, tracking nothing, and the base
  # in the config.
  local name
  for name in ok clash; do
    repo_git worktree add -q --no-track -b "agent/$name" "$E2E_WORK/$name" main
    repo_git config "branch.agent/$name.base" main
  done
  sed -i 's/^one$/one, from ok/' "$E2E_WORK/ok/notes.txt"
  git -C "$E2E_WORK/ok" commit -q -am "ok's change"
  printf 'the first line, from clash\nthe second line\n' >"$E2E_WORK/clash/README"
  git -C "$E2E_WORK/clash" commit -q -am "clash's change"
  repo_git worktree list

  real_git=$(type -P git)
  sed -e "s|@GIT@|$real_git|" -e "s|@LOG@|$E2E_WORK/git.log|" \
    -e "s|@FLAG@|$E2E_WORK/no-write-tree|" >"$E2E_WORK/marley-git" <<'WRAPPER'
#!/usr/bin/env bash
# The chip's git: logs each call's arguments, and refuses merge-tree as a git before 2.38 does
# while the flag file exists.
printf '%q ' "$@" >>"@LOG@"
printf '\n' >>"@LOG@"
if [[ -e "@FLAG@" ]]; then
  for argument in "$@"; do
    if [[ $argument == merge-tree ]]; then
      echo "error: unknown option \`write-tree'" >&2
      echo "usage: git merge-tree [--write-tree] [<options>] <branch1> <branch2>" >&2
      exit 129
    fi
  done
fi
exec "@GIT@" "$@"
WRAPPER
  chmod +x "$E2E_WORK/marley-git"
  export MARLEY_GIT=$E2E_WORK/marley-git
  git_state >"$E2E_WORK/state-0.txt"
  open_path "$repo"
}

# Types `$1` into the main checkout's terminal and runs it.
in_main() {
  click "$MAIN_ROW_X" "$MAIN_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

steps() {
  settle 12
  # Trusts the scratch repository. Zed flips a repository's trust without an event, so the
  # rail's next rebuild, here the terminal's output, schedules the first run.
  press "" Return
  settle 2
  in_main "git log --oneline --all --graph"
  settle 6

  echo "== up to date: no chip"
  shot 560-01-clean
  expect "the chip's git ran" test -s "$E2E_WORK/git.log"
  git_state >"$E2E_WORK/state-1.txt"
  expect "the runs moved no ref and changed no index or checkout" \
    diff "$E2E_WORK/state-0.txt" "$E2E_WORK/state-1.txt"

  echo "== two commits on main: ok behind, clash conflicting"
  in_main "printf 'the first line, from main\\nthe second line\\n' > README && git commit -qam 'main 1' && echo six >> notes.txt && git commit -qam 'main 2'"
  # REQ-002 and REQ-004 give the chip five seconds.
  settle 4
  shot 560-02-drift
  echo "main is at $(repo_git rev-parse --short main)"

  echo "== the conflict's tooltip"
  pointer_to "$CHIP_X" "$FIRST_ROW_Y"
  settle 2
  shot 560-03-tooltip

  echo "== each worktree merges main: no chip"
  in_main "git -C ../clash merge -q --no-edit -X theirs main && git -C ../ok merge -q --no-edit main"
  settle 4
  shot 560-04-merged

  echo "== merge-tree refused: the behind count alone, and one log line"
  touch "$E2E_WORK/no-write-tree"
  in_main "echo seven >> notes.txt && git commit -qam 'main 3'"
  settle 4
  pointer_to "$CHIP_X" "$SECOND_ROW_Y"
  settle 2
  shot 560-05-unsupported
  echo "main is at $(repo_git rev-parse --short main)"

  echo "== what the chip's git ran"
  awk '{ for (i = 1; i <= NF; i++) if ($i == "--no-pager") { print $(i + 1); break } }' \
    "$E2E_WORK/git.log" | sort | uniq -c
  # The project row's changed lines (#531) add `rev-parse` and `diff --numstat`, read-only too.
  expect "the rail's git ran only config, merge-base, rev-list, merge-tree, rev-parse and diff" \
    bash -c "awk '{ for (i = 1; i <= NF; i++) if (\$i == \"--no-pager\") { print \$(i + 1); break } }' '$E2E_WORK/git.log' | grep -vxE 'config|merge-base|rev-list|merge-tree|rev-parse|diff' | wc -l | grep -qx 0"
  expect "no fetch" bash -c "! grep -q fetch '$E2E_WORK/git.log'"
  grep -n "merge-tree --write-tree" "$E2E_PROFILE/logs/Marley.log" || echo "no log line"
  expect "one log line for the repository" \
    test "$(grep -c "refuses merge-tree --write-tree" "$E2E_PROFILE/logs/Marley.log")" -eq 1
}
