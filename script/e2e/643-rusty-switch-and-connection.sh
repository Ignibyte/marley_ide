# shellcheck shell=bash
# #643's visual check: Marley connects to Rusty when Rusty is turned on. `marley_rusty`'s stand-in
# `rusty-mcp`, named by `MARLEY_RUSTY_MCP`, over a scratch state folder (`RUSTY_STAND_IN_STATE`),
# never the user's Rusty (R-D8); it logs every request with its pid in `calls`. The run starts on
# the shipped defaults: setup checks the harness's copy, then takes `marley.rusty` out. Every change
# to Marley's settings is an edit of the run's file from outside (L-607); the dropdown writes to the
# stand-in, never to `settings.json`.
#
# Rusty off (`643-01-off`); the old `marley.rusty_tools: true` carried into the Rusty section
# (`643-02-carried`); connected to the embedded stand-in with Rusty's provider read
# (`643-03-connected`); the provider written from the dropdown (`643-04-written`); a change the
# stand-in announces read again (`643-05-refreshed`); Rusty's tools offered to Zed's agents
# (`643-06-agent-tools`); a killed stand-in started again (`643-07-restarted`); a missing one named
# (`643-08-missing`); Rusty turned off and on while Marley runs (`643-09-off-live`,
# `643-10-no-agent-tools`, `643-11-on-again`); the service connection (`643-12-service`).
compositor sway

# Where the Rusty's Server page's provider dropdown and its Off entry sit, from the first runs'
# shots; 0 skips the click (for the dropdown) or shoots the open menu instead (for Off).
DROPDOWN_X=${DROPDOWN_X:-1525}
DROPDOWN_Y=${DROPDOWN_Y:-402}
OFF_X=${OFF_X:-1410}
OFF_Y=${OFF_Y:-501}

HTTP_PID=""

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

calls() { cat "$E2E_WORK/rusty/calls" 2>/dev/null; }

# The pids the stand-in logged, but `$1`.
logged_pids() { calls | awk '{print $1}' | sort -u | grep -vx "${1:-0}"; }

# Whether no pid the stand-in logged, but `$1`, is alive.
none_alive() {
  local pid pids
  # The run is under `set -u`: an empty argument, not a missing one, so the list is never empty
  # by an error.
  pids=$(logged_pids "${1:-0}") || return 1
  [[ -n $pids ]] || return 1
  for pid in $pids; do
    if kill -0 "$pid" 2>/dev/null; then
      echo "still alive: $pid"
      return 1
    fi
  done
}

# The pid of the stand-in Marley itself connects to: the one that read the settings last.
marleys_pid() { calls | grep ' tools/call settings_list ' | tail -1 | awk '{print $1}'; }

# Writes `$1` as the stand-in's stored embedding provider, from outside, as another process would.
store_provider() {
  python3 - "$E2E_WORK/rusty/settings.json" "$1" <<'PY'
import json, os, sys

path, value = sys.argv[1], sys.argv[2]
settings = json.load(open(path))
settings["embedding_provider"] = value
with open(path + ".new", "w") as out:
    json.dump(settings, out, indent=2, sort_keys=True)
os.replace(path + ".new", path)
PY
}

setup() {
  local bin=$E2E_WORK/bin
  mkdir -p "$bin" "$E2E_WORK/repo" "$E2E_WORK/home"
  printf 'PS1=%q\n' '$ ' >"$E2E_WORK/home/.bashrc"
  terminal_env HOME "$E2E_WORK/home"
  expect "the harness names no rusty-mcp" test ! -e "$MARLEY_RUSTY_MCP"
  expect "the harness's copy turns Rusty off" setting_is marley.rusty.enabled false
  expect "the harness's copy points Rusty's service at nothing" \
    setting_is marley.rusty.service_url '"http://127.0.0.1:9/mcp"'
  expect "the harness's copy has no marley.rusty_tools" setting_is marley.rusty_tools null
  drop_setting marley.rusty
  ln -s "$PWD/crates/marley_rusty/stand_in/rusty-mcp" "$bin/rusty-mcp"
  export MARLEY_RUSTY_MCP=$bin/rusty-mcp RUSTY_STAND_IN_STATE=$E2E_WORK/rusty
  # Keys for the Rusty's Server page and the MCP Servers page, which the search does not list.
  printf '%s\n' '[{"bindings": {"ctrl-alt-shift-r": ["zed::OpenSettingsAt", {"path": "marley.rusty.server"}], "ctrl-alt-shift-m": ["zed::OpenSettingsAt", {"path": "context_servers"}]}}]' \
    >"$E2E_PROFILE/config/keymap.json"
  printf '# repo\n' >"$E2E_WORK/repo/README.md"
  open_path "$E2E_WORK/repo"
}

