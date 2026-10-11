# shellcheck shell=bash
# #764's visual check: a restored terminal's earlier commands come back as blocks over their dimmed
# output. A scratch repository; a HOME whose `.bashrc` sets a plain prompt.
# - The first terminal: `echo one`, `ls missing-file` (exit 2), `echo two` (`before`); a second
#   terminal runs `sleep 600` at the quit.
# - A quit and a launch: the second terminal shows `sleep 600`'s block with no exit pill
#   (`running`, REQ-008). The first shows three blocks over the dimmed output, bars and pills,
#   headers naming the folder and `restored`, and its rail row no command line (`restored`,
#   REQ-001, REQ-002, REQ-005); the pointer over the failed one shows no Rerun and no "Ask the
#   agent" (`hover`, REQ-003); `echo fresh` makes a live block with Rerun and the rail row reads
#   it (`fresh`, REQ-005, REQ-007).
# - `terminal_blocks` marks the restored blocks `restored` and `echo fresh` not (REQ-006); the
#   kept files hold the command marks and no nonce (REQ-009).
compositor sway

# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

# Places in the window, as the first run's shots found them; `STOP_AT=<shot>` ends the run after
# that shot.
FAILED_X=${FAILED_X:-700}
FAILED_Y=${FAILED_Y:-800}
FRESH_X=${FRESH_X:-700}
FRESH_Y=${FRESH_Y:-880}
STOP_AT=${STOP_AT:-}

setup() {
  local home=$E2E_WORK/home repo=$E2E_WORK/repo
  mkdir -p "$home" "$repo"
  printf 'PS1=%q\n' '$ ' >"$home/.bashrc"
  terminal_env HOME "$home"
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  open_path "$repo"
}

history_dir() { echo "$E2E_PROFILE/terminal_history"; }

# Whether the kept files hold at least four command marks (three and `sleep 600`) and no nonce.
marks_kept() {
  local marks nonces
  marks=$(cat "$(history_dir)"/*.log | grep -ao 'mark;kind=command' | wc -l)
  nonces=$(cat "$(history_dir)"/*.log | grep -a -c -i 'nonce' || true)
  echo "command marks $marks, nonces $nonces"
  ((marks >= 4 && nonces == 0))
}

# Whether `terminal_blocks` marks the three earlier blocks restored and `echo fresh` not.
restored_listed() {
  local listed
  listed=$(mcp_agent blocks)
  echo "$listed"
  grep -q "'echo one', .*restored True" <<<"$listed" &&
    grep -q "'ls missing-file', exit 2, .*restored True" <<<"$listed" &&
    grep -q "'echo two', .*restored True" <<<"$listed" &&
    grep -q "'echo fresh', .*restored False" <<<"$listed"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
  settle "${2:-2}"
}

cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

# Shoots `$1`, and ends the run there when `STOP_AT` names it.
take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3
  echo "== the first terminal"
  cue "echo one"
  cue "ls missing-file"
  cue "echo two"
  take 764-01-before || return 0
  echo "== the second terminal, running at the quit"
  palette "workspace: new terminal" 4
  cue "sleep 600" 3
  expect "the kept files hold the marks and no nonce" marks_kept
  echo "== a quit and a launch"
  quit_marley
  launch_marley
  settle 16
  take 764-05-running || return 0
  palette "pane: activate previous item"
  take 764-02-restored || return 0
  echo "== the failed restored block"
  pointer_to "$FAILED_X" "$FAILED_Y"
  settle 1
  take 764-03-hover || return 0
  echo "== a new command"
  cue "echo fresh"
  pointer_to "$FRESH_X" "$FRESH_Y"
  settle 1
  take 764-04-fresh || return 0
  expect "terminal_blocks marks the restored blocks" restored_listed
}
