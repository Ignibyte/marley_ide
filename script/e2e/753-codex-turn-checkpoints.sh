# shellcheck shell=bash
# #753's visual check: per-turn diffs for Codex on its App Server. A scratch repository, committed
# with `src/lib.txt` and `src/util.txt`, holds a staged change of the user's to `src/util.txt`,
# under a git config of the scenario's own. #650's stand-in Codex on its App Server
# (`codex-fixture.sh`, `marley.codex_app_server` on), cued through its `tui.fifo`, runs four
# turns: "Add a README" writes `README.md` and runs `sed -i` on `src/lib.txt`, then `done`;
# "Explain the build" changes nothing, then `done`; "Fix the typo in util" edits `src/util.txt`,
# then `interrupt`; "Refactor the parser" edits `src/util.txt`, then `fail`. The Codex row's
# "Turns (3)" opens to the three that changed the tree, newest first, the failed one marked
# (REQ-001, REQ-004, REQ-005); a click opens a turn in the commit view with its changes alone
# (REQ-002, REQ-003); the refs are under the thread's id, each on a checkpoint, and the user's
# index and HEAD are as they were (REQ-006).
compositor sway
# shellcheck source=script/e2e/codex-fixture.sh
. script/e2e/codex-fixture.sh

# The turns' line and rows, as the first run's shots found them; `STOP_AT=<shot>` ends the run
# after that shot.
TURNS_X=${TURNS_X:-60}
TURNS_Y=${TURNS_Y:-268}
FIRST_TURN_X=${FIRST_TURN_X:-120}
FIRST_TURN_Y=${FIRST_TURN_Y:-340}
SECOND_TURN_X=${SECOND_TURN_X:-120}
SECOND_TURN_Y=${SECOND_TURN_Y:-316}
STOP_AT=${STOP_AT:-}

repo_git() {
  git -C "$E2E_WORK/repo" "$@"
}

# The user's side of the repository: what no turn may change.
user_state() {
  echo "HEAD $(repo_git rev-parse HEAD)"
  echo "index:"
  repo_git diff --cached
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin versions=$E2E_WORK/versions repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$versions" "$repo/src"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  # A git config of the scenario's own, which Marley's git and Zed's inherit.
  cat >"$E2E_WORK/gitconfig" <<'CONFIG'
[user]
	name = Scenario
	email = scenario@example.invalid
CONFIG
  export GIT_CONFIG_GLOBAL=$E2E_WORK/gitconfig GIT_CONFIG_NOSYSTEM=1
  git init -q -b turns "$repo"
  printf 'alpha\nbeta\n' >"$repo/src/lib.txt"
  printf 'the utl module\n' >"$repo/src/util.txt"
  repo_git add -A
  repo_git commit -q -m "Start"
  # The user's own change, staged before the session.
  printf 'the utl module\nstaged by the user\n' >"$repo/src/util.txt"
  repo_git add src/util.txt
  echo "== before the session"
  user_state | tee "$E2E_WORK/before.txt"
  write_codex_stand_in "$versions/0.155.1"
  ln -sfn "$versions/0.155.1" "$bin/codex"
  export MARLEY_CODEX=$bin/codex
  profile_setting marley.codex_app_server true
  mkfifo "$E2E_WORK/tui.fifo"
  open_path "$repo"
}

# Writes a cue to the stand-in Codex's TUI.
tui() {
  echo "$1" >"$E2E_WORK/tui.fifo"
  settle "${2:-2}"
}

# A turn: its prompt, then each edit after the start's checkpoint, then its end.
turn() {
  local prompt=$1 end=$2
  shift 2
  tui "prompt $prompt" 3
  local edit
  for edit in "$@"; do
    tui "$edit" 1
  done
  tui "$end" 4
}

# The thread's turn refs, oldest first.
thread_refs() {
  repo_git for-each-ref --sort=refname --format='%(refname)' refs/marley/turns/
}

# The files turn ref `$1` changed against its parent.
turn_files() {
  repo_git diff --name-only "$1^" "$1" | sort | tr '\n' ' ' | sed 's/ $//'
}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== Codex on its App Server"
  press "CTRL ALT" n
  settle 2
  type_text "Codex"
  settle 1
  press "" Return
  settle 2
  # New Agent asks where Codex starts; the project comes first.
  press "" Return
  settle 10
  echo "== four turns"
  turn "Add a README" "done" "write README.md # repo" "sh sed -i s/beta/gamma/ src/lib.txt"
  turn "Explain the build" "done"
  turn "Fix the typo in util" "interrupt" "write src/util.txt the util module"
  turn "Refactor the parser" "fail" "write src/util.txt the util module, refactored"
  settle 3
  take 753-00-row || return 0
  echo "== the row's turns, opened"
  click "$TURNS_X" "$TURNS_Y"
  settle 2
  take 753-01-turns || return 0

  echo "== the refs"
  repo_git for-each-ref --format='%(refname) %(objectname)' refs/marley/turns/ | tee "$E2E_WORK/refs.txt"
  expect "the thread pinned three turns, under its id" test "$(thread_refs | wc -l)" -eq 3
  local ref
  for ref in $(thread_refs); do
    echo "$ref: $(repo_git log -1 --format='%an <%ae> | %s' "$ref") | parent: $(repo_git log -1 --format=%s "$ref^") | $(turn_files "$ref")"
  done | tee "$E2E_WORK/turns.txt"
  # A checkpoint is Zed's worktree archive commit since #950, titled `WIP unstaged`.
  expect "each turn is Marley's, on a checkpoint" \
    test "$(grep -c ': Marley <marley@localhost> | .* | parent: WIP unstaged |' "$E2E_WORK/turns.txt")" -eq 3
  expect "turn 1 holds the README and the sed" \
    bash -c "grep -q '/1: Marley <marley@localhost> | Add a README | parent: WIP unstaged | README.md src/lib.txt$' '$E2E_WORK/turns.txt'"
  expect "turn 2 is the interrupted fix, alone" \
    bash -c "grep -q '/2: Marley <marley@localhost> | Fix the typo in util | parent: WIP unstaged | src/util.txt$' '$E2E_WORK/turns.txt'"
  expect "turn 3 is the failed refactor" \
    bash -c "grep -q '/3: Marley <marley@localhost> | Refactor the parser | parent: WIP unstaged | src/util.txt$' '$E2E_WORK/turns.txt'"

  echo "== the user's index and HEAD, untouched"
  user_state | tee "$E2E_WORK/after.txt"
  expect "HEAD and the index are as the user left them" cmp -s "$E2E_WORK/before.txt" "$E2E_WORK/after.txt"

  echo "== the first turn, in the commit view"
  click "$FIRST_TURN_X" "$FIRST_TURN_Y"
  settle 4
  take 753-02-first-turn || return 0

  echo "== the interrupted turn, in the commit view"
  click "$SECOND_TURN_X" "$SECOND_TURN_Y"
  settle 4
  take 753-03-interrupted || return 0
}