teardown() {
  if [[ -n $HTTP_PID ]]; then
    kill "$HTTP_PID" 2>/dev/null
  fi
  return 0
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

server_page() {
  press "CTRL ALT SHIFT" r
  settle 4
}

steps() {
  local pid before
  settle 14
  # Trusts the scratch project.
  press "" Return
  settle 4

  echo "== Rusty off"
  server_page
  shot 643-01-off
  expect "no rusty-mcp ran" test ! -e "$E2E_WORK/rusty/calls"
  close_settings

  echo "== the old key carried"
  profile_setting marley.rusty_tools true
  settle 6
  palette "marley: open settings"
  settle 4
  press "CTRL" f
  settle 1
  type_text "Rusty"
  settle 2
  shot 643-02-carried
  close_settings

  echo "== connected, the provider read"
  server_page
  shot 643-03-connected
  expect "Marley read Rusty's settings" bash -c "grep -q ' tools/call settings_list ' '$E2E_WORK/rusty/calls'"

  if ((DROPDOWN_X > 0)); then
    echo "== the provider written from the dropdown"
    click "$DROPDOWN_X" "$DROPDOWN_Y"
    settle 2
    if ((OFF_X > 0)); then
      click "$OFF_X" "$OFF_Y"
      settle 3
      shot 643-04-written
      expect "Marley wrote the provider with setting_set" \
        bash -c "grep -qF 'tools/call setting_set {\"key\": \"embedding_provider\", \"value\": \"off\"}' '$E2E_WORK/rusty/calls'"
      expect "and read the settings again" \
        bash -c "grep -A9 'tools/call setting_set' '$E2E_WORK/rusty/calls' | grep -q ' tools/call settings_list '"
    else
      shot 643-04a-menu
      press "" Escape
      settle 1
    fi
  fi

  echo "== a change the stand-in announces"
  store_provider openai
  settle 3
  shot 643-05-refreshed
  close_settings

  echo "== Rusty's tools for Zed's agents"
  press "CTRL ALT SHIFT" m
  settle 5
  shot 643-06-agent-tools
  close_settings

  echo "== Marley's stand-in killed"
  pid=$(marleys_pid)
  echo "Marley's stand-in: $pid"
  kill "$pid"
  settle 15
  server_page
  shot 643-07-restarted
  expect "a new stand-in read the settings" test "$(marleys_pid)" != "$pid"

  echo "== the stand-in gone"
  mv "$E2E_WORK/bin/rusty-mcp" "$E2E_WORK/bin/rusty-mcp.away"
  kill "$(marleys_pid)"
  settle 10
  shot 643-08-missing
  close_settings

  echo "== Rusty off while Marley runs"
  profile_setting marley.rusty.enabled false
  settle 5
  server_page
  shot 643-09-off-live
  expect "no stand-in is alive" none_alive
  close_settings
  press "CTRL ALT SHIFT" m
  settle 5
  shot 643-10-no-agent-tools
  close_settings

  echo "== Rusty on again"
  mv "$E2E_WORK/bin/rusty-mcp.away" "$E2E_WORK/bin/rusty-mcp"
  profile_setting marley.rusty.agent_tools false
  profile_setting marley.rusty.enabled true
  settle 6
  server_page
  shot 643-11-on-again
  close_settings

  echo "== the service"
  "$E2E_WORK/bin/rusty-mcp" --http 127.0.0.1:0 &
  HTTP_PID=$!
  for _ in $(seq 50); do
    [[ -s $E2E_WORK/rusty/http-addr ]] && break
    sleep 0.1
  done
  before=$(cat "$E2E_WORK/rusty/http-addr")
  echo "the service: $before (pid $HTTP_PID)"
  profile_setting marley.rusty.service_url "\"http://$before/mcp\""
  profile_setting marley.rusty.connection '"service"'
  settle 6
  server_page
  shot 643-12-service
  expect "no stdio stand-in is alive" none_alive "$HTTP_PID"
  expect "the service answered settings_list" \
    bash -c "grep -q '^$HTTP_PID tools/call settings_list ' '$E2E_WORK/rusty/calls'"
  calls | tail -6
}
