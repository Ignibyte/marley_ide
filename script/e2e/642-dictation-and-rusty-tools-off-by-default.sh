# shellcheck shell=bash
# #642's visual check: dictation and Rusty's tools wait to be turned on. A stand-in Claude Code in
# the terminal (480's), a fake Voxtype first on the PATH that logs each call (480's, plus the log),
# and `marley_rusty`'s stand-in `rusty-mcp` (#643), named by `MARLEY_RUSTY_MCP`. The run starts on
# the shipped defaults: setup checks that the harness turned both switches off in its copy of the
# settings, then takes both out.
#
# Off: no microphone and no `voxtype` started (`642-01-no-microphone`), the action's toast
# (`642-02-dictation-is-off`), the Marley page's Voice and Rusty Tools for Agents off (`642-03`,
# `642-04`), no `rusty` server (`642-05-no-rusty`). Then each switch is turned on and off by
# editing the run's settings from outside while Marley runs, never by Marley (L-607): `rusty`
# offered (`642-06-rusty-on`), the microphone shown and following Voxtype (`642-07`), a dictation
# (`642-08-recording`), and the microphone gone with its status process (`642-09`).
compositor sway

# Whether the run's copy of the settings holds the JSON value `$2` at the dotted key path `$1`.
setting_is() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" "$2" <<'SETTINGS'
import json, pathlib, sys

node = json.loads(pathlib.Path(sys.argv[1]).read_text())
for key in sys.argv[2].split("."):
    node = node.get(key) if isinstance(node, dict) else None
sys.exit(0 if node == json.loads(sys.argv[3]) else 1)
SETTINGS
}

# Takes the dotted key path `$1` out of the run's copy of the settings, where it is set.
drop_setting() {
  python3 - "$E2E_PROFILE/config/settings.json" "$1" <<'SETTINGS'
import json, pathlib, sys

path = pathlib.Path(sys.argv[1])
settings = json.loads(path.read_text())
*parents, last = sys.argv[2].split(".")
node = settings
for key in parents:
    node = node.get(key) if isinstance(node, dict) else None
if isinstance(node, dict):
    node.pop(last, None)
path.write_text(json.dumps(settings, indent=2) + "\n")
SETTINGS
}

# Whether the fake Voxtype has been called with nothing.
no_calls() { [[ ! -s $E2E_WORK/bin/calls ]]; }

# Whether the fake Voxtype's `status --follow` runs: the `tail` it becomes. pgrep reads a pattern,
# so the `+` sits in a bracket.
status_runs() { pgrep -f "tail -n [+]1 -f $E2E_WORK/bin/status" >/dev/null; }

no_status() { ! status_runs; }

setup() {
  local home=$E2E_WORK/home bin=$E2E_WORK/bin
  mkdir -p "$home" "$bin" "$E2E_WORK/repo"
  # The terminal's shell becomes `claude` by name and prints a dot a second, so the terminal
  # keeps reading its foreground process.
  printf '%s\n' "exec -a claude bash -c 'while :; do printf .; sleep 1; done'" >"$home/.bashrc"
  terminal_env HOME "$home"
  cat >"$bin/voxtype" <<'FAKE'
#!/bin/sh
dir=$(dirname "$0")
echo "$*" >> "$dir/calls"
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
  chmod +x "$bin/voxtype"
  printf '{"text": "", "alt": "idle", "class": "idle", "tooltip": ""}\n' >"$bin/status"
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  export PATH="$bin:$PATH"
  expect "the harness's copy turns Rusty off" setting_is marley.rusty.enabled false
  expect "the harness's copy turns Voice off" setting_is marley.voice.enabled false
  drop_setting marley.rusty
  drop_setting marley.voice.enabled
  drop_setting context_servers.rusty
  # A key for the Settings window's MCP Servers page, which its search does not list.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-m": ["zed::OpenSettingsAt", {"path": "context_servers"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  git init -q -b voice "$E2E_WORK/repo"
  open_path "$E2E_WORK/repo"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

close_settings() {
  sway_msg '[title="Settings"] kill' >/dev/null
  settle 2
}

steps() {
  settle 14
  # Trusts the scratch repository.
  press "" Return
  settle 5
  shot 642-01-no-microphone
  expect "no voxtype started with Voice unset" no_calls

  echo "== marley: toggle dictation, Voice off"
  palette "marley: toggle dictation"
  settle 2
  shot 642-02-dictation-is-off
  expect "the action started no voxtype" no_calls

  echo "== the Marley page"
  palette "marley: open settings"
  settle 4
  # The window opens with its page list focused; Ctrl+F puts the keys in its search.
  press "CTRL" f
  settle 1
  type_text "Voxtype"
  settle 2
  shot 642-03-voice-toggle-off
  press "CTRL" a
  type_text "Rusty"
  settle 2
  shot 642-04-rusty-toggle-off
  close_settings

  echo "== the MCP Servers page, Rusty's tools unset"
  press "CTRL ALT SHIFT" m
  settle 4
  shot 642-05-no-rusty

  echo "== Rusty's tools turned on"
  profile_setting marley.rusty '{"enabled": true, "agent_tools": true}'
  settle 6
  shot 642-06-rusty-on
  close_settings

  echo "== Voice turned on"
  profile_setting marley.voice.enabled true
  settle 4
  shot 642-07-microphone-on
  expect "the microphone follows Voxtype's status" holds "$E2E_WORK/bin/calls" "status --follow"
  expect "the status process runs" status_runs

  echo "== marley: toggle dictation, Voice on"
  palette "marley: toggle dictation"
  settle 3
  shot 642-08-recording
  expect "the action ran voxtype record toggle" holds "$E2E_WORK/bin/calls" "record toggle"
  palette "marley: toggle dictation"
  settle 9

  echo "== Voice turned off"
  profile_setting marley.voice.enabled false
  settle 4
  shot 642-09-microphone-off
  expect "the status process ended" no_status
  cat "$E2E_WORK/bin/calls"
}
