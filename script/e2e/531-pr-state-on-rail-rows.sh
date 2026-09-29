# shellcheck shell=bash
# #531's visual check, after the fact: a project row's changed lines and its pull request. A
# scratch repository whose `origin` is a GitHub URL, on `feature` with a commit of its own and an
# edit not committed, and a stand-in `gh` first on Marley's PATH that answers `pr list` for
# `feature` with an open pull request whose base is `main`, and for any other branch with none.
# The row shows `+N −M` against main and the chip #42 (REQ-001, REQ-002); an edit moves the counts
# (REQ-004); a branch switch to one with no pull request takes the chip away (REQ-003, REQ-005);
# a project with nothing changed shows no counts.
compositor sway

MAIN_ROW_X=${MAIN_ROW_X:-120}
MAIN_ROW_Y=${MAIN_ROW_Y:-136}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  printf '%s\n' "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  cat >"$bin/gh" <<SH
#!/bin/sh
# The stand-in gh: an open pull request for feature, none for any other branch.
printf '%s\n' "\$*" >>"$E2E_WORK/gh.log"
case " \$* " in
  *" --head=feature "*)
    printf '%s\n' '[{"number":42,"state":"OPEN","isDraft":false,"title":"Add the greeting","url":"https://github.com/example/demo/pull/42","baseRefName":"main"}]' ;;
  *) printf '[]\n' ;;
esac
SH
  chmod +x "$bin/gh"
  # Named, not first on the PATH: Marley takes its PATH from the user's login shell.
  export MARLEY_GH=$bin/gh
  git init -q -b main "$repo"
  printf 'one\ntwo\nthree\n' >"$repo/notes.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  repo_git remote add origin https://github.com/example/demo.git
  repo_git checkout -q -b feature
  printf 'one\ntwo\nthree\nfour\nfive\n' >"$repo/notes.txt"
  repo_git commit -q -am "Two more lines"
  # An edit not committed: one line changed.
  printf 'one\nTWO\nthree\nfour\nfive\n' >"$repo/notes.txt"
  open_path "$repo"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 6
  shot 531-01-counts-and-chip
  cat "$E2E_WORK/gh.log" 2>/dev/null || echo "gh was not asked"
  expect "gh was asked about feature with the branch as one argument" \
    holds "$E2E_WORK/gh.log" "pr list --repo=example/demo --head=feature --state=all"
  echo "== an edit moves the counts"
  printf 'one\nTWO\nthree\nfour\nfive\nsix\nseven\n' >"$E2E_WORK/repo/notes.txt"
  settle 6
  shot 531-02-edited
  echo "== a branch with no pull request and nothing changed"
  repo_git stash -q
  repo_git checkout -q -b quiet main
  settle 8
  shot 531-03-no-pr
  expect "gh was asked about the new branch" holds "$E2E_WORK/gh.log" "--head=quiet"
}
