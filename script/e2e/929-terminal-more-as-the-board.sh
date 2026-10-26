# shellcheck shell=bash
# #929's visual check: a table tab's header, the restart divider and the microphone's words, as
# the board draws them. A scratch repository; `marley.block_tables` offers a table; a fake Voxtype
# first on Marley's PATH (#480's) with `marley.voice.enabled`; a stand-in Claude Code first on the
# terminals' PATH through the `.bashrc`. The dropped link has its own scenario,
# `929-dropped-link.sh`, and the screen-read row is #640's scenario's `640-01-rows`. No agent CLI
# runs.
# - `929-01-table`: a CSV block's Table chip clicked: the tab's header, "csv", the name, "3 rows ·
#   from printf …" (REQ-001).
# - `929-02-restored`: two commands run, Marley quit and started again: the restored blocks, then
#   "Marley started again · HH:MM" between two rules over the live prompt (REQ-002).
# - `929-05-mic`: dictation toggled on the stand-in Claude Code's terminal: "Listening · Voxtype"
#   (REQ-004).
# - `929-07-day`: the divider and the table header in Marley Day (REQ-001, REQ-002).
compositor sway

STOP_AT=${STOP_AT:-}
AWAY_X=${AWAY_X:-1300}
AWAY_Y=${AWAY_Y:-300}
# From the probe run's shot: the CSV block's Table chip.
TABLE_X=${TABLE_X:-1470}
TABLE_Y=${TABLE_Y:-722}

take() {
  shot "$1"
  [[ $STOP_AT != "$1" ]]
}

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin repo=$E2E_WORK/repo
  mkdir -p "$home" "$bin" "$repo"
  cat >"$home/.bashrc" <<RC
PS1='\$ '
export PATH="$bin:\$PATH"
RC
  terminal_env HOME "$home"
  printf '%s\n' '#!/bin/bash' "exec -a claude bash -c 'while :; do printf .; sleep 1; done'" >"$bin/claude"
  cat >"$bin/voxtype" <<'FAKE'
#!/bin/sh
dir=$(dirname "$0")
state() { printf '{"text": "", "alt": "%s", "class": "%s", "tooltip": ""}\n' "$1" "$1" >> "$dir/status"; }
case "$1" in
status) exec tail -n +1 -f "$dir/status" ;;
record)
  if tail -n 1 "$dir/status" | grep -q '"class": "recording"'; then
    state transcribing
    (sleep 6; state idle) &
  else
    state recording
  fi ;;
esac
FAKE
  chmod +x "$bin/claude" "$bin/voxtype"
  printf '{"text": "", "alt": "idle", "class": "idle", "tooltip": ""}\n' >"$bin/status"
  PATH=$bin:$PATH
  printf '# repo\n' >"$repo/README.md"
  git init -q -b main "$repo"
  profile_setting marley.tmux_hosts '[]'
  profile_setting marley.block_tables '"offer"'
  profile_setting marley.voice.enabled true
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

cue() {
  type_text "$1"
  press "" Return
  settle "${2:-2}"
}

away() {
  pointer_to "$AWAY_X" "$AWAY_Y"
  settle 1
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 3

  echo "== a CSV block and its table"
  cue "printf 'crate,tests\\nrail,212\\nterminal,148\\nthreads,96\\n'" 3
  away
  take 929-00-chip || return 0
  click "$TABLE_X" "$TABLE_Y"
  settle 4
  away
  take 929-01-table || return 0
  palette "pane: activate previous item" 2

  echo "== two commands, a quit and a launch"
  cue "echo one"
  cue "echo two"
  quit_marley
  launch_marley
  settle 16
  away
  take 929-02-restored || return 0

  echo "== Marley Day"
  profile_setting theme '"Marley Day"'
  settle 4
  away
  shot 929-07-day
  profile_setting theme '"Marley Night"'
  settle 3

  echo "== the stand-in Claude Code, dictation on"
  palette "workspace: new terminal" 3
  cue "claude" 3
  palette "marley: toggle dictation" 3
  away
  take 929-05-mic || return 0
  palette "marley: toggle dictation" 2
}
