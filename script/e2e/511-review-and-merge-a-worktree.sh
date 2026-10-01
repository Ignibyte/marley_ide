# shellcheck shell=bash
# #511's e2e test: Review and Merge on a worktree's row. A scratch repository on `main`, pushed to
# a bare `origin`, with worktrees made the way #510 makes them, each on `agent/<name>` from `main`
# with its base in the config: `ok` with two commits of its own, and `clash` with a commit to
# README's first line, which a later commit on `main` changes too; and `manual`, made by hand with
# no base. `ok`'s menu offers Review and Merge 2 commits into main, and its row reads
# `2 ahead of main` (REQ-001, REQ-002); Review opens Changes since main in ok's workspace
# (REQ-003). With README changed in the main checkout, Merge refuses and changes nothing
# (REQ-007), as it does with the main checkout on another branch and with a change in the
# worktree (REQ-008); clean again, Merge asks, then makes a merge commit on main and pushes
# nothing, the toast names the commit, and the row loses its count and its chip (REQ-005,
# REQ-006, REQ-011). Merge on `clash` stops on README, is aborted and names it (REQ-009).
# `manual`'s menu has Review and No base recorded, and its Review compares with main, the default
# branch (REQ-004). With `marley.merge` set to `workflow`, clash's menu shows its state and leaves
# the merge to the workflow; the key unset, Merge is back; with a `workflow.toml` holding
# `[project]`, the workflow merges again (REQ-010). The git Marley runs is a wrapper, named by
# `MARLEY_GIT`, that logs its arguments: no push and no fetch.
compositor sway

# The rows, from the first run's shots: the main checkout's terminal, then the worktrees by name.
# A row's menu opens from its right edge, so the menu leaves the row's lines in view.
ROW_X=${ROW_X:-120}
MENU_X=${MENU_X:-248}
MAIN_ROW_Y=${MAIN_ROW_Y:-136}
CLASH_ROW_Y=${CLASH_ROW_Y:-180}
MANUAL_ROW_Y=${MANUAL_ROW_Y:-224}
OK_ROW_Y=${OK_ROW_Y:-268}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo real_git name
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

  git init -q --bare -b main "$E2E_WORK/origin.git"
  git init -q -b main "$repo"
  printf 'the first line\nthe second line\n' >"$repo/README"
  printf 'one\ntwo\nthree\nfour\nfive\n' >"$repo/notes.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  repo_git remote add origin "$E2E_WORK/origin.git"
  repo_git push -q origin main
  # As #510 makes a worktree agent's: a new branch from the base, tracking nothing, and the base
  # in the config.
  for name in ok clash; do
    repo_git worktree add -q --no-track -b "agent/$name" "$E2E_WORK/$name" main
    repo_git config "branch.agent/$name.base" main
  done
  sed -i 's/^one$/one, from ok/' "$E2E_WORK/ok/notes.txt"
  git -C "$E2E_WORK/ok" commit -q -am "ok's first change"
  printf 'from ok\n' >"$E2E_WORK/ok/ok.txt"
  git -C "$E2E_WORK/ok" add ok.txt
  git -C "$E2E_WORK/ok" commit -q -m "ok's second change"
  printf 'the first line, from clash\nthe second line\n' >"$E2E_WORK/clash/README"
  git -C "$E2E_WORK/clash" commit -q -am "clash's change"
  printf 'the first line, from main\nthe second line\n' >"$repo/README"
  repo_git commit -q -am "main's change"
  # A worktree made by hand: no base recorded.
  repo_git worktree add -q "$E2E_WORK/manual" -b manual
  printf 'from manual\n' >"$E2E_WORK/manual/manual.txt"
  git -C "$E2E_WORK/manual" add manual.txt
  git -C "$E2E_WORK/manual" commit -q -m "manual's change"
  repo_git worktree list

  real_git=$(type -P git)
  sed -e "s|@GIT@|$real_git|" -e "s|@LOG@|$E2E_WORK/git.log|" >"$E2E_WORK/marley-git" <<'WRAPPER'
#!/usr/bin/env bash
# Marley's git: logs each call's arguments.
printf '%q ' "$@" >>"@LOG@"
printf '\n' >>"@LOG@"
exec "@GIT@" "$@"
WRAPPER
  chmod +x "$E2E_WORK/marley-git"
  export MARLEY_GIT=$E2E_WORK/marley-git
  git -C "$repo" ls-remote origin main >"$E2E_WORK/remote-0.txt"
  open_path "$repo"
}

# Types `$1` into the main checkout's terminal and runs it.
in_main() {
  click "$ROW_X" "$MAIN_ROW_Y"
  settle 1
  type_text "$1"
  press "" Return
}

# Opens the menu of the worktree row at `$1`.
row_menu() {
  click "$MENU_X" "$1" right
  settle 1
}

# Chooses Merge on the row at `$1` and shoots the refusal as `$2`, checking that it changed
# nothing, then dismisses it.
refused() {
  main_state >"$E2E_WORK/$2-0.txt"
  merge "$1"
  settle 3
  shot "$2"
  main_state >"$E2E_WORK/$2-1.txt"
  expect "the refusal changed nothing" diff "$E2E_WORK/$2-0.txt" "$E2E_WORK/$2-1.txt"
  press "" Return
  settle 1
}

# Chooses Review from the menu of the worktree row at `$1`.
review() {
  row_menu "$1"
  press "" Home
  press "" Return
}

# Chooses Merge from the menu of the worktree row at `$1`, its second entry: Remove… comes last
# since #589.
merge() {
  row_menu "$1"
  press "" Home
  press "" Down
  press "" Return
}

# What a refusal must leave as it was: main's commit, the main checkout's status, no merge going.
main_state() {
  repo_git rev-parse main
  repo_git status --porcelain --untracked-files=no
  test -e "$E2E_WORK/repo/.git/MERGE_HEAD" && echo "a merge is going"
  true
}

steps() {
  local before
  settle 12
  # Trusts the scratch repository. Zed flips a repository's trust without an event, so the
  # rail's next rebuild, here the terminal's output, schedules the first run.
  press "" Return
  settle 2
  in_main "git log --oneline --all --graph"
  settle 6

  echo "== ok's menu: Review, and Merge 2 commits into main"
  row_menu "$OK_ROW_Y"
  shot 511-01-menu
  press "" Escape
  settle 1

  echo "== Review: Changes since main in ok's workspace"
  review "$OK_ROW_Y"
  settle 8
  shot 511-02-review

  echo "== README changed in the main checkout: Merge refuses"
  in_main "echo x >> README && git status -sb"
  settle 2
  refused "$OK_ROW_Y" 511-03-refused
  in_main "git checkout README && git status -sb"
  settle 2

  echo "== the main checkout on another branch: Merge refuses"
  in_main "git switch -q -c other && git status -sb"
  settle 2
  refused "$OK_ROW_Y" 511-03-off-base
  in_main "git switch -q main && git branch -q -D other && git status -sb"
  settle 2

  echo "== a change not committed in the worktree: Merge refuses"
  in_main "echo y >> ../ok/ok.txt && git -C ../ok status -sb"
  settle 2
  refused "$OK_ROW_Y" 511-03-worktree-dirty
  in_main "git -C ../ok checkout ok.txt && git -C ../ok status -sb"
  settle 2

  echo "== Merge ok: asked, then a merge commit on main, and nothing pushed"
  before=$(repo_git rev-parse main)
  merge "$OK_ROW_Y"
  settle 3
  shot 511-04-asked
  press "" Return
  settle 4
  in_main "git log --graph --oneline -6 && git status -sb"
  settle 5
  shot 511-04-merged
  repo_git log -1 --format='%H %P %s' main
  expect "main's tip is a merge of agent/ok on the old main" \
    test "$(repo_git log -1 --format=%P main)" = "$before $(repo_git rev-parse agent/ok)"
  expect "its title names the branch and the base" \
    test "$(repo_git log -1 --format=%s main)" = "Merge branch 'agent/ok' into main"
  git -C "$E2E_WORK/repo" ls-remote origin main >"$E2E_WORK/remote-1.txt"
  cat "$E2E_WORK/remote-0.txt" "$E2E_WORK/remote-1.txt"
  expect "origin's main is as it was" diff "$E2E_WORK/remote-0.txt" "$E2E_WORK/remote-1.txt"

  echo "== Merge clash: it stops on README, and is aborted"
  main_state | tee "$E2E_WORK/conflict-0.txt"
  merge "$CLASH_ROW_Y"
  settle 3
  press "" Return
  settle 4
  shot 511-05-conflict
  main_state | tee "$E2E_WORK/conflict-1.txt"
  expect "the aborted merge left the main checkout as it was" \
    diff "$E2E_WORK/conflict-0.txt" "$E2E_WORK/conflict-1.txt"
  press "" Return
  settle 1
  in_main "git status -sb && git log --oneline -1"
  settle 2
  shot 511-05-clean

  echo "== manual, no base recorded: Review against main, and no Merge"
  row_menu "$MANUAL_ROW_Y"
  shot 511-06-no-base
  press "" Escape
  settle 1
  review "$MANUAL_ROW_Y"
  settle 8
  shot 511-06-no-base-review

  echo "== marley.merge set to workflow: clash's state, and the workflow merges"
  in_main "git config marley.merge workflow && git commit -q --allow-empty -m tick"
  settle 4
  row_menu "$CLASH_ROW_Y"
  shot 511-07-workflow
  press "" Escape
  settle 1

  echo "== the key unset: Merge is back"
  in_main "git config --unset marley.merge && git commit -q --allow-empty -m tock"
  settle 4
  row_menu "$CLASH_ROW_Y"
  shot 511-07-unset
  press "" Escape
  settle 1

  echo "== a workflow.toml with [project]: the workflow merges"
  in_main "printf '[project]\\nslug = \"repo\"\\n' > workflow.toml && git commit -q --allow-empty -m tack"
  settle 4
  row_menu "$CLASH_ROW_Y"
  shot 511-08-workflow-toml
  press "" Escape
  settle 1

  echo "== what Marley's git ran"
  awk '{ for (i = 1; i <= NF; i++) if ($i == "--no-pager") { print $(i + 1); break } }' \
    "$E2E_WORK/git.log" | sort | uniq -c
  expect "no push" bash -c "! grep -qw push '$E2E_WORK/git.log'"
  expect "no fetch" bash -c "! grep -qw fetch '$E2E_WORK/git.log'"
  grep -F -- "merge --no-ff" "$E2E_WORK/git.log" || echo "no merge ran"
  expect "two merges ran, ok's and clash's" \
    test "$(grep -cF -- "merge --no-ff --no-edit" "$E2E_WORK/git.log")" -eq 2
  expect "clash's was aborted" grep -qF -- "merge --abort" "$E2E_WORK/git.log"
}
